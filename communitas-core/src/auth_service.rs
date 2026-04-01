// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared Authentication Service
//!
//! This module provides a unified authentication service that can be used by all frontends
//! (Dioxus UI, CLI, MCP, headless, etc.). It encapsulates all business logic for:
//! - Multi-identity management
//! - Vault creation and authentication
//! - Session management
//! - Auto-login functionality

use crate::encrypted_storage::{
    EncryptedStorageManager, RecentIdentity, Session, VaultInfo, load_identity_keys,
};
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
    /// Session expiration timestamp (Unix seconds)
    pub expires_at: u64,
}

impl From<Session> for SessionInfo {
    fn from(session: Session) -> Self {
        Self {
            session_id: session.id,
            four_words: session.four_words,
            display_name: session.display_name,
            // Use pubkey_hex from session if available, otherwise empty
            pubkey_hex: session.pubkey_hex.unwrap_or_default(),
            expires_at: session.expires_at,
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
    /// This creates a local vault directory and metadata.
    pub async fn create_vault(&mut self, four_words: &str, display_name: &str) -> Result<String> {
        tracing::info!("AuthService: Creating vault for {}", four_words);

        let vault_id = self
            .storage_manager
            .create_vault(four_words, display_name)
            .await?;

        tracing::info!("AuthService: Vault created with ID: {}", vault_id);
        Ok(vault_id)
    }

    /// Login with four-word identity
    ///
    /// On success, stores session for the active user.
    pub async fn login(&mut self, four_words: &str) -> Result<SessionInfo> {
        tracing::info!("AuthService: Login attempt for {}", four_words);

        let mut session = self.storage_manager.login(four_words).await?;

        if let Ok(keys) = load_identity_keys(self.storage_manager.vault_dir(), four_words).await {
            let pubkey_hex = hex::encode(&keys.public_key);
            session = session.with_pubkey_hex(pubkey_hex);
            tracing::debug!("Attached pubkey_hex to session for {}", four_words);
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
            expires_at: s.expires_at,
        })
    }

    /// Get time remaining until session expiration
    pub fn session_time_remaining(&self) -> Option<std::time::Duration> {
        self.active_session.as_ref().map(|s| s.time_remaining())
    }

    /// Check if session is expiring soon (less than given seconds remaining)
    pub fn session_expires_soon(&self, threshold_secs: u64) -> bool {
        self.active_session
            .as_ref()
            .map(|s| s.time_remaining().as_secs() < threshold_secs)
            .unwrap_or(false)
    }

    /// Refresh the current session, extending its expiration
    ///
    /// Returns the updated session info if successful.
    pub async fn refresh_session(&mut self) -> Result<SessionInfo> {
        let session = self
            .active_session
            .as_mut()
            .ok_or_else(|| anyhow!("No active session to refresh"))?;

        // Extend session by default duration (8 hours)
        const DEFAULT_SESSION_EXTENSION_SECS: u64 = 8 * 60 * 60;
        session.extend(DEFAULT_SESSION_EXTENSION_SECS);

        tracing::info!(
            "AuthService: Session refreshed for {}, new expiry: {}",
            session.four_words,
            session.expires_at
        );

        Ok(SessionInfo::from(session.clone()))
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
    pub async fn import_vault(&mut self, backup_data: &[u8]) -> Result<String> {
        self.storage_manager.import_vault(backup_data).await
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

    /// Delete a vault
    pub async fn delete_vault(&mut self, four_words: &str) -> Result<()> {
        tracing::warn!("AuthService: Deleting vault for {}", four_words);

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
    // Auto-Login Methods
    // ========================================================================

    /// Attempt auto-login using last-used identity
    ///
    /// Returns session info if successful, None if no auto-login available.
    pub async fn try_auto_login(&mut self) -> Result<Option<SessionInfo>> {
        tracing::info!("AuthService: Attempting auto-login");
        match self.storage_manager.try_auto_login().await? {
            Some(session) => {
                let mut session = session;
                if let Ok(keys) =
                    load_identity_keys(self.storage_manager.vault_dir(), &session.four_words).await
                {
                    let pubkey_hex = hex::encode(&keys.public_key);
                    session = session.with_pubkey_hex(pubkey_hex);
                }

                let session_info = SessionInfo::from(session.clone());
                self.active_session = Some(session);
                tracing::info!("AuthService: Auto-login successful");
                Ok(Some(session_info))
            }
            None => {
                tracing::info!("AuthService: No auto-login available");
                Ok(None)
            }
        }
    }

    /// Enable auto-login for current session (no-op; auto-login always on for local vaults)
    pub async fn enable_auto_login(&mut self) -> Result<()> {
        if self.active_session.is_none() {
            return Err(anyhow!("No active session"));
        }
        tracing::info!("AuthService: Auto-login enabled (local vaults)");
        Ok(())
    }

    /// Disable auto-login for an identity (no-op; retained for API compatibility)
    pub async fn disable_auto_login(&mut self, _four_words: &str) -> Result<()> {
        tracing::info!("AuthService: Auto-login disable requested (no-op)");
        Ok(())
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
            .create_vault("ocean-forest-moon-star", "Test User")
            .await
            .expect("Failed to create vault");

        assert!(!vault_id.is_empty());

        // Login
        let session_info = auth_service
            .login("ocean-forest-moon-star")
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
            .create_vault("ocean-forest-moon-star", "User 1")
            .await
            .expect("Failed to create vault 1");

        auth_service
            .login("ocean-forest-moon-star")
            .await
            .expect("Failed to login 1");

        auth_service.logout().await.expect("Failed to logout 1");

        // Create and login with second identity
        auth_service
            .create_vault("river-cloud-stone-tree", "User 2")
            .await
            .expect("Failed to create vault 2");

        auth_service
            .login("river-cloud-stone-tree")
            .await
            .expect("Failed to login 2");

        // Get recent identities
        let recent = auth_service
            .get_recent_identities()
            .await
            .expect("Failed to get recent");

        assert_eq!(recent.len(), 2);
        assert!(
            recent
                .iter()
                .any(|r| r.four_words == "river-cloud-stone-tree")
        );
        assert!(
            recent
                .iter()
                .any(|r| r.four_words == "ocean-forest-moon-star")
        );
        // Most recent should be first; if timestamps match, either order is acceptable.
        assert!(recent[0].last_used >= recent[1].last_used);
    }
}
