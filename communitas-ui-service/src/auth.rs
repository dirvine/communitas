use async_trait::async_trait;
use communitas_core::ui_core::{
    generate_id_words, CommunitasApi, UiRecentIdentity, UiSessionInfo,
    recover_identity_from_mnemonic,
};
use std::env;
use thiserror::Error;
use tokio::sync::{RwLock, watch};
use tracing::{info, instrument, warn};

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
    pub has_passkey: bool,
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
            has_passkey: ui.has_passkey,
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
    async fn login(&self, four_words: &str, password: &str) -> Result<AuthSession, AuthError>;
    async fn create_identity(
        &self,
        display_name: &str,
        password: &str,
    ) -> Result<AuthSession, AuthError>;
    async fn recover_identity(
        &self,
        mnemonic: &str,
        passphrase: Option<&str>,
        display_name: &str,
        password: &str,
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

    /// Switch to another identity using passkey/biometric authentication
    async fn switch_identity(&self, four_words: &str) -> Result<AuthSession, AuthError>;

    /// Attempt auto-login using the most recent identity with passkey
    async fn try_auto_login(&self) -> Result<Option<AuthSession>, AuthError>;

    /// Check if an identity has a passkey registered for biometric auth
    async fn has_passkey(&self, four_words: &str) -> Result<bool, AuthError>;

    /// Register a passkey for the current session (enables biometric auth)
    async fn register_passkey(&self) -> Result<(), AuthError>;

    /// Delete passkey for an identity (disables biometric auth)
    async fn delete_passkey(&self, four_words: &str) -> Result<(), AuthError>;

    /// Remove a recent identity from the list (does not delete the vault)
    async fn remove_recent_identity(&self, four_words: &str) -> Result<(), AuthError>;

    /// List available vaults for login selection (by display_name).
    /// Returns vaults sorted by last accessed time, most recent first.
    async fn list_vaults(&self) -> Result<Vec<VaultInfo>, AuthError>;
}

