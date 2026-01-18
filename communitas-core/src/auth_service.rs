//! Shared Authentication Service
//!
//! This module provides a unified authentication service that can be used by all frontends
//! (Dioxus UI, CLI, MCP, headless, etc.). It encapsulates all business logic for:
//! - Multi-identity management
//! - Vault creation and authentication
//! - Passkey/biometric support
//! - Session management
//! - Auto-login functionality

use crate::encrypted_storage::{
    EncryptedStorageManager, PasskeyInfo, RecentIdentity, Session, VaultInfo,
};
use crate::keystore::Keystore;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Session information for active authenticated user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub four_words: String,
    pub display_name: String,
    /// Hex-encoded ML-DSA-87 public key (the user's cryptographic identity)
    pub pubkey_hex: String,
}

impl From<Session> for SessionInfo {
    fn from(session: Session) -> Self {
        Self {
            session_id: session.id,
            four_words: session.four_words,
            display_name: session.display_name,
            // Use pubkey_hex from session if available, otherwise empty
            pubkey_hex: session.pubkey_hex.unwrap_or_default(),
        }
    }
}

/// Unified authentication service for all UI frontends
pub struct AuthService {
    storage_manager: EncryptedStorageManager,
    active_session: Option<Session>,
}

impl AuthService {
    /// Create new auth service with storage manager
    pub fn new(storage_manager: EncryptedStorageManager) -> Self {
        Self {
            storage_manager,
            active_session: None,
        }
    }

    /// Get reference to storage manager
    pub fn storage_manager(&self) -> &EncryptedStorageManager {
        &self.storage_manager
    }

    /// Get mutable reference to storage manager
    pub fn storage_manager_mut(&mut self) -> &mut EncryptedStorageManager {
        &mut self.storage_manager
    }

    /// Create a new vault for a four-word identity
    ///
    /// This creates an encrypted vault with PBKDF2 key derivation (100,000 iterations)
    /// and ChaCha20-Poly1305 encryption.
    pub async fn create_vault(
        &mut self,
        four_words: &str,
        password: &str,
        display_name: &str,
    ) -> Result<String> {
        tracing::info!("AuthService: Creating vault for {}", four_words);

        let vault_id = self
            .storage_manager
            .create_vault(four_words, password, display_name)
            .await?;

        tracing::info!("AuthService: Vault created with ID: {}", vault_id);
        Ok(vault_id)
    }

    /// Login with four-word identity and password
    ///
    /// On success, stores session and optionally saves password to keyring for auto-login.
    pub async fn login(
        &mut self,
        four_words: &str,
        password: &str,
        _device_name: Option<&str>,
    ) -> Result<SessionInfo> {
        tracing::info!("AuthService: Login attempt for {}", four_words);

        // Note: EncryptedStorageManager::login expects Option<Vec<u8>> for passkey, we pass None
        let mut session = self
            .storage_manager
            .login(four_words, password, None)
            .await?;

        // Retrieve the ML-DSA-87 public key from keystore and attach to session
        let id_hex = blake3::hash(four_words.as_bytes()).to_hex().to_string();
        let keystore = Keystore::new();
        if let Ok((pk_bytes, _)) = keystore.load_mldsa_keys(&id_hex) {
            let pubkey_hex = hex::encode(&pk_bytes);
            session = session.with_pubkey_hex(pubkey_hex);
            tracing::debug!("Attached pubkey_hex to session for {}", four_words);
        } else {
            tracing::warn!(
                "Could not load ML-DSA keys for {} - pubkey_hex will be empty",
                four_words
            );
        }

        let session_info = SessionInfo::from(session.clone());
        self.active_session = Some(session);

        tracing::info!("AuthService: Login successful for {}", four_words);
        Ok(session_info)
    }

    /// Logout current session
    pub async fn logout(&mut self) -> Result<()> {
        if let Some(session) = &self.active_session {
            tracing::info!("AuthService: Logging out {}", session.four_words);
            self.storage_manager.logout(&session.id).await?;
            self.active_session = None;
            tracing::info!("AuthService: Logout successful");
            Ok(())
        } else {
            Err(anyhow!("No active session to logout"))
        }
    }

    /// Get current active session
    pub fn get_current_session(&self) -> Option<SessionInfo> {
        self.active_session.as_ref().map(|s| SessionInfo {
            session_id: s.id.clone(),
            four_words: s.four_words.clone(),
            display_name: s.display_name.clone(),
            pubkey_hex: s.pubkey_hex.clone().unwrap_or_default(),
        })
    }

    /// Check if user is currently logged in
    pub fn is_logged_in(&self) -> bool {
        self.active_session.is_some()
    }

    /// List all available vaults
    pub async fn list_vaults(&self) -> Result<Vec<VaultInfo>> {
        self.storage_manager.list_vaults().await
    }

    /// Export vault for backup
    pub async fn export_vault(&self, session_id: &str, include_data: bool) -> Result<Vec<u8>> {
        self.storage_manager
            .export_vault(session_id, include_data)
            .await
    }

