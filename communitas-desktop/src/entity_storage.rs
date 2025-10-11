// Entity storage with threshold encryption via saorsa-seal/saorsa-fec

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use saorsa_fec::{FecCodec, FecParams};
use saorsa_seal::{
    EnvelopeKind,
    Recipient,
    RecipientId,
    SealPolicy,
    seal_bytes,
    unseal
};
use communitas_core::storage::reed_solomon_manager::DhtStorage;
use communitas_core::keystore::Keystore;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Individual,
    Group,
    Channel,
    Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub k: usize,  // Minimum shards needed for reconstruction
    pub m: usize,  // Additional redundancy shards
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResult {
    pub success: bool,
    pub path: String,
    pub shard_count: usize,
    pub encryption_status: EncryptionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionStatus {
    pub enabled: bool,
    pub threshold: ThresholdConfig,
    pub available_shards: usize,
    pub health_status: String, // "healthy", "degraded", "critical"
}

pub struct EntityStorage {
    keystore: Arc<Keystore>,
    dht: Arc<dyn DhtStorage>,
}

impl EntityStorage {
    pub fn new(keystore: Arc<Keystore>, dht: Arc<dyn DhtStorage>) -> Self {
        Self { keystore, dht }
    }

    pub fn get_threshold_config(entity_type: &EntityType) -> ThresholdConfig {
        match entity_type {
            EntityType::Individual => ThresholdConfig { k: 2, m: 1 }, // 2-of-3
            EntityType::Group => ThresholdConfig { k: 3, m: 2 },      // 3-of-5
            EntityType::Channel => ThresholdConfig { k: 4, m: 2 },    // 4-of-6
            EntityType::Project => ThresholdConfig { k: 5, m: 3 },    // 5-of-8
        }
    }

    pub async fn write_file_sealed(
        &self,
        entity_id: String,
        entity_type: EntityType,
        path: String,
        content: Vec<u8>,
        recipients: Vec<String>, // Four-word addresses
        threshold_override: Option<ThresholdConfig>,
    ) -> Result<StorageResult> {
        // 1. Get threshold config for entity
        let config = threshold_override.unwrap_or_else(||
            Self::get_threshold_config(&entity_type)
        );

        // 2. Apply FEC encoding
        let fec_params = FecParams {
            data_shards: config.k,
            parity_shards: config.m,
        };

        let codec = FecCodec::new(fec_params)?;
        let shards = codec.encode(&content)?;

        // 3. Seal each shard with recipient keys
        let policy = SealPolicy {
            envelope_kind: EnvelopeKind::ThresholdSharing,
            fec: saorsa_seal::FecParams {
                data_shards: config.k,
                parity_shards: config.m,
            },
        };

        let mut sealed_shards = Vec::new();
        for (i, shard) in shards.iter().enumerate() {
            // Convert four-word addresses to recipients
            let recipients_list: Vec<Recipient> = recipients
                .iter()
                .map(|fw| Recipient::from_four_words(fw))
                .collect::<Result<Vec<_>>>()?;

            let sealed = seal_bytes(
                shard,
                &recipients_list,
                policy.clone()
            )?;

            // Store in DHT with shard-specific key
            let shard_key = self.compute_shard_key(&entity_id, &path, i);
            self.dht.put(&shard_key, &sealed, None)?;
            sealed_shards.push(sealed);
        }

        // 4. Store metadata
        let metadata = ShardMetadata {
            entity_id: entity_id.clone(),
            path: path.clone(),
            threshold: config.clone(),
            shard_count: shards.len(),
            created_at: chrono::Utc::now(),
            recipients,
        };

        let metadata_key = self.compute_metadata_key(&entity_id, &path);
        let metadata_bytes = serde_json::to_vec(&metadata)?;
        self.dht.put(&metadata_key, &metadata_bytes, None)?;

        Ok(StorageResult {
            success: true,
            path,
            shard_count: shards.len(),
            encryption_status: EncryptionStatus {
                enabled: true,
                threshold: config,
                available_shards: shards.len(),
                health_status: "healthy".to_string(),
            },
        })
    }

    pub async fn read_file_sealed(
        &self,
        entity_id: String,
        path: String,
        private_key: Vec<u8>,
    ) -> Result<Vec<u8>> {
        // 1. Retrieve metadata
        let metadata_key = self.compute_metadata_key(&entity_id, &path);
        let metadata_bytes = self.dht.get(&metadata_key)?;
        let metadata: ShardMetadata = serde_json::from_slice(&metadata_bytes)?;

        // 2. Retrieve and unseal shards
        let mut unsealed_shards = Vec::new();
        let mut shard_indices = Vec::new();

        for i in 0..metadata.shard_count {
            let shard_key = self.compute_shard_key(&entity_id, &path, i);

            if let Ok(sealed_shard) = self.dht.get(&shard_key) {
                if let Ok(data) = unseal(&sealed_shard, &private_key) {
                    unsealed_shards.push(data);
                    shard_indices.push(i);

                    // Stop once we have k shards
                    if unsealed_shards.len() >= metadata.threshold.k {
                        break;
                    }
                }
            }
        }

        // 3. Check if we have enough shards
        if unsealed_shards.len() < metadata.threshold.k {
            bail!(
                "Insufficient shards for reconstruction: {} available, {} required",
                unsealed_shards.len(),
                metadata.threshold.k
            );
        }

        // 4. Reconstruct original data using FEC
        let fec_params = FecParams {
            data_shards: metadata.threshold.k,
            parity_shards: metadata.threshold.m,
        };

        let codec = FecCodec::new(fec_params)?;
        let content = codec.decode(&unsealed_shards, &shard_indices)?;

        Ok(content)
    }

    pub async fn get_encryption_status(
        &self,
        entity_id: String,
        path: String,
    ) -> Result<EncryptionStatus> {
        // Retrieve metadata
        let metadata_key = self.compute_metadata_key(&entity_id, &path);
        let metadata_bytes = self.dht.get(&metadata_key)?;
        let metadata: ShardMetadata = serde_json::from_slice(&metadata_bytes)?;

        // Count available shards
        let mut available = 0;
        for i in 0..metadata.shard_count {
            let shard_key = self.compute_shard_key(&entity_id, &path, i);
            if self.dht.get(&shard_key).is_ok() {
                available += 1;
            }
        }

        // Determine health status
        let health_status = if available >= metadata.threshold.k + metadata.threshold.m {
            "healthy"
        } else if available >= metadata.threshold.k {
            "degraded"
        } else {
            "critical"
        };

        Ok(EncryptionStatus {
            enabled: true,
            threshold: metadata.threshold,
            available_shards: available,
            health_status: health_status.to_string(),
        })
    }

    fn compute_shard_key(&self, entity_id: &str, path: &str, index: usize) -> [u8; 32] {
        let key_str = format!("{}/storage/{}/shard/{}", entity_id, path, index);
        let hash = blake3::hash(key_str.as_bytes());
        *hash.as_bytes()
    }

    fn compute_metadata_key(&self, entity_id: &str, path: &str) -> [u8; 32] {
        let key_str = format!("{}/storage/{}/metadata", entity_id, path);
        let hash = blake3::hash(key_str.as_bytes());
        *hash.as_bytes()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShardMetadata {
    entity_id: String,
    path: String,
    threshold: ThresholdConfig,
    shard_count: usize,
    created_at: chrono::DateTime<chrono::Utc>,
    recipients: Vec<String>,
}

// Tauri commands
#[tauri::command]
pub async fn core_entity_write_file_sealed(
    entity_storage: State<'_, Arc<RwLock<Option<EntityStorage>>>>,
    entity_id: String,
    entity_type: String,
    path: String,
    content: Vec<u8>,
    recipients: Vec<String>,
    threshold_k: Option<usize>,
    threshold_m: Option<usize>,
) -> Result<StorageResult, String> {
    let storage_guard = entity_storage.read().await;
    let storage = storage_guard.as_ref()
        .ok_or_else(|| "Entity storage not initialized".to_string())?;

    let entity_type = match entity_type.as_str() {
        "individual" => EntityType::Individual,
        "group" => EntityType::Group,
        "channel" => EntityType::Channel,
        "project" => EntityType::Project,
        _ => return Err("Invalid entity type".to_string()),
    };

    let threshold_override = match (threshold_k, threshold_m) {
        (Some(k), Some(m)) => Some(ThresholdConfig { k, m }),
        _ => None,
    };

    storage
        .write_file_sealed(entity_id, entity_type, path, content, recipients, threshold_override)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn core_entity_read_file_sealed(
    entity_storage: State<'_, Arc<RwLock<Option<EntityStorage>>>>,
    entity_id: String,
    path: String,
) -> Result<Vec<u8>, String> {
    let storage_guard = entity_storage.read().await;
    let storage = storage_guard.as_ref()
        .ok_or_else(|| "Entity storage not initialized".to_string())?;

    // Get private key from keystore
    let keystore = Keystore::new();
    let current_id = keystore.load_current_identity()
        .map_err(|e| format!("Failed to load identity: {}", e))?;
    let (_, sk_bytes) = keystore.load_mldsa_keys(&current_id)
        .map_err(|e| format!("Failed to load keys: {}", e))?;

    storage
        .read_file_sealed(entity_id, path, sk_bytes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn core_entity_get_encryption_status(
    entity_storage: State<'_, Arc<RwLock<Option<EntityStorage>>>>,
    entity_id: String,
    path: String,
) -> Result<EncryptionStatus, String> {
    let storage_guard = entity_storage.read().await;
    let storage = storage_guard.as_ref()
        .ok_or_else(|| "Entity storage not initialized".to_string())?;

    storage
        .get_encryption_status(entity_id, path)
        .await
        .map_err(|e| e.to_string())
}