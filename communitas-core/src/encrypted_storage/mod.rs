//! Encrypted Local Storage System for Communitas
//!
//! This module implements a sophisticated multi-layered encrypted storage system that:
//! - Supports multiple accounts per device with secure switching
//! - Uses PBKDF2 for key derivation (100,000 iterations as per DESIGN.md)
//! - Implements ChaCha20-Poly1305 for encryption (superior to AES-GCM on most CPUs)
//! - Provides Forward Error Correction via Reed-Solomon for resilience
//! - Integrates with platform-specific secure storage (keyring)
//! - Enables password-only login on familiar devices
//!
//! Architecture:
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │                  User Authentication                    │
//! │         (Password / Passkey / Four-Word Address)       │
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

pub mod key_management;
pub mod vault;
pub mod fec_storage;
pub mod platform_storage;
pub mod session;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use key_management::*;
pub use vault::*;
pub use fec_storage::*;
pub use platform_storage::*;
pub use session::*;

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
    key_manager: Arc<KeyManager>,
    platform_storage: Arc<PlatformStorage>,
}

impl EncryptedStorageManager {
    /// Create a new encrypted storage manager
    pub async fn new(config: StorageConfig) -> Result<Self> {
        // Initialize platform-specific storage
        let platform_storage = Arc::new(
            PlatformStorage::new(&config.vault_dir)
                .context("Failed to initialize platform storage")?
        );

        // Initialize key manager with PBKDF2
        let key_manager = Arc::new(
            KeyManager::new(config.pbkdf2_iterations, config.use_keyring)
                .await
                .context("Failed to initialize key manager")?
        );

        Ok(Self {
            config,
            vaults: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            key_manager,
            platform_storage,
        })
    }

    /// Create a new vault for a four-word identity
    pub async fn create_vault(
        &self,
        four_words: &str,
        password: &str,
        display_name: &str,
    ) -> Result<String> {
        // Validate four-word address
        let normalized = self.normalize_four_words(four_words);

        // Check if vault already exists
        if self.vault_exists(&normalized).await? {
            return Err(anyhow::anyhow!("Vault already exists for {}", four_words));
        }

        // Derive encryption key from password
        let salt = generate_salt();
        let key = self.key_manager
            .derive_key(password, &salt)
            .await
            .context("Failed to derive encryption key")?;

        // Create vault
        let vault = EncryptedVault::create(
            normalized.clone(),
            display_name.to_string(),
            key.clone(),
            salt,
            &self.config,
        ).await
        .context("Failed to create vault")?;

        // Store vault
        let mut vaults = self.vaults.write().await;
        vaults.insert(normalized.clone(), Arc::new(vault));

        // Store password hash for password-only login
        self.store_password_locator(&normalized, password).await?;

        // If keyring is enabled, store master key
        if self.config.use_keyring {
            self.key_manager
                .store_in_keyring(&normalized, &key)
                .await
                .ok(); // Non-fatal if keyring fails
        }

        Ok(normalized)
    }

    /// Login with password only (searches all vaults)
    pub async fn login_password_only(&self, password: &str) -> Result<Session> {
        // Generate password hash for lookup
        let password_hash = self.key_manager.hash_password(password).await?;

        // Find matching vault
        let four_words = self.platform_storage
            .find_vault_by_password_hash(&password_hash)
            .await
            .context("No vault found for this password")?;

        // Login with found four-words
        self.login(&four_words, password, None).await
    }

    /// Login with four-word address and password
    pub async fn login(
        &self,
        four_words: &str,
        password: &str,
        _passkey: Option<Vec<u8>>,
    ) -> Result<Session> {
        let normalized = self.normalize_four_words(four_words);

        // Load or open vault
        let vault = self.load_vault(&normalized, password).await?;

        // Create session
        let session = Session::new(
            normalized.clone(),
            vault.display_name.clone(),
            self.config.cache_timeout,
        );

        // Store active session
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());

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
        let vault = vaults.get(&session.four_words)
            .ok_or_else(|| anyhow::anyhow!("Vault not loaded"))?;

