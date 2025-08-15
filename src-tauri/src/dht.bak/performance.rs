// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


//! Performance Monitoring and Optimization
//!
//! This module provides comprehensive performance monitoring with:
//! - Real-time metrics collection
//! - Performance analysis and optimization
//! - Mobile/battery-aware adaptations
//! - Resource usage tracking
//! - Automated performance tuning

use anyhow::{Result};
use serde::{Deserialize, Serialize};
use std::collections::{VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{debug, info, warn, error, instrument};

/// Performance metrics container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Operation latencies
    pub latencies: LatencyMetrics,
    /// Throughput metrics
    pub throughput: ThroughputMetrics,
    /// Resource usage
    pub resources: ResourceMetrics,
    /// Network performance
    pub network: NetworkMetrics,
    /// Cache performance
    pub cache: CacheMetrics,
    /// Battery/mobile metrics
    pub mobile: MobileMetrics,
    /// Error rates
    pub errors: ErrorMetrics,
    /// Timestamp of last update
    pub last_updated: SystemTime,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            latencies: LatencyMetrics::default(),
            throughput: ThroughputMetrics::default(),
            resources: ResourceMetrics::default(),
            network: NetworkMetrics::default(),
            cache: CacheMetrics::default(),
            mobile: MobileMetrics::default(),
            errors: ErrorMetrics::default(),
            last_updated: SystemTime::UNIX_EPOCH,
        }
    }
}

/// Latency-related metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencyMetrics {
    /// Average lookup latency
    pub avg_lookup_latency: Duration,
    /// 95th percentile lookup latency
    pub p95_lookup_latency: Duration,
    /// Average store latency
    pub avg_store_latency: Duration,
    /// 95th percentile store latency
    pub p95_store_latency: Duration,
    /// Average ping latency
    pub avg_ping_latency: Duration,
    /// Network round-trip time
    pub network_rtt: Duration,
}

/// Throughput metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Operations per second
    pub operations_per_second: f64,
    /// Lookups per second
    pub lookups_per_second: f64,
    /// Stores per second
    pub stores_per_second: f64,
    /// Bytes transferred per second
    pub bytes_per_second: f64,
    /// Peak operations per second
    pub peak_ops_per_second: f64,
}

/// Resource usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Peak memory usage in bytes
    pub peak_memory_usage: u64,
    /// CPU usage percentage (0.0 to 1.0)
    pub cpu_usage: f64,
    /// Disk usage in bytes
    pub disk_usage: u64,
    /// File descriptor count
    pub file_descriptors: u64,
    /// Thread count
    pub thread_count: u64,
}

/// Network performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Active connections
    pub active_connections: u64,
    /// Connection pool utilization (0.0 to 1.0)
    pub pool_utilization: f64,
    /// Bandwidth utilization (0.0 to 1.0)
    pub bandwidth_utilization: f64,
    /// Packet loss rate (0.0 to 1.0)
    pub packet_loss_rate: f64,
}

/// Cache performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Cache size in entries
    pub cache_size: u64,
    /// Cache memory usage
    pub cache_memory: u64,
    /// Eviction rate
    pub eviction_rate: f64,
}

/// Mobile/battery-specific metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MobileMetrics {
    /// Battery level (0.0 to 1.0)
    pub battery_level: f64,
    /// Power consumption estimate (watts)
    pub power_consumption: f64,
    /// Background mode status
    pub background_mode: bool,
    /// Network type (WiFi, Cellular, etc.)
    pub network_type: String,
    /// Signal strength (0.0 to 1.0)
    pub signal_strength: f64,
}

/// Error rate metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Overall error rate (0.0 to 1.0)
    pub overall_error_rate: f64,
    /// Timeout error rate
    pub timeout_error_rate: f64,
    /// Network error rate
    pub network_error_rate: f64,
    /// Protocol error rate
    pub protocol_error_rate: f64,
    /// Total errors in the last period
    pub total_errors: u64,
}

/// Performance sample for time-series analysis
#[derive(Debug, Clone)]
struct PerformanceSample {
    timestamp: Instant,
    operation_type: OperationType,
    latency: Duration,
    success: bool,
    bytes_transferred: u64,
}

