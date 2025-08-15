// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


//! Communitas Identity Manager
//!
//! This module provides high-level identity management for the Communitas app,
//! integrating the saorsa-core identity system with secure platform storage.

use super::{SecureStorage, SecureStorageFactory, KeyMetadata};
use anyhow::{Result, Context};
use saorsa_core::identity::{
    manager::IdentityManager as CoreIdentityManager,
    NodeIdentity, 
    FourWordAddress,
};
use saorsa_core::config::Config;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// High-level identity information for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityInfo {
    /// Four-word human-readable address
    pub four_word_address: String,
    /// Display name (if set)
    pub display_name: Option<String>,
    /// When the identity was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Whether this is the primary identity
    pub is_primary: bool,
    /// Current verification status
    pub verification_status: String,
    /// Public key (for sharing)
    pub public_key_hex: String,
}

/// Identity generation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityGenerationParams {
    /// Optional display name
    pub display_name: Option<String>,
    /// Whether to use hardware entropy (slower but more secure)
    pub use_hardware_entropy: bool,
    /// Proof of work difficulty (0 = no PoW, higher = more secure but slower)
    pub pow_difficulty: u32,
}

impl Default for IdentityGenerationParams {
    fn default() -> Self {
        Self {
            display_name: None,
            use_hardware_entropy: true,
            pow_difficulty: 8, // Moderate difficulty
        }
    }
}

/// Main identity manager for Communitas
pub struct CommunidentityManager {
    /// Secure storage backend
    storage: Arc<dyn SecureStorage>,
    /// Core identity manager from saorsa-core
    core_manager: Arc<RwLock<Option<CoreIdentityManager>>>,
    /// Application configuration
    config: Config,
    /// Currently active identity
    current_identity: Arc<RwLock<Option<IdentityInfo>>>,
}

impl CommunidentityManager {
    /// Create new identity manager
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Communitas Identity Manager");
        
        // Create secure storage backend
        let storage = SecureStorageFactory::create_default().await
            .context("Failed to initialize secure storage")?;
        
        let storage_info = storage.get_storage_info().await?;
        info!("Using storage backend: {} with {} keys", 
              storage_info.backend_type, 
              storage_info.key_count);
        
        let manager = Self {
            storage: storage,
            core_manager: Arc::new(RwLock::new(None)),
            config,
            current_identity: Arc::new(RwLock::new(None)),
        };
        
        // Try to load existing identity
        if let Err(e) = manager.load_primary_identity().await {
            warn!("Failed to load existing identity: {}. User will need to create or restore one.", e);
        }
        
