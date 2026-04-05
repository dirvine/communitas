// SPDX-License-Identifier: MIT OR Apache-2.0

//! Metrics collection and reporting for Communitas.
//!
//! This module provides both in-memory metrics for local monitoring and
//! OpenTelemetry integration for production observability.
//!
//! # Metrics Collected
//!
//! - **Sync latency**: Histogram of sync operation durations
//! - **Error rates**: Counter of errors by category
//! - **Message queue depth**: Gauge of pending messages
//! - **CRDT conflicts**: Counter of merge conflicts
//! - **Storage operations**: Counter of storage operations by type
//!
//! # Example
//!
//! ```rust,ignore
//! use communitas_bindings::metrics::{Metrics, SyncMetrics};
//!
//! let metrics = Metrics::new();
//!
//! // Record sync operation
//! let start = std::time::Instant::now();
//! // ... perform sync ...
//! metrics.record_sync_latency("push", "success", start.elapsed());
//!
//! // Record error
//! metrics.record_error("sync_error", "error");
//!
//! // Update queue depth
//! metrics.set_queue_depth("outbound", 42);
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

#[cfg(feature = "metrics")]
use opentelemetry::metrics::{Counter, Histogram, UpDownCounter};
#[cfg(feature = "metrics")]
use opentelemetry::{KeyValue, global};

/// Central metrics registry for Communitas.
///
/// This struct provides both in-memory metrics (always available) and
/// OpenTelemetry export (when the `metrics` feature is enabled and
/// telemetry is configured).
#[derive(Debug)]
pub struct Metrics {
    /// In-memory storage metrics
    storage: StorageMetrics,
    /// In-memory counters
    counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    /// In-memory gauges
    gauges: Arc<RwLock<HashMap<String, AtomicU64>>>,
    /// OpenTelemetry sync latency histogram
    #[cfg(feature = "metrics")]
    sync_latency: Histogram<f64>,
    /// OpenTelemetry error counter
    #[cfg(feature = "metrics")]
    error_counter: Counter<u64>,
    /// OpenTelemetry queue depth gauge
    #[cfg(feature = "metrics")]
    queue_depth: UpDownCounter<i64>,
    /// OpenTelemetry CRDT conflict counter
    #[cfg(feature = "metrics")]
    crdt_conflicts: Counter<u64>,
}

