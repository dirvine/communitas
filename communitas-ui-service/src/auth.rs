use async_trait::async_trait;
use communitas_core::{
    encrypted_storage::{
        EncryptedStorageManager, StorageConfig, VaultMetadata, ensure_identity_keys,
        vault_dir_from_root,
    },
    recovery::{Language, RecoveryConfig, create_new_identity, recover_identity},
    ui_core::{CommunitasApi, UiRecentIdentity, UiSessionInfo},
};
use std::{
    env,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::{
    io::AsyncWriteExt,
    sync::{RwLock, watch},
};
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::storage::{StorageError, UiStorage};

/// Snapshot of authentication/session state.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthStateSnapshot {
    LoggedOut,
    Authenticating,
    /// Authenticated with session and whether it expires soon (< 5 minutes)
    Authenticated {
        session: AuthSession,
        expires_soon: bool,
    },
}

/// Warning threshold for session expiration (5 minutes)
pub const SESSION_EXPIRY_WARNING_SECS: u64 = 5 * 60;

/// Information about the active identity/session.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthSession {
    pub pubkey_hex: String,
    pub four_words: String,
    pub display_name: String,
    pub device_name: String,
    /// Session expiration timestamp (Unix seconds)
    pub expires_at: u64,
}

/// Result of creating a new identity (session + mnemonic)
#[derive(Debug, Clone, PartialEq)]
pub struct CreateIdentityResult {
    pub session: AuthSession,
    pub mnemonic: String,
}

impl From<(UiSessionInfo, String)> for AuthSession {
    fn from((info, device_name): (UiSessionInfo, String)) -> Self {
        Self {
            pubkey_hex: info.pubkey_hex,
            four_words: info.four_words,
            display_name: info.display_name,
            device_name,
            expires_at: info.expires_at,
        }
    }
}

impl AuthSession {
    /// Check if session is expiring soon (within threshold)
    pub fn expires_soon(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.expires_at.saturating_sub(now) < SESSION_EXPIRY_WARNING_SECS
    }

    /// Get time remaining until expiration
    pub fn time_remaining(&self) -> std::time::Duration {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        std::time::Duration::from_secs(self.expires_at.saturating_sub(now))
    }
}

/// Recent identity for quick switch UI
#[derive(Debug, Clone, PartialEq)]
pub struct RecentIdentity {
    pub four_words: String,
    pub display_name: String,
    pub last_used: u64,
}

/// Information about an available vault for login selection.
#[derive(Debug, Clone, PartialEq)]
pub struct VaultInfo {
    pub four_words: String,
    pub display_name: String,
    pub last_accessed: u64,
}

impl From<UiRecentIdentity> for RecentIdentity {
    fn from(ui: UiRecentIdentity) -> Self {
        Self {
            four_words: ui.four_words,
            display_name: ui.display_name,
            last_used: ui.last_used,
        }
    }
}

/// Errors returned by the authentication service.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    #[error("core error: {0}")]
    Core(String),
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("internal state error: {0}")]
    State(&'static str),
}

impl From<String> for AuthError {
    fn from(value: String) -> Self {
        AuthError::Core(value)
    }
}

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn login(&self, four_words: &str) -> Result<AuthSession, AuthError>;
    async fn create_identity(&self, display_name: &str) -> Result<CreateIdentityResult, AuthError>;
    async fn recover_identity(
        &self,
        mnemonic: &str,
        passphrase: Option<&str>,
        display_name: &str,
        friend_four_words: Option<&str>,
        temporary: bool,
    ) -> Result<AuthSession, AuthError>;
    async fn logout(&self) -> Result<(), AuthError>;
    fn current_session(&self) -> Option<AuthSession>;
    fn subscribe(&self) -> watch::Receiver<AuthStateSnapshot>;

    /// Get the session expiration timestamp (Unix seconds)
    fn session_expires_at(&self) -> Option<u64>;

    /// Check if the session is expiring soon (within warning threshold)
    fn session_expires_soon(&self) -> bool;

    /// Refresh the current session, extending its expiration
    async fn refresh_session(&self) -> Result<AuthSession, AuthError>;

    // =====================
    // Multi-Identity Quick Switch
    // =====================

    /// Get list of recent identities for quick switch UI
    async fn list_recent_identities(&self) -> Result<Vec<RecentIdentity>, AuthError>;

    /// Attempt auto-login using the most recent identity
    async fn try_auto_login(&self) -> Result<Option<AuthSession>, AuthError>;

    /// Remove a recent identity from the list (does not delete the vault)
    async fn remove_recent_identity(&self, four_words: &str) -> Result<(), AuthError>;

    /// List available vaults for login selection (by display_name).
    /// Returns vaults sorted by last accessed time, most recent first.
    async fn list_vaults(&self) -> Result<Vec<VaultInfo>, AuthError>;
}

