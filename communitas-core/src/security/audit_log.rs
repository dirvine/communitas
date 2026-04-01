// SPDX-License-Identifier: MIT OR Apache-2.0

//! Persistent Audit Log for Security Events
//!
//! Provides encrypted, append-only logging of security-relevant events including:
//! - Authentication attempts (success and failure)
//! - Identity switches
//! - Device changes
//! - Recovery operations
//!
//! Events are stored encrypted using ChaCha20-Poly1305 with a device-derived key.
//! Logs rotate based on size (10MB) and age (60 days).

use anyhow::{Context, Result};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use zeroize::Zeroizing;

/// Maximum size of a single audit log file (10MB)
const MAX_LOG_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Maximum age of audit log files before rotation (60 days)
const MAX_LOG_AGE_DAYS: i64 = 60;

/// Type of security event being logged
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Successful login
    Login,
    /// Logout (explicit or timeout)
    Logout,
    /// Failed login attempt
    FailedLogin,
    /// Switched between identities
    IdentitySwitch,
    /// New device detected accessing vault
    DeviceChange,
    /// Identity recovered from mnemonic
    Recovery,
    /// Session refreshed before expiration
    SessionRefresh,
    /// Session expired
    SessionExpired,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Login => write!(f, "login"),
            Self::Logout => write!(f, "logout"),
            Self::FailedLogin => write!(f, "failed_login"),
            Self::IdentitySwitch => write!(f, "identity_switch"),
            Self::DeviceChange => write!(f, "device_change"),
            Self::Recovery => write!(f, "recovery"),
            Self::SessionRefresh => write!(f, "session_refresh"),
            Self::SessionExpired => write!(f, "session_expired"),
        }
    }
}

/// A single audit event with all relevant metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub id: String,

    /// When the event occurred
    pub timestamp: DateTime<Utc>,

    /// Type of event
    pub event_type: AuditEventType,

    /// Whether the operation succeeded
    pub success: bool,

    /// Identity four-words (redacted to first 2 words + "••••")
    pub identity_redacted: String,

    /// Device fingerprint hash
    pub device_fingerprint: String,

    /// Additional event-specific metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        event_type: AuditEventType,
        success: bool,
        four_words: &str,
        device_fingerprint: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            timestamp: Utc::now(),
            event_type,
            success,
            identity_redacted: redact_identity(four_words),
            device_fingerprint: device_fingerprint.to_string(),
            metadata: None,
        }
    }

    /// Create a new audit event with metadata
    pub fn with_metadata(
        event_type: AuditEventType,
        success: bool,
        four_words: &str,
        device_fingerprint: &str,
        metadata: serde_json::Value,
    ) -> Self {
        let mut event = Self::new(event_type, success, four_words, device_fingerprint);
        event.metadata = Some(metadata);
        event
    }
}

/// Redact identity to first 2 words plus masked suffix
fn redact_identity(four_words: &str) -> String {
    let words: Vec<&str> = four_words.split('-').collect();
    if words.len() >= 2 {
        format!("{}-{}-••••", words[0], words[1])
    } else {
        "••••-••••-••••".to_string()
    }
}

/// Persistent audit log manager
///
/// Handles encrypted storage of audit events with automatic rotation.
pub struct AuditLog {
    /// Directory containing audit log files
    log_dir: PathBuf,

    /// Encryption key derived from device secret
    encryption_key: Zeroizing<Vec<u8>>,

    /// Current log file path
    current_log: RwLock<PathBuf>,
}

impl AuditLog {
    /// Create or open an audit log
    ///
    /// # Arguments
    /// * `log_dir` - Directory to store audit logs
    /// * `device_key` - 32-byte device-derived encryption key
    ///
    /// # Errors
    /// Returns error if directory creation fails or key is invalid
    #[instrument(skip(device_key))]
    pub async fn new(log_dir: PathBuf, device_key: Zeroizing<Vec<u8>>) -> Result<Self> {
        // Ensure directory exists
        fs::create_dir_all(&log_dir)
            .with_context(|| format!("Failed to create audit log directory: {:?}", log_dir))?;

        // Validate key length
        if device_key.len() != 32 {
            return Err(anyhow::anyhow!(
                "Invalid device key length: expected 32, got {}",
                device_key.len()
            ));
        }

        // Find or create current log file
        let current_log = Self::find_or_create_current_log(&log_dir)?;

        info!("Audit log initialized at {:?}", log_dir);

        Ok(Self {
            log_dir,
            encryption_key: device_key,
            current_log: RwLock::new(current_log),
        })
    }

