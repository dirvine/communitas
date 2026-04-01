// SPDX-License-Identifier: MIT OR Apache-2.0

//! Session Management for Multi-Account Support
//!
//! Handles authenticated sessions with automatic expiration,
//! allowing seamless switching between multiple accounts.
//!
//! Session data is encrypted at rest using ChaCha20-Poly1305 with a
//! device-derived key for security.

use anyhow::{Context, Result};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit},
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};
use zeroize::Zeroizing;

/// Represents an authenticated session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub four_words: String,
    pub display_name: String,
    /// Hex-encoded ML-DSA-87 public key (the user's cryptographic identity)
    #[serde(default)]
    pub pubkey_hex: Option<String>,
    pub created_at: u64,
    pub last_activity: u64,
    pub expires_at: u64,
    pub auth_method: AuthMethod,
}

/// Authentication method used for the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Local,
    Password,
    PasswordOnly, // Familiar device login
    Passkey,
    Combined, // Password + Passkey
}

impl Session {
    /// Create a new session
    pub fn new(four_words: String, display_name: String, timeout_seconds: u64) -> Self {
        let now = current_timestamp();
        Self {
            id: generate_session_id(),
            four_words,
            display_name,
            pubkey_hex: None,
            created_at: now,
            last_activity: now,
            expires_at: now + timeout_seconds,
            auth_method: AuthMethod::Local,
        }
    }

    /// Create a new session with pubkey_hex
    pub fn new_with_pubkey(
        four_words: String,
        display_name: String,
        pubkey_hex: String,
        timeout_seconds: u64,
    ) -> Self {
        let now = current_timestamp();
        Self {
            id: generate_session_id(),
            four_words,
            display_name,
            pubkey_hex: Some(pubkey_hex),
            created_at: now,
            last_activity: now,
            expires_at: now + timeout_seconds,
            auth_method: AuthMethod::Local,
        }
    }

    /// Create a password-only session (familiar device)
    pub fn new_password_only(
        four_words: String,
        display_name: String,
        timeout_seconds: u64,
    ) -> Self {
        let mut session = Self::new(four_words, display_name, timeout_seconds);
        session.auth_method = AuthMethod::PasswordOnly;
        session
    }

    /// Set the pubkey_hex on an existing session
    pub fn with_pubkey_hex(mut self, pubkey_hex: String) -> Self {
        self.pubkey_hex = Some(pubkey_hex);
        self
    }

    /// Check if the session has expired
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }

    /// Update the last activity time
    pub fn touch(&mut self) {
        self.last_activity = current_timestamp();
    }

    /// Extend the session expiration
    pub fn extend(&mut self, additional_seconds: u64) {
        self.expires_at = current_timestamp() + additional_seconds;
        self.touch();
    }

    /// Get remaining time until expiration
    pub fn time_remaining(&self) -> Duration {
        let now = current_timestamp();
        if now >= self.expires_at {
            Duration::from_secs(0)
        } else {
            Duration::from_secs(self.expires_at - now)
        }
    }
}

/// Session manager for handling multiple active sessions
pub struct SessionManager {
    sessions: std::sync::Arc<tokio::sync::RwLock<Vec<Session>>>,
    max_sessions: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(max_sessions: usize) -> Self {
        Self {
            sessions: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            max_sessions,
        }
    }

    /// Add a new session
    pub async fn add_session(&self, session: Session) -> Result<String, anyhow::Error> {
        let mut sessions = self.sessions.write().await;

        // Check for existing session with same four_words
        if let Some(existing) = sessions
            .iter_mut()
            .find(|s| s.four_words == session.four_words)
        {
            // Update existing session
            existing.extend(session.expires_at - session.created_at);
            return Ok(existing.id.clone());
        }

        // Enforce maximum sessions limit
        if sessions.len() >= self.max_sessions {
            // Remove oldest expired session or oldest by creation time
            sessions.sort_by_key(|s| s.created_at);
            sessions.remove(0);
        }

        let session_id = session.id.clone();
        sessions.push(session);

        Ok(session_id)
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .iter()
            .find(|s| s.id == session_id && !s.is_expired())
            .cloned()
    }