struct AuthInner {
    api: Option<CommunitasApi>,
    session: Option<AuthSession>,
    temp_root: Option<PathBuf>,
}

/// Concrete implementation shared by all UIs.
pub struct AuthController {
    storage: UiStorage,
    device_name: String,
    inner: RwLock<AuthInner>,
    state_tx: watch::Sender<AuthStateSnapshot>,
    /// Keep one receiver alive so send() never fails when there are no external subscribers.
    #[allow(dead_code)]
    state_rx: watch::Receiver<AuthStateSnapshot>,
    failure_mode: AuthFailureMode,
}

impl AuthController {
    pub fn new(storage: UiStorage) -> Result<Self, AuthError> {
        let (state_tx, state_rx) = watch::channel(AuthStateSnapshot::LoggedOut);
        let failure_mode = AuthFailureMode::from_env();
        Ok(Self {
            storage,
            device_name: format!("Communitas-{}", whoami::devicename()),
            inner: RwLock::new(AuthInner {
                api: None,
                session: None,
                temp_root: None,
            }),
            state_tx,
            state_rx,
            failure_mode,
        })
    }

    fn storage_path(&self) -> Result<String, AuthError> {
        Ok(self.storage.root_string()?)
    }

    fn storage_path_from_root(&self, root: &Path) -> Result<String, AuthError> {
        root.to_str()
            .map(|s| s.to_string())
            .ok_or(AuthError::InvalidInput("invalid storage path"))
    }

    pub fn is_temporary_session(&self) -> bool {
        self.inner.blocking_read().temp_root.is_some()
    }

    fn vault_dir(&self) -> PathBuf {
        vault_dir_from_root(self.storage.root())
    }

    async fn load_vault_display_name(&self, four_words: &str) -> Option<String> {
        let vault_meta = self.vault_dir().join(four_words).join("vault.meta");
        if let Ok(raw) = tokio::fs::read(&vault_meta).await
            && let Ok(metadata) = serde_json::from_slice::<VaultMetadata>(&raw)
            && !metadata.display_name.is_empty()
        {
            return Some(metadata.display_name);
        }
        None
    }

    async fn build_storage_manager(&self) -> Result<EncryptedStorageManager, AuthError> {
        let storage_root = self.storage.root_path();
        let vault_dir = vault_dir_from_root(&storage_root);
        tokio::fs::create_dir_all(&vault_dir)
            .await
            .map_err(|e| AuthError::Core(e.to_string()))?;

        let config = StorageConfig {
            vault_dir,
            ..StorageConfig::default()
        };
        EncryptedStorageManager::new(config)
            .await
            .map_err(|e| AuthError::Core(e.to_string()))
    }