/// Types of operations for performance tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OperationType {
    Lookup,
    Store,
    Ping,
    Connect,
    Bootstrap,
}

/// Performance analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// Overall performance score (0.0 to 1.0)
    pub overall_score: f64,
    /// Performance trend (improving, stable, degrading)
    pub trend: PerformanceTrend,
    /// Identified bottlenecks
    pub bottlenecks: Vec<Bottleneck>,
    /// Optimization recommendations
    pub recommendations: Vec<Recommendation>,
    /// Resource predictions
    pub predictions: ResourcePredictions,
}

/// Performance trend indicators
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
    #[default]
    Unknown,
}

/// Performance bottleneck identification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Bottleneck {
    pub component: String,
    pub severity: BottleneckSeverity,
    pub description: String,
    pub impact: f64, // Performance impact (0.0 to 1.0)
}

/// Bottleneck severity levels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

/// Performance optimization recommendations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: String,
    pub action: String,
    pub expected_improvement: f64,
    pub implementation_effort: ImplementationEffort,
}

/// Implementation effort levels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ImplementationEffort {
    #[default]
    Low,
    Medium,
    High,
}

/// Resource usage predictions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourcePredictions {
    pub memory_trend: f64, // Growth rate per hour
    pub cpu_trend: f64,
    pub disk_trend: f64,
    pub network_trend: f64,
    pub time_to_capacity: Option<Duration>, // When resources will be exhausted
}

/// Main performance monitor
pub struct PerformanceMonitor {
    /// Current metrics
    metrics: Arc<RwLock<Metrics>>,
    /// Performance samples for analysis
    samples: Arc<RwLock<VecDeque<PerformanceSample>>>,
    /// Configuration
    config: PerformanceConfig,
    /// Analysis results
    analysis: Arc<RwLock<Option<PerformanceAnalysis>>>,
    /// Mobile/battery state
    mobile_state: Arc<RwLock<MobileState>>,
    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// Performance monitoring configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// How often to update metrics
    pub update_interval: Duration,
    /// How often to run analysis
    pub analysis_interval: Duration,
    /// Sample retention period
    pub sample_retention: Duration,
    /// Maximum samples to keep
    pub max_samples: usize,
    /// Enable mobile optimizations
    pub mobile_optimizations: bool,
    /// Battery threshold for power saving
    pub battery_threshold: f64,
    /// Enable predictive scaling
    pub predictive_scaling: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(10),
            analysis_interval: Duration::from_secs(60),
            sample_retention: Duration::from_secs(3600), // 1 hour
            max_samples: 10000,
            mobile_optimizations: true,
            battery_threshold: 0.2, // 20%
            predictive_scaling: true,
        }
    }
}

/// Mobile device state
#[derive(Debug, Clone, Default)]
struct MobileState {
    battery_level: f64,
    is_charging: bool,
    background_mode: bool,
    network_type: String,
    low_power_mode: bool,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    #[instrument]
    pub fn new() -> Self {
        info!("Creating performance monitor");
        
        Self {
            metrics: Arc::new(RwLock::new(Metrics::default())),
            samples: Arc::new(RwLock::new(VecDeque::new())),
            config: PerformanceConfig::default(),
            analysis: Arc::new(RwLock::new(None)),
            mobile_state: Arc::new(RwLock::new(MobileState::default())),
            shutdown_tx: None,
        }
    }
    
