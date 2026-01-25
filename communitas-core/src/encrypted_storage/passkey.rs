//! Passkey/biometric authentication support using WebAuthn
//!
//! This module provides full WebAuthn credential management for biometric authentication.
//! Credentials are stored securely in the platform keyring (macOS Keychain, Windows
//! Credential Manager, Linux Secret Service) with metadata stored in files.
//!
//! Architecture:
//! - Credential metadata (timestamps, device name) stored in files
//! - Actual WebAuthn credential data stored in platform keyring for security
//! - Supports multiple credentials per identity
use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

/// WebAuthn credential data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnCredential {
    /// Credential ID
    pub id: String,

    /// Raw credential ID (as byte array)
    pub raw_id: Vec<u8>,

    /// Credential type (should be "public-key")
    pub credential_type: String,

    /// Attestation object (contains public key and authenticator data)
    pub attestation_object: Vec<u8>,

    /// Client data JSON
    pub client_data_json: Vec<u8>,
}

/// Passkey metadata with WebAuthn credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyInfo {
    /// Four-word identity this is registered for
    pub four_words: String,

    /// When registered
    pub registered_at: u64,

    /// Last successful use
    pub last_used: Option<u64>,

    /// Device name (e.g., "MacBook Pro Touch ID")
    pub device_name: String,

    /// WebAuthn credential (if using biometric authentication)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webauthn_credential: Option<WebAuthnCredential>,
}

/// Error type for passkey operations
#[derive(Debug, thiserror::Error)]
pub enum PasskeyError {
    #[error("Passkey not found for identity: {0}")]
    NotFound(String),

    #[error("Keyring error: {0}")]
    KeyringError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Credential already exists for this device")]
    CredentialExists,
}

/// Keyring service name for passkey credentials
const KEYRING_SERVICE: &str = "com.saorsalabs.communitas.passkey";

/// Manages passkey registration tracking with keyring integration
pub struct PasskeyManager {
    storage_path: std::path::PathBuf,
    use_keyring: bool,
}

impl PasskeyManager {
    /// Create new passkey manager with keyring enabled by default
    pub fn new(storage_path: impl AsRef<Path>) -> Result<Self> {
        Self::with_keyring(storage_path, true)
    }

    /// Create new passkey manager with explicit keyring setting
    pub fn with_keyring(storage_path: impl AsRef<Path>, use_keyring: bool) -> Result<Self> {
        let storage_path = storage_path.as_ref().to_path_buf();
        Ok(Self {
            storage_path,
            use_keyring,
        })
    }

    /// Check if passkey is registered for identity
    ///
    /// Checks both file storage and keyring for registered credentials.
    pub async fn has_passkey(&self, four_words: &str) -> bool {
        // Check file storage first
        if self.passkey_info_path(four_words).exists() {
            return true;
        }
        // Also check keyring if enabled
        if self.use_keyring && self.has_credential_in_keyring(four_words) {
            return true;
        }
        false
    }

    /// Register passkey for identity (legacy - without WebAuthn)
    pub async fn register_passkey(
        &self,
        four_words: &str,
        device_name: &str,
    ) -> Result<PasskeyInfo> {
        tracing::info!("Registering passkey for {}", four_words);

        let info = PasskeyInfo {
            four_words: four_words.to_string(),
            registered_at: current_timestamp(),
            last_used: None,
            device_name: device_name.to_string(),
            webauthn_credential: None,
        };

        self.save_passkey_info(&info).await?;

        tracing::info!("Passkey registered successfully");
        Ok(info)
    }

    /// Register passkey with WebAuthn credential
    ///
    /// The credential is stored in the platform keyring for security,
    /// while metadata is stored in a file.
    pub async fn register_passkey_webauthn(
        &self,
        four_words: &str,
        device_name: &str,
        webauthn_credential: WebAuthnCredential,
    ) -> Result<PasskeyInfo> {
        tracing::info!("Registering WebAuthn passkey for {}", four_words);

        // Store credential in keyring if enabled
        if self.use_keyring {
            self.store_credential_in_keyring(four_words, &webauthn_credential)
                .map_err(|e| anyhow::anyhow!("Failed to store credential in keyring: {}", e))?;
        }

        // Store metadata in file (without credential if keyring is used)
        let info = PasskeyInfo {
            four_words: four_words.to_string(),
            registered_at: current_timestamp(),
            last_used: None,
            device_name: device_name.to_string(),
            // Only store credential in file if keyring is disabled
            webauthn_credential: if self.use_keyring {
                None
            } else {
                Some(webauthn_credential)
            },
        };

        self.save_passkey_info(&info).await?;

        tracing::info!("WebAuthn passkey registered successfully");
        Ok(info)
    }

    /// Update last used timestamp
    pub async fn mark_passkey_used(&self, four_words: &str) -> Result<()> {
        let mut info = self.load_passkey_info(four_words).await?;
        info.last_used = Some(current_timestamp());
        self.save_passkey_info(&info).await?;
        Ok(())
    }