    /// Import vault from backup
    pub async fn import_vault(&mut self, backup_data: &[u8], password: &str) -> Result<String> {
        self.storage_manager
            .import_vault(backup_data, password)
            .await
    }

    /// Get recent identities (sorted by last used, max 10)
    pub async fn get_recent_identities(&self) -> Result<Vec<RecentIdentity>> {
        // Note: storage_manager returns Vec directly, not Result
        Ok(self.storage_manager.get_recent_identities().await)
    }

    /// Remove a recent identity from the list (does not delete the vault)
    pub async fn remove_recent_identity(&mut self, four_words: &str) -> Result<()> {
        self.storage_manager
            .remove_recent_identity(four_words)
            .await
    }

    /// Check if vault exists for four-word identity
    pub async fn vault_exists(&self, four_words: &str) -> Result<bool> {
        self.storage_manager.vault_exists(four_words).await
    }

    /// Delete a vault (requires password confirmation)
    pub async fn delete_vault(&mut self, four_words: &str, password: &str) -> Result<()> {
        tracing::warn!("AuthService: Deleting vault for {}", four_words);

        // Verify password before deletion
        let _ = self.login(four_words, password, None).await?;

        self.storage_manager.delete_vault(four_words).await?;

        // Logout if this was the active session
        if let Some(session) = &self.active_session
            && session.four_words == four_words
        {
            self.active_session = None;
        }

        tracing::warn!("AuthService: Vault deleted for {}", four_words);
        Ok(())
    }

    // ========================================================================
    // Passkey / Biometric Authentication Methods
    // ========================================================================

    /// Register a passkey for biometric authentication (legacy - without WebAuthn)
    ///
    /// This enables Touch ID, Face ID, or Windows Hello for the identity.
    /// The password is stored in the platform keyring for secure retrieval.
    pub async fn passkey_register(
        &mut self,
        four_words: &str,
        device_name: &str,
    ) -> Result<PasskeyInfo> {
        tracing::info!(
            "AuthService: Registering passkey for {} on {}",
            four_words,
            device_name
        );

        let info = self
            .storage_manager
            .passkey_register(four_words, device_name)
            .await?;

        tracing::info!("AuthService: Passkey registered successfully");
        Ok(info)
    }

    /// Register a passkey with WebAuthn credential
    ///
    /// This stores the WebAuthn credential for true biometric authentication.
    pub async fn passkey_register_webauthn(
        &mut self,
        four_words: &str,
        device_name: &str,
        credential: crate::encrypted_storage::passkey::WebAuthnCredential,
    ) -> Result<PasskeyInfo> {
        tracing::info!(
            "AuthService: Registering WebAuthn passkey for {} on {}",
            four_words,
            device_name
        );

        let info = self
            .storage_manager
            .passkey_register_webauthn(four_words, device_name, credential)
            .await?;

        tracing::info!("AuthService: WebAuthn passkey registered successfully");
        Ok(info)
    }

    /// Authenticate using passkey/biometric
    ///
    /// This retrieves the password from keyring and performs standard vault login.
    pub async fn passkey_authenticate(&mut self, four_words: &str) -> Result<SessionInfo> {
        tracing::info!("AuthService: Passkey authentication for {}", four_words);

        let session = self
            .storage_manager
            .passkey_authenticate(four_words)
            .await?;

        let session_info = SessionInfo::from(session.clone());
        self.active_session = Some(session);

        tracing::info!("AuthService: Passkey authentication successful");
        Ok(session_info)
    }

    /// Check if identity has a registered passkey
    pub async fn passkey_has_passkey(&self, four_words: &str) -> Result<bool> {
        Ok(self.storage_manager.passkey_has_passkey(four_words).await)
    }

    /// Get passkey information for an identity
    pub async fn passkey_get_info(&self, four_words: &str) -> Result<PasskeyInfo> {
        self.storage_manager.passkey_get_info(four_words).await
    }

    /// Delete passkey for an identity
    pub async fn passkey_delete(&mut self, four_words: &str) -> Result<()> {
        tracing::warn!("AuthService: Deleting passkey for {}", four_words);
        self.storage_manager.passkey_delete(four_words).await?;
        tracing::warn!("AuthService: Passkey deleted");
        Ok(())
    }

    // ========================================================================
    // Auto-Login Methods
    // ========================================================================

    /// Attempt auto-login using last-used identity
    ///
    /// Returns session info if successful, None if no auto-login available.
    pub async fn try_auto_login(&mut self) -> Result<Option<SessionInfo>> {
        tracing::info!("AuthService: Attempting auto-login");

        // Get last used identity from app config (returns Vec directly, not Result)
        let recent = self.storage_manager.get_recent_identities().await;

        if recent.is_empty() {
            tracing::info!("AuthService: No recent identities for auto-login");
            return Ok(None);
        }

        let last_identity = &recent[0];
        tracing::info!(
            "AuthService: Attempting auto-login for {}",
            last_identity.four_words
        );

        // Check if passkey is available
        if last_identity.has_passkey {
            match self.passkey_authenticate(&last_identity.four_words).await {
                Ok(session_info) => {
                    tracing::info!("AuthService: Auto-login successful via passkey");
                    return Ok(Some(session_info));
                }
                Err(e) => {
                    tracing::warn!("AuthService: Passkey auto-login failed: {}", e);
                    // Fall through to return None
                }
            }
        }

        tracing::info!("AuthService: No auto-login available");
        Ok(None)
    }

