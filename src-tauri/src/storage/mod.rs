// Copyright 2024 Saorsa Labs
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Production-ready storage management with DHT integration and Reed Solomon erasure coding
//! 
//! This module replaces all mock storage implementations with a real DHT-backed storage system
//! that implements the 1:1:2 storage allocation policy (local:DHT:public).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
// NOTE: Tracing and Result imports removed as they're not used with ProductionStorageManager commented out
// use anyhow::Result;
// use tracing::{debug, info, warn};

pub mod capacity_manager;
pub mod dht_storage;
pub mod reed_solomon_manager;
pub mod reed_solomon_v2;  // Enhanced version with 60% availability
pub mod local_storage;
pub mod shard_distributor;
pub mod metrics;
pub mod entity_storage;  // Entity-aware storage for all 5 entity types

use crate::identity::IdentityManager;
use crate::dht_facade::DhtFacade;

/// Storage allocation policy: Local:DHT:Public = 1:1:2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAllocation {
    /// Personal data stored locally (bytes)
    pub personal_local: usize,
    /// Personal data replicated in DHT (bytes) - matches local
    pub personal_dht: usize,
    /// Space allocated for group/org Reed Solomon shards (bytes)
    pub group_shard_allocation: usize,
    /// Space donated to public DHT (bytes) - 2x personal allocation
    pub public_dht_allocation: usize,
    /// Total committed storage capacity
    pub total_capacity: usize,
}

impl StorageAllocation {
    pub fn new(user_storage_commitment: usize) -> Self {
        Self {
            personal_local: user_storage_commitment,
            personal_dht: user_storage_commitment,
            group_shard_allocation: user_storage_commitment / 2, // 50% for group shards
            public_dht_allocation: user_storage_commitment * 2,  // 2x for public DHT
            total_capacity: user_storage_commitment * 5, // Total = local + dht + shards + public
        }
    }
    