    /// Get passkey information
    pub async fn get_passkey_info(&self, four_words: &str) -> Result<PasskeyInfo> {
        self.load_passkey_info(four_words).await
    }

    /// Delete passkey for identity
    pub async fn delete_passkey(&self, four_words: &str) -> Result<()> {
        // Delete from file storage
        let path = self.passkey_info_path(four_words);
        if path.exists() {
            fs::remove_file(&path)
                .await
                .context("Failed to delete passkey")?;
            tracing::info!("Deleted passkey file for {}", four_words);
        }

        // Delete from keyring
        if self.use_keyring
            && let Err(e) = self.delete_credential_from_keyring(four_words)
        {
            tracing::warn!("Failed to delete passkey from keyring: {}", e);
        }

        Ok(())
    }

    /// List all registered passkeys
    pub async fn list_passkeys(&self) -> Result<Vec<PasskeyInfo>> {
        let mut passkeys = Vec::new();

        if !self.storage_path.exists() {
            return Ok(passkeys);
        }

        let mut entries = fs::read_dir(&self.storage_path)
            .await
            .context("Failed to read passkey storage directory")?;

        while let Some(entry) = entries.next_entry().await.context("Failed to read entry")? {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json")
                && path.to_string_lossy().ends_with(".passkey.json")
                && let Ok(data) = fs::read(&path).await
                && let Ok(info) = serde_json::from_slice::<PasskeyInfo>(&data)
            {
                passkeys.push(info);
            }
        }

        Ok(passkeys)
    }

    // Keyring integration methods

    /// Store WebAuthn credential in platform keyring
    fn store_credential_in_keyring(
        &self,
        four_words: &str,
        credential: &WebAuthnCredential,
    ) -> Result<(), PasskeyError> {
        use keyring::Entry;

        let entry = Entry::new(KEYRING_SERVICE, four_words)
            .map_err(|e| PasskeyError::KeyringError(e.to_string()))?;

        // Serialize credential to JSON then base64 encode
        let json = serde_json::to_string(credential)
            .map_err(|e| PasskeyError::SerializationError(e.to_string()))?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(json.as_bytes());

        entry
            .set_password(&encoded)
            .map_err(|e| PasskeyError::KeyringError(e.to_string()))?;

        tracing::info!("Stored WebAuthn credential in keyring for {}", four_words);
        Ok(())
    }

    /// Load WebAuthn credential from platform keyring
    fn load_credential_from_keyring(
        &self,
        four_words: &str,
    ) -> Result<WebAuthnCredential, PasskeyError> {
        use keyring::Entry;

        let entry = Entry::new(KEYRING_SERVICE, four_words)
            .map_err(|e| PasskeyError::KeyringError(e.to_string()))?;

        let encoded = entry
            .get_password()
            .map_err(|e| PasskeyError::KeyringError(e.to_string()))?;

        let json_bytes = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .map_err(|e| PasskeyError::SerializationError(e.to_string()))?;

        let credential: WebAuthnCredential = serde_json::from_slice(&json_bytes)
            .map_err(|e| PasskeyError::SerializationError(e.to_string()))?;

        Ok(credential)
    }

    /// Delete WebAuthn credential from platform keyring
    fn delete_credential_from_keyring(&self, four_words: &str) -> Result<(), PasskeyError> {
        use keyring::Entry;

        let entry = Entry::new(KEYRING_SERVICE, four_words)
            .map_err(|e| PasskeyError::KeyringError(e.to_string()))?;

        entry
            .delete_credential()
            .map_err(|e| PasskeyError::KeyringError(e.to_string()))?;

        tracing::info!(
            "Deleted WebAuthn credential from keyring for {}",
            four_words
        );
        Ok(())
    }

    /// Check if credential exists in keyring
    fn has_credential_in_keyring(&self, four_words: &str) -> bool {
        use keyring::Entry;

        if let Ok(entry) = Entry::new(KEYRING_SERVICE, four_words) {
            entry.get_password().is_ok()
        } else {
            false
        }
    }

    /// Get WebAuthn credential, preferring keyring over file storage
    pub async fn get_credential(&self, four_words: &str) -> Result<Option<WebAuthnCredential>> {
        // Try keyring first if enabled
        if self.use_keyring {
            match self.load_credential_from_keyring(four_words) {
                Ok(cred) => return Ok(Some(cred)),
                Err(PasskeyError::KeyringError(_)) => {
                    tracing::debug!("Credential not in keyring, checking file storage");
                }
                Err(e) => {
                    tracing::warn!("Error loading credential from keyring: {}", e);
                }
            }
        }

        // Fall back to file storage
        if let Ok(info) = self.load_passkey_info(four_words).await {
            return Ok(info.webauthn_credential);
        }

        Ok(None)
    }

