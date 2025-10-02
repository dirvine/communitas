use anyhow::Result;
use communitas_core::CoreContext;
use saorsa_core::identity::enhanced::DeviceType;
use std::path::PathBuf;

/// Backend wrapper around CoreContext
pub struct Backend {
    /// Core context (None if not initialized)
    ctx: Option<CoreContext>,
    /// Data directory
    data_dir: PathBuf,
    /// Offline mode
    offline: bool,
}

impl Backend {
    /// Create new backend without initializing CoreContext
    pub fn new(data_dir: PathBuf, offline: bool) -> Self {
        Self {
            ctx: None,
            data_dir,
            offline,
        }
    }

    /// Initialize CoreContext with identity
    pub async fn initialize(
        &mut self,
        four_words: String,
        display_name: String,
        device_name: String,
    ) -> Result<()> {
        tracing::info!("Initializing CoreContext with identity: {}", four_words);

        let ctx = CoreContext::initialize(
            four_words,
            display_name,
            device_name,
            DeviceType::Desktop,
        )
        .await
        .map_err(|e| anyhow::anyhow!("CoreContext initialization failed: {}", e))?;

        self.ctx = Some(ctx);

        Ok(())
    }

    /// Initialize CoreContext with existing identity (for login)
    pub async fn initialize_identity(
        &mut self,
        four_words: &str,
        display_name: &str,
        device_name: &str,
    ) -> Result<()> {
        self.initialize(
            four_words.to_string(),
            display_name.to_string(),
            device_name.to_string(),
        )
        .await
    }

    /// Generate new identity (for signup)
    pub async fn generate_identity(
        &mut self,
        display_name: &str,
        device_name: &str,
    ) -> Result<String> {
        tracing::info!("Generating new identity for: {}", display_name);

        // Generate random four-word identity
        // For now, use simple word generation - TODO: Use proper four-word-networking crate
        use rand::Rng;
        let words = [
            "ocean", "forest", "mountain", "river", "cloud", "star", "moon", "sun",
            "wind", "rain", "snow", "fire", "earth", "water", "stone", "tree",
            "flower", "grass", "leaf", "branch", "root", "seed", "fruit", "berry",
        ];

        let mut rng = rand::thread_rng();
        let four_words = format!(
            "{}-{}-{}-{}",
            words[rng.gen_range(0..words.len())],
            words[rng.gen_range(0..words.len())],
            words[rng.gen_range(0..words.len())],
            words[rng.gen_range(0..words.len())]
        );

        // Initialize with the generated identity
        self.initialize(
            four_words.clone(),
            display_name.to_string(),
            device_name.to_string(),
        )
        .await?;

        Ok(four_words)
    }

    /// Check if CoreContext is initialized
    pub fn is_initialized(&self) -> bool {
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
    pub fn identity(&self) -> Option<&str> {
        self.ctx.as_ref().map(|c| c.four_words.as_str())
    }

    /// Check DHT connection status
    pub async fn check_dht_connection(&self) -> Result<bool> {
        if self.offline {
            return Ok(false);
        }

        // TODO: Implement actual DHT connection check
        // For now, return true if context is initialized
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
}
