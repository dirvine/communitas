//! Audit log service for UI layer.
//!
//! Wraps the core AuditLog to provide a simple interface for MCP and Dioxus
//! to read audit events. Events are logged through AuthController automatically.

use communitas_core::security::{AuditEvent, AuditEventType, AuditLog, DeviceFingerprint};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, instrument};
use zeroize::Zeroizing;

/// Errors from the audit service.
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("audit log not initialized")]
    NotInitialized,
    #[error("invalid device fingerprint: {0}")]
    InvalidFingerprint(String),
    #[error("audit log error: {0}")]
    Log(String),
}

/// Service for reading audit log events.
///
/// This service is lazily initialized on first access to avoid blocking
/// UiServices construction. The audit log requires a valid device fingerprint
/// to derive the encryption key.
pub struct AuditService {
    /// Directory for storing audit logs
    log_dir: PathBuf,

    /// Lazily initialized audit log
    audit_log: RwLock<Option<Arc<AuditLog>>>,
}

impl AuditService {
    /// Create a new audit service.
    ///
    /// The audit log is lazily initialized on first read operation.
    pub fn new(log_dir: PathBuf) -> Self {
        Self {
            log_dir,
            audit_log: RwLock::new(None),
        }
    }

    /// Ensure the audit log is initialized.
    ///
    /// This derives the encryption key from the device fingerprint and
    /// initializes the AuditLog if not already done.
    #[instrument(skip(self))]
    async fn ensure_initialized(&self) -> Result<Arc<AuditLog>, AuditError> {
        // Check if already initialized
        {
            let guard = self.audit_log.read().await;
            if let Some(ref log) = *guard {
                return Ok(Arc::clone(log));
            }
        }

        // Need to initialize - take write lock
        let mut guard = self.audit_log.write().await;

        // Double-check after acquiring write lock
        if let Some(ref log) = *guard {
            return Ok(Arc::clone(log));
        }

        // Get device fingerprint to derive encryption key
        let fingerprint = DeviceFingerprint::current()
            .map_err(|e| AuditError::InvalidFingerprint(e.to_string()))?;

        // The fingerprint is 64 hex chars (32 bytes when decoded)
        let key_bytes = hex::decode(&fingerprint.fingerprint)
            .map_err(|e| AuditError::InvalidFingerprint(format!("hex decode failed: {e}")))?;

        if key_bytes.len() != 32 {
            return Err(AuditError::InvalidFingerprint(format!(
                "expected 32 bytes, got {}",
                key_bytes.len()
            )));
        }

        let device_key = Zeroizing::new(key_bytes);

        // Create the audit log
        let audit_log = AuditLog::new(self.log_dir.clone(), device_key)
            .await
            .map_err(|e| AuditError::Log(e.to_string()))?;

        let log = Arc::new(audit_log);
        *guard = Some(Arc::clone(&log));

        debug!("Audit service initialized with log dir {:?}", self.log_dir);
        Ok(log)
    }

    /// Read recent audit events.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of events to return (newest first)
    /// * `event_types` - Optional filter for specific event types
    #[instrument(skip(self))]
    pub async fn read_recent(
        &self,
        limit: usize,
        event_types: Option<Vec<AuditEventType>>,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        let log = self.ensure_initialized().await?;
        log.read_recent(limit, event_types)
            .await
            .map_err(|e| AuditError::Log(e.to_string()))
    }

    /// Export audit events within a date range.
    ///
    /// # Arguments
    /// * `start_iso` - Start date in ISO 8601 format (inclusive)
    /// * `end_iso` - End date in ISO 8601 format (inclusive)
    /// * `event_types` - Optional filter for specific event types
    #[instrument(skip(self))]
    pub async fn export_range(
        &self,
        start_iso: &str,
        end_iso: &str,
        event_types: Option<Vec<AuditEventType>>,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        let log = self.ensure_initialized().await?;

        // Parse ISO dates
        let start = chrono::DateTime::parse_from_rfc3339(start_iso)
            .map_err(|e| AuditError::Log(format!("invalid start date: {e}")))?
            .with_timezone(&chrono::Utc);

        let end = chrono::DateTime::parse_from_rfc3339(end_iso)
            .map_err(|e| AuditError::Log(format!("invalid end date: {e}")))?
            .with_timezone(&chrono::Utc);

        log.export_range(start, end, event_types)
            .await
            .map_err(|e| AuditError::Log(e.to_string()))
    }

    /// Log an audit event.
    ///
    /// This is typically called internally by AuthController, not by UI code.
    #[instrument(skip(self, event), fields(event_type = %event.event_type))]
    pub async fn log_event(&self, event: AuditEvent) -> Result<(), AuditError> {
        let log = self.ensure_initialized().await?;
        log.log(event)
            .await
            .map_err(|e| AuditError::Log(e.to_string()))
    }

    /// Clean up old log files (older than 60 days).
    #[instrument(skip(self))]
    pub async fn cleanup_old_logs(&self) -> Result<usize, AuditError> {
        let log = self.ensure_initialized().await?;
        log.cleanup_old_logs()
            .await
            .map_err(|e| AuditError::Log(e.to_string()))
    }
}

/// Parse event type strings to enum values.
pub fn parse_event_types(types: &[String]) -> Result<Vec<AuditEventType>, AuditError> {
    types
        .iter()
        .map(|s| match s.to_lowercase().as_str() {
            "login" => Ok(AuditEventType::Login),
            "logout" => Ok(AuditEventType::Logout),
            "failed_login" => Ok(AuditEventType::FailedLogin),
            "identity_switch" => Ok(AuditEventType::IdentitySwitch),
            "device_change" => Ok(AuditEventType::DeviceChange),
            "recovery" => Ok(AuditEventType::Recovery),
            "passkey_register" => Ok(AuditEventType::PasskeyRegister),
            "passkey_auth" => Ok(AuditEventType::PasskeyAuth),
            "session_refresh" => Ok(AuditEventType::SessionRefresh),
            "session_expired" => Ok(AuditEventType::SessionExpired),
            other => Err(AuditError::Log(format!("unknown event type: {other}"))),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_audit_service_lazy_init() {
        let temp_dir = TempDir::new().unwrap();
        let service = AuditService::new(temp_dir.path().join("audit_logs"));

        // Service should initialize on first read
        let events = service.read_recent(10, None).await.unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_parse_event_types() {
        let types = vec![
            "login".to_string(),
            "logout".to_string(),
            "failed_login".to_string(),
        ];
        let parsed = parse_event_types(&types).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], AuditEventType::Login);
        assert_eq!(parsed[1], AuditEventType::Logout);
        assert_eq!(parsed[2], AuditEventType::FailedLogin);
    }

    #[tokio::test]
    async fn test_parse_event_types_invalid() {
        let types = vec!["invalid_type".to_string()];
        let result = parse_event_types(&types);
        assert!(result.is_err());
    }
}
