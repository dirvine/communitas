// SPDX-License-Identifier: MIT OR Apache-2.0

//! Local Storage System for Communitas
//!
//! This module implements a multi-identity local storage system that:
//! - Supports multiple accounts per device with secure switching
//! - Provides Forward Error Correction via Reed-Solomon for resilience
//! - Integrates with platform-specific storage paths
//!
//! Architecture:
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                  User Authentication                    │
//! │         (Password / Four-Word Address)                │
//! └──────────────────────┬──────────────────────────────────┘
//!                        ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │              Key Derivation Layer (PBKDF2)             │
//! │            100,000 iterations, per-vault salt          │
//! └──────────────────────┬──────────────────────────────────┘
//!                        ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │            Encryption Layer (ChaCha20-Poly1305)        │
//! │              Per-file IV, authenticated encryption     │
//! └──────────────────────┬──────────────────────────────────┘
//!                        ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │          Forward Error Correction (Reed-Solomon)       │
//! │            Data sharding with redundancy               │
//! └──────────────────────┬──────────────────────────────────┘
//!                        ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │               Platform Storage Layer                    │
//! │     (macOS Keychain / Windows DPAPI / Linux Secret)    │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod app_config;
pub mod fec_storage;
pub mod identity_store;
pub mod key_management;
pub mod platform_storage;
pub mod session;
pub mod vault;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub use app_config::*;
pub use fec_storage::*;
pub use identity_store::*;
pub use key_management::*;
pub use platform_storage::*;
pub use session::*;
pub use vault::*;

/// Configuration for the encrypted storage system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base directory for encrypted vaults
    pub vault_dir: PathBuf,

    /// PBKDF2 iteration count (default: 100,000 as per DESIGN.md)
    pub pbkdf2_iterations: u32,

    /// Enable Forward Error Correction for stored files
    pub enable_fec: bool,

    /// FEC redundancy factor (e.g., 1.5 = 50% redundancy)
    pub fec_redundancy: f32,

    /// Maximum vault size in bytes (0 = unlimited)
    pub max_vault_size: u64,

    /// Enable platform keyring integration
    pub use_keyring: bool,

    /// Cache timeout for decrypted data (seconds)
    pub cache_timeout: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            vault_dir: get_vault_directory(),
            pbkdf2_iterations: 100_000, // As specified in DESIGN.md
            enable_fec: true,
            fec_redundancy: 1.5,
            max_vault_size: 0, // Unlimited
            use_keyring: true,
            cache_timeout: 300, // 5 minutes
        }
    }
}

/// Main encrypted storage manager
pub struct EncryptedStorageManager {
    config: StorageConfig,
    vaults: Arc<RwLock<HashMap<String, Arc<EncryptedVault>>>>,
    active_sessions: Arc<RwLock<HashMap<String, Session>>>,
    platform_storage: Arc<PlatformStorage>,
    app_config: Arc<RwLock<AppConfigManager>>,
}

impl EncryptedStorageManager {
    /// Create a new encrypted storage manager
    pub async fn new(config: StorageConfig) -> Result<Self> {
        // Initialize platform-specific storage
        let platform_storage = Arc::new(
            PlatformStorage::new(&config.vault_dir)
                .context("Failed to initialize platform storage")?,
        );

        // Initialize app config manager (config stored in parent of vault_dir)
        let config_dir = config
            .vault_dir
            .parent()
            .unwrap_or(&config.vault_dir)
            .to_path_buf();
        let app_config = Arc::new(RwLock::new(
            AppConfigManager::new(config_dir.clone())
                .await
                .context("Failed to initialize app config")?,
        ));

        Ok(Self {
            config,
            vaults: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            platform_storage,
            app_config,
        })
    }

    /// Get the vault directory path
    pub fn vault_dir(&self) -> &Path {
        &self.config.vault_dir
    }

    /// Create a new vault for a four-word identity
    pub async fn create_vault(&self, four_words: &str, display_name: &str) -> Result<String> {
        // Validate four-word address
        let normalized = self.normalize_four_words(four_words);

        // Check if vault already exists
        if self.vault_exists(&normalized).await? {
            return Err(anyhow::anyhow!("Vault already exists for {}", four_words));
        }

        // Create vault
        let vault =
            EncryptedVault::create(normalized.clone(), display_name.to_string(), &self.config)
                .await
                .context("Failed to create vault")?;

        // Store vault
        let mut vaults = self.vaults.write().await;
        vaults.insert(normalized.clone(), Arc::new(vault));

        Ok(normalized)
    }

    /// Login with four-word address
    pub async fn login(&self, four_words: &str) -> Result<Session> {
        let normalized = self.normalize_four_words(four_words);

        // Load or open vault
        let vault = self.load_vault(&normalized).await?;

        // Create session
        let session = Session::new(
            normalized.clone(),
            vault.display_name.clone(),
            self.config.cache_timeout,
        );

        // Store active session
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());

