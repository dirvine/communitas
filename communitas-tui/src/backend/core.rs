use anyhow::Result;
use communitas_core::{
    AuthService, CoreContext, SessionInfo,
    encrypted_storage::{EncryptedStorageManager, RecentIdentity, StorageConfig},
};
use saorsa_core::identity::enhanced::DeviceType;
use std::path::PathBuf;

/// Backend wrapper around CoreContext and AuthService
pub struct Backend {
    /// Authentication service
    auth_service: AuthService,
    /// Core context (None if not authenticated)
    ctx: Option<CoreContext>,
    /// Data directory
    data_dir: PathBuf,
    /// Offline mode
    offline: bool,
}

impl Backend {
    /// Create new backend with AuthService
    pub async fn new(data_dir: PathBuf, offline: bool) -> Result<Self> {
        // Create storage configuration
        let config = StorageConfig {
            vault_dir: data_dir.join("vaults"),
            use_keyring: true,
            ..Default::default()
        };

        // Initialize encrypted storage manager
        let storage_manager = EncryptedStorageManager::new(config).await?;

        // Create auth service
        let auth_service = AuthService::new(storage_manager);

        Ok(Self {
            auth_service,
            ctx: None,
            data_dir,
            offline,
        })
    }

    /// Create new backend with custom configuration
    pub async fn new_with_config(
        data_dir: PathBuf,
        pbkdf2_iterations: u32,
        use_keyring: bool,
        offline: bool,
    ) -> Result<Self> {
        // Create storage configuration with custom values
        let config = StorageConfig {
            vault_dir: data_dir.join("vaults"),
            pbkdf2_iterations,
            use_keyring,
            ..Default::default()
        };

        // Initialize encrypted storage manager
        let storage_manager = EncryptedStorageManager::new(config).await?;

        // Create auth service
        let auth_service = AuthService::new(storage_manager);

        Ok(Self {
            auth_service,
            ctx: None,
            data_dir,
            offline,
        })
    }

    // ========================================================================
    // Authentication Methods (delegated to AuthService)
    // ========================================================================

    /// Create a new vault and login
    pub async fn create_vault(
        &mut self,
        four_words: &str,
        password: &str,
        display_name: &str,
    ) -> Result<SessionInfo> {
        tracing::info!("Creating vault for: {}", four_words);

        // Create vault via auth service
        let _vault_id = self
            .auth_service
            .create_vault(four_words, password, display_name)
            .await?;

        // Login to the new vault
        let session_info = self
            .auth_service
            .login(four_words, password, Some("TUI"))
            .await?;

        tracing::info!("Vault created and logged in: {}", four_words);
        Ok(session_info)
    }