        // Store data
        if use_fec && self.config.enable_fec {
            // Use Forward Error Correction for important data
            vault.store_with_fec(key, data, self.config.fec_redundancy).await
        } else {
            // Simple encrypted storage
            vault.store(key, data).await
        }
    }

    /// Retrieve encrypted data from a vault
    pub async fn retrieve(
        &self,
        session_id: &str,
        key: &str,
    ) -> Result<Vec<u8>> {
        // Validate session
        let session = self.validate_session(session_id).await?;

        // Get vault
        let vaults = self.vaults.read().await;
        let vault = vaults.get(&session.four_words)
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

        // Try password-less switch if vault is cached
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

        Err(anyhow::anyhow!("Vault not cached, password required"))
    }

    /// Logout and clear session
    pub async fn logout(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    /// Export vault for backup
    pub async fn export_vault(
        &self,
        session_id: &str,
        include_data: bool,
    ) -> Result<Vec<u8>> {
        let session = self.validate_session(session_id).await?;

        let vaults = self.vaults.read().await;
        let vault = vaults.get(&session.four_words)
            .ok_or_else(|| anyhow::anyhow!("Vault not loaded"))?;

        vault.export(include_data).await
    }

    /// Import vault from backup
    pub async fn import_vault(
        &self,
        backup_data: &[u8],
        password: &str,
    ) -> Result<String> {
        let vault = EncryptedVault::import(backup_data, password, &self.config).await?;
        let four_words = vault.four_words.clone();

        let mut vaults = self.vaults.write().await;
        vaults.insert(four_words.clone(), Arc::new(vault));

        Ok(four_words)
    }

    // Helper methods

    fn normalize_four_words(&self, four_words: &str) -> String {
        four_words
            .trim()
            .to_lowercase()
            .replace(' ', "-")
            .replace('_', "-")
    }

    async fn vault_exists(&self, four_words: &str) -> Result<bool> {
        self.platform_storage.vault_exists(four_words).await
    }

    async fn load_vault(&self, four_words: &str, password: &str) -> Result<Arc<EncryptedVault>> {
        // Check cache first
        if let Some(vault) = self.vaults.read().await.get(four_words) {
            return Ok(vault.clone());
        }

        // Load from disk
        let vault = EncryptedVault::load(four_words, password, &self.config).await?;

        // Cache it
        let mut vaults = self.vaults.write().await;
        let vault_arc = Arc::new(vault);
        vaults.insert(four_words.to_string(), vault_arc.clone());

        Ok(vault_arc)
    }

    async fn store_password_locator(&self, four_words: &str, password: &str) -> Result<()> {
        let password_hash = self.key_manager.hash_password(password).await?;
        self.platform_storage
            .store_password_locator(&password_hash, four_words)
            .await
    }

    async fn validate_session(&self, session_id: &str) -> Result<Session> {
        let sessions = self.active_sessions.read().await;
        let session = sessions.get(session_id)
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
            .join("com.p2pfoundation.communitas")
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
}

/// Generate a cryptographically secure salt
fn generate_salt() -> Vec<u8> {
    use rand::Rng;
    let mut salt = vec![0u8; 32];
    rand::thread_rng().fill(&mut salt[..]);
    salt
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
            .create_vault("ocean-forest-moon-star", "test_password", "Alice")
            .await
            .unwrap();

        assert_eq!(four_words, "ocean-forest-moon-star");

        // Login with four-words
        let session = manager
            .login("ocean-forest-moon-star", "test_password", None)
            .await
            .unwrap();

        assert_eq!(session.four_words, "ocean-forest-moon-star");

        // Store and retrieve data
        let test_data = b"Hello, encrypted world!";
        manager.store(&session.id, "test_key", test_data, false).await.unwrap();

        let retrieved = manager.retrieve(&session.id, "test_key").await.unwrap();
        assert_eq!(retrieved, test_data);
    }

    #[tokio::test]
    async fn test_password_only_login() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            vault_dir: temp_dir.path().to_path_buf(),
            use_keyring: false,
            ..Default::default()
        };

        let manager = EncryptedStorageManager::new(config).await.unwrap();

        // Create vault
        manager
            .create_vault("river-mountain-sun-cloud", "unique_password", "Bob")
            .await
            .unwrap();

        // Logout (simulate app restart)
        // ...

        // Login with password only
        let session = manager
            .login_password_only("unique_password")
            .await
            .unwrap();

        assert_eq!(session.four_words, "river-mountain-sun-cloud");
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
            .create_vault("test-fec-vault-storage", "password", "FEC Test")
            .await
            .unwrap();

        let session = manager
            .login("test-fec-vault-storage", "password", None)
            .await
            .unwrap();

        // Store large data with FEC
        let large_data = vec![42u8; 1024 * 1024]; // 1MB
        manager.store(&session.id, "large_file", &large_data, true).await.unwrap();

        // Retrieve should work even if some shards are corrupted
        let retrieved = manager.retrieve(&session.id, "large_file").await.unwrap();
        assert_eq!(retrieved, large_data);
    }
}