        // Update app config with last used identity
        let mut app_config = self.app_config.write().await;
        app_config
            .set_last_identity(normalized.clone(), vault.display_name.clone())
            .await
            .ok(); // Non-fatal if config update fails

        Ok(session)
    }

    /// Store encrypted data in a vault
    pub async fn store(
        &self,
        session_id: &str,
        key: &str,
        data: &[u8],
        use_fec: bool,
    ) -> Result<()> {
        // Validate session
        let session = self.validate_session(session_id).await?;

        // Get vault
        let vaults = self.vaults.read().await;
        let vault = vaults
            .get(&session.four_words)
            .ok_or_else(|| anyhow::anyhow!("Vault not loaded"))?;

        // Store data
        if use_fec && self.config.enable_fec {
            // Use Forward Error Correction for important data
            vault
                .store_with_fec(key, data, self.config.fec_redundancy)
                .await
        } else {
            // Simple encrypted storage
            vault.store(key, data).await
        }
    }

    /// Retrieve encrypted data from a vault
    pub async fn retrieve(&self, session_id: &str, key: &str) -> Result<Vec<u8>> {
        // Validate session
        let session = self.validate_session(session_id).await?;

        // Get vault
        let vaults = self.vaults.read().await;
        let vault = vaults
            .get(&session.four_words)
            .ok_or_else(|| anyhow::anyhow!("Vault not loaded"))?;

        // Retrieve data
        vault.retrieve(key).await
    }

    /// List all available vaults on this device
    pub async fn list_vaults(&self) -> Result<Vec<VaultInfo>> {
        self.platform_storage.list_vaults().await
    }

    /// Switch to a different account
    pub async fn switch_account(&self, session_id: &str, four_words: &str) -> Result<Session> {
        // End current session
        self.logout(session_id).await?;

        // Try switch if vault is cached
        if let Some(vault) = self.vaults.read().await.get(four_words) {
            // Create new session for cached vault
            let session = Session::new(
                four_words.to_string(),
                vault.display_name.clone(),
                self.config.cache_timeout,
            );

            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session.id.clone(), session.clone());

            return Ok(session);
        }

        // Load vault from disk
        self.login(four_words).await
    }

    /// Logout and clear session
    pub async fn logout(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    /// Export vault for backup
    pub async fn export_vault(&self, session_id: &str, include_data: bool) -> Result<Vec<u8>> {
        let session = self.validate_session(session_id).await?;

        let vaults = self.vaults.read().await;
        let vault = vaults
            .get(&session.four_words)
            .ok_or_else(|| anyhow::anyhow!("Vault not loaded"))?;

        vault.export(include_data).await
    }

    /// Import vault from backup
    pub async fn import_vault(&self, backup_data: &[u8]) -> Result<String> {
        let vault = EncryptedVault::import(backup_data, &self.config).await?;
        let four_words = vault.four_words.clone();

        let mut vaults = self.vaults.write().await;
        vaults.insert(four_words.clone(), Arc::new(vault));

        Ok(four_words)
    }

    /// Get app configuration
    pub async fn get_app_config(&self) -> AppConfig {
        self.app_config.read().await.get_config().clone()
    }

    /// Try auto-login with last used identity
    pub async fn try_auto_login(&self) -> Result<Option<Session>> {
        let app_config = self.app_config.read().await;
        let config = app_config.get_config();
        let mut candidate = config.last_identity.clone();

        drop(app_config); // Release lock

        if candidate.is_none() {
            let vaults = self.list_vaults().await?;
            if let Some(most_recent) = vaults.iter().max_by_key(|vault| vault.last_accessed) {
                candidate = Some(most_recent.four_words.clone());
            }
        }

        let Some(four_words) = candidate else {
            return Ok(None);
        };

        self.login(&four_words).await.map(Some)
    }

    /// Update app configuration setting
    pub async fn set_auto_login_enabled(&self, enabled: bool) -> Result<()> {
        let mut app_config = self.app_config.write().await;
        app_config.set_auto_login(enabled).await
    }

    /// Update keyring setting
    pub async fn set_keyring_enabled(&self, enabled: bool) -> Result<()> {
        let mut app_config = self.app_config.write().await;
        app_config.set_keyring_enabled(enabled).await
    }

    /// Get recent identities
    pub async fn get_recent_identities(&self) -> Vec<RecentIdentity> {
        self.app_config
            .read()
            .await
            .get_config()
            .recent_identities
            .clone()
    }

    /// Remove a recent identity from the list (does not delete the vault)
    pub async fn remove_recent_identity(&self, four_words: &str) -> Result<()> {
        self.app_config
            .write()
            .await
            .remove_recent_identity(four_words)
            .await
    }

    /// Delete a vault permanently
    ///
    /// WARNING: This permanently deletes all encrypted data for this identity.
    pub async fn delete_vault(&self, four_words: &str) -> Result<()> {
        let normalized = self.normalize_four_words(four_words);

        // Remove from cache
        let mut vaults = self.vaults.write().await;
        vaults.remove(&normalized);
        drop(vaults);

        // Delete vault directory from filesystem
        let vault_path = self.config.vault_dir.join(&normalized);
        if vault_path.exists() {
            tokio::fs::remove_dir_all(&vault_path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to delete vault directory: {}", e))?;
        }

        // Remove from app config recent identities
        let mut app_config = self.app_config.write().await;
        app_config.remove_recent_identity(&normalized).await?;

        Ok(())
    }

    // Helper methods

    fn normalize_four_words(&self, four_words: &str) -> String {
        four_words.trim().to_lowercase().replace([' ', '_'], "-")
    }

    pub async fn vault_exists(&self, four_words: &str) -> Result<bool> {
        self.platform_storage.vault_exists(four_words).await
    }

    async fn load_vault(&self, four_words: &str) -> Result<Arc<EncryptedVault>> {
        // Load from disk
        let vault = EncryptedVault::load(four_words, &self.config).await?;

        // Cache only AFTER successful load
        let mut vaults = self.vaults.write().await;
        let vault_arc = Arc::new(vault);
        vaults.insert(four_words.to_string(), vault_arc.clone());

        Ok(vault_arc)
    }

    async fn validate_session(&self, session_id: &str) -> Result<Session> {
        let sessions = self.active_sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired session"))?;

        if session.is_expired() {
            return Err(anyhow::anyhow!("Session expired"));
        }

        Ok(session.clone())
    }
}

/// Get platform-specific vault directory
fn get_vault_directory() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Library")
            .join("Application Support")
            .join("com.saorsalabs.communitas")
            .join("vaults")
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
            .join("communitas")
            .join("vaults")
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("communitas")
            .join("vaults")
    }

    #[cfg(target_os = "ios")]
    {
        // On iOS, use the app's document directory
        dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("/var/mobile/Containers/Data/Application"))
            .join("communitas")
            .join("vaults")
    }

    #[cfg(not(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "ios"
    )))]
    {
        // Fallback for other platforms
        PathBuf::from("/tmp").join("communitas").join("vaults")
    }
}

