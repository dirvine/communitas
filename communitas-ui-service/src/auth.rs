use async_trait::async_trait;
use communitas_core::ui_core::{CommunitasApi, UiSessionInfo, recover_identity_from_mnemonic};
use std::env;
use thiserror::Error;
use tokio::sync::{RwLock, watch};
use tracing::{info, instrument, warn};

use crate::storage::{StorageError, UiStorage};

/// Snapshot of authentication/session state.
#[derive(Debug, Clone)]
pub enum AuthStateSnapshot {
    LoggedOut,
    Authenticating,
    Authenticated(AuthSession),
}

/// Information about the active identity/session.
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub pubkey_hex: String,
    pub four_words: String,
    pub display_name: String,
    pub device_name: String,
}

impl From<(UiSessionInfo, String)> for AuthSession {
    fn from((info, device_name): (UiSessionInfo, String)) -> Self {
        Self {
            pubkey_hex: info.pubkey_hex,
            four_words: info.four_words,
            display_name: info.display_name,
            device_name,
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
        four_words: &str,
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
    failure_mode: AuthFailureMode,
}

impl AuthController {
    pub fn new(storage: UiStorage) -> Result<Self, AuthError> {
        let (state_tx, _) = watch::channel(AuthStateSnapshot::LoggedOut);
        let failure_mode = AuthFailureMode::from_env();
        Ok(Self {
            storage,
            device_name: format!("Communitas-{}", whoami::devicename()),
            inner: RwLock::new(AuthInner {
                api: None,
                session: None,
            }),
            state_tx,
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

    async fn start_network(api: &CommunitasApi) {
        if let Err(err) = api.gossip_start(None).await {
            warn!(
                target = "ui.auth",
                "failed to start gossip (non-fatal): {err}"
            );
        } else {
            info!(target = "ui.auth", "gossip runtime started");
        }
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

        Self::start_network(&api).await;

        let session = AuthSession::from((session_info, self.device_name.clone()));
        self.set_session(api, session.clone()).await;
        self.set_state(AuthStateSnapshot::Authenticated(session.clone()));
        info!(target = "ui.auth", "login complete");
        Ok(session)
    }

    #[instrument(name = "ui.auth.create", skip(self, password), fields(identity = %redact_identity(four_words), display_name))]
    async fn create_identity(
        &self,
        four_words: &str,
        display_name: &str,
        password: &str,
    ) -> Result<AuthSession, AuthError> {
        self.fail_if_requested()?;
        let four_words = self.ensure_four_words(four_words)?;
        let display_name = self.ensure_display_name(display_name)?;
        let password = self.ensure_password(password)?;

        self.set_state(AuthStateSnapshot::Authenticating);

        let api = CommunitasApi::create(
            four_words.to_string(),
            display_name.to_string(),
            self.device_name.clone(),
            self.storage_path()?,
        )
        .await
        .map_err(AuthError::Core)?;

        api.auth_create_vault(
            four_words.to_string(),
            display_name.to_string(),
            password.to_string(),
        )
        .await
        .map_err(AuthError::Core)?;

        let session_info = api
            .auth_login(four_words.to_string(), password.to_string())
            .await
            .map_err(AuthError::Core)?;

        Self::start_network(&api).await;

        let session = AuthSession::from((session_info, self.device_name.clone()));
        self.set_session(api, session.clone()).await;
        self.set_state(AuthStateSnapshot::Authenticated(session.clone()));
        info!(target = "ui.auth", "identity created");
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

        Self::start_network(&api).await;

        let session = AuthSession::from((session_info, self.device_name.clone()));
        self.set_session(api, session.clone()).await;
        self.set_state(AuthStateSnapshot::Authenticated(session.clone()));
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
}