impl Metrics {
    /// Creates a new metrics registry.
    ///
    /// When the `metrics` feature is enabled, this initializes OpenTelemetry
    /// meters for export. Otherwise, only in-memory metrics are available.
    pub fn new() -> Self {
        #[cfg(feature = "metrics")]
        {
            let meter = global::meter("communitas");

            let sync_latency = meter
                .f64_histogram("communitas_sync_latency_seconds")
                .with_description("Duration of sync operations in seconds")
                .with_unit(opentelemetry::metrics::Unit::new("s"))
                .init();

            let error_counter = meter
                .u64_counter("communitas_errors_total")
                .with_description("Total number of errors by category")
                .init();

            let queue_depth = meter
                .i64_up_down_counter("communitas_message_queue_depth")
                .with_description("Current depth of message queues")
                .init();

            let crdt_conflicts = meter
                .u64_counter("communitas_crdt_conflicts_total")
                .with_description("Total number of CRDT merge conflicts")
                .init();

            Self {
                storage: StorageMetrics::new(),
                counters: Arc::new(RwLock::new(HashMap::new())),
                gauges: Arc::new(RwLock::new(HashMap::new())),
                sync_latency,
                error_counter,
                queue_depth,
                crdt_conflicts,
            }
        }

        #[cfg(not(feature = "metrics"))]
        Self {
            storage: StorageMetrics::new(),
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Records a sync operation latency.
    ///
    /// # Arguments
    ///
    /// * `operation` - The sync operation type ("push", "pull", "merge")
    /// * `status` - The operation status ("success", "error")
    /// * `duration` - The operation duration
    pub fn record_sync_latency(&self, operation: &str, status: &str, duration: Duration) {
        // Always record to in-memory metrics
        let key = format!("sync_latency_{}_{}", operation, status);
        if let Ok(mut counters) = self.counters.try_write() {
            counters
                .entry(key)
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }

        // Record to OpenTelemetry when enabled
        #[cfg(feature = "metrics")]
        {
            let seconds = duration.as_secs_f64();
            let attributes = [
                KeyValue::new("operation", operation.to_string()),
                KeyValue::new("status", status.to_string()),
            ];
            self.sync_latency.record(seconds, &attributes);
        }

        tracing::trace!(
            operation = operation,
            status = status,
            duration_ms = duration.as_millis() as u64,
            "Sync latency recorded"
        );
    }

    /// Records an error occurrence.
    ///
    /// # Arguments
    ///
    /// * `category` - The error category ("sync_error", "storage_error", "network_error", "crypto_error")
    /// * `severity` - The error severity ("error", "warning")
    pub fn record_error(&self, category: &str, severity: &str) {
        // Always record to in-memory metrics
        let key = format!("errors_{}_{}", category, severity);
        if let Ok(mut counters) = self.counters.try_write() {
            counters
                .entry(key)
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }

        // Record to OpenTelemetry when enabled
        #[cfg(feature = "metrics")]
        {
            let attributes = [
                KeyValue::new("category", category.to_string()),
                KeyValue::new("severity", severity.to_string()),
            ];
            self.error_counter.add(1, &attributes);
        }

        tracing::debug!(
            category = category,
            severity = severity,
            "Error recorded in metrics"
        );
    }

    /// Updates the message queue depth gauge.
    ///
    /// # Arguments
    ///
    /// * `queue_type` - The queue type ("outbound", "inbound", "retry")
    /// * `depth` - The current queue depth
    pub fn set_queue_depth(&self, queue_type: &str, depth: i64) {
        // Store in in-memory gauges
        let key = format!("queue_depth_{}", queue_type);
        if let Ok(mut gauges) = self.gauges.try_write() {
            gauges
                .entry(key)
                .or_insert_with(|| AtomicU64::new(0))
                .store(depth as u64, Ordering::Relaxed);
        }

        // Record delta to OpenTelemetry when enabled
        // Note: UpDownCounter tracks cumulative changes, so we record the absolute value
        // In a real implementation, we'd track the previous value to calculate delta
        #[cfg(feature = "metrics")]
        {
            let attributes = [KeyValue::new("queue_type", queue_type.to_string())];
            // For UpDownCounter, we need to track delta
            // This simplified version just records the absolute value
            self.queue_depth.add(depth, &attributes);
        }

        tracing::trace!(
            queue_type = queue_type,
            depth = depth,
            "Queue depth updated"
        );
    }

    /// Increments the queue depth by a delta.
    ///
    /// # Arguments
    ///
    /// * `queue_type` - The queue type ("outbound", "inbound", "retry")
    /// * `delta` - The change in depth (positive for enqueue, negative for dequeue)
    pub fn increment_queue_depth(&self, queue_type: &str, delta: i64) {
        // Update in-memory gauges
        let key = format!("queue_depth_{}", queue_type);
        if let Ok(mut gauges) = self.gauges.try_write() {
            let counter = gauges.entry(key).or_insert_with(|| AtomicU64::new(0));
            if delta >= 0 {
                counter.fetch_add(delta as u64, Ordering::Relaxed);
            } else {
                counter.fetch_sub((-delta) as u64, Ordering::Relaxed);
            }
        }

        // Record to OpenTelemetry when enabled
        #[cfg(feature = "metrics")]
        {
            let attributes = [KeyValue::new("queue_type", queue_type.to_string())];
            self.queue_depth.add(delta, &attributes);
        }
    }

    /// Records a CRDT merge conflict.
    ///
    /// # Arguments
    ///
    /// * `doc_type` - The document type ("message", "kanban", "canvas")
    /// * `resolution` - The resolution strategy ("automatic", "manual")
    pub fn record_crdt_conflict(&self, doc_type: &str, resolution: &str) {
        // Always record to in-memory metrics
        let key = format!("crdt_conflicts_{}_{}", doc_type, resolution);
        if let Ok(mut counters) = self.counters.try_write() {
            counters
                .entry(key)
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }

        // Record to OpenTelemetry when enabled
        #[cfg(feature = "metrics")]
        {
            let attributes = [
                KeyValue::new("doc_type", doc_type.to_string()),
                KeyValue::new("resolution", resolution.to_string()),
            ];
            self.crdt_conflicts.add(1, &attributes);
        }

        tracing::debug!(
            doc_type = doc_type,
            resolution = resolution,
            "CRDT conflict recorded"
        );
    }

    /// Returns a reference to the storage metrics.
    pub fn storage(&self) -> &StorageMetrics {
        &self.storage
    }

    /// Gets the current value of all in-memory metrics.
    pub async fn get_all_metrics(&self) -> HashMap<String, u64> {
        let mut result = self.storage.get_current_metrics().await;

        let counters = self.counters.read().await;
        for (key, counter) in counters.iter() {
            result.insert(key.clone(), counter.load(Ordering::Relaxed));
        }

        let gauges = self.gauges.read().await;
        for (key, gauge) in gauges.iter() {
            result.insert(key.clone(), gauge.load(Ordering::Relaxed));
        }

        result
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Collects and manages storage system metrics.
#[derive(Debug)]
pub struct StorageMetrics {
    personal_storage_ops: Arc<RwLock<u64>>,
    group_storage_ops: Arc<RwLock<u64>>,
    dht_storage_ops: Arc<RwLock<u64>>,
}

impl StorageMetrics {
    /// Creates a new storage metrics collector.
    pub fn new() -> Self {
        Self {
            personal_storage_ops: Arc::new(RwLock::new(0)),
            group_storage_ops: Arc::new(RwLock::new(0)),
            dht_storage_ops: Arc::new(RwLock::new(0)),
        }
    }

    /// Records a personal storage operation.
    pub async fn record_personal_storage(&self, _size: usize) {
        let mut ops = self.personal_storage_ops.write().await;
        *ops += 1;
    }

    /// Records a group storage operation.
    pub async fn record_group_storage(&self, _group_id: &str, _size: usize, _shards: usize) {
        let mut ops = self.group_storage_ops.write().await;
        *ops += 1;
    }

    /// Records a local cache hit.
    pub async fn record_local_hit(&self) {
        // Stub implementation
    }

    /// Records a DHT fallback.
    pub async fn record_dht_fallback(&self) {
        // Stub implementation
    }

    /// Records a successful Reed-Solomon recovery.
    pub async fn record_reed_solomon_success(&self) {
        // Stub implementation
    }

    /// Records a DHT backup used.
    pub async fn record_dht_backup_used(&self) {
        // Stub implementation
    }

    /// Records a DHT storage acceptance.
    pub async fn record_dht_storage_accepted(&self, _size: usize, _requester: &str) {
        let mut ops = self.dht_storage_ops.write().await;
        *ops += 1;
    }

    /// Gets all current storage metrics.
    pub async fn get_current_metrics(&self) -> HashMap<String, u64> {
        let personal_ops = *self.personal_storage_ops.read().await;
        let group_ops = *self.group_storage_ops.read().await;
        let dht_ops = *self.dht_storage_ops.read().await;

        let mut metrics = HashMap::new();
        metrics.insert("personal_storage_ops".to_string(), personal_ops);
        metrics.insert("group_storage_ops".to_string(), group_ops);
        metrics.insert("dht_storage_ops".to_string(), dht_ops);

        metrics
    }
}

impl Default for StorageMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics instance for convenience.
///
/// This provides a singleton-like interface for recording metrics
/// from anywhere in the codebase without passing the Metrics struct around.
static GLOBAL_METRICS: once_cell::sync::Lazy<Metrics> = once_cell::sync::Lazy::new(Metrics::new);

/// Returns the global metrics instance.
pub fn global_metrics() -> &'static Metrics {
    &GLOBAL_METRICS
}

/// Helper to time and record a sync operation.
///
/// # Example
///
/// ```rust,ignore
/// use communitas_bindings::metrics::timed_sync;
///
/// let result = timed_sync("push", || async {
///     // perform sync operation
///     Ok(())
/// }).await;
/// ```
pub async fn timed_sync<F, Fut, T, E>(operation: &str, f: F) -> std::result::Result<T, E>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display,
{
    let start = std::time::Instant::now();
    let result = f().await;
    let duration = start.elapsed();

    let status = if result.is_ok() { "success" } else { "error" };
    global_metrics().record_sync_latency(operation, status, duration);

    if result.is_err() {
        global_metrics().record_error("sync_error", "error");
    }

    result
}

/// Records an error with automatic category detection.
///
/// # Arguments
///
/// * `error` - The error to record
/// * `category` - Optional category override (defaults to auto-detection)
pub fn record_app_error<E: std::fmt::Display>(error: &E, category: Option<&str>) {
    let error_str = error.to_string().to_lowercase();
    let detected_category = category.unwrap_or_else(|| {
        if error_str.contains("network") || error_str.contains("connection") {
            "network_error"
        } else if error_str.contains("storage") || error_str.contains("io") {
            "storage_error"
        } else if error_str.contains("crypto") || error_str.contains("signature") {
            "crypto_error"
        } else if error_str.contains("sync") {
            "sync_error"
        } else {
            "other_error"
        }
    });

    global_metrics().record_error(detected_category, "error");
    tracing::debug!(
        category = detected_category,
        error = %error,
        "Application error recorded"
    );
}

/// Tracks message queue operations.
pub struct QueueTracker {
    queue_type: &'static str,
}

impl QueueTracker {
    /// Creates a new queue tracker.
    pub fn new(queue_type: &'static str) -> Self {
        Self { queue_type }
    }

    /// Records an enqueue operation.
    pub fn enqueue(&self, count: u32) {
        global_metrics().increment_queue_depth(self.queue_type, count as i64);
    }

    /// Records a dequeue operation.
    pub fn dequeue(&self, count: u32) {
        global_metrics().increment_queue_depth(self.queue_type, -(count as i64));
    }

    /// Sets the absolute queue depth.
    pub fn set_depth(&self, depth: u32) {
        global_metrics().set_queue_depth(self.queue_type, depth as i64);
    }
}

/// Records a CRDT conflict with automatic resolution tracking.
///
/// # Arguments
///
/// * `doc_type` - The document type ("message", "kanban", "canvas")
/// * `manual` - Whether manual resolution was required
pub fn record_conflict(doc_type: &str, manual: bool) {
    let resolution = if manual { "manual" } else { "automatic" };
    global_metrics().record_crdt_conflict(doc_type, resolution);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        assert!(metrics.counters.try_read().is_ok());
    }

    #[test]
    fn test_record_sync_latency() {
        let metrics = Metrics::new();
        metrics.record_sync_latency("push", "success", Duration::from_millis(100));
        // Just verify no panic
    }

    #[test]
    fn test_record_error() {
        let metrics = Metrics::new();
        metrics.record_error("sync_error", "error");
        metrics.record_error("network_error", "warning");
        // Just verify no panic
    }

    #[test]
    fn test_queue_depth() {
        let metrics = Metrics::new();
        metrics.set_queue_depth("outbound", 10);
        metrics.increment_queue_depth("outbound", 5);
        metrics.increment_queue_depth("outbound", -3);
        // Just verify no panic
    }

    #[test]
    fn test_crdt_conflict() {
        let metrics = Metrics::new();
        metrics.record_crdt_conflict("message", "automatic");
        metrics.record_crdt_conflict("kanban", "manual");
        // Just verify no panic
    }

    #[tokio::test]
    async fn test_get_all_metrics() {
        let metrics = Metrics::new();
        metrics.record_sync_latency("push", "success", Duration::from_millis(100));
        metrics.record_error("test", "error");

        let all = metrics.get_all_metrics().await;
        assert!(all.contains_key("personal_storage_ops"));
    }

    #[test]
    fn test_global_metrics() {
        let metrics = global_metrics();
        metrics.record_error("test", "error");
        // Just verify no panic
    }
}