    /// Enable auto-login for current session
    ///
    /// Stores password in keyring so passkey authentication can work.
    pub async fn enable_auto_login(&mut self, password: &str) -> Result<()> {
        let session = self
            .active_session
            .as_ref()
            .ok_or_else(|| anyhow!("No active session"))?;

        tracing::info!(
            "AuthService: Enabling auto-login for {}",
            session.four_words
        );

        // Store password in keyring via storage manager
        self.storage_manager
            .store_password_in_keyring(&session.four_words, password)
            .await?;

        tracing::info!("AuthService: Auto-login enabled");
        Ok(())
    }

    /// Disable auto-login for an identity
    pub async fn disable_auto_login(&mut self, four_words: &str) -> Result<()> {
        tracing::info!("AuthService: Disabling auto-login for {}", four_words);

        // Remove password from keyring
        self.storage_manager
            .remove_password_from_keyring(four_words)
            .await?;

        // Delete passkey if exists
        if self.passkey_has_passkey(four_words).await? {
            self.passkey_delete(four_words).await?;
        }

        tracing::info!("AuthService: Auto-login disabled");
        Ok(())
    }

    // ========================================================================
    // Identity Switching Methods
    // ========================================================================

    /// Switch to another identity (logout current, login new)
    pub async fn switch_identity(&mut self, four_words: &str) -> Result<SessionInfo> {
        tracing::info!("AuthService: Switching to identity {}", four_words);

        // Logout current session if exists
        if self.active_session.is_some() {
            self.logout().await.ok(); // Ignore logout errors
        }

        // Try passkey authentication first
        if self.passkey_has_passkey(four_words).await? {
            match self.passkey_authenticate(four_words).await {
                Ok(session_info) => {
                    tracing::info!("AuthService: Identity switch successful via passkey");
                    return Ok(session_info);
                }
                Err(e) => {
                    tracing::warn!("AuthService: Passkey switch failed: {}", e);
                    return Err(anyhow!("Passkey authentication required but failed"));
                }
            }
        }

        Err(anyhow!(
            "Cannot switch to identity without password or passkey"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encrypted_storage::StorageConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_auth_service_basic_flow() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = StorageConfig {
            vault_dir: temp_dir.path().to_path_buf(),
            use_keyring: false,
            ..Default::default()
        };

        let storage_manager = EncryptedStorageManager::new(config)
            .await
            .expect("Failed to create storage manager");

        let mut auth_service = AuthService::new(storage_manager);

        // Create vault
        let vault_id = auth_service
            .create_vault("ocean-forest-moon-star", "test-password", "Test User")
            .await
            .expect("Failed to create vault");

        assert!(!vault_id.is_empty());

        // Login
        let session_info = auth_service
            .login(
                "ocean-forest-moon-star",
                "test-password",
                Some("Test Device"),
            )
            .await
            .expect("Failed to login");

        assert_eq!(session_info.four_words, "ocean-forest-moon-star");
        assert_eq!(session_info.display_name, "Test User");
        assert!(auth_service.is_logged_in());

        // Get current session
        let current = auth_service.get_current_session();
        assert!(current.is_some());
        assert_eq!(current.unwrap().four_words, "ocean-forest-moon-star");

        // Logout
        auth_service.logout().await.expect("Failed to logout");
        assert!(!auth_service.is_logged_in());
    }

    #[tokio::test]
    async fn test_auth_service_recent_identities() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let vault_subdir = temp_dir.path().join("vaults");
        std::fs::create_dir_all(&vault_subdir).expect("Failed to create vault subdir");
        let config = StorageConfig {
            vault_dir: vault_subdir,
            use_keyring: false,
            ..Default::default()
        };

        let storage_manager = EncryptedStorageManager::new(config)
            .await
            .expect("Failed to create storage manager");

        let mut auth_service = AuthService::new(storage_manager);

        // Create and login with first identity
        auth_service
            .create_vault("ocean-forest-moon-star", "pass1", "User 1")
            .await
            .expect("Failed to create vault 1");

        auth_service
            .login("ocean-forest-moon-star", "pass1", Some("Device 1"))
            .await
            .expect("Failed to login 1");

        auth_service.logout().await.expect("Failed to logout 1");

        // Create and login with second identity
        auth_service
            .create_vault("river-cloud-stone-tree", "pass2", "User 2")
            .await
            .expect("Failed to create vault 2");

        auth_service
            .login("river-cloud-stone-tree", "pass2", Some("Device 2"))
            .await
            .expect("Failed to login 2");

        // Get recent identities
        let recent = auth_service
            .get_recent_identities()
            .await
            .expect("Failed to get recent");

        assert_eq!(recent.len(), 2);
        // Most recent should be first
        assert_eq!(recent[0].four_words, "river-cloud-stone-tree");
        assert_eq!(recent[1].four_words, "ocean-forest-moon-star");
    }
}