struct AuthInner {
    api: Option<CommunitasApi>,
    session: Option<AuthSession>,
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
            }),
            state_tx,
            state_rx,
            failure_mode,
        })
    }

    fn storage_path(&self) -> Result<String, AuthError> {
        Ok(self.storage.root_string()?)
    }

    fn set_state(&self, state: AuthStateSnapshot) {
        if self.state_tx.send(state).is_err() {
            warn!("auth state receiver dropped");
        }
    }

    async fn set_session(&self, api: CommunitasApi, session: AuthSession) {
        let mut inner = self.inner.write().await;
        inner.api = Some(api);
        inner.session = Some(session);
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

    fn ensure_password<'a>(&self, value: &'a str) -> Result<&'a str, AuthError> {
        if value.is_empty() {
            Err(AuthError::InvalidInput("password cannot be empty"))
        } else {
            Ok(value)
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
    #[instrument(name = "ui.auth.login", skip(self, password), fields(identity = %redact_identity(four_words)))]
    async fn login(&self, four_words: &str, password: &str) -> Result<AuthSession, AuthError> {
        self.fail_if_requested()?;
        let four_words = self.ensure_four_words(four_words)?;
        let password = self.ensure_password(password)?;

        self.set_state(AuthStateSnapshot::Authenticating);

        let api = CommunitasApi::create(
            four_words.to_string(),
            four_words.to_string(),
            self.device_name.clone(),
            self.storage_path()?,
        )
        .await
        .map_err(AuthError::Core)?;

        let session_info = api
            .auth_login(four_words.to_string(), password.to_string())
            .await
            .map_err(AuthError::Core)?;

        // Spawn network start in background (non-blocking)
        Self::spawn_network_start(api.clone());

        let session = AuthSession::from((session_info, self.device_name.clone()));
        let expires_soon = session.expires_soon();
        self.set_session(api, session.clone()).await;
        self.set_state(AuthStateSnapshot::Authenticated {
            session: session.clone(),
            expires_soon,
        });
        info!(target = "ui.auth", "login complete");
        Ok(session)
    }

    #[instrument(name = "ui.auth.create", skip(self, password), fields(display_name))]
    async fn create_identity(
        &self,
        display_name: &str,
        password: &str,
    ) -> Result<AuthSession, AuthError> {
        self.fail_if_requested()?;
        // Generate four_words automatically for connection bootstrap
        let four_words = generate_id_words().map_err(|e| AuthError::Core(e.to_string()))?;
        let display_name = self.ensure_display_name(display_name)?;
        let password = self.ensure_password(password)?;

        self.set_state(AuthStateSnapshot::Authenticating);

        let api = CommunitasApi::create(
            four_words.clone(),
            display_name.to_string(),
            self.device_name.clone(),
            self.storage_path()?,
        )
        .await
        .map_err(AuthError::Core)?;

        api.auth_create_vault(
            four_words.clone(),
            display_name.to_string(),
            password.to_string(),
        )
        .await
        .map_err(AuthError::Core)?;

        let session_info = api
            .auth_login(four_words, password.to_string())
            .await
            .map_err(AuthError::Core)?;

        // Spawn network start in background (non-blocking)
        Self::spawn_network_start(api.clone());

        let session = AuthSession::from((session_info, self.device_name.clone()));
        let expires_soon = session.expires_soon();
        self.set_session(api, session.clone()).await;
        self.set_state(AuthStateSnapshot::Authenticated {
            session: session.clone(),
            expires_soon,
        });
        info!(target: "ui.auth", "identity created");
        Ok(session)
    }

    #[instrument(name = "ui.auth.recover", skip(self, password), fields(display_name, mnemonic_len = mnemonic.len()))]
    async fn recover_identity(
        &self,
        mnemonic: &str,
        passphrase: Option<&str>,
        display_name: &str,
        password: &str,
    ) -> Result<AuthSession, AuthError> {
        self.fail_if_requested()?;
        if mnemonic.trim().is_empty() {
            return Err(AuthError::InvalidInput("mnemonic is required"));
        }
        let display_name = self.ensure_display_name(display_name)?;
        let password = self.ensure_password(password)?;

        self.set_state(AuthStateSnapshot::Authenticating);

        let recovered =
            recover_identity_from_mnemonic(mnemonic.to_string(), passphrase.map(str::to_string))
                .map_err(AuthError::Core)?;
        let four_words = recovered.four_words.clone();

        let api = CommunitasApi::create(
            four_words.clone(),
            display_name.to_string(),
            self.device_name.clone(),
            self.storage_path()?,
        )
        .await
        .map_err(AuthError::Core)?;

        let exists = api
            .auth_vault_exists(four_words.clone())
            .await
            .map_err(AuthError::Core)?;
        if !exists {
            api.auth_create_vault(
                four_words.clone(),
                display_name.to_string(),
                password.to_string(),
            )
            .await
            .map_err(AuthError::Core)?;
        }

        let session_info = api
            .auth_login(four_words.clone(), password.to_string())
            .await
            .map_err(AuthError::Core)?;

        // Spawn network start in background (non-blocking)
        Self::spawn_network_start(api.clone());

        let session = AuthSession::from((session_info, self.device_name.clone()));
        let expires_soon = session.expires_soon();
        self.set_session(api, session.clone()).await;
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

    #[instrument(name = "ui.auth.switch_identity", skip(self), fields(identity = %redact_identity(four_words)))]
    async fn switch_identity(&self, four_words: &str) -> Result<AuthSession, AuthError> {
        self.fail_if_requested()?;
        let four_words = self.ensure_four_words(four_words)?;

        self.set_state(AuthStateSnapshot::Authenticating);

        let mut inner = self.inner.write().await;
        let api = inner
            .api
            .as_ref()
            .ok_or(AuthError::State("no active session"))?;

        let session_info = api
            .auth_switch_identity(four_words.to_string())
            .await
            .map_err(AuthError::Core)?;

        let session = AuthSession::from((session_info, self.device_name.clone()));
        let expires_soon = session.expires_soon();
        inner.session = Some(session.clone());
        drop(inner);

        self.set_state(AuthStateSnapshot::Authenticated {
            session: session.clone(),
            expires_soon,
        });

        info!(target = "ui.auth", "switched identity");
        Ok(session)
    }

    #[instrument(name = "ui.auth.try_auto_login", skip(self))]
    async fn try_auto_login(&self) -> Result<Option<AuthSession>, AuthError> {
        self.fail_if_requested()?;

        // Need to create a temporary API instance to check for auto-login
        // since we're not logged in yet
        let api = CommunitasApi::create(
            "temp-auto-login".to_string(),
            "Auto Login Check".to_string(),
            self.device_name.clone(),
            self.storage_path()?,
        )
        .await
        .map_err(AuthError::Core)?;

        let result = api.auth_try_auto_login().await.map_err(AuthError::Core)?;

        match result {
            Some(session_info) => {
                self.set_state(AuthStateSnapshot::Authenticating);

                // Re-create API with actual identity
                let real_api = CommunitasApi::create(
                    session_info.four_words.clone(),
                    session_info.display_name.clone(),
                    self.device_name.clone(),
                    self.storage_path()?,
                )
                .await
                .map_err(AuthError::Core)?;

                // Spawn network start in background (non-blocking)
                Self::spawn_network_start(real_api.clone());

                let session = AuthSession::from((session_info, self.device_name.clone()));
                let expires_soon = session.expires_soon();
                self.set_session(real_api, session.clone()).await;
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

    #[instrument(name = "ui.auth.has_passkey", skip(self), fields(identity = %redact_identity(four_words)))]
    async fn has_passkey(&self, four_words: &str) -> Result<bool, AuthError> {
        let four_words = self.ensure_four_words(four_words)?;

        let inner = self.inner.read().await;
        let api = inner
            .api
            .as_ref()
            .ok_or(AuthError::State("no active session"))?;

        api.auth_has_passkey(four_words.to_string())
            .await
            .map_err(AuthError::Core)
    }

    #[instrument(name = "ui.auth.register_passkey", skip(self))]
    async fn register_passkey(&self) -> Result<(), AuthError> {
        let inner = self.inner.read().await;
        let api = inner
            .api
            .as_ref()
            .ok_or(AuthError::State("no active session"))?;

        api.auth_register_passkey().await.map_err(AuthError::Core)?;

        info!(target = "ui.auth", "passkey registered");
        Ok(())
    }

    #[instrument(name = "ui.auth.delete_passkey", skip(self), fields(identity = %redact_identity(four_words)))]
    async fn delete_passkey(&self, four_words: &str) -> Result<(), AuthError> {
        let four_words = self.ensure_four_words(four_words)?;

        let inner = self.inner.read().await;
        let api = inner
            .api
            .as_ref()
            .ok_or(AuthError::State("no active session"))?;

        api.auth_delete_passkey(four_words.to_string())
            .await
            .map_err(AuthError::Core)?;

        info!(target = "ui.auth", "passkey deleted");
        Ok(())
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
        // Create a temporary API instance to list vaults since we're not logged in
        let api = CommunitasApi::create(
            "temp-list-vaults".to_string(),
            "Vault Lister".to_string(),
            self.device_name.clone(),
            self.storage_path()?,
        )
        .await
        .map_err(AuthError::Core)?;

        let vaults = api.auth_list_vaults().await.map_err(AuthError::Core)?;

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
    fn ensure_password_rejects_empty() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller.ensure_password("");
        assert!(result.is_err());
    }

    #[test]
    fn ensure_password_allows_whitespace_only() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        // Whitespace-only password is technically allowed (not trimmed)
        let result = controller.ensure_password("   ");
        assert!(result.is_ok());
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
            has_passkey: true,
        };

        let recent = RecentIdentity::from(ui_recent);
        assert_eq!(recent.four_words, "alpha-beta-gamma-delta");
        assert_eq!(recent.display_name, "Alice");
        assert_eq!(recent.last_used, 1700000000);
        assert!(recent.has_passkey);
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
    async fn switch_identity_requires_session() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller.switch_identity("alpha-beta-gamma-delta").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::State(_)));
    }

    #[tokio::test]
    async fn switch_identity_validates_input() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        // Empty four_words should fail with InvalidInput before checking session
        let result = controller.switch_identity("").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn has_passkey_requires_session() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller.has_passkey("alpha-beta-gamma-delta").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::State(_)));
    }

    #[tokio::test]
    async fn has_passkey_validates_input() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        // Empty four_words should fail with InvalidInput before checking session
        let result = controller.has_passkey("").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn register_passkey_requires_session() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller.register_passkey().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::State(_)));
    }

    #[tokio::test]
    async fn delete_passkey_requires_session() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        let result = controller.delete_passkey("alpha-beta-gamma-delta").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::State(_)));
    }

    #[tokio::test]
    async fn delete_passkey_validates_input() {
        let temp = TempDir::new().unwrap();
        let controller = make_controller(&temp);

        // Empty four_words should fail with InvalidInput before checking session
        let result = controller.delete_passkey("").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::InvalidInput(_)));
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