    /// Get all active sessions
    pub async fn get_active_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .iter()
            .filter(|s| !s.is_expired())
            .cloned()
            .collect()
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(pos) = sessions.iter().position(|s| s.id == session_id) {
            sessions.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let original_len = sessions.len();
        sessions.retain(|s| !s.is_expired());
        original_len - sessions.len()
    }

    /// Switch to a different account session
    pub async fn switch_session(&self, session_id: &str) -> Option<Session> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id)
            && !session.is_expired()
        {
            session.touch();
            return Some(session.clone());
        }
        None
    }
}

/// Session storage for persistence across app restarts
///
/// Sessions are encrypted at rest using ChaCha20-Poly1305 with a
/// device-derived encryption key.
pub struct SessionStorage {
    /// Path to encrypted session file
    storage_path: std::path::PathBuf,

    /// Path to legacy plaintext file (for migration)
    legacy_path: std::path::PathBuf,

    /// Encryption key (32 bytes, derived from device fingerprint + vault key)
    encryption_key: Zeroizing<Vec<u8>>,
}

impl SessionStorage {
    /// Create session storage with encryption
    ///
    /// # Arguments
    /// * `base_path` - Directory to store sessions
    /// * `encryption_key` - 32-byte key derived from device fingerprint
    ///
    /// # Errors
    /// Returns error if encryption key is invalid length
    pub fn new(base_path: &std::path::Path, encryption_key: Zeroizing<Vec<u8>>) -> Result<Self> {
        if encryption_key.len() != 32 {
            return Err(anyhow::anyhow!(
                "Invalid encryption key length: expected 32, got {}",
                encryption_key.len()
            ));
        }

        let storage_path = base_path.join("sessions.enc");
        let legacy_path = base_path.join("sessions.json");

        Ok(Self {
            storage_path,
            legacy_path,
            encryption_key,
        })
    }

    /// Create session storage without encryption (for testing/development only)
    ///
    /// Uses a zeroed key which provides no real security.
    /// DO NOT use in production.
    #[cfg(test)]
    pub fn new_unencrypted(base_path: &std::path::Path) -> Self {
        Self {
            storage_path: base_path.join("sessions.enc"),
            legacy_path: base_path.join("sessions.json"),
            encryption_key: Zeroizing::new(vec![0u8; 32]),
        }
    }

    /// Save sessions to encrypted storage
    ///
    /// Sessions are serialized to JSON, then encrypted with ChaCha20-Poly1305.
    /// A new random nonce is generated for each save operation.
    pub async fn save_sessions(&self, sessions: &[Session]) -> Result<()> {
        // Filter out expired sessions
        let active_sessions: Vec<_> = sessions
            .iter()
            .filter(|s| !s.is_expired())
            .cloned()
            .collect();

        // Serialize to JSON
        let json = serde_json::to_vec(&active_sessions).context("Failed to serialize sessions")?;

        // Encrypt with ChaCha20-Poly1305
        let encrypted = self.encrypt(&json)?;

        // Ensure parent directory exists
        if let Some(parent) = self.storage_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create session storage directory")?;
        }

        // Write encrypted data
        tokio::fs::write(&self.storage_path, encrypted)
            .await
            .context("Failed to write encrypted sessions")?;