    /// Start performance monitoring
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting performance monitoring");
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        
        // Start metrics update task
        let metrics = self.metrics.clone();
        let samples = self.samples.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(config.update_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::update_metrics_task(&metrics, &samples, &config).await {
                            error!("Metrics update task error: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Performance monitoring metrics task shutting down");
                        break;
                    }
                }
            }
        });
        
        // Start analysis task
        let metrics = self.metrics.clone();
        let samples = self.samples.clone();
        let analysis = self.analysis.clone();
        let config = self.config.clone();
        let (shutdown_tx2, mut shutdown_rx2) = mpsc::channel(1);
        
        tokio::spawn(async move {
            let mut interval = interval(config.analysis_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::analysis_task(&metrics, &samples, &analysis, &config).await {
                            error!("Performance analysis task error: {}", e);
                        }
                    }
                    _ = shutdown_rx2.recv() => {
                        info!("Performance analysis task shutting down");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop performance monitoring
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping performance monitoring");
        
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(()).await;
        }
        
        Ok(())
    }
    
    /// Record operation latency
    #[instrument(skip(self))]
    pub async fn record_lookup_latency(&self, latency: Duration) {
        self.record_operation(OperationType::Lookup, latency, true, 0).await;
    }
    
    /// Record store latency
    #[instrument(skip(self))]
    pub async fn record_store_latency(&self, latency: Duration) {
        self.record_operation(OperationType::Store, latency, true, 0).await;
    }
    
    /// Record ping latency
    #[instrument(skip(self))]
    pub async fn record_ping_latency(&self, latency: Duration) {
        self.record_operation(OperationType::Ping, latency, true, 0).await;
    }

    /// Record get latency
    #[instrument(skip(self))]
    pub async fn record_get_latency(&self, latency: Duration) {
        self.record_operation(OperationType::Lookup, latency, true, 0).await;
    }
    
    /// Record generic operation
    async fn record_operation(&self, op_type: OperationType, latency: Duration, success: bool, bytes: u64) {
        let sample = PerformanceSample {
            timestamp: Instant::now(),
            operation_type: op_type,
            latency,
            success,
            bytes_transferred: bytes,
        };
        
        let mut samples = self.samples.write().await;
        samples.push_back(sample);
        
        // Maintain sample limit
        while samples.len() > self.config.max_samples {
            samples.pop_front();
        }
        
        // Remove old samples
        let cutoff = Instant::now() - self.config.sample_retention;
        while let Some(front) = samples.front() {
            if front.timestamp < cutoff {
                samples.pop_front();
            } else {
                break;
            }
        }
    }
    
    /// Get current performance metrics
    pub async fn metrics(&self) -> Metrics {
        let mut metrics = self.metrics.read().await.clone();
        metrics.last_updated = SystemTime::now();
        metrics
    }
    
    /// Get performance analysis
    pub async fn analysis(&self) -> Option<PerformanceAnalysis> {
        self.analysis.read().await.clone()
    }
    
    /// Update mobile/battery state
    #[instrument(skip(self))]
    pub async fn update_mobile_state(&self, battery_level: f64, is_charging: bool, background_mode: bool) {
        let mut state = self.mobile_state.write().await;
        state.battery_level = battery_level;
        state.is_charging = is_charging;
        state.background_mode = background_mode;
        
        // Update mobile metrics
        let mut metrics = self.metrics.write().await;
        metrics.mobile.battery_level = battery_level;
        metrics.mobile.background_mode = background_mode;
        
        // Activate low power mode if battery is low
        state.low_power_mode = battery_level < self.config.battery_threshold && !is_charging;
        
        if state.low_power_mode {
            info!("Activating low power mode (battery: {:.1}%)", battery_level * 100.0);
        }
    }
    
    /// Check if in low power mode
    pub async fn is_low_power_mode(&self) -> bool {
        self.mobile_state.read().await.low_power_mode
    }
    
    /// Get optimization recommendations
    pub async fn get_recommendations(&self) -> Vec<Recommendation> {
        if let Some(analysis) = self.analysis().await {
            analysis.recommendations
        } else {
            Vec::new()
        }
    }
    
    /// Metrics update task
    async fn update_metrics_task(
        metrics: &Arc<RwLock<Metrics>>,
        samples: &Arc<RwLock<VecDeque<PerformanceSample>>>,
        config: &PerformanceConfig,
    ) -> Result<()> {
        let samples_guard = samples.read().await;
        let mut metrics_guard = metrics.write().await;
        
        let now = Instant::now();
        let window = Duration::from_secs(60); // 1-minute window
        let cutoff = now - window;
        
        // Filter recent samples
        let recent_samples: Vec<&PerformanceSample> = samples_guard
            .iter()
            .filter(|s| s.timestamp >= cutoff)
            .collect();
        
        if recent_samples.is_empty() {
            return Ok(());
        }
        
        // Calculate latency metrics
        let lookup_latencies: Vec<Duration> = recent_samples
            .iter()
            .filter(|s| s.operation_type == OperationType::Lookup)
            .map(|s| s.latency)
            .collect();
        
        if !lookup_latencies.is_empty() {
            let total: u64 = lookup_latencies.iter().map(|d| d.as_millis() as u64).sum();
            metrics_guard.latencies.avg_lookup_latency = Duration::from_millis(total / lookup_latencies.len() as u64);
            
            let mut sorted = lookup_latencies.clone();
            sorted.sort();
            let p95_index = (sorted.len() as f64 * 0.95) as usize;
            metrics_guard.latencies.p95_lookup_latency = sorted.get(p95_index).copied()
                .unwrap_or(Duration::from_millis(0));
        }
        
        let store_latencies: Vec<Duration> = recent_samples
            .iter()
            .filter(|s| s.operation_type == OperationType::Store)
            .map(|s| s.latency)
            .collect();
        
        if !store_latencies.is_empty() {
            let total: u64 = store_latencies.iter().map(|d| d.as_millis() as u64).sum();
            metrics_guard.latencies.avg_store_latency = Duration::from_millis(total / store_latencies.len() as u64);
            
            let mut sorted = store_latencies.clone();
            sorted.sort();
            let p95_index = (sorted.len() as f64 * 0.95) as usize;
            metrics_guard.latencies.p95_store_latency = sorted.get(p95_index).copied()
                .unwrap_or(Duration::from_millis(0));
        }
        
        // Calculate throughput metrics
        let successful_ops = recent_samples.iter().filter(|s| s.success).count();
        let window_secs = window.as_secs_f64();
        metrics_guard.throughput.operations_per_second = successful_ops as f64 / window_secs;
        
        let successful_lookups = recent_samples
            .iter()
            .filter(|s| s.success && s.operation_type == OperationType::Lookup)
            .count();
        metrics_guard.throughput.lookups_per_second = successful_lookups as f64 / window_secs;
        
        let successful_stores = recent_samples
            .iter()
            .filter(|s| s.success && s.operation_type == OperationType::Store)
            .count();
        metrics_guard.throughput.stores_per_second = successful_stores as f64 / window_secs;
        
        // Calculate error metrics
        let total_ops = recent_samples.len();
        let failed_ops = recent_samples.iter().filter(|s| !s.success).count();
        metrics_guard.errors.overall_error_rate = if total_ops > 0 {
            failed_ops as f64 / total_ops as f64
        } else {
            0.0
        };
        
        // Update timestamp
        metrics_guard.last_updated = SystemTime::now();
        
        debug!("Updated performance metrics: {} ops/sec, {:.1}% error rate", 
               metrics_guard.throughput.operations_per_second,
               metrics_guard.errors.overall_error_rate * 100.0);
        
        Ok(())
    }
    
    /// Performance analysis task
    async fn analysis_task(
        metrics: &Arc<RwLock<Metrics>>,
        samples: &Arc<RwLock<VecDeque<PerformanceSample>>>,
        analysis: &Arc<RwLock<Option<PerformanceAnalysis>>>,
        config: &PerformanceConfig,
    ) -> Result<()> {
        debug!("Running performance analysis");
        
        let current_metrics = metrics.read().await.clone();
        let samples_guard = samples.read().await;
        
        // Calculate overall performance score
        let mut score = 1.0;
        
        // Penalize high latencies
        if current_metrics.latencies.avg_lookup_latency > Duration::from_millis(1000) {
            score *= 0.5;
        } else if current_metrics.latencies.avg_lookup_latency > Duration::from_millis(500) {
            score *= 0.8;
        }
        
        // Penalize high error rates
        if current_metrics.errors.overall_error_rate > 0.1 {
            score *= 0.3;
        } else if current_metrics.errors.overall_error_rate > 0.05 {
            score *= 0.7;
        }
        
        // Penalize low throughput
        if current_metrics.throughput.operations_per_second < 1.0 {
            score *= 0.6;
        }
        
        // Identify bottlenecks
        let mut bottlenecks = Vec::new();
        
        if current_metrics.latencies.avg_lookup_latency > Duration::from_millis(500) {
            bottlenecks.push(Bottleneck {
                component: "Lookup Operations".to_string(),
                severity: BottleneckSeverity::High,
                description: "Lookup latency is high".to_string(),
                impact: 0.3,
            });
        }
        
        if current_metrics.errors.overall_error_rate > 0.05 {
            bottlenecks.push(Bottleneck {
                component: "Error Handling".to_string(),
                severity: BottleneckSeverity::Medium,
                description: "High error rate detected".to_string(),
                impact: 0.2,
            });
        }
        
        // Generate recommendations
        let mut recommendations = Vec::new();
        
        if current_metrics.latencies.avg_lookup_latency > Duration::from_millis(500) {
            recommendations.push(Recommendation {
                category: "Latency Optimization".to_string(),
                action: "Increase connection pool size".to_string(),
                expected_improvement: 0.2,
                implementation_effort: ImplementationEffort::Low,
            });
        }
        
        if current_metrics.throughput.operations_per_second < 10.0 {
            recommendations.push(Recommendation {
                category: "Throughput Optimization".to_string(),
                action: "Enable request batching".to_string(),
                expected_improvement: 0.4,
                implementation_effort: ImplementationEffort::Medium,
            });
        }
        
        // Determine trend (simplified)
        let trend = if score > 0.8 {
            PerformanceTrend::Stable
        } else if score < 0.5 {
            PerformanceTrend::Degrading
        } else {
            PerformanceTrend::Unknown
        };
        
        
        // Calculate lengths before moving vectors
        let bottleneck_count = bottlenecks.len();
        let recommendation_count = recommendations.len();
        // Create analysis result
        let new_analysis = PerformanceAnalysis {
            overall_score: score,
            trend,
            bottlenecks,
            recommendations,
            predictions: ResourcePredictions {
                memory_trend: 0.0,
                cpu_trend: 0.0,
                disk_trend: 0.0,
                network_trend: 0.0,
                time_to_capacity: None,
            },
        };
        
        *analysis.write().await = Some(new_analysis);
        
        
        info!("Performance analysis completed: score={:.2}, bottlenecks={}, recommendations={}", 
              score, bottleneck_count, recommendation_count);        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    
    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        let metrics = monitor.metrics().await;
        
        assert_eq!(metrics.throughput.operations_per_second, 0.0);
        assert_eq!(metrics.errors.overall_error_rate, 0.0);
    }
    
    #[tokio::test]
    async fn test_latency_recording() {
        let monitor = PerformanceMonitor::new();
        
        monitor.record_lookup_latency(Duration::from_millis(100)).await;
        monitor.record_store_latency(Duration::from_millis(50)).await;
        
        let samples = monitor.samples.read().await;
        assert_eq!(samples.len(), 2);
        
        let lookup_sample = samples.iter().find(|s| s.operation_type == OperationType::Lookup).unwrap();
        assert_eq!(lookup_sample.latency, Duration::from_millis(100));
        assert!(lookup_sample.success);
    }
    
    #[tokio::test]
    async fn test_mobile_state_updates() {
        let monitor = PerformanceMonitor::new();
        
        monitor.update_mobile_state(0.15, false, true).await;
        assert!(monitor.is_low_power_mode().await);
        
        monitor.update_mobile_state(0.8, true, false).await;
        assert!(!monitor.is_low_power_mode().await);
    }
    
    #[test]
    fn test_performance_sample() {
        let sample = PerformanceSample {
            timestamp: Instant::now(),
            operation_type: OperationType::Lookup,
            latency: Duration::from_millis(150),
            success: true,
            bytes_transferred: 1024,
        };
        
        assert_eq!(sample.operation_type, OperationType::Lookup);
        assert!(sample.success);
        assert_eq!(sample.bytes_transferred, 1024);
    }
}