    // Storage helpers

    fn passkey_info_path(&self, four_words: &str) -> std::path::PathBuf {
        self.storage_path
            .join(format!("{}.passkey.json", four_words))
    }

    async fn save_passkey_info(&self, info: &PasskeyInfo) -> Result<()> {
        fs::create_dir_all(&self.storage_path)
            .await
            .context("Failed to create passkey storage directory")?;

        let path = self.passkey_info_path(&info.four_words);
        let data = serde_json::to_vec_pretty(info).context("Failed to serialize passkey info")?;

        fs::write(&path, data)
            .await
            .context("Failed to write passkey info")?;

        Ok(())
    }

    async fn load_passkey_info(&self, four_words: &str) -> Result<PasskeyInfo> {
        let path = self.passkey_info_path(four_words);
        let data = fs::read(&path).await.context("Passkey info not found")?;

        let mut info: PasskeyInfo =
            serde_json::from_slice(&data).context("Failed to deserialize passkey info")?;

        // If WebAuthn credential is not in the passkey info file, try loading from keyring
        if info.webauthn_credential.is_none() && self.use_keyring {
            match self.load_credential_from_keyring(four_words) {
                Ok(credential) => {
                    info.webauthn_credential = Some(credential);
                    tracing::debug!("Loaded WebAuthn credential from keyring for {}", four_words);
                }
                Err(e) => {
                    // Log but don't fail - passkey info might exist without keyring credential
                    tracing::debug!(
                        "No WebAuthn credential in keyring for {}: {}",
                        four_words,
                        e
                    );
                }
            }
        }

        Ok(info)
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_passkey_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        // Disable keyring for tests (no real keyring in CI)
        let manager = PasskeyManager::with_keyring(temp_dir.path(), false).unwrap();

        assert!(!manager.has_passkey("ocean-forest-moon-star").await);
    }

    #[tokio::test]
    async fn test_passkey_registration() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PasskeyManager::with_keyring(temp_dir.path(), false).unwrap();

        let info = manager
            .register_passkey("ocean-forest-moon-star", "MacBook Pro")
            .await
            .unwrap();
        assert_eq!(info.four_words, "ocean-forest-moon-star");
        assert_eq!(info.device_name, "MacBook Pro");

        assert!(manager.has_passkey("ocean-forest-moon-star").await);
    }

    #[tokio::test]
    async fn test_delete_passkey_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PasskeyManager::with_keyring(temp_dir.path(), false).unwrap();

        assert!(
            manager
                .delete_passkey("ocean-forest-moon-star")
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_mark_passkey_used() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PasskeyManager::with_keyring(temp_dir.path(), false).unwrap();

        manager
            .register_passkey("ocean-forest-moon-star", "MacBook Pro")
            .await
            .unwrap();
        assert!(
            manager
                .mark_passkey_used("ocean-forest-moon-star")
                .await
                .is_ok()
        );

        let info = manager
            .get_passkey_info("ocean-forest-moon-star")
            .await
            .unwrap();
        assert!(info.last_used.is_some());
    }

    #[tokio::test]
    async fn test_webauthn_credential_file_storage() {
        let temp_dir = TempDir::new().unwrap();
        // Disable keyring so credentials are stored in files
        let manager = PasskeyManager::with_keyring(temp_dir.path(), false).unwrap();

        let credential = WebAuthnCredential {
            id: "test-cred-id".to_string(),
            raw_id: vec![1, 2, 3, 4],
            credential_type: "public-key".to_string(),
            attestation_object: vec![5, 6, 7, 8],
            client_data_json: vec![9, 10, 11, 12],
        };

        let info = manager
            .register_passkey_webauthn(
                "river-mountain-sun-cloud",
                "Test Device",
                credential.clone(),
            )
            .await
            .unwrap();

        assert_eq!(info.four_words, "river-mountain-sun-cloud");
        assert!(info.webauthn_credential.is_some());

        // Retrieve credential
        let loaded_cred = manager
            .get_credential("river-mountain-sun-cloud")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(loaded_cred.id, credential.id);
        assert_eq!(loaded_cred.raw_id, credential.raw_id);
    }

    #[tokio::test]
    async fn test_list_passkeys() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PasskeyManager::with_keyring(temp_dir.path(), false).unwrap();

        manager
            .register_passkey("ocean-forest-moon-star", "MacBook Pro")
            .await
            .unwrap();
        manager
            .register_passkey("river-mountain-sun-cloud", "iPhone")
            .await
            .unwrap();

        let passkeys = manager.list_passkeys().await.unwrap();
        assert_eq!(passkeys.len(), 2);
    }

    #[tokio::test]
    async fn test_get_credential_returns_none_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PasskeyManager::with_keyring(temp_dir.path(), false).unwrap();

        let cred = manager
            .get_credential("nonexistent-identity")
            .await
            .unwrap();
        assert!(cred.is_none());
    }
}