        debug!(
            "Saved {} sessions to encrypted storage",
            active_sessions.len()
        );
        Ok(())
    }

    /// Load sessions from encrypted storage
    ///
    /// Handles migration from legacy plaintext sessions.json if present.
    pub async fn load_sessions(&self) -> Result<Vec<Session>> {
        // First, check for and migrate legacy plaintext sessions
        if self.legacy_path.exists() && !self.storage_path.exists() {
            info!("Migrating plaintext sessions to encrypted storage");
            if let Err(e) = self.migrate_legacy_sessions().await {
                warn!("Failed to migrate legacy sessions: {}. Starting fresh.", e);
            }
        }

        // Load encrypted sessions
        if !self.storage_path.exists() {
            return Ok(Vec::new());
        }

        let encrypted_data = tokio::fs::read(&self.storage_path)
            .await
            .context("Failed to read encrypted sessions")?;

        // Decrypt
        let json = self
            .decrypt(&encrypted_data)
            .context("Failed to decrypt sessions")?;

        // Deserialize
        let sessions: Vec<Session> =
            serde_json::from_slice(&json).context("Failed to parse session data")?;

        // Filter out expired sessions
        let active: Vec<_> = sessions.into_iter().filter(|s| !s.is_expired()).collect();

        debug!(
            "Loaded {} active sessions from encrypted storage",
            active.len()
        );
        Ok(active)
    }

    /// Clear all stored sessions
    pub async fn clear(&self) -> Result<()> {
        // Remove encrypted file
        if self.storage_path.exists() {
            tokio::fs::remove_file(&self.storage_path)
                .await
                .context("Failed to remove encrypted sessions")?;
        }

        // Also remove any legacy file
        if self.legacy_path.exists() {
            tokio::fs::remove_file(&self.legacy_path)
                .await
                .context("Failed to remove legacy sessions")?;
        }

        info!("Cleared all stored sessions");
        Ok(())
    }

    /// Migrate legacy plaintext sessions to encrypted storage
    async fn migrate_legacy_sessions(&self) -> Result<()> {
        // Read plaintext file
        let data = tokio::fs::read(&self.legacy_path)
            .await
            .context("Failed to read legacy sessions")?;

        // Parse sessions
        let sessions: Vec<Session> =
            serde_json::from_slice(&data).context("Failed to parse legacy sessions")?;

        // Save using encryption
        self.save_sessions(&sessions).await?;

        // Remove legacy file after successful migration
        tokio::fs::remove_file(&self.legacy_path)
            .await
            .context("Failed to remove legacy session file after migration")?;

        info!(
            "Successfully migrated {} sessions from plaintext to encrypted storage",
            sessions.len()
        );
        Ok(())
    }

    /// Encrypt data with ChaCha20-Poly1305
    ///
    /// Output format: nonce (12 bytes) || ciphertext
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key = Key::from_slice(&self.encryption_key);
        let cipher = ChaCha20Poly1305::new(key);

        // Generate random nonce
        let nonce = ChaCha20Poly1305::generate_nonce(&mut rand::thread_rng());

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Session encryption failed: {:?}", e))?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend(ciphertext);

        Ok(result)
    }

    /// Decrypt data with ChaCha20-Poly1305
    ///
    /// Input format: nonce (12 bytes) || ciphertext
    fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 12 {
            return Err(anyhow::anyhow!("Encrypted data too short: missing nonce"));
        }

        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let key = Key::from_slice(&self.encryption_key);
        let cipher = ChaCha20Poly1305::new(key);

        // Decrypt
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Session decryption failed: {:?}", e))
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) // Fallback to epoch if clock is before 1970 (extremely rare)
}