    async fn zeroize_dir(path: &Path) -> Result<(), AuthError> {
        if !path.exists() {
            return Ok(());
        }

        let mut stack = vec![path.to_path_buf()];
        let mut dirs_to_remove = Vec::new();

        while let Some(dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&dir)
                .await
                .map_err(|e| AuthError::Core(e.to_string()))?;
            dirs_to_remove.push(dir.clone());

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| AuthError::Core(e.to_string()))?
            {
                let file_type = entry
                    .file_type()
                    .await
                    .map_err(|e| AuthError::Core(e.to_string()))?;
                let entry_path = entry.path();

                if file_type.is_dir() {
                    stack.push(entry_path);
                    continue;
                }

                if file_type.is_symlink() {
                    let _ = tokio::fs::remove_file(&entry_path).await;
                    continue;
                }

                if let Ok(metadata) = entry.metadata().await {
                    let mut remaining = metadata.len();
                    if let Ok(mut file) = tokio::fs::OpenOptions::new()
                        .write(true)
                        .open(&entry_path)
                        .await
                    {
                        let zeros = vec![0u8; 8192];
                        while remaining > 0 {
                            let chunk = zeros.len().min(remaining as usize);
                            file.write_all(&zeros[..chunk])
                                .await
                                .map_err(|e| AuthError::Core(e.to_string()))?;
                            remaining = remaining.saturating_sub(chunk as u64);
                        }
                        let _ = file.sync_all().await;
                    }
                }

                let _ = tokio::fs::remove_file(&entry_path).await;
            }
        }

        for dir in dirs_to_remove.into_iter().rev() {
            let _ = tokio::fs::remove_dir(&dir).await;
        }

        Ok(())
    }

    fn set_state(&self, state: AuthStateSnapshot) {
        if self.state_tx.send(state).is_err() {
            warn!("auth state receiver dropped");
        }
    }

    async fn set_session(
        &self,
        api: CommunitasApi,
        session: AuthSession,
        temp_root: Option<PathBuf>,
    ) {
        let mut inner = self.inner.write().await;
        inner.api = Some(api);
        inner.session = Some(session);
        inner.temp_root = temp_root;
    }

    /// Spawn network start in background (non-blocking)
    fn spawn_network_start(api: CommunitasApi) {
        tokio::spawn(async move {
            if let Err(err) = api.gossip_start(None).await {
                warn!(
                    target = "ui.auth",
                    "failed to start gossip (non-fatal): {err}"
                );
            } else {
                info!(target = "ui.auth", "gossip runtime started");
            }
        });
    }

    fn ensure_four_words<'a>(&self, value: &'a str) -> Result<&'a str, AuthError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(AuthError::InvalidInput("four words cannot be empty"))
        } else {
            Ok(trimmed)
        }
    }

    fn ensure_display_name<'a>(&self, value: &'a str) -> Result<&'a str, AuthError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(AuthError::InvalidInput("display name is required"))
        } else {
            Ok(trimmed)
        }
    }

    pub fn api(&self) -> Option<CommunitasApi> {
        self.inner.blocking_read().api.clone()
    }

    /// Async-compatible API accessor (safe to call from within async context).
    pub async fn api_async(&self) -> Option<CommunitasApi> {
        self.inner.read().await.api.clone()
    }

    fn fail_if_requested(&self) -> Result<(), AuthError> {
        if let AuthFailureMode::AlwaysFail = self.failure_mode {
            warn!(
                target = "ui.auth",
                "forcing authentication failure via QA flag"
            );
            return Err(AuthError::State("forced auth failure"));
        }
        Ok(())
    }

    /// Enable demo mode for testing and development. Sets an authenticated state with a demo session.
    /// This is useful for unit tests and demo scenarios that need to bypass real authentication.
    pub fn enable_demo_mode(&self) {
        // Demo session expires in 8 hours
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let demo_session = AuthSession {
            pubkey_hex: "demo_pubkey_hex_1234567890".to_string(),
            four_words: "demo-test-user-mode".to_string(),
            display_name: "Demo User".to_string(),
            device_name: "Test Device".to_string(),
            expires_at: now + 8 * 60 * 60, // 8 hours
        };
        self.set_state(AuthStateSnapshot::Authenticated {
            session: demo_session,
            expires_soon: false,
        });
    }
}

fn redact_identity(words: &str) -> String {
    let mut parts: Vec<&str> = words.split('-').collect();
    if parts.len() > 2 {
        parts.truncate(2);
        parts.push("••••");
    }
    parts.join("-")
}

#[derive(Debug, Clone, Copy)]
enum AuthFailureMode {
    None,
    AlwaysFail,
}

impl AuthFailureMode {
    fn from_env() -> Self {
        match env::var("COMMUNITAS_UI_FORCE_AUTH_ERROR") {
            Ok(value) if value == "1" || value.eq_ignore_ascii_case("true") => {
                AuthFailureMode::AlwaysFail
            }
            _ => AuthFailureMode::None,
        }
    }
}

