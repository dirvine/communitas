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

        // Create storage directory for bridge
        let storage_dir = PathBuf::from("./bridge-data");

        // Create CoreContext using communitas-core
        // Use Desktop device type for bridge server
        let ctx = CoreContext::initialize(
            four_words,
            display_name,
            device_name,
            DeviceType::Desktop,
            storage_dir,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize CoreContext: {}", e))?;

        *core = Some(ctx);
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
}