fn generate_session_id() -> String {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::from_entropy();
    let random_bytes: Vec<u8> = (0..16).map(|_| rng.r#gen::<u8>()).collect();
    hex::encode(random_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "test-four-words".to_string(),
            "Test User".to_string(),
            300, // 5 minutes
        );

        assert!(!session.is_expired());
        assert_eq!(session.four_words, "test-four-words");
        assert_eq!(session.display_name, "Test User");
    }

    #[test]
    fn test_session_expiration() {
        let mut session = Session::new(
            "test-four-words".to_string(),
            "Test User".to_string(),
            1, // 1 second
        );

        assert!(!session.is_expired());

        // Manually set expiration to past
        session.expires_at = current_timestamp() - 10;
        assert!(session.is_expired());
    }

    #[tokio::test]
    async fn test_session_manager() {
        let manager = SessionManager::new(3);

        // Add sessions
        let session1 = Session::new("user1".to_string(), "User 1".to_string(), 300);
        let session2 = Session::new("user2".to_string(), "User 2".to_string(), 300);
        let session3 = Session::new("user3".to_string(), "User 3".to_string(), 300);

        let id1 = manager.add_session(session1).await.unwrap();
        let id2 = manager.add_session(session2).await.unwrap();
        let _id3 = manager.add_session(session3).await.unwrap();

        // Get active sessions
        let active = manager.get_active_sessions().await;
        assert_eq!(active.len(), 3);

        // Switch session
        let switched = manager.switch_session(&id2).await;
        assert!(switched.is_some());
        assert_eq!(switched.unwrap().four_words, "user2");

        // Remove session
        assert!(manager.remove_session(&id1).await);
        let active_after = manager.get_active_sessions().await;
        assert_eq!(active_after.len(), 2);
    }

    #[tokio::test]
    async fn test_session_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new_unencrypted(temp_dir.path());

        let sessions = vec![
            Session::new("user1".to_string(), "User 1".to_string(), 300),
            Session::new("user2".to_string(), "User 2".to_string(), 300),
        ];

        // Save sessions
        storage.save_sessions(&sessions).await.unwrap();

        // Load sessions
        let loaded = storage.load_sessions().await.unwrap();
        assert_eq!(loaded.len(), 2);

        // Clear sessions
        storage.clear().await.unwrap();
        let after_clear = storage.load_sessions().await.unwrap();
        assert_eq!(after_clear.len(), 0);
    }

    #[tokio::test]
    async fn test_session_storage_encryption() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let key = Zeroizing::new(vec![42u8; 32]); // Test key
        let storage = SessionStorage::new(temp_dir.path(), key).unwrap();

        let sessions = vec![Session::new(
            "secure-user".to_string(),
            "Secure User".to_string(),
            300,
        )];

        // Save sessions
        storage.save_sessions(&sessions).await.unwrap();

        // Verify file is encrypted (not readable as plain JSON)
        let raw_data = std::fs::read(temp_dir.path().join("sessions.enc")).unwrap();
        let parse_result: Result<Vec<Session>, _> = serde_json::from_slice(&raw_data);
        assert!(
            parse_result.is_err(),
            "Encrypted data should not be valid JSON"
        );

        // Load sessions (should decrypt successfully)
        let loaded = storage.load_sessions().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].four_words, "secure-user");
    }

    #[tokio::test]
    async fn test_session_storage_invalid_key_length() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let short_key = Zeroizing::new(vec![0u8; 16]); // Too short

        let result = SessionStorage::new(temp_dir.path(), short_key);
        assert!(result.is_err());

        let err_msg = result.err().map(|e| e.to_string()).unwrap_or_default();
        assert!(
            err_msg.contains("Invalid encryption key length"),
            "Error: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_session_storage_migration() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create a legacy plaintext sessions.json
        let legacy_sessions = vec![Session::new(
            "legacy-user".to_string(),
            "Legacy User".to_string(),
            300,
        )];
        let legacy_json = serde_json::to_vec(&legacy_sessions).unwrap();
        std::fs::write(temp_dir.path().join("sessions.json"), legacy_json).unwrap();

        // Create encrypted storage and load (should trigger migration)
        let key = Zeroizing::new(vec![1u8; 32]);
        let storage = SessionStorage::new(temp_dir.path(), key).unwrap();

        let loaded = storage.load_sessions().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].four_words, "legacy-user");

        // Verify legacy file was removed
        assert!(!temp_dir.path().join("sessions.json").exists());

        // Verify encrypted file was created
        assert!(temp_dir.path().join("sessions.enc").exists());
    }

    #[tokio::test]
    async fn test_session_storage_wrong_key_fails() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Save with one key
        let key1 = Zeroizing::new(vec![1u8; 32]);
        let storage1 = SessionStorage::new(temp_dir.path(), key1).unwrap();
        storage1
            .save_sessions(&[Session::new(
                "test-user".to_string(),
                "Test".to_string(),
                300,
            )])
            .await
            .unwrap();

        // Try to load with different key
        let key2 = Zeroizing::new(vec![2u8; 32]);
        let storage2 = SessionStorage::new(temp_dir.path(), key2).unwrap();
        let result = storage2.load_sessions().await;

        // Should fail decryption
        assert!(result.is_err());
    }
}
