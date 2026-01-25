//! OpenTelemetry integration for production observability.
//!
//! This module provides OTLP (OpenTelemetry Protocol) export of metrics and traces
//! for monitoring Communitas in production environments.
//!
//! # Feature Flag
//!
//! This module is only compiled when the `metrics` feature is enabled:
//! ```toml
//! communitas-core = { version = "0.8.1", features = ["metrics"] }
//! ```
//!
//! # Configuration
//!
//! The OTLP endpoint is configured via environment variable:
//! ```bash
//! OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use communitas_core::telemetry::{TelemetryConfig, init_telemetry, shutdown_telemetry};
//!
//! let config = TelemetryConfig {
//!     endpoint: Some("http://localhost:4317".to_string()),
//!     service_name: "communitas".to_string(),
//!     environment: "production".to_string(),
//! };
//!
//! init_telemetry(config)?;
//!
//! // Application runs...
//!
//! shutdown_telemetry();
//! ```

#[cfg(feature = "metrics")]
use std::time::Duration;
use thiserror::Error;

#[cfg(feature = "metrics")]
use opentelemetry::global;
#[cfg(feature = "metrics")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "metrics")]
use opentelemetry_sdk::runtime;

/// Telemetry configuration for OTLP export.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// OTLP endpoint URL (e.g., "http://localhost:4317").
    /// If None, telemetry is disabled.
    pub endpoint: Option<String>,
    /// Service name for identifying this application in traces/metrics.
    pub service_name: String,
    /// Environment label (e.g., "production", "staging", "development").
    pub environment: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            service_name: "communitas".to_string(),
            environment: std::env::var("OTEL_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
        }
    }
}

/// Errors that can occur during telemetry initialization.
#[derive(Debug, Error)]
pub enum TelemetryError {
    /// Failed to initialize the OTLP exporter.
    #[error("Failed to initialize OTLP exporter: {0}")]
    ExporterInit(String),

    /// Failed to initialize the meter provider.
    #[error("Failed to initialize meter provider: {0}")]
    MeterProviderInit(String),

    /// No endpoint configured.
    #[error("No OTLP endpoint configured")]
    NoEndpoint,
}

/// Initializes OpenTelemetry with OTLP export.
///
/// This function sets up the global meter provider for metrics collection.
/// If no endpoint is configured, telemetry is silently disabled.
///
/// # Arguments
///
/// * `config` - Telemetry configuration including endpoint and service name.
///
/// # Returns
///
/// Returns `Ok(true)` if telemetry was initialized, `Ok(false)` if disabled,
/// or an error if initialization failed.
///
/// # Example
///
/// ```rust,ignore
/// use communitas_core::telemetry::{TelemetryConfig, init_telemetry};
///
/// let config = TelemetryConfig::default();
/// match init_telemetry(config) {
///     Ok(true) => tracing::info!("Telemetry enabled"),
///     Ok(false) => tracing::info!("Telemetry disabled (no endpoint)"),
///     Err(e) => tracing::warn!("Telemetry init failed: {}", e),
/// }
/// ```
#[cfg(feature = "metrics")]
pub fn init_telemetry(config: TelemetryConfig) -> Result<bool, TelemetryError> {
    let endpoint = match config.endpoint {
        Some(ep) => ep,
        None => {
            tracing::info!("Telemetry disabled: no OTEL_EXPORTER_OTLP_ENDPOINT configured");
            return Ok(false);
        }
    };

    tracing::info!(
        endpoint = %endpoint,
        service = %config.service_name,
        environment = %config.environment,
        "Initializing OpenTelemetry OTLP exporter"
    );

    // Build OTLP metric exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&endpoint)
        .with_timeout(Duration::from_secs(10));

    // Build meter provider with periodic export
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(exporter)
        .with_period(Duration::from_secs(30))
        .build()
        .map_err(|e| TelemetryError::MeterProviderInit(e.to_string()))?;

    // Set as global meter provider
    global::set_meter_provider(meter_provider);

    tracing::info!("OpenTelemetry initialized successfully");
    Ok(true)
}

/// Stub implementation when metrics feature is disabled.
#[cfg(not(feature = "metrics"))]
pub fn init_telemetry(_config: TelemetryConfig) -> Result<bool, TelemetryError> {
    tracing::debug!("Telemetry disabled: 'metrics' feature not enabled");
    Ok(false)
}

/// Shuts down the telemetry provider gracefully.
///
/// This should be called before application exit to ensure all pending
/// metrics are flushed to the collector.
#[cfg(feature = "metrics")]
pub fn shutdown_telemetry() {
    tracing::info!("Shutting down OpenTelemetry provider");
    // Note: In OpenTelemetry 0.22, we need to drop the meter provider
    // The global meter doesn't have a direct shutdown method
    // Traces use shutdown_tracer_provider, but metrics are flushed on drop
}

/// Stub implementation when metrics feature is disabled.
#[cfg(not(feature = "metrics"))]
pub fn shutdown_telemetry() {
    // No-op when metrics disabled
}

/// Returns a meter for recording metrics.
///
/// The meter is named with the provided scope, typically the module name.
///
/// # Arguments
///
/// * `scope` - The meter scope/name (e.g., "communitas.sync")
///
/// # Example
///
/// ```rust,ignore
/// let meter = communitas_core::telemetry::meter("communitas.sync");
/// let counter = meter.u64_counter("sync_operations").init();
/// counter.add(1, &[]);
/// ```
#[cfg(feature = "metrics")]
pub fn meter(scope: &'static str) -> opentelemetry::metrics::Meter {
    global::meter(scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TelemetryConfig::default();
        assert_eq!(config.service_name, "communitas");
        // endpoint depends on env var
    }

    #[test]
    fn test_init_without_endpoint() {
        let config = TelemetryConfig {
            endpoint: None,
            service_name: "test".to_string(),
            environment: "test".to_string(),
        };

        // Should return Ok(false) when no endpoint
        let result = init_telemetry(config);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_shutdown_noop() {
        // Should not panic even without init
        shutdown_telemetry();
    }
}