    /// Find the most recent log file or create a new one
    fn find_or_create_current_log(log_dir: &Path) -> Result<PathBuf> {
        let mut log_files: Vec<_> = fs::read_dir(log_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name().to_string_lossy().starts_with("audit_")
                    && entry.file_name().to_string_lossy().ends_with(".enc")
            })
            .collect();

        // Sort by name (which includes timestamp) descending
        log_files.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));

        if let Some(latest) = log_files.first() {
            // Check if latest file is under size limit
            let metadata = latest.metadata()?;
            if metadata.len() < MAX_LOG_SIZE_BYTES {
                return Ok(latest.path());
            }
        }

        // Create new log file
        Self::create_new_log_file(log_dir)
    }

    /// Create a new timestamped log file
    fn create_new_log_file(log_dir: &Path) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("audit_{}.enc", timestamp);
        let path = log_dir.join(filename);

        // Touch the file to create it
        File::create(&path).with_context(|| format!("Failed to create audit log: {:?}", path))?;

        debug!("Created new audit log file: {:?}", path);
        Ok(path)
    }

    /// Log a security event
    ///
    /// # Arguments
    /// * `event` - The audit event to log
    ///
    /// # Errors
    /// Returns error if encryption or write fails
    #[instrument(skip(self), fields(event_type = %event.event_type))]
    pub async fn log(&self, event: AuditEvent) -> Result<()> {
        // Serialize event to JSON
        let event_json =
            serde_json::to_string(&event).with_context(|| "Failed to serialize audit event")?;

        // Encrypt the event
        let encrypted = self
            .encrypt_line(&event_json)
            .await
            .with_context(|| "Failed to encrypt audit event")?;

        // Get current log file, rotating if needed
        let log_path = self.get_or_rotate_log().await?;

        // Append to log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("Failed to open audit log: {:?}", log_path))?;

        let mut writer = BufWriter::new(file);
        writeln!(writer, "{}", encrypted).with_context(|| "Failed to write audit event")?;
        writer
            .flush()
            .with_context(|| "Failed to flush audit log")?;

        debug!("Logged audit event: {} ({})", event.event_type, event.id);
        Ok(())
    }

    /// Get current log file path, rotating if size exceeded
    ///
    /// Uses a single write lock for the entire check-and-rotate operation
    /// to prevent race conditions between concurrent callers.
    async fn get_or_rotate_log(&self) -> Result<PathBuf> {
        // Use write lock for entire operation to prevent TOCTOU race
        let mut current_lock = self.current_log.write().await;
        let current = current_lock.clone();

        // Check if rotation needed
        let metadata = fs::metadata(&current);
        let needs_rotation = match metadata {
            Ok(m) => m.len() >= MAX_LOG_SIZE_BYTES,
            Err(_) => true, // File doesn't exist, create new one
        };

        if needs_rotation {
            let new_log = Self::create_new_log_file(&self.log_dir)?;
            *current_lock = new_log.clone();
            return Ok(new_log);
        }

        Ok(current)
    }

    /// Encrypt a single line for storage
    async fn encrypt_line(&self, plaintext: &str) -> Result<String> {
        let key = Key::from_slice(&self.encryption_key);
        let cipher = ChaCha20Poly1305::new(key);

        // Generate random nonce
        let nonce = ChaCha20Poly1305::generate_nonce(&mut rand::thread_rng());

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

        // Encode as base64: nonce || ciphertext
        let mut combined = nonce.to_vec();
        combined.extend(ciphertext);

        Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            combined,
        ))
    }

    /// Decrypt a single line from storage
    fn decrypt_line(&self, encrypted: &str) -> Result<String> {
        // Decode from base64
        let combined =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encrypted.trim())
                .with_context(|| "Failed to decode audit line")?;

        if combined.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted line: too short"));
        }

        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let key = Key::from_slice(&self.encryption_key);
        let cipher = ChaCha20Poly1305::new(key);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

        String::from_utf8(plaintext).with_context(|| "Decrypted data is not valid UTF-8")
    }

    /// Read recent audit events
    ///
    /// # Arguments
    /// * `limit` - Maximum number of events to return
    /// * `event_filter` - Optional filter for event types
    ///
    /// # Returns
    /// Events in reverse chronological order (newest first)
    #[instrument(skip(self))]
    pub async fn read_recent(
        &self,
        limit: usize,
        event_filter: Option<Vec<AuditEventType>>,
    ) -> Result<Vec<AuditEvent>> {
        let mut events = Vec::new();

        // Get all log files sorted by name (most recent first)
        let mut log_files: Vec<_> = fs::read_dir(&self.log_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name().to_string_lossy().starts_with("audit_")
                    && entry.file_name().to_string_lossy().ends_with(".enc")
            })
            .collect();

        log_files.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));

        // Read events from files until we have enough
        for entry in log_files {
            if events.len() >= limit {
                break;
            }

            let file = File::open(entry.path())?;
            let reader = BufReader::new(file);

            // Read all lines from this file
            let mut file_events: Vec<AuditEvent> = Vec::new();
            for line in reader.lines() {
                let line = match line {
                    Ok(l) if !l.is_empty() => l,
                    _ => continue,
                };

                match self.decrypt_line(&line) {
                    Ok(decrypted) => {
                        if let Ok(event) = serde_json::from_str::<AuditEvent>(&decrypted) {
                            // Apply filter if specified
                            if let Some(ref filter) = event_filter
                                && !filter.contains(&event.event_type)
                            {
                                continue;
                            }
                            file_events.push(event);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to decrypt audit line: {}", e);
                        continue;
                    }
                }
            }

            // Reverse to get newest first within this file
            file_events.reverse();
            events.extend(file_events);
        }

        // Truncate to limit
        events.truncate(limit);
        Ok(events)
    }

    /// Clean up old log files
    ///
    /// Removes log files older than `MAX_LOG_AGE_DAYS`
    #[instrument(skip(self))]
    pub async fn cleanup_old_logs(&self) -> Result<usize> {
        let cutoff = Utc::now() - Duration::days(MAX_LOG_AGE_DAYS);
        let mut removed = 0;

        let entries = fs::read_dir(&self.log_dir)?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip non-audit files
            if !filename.starts_with("audit_") || !filename.ends_with(".enc") {
                continue;
            }

            // Parse timestamp from filename: audit_YYYYMMDD_HHMMSS.enc
            if let Some(timestamp_str) = filename
                .strip_prefix("audit_")
                .and_then(|s| s.strip_suffix(".enc"))
                && let Ok(file_time) =
                    chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y%m%d_%H%M%S")
            {
                let file_datetime = file_time.and_utc();
                if file_datetime < cutoff {
                    if let Err(e) = fs::remove_file(&path) {
                        warn!("Failed to remove old audit log {:?}: {}", path, e);
                    } else {
                        info!("Removed old audit log: {:?}", path);
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
    }

    /// Export events within a date range to JSON
    ///
    /// # Arguments
    /// * `start` - Start of date range (inclusive)
    /// * `end` - End of date range (inclusive)
    /// * `event_filter` - Optional filter for event types
    #[instrument(skip(self))]
    pub async fn export_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        event_filter: Option<Vec<AuditEventType>>,
    ) -> Result<Vec<AuditEvent>> {
        let mut events = Vec::new();

        // Get all log files
        let log_files: Vec<_> = fs::read_dir(&self.log_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name().to_string_lossy().starts_with("audit_")
                    && entry.file_name().to_string_lossy().ends_with(".enc")
            })
            .collect();

        for entry in log_files {
            let file = File::open(entry.path())?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = match line {
                    Ok(l) if !l.is_empty() => l,
                    _ => continue,
                };

                match self.decrypt_line(&line) {
                    Ok(decrypted) => {
                        if let Ok(event) = serde_json::from_str::<AuditEvent>(&decrypted) {
                            // Filter by date range
                            if event.timestamp < start || event.timestamp > end {
                                continue;
                            }

                            // Apply event type filter if specified
                            if let Some(ref filter) = event_filter
                                && !filter.contains(&event.event_type)
                            {
                                continue;
                            }

                            events.push(event);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to decrypt audit line during export: {}", e);
                        continue;
                    }
                }
            }
        }

        // Sort by timestamp ascending
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_log() -> (AuditLog, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let key = Zeroizing::new(vec![0u8; 32]); // Test key
        let log = AuditLog::new(temp_dir.path().to_path_buf(), key)
            .await
            .unwrap();
        (log, temp_dir)
    }

    #[tokio::test]
    async fn test_create_audit_log() {
        let (log, _temp_dir) = create_test_log().await;
        let current = log.current_log.read().await;
        assert!(current.exists());
    }

    #[tokio::test]
    async fn test_log_and_read_event() {
        let (log, _temp_dir) = create_test_log().await;

        let event = AuditEvent::new(
            AuditEventType::Login,
            true,
            "ocean-forest-moon-star",
            "device123",
        );

        log.log(event.clone()).await.unwrap();

        let events = log.read_recent(10, None).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, AuditEventType::Login);
        assert_eq!(events[0].identity_redacted, "ocean-forest-••••");
        assert!(events[0].success);
    }

    #[tokio::test]
    async fn test_event_filter() {
        let (log, _temp_dir) = create_test_log().await;

        // Log different event types
        log.log(AuditEvent::new(
            AuditEventType::Login,
            true,
            "test-identity-one-two",
            "device1",
        ))
        .await
        .unwrap();

        log.log(AuditEvent::new(
            AuditEventType::FailedLogin,
            false,
            "test-identity-one-two",
            "device1",
        ))
        .await
        .unwrap();

        log.log(AuditEvent::new(
            AuditEventType::Logout,
            true,
            "test-identity-one-two",
            "device1",
        ))
        .await
        .unwrap();

        // Filter for only failed logins
        let events = log
            .read_recent(10, Some(vec![AuditEventType::FailedLogin]))
            .await
            .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, AuditEventType::FailedLogin);
    }

    #[tokio::test]
    async fn test_redact_identity() {
        assert_eq!(
            redact_identity("ocean-forest-moon-star"),
            "ocean-forest-••••"
        );
        assert_eq!(redact_identity("alpha-beta-gamma-delta"), "alpha-beta-••••");
        assert_eq!(redact_identity("short"), "••••-••••-••••");
    }

    #[tokio::test]
    async fn test_event_with_metadata() {
        let (log, _temp_dir) = create_test_log().await;

        let metadata = serde_json::json!({
            "ip_address": "192.168.1.1",
            "user_agent": "Communitas/1.0"
        });

        let event = AuditEvent::with_metadata(
            AuditEventType::DeviceChange,
            true,
            "test-words-one-two",
            "device456",
            metadata,
        );

        log.log(event).await.unwrap();

        let events = log.read_recent(10, None).await.unwrap();
        assert_eq!(events.len(), 1);
        assert!(events[0].metadata.is_some());
    }

    #[tokio::test]
    async fn test_invalid_key_length() {
        let temp_dir = TempDir::new().unwrap();
        let key = Zeroizing::new(vec![0u8; 16]); // Invalid: too short
        let result = AuditLog::new(temp_dir.path().to_path_buf(), key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_decryption_with_wrong_key_fails() {
        let temp_dir = TempDir::new().unwrap();

        // Create log with first key and write an event
        let key1 = Zeroizing::new(vec![0u8; 32]);
        let log1 = AuditLog::new(temp_dir.path().to_path_buf(), key1)
            .await
            .unwrap();

        let event = AuditEvent::new(
            AuditEventType::Login,
            true,
            "ocean-forest-moon-star",
            "device123",
        );
        log1.log(event).await.unwrap();

        // Create log with different key pointing to same directory
        let key2 = Zeroizing::new(vec![1u8; 32]); // Different key
        let log2 = AuditLog::new(temp_dir.path().to_path_buf(), key2)
            .await
            .unwrap();

        // Attempt to read events - should return empty due to decryption failures
        // The read_recent method logs warnings for failed decryptions and continues
        let events = log2.read_recent(10, None).await.unwrap();

        // Should be empty because decryption with wrong key fails
        assert!(
            events.is_empty(),
            "Events should be empty when using wrong key"
        );
    }

    #[tokio::test]
    async fn test_decrypt_line_with_wrong_key_returns_error() {
        let temp_dir = TempDir::new().unwrap();

        // Create log and encrypt a line
        let key1 = Zeroizing::new(vec![0u8; 32]);
        let log1 = AuditLog::new(temp_dir.path().to_path_buf(), key1)
            .await
            .unwrap();

        let encrypted = log1.encrypt_line("secret message").await.unwrap();

        // Create log with different key
        let key2 = Zeroizing::new(vec![1u8; 32]);
        let log2 = AuditLog::new(temp_dir.path().to_path_buf(), key2)
            .await
            .unwrap();

        // Decryption should fail with wrong key
        let result = log2.decrypt_line(&encrypted);
        assert!(result.is_err(), "Decryption should fail with wrong key");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Decryption failed"),
            "Error should indicate decryption failure: {}",
            err_msg
        );
    }
}
