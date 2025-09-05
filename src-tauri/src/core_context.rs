use saorsa_core::chat::ChatManager;
use saorsa_core::identity::IdentityManager;
use saorsa_core::identity::enhanced::{DeviceType, EnhancedIdentity, EnhancedIdentityManager};
use saorsa_core::identity::manager::IdentityManagerConfig;
use saorsa_core::messaging::DhtClient;
use saorsa_core::messaging::service::MessagingService;
use saorsa_core::storage::StorageManager;
use saorsa_core::{
    dht::core_engine::{DhtCoreEngine, NodeId},
    identity::FourWordAddress,
};
use std::collections::HashMap;

// Group key storage for membership updates
use saorsa_core::api::GroupKeyPair;

/// Centralized context that wires Communitas to saorsa-core components.
/// This avoids re-implementations in this repo and delegates to saorsa-core.
pub struct CoreContext {
    pub four_words: String,
    pub identity: EnhancedIdentity,
    pub storage: StorageManager,
    pub chat: ChatManager,
    pub messaging: MessagingService,
    pub dht_client: DhtClient,
    pub group_keys: HashMap<String, GroupKeyPair>,
}

impl CoreContext {
    /// Build a new CoreContext from a four-word identity and display/device info.
    pub async fn initialize(
        four_words: String,
        display_name: String,
        device_name: String,
        device_type: DeviceType,
    ) -> Result<Self, String> {
        // Basic validation of four-word address (delegate to saorsa-core when possible)
        let words: Vec<&str> = four_words.split('-').collect();
        if words.len() != 4 {
            return Err(
                "Four-word address must contain exactly 4 words separated by hyphens".to_string(),
            );
        }
        let word_array = [
            words[0].to_string(),
            words[1].to_string(),
            words[2].to_string(),
            words[3].to_string(),
        ];
        if !saorsa_core::fwid::fw_check(word_array) {
            return Err("Invalid four-word address format".to_string());
        }

        // Identity manager and base identity
        let id_mgr = IdentityManager::new(IdentityManagerConfig::default());
        let base = id_mgr
            .create_identity(display_name, four_words.clone(), None, None)
            .await
            .map_err(|e| format!("Failed to create identity: {}", e))?;

        // Enhanced identity (PQC + threshold-ready)
        let mut enhanced_mgr = EnhancedIdentityManager::new(id_mgr);
        let enhanced_identity = enhanced_mgr
            .create_enhanced_identity(base, device_name, device_type)
            .await
            .map_err(|e| format!("Failed to create enhanced identity: {}", e))?;

        // Messaging DHT client (single-node engine for now)
        let dht_client = DhtClient::new().map_err(|e| format!("DHT init failed: {}", e))?;

        // Storage manager requires a DHT engine instance
        let dht_engine = DhtCoreEngine::new(NodeId::from_bytes([42u8; 32]))
            .map_err(|e| format!("DHT engine creation failed: {}", e))?;
        let storage = StorageManager::new(dht_engine, &enhanced_identity)
            .map_err(|e| format!("Storage init failed: {}", e))?;

        // Chat manager backed by storage and identity
        // Note: StorageManager doesn't implement Clone, so we create a new instance
        let dht_engine_chat = DhtCoreEngine::new(NodeId::from_bytes([43u8; 32]))
            .map_err(|e| format!("DHT engine creation for chat failed: {}", e))?;
        let storage_chat = StorageManager::new(dht_engine_chat, &enhanced_identity)
            .map_err(|e| format!("Storage init for chat failed: {}", e))?;
        let chat = ChatManager::new(storage_chat, enhanced_identity.clone());

        // Messaging service
        let messaging =
            MessagingService::new(FourWordAddress(four_words.clone()), dht_client.clone())
                .await
                .map_err(|e| format!("Messaging init failed: {}", e))?;

        Ok(Self {
            four_words,
            identity: enhanced_identity,
            storage,
            chat,
            messaging,
            dht_client,
            group_keys: HashMap::new(),
        })
    }
}
// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.