/// Vault metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub four_words: String,
    pub display_name: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_vault_creation_and_login() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            vault_dir: temp_dir.path().to_path_buf(),
            use_keyring: false, // Disable for tests
            ..Default::default()
        };

        let manager = EncryptedStorageManager::new(config).await.unwrap();

        // Create vault
        let four_words = manager
            .create_vault("ocean-forest-moon-star", "Alice")
            .await
            .unwrap();

        assert_eq!(four_words, "ocean-forest-moon-star");

        // Login with four-words
        let session = manager.login("ocean-forest-moon-star").await.unwrap();

        assert_eq!(session.four_words, "ocean-forest-moon-star");

        // Store and retrieve data
        let test_data = b"Hello, encrypted world!";
        manager
            .store(&session.id, "test_key", test_data, false)
            .await
            .unwrap();

        let retrieved = manager.retrieve(&session.id, "test_key").await.unwrap();
        assert_eq!(retrieved, test_data);
    }

    #[tokio::test]
    async fn test_fec_storage() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            vault_dir: temp_dir.path().to_path_buf(),
            use_keyring: false,
            enable_fec: true,
            fec_redundancy: 2.0, // 100% redundancy
            ..Default::default()
        };

        let manager = EncryptedStorageManager::new(config).await.unwrap();

        // Create vault and login
        manager
            .create_vault("test-fec-vault-storage", "FEC Test")
            .await
            .unwrap();

        let session = manager.login("test-fec-vault-storage").await.unwrap();

        // Store large data with FEC
        let large_data = vec![42u8; 128 * 1024]; // 128KB
        manager
            .store(&session.id, "large_file", &large_data, true)
            .await
            .unwrap();

        // Simulate a missing shard and ensure recovery still succeeds
        let shard_path = temp_dir
            .path()
            .join("test-fec-vault-storage")
            .join("fec")
            .join("large_file")
            .join("shard_0.bin");
        if shard_path.exists() {
            tokio::fs::remove_file(shard_path).await.unwrap();
        }

        // Retrieve should work even if some shards are corrupted
        let retrieved = manager.retrieve(&session.id, "large_file").await.unwrap();
        assert_eq!(retrieved, large_data);
    }
}