#[async_trait]
impl AuthService for AuthController {
    #[instrument(name = "ui.auth.login", skip(self), fields(identity = %redact_identity(four_words)))]
    async fn login(&self, four_words: &str) -> Result<AuthSession, AuthError> {
        self.fail_if_requested()?;
        let four_words = self.ensure_four_words(four_words)?;

        self.set_state(AuthStateSnapshot::Authenticating);

        let display_name = self
            .load_vault_display_name(four_words)
            .await
            .unwrap_or_else(|| four_words.to_string());

        let api = CommunitasApi::create(
            four_words.to_string(),
            display_name,
            self.device_name.clone(),
            self.storage_path()?,
        )
        .await
        .map_err(AuthError::Core)?;

        let session_info = api
            .auth_login(four_words.to_string())
            .await
            .map_err(AuthError::Core)?;

        // Spawn network start in background (non-blocking)
        Self::spawn_network_start(api.clone());

        let session = AuthSession::from((session_info, self.device_name.clone()));
        let expires_soon = session.expires_soon();
        self.set_session(api, session.clone(), None).await;
        self.set_state(AuthStateSnapshot::Authenticated {
            session: session.clone(),
            expires_soon,
        });
        info!(target = "ui.auth", "login complete");
        Ok(session)
    }

    #[instrument(name = "ui.auth.create", skip(self), fields(display_name))]
    async fn create_identity(&self, display_name: &str) -> Result<CreateIdentityResult, AuthError> {
        self.fail_if_requested()?;
        let display_name = self.ensure_display_name(display_name)?;

        self.set_state(AuthStateSnapshot::Authenticating);

        let config = RecoveryConfig::default();
        let (mnemonic, keys) =
            create_new_identity(&config, None).map_err(|e| AuthError::Core(e.to_string()))?;
        let four_words = keys.four_words.clone();

        let storage_root = self.storage.root_path();
        let vault_dir = vault_dir_from_root(&storage_root);
        ensure_identity_keys(
            &vault_dir,
            &four_words,
            display_name,
            keys.verifying_key_bytes(),
            keys.signing_key_bytes(),
        )
        .await
        .map_err(|e| AuthError::Core(e.to_string()))?;

        let api = CommunitasApi::create(
            four_words.clone(),
            display_name.to_string(),
            self.device_name.clone(),
            self.storage_path()?,
        )
        .await
        .map_err(AuthError::Core)?;

        api.auth_create_vault(four_words.clone(), display_name.to_string())
            .await
            .map_err(AuthError::Core)?;

        let session_info = api.auth_login(four_words).await.map_err(AuthError::Core)?;

        // Spawn network start in background (non-blocking)
        Self::spawn_network_start(api.clone());

        let session = AuthSession::from((session_info, self.device_name.clone()));
        let expires_soon = session.expires_soon();
        self.set_session(api, session.clone(), None).await;
        self.set_state(AuthStateSnapshot::Authenticated {
            session: session.clone(),
            expires_soon,
        });
        info!(target: "ui.auth", "identity created");
        Ok(CreateIdentityResult {
            session,
            mnemonic: mnemonic.to_string(),
        })
    }

    #[instrument(name = "ui.auth.recover", skip(self), fields(display_name, mnemonic_len = mnemonic.len()))]
    async fn recover_identity(
        &self,
        mnemonic: &str,
        passphrase: Option<&str>,
        display_name: &str,
        friend_four_words: Option<&str>,
        temporary: bool,
    ) -> Result<AuthSession, AuthError> {
        self.fail_if_requested()?;
        if mnemonic.trim().is_empty() {
            return Err(AuthError::InvalidInput("mnemonic is required"));
        }
        let display_name = self.ensure_display_name(display_name)?;

        self.set_state(AuthStateSnapshot::Authenticating);

        if let Some(friend) = friend_four_words {
            info!(
                target = "ui.auth",
                "friend connection provided for recovery: {}",
                redact_identity(friend)
            );
        }

        let keys = recover_identity(mnemonic, Language::English, passphrase)
            .map_err(|e| AuthError::Core(e.to_string()))?;
        let four_words = keys.four_words.clone();

        let temp_root = if temporary {
            let mut root = env::temp_dir();
            root.push(format!("communitas-temp-{}", Uuid::new_v4()));
            tokio::fs::create_dir_all(&root)
                .await
                .map_err(|e| AuthError::Core(e.to_string()))?;
            Some(root)
        } else {
            None
        };

        let storage_root = temp_root
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.storage.root_path());
        let vault_dir = vault_dir_from_root(&storage_root);
        ensure_identity_keys(
            &vault_dir,
            &four_words,
            display_name,
            keys.verifying_key_bytes(),
            keys.signing_key_bytes(),
        )
        .await
        .map_err(|e| AuthError::Core(e.to_string()))?;

