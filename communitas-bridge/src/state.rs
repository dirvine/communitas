//! Bridge server state management

use anyhow::Result;
use communitas_core::{CoreContext, types::DeviceType};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for the bridge server
#[derive(Clone)]
pub struct BridgeState {
    /// Core context with P2P networking
    pub core: Arc<RwLock<Option<CoreContext>>>,
}

impl BridgeState {
    /// Create new bridge state
    pub async fn new() -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            core: Arc::new(RwLock::new(None)),
        }))
    }

    /// Initialize core context with identity
    pub async fn initialize_core(
        &self,
        four_words: String,
        display_name: String,
        device_name: String,
    ) -> Result<()> {
        let mut core = self.core.write().await;

        // Validate four-word format to prevent path traversal attacks
        Self::validate_four_words(&four_words)?;

        // Get base storage directory from env var or use default
        let base_dir = std::env::var("BRIDGE_STORAGE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./bridge-data"));

        // Create per-user storage directory: ./bridge-data/{four-words}/
        let user_storage_dir = base_dir.join(&four_words);

        // Ensure directory exists with proper permissions
        std::fs::create_dir_all(&user_storage_dir).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create user storage directory {}: {}",
                user_storage_dir.display(),
                e
            )
        })?;

        tracing::info!(
            "Initializing user '{}' with isolated storage at: {}",
            four_words,
            user_storage_dir.display()
        );

        // Create CoreContext using communitas-core with per-user storage
        // Use Desktop device type for bridge server
        let ctx = CoreContext::initialize(
            four_words.clone(),
            display_name,
            device_name,
            DeviceType::Desktop,
            user_storage_dir,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize CoreContext: {}", e))?;

        *core = Some(ctx);
        Ok(())
    }

    /// Validate four-word format to prevent path traversal attacks
    fn validate_four_words(four_words: &str) -> Result<()> {
        // Check for path traversal attempts
        if four_words.contains("..") || four_words.contains('/') || four_words.contains('\\') {
            return Err(anyhow::anyhow!(
                "Invalid four-word format: contains invalid characters"
            ));
        }

        // Validate format: word-word-word-word
        let parts: Vec<&str> = four_words.split('-').collect();
        if parts.len() != 4 {
            return Err(anyhow::anyhow!(
                "Invalid four-word format: expected 4 words separated by hyphens, got {}",
                parts.len()
            ));
        }

        // Ensure each word is non-empty and contains only alphanumeric characters
        for (i, word) in parts.iter().enumerate() {
            if word.is_empty() {
                return Err(anyhow::anyhow!(
                    "Invalid four-word format: word {} is empty",
                    i + 1
                ));
            }
            if !word.chars().all(|c| c.is_alphanumeric()) {
                return Err(anyhow::anyhow!(
                    "Invalid four-word format: word {} contains non-alphanumeric characters",
                    i + 1
                ));
            }
        }

        Ok(())
    }

    /// Get core context reference (returns error if not initialized)
    pub async fn get_core(&self) -> Result<tokio::sync::RwLockReadGuard<'_, Option<CoreContext>>> {
        Ok(self.core.read().await)
    }

    /// Check if core is initialized
    pub async fn is_initialized(&self) -> bool {
        self.core.read().await.is_some()
    }

    /// Start P2P networking for initialized core
    pub async fn start_networking(&self, preferred_port: Option<u16>) -> Result<(String, String)> {
        let mut core_guard = self.core.write().await;
        let core = core_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Core not initialized"))?;

        // Start networking with optional preferred port
        let connection_identity = core
            .start_networking(preferred_port)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start networking: {}", e))?;

        let listen_addr = core
            .listen_address
            .map(|a| a.to_string())
            .unwrap_or_else(|| "not-available".to_string());

        Ok((connection_identity, listen_addr))
    }
}