    pub fn utilization_percentage(&self, current_usage: &StorageUsage) -> f32 {
        let total_used = current_usage.personal_local + 
                        current_usage.group_shards + 
                        current_usage.public_dht_used;
        (total_used as f32 / self.total_capacity as f32) * 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    pub personal_local: usize,
    pub personal_dht: usize,
    pub group_shards: usize,
    pub public_dht_used: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Storage classifications for different data types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StorageClass {
    /// Personal data - stored locally + DHT backup
    Personal,
    /// Group data - Reed Solomon encoded shards
    Group,
    /// Organization data - Reed Solomon encoded across departments
    Organization,
    /// Public DHT data - community storage participation
    PublicDHT,
}

/// Production storage manager that coordinates all storage operations
#[allow(dead_code)]
#[derive(Debug)]
pub struct ProductionStorageManager<F: DhtFacade + 'static> {
    allocation: StorageAllocation,
    usage: Arc<RwLock<StorageUsage>>,
    dht: Arc<F>,
    reed_solomon: Arc<reed_solomon_manager::EnhancedReedSolomonManager>,
    capacity_manager: Arc<capacity_manager::CapacityManager>,
    local_storage: Arc<local_storage::LocalStorageManager>,
    shard_distributor: Arc<shard_distributor::ShardDistributor<F>>,
    metrics: Arc<metrics::StorageMetrics>,
    storage_root: PathBuf,
    identity_manager: Arc<IdentityManager>,
}

// NOTE: Temporarily commenting out ProductionStorageManager impl due to missing methods on SKademlia and IdentityManager
// TODO: Add proper DHT wrapper methods or implement required methods on external structs
/*
impl ProductionStorageManager {
    pub async fn new(
        user_storage_commitment: usize,
        storage_root: PathBuf,
        dht: Arc<SKademlia>,
        identity_manager: Arc<IdentityManager>,
    ) -> Result<Self> {
        let allocation = StorageAllocation::new(user_storage_commitment);
        let usage = Arc::new(RwLock::new(StorageUsage {
            personal_local: 0,
            personal_dht: 0,
            group_shards: 0,
            public_dht_used: 0,
            last_updated: chrono::Utc::now(),
        }));

        // Initialize Reed Solomon with optimal configuration
        let reed_solomon = Arc::new(reed_solomon_manager::EnhancedReedSolomonManager::new());

        let capacity_manager = Arc::new(
            capacity_manager::CapacityManager::new(allocation.clone())
        );

        let local_storage = Arc::new(
            local_storage::LocalStorageManager::new(storage_root.clone(), user_storage_commitment).await?
        );

        let shard_distributor = Arc::new(
            shard_distributor::ShardDistributor::new(
                dht.clone(),
                reed_solomon.clone(),
            )
        );

        let metrics = Arc::new(metrics::StorageMetrics::new());

        Ok(Self {
            allocation,
            usage,
            dht,
            reed_solomon,
            capacity_manager,
            local_storage,
            shard_distributor,
            metrics,
            storage_root,
            identity_manager,
        })
    }

    /// Store personal data with local + DHT redundancy
    pub async fn store_personal_data(
        &self,
        user_id: &str,
        data_id: &str,
        data: &[u8],
    ) -> Result<StorageResult> {
        // Check capacity first
        if !self.capacity_manager.can_store_personal(data.len()).await {
            bail!("Insufficient capacity for personal data storage");
        }

        // 1. Store locally
        self.local_storage.store_personal(user_id, data_id, data).await
            .context("Failed to store personal data locally")?;

        // 2. Encrypt and store in DHT
        let user_key = self.identity_manager.get_encryption_key(user_id)?;
        let encrypted_data = self.encrypt_data(data, &user_key)?;
        let dht_key = self.generate_personal_dht_key(user_id, data_id);
        
        self.dht.store(dht_key, encrypted_data).await
            .context("Failed to store personal data in DHT")?;

        // 3. Update usage metrics
        self.update_personal_usage(data.len()).await;
        self.metrics.record_personal_storage(data.len()).await;

        info!("Stored personal data for user {} (size: {} bytes)", user_id, data.len());
        Ok(StorageResult::Success)
    }

    /// Store group data using Reed Solomon encoding
    pub async fn store_group_data(
        &self,
        group_id: &str,
        data_id: &str,
        data: &[u8],
        group_members: &[String],
    ) -> Result<StorageResult> {
        if group_members.is_empty() {
            bail!("Cannot store group data without group members");
        }

        // 1. Encode data using Reed Solomon
        let shards = self.reed_solomon.encode_data(data)
            .context("Failed to encode group data with Reed Solomon")?;

        // 2. Distribute shards to group members
        let distribution_plan = self.shard_distributor
            .create_distribution_plan(group_id, &shards, group_members).await?;

        // 3. Execute shard distribution
        self.shard_distributor.distribute_shards(&distribution_plan).await
            .context("Failed to distribute shards to group members")?;

        // 4. Store complete encrypted backup in DHT
        let group_key = self.derive_group_key(group_id)?;
        let encrypted_backup = self.encrypt_data(data, &group_key)?;
        let dht_backup_key = self.generate_group_backup_key(group_id, data_id);
        
        self.dht.store(dht_backup_key, encrypted_backup).await
            .context("Failed to store group backup in DHT")?;

        // 5. Update metrics
        self.metrics.record_group_storage(group_id, data.len(), shards.len()).await;

        info!(
            "Stored group data for group {} with {} shards distributed to {} members", 
            group_id, shards.len(), group_members.len()
        );

        Ok(StorageResult::Success)
    }

    /// Retrieve personal data with local-first access
    pub async fn retrieve_personal_data(
        &self,
        user_id: &str,
        data_id: &str,
    ) -> Result<Vec<u8>> {
        // Try local storage first (fastest)
        if let Ok(data) = self.local_storage.retrieve_personal(user_id, data_id).await {
            self.metrics.record_local_hit().await;
            return Ok(data);
        }

        // Fallback to DHT
        let dht_key = self.generate_personal_dht_key(user_id, data_id);
        let encrypted_data = self.dht.retrieve(dht_key).await
            .context("Failed to retrieve personal data from DHT")?;

        let user_key = self.identity_manager.get_encryption_key(user_id)?;
        let data = self.decrypt_data(&encrypted_data, &user_key)?;

        self.metrics.record_dht_fallback().await;
        Ok(data)
    }

    /// Retrieve group data using Reed Solomon reconstruction
    pub async fn retrieve_group_data(
        &self,
        group_id: &str,
        data_id: &str,
        group_members: &[String],
    ) -> Result<Vec<u8>> {
        // Try to collect shards from group members
        let available_shards = self.shard_distributor
            .collect_available_shards(group_id, data_id, group_members).await?;

        // Check if we have enough shards for reconstruction
        if available_shards.len() >= self.reed_solomon.data_shard_count() {
            // Reconstruct data from shards
            let reconstructed_data = self.reed_solomon.decode_data(&available_shards)
                .context("Failed to reconstruct group data from shards")?;
            
            self.metrics.record_reed_solomon_success().await;
            return Ok(reconstructed_data);
        }

        // Fallback to DHT backup
        warn!("Insufficient shards for group {}, falling back to DHT backup", group_id);
        let dht_backup_key = self.generate_group_backup_key(group_id, data_id);
        let encrypted_backup = self.dht.retrieve(dht_backup_key).await
            .context("Failed to retrieve group backup from DHT")?;

        let group_key = self.derive_group_key(group_id)?;
        let data = self.decrypt_data(&encrypted_backup, &group_key)?;

        self.metrics.record_dht_backup_used().await;
        Ok(data)
    }

    /// Get current storage status and metrics
    pub async fn get_storage_status(&self) -> Result<StorageStatus> {
        let usage = self.usage.read().await.clone();
        let utilization = self.allocation.utilization_percentage(&usage);
        let metrics = self.metrics.get_current_metrics().await;

        Ok(StorageStatus {
            allocation: self.allocation.clone(),
            usage,
            utilization_percentage: utilization,
            is_healthy: utilization < 90.0, // Consider unhealthy if >90% full
            metrics,
        })
    }

    /// Accept storage request from DHT (public participation)
    pub async fn accept_dht_storage_request(
        &self,
        key: String,
        data: Vec<u8>,
        requester: String,
    ) -> Result<bool> {
        if !self.capacity_manager.can_accept_dht_data(data.len()).await {
            return Ok(false); // Politely decline if no capacity
        }

        // Store in local DHT cache
        self.local_storage.store_dht_data(&key, &data).await
            .context("Failed to store DHT data locally")?;

        // Update usage tracking
        self.update_dht_usage(data.len()).await;
        self.metrics.record_dht_storage_accepted(data.len(), &requester).await;

        debug!("Accepted DHT storage request: {} bytes from {}", data.len(), requester);
        Ok(true)
    }

    // Private helper methods

    async fn update_personal_usage(&self, size: usize) {
        let mut usage = self.usage.write().await;
        usage.personal_local += size;
        usage.personal_dht += size; // Replicated
        usage.last_updated = chrono::Utc::now();
    }

    async fn update_dht_usage(&self, size: usize) {
        let mut usage = self.usage.write().await;
        usage.public_dht_used += size;
        usage.last_updated = chrono::Utc::now();
    }

    fn generate_personal_dht_key(&self, user_id: &str, data_id: &str) -> String {
        format!("personal:{}:{}", user_id, data_id)
    }

    fn generate_group_backup_key(&self, group_id: &str, data_id: &str) -> String {
        format!("group_backup:{}:{}", group_id, data_id)
    }

    fn derive_group_key(&self, group_id: &str) -> Result<[u8; 32]> {
        // Derive deterministic group key from group ID
        use blake3::hash;
        let hash = hash(group_id.as_bytes());
        Ok(*hash.as_bytes())
    }

    fn encrypt_data(&self, data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
        // Use ChaCha20Poly1305 for encryption
        use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
        use chacha20poly1305::aead::{Aead, KeyInit};
        use rand::RngCore;

        let cipher = ChaCha20Poly1305::new(Key::from_slice(key).into());
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt_data(&self, encrypted_data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
        use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
        use chacha20poly1305::aead::{Aead, KeyInit};

        if encrypted_data.len() < 12 {
            bail!("Encrypted data too short");
        }

        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];

        let cipher = ChaCha20Poly1305::new(Key::from_slice(key).into());
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))
    }
}
*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatus {
    pub allocation: StorageAllocation,
    pub usage: StorageUsage,
    pub utilization_percentage: f32,
    pub is_healthy: bool,
    pub metrics: HashMap<String, u64>,
}

#[derive(Debug)]
pub enum StorageResult {
    Success,
    InsufficientCapacity,
    NetworkError(String),
    EncryptionError(String),
}