        let api = CommunitasApi::create(
            four_words.clone(),
            display_name.to_string(),
            self.device_name.clone(),
            self.storage_path_from_root(&storage_root)?,
        )
        .await
        .map_err(AuthError::Core)?;

        let exists = api
            .auth_vault_exists(four_words.clone())
            .await
            .map_err(AuthError::Core)?;
        if !exists {
            api.auth_create_vault(four_words.clone(), display_name.to_string())
                .await
                .map_err(AuthError::Core)?;
        }

        let session_info = api
            .auth_login(four_words.clone())
            .await
            .map_err(AuthError::Core)?;

        // Spawn network start in background (non-blocking)
        Self::spawn_network_start(api.clone());

        let session = AuthSession::from((session_info, self.device_name.clone()));
        let expires_soon = session.expires_soon();
        self.set_session(api, session.clone(), temp_root).await;
        self.set_state(AuthStateSnapshot::Authenticated {
            session: session.clone(),
            expires_soon,
        });
        info!(target = "ui.auth", "identity recovered");
        Ok(session)
    }

    #[instrument(name = "ui.auth.logout", skip(self))]
    async fn logout(&self) -> Result<(), AuthError> {
        let mut inner = self.inner.write().await;
        let temp_root = inner.temp_root.take();
        if let Some(api) = inner.api.take() {
            if let Err(err) = api.gossip_stop().await {
                warn!("failed to stop gossip: {err}");
            }
            if let Err(err) = api.auth_logout().await {
                warn!("failed to logout: {err}");
            }
        }
        inner.session = None;
        drop(inner);
        self.set_state(AuthStateSnapshot::LoggedOut);
        if let Some(root) = temp_root {
            if let Err(err) = Self::zeroize_dir(&root).await {
                warn!("failed to zeroize temp data at {}: {err}", root.display());
            }
            let _ = tokio::fs::remove_dir_all(&root).await;
            if let Err(err) = self.try_auto_login().await {
                warn!("failed to auto-login after temp logout: {err}");
            }
        }
        info!(target = "ui.auth", "session terminated");
        Ok(())
    }

    fn current_session(&self) -> Option<AuthSession> {
        self.inner.blocking_read().session.clone()
    }

    fn subscribe(&self) -> watch::Receiver<AuthStateSnapshot> {
        self.state_tx.subscribe()
    }

    fn session_expires_at(&self) -> Option<u64> {
        self.inner
            .blocking_read()
            .session
            .as_ref()
            .map(|s| s.expires_at)
    }

    fn session_expires_soon(&self) -> bool {
        self.inner
            .blocking_read()
            .session
            .as_ref()
            .map(|s| s.expires_soon())
            .unwrap_or(false)
    }

    #[instrument(name = "ui.auth.refresh", skip(self))]
    async fn refresh_session(&self) -> Result<AuthSession, AuthError> {
        let mut inner = self.inner.write().await;

        // Get the API to refresh at the core level
        let api = inner
            .api
            .as_ref()
            .ok_or(AuthError::State("no active session to refresh"))?;

        // Refresh the session at the core level
        let session_info = api.auth_refresh_session().await.map_err(AuthError::Core)?;

        // Update local session
        let session = AuthSession::from((session_info, self.device_name.clone()));
        let expires_soon = session.expires_soon();
        inner.session = Some(session.clone());
        drop(inner);

        // Broadcast updated state
        self.set_state(AuthStateSnapshot::Authenticated {
            session: session.clone(),
            expires_soon,
        });

        info!(
            target = "ui.auth",
            "session refreshed, expires_at: {}", session.expires_at
        );

        Ok(session)
    }

    // =====================
    // Multi-Identity Quick Switch
    // =====================

    #[instrument(name = "ui.auth.list_recent_identities", skip(self))]
    async fn list_recent_identities(&self) -> Result<Vec<RecentIdentity>, AuthError> {
        let inner = self.inner.read().await;
        let api = inner
            .api
            .as_ref()
            .ok_or(AuthError::State("no active session"))?;

        let recent = api
            .auth_get_recent_identities()
            .await
            .map_err(AuthError::Core)?;

        Ok(recent.into_iter().map(RecentIdentity::from).collect())
    }

    #[instrument(name = "ui.auth.try_auto_login", skip(self))]
    async fn try_auto_login(&self) -> Result<Option<AuthSession>, AuthError> {
        self.fail_if_requested()?;

        let storage_manager = self.build_storage_manager().await?;
        let mut auth = communitas_core::auth_service::AuthService::new(storage_manager);
        let result = auth
            .try_auto_login()
            .await
            .map_err(|e| AuthError::Core(e.to_string()))?;

        match result {
            Some(session_info) => {
                self.set_state(AuthStateSnapshot::Authenticating);

                let ui_session = UiSessionInfo::from(session_info);

                let real_api = CommunitasApi::create(
                    ui_session.four_words.clone(),
                    ui_session.display_name.clone(),
                    self.device_name.clone(),
                    self.storage_path()?,
                )
                .await
                .map_err(AuthError::Core)?;

                Self::spawn_network_start(real_api.clone());

                let session = AuthSession::from((ui_session, self.device_name.clone()));
                let expires_soon = session.expires_soon();
                self.set_session(real_api, session.clone(), None).await;
                self.set_state(AuthStateSnapshot::Authenticated {
                    session: session.clone(),
                    expires_soon,
                });

                info!(target = "ui.auth", "auto-login successful");
                Ok(Some(session))
            }
            None => {
                info!(target = "ui.auth", "no identity available for auto-login");
                Ok(None)
            }
        }
    }

    #[instrument(name = "ui.auth.remove_recent_identity", skip(self), fields(identity = %redact_identity(four_words)))]
    async fn remove_recent_identity(&self, four_words: &str) -> Result<(), AuthError> {
        let four_words = self.ensure_four_words(four_words)?;

        let inner = self.inner.read().await;
        let api = inner
            .api
            .as_ref()
            .ok_or(AuthError::State("no active session"))?;

        api.auth_remove_recent_identity(four_words.to_string())
            .await
            .map_err(AuthError::Core)?;

        info!(target = "ui.auth", "removed recent identity");
        Ok(())
    }

    #[instrument(name = "ui.auth.list_vaults", skip(self))]
    async fn list_vaults(&self) -> Result<Vec<VaultInfo>, AuthError> {
        let storage_manager = self.build_storage_manager().await?;
        let vaults = storage_manager
            .list_vaults()
            .await
            .map_err(|e| AuthError::Core(e.to_string()))?;

        // Convert to VaultInfo and sort by last_accessed (most recent first)
        let mut vault_infos: Vec<VaultInfo> = vaults
            .into_iter()
            .map(|v| VaultInfo {
                four_words: v.four_words,
                display_name: v.display_name,
                last_accessed: v.last_accessed,
            })
            .collect();

        vault_infos.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));

        Ok(vault_infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_controller(temp: &TempDir) -> AuthController {
        let storage = UiStorage::from_path(temp.path()).unwrap();
        AuthController::new(storage).unwrap()
    }

    #[test]
    fn auth_controller_creates_successfully() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);
        assert!(controller.current_session().is_none());
    }

    #[test]
    fn ensure_four_words_rejects_empty() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller.ensure_four_words("");
        assert!(result.is_err());

        let result = controller.ensure_four_words("   ");
        assert!(result.is_err());
    }

    #[test]
    fn ensure_four_words_trims_whitespace() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller.ensure_four_words("  alpha-beta-gamma-delta  ");
        assert_eq!(result.unwrap(), "alpha-beta-gamma-delta");
    }

    #[test]
    fn ensure_display_name_rejects_empty() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller.ensure_display_name("");
        assert!(result.is_err());

        let result = controller.ensure_display_name("   ");
        assert!(result.is_err());
    }

    #[test]
    fn ensure_display_name_trims_whitespace() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller.ensure_display_name("  Alice  ");
        assert_eq!(result.unwrap(), "Alice");
    }

    #[test]
    fn redact_identity_masks_correctly() {
        assert_eq!(redact_identity("alpha-beta-gamma-delta"), "alpha-beta-••••");
        assert_eq!(
            redact_identity("word1-word2-word3-word4"),
            "word1-word2-••••"
        );
        assert_eq!(redact_identity("alpha-beta"), "alpha-beta");
        assert_eq!(redact_identity("single"), "single");
    }

    #[test]
    fn subscribe_returns_logged_out_initially() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);
        let rx = controller.subscribe();

        match &*rx.borrow() {
            AuthStateSnapshot::LoggedOut => {}
            other => panic!("expected LoggedOut, got {other:?}"),
        }
    }

    #[test]
    fn set_state_broadcasts_to_subscribers() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);
        let rx = controller.subscribe();

        controller.set_state(AuthStateSnapshot::Authenticating);

        match &*rx.borrow() {
            AuthStateSnapshot::Authenticating => {}
            other => panic!("expected Authenticating, got {other:?}"),
        }
    }

    #[test]
    fn auth_failure_mode_parses_env() {
        // SAFETY: Tests run single-threaded by default with `cargo test`
        // This test modifies env vars but restores them at the end.
        unsafe {
            // Default (no env var) should be None
            env::remove_var("COMMUNITAS_UI_FORCE_AUTH_ERROR");
            let mode = AuthFailureMode::from_env();
            assert!(matches!(mode, AuthFailureMode::None));

            // "1" enables failure mode
            env::set_var("COMMUNITAS_UI_FORCE_AUTH_ERROR", "1");
            let mode = AuthFailureMode::from_env();
            assert!(matches!(mode, AuthFailureMode::AlwaysFail));

            // "true" (case-insensitive) enables failure mode
            env::set_var("COMMUNITAS_UI_FORCE_AUTH_ERROR", "TRUE");
            let mode = AuthFailureMode::from_env();
            assert!(matches!(mode, AuthFailureMode::AlwaysFail));

            // Other values are ignored
            env::set_var("COMMUNITAS_UI_FORCE_AUTH_ERROR", "no");
            let mode = AuthFailureMode::from_env();
            assert!(matches!(mode, AuthFailureMode::None));

            // Cleanup
            env::remove_var("COMMUNITAS_UI_FORCE_AUTH_ERROR");
        }
    }

    #[test]
    fn auth_session_from_ui_session_info() {
        use communitas_core::ui_core::UiSessionInfo;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = now + 8 * 60 * 60; // 8 hours from now

        let info = UiSessionInfo {
            session_id: "test-session-id".to_string(),
            pubkey_hex: "abcd1234".to_string(),
            four_words: "alpha-beta-gamma-delta".to_string(),
            display_name: "Alice".to_string(),
            expires_at,
        };

        let session = AuthSession::from((info, "DeviceName".to_string()));
        assert_eq!(session.pubkey_hex, "abcd1234");
        assert_eq!(session.four_words, "alpha-beta-gamma-delta");
        assert_eq!(session.display_name, "Alice");
        assert_eq!(session.device_name, "DeviceName");
        assert_eq!(session.expires_at, expires_at);
        assert!(!session.expires_soon()); // Should not expire soon
    }

    #[test]
    fn auth_session_expires_soon_detection() {
        use communitas_core::ui_core::UiSessionInfo;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Session expiring in 2 minutes (within the 5 minute threshold)
        let expires_soon_time = now + 2 * 60;

        let info = UiSessionInfo {
            session_id: "test-session-id".to_string(),
            pubkey_hex: "abcd1234".to_string(),
            four_words: "alpha-beta-gamma-delta".to_string(),
            display_name: "Alice".to_string(),
            expires_at: expires_soon_time,
        };

        let session = AuthSession::from((info, "DeviceName".to_string()));
        assert!(session.expires_soon()); // Should expire soon
        assert!(session.time_remaining().as_secs() <= 2 * 60);
    }

    #[test]
    fn recent_identity_from_ui_recent_identity() {
        let ui_recent = UiRecentIdentity {
            four_words: "alpha-beta-gamma-delta".to_string(),
            display_name: "Alice".to_string(),
            last_used: 1700000000,
        };

        let recent = RecentIdentity::from(ui_recent);
        assert_eq!(recent.four_words, "alpha-beta-gamma-delta");
        assert_eq!(recent.display_name, "Alice");
        assert_eq!(recent.last_used, 1700000000);
    }

    #[tokio::test]
    async fn list_recent_identities_requires_session() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller.list_recent_identities().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::State(_)));
    }

    #[tokio::test]
    async fn remove_recent_identity_requires_session() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller
            .remove_recent_identity("alpha-beta-gamma-delta")
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::State(_)));
    }

    #[tokio::test]
    async fn remove_recent_identity_validates_input() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        // Empty four_words should fail with InvalidInput before checking session
        let result = controller.remove_recent_identity("").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::InvalidInput(_)));
    }
}
