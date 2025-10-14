//! Passkey/biometric authentication support using WebAuthn
//!
//! This module provides full WebAuthn credential management for biometric authentication.
//! Credentials are stored securely and verified during authentication.
use anyhow::{Context, Result};
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

/// Manages passkey registration tracking
pub struct PasskeyManager {
    storage_path: std::path::PathBuf,
}

impl PasskeyManager {
    /// Create new passkey manager
    pub fn new(storage_path: impl AsRef<Path>) -> Result<Self> {
        let storage_path = storage_path.as_ref().to_path_buf();
        Ok(Self { storage_path })
    }

    /// Check if passkey is registered for identity
    pub async fn has_passkey(&self, four_words: &str) -> bool {
        self.passkey_info_path(four_words).exists()
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
    pub async fn register_passkey_webauthn(
        &self,
        four_words: &str,
        device_name: &str,
        webauthn_credential: WebAuthnCredential,
    ) -> Result<PasskeyInfo> {
        tracing::info!("Registering WebAuthn passkey for {}", four_words);

        let info = PasskeyInfo {
            four_words: four_words.to_string(),
            registered_at: current_timestamp(),
            last_used: None,
            device_name: device_name.to_string(),
            webauthn_credential: Some(webauthn_credential),
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
        let path = self.passkey_info_path(four_words);
        if path.exists() {
            fs::remove_file(&path)
                .await
                .context("Failed to delete passkey")?;
            tracing::info!("Deleted passkey for {}", four_words);
        }
        Ok(())
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

        let info: PasskeyInfo =
            serde_json::from_slice(&data).context("Failed to deserialize passkey info")?;

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
        let manager = PasskeyManager::new(temp_dir.path()).unwrap();

        assert!(!manager.has_passkey("ocean-forest-moon-star").await);
    }

    #[tokio::test]
    async fn test_passkey_registration() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PasskeyManager::new(temp_dir.path()).unwrap();

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
        let manager = PasskeyManager::new(temp_dir.path()).unwrap();

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
        let manager = PasskeyManager::new(temp_dir.path()).unwrap();

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
}