        Ok(manager)
    }
    
    /// Generate a new identity
    pub async fn generate_identity(&self, params: IdentityGenerationParams) -> Result<IdentityInfo> {
        info!("Generating new identity with parameters: {:?}", params);
        
        // Generate using saorsa-core
        let node_identity = if params.use_hardware_entropy {
            NodeIdentity::generate(params.pow_difficulty)
                .context("Failed to generate identity with hardware entropy")?
        } else {
            NodeIdentity::generate(params.pow_difficulty)
                .context("Failed to generate identity")?
        };
        
        // Create four-word address
        let four_word_address = FourWordAddress::from_bytes(node_identity.node_id().to_bytes())
            .context("Failed to create four-word address")?;
        
        let address_str = four_word_address.to_string();
        info!("Generated identity with address: {}", address_str);
        
        // Create identity info
        let identity_info = IdentityInfo {
            four_word_address: address_str.clone(),
            display_name: params.display_name.clone(),
            created_at: chrono::Utc::now(),
            is_primary: true, // First identity is primary
            verification_status: "Generated".to_string(),
            public_key_hex: hex::encode(node_identity.public_key().as_bytes()),
        };
        
        // Store securely
        self.store_identity(&node_identity, &identity_info).await
            .context("Failed to store generated identity")?;
        
        // Set as current identity
        {
            let mut current = self.current_identity.write().await;
            *current = Some(identity_info.clone());
        }
        
        info!("Successfully generated and stored identity: {}", address_str);
        Ok(identity_info)
    }
    
    /// Get current identity information
    pub async fn get_current_identity(&self) -> Option<IdentityInfo> {
        let current = self.current_identity.read().await;
        current.clone()
    }
    
    /// List all stored identities
    pub async fn list_identities(&self) -> Result<Vec<IdentityInfo>> {
        let keys = self.storage.list_keys().await?;
        let mut identities = Vec::new();
        
        for (key_id, metadata) in keys {
            if metadata.key_type == "identity" {
                // Try to reconstruct identity info from metadata
                if let Some(address) = metadata.four_word_address {
                    let identity_info = IdentityInfo {
                        four_word_address: address,
                        display_name: Some(metadata.description),
                        created_at: chrono::Utc::now(), // TODO: Store creation time in metadata
                        is_primary: key_id.contains("primary"),
                        verification_status: "Stored".to_string(),
                        public_key_hex: "".to_string(), // TODO: Store public key in metadata
                    };
                    identities.push(identity_info);
                }
            }
        }
        
        Ok(identities)
    }
    
    /// Check if an identity exists
    pub async fn has_identity(&self) -> Result<bool> {
        let identities = self.list_identities().await?;
        Ok(!identities.is_empty())
    }
    
    /// Load primary identity from storage
    async fn load_primary_identity(&self) -> Result<()> {
        // Look for primary identity key
        let keys = self.storage.list_keys().await?;
        
        for (key_id, metadata) in keys {
            if metadata.key_type == "identity" && key_id.contains("primary") {
                if let Some(address) = metadata.four_word_address {
                    let identity_info = IdentityInfo {
                        four_word_address: address,
                        display_name: Some(metadata.description),
                        created_at: chrono::Utc::now(),
                        is_primary: true,
                        verification_status: "Loaded".to_string(),
                        public_key_hex: "".to_string(),
                    };
                    
                    let mut current = self.current_identity.write().await;
                    *current = Some(identity_info);
                    
                    info!("Loaded primary identity from storage");
                    return Ok(());
                }
            }
        }
        
        anyhow::bail!("No primary identity found in storage")
    }
    
    /// Store identity securely
    async fn store_identity(&self, node_identity: &NodeIdentity, identity_info: &IdentityInfo) -> Result<()> {
        // Create storage key ID
        let key_id = if identity_info.is_primary {
            "primary_identity".to_string()
        } else {
            format!("identity_{}", identity_info.four_word_address.replace("-", "_"))
        };
        
        // Serialize the node identity
        let identity_data = serde_json::to_vec(&node_identity.export())
            .context("Failed to serialize identity")?;
        
        // Create metadata
        let metadata = KeyMetadata {
            description: identity_info.display_name.clone()
                .unwrap_or_else(|| "Communitas Identity".to_string()),
            key_type: "identity".to_string(),
            algorithm: "Ed25519".to_string(),
            four_word_address: Some(identity_info.four_word_address.clone()),
        };
        
        // Store securely
        self.storage.store_key(&key_id, &identity_data, metadata).await
            .context("Failed to store identity in secure storage")?;
        
        Ok(())
    }
    
    /// Validate a four-word address format
    pub fn validate_four_word_address(address: &str) -> Result<()> {
        FourWordAddress::from_str(address)
            .context("Invalid four-word address format")?;
        Ok(())
    }
    
    /// Get storage backend information
    pub async fn get_storage_info(&self) -> Result<super::StorageInfo> {
        self.storage.get_storage_info().await
    }
}

// Implement Debug manually to avoid exposing sensitive data
impl std::fmt::Debug for CommunidentityManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommunidentityManager")
            .field("storage_type", &"SecureStorage")
            .field("has_core_manager", &self.core_manager.try_read().map(|m| m.is_some()).unwrap_or(false))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_core::config::Config;
    
    #[tokio::test]
    async fn test_identity_manager_creation() {
        let config = Config::development();
        let manager = CommunidentityManager::new(config).await.unwrap();
        
        let storage_info = manager.get_storage_info().await.unwrap();
        assert!(!storage_info.backend_type.is_empty());
        assert!(storage_info.is_available);
    }
    
    #[tokio::test]
    async fn test_identity_generation() {
        let config = Config::development();
        let manager = CommunidentityManager::new(config).await.unwrap();
        
        let params = IdentityGenerationParams {
            display_name: Some("Test Identity".to_string()),
            use_hardware_entropy: false, // Faster for tests
            pow_difficulty: 4, // Lower difficulty for tests
        };
        
        let identity = manager.generate_identity(params).await.unwrap();
        assert!(!identity.four_word_address.is_empty());
        assert_eq!(identity.display_name, Some("Test Identity".to_string()));
        assert!(identity.is_primary);
        
        // Should be able to retrieve it
        let current = manager.get_current_identity().await;
        assert!(current.is_some());
        assert_eq!(current.unwrap().four_word_address, identity.four_word_address);
    }
    
    #[test]
    fn test_four_word_address_validation() {
        // Valid addresses
        assert!(CommunidentityManager::validate_four_word_address("warm-ocean-gentle-breeze").is_ok());
        
        // Invalid addresses
        assert!(CommunidentityManager::validate_four_word_address("").is_err());
        assert!(CommunidentityManager::validate_four_word_address("too-few-words").is_err());
        assert!(CommunidentityManager::validate_four_word_address("too-many-words-here-now").is_err());
    }
}
