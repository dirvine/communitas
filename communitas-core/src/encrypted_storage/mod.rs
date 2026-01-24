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
//! ```text
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

pub mod app_config;
pub mod fec_storage;
pub mod key_management;
pub mod passkey;
pub mod platform_storage;
pub mod session;
pub mod vault;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use app_config::*;
pub use fec_storage::*;
pub use key_management::*;
pub use passkey::*;
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
    key_manager: Arc<KeyManager>,
    platform_storage: Arc<PlatformStorage>,
    app_config: Arc<RwLock<AppConfigManager>>,
    passkey_manager: Arc<PasskeyManager>,
}

impl EncryptedStorageManager {
    /// Create a new encrypted storage manager
    pub async fn new(config: StorageConfig) -> Result<Self> {
        // Initialize platform-specific storage
        let platform_storage = Arc::new(
            PlatformStorage::new(&config.vault_dir)
                .context("Failed to initialize platform storage")?,
        );

        // Initialize key manager with PBKDF2
        let key_manager = Arc::new(
            KeyManager::new(config.pbkdf2_iterations, config.use_keyring)
                .await
                .context("Failed to initialize key manager")?,
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

        // Initialize passkey manager (passkeys stored in config dir)
        let passkey_storage_dir = config_dir.join("passkeys");
        let passkey_manager = Arc::new(
            PasskeyManager::new(&passkey_storage_dir)
                .context("Failed to initialize passkey manager")?,
        );

        Ok(Self {
            config,
            vaults: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            key_manager,
            platform_storage,
            app_config,
            passkey_manager,
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
        let key = self
            .key_manager
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
        )
        .await
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
        let four_words = self
            .platform_storage
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

        // Update app config with last used identity
        let mut app_config = self.app_config.write().await;
        app_config
            .set_last_identity(normalized.clone(), vault.display_name.clone())
            .await
            .ok(); // Non-fatal if config update fails

        // Store password in keyring if enabled
        if self.config.use_keyring && app_config.get_config().keyring_enabled {
            tracing::info!(
                "🔑 LOGIN: Attempting to store password in keyring for '{}'",
                normalized
            );
            match self
                .key_manager
                .store_in_keyring(&normalized, password.as_bytes())
                .await
            {
                Ok(()) => {
                    tracing::info!(
                        "✅ LOGIN: Password stored in keyring successfully for '{}'",
                        normalized
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "❌ LOGIN: Failed to store password in keyring for '{}': {}",
                        normalized,
                        e
                    );
                    tracing::error!(
                        "⚠️ LOGIN: This means passkey/Touch ID authentication will fail later!"
                    );
                }
            }
        } else {
            tracing::warn!(
                "⚠️ LOGIN: Keyring storage skipped - use_keyring={}, keyring_enabled={}",
                self.config.use_keyring,
                app_config.get_config().keyring_enabled
            );
        }

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
    pub async fn export_vault(&self, session_id: &str, include_data: bool) -> Result<Vec<u8>> {
        let session = self.validate_session(session_id).await?;

        let vaults = self.vaults.read().await;
        let vault = vaults
            .get(&session.four_words)
            .ok_or_else(|| anyhow::anyhow!("Vault not loaded"))?;

        vault.export(include_data).await
    }

    /// Import vault from backup
    pub async fn import_vault(&self, backup_data: &[u8], password: &str) -> Result<String> {
        let vault = EncryptedVault::import(backup_data, password, &self.config).await?;
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

        // Check if auto-login is enabled
        if !config.auto_login_enabled {
            return Ok(None);
        }

        // Get last identity
        let four_words = match &config.last_identity {
            Some(fw) => fw.clone(),
            None => return Ok(None),
        };

        drop(app_config); // Release lock

        // Try to get password from keyring
        if self.config.use_keyring
            && let Ok(password_bytes) = self.key_manager.get_from_keyring(&four_words).await
            && let Ok(password) = String::from_utf8(password_bytes.to_vec())
        {
            // Attempt login
            match self.login(&four_words, &password, None).await {
                Ok(session) => return Ok(Some(session)),
                Err(_) => {
                    // Login failed - possibly password changed
                    // Remove from keyring
                    self.key_manager.delete_from_keyring(&four_words).await.ok();
                }
            }
        }

        Ok(None)
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

    // Passkey (biometric) methods

    /// Register passkey/biometric for an identity
    pub async fn passkey_register(
        &self,
        four_words: &str,
        device_name: &str,
    ) -> Result<PasskeyInfo> {
        let normalized = self.normalize_four_words(four_words);

        // Register passkey
        let info = self
            .passkey_manager
            .register_passkey(&normalized, device_name)
            .await?;

        // Update app config to mark passkey as available
        let mut app_config = self.app_config.write().await;
        app_config
            .set_identity_has_passkey(&normalized, true)
            .await?;

        Ok(info)
    }

    /// Register passkey with WebAuthn credential
    ///
    /// This stores the WebAuthn credential for true biometric authentication.
    pub async fn passkey_register_webauthn(
        &self,
        four_words: &str,
        device_name: &str,
        credential: WebAuthnCredential,
    ) -> Result<PasskeyInfo> {
        let normalized = self.normalize_four_words(four_words);

        // Register passkey with WebAuthn credential
        let info = self
            .passkey_manager
            .register_passkey_webauthn(&normalized, device_name, credential)
            .await?;

        // Update app config to mark passkey as available
        let mut app_config = self.app_config.write().await;
        app_config
            .set_identity_has_passkey(&normalized, true)
            .await?;

        Ok(info)
    }

    /// Authenticate with passkey/biometric and create session
    ///
    /// This uses the password stored in keyring after biometric verification
    pub async fn passkey_authenticate(&self, four_words: &str) -> Result<Session> {
        let normalized = self.normalize_four_words(four_words);

        tracing::info!(
            "🔍 RETRIEVAL: Attempting passkey auth for four_words='{}' -> normalized='{}'",
            four_words,
            normalized
        );

        // Check passkey is registered
        if !self.passkey_manager.has_passkey(&normalized).await {
            tracing::error!("❌ RETRIEVAL: No passkey registered for '{}'", normalized);
            anyhow::bail!("No passkey registered for this identity");
        }

        tracing::info!("✅ RETRIEVAL: Passkey IS registered for '{}'", normalized);

        // Mark passkey as used
        self.passkey_manager
            .mark_passkey_used(&normalized)
            .await
            .ok();

        // Load vault using password from keyring
        tracing::info!(
            "🔍 RETRIEVAL: Checking keyring config - use_keyring={}",
            self.config.use_keyring
        );

        if self.config.use_keyring {
            tracing::info!(
                "🔍 RETRIEVAL: Attempting to get password from keyring for '{}'",
                normalized
            );

            match self.key_manager.get_from_keyring(&normalized).await {
                Ok(password_bytes) => {
                    tracing::info!(
                        "✅ RETRIEVAL: Password bytes retrieved from keyring for '{}'",
                        normalized
                    );

                    match String::from_utf8(password_bytes.to_vec()) {
                        Ok(password) => {
                            tracing::info!(
                                "✅ RETRIEVAL: Password successfully decoded, attempting login for '{}'",
                                normalized
                            );
                            // Login with stored password
                            return self.login(&normalized, &password, None).await;
                        }
                        Err(e) => {
                            tracing::error!(
                                "❌ RETRIEVAL: Failed to decode password bytes for '{}': {}",
                                normalized,
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "❌ RETRIEVAL: Failed to get password from keyring for '{}': {}",
                        normalized,
                        e
                    );
                }
            }
        } else {
            tracing::warn!("⚠️ RETRIEVAL: Keyring is disabled in config");
        }

        // If no password in keyring, cannot proceed
        tracing::error!(
            "❌ RETRIEVAL: No password found in keyring for '{}'",
            normalized
        );
        anyhow::bail!(
            "Passkey registered but vault password not found in keyring. Please login with password first."
        )
    }

    /// Check if passkey is registered for identity
    pub async fn passkey_has_passkey(&self, four_words: &str) -> bool {
        let normalized = self.normalize_four_words(four_words);
        self.passkey_manager.has_passkey(&normalized).await
    }

    /// Get passkey information
    pub async fn passkey_get_info(&self, four_words: &str) -> Result<PasskeyInfo> {
        let normalized = self.normalize_four_words(four_words);
        self.passkey_manager.get_passkey_info(&normalized).await
    }

    /// Delete passkey for identity
    pub async fn passkey_delete(&self, four_words: &str) -> Result<()> {
        let normalized = self.normalize_four_words(four_words);
        self.passkey_manager.delete_passkey(&normalized).await?;

        // Update app config
        let mut app_config = self.app_config.write().await;
        app_config
            .set_identity_has_passkey(&normalized, false)
            .await?;

        Ok(())
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

        // Remove password from keyring if exists
        self.key_manager.delete_from_keyring(&normalized).await.ok();

        // Remove passkey if exists
        self.passkey_manager.delete_passkey(&normalized).await.ok();

        // Remove from app config recent identities
        let mut app_config = self.app_config.write().await;
        app_config.remove_recent_identity(&normalized).await?;

        Ok(())
    }

    /// Store password in platform keyring (for auto-login)
    pub async fn store_password_in_keyring(&self, four_words: &str, password: &str) -> Result<()> {
        let normalized = self.normalize_four_words(four_words);
        tracing::info!(
            "🔑 STORAGE: Storing password in keyring for four_words='{}' -> normalized='{}'",
            four_words,
            normalized
        );
        let result = self
            .key_manager
            .store_in_keyring(&normalized, password.as_bytes())
            .await;

        if result.is_ok() {
            tracing::info!(
                "✅ STORAGE: Password successfully stored in keyring for '{}'",
                normalized
            );
        } else {
            tracing::error!(
                "❌ STORAGE: Failed to store password in keyring for '{}': {:?}",
                normalized,
                result
            );
        }

        result
    }

    /// Remove password from platform keyring
    pub async fn remove_password_from_keyring(&self, four_words: &str) -> Result<()> {
        let normalized = self.normalize_four_words(four_words);
        self.key_manager.delete_from_keyring(&normalized).await
    }

    // Helper methods

    fn normalize_four_words(&self, four_words: &str) -> String {
        four_words.trim().to_lowercase().replace([' ', '_'], "-")
    }

    pub async fn vault_exists(&self, four_words: &str) -> Result<bool> {
        self.platform_storage.vault_exists(four_words).await
    }

    async fn load_vault(&self, four_words: &str, password: &str) -> Result<Arc<EncryptedVault>> {
        // 🔒 SECURITY FIX: Always validate password by loading vault from disk
        // Never trust cached vaults during authentication - they bypass password validation
        // ChaCha20-Poly1305 AEAD will automatically fail decryption with wrong password

        // Load from disk (this validates password via AEAD decryption)
        let vault = EncryptedVault::load(four_words, password, &self.config).await?;

        // Cache only AFTER successful password validation
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

/// Generate a cryptographically secure salt
fn generate_salt() -> Vec<u8> {
    use rand::{Rng, SeedableRng};
    let mut salt = vec![0u8; 32];
    rand::rngs::StdRng::from_entropy().fill(&mut salt[..]);
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
        manager
            .store(&session.id, "test_key", test_data, false)
            .await
            .unwrap();

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