    /// Create a new vault and login with timeout
    pub async fn create_vault_with_timeout(
        &mut self,
        four_words: &str,
        password: &str,
        display_name: &str,
    ) -> Result<SessionInfo> {
        use tokio::time::{Duration, timeout};

        let result = timeout(
            Duration::from_secs(60), // 60 second timeout
            self.create_vault(four_words, password, display_name),
        )
        .await;

        match result {
            Ok(Ok(session_info)) => Ok(session_info),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow::anyhow!("Vault creation timed out after 60 seconds")),
        }
    }

    /// Login with four-word identity and password
    pub async fn login(&mut self, four_words: &str, password: &str) -> Result<SessionInfo> {
        tracing::info!("Logging in: {}", four_words);

        let session_info = self
            .auth_service
            .login(four_words, password, Some("TUI"))
            .await?;

        tracing::info!("Login successful: {}", four_words);
        Ok(session_info)
    }

    /// Logout current session
    pub async fn logout(&mut self) -> Result<()> {
        tracing::info!("Logging out");

        self.auth_service.logout().await?;
        self.ctx = None;

        tracing::info!("Logout successful");
        Ok(())
    }

    /// Get current session info
    pub fn get_current_session(&self) -> Option<SessionInfo> {
        self.auth_service.get_current_session()
    }

    /// Check if user is logged in
    pub fn is_logged_in(&self) -> bool {
        self.auth_service.is_logged_in()
    }

    /// Get list of recent identities
    pub async fn get_recent_identities(&self) -> Result<Vec<RecentIdentity>> {
        self.auth_service.get_recent_identities().await
    }

    /// Try auto-login with last used identity
    pub async fn try_auto_login(&mut self) -> Result<Option<SessionInfo>> {
        self.auth_service.try_auto_login().await
    }

    /// Switch to another identity
    pub async fn switch_identity(&mut self, four_words: &str) -> Result<SessionInfo> {
        tracing::info!("Switching to identity: {}", four_words);

        let session_info = self.auth_service.switch_identity(four_words).await?;

        // Clear CoreContext when switching
        self.ctx = None;

        tracing::info!("Switched to identity: {}", four_words);
        Ok(session_info)
    }

    /// Check if vault exists
    pub async fn vault_exists(&self, four_words: &str) -> Result<bool> {
        self.auth_service.vault_exists(four_words).await
    }

    /// Check if identity has passkey registered
    pub async fn has_passkey(&self, four_words: &str) -> Result<bool> {
        self.auth_service.passkey_has_passkey(four_words).await
    }

    // ========================================================================
    // CoreContext Integration
    // ========================================================================

    /// Initialize CoreContext with current session
    pub async fn initialize_core_context(&mut self) -> Result<()> {
        let session = self
            .auth_service
            .get_current_session()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        tracing::info!("Initializing CoreContext for: {}", session.four_words);

        let ctx = CoreContext::initialize(
            session.four_words.clone(),
            session.display_name.clone(),
            "TUI".to_string(),
            DeviceType::Desktop,
        )
        .await
        .map_err(|e| anyhow::anyhow!("CoreContext initialization failed: {}", e))?;

        self.ctx = Some(ctx);

        tracing::info!("CoreContext initialized successfully");
        Ok(())
    }

    /// Check if CoreContext is initialized
    pub fn is_core_initialized(&self) -> bool {
        self.ctx.is_some()
    }

    /// Get reference to CoreContext (returns error if not initialized)
    pub fn context(&self) -> Result<&CoreContext> {
        self.ctx
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("CoreContext not initialized"))
    }

    /// Get mutable reference to CoreContext (returns error if not initialized)
    pub fn context_mut(&mut self) -> Result<&mut CoreContext> {
        self.ctx
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("CoreContext not initialized"))
    }

    /// Get current identity four-words
    pub fn identity(&self) -> Option<String> {
        self.get_current_session().map(|s| s.four_words)
    }

    // ========================================================================
    // Network / DHT Methods
    // ========================================================================

    /// Check DHT connection status
    pub async fn check_dht_connection(&self) -> Result<bool> {
        if self.offline {
            return Ok(false);
        }

        // Return true if context is initialized
        Ok(self.ctx.is_some())
    }

    /// Get bootstrap nodes
    pub async fn get_bootstrap_nodes(&self) -> Result<Vec<String>> {
        let ctx = self.context()?;

        if let Some(bootstrap_manager) = &ctx.bootstrap_manager {
            let mut nodes = bootstrap_manager.get_custom_nodes().await;
            let candidates = bootstrap_manager
                .get_bootstrap_candidates(10)
                .await
                .unwrap_or_default();
            nodes.extend(candidates);
            Ok(nodes)
        } else {
            Ok(vec![])
        }
    }

    /// Add bootstrap node
    pub async fn add_bootstrap_node(&self, node: String) -> Result<()> {
        let ctx = self.context()?;

        if let Some(bootstrap_manager) = &ctx.bootstrap_manager {
            bootstrap_manager.add_bootstrap_node(&node).await?;
        }

        Ok(())
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Generate a new four-word identity
    pub fn generate_four_words() -> String {
        use rand::RngCore;
        use rand::rngs::OsRng;
        use saorsa_core::address::NetworkAddress;
        use saorsa_core::fwid::fw_check;
        use std::net::Ipv4Addr;

        let mut rng = OsRng;
        const MIN_PORT: u16 = 1024;
        const PORT_SPAN: u32 = u16::MAX as u32 - MIN_PORT as u32 + 1;
        const GENERATION_ATTEMPTS: usize = 1000;

        for _ in 0..GENERATION_ATTEMPTS {
            let ipv4 = Ipv4Addr::from(rng.next_u32());
            let port = (rng.next_u32() % PORT_SPAN) as u16 + MIN_PORT;
            let candidate = NetworkAddress::from_ipv4(ipv4, port);

            if let Some(words) = candidate.four_words() {
                // Parse to ensure it's valid
                if let Ok(parsed) = saorsa_core::identity::FourWordAddress::parse_str(words) {
                    let words_vec = parsed.words();
                    // Try to convert to array of exactly 4 strings
                    let words_result: Result<[String; 4], _> = words_vec.try_into();
                    if let Ok(words_array) = words_result {
                        // Validate with saorsa-core
                        if fw_check(words_array.clone()) {
                            return words.to_string();
                        }
                    }
                }
            }
        }

        // Fallback: return a simple valid format (this should rarely happen)
        "ocean-forest-moon-star".to_string()
    }

    /// Validate four-word format
    pub fn validate_four_words(four_words: &str) -> bool {
        // Basic validation: exactly 4 words separated by hyphens
        let parts: Vec<&str> = four_words.split('-').collect();
        parts.len() == 4 && parts.iter().all(|p| !p.is_empty())
    }
}
