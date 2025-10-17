// Copyright (c) 2025 Saorsa Labs Limited
//
// Application monitoring and metrics service
//
// Tracks performance metrics, errors, and usage statistics for observability

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub name: String,
    pub value: f64,
    pub timestamp: u64,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub id: String,
    pub message: String,
    pub error_type: String,
    pub stack_trace: Option<String>,
    pub timestamp: u64,
    pub context: HashMap<String, String>,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug)]
pub struct MonitoringService {
    metrics: Arc<RwLock<HashMap<String, Vec<MetricValue>>>>,
    errors: Arc<RwLock<Vec<ErrorReport>>>,
    start_time: Instant,
}

impl MonitoringService {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            errors: Arc::new(RwLock::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    /// Record an application error
    pub async fn record_error(
        &self,
        message: &str,
        error_type: &str,
        severity: ErrorSeverity,
        context: HashMap<String, String>,
        stack_trace: Option<String>,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Log based on severity first (before moving severity)
        match severity {
            ErrorSeverity::Critical => error!("CRITICAL ERROR: {} - {}", error_type, message),
            ErrorSeverity::High => error!("HIGH ERROR: {} - {}", error_type, message),
            ErrorSeverity::Medium => warn!("MEDIUM ERROR: {} - {}", error_type, message),
            ErrorSeverity::Low => info!("LOW ERROR: {} - {}", error_type, message),
        }

        let report = ErrorReport {
            id: format!("error_{}", timestamp),
            message: message.to_string(),
            error_type: error_type.to_string(),
            stack_trace,
            timestamp,
            context,
            severity,
        };

        let mut errors = self.errors.write().await;
        errors.push(report.clone());

        // Keep only last 500 errors
        let current_len = errors.len();
        if current_len > 500 {
            errors.drain(0..current_len - 500);
        }
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get recent metrics
    pub async fn get_recent_metrics(&self, name: Option<&str>, limit: usize) -> Vec<MetricValue> {
        let metrics = self.metrics.read().await;

        if let Some(metric_name) = name {
            if let Some(values) = metrics.get(metric_name) {
                return values
                    .iter()
                    .rev()
                    .take(limit)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
            }
        } else {
            // Return all metrics, sorted by timestamp
            let mut all_metrics: Vec<MetricValue> = metrics.values().flatten().cloned().collect();

            all_metrics.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            return all_metrics.into_iter().take(limit).collect();
        }

        Vec::new()
    }

    /// Get recent errors
    pub async fn get_recent_errors(&self, limit: usize) -> Vec<ErrorReport> {
        let errors = self.errors.read().await;
        let recent: Vec<ErrorReport> = errors.iter().rev().take(limit).cloned().collect();

        recent.into_iter().rev().collect()
    }

    /// Get error statistics
    pub async fn get_error_stats(&self) -> HashMap<String, u32> {
        let errors = self.errors.read().await;
        let mut stats = HashMap::new();

        for error in errors.iter() {
            *stats.entry(error.error_type.clone()).or_insert(0) += 1;
        }

        stats
    }

    /// Export metrics for external monitoring systems
    pub async fn export_metrics(&self) -> String {
        let metrics = self.metrics.read().await;
        let mut output = String::new();

        for (name, values) in metrics.iter() {
            if let Some(latest) = values.last() {
                output.push_str(&format!(
                    "# HELP communitas_{} Latest value\n",
                    name.replace("-", "_")
                ));
                output.push_str(&format!(
                    "# TYPE communitas_{} gauge\n",
                    name.replace("-", "_")
                ));
                output.push_str(&format!(
                    "communitas_{}{} {}\n",
                    name.replace("-", "_"),
                    if latest.tags.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "{{{}}}",
                            latest
                                .tags
                                .iter()
                                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                                .collect::<Vec<_>>()
                                .join(",")
                        )
                    },
                    latest.value
                ));
            }
        }

        output
    }
}

// Global monitoring service instance
lazy_static::lazy_static! {
    pub static ref MONITORING: MonitoringService = MonitoringService::new();
}

// Convenience macros for recording metrics and errors
#[macro_export]
macro_rules! record_metric {
    ($name:expr, $value:expr) => {
        tokio::spawn(async move {
            $crate::services::monitoring::MONITORING
                .record_metric($name, $value, std::collections::HashMap::new())
                .await;
        });
    };
    ($name:expr, $value:expr, $tags:expr) => {
        tokio::spawn(async move {
            $crate::services::monitoring::MONITORING
                .record_metric($name, $value, $tags)
                .await;
        });
    };
}

#[macro_export]
macro_rules! record_error {
    ($message:expr, $error_type:expr, $severity:expr) => {
        tokio::spawn(async move {
            $crate::services::monitoring::MONITORING
                .record_error(
                    $message,
                    $error_type,
                    $severity,
                    std::collections::HashMap::new(),
                    None,
                )
                .await;
        });
    };
    ($message:expr, $error_type:expr, $severity:expr, $context:expr) => {
        tokio::spawn(async move {
            $crate::services::monitoring::MONITORING
                .record_error($message, $error_type, $severity, $context, None)
                .await;
        });
    };
    ($message:expr, $error_type:expr, $severity:expr, $context:expr, $stack:expr) => {
        tokio::spawn(async move {
            $crate::services::monitoring::MONITORING
                .record_error($message, $error_type, $severity, $context, Some($stack))
                .await;
        });
    };
}

// Tauri commands for monitoring
#[tauri::command]
pub async fn monitoring_get_metrics(
    limit: Option<usize>,
) -> Result<Vec<serde_json::Value>, String> {
    let metrics = MONITORING
        .get_recent_metrics(None, limit.unwrap_or(100))
        .await;
    metrics
        .into_iter()
        .map(|m| serde_json::to_value(m).map_err(|e| format!("Failed to serialize metric: {}", e)))
        .collect()
}

#[tauri::command]
pub async fn monitoring_get_errors(limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    let errors = MONITORING.get_recent_errors(limit.unwrap_or(50)).await;
    errors
        .into_iter()
        .map(|e| serde_json::to_value(e).map_err(|e| format!("Failed to serialize error: {}", e)))
        .collect()
}

#[tauri::command]
pub async fn monitoring_get_stats() -> Result<serde_json::Value, String> {
    let uptime = MONITORING.uptime_seconds();
    let error_stats = MONITORING.get_error_stats().await;

    let stats = serde_json::json!({
        "uptime_seconds": uptime,
        "error_counts": error_stats,
        "version": env!("CARGO_PKG_VERSION"),
    });

    Ok(stats)
}

#[tauri::command]
pub async fn monitoring_export_prometheus() -> Result<String, String> {
    Ok(MONITORING.export_metrics().await)
}
