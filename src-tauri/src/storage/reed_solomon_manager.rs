// Copyright 2024 Saorsa Labs
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Enhanced Reed Solomon manager for group and organization data

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use blake3;
use hex;
use saorsa_fec::{FecCodec, FecParams};
use saorsa_seal::{
    EnvelopeKind, ProvidedShare, Recipient, RecipientId, SealPolicy, open_bytes, seal_bytes,
};

// DHT interface for saorsa-seal
pub trait DhtStorage {
    fn put(&self, key: &[u8; 32], value: &[u8], ttl: Option<u64>) -> anyhow::Result<()>;
    fn get(&self, key: &[u8; 32]) -> anyhow::Result<Vec<u8>>;
}

/// Shard identifier and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shard {
    pub index: usize,
    pub shard_type: ShardType,
    pub data: Vec<u8>,
    pub group_id: String,
    pub data_id: String,
    pub integrity_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShardType {
    Data,   // Original data shard (k shards needed for reconstruction)
    Parity, // Redundancy shard (m additional shards for fault tolerance)
}

/// Reed Solomon configuration based on group size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedSolomonConfig {
    pub data_shards: usize,               // k
    pub parity_shards: usize,             // m
    pub shard_size: usize,                // bytes per shard
    pub group_size_range: (usize, usize), // min/max group members for this config
}

impl ReedSolomonConfig {
    pub fn for_group_size(member_count: usize) -> Self {
        match member_count {
            1..=5 => Self {
                data_shards: 3,
                parity_shards: 2,
                shard_size: 4096,
                group_size_range: (1, 5),
            },
            6..=15 => Self {
                data_shards: 8,
                parity_shards: 4,
                shard_size: 4096,
                group_size_range: (6, 15),
            },
            16..=50 => Self {
                data_shards: 12,
                parity_shards: 6,
                shard_size: 8192,
                group_size_range: (16, 50),
            },
            _ => Self {
                data_shards: 16,
                parity_shards: 8,
                shard_size: 8192,
                group_size_range: (51, usize::MAX),
            },
        }
    }

    pub fn total_shards(&self) -> usize {
        self.data_shards + self.parity_shards
    }

    pub fn can_lose_members(&self) -> usize {
        self.parity_shards
    }

    pub fn redundancy_factor(&self) -> f32 {
        (self.total_shards() as f32) / (self.data_shards as f32)
    }
}

/// Enhanced Reed Solomon manager with adaptive configuration and PQC sealing
#[derive(Debug)]
pub struct EnhancedReedSolomonManager<D: DhtStorage + saorsa_seal::Dht> {
    configs: Arc<RwLock<HashMap<String, ReedSolomonConfig>>>,
    shard_cache: Arc<RwLock<HashMap<String, Vec<Shard>>>>,
    integrity_tracker: Arc<RwLock<HashMap<String, IntegrityStatus>>>,
    dht: D,
}

impl<D: DhtStorage + saorsa_seal::Dht> EnhancedReedSolomonManager<D> {
    pub fn new(dht: D) -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            shard_cache: Arc::new(RwLock::new(HashMap::new())),
            integrity_tracker: Arc::new(RwLock::new(HashMap::new())),
            dht,
        }
    }

    /// Encode data for a specific group using saorsa-seal with PQC encryption
    pub async fn encode_group_data(
        &self,
        group_id: &str,
        data_id: &str,
        data: &[u8],
        group_member_count: usize,
    ) -> Result<Vec<Shard>> {
        // Select optimal configuration for group size
        let config = ReedSolomonConfig::for_group_size(group_member_count);

        // Store configuration for this group
        {
            let mut configs = self.configs.write().await;
            configs.insert(group_id.to_string(), config.clone());
        }

        debug!(
            "Sealing data for group {} with threshold {}/{} using saorsa-seal",
            group_id,
            config.data_shards,
            config.total_shards()
        );

        // Create recipients for the group (simplified - in real implementation,
        // these would be actual group member identities)
        let recipients: Vec<Recipient> = (0..group_member_count)
            .map(|i| Recipient {
                id: RecipientId::from_bytes(format!("{}:member:{}", group_id, i).into_bytes()),
                public_key: None, // Would include ML-KEM public key in real implementation
            })
            .collect();

        // Configure sealing policy with PQC encryption
        let policy = SealPolicy {
            n: config.total_shards(), // Total shares
            t: config.data_shards,    // Threshold needed to recover
            recipients,
            fec: saorsa_seal::FecParams {
                data_shares: config.data_shards,
                parity_shares: config.parity_shards,
                symbol_size: config.shard_size,
            },
            envelope: EnvelopeKind::PostQuantum, // ML-KEM-768 post-quantum encryption
            aad: format!("{}:{}", group_id, data_id).into_bytes(), // Additional authenticated data
        };

        // Seal the data using saorsa-seal
        let summary = seal_bytes(data, &policy, &self.dht)
            .await
            .context("Failed to seal data with saorsa-seal")?;

        debug!("Data sealed successfully with handle: {:?}", summary.handle);

        // Convert saorsa-seal result to our Shard format for compatibility
        let mut all_shards = Vec::new();

        // Create data shards from the sealed result
        for i in 0..config.data_shards {
            let shard = Shard {
                index: i,
                shard_type: ShardType::Data,
                data: vec![], // Data is stored in DHT by saorsa-seal
                group_id: group_id.to_string(),
                data_id: data_id.to_string(),
                integrity_hash: hex::encode(
                    blake3::hash(&summary.handle.sealed_meta_key).as_bytes(),
                ),
                created_at: chrono::Utc::now(),
                size: data.len(),
            };
            all_shards.push(shard);
        }

        // Create parity shards
        for i in 0..config.parity_shards {
            let shard = Shard {
                index: config.data_shards + i,
                shard_type: ShardType::Parity,
                data: vec![], // Parity data handled by saorsa-seal
                group_id: group_id.to_string(),
                data_id: data_id.to_string(),
                integrity_hash: hex::encode(
                    blake3::hash(&summary.handle.sealed_meta_key).as_bytes(),
                ),
                created_at: chrono::Utc::now(),
                size: data.len(),
            };
            all_shards.push(shard);
        }

        // Cache shards for quick access
        {
            let mut cache = self.shard_cache.write().await;
            let cache_key = format!("{}:{}", group_id, data_id);
            cache.insert(cache_key, all_shards.clone());
        }

        // Track integrity status
        {
            let mut tracker = self.integrity_tracker.write().await;
            tracker.insert(
                format!("{}:{}", group_id, data_id),
                IntegrityStatus {
                    total_shards: all_shards.len(),
                    created_at: chrono::Utc::now(),
                    last_verified: chrono::Utc::now(),
                    verification_count: 0,
                    corruption_detected: false,
                },
            );
        }

        info!(
            "Successfully encoded {} bytes into {} shards for group {}",
            data.len(),
            all_shards.len(),
            group_id
        );

        Ok(all_shards)
    }

    /// Decode data from available shards
    pub async fn decode_group_data(
        &self,
        group_id: &str,
        _data_id: &str,
        available_shards: &[Shard],
    ) -> Result<Vec<u8>> {
        // Get configuration for this group
        let config = {
            let configs = self.configs.read().await;
            configs.get(group_id).cloned().ok_or_else(|| {
                anyhow::anyhow!("No Reed Solomon configuration found for group {}", group_id)
            })?
        };

        if available_shards.len() < config.data_shards {
            bail!(
                "Insufficient shards for reconstruction: have {}, need {}",
                available_shards.len(),
                config.data_shards
            );
        }

        debug!(
            "Decoding data for group {} using {} available shards",
            group_id,
            available_shards.len()
        );

        // Create Reed Solomon codec (saorsa-fec)
        let fec_params = FecParams::new(config.data_shards as u16, config.parity_shards as u16)
            .context("Failed to create FEC parameters")?;
        let codec = FecCodec::new(fec_params).context("Failed to create Reed Solomon codec")?;

        // Group shards by chunk index
        let mut chunks_map: HashMap<usize, Vec<&Shard>> = HashMap::new();
        for shard in available_shards {
            let chunk_index = self.extract_chunk_index_from_shard(shard)?;
            chunks_map.entry(chunk_index).or_default().push(shard);
        }

        let mut decoded_chunks = Vec::new();

        // Decode each chunk
        for chunk_index in 0..chunks_map.len() {
            let chunk_shards = chunks_map
                .get(&chunk_index)
                .ok_or_else(|| anyhow::anyhow!("Missing chunk {} shards", chunk_index))?;

            if chunk_shards.len() < config.data_shards {
                bail!(
                    "Insufficient shards for chunk {}: have {}, need {}",
                    chunk_index,
                    chunk_shards.len(),
                    config.data_shards
                );
            }

            let decoded_chunk = self.decode_chunk(chunk_shards, &config, &codec).await?;

            decoded_chunks.push(decoded_chunk);
        }

        // Concatenate all decoded chunks
        let mut full_data = Vec::new();
        for chunk in decoded_chunks {
            full_data.extend_from_slice(&chunk);
        }

        // Remove padding to get original data
        let original_data = self.unpad_decoded_data(&full_data)?;

        info!(
            "Successfully decoded {} bytes from {} shards for group {}",
            original_data.len(),
            available_shards.len(),
            group_id
        );

        Ok(original_data)
    }

    /// Verify shard integrity using stored hashes
    pub async fn verify_shard_integrity(&self, shard: &Shard) -> Result<bool> {
        let calculated_hash = blake3::hash(&shard.data);
        let is_valid = calculated_hash.to_string() == shard.integrity_hash;

        if !is_valid {
            error!(
                "Shard integrity check failed for group {} shard {}",
                shard.group_id, shard.index
            );

            // Update integrity tracker
            let mut tracker = self.integrity_tracker.write().await;
            let key = format!("{}:{}", shard.group_id, shard.data_id);
            if let Some(status) = tracker.get_mut(&key) {
                status.corruption_detected = true;
            }
        } else {
            debug!(
                "Shard integrity verified for group {} shard {}",
                shard.group_id, shard.index
            );
        }

        Ok(is_valid)
    }

    /// Get optimal shard distribution plan for group members
    pub async fn create_distribution_plan(
        &self,
        group_id: &str,
        shards: &[Shard],
        group_members: &[String],
    ) -> Result<ShardDistributionPlan> {
        if group_members.is_empty() {
            bail!("Cannot create distribution plan without group members");
        }

        let config = {
            let configs = self.configs.read().await;
            configs
                .get(group_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No configuration found for group {}", group_id))?
        };

        let mut distribution = ShardDistributionPlan {
            group_id: group_id.to_string(),
            total_shards: shards.len(),
            member_assignments: HashMap::new(),
            redundancy_level: config.redundancy_factor(),
        };

        // Distribute shards evenly across members
        // Prioritize giving each member at least one data shard
        let mut member_index = 0;

        for (shard_index, shard) in shards.iter().enumerate() {
            let member_id = &group_members[member_index];

            distribution
                .member_assignments
                .entry(member_id.clone())
                .or_insert_with(Vec::new)
                .push(shard.clone());

            member_index = (member_index + 1) % group_members.len();

            debug!(
                "Assigned shard {} (type: {:?}) to member {}",
                shard_index, shard.shard_type, member_id
            );
        }

        // Verify distribution quality
        self.validate_distribution_plan(&distribution, &config)?;

        Ok(distribution)
    }

    /// Get reconstruction status for a group's data
    pub async fn get_reconstruction_status(
        &self,
        group_id: &str,
        _data_id: &str,
        available_shards: &[Shard],
    ) -> Result<ReconstructionStatus> {
        let config = {
            let configs = self.configs.read().await;
            configs
                .get(group_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No configuration found for group {}", group_id))?
        };

        let data_shards_available = available_shards
            .iter()
            .filter(|s| s.shard_type == ShardType::Data)
            .count();

        let parity_shards_available = available_shards
            .iter()
            .filter(|s| s.shard_type == ShardType::Parity)
            .count();

        let can_reconstruct = available_shards.len() >= config.data_shards;
        let corruption_tolerance = available_shards.len().saturating_sub(config.data_shards);

        Ok(ReconstructionStatus {
            can_reconstruct,
            available_shards: available_shards.len(),
            required_shards: config.data_shards,
            data_shards_available,
            parity_shards_available,
            corruption_tolerance,
            redundancy_factor: config.redundancy_factor(),
        })
    }

    // Private helper methods

    async fn encode_chunk(
        &self,
        chunk: &[u8],
        config: &ReedSolomonConfig,
        codec: &FecCodec,
        group_id: &str,
        data_id: &str,
        chunk_index: usize,
    ) -> Result<Vec<Shard>> {
        // Use FecCodec to encode the chunk
        let encoded_shares = codec
            .encode(chunk)
            .map_err(|e| anyhow::anyhow!("Reed Solomon encoding failed: {:?}", e))?;

        let mut shards = Vec::new();

        // Create shards from encoded shares
        for (index, share) in encoded_shares.iter().enumerate() {
            let shard_type = if index < config.data_shards {
                ShardType::Data
            } else {
                ShardType::Parity
            };

            let shard = Shard {
                index: chunk_index * config.total_shards() + index,
                shard_type,
                data: share.clone(),
                group_id: group_id.to_string(),
                data_id: data_id.to_string(),
                integrity_hash: blake3::hash(share).to_string(),
                created_at: chrono::Utc::now(),
                size: share.len(),
            };
            shards.push(shard);
        }

        Ok(shards)
    }

    async fn decode_chunk(
        &self,
        chunk_shards: &[&Shard],
        config: &ReedSolomonConfig,
        codec: &FecCodec,
    ) -> Result<Vec<u8>> {
        // Sort shards by index to maintain order
        let mut sorted_shards: Vec<_> = chunk_shards.iter().collect();
        sorted_shards.sort_by_key(|s| s.index);

        // Verify shard integrity before decoding
        for shard in &sorted_shards {
            if !self.verify_shard_integrity(shard).await? {
                warn!(
                    "Corrupted shard detected during decoding: group {}, shard {}",
                    shard.group_id, shard.index
                );
            }
        }

        // Prepare shares for decoding (Some for available, None for missing)
        let mut shares: Vec<Option<Vec<u8>>> = vec![None; config.total_shards()];
        for shard in &sorted_shards {
            let local_index = shard.index % config.total_shards();
            if local_index < shares.len() {
                shares[local_index] = Some(shard.data.clone());
            }
        }

        // Decode using Reed Solomon
        let decoded_data = codec
            .decode(&shares)
            .map_err(|e| anyhow::anyhow!("Reed Solomon decoding failed: {:?}", e))?;

        Ok(decoded_data)
    }

    fn pad_data_for_encoding(&self, data: &[u8], config: &ReedSolomonConfig) -> Result<Vec<u8>> {
        let mut padded = data.to_vec();

        // Calculate how much padding needed to make data divisible by shard_size
        let remainder = data.len() % config.shard_size;
        if remainder != 0 {
            let padding_needed = config.shard_size - remainder;
            padded.resize(data.len() + padding_needed, 0);
        }

        // Store original length in the first 8 bytes of padding
        let original_len = data.len() as u64;
        let len_bytes = original_len.to_le_bytes();
        let padding_start = data.len();

        if padded.len() >= padding_start + 8 {
            padded[padding_start..padding_start + 8].copy_from_slice(&len_bytes);
        }

        Ok(padded)
    }

    fn unpad_decoded_data(&self, padded_data: &[u8]) -> Result<Vec<u8>> {
        if padded_data.len() < 8 {
            return Ok(padded_data.to_vec());
        }

        // Try to find original length from padding
        // Look for length marker in the last few bytes
        for i in (0..padded_data.len().saturating_sub(8)).rev() {
            let len_bytes = &padded_data[i..i + 8];
            // Avoid unwrap by copying into a fixed array
            let mut arr = [0u8; 8];
            arr.copy_from_slice(len_bytes);
            let potential_len = u64::from_le_bytes(arr) as usize;

            if potential_len <= padded_data.len() && potential_len > 0 {
                return Ok(padded_data[..potential_len].to_vec());
            }
        }

        // If we can't find the original length, return as-is
        Ok(padded_data.to_vec())
    }

    fn extract_chunk_index_from_shard(&self, shard: &Shard) -> Result<usize> {
        // Chunk index is encoded in the shard index
        // For config with k+m shards per chunk, chunk_index = shard.index / (k+m)
        let configs = futures::executor::block_on(self.configs.read());
        let config = configs.get(&shard.group_id).ok_or_else(|| {
            anyhow::anyhow!("No configuration found for group {}", shard.group_id)
        })?;

        Ok(shard.index / config.total_shards())
    }

    fn validate_distribution_plan(
        &self,
        plan: &ShardDistributionPlan,
        config: &ReedSolomonConfig,
    ) -> Result<()> {
        // Verify each member has at least one shard
        if plan
            .member_assignments
            .values()
            .any(|shards| shards.is_empty())
        {
            bail!("Distribution plan has members with no shards assigned");
        }

        // Verify we can still reconstruct if we lose the maximum allowed members
        let members_count = plan.member_assignments.len();
        if members_count < config.data_shards {
            bail!(
                "Too few members ({}) for Reed Solomon configuration (need at least {})",
                members_count,
                config.data_shards
            );
        }

        debug!(
            "Distribution plan validated: {} members can tolerate {} failures",
            members_count, config.parity_shards
        );

        Ok(())
    }

    /// Get the data shard count for reconstruction
    pub async fn data_shard_count(&self) -> usize {
        // Return a default reasonable value - this would normally be configured per group
        8 // Default Reed Solomon configuration uses 8 data shards
    }

    /// Simplified encode_data method for compatibility
    pub async fn encode_data(&self, data: &[u8]) -> Result<Vec<Shard>> {
        // Use default group settings for simplified interface
        self.encode_group_data("default", "default", data, 8).await
    }

    /// Simplified decode_data method for compatibility
    pub async fn decode_data(&self, shards: &[Shard]) -> Result<Vec<u8>> {
        // Use default group settings for simplified interface
        self.decode_group_data("default", "default", shards).await
    }
}

#[derive(Debug, Clone)]
pub struct ShardDistributionPlan {
    pub group_id: String,
    pub total_shards: usize,
    pub member_assignments: HashMap<String, Vec<Shard>>,
    pub redundancy_level: f32,
}

#[derive(Debug, Clone)]
pub struct ReconstructionStatus {
    pub can_reconstruct: bool,
    pub available_shards: usize,
    pub required_shards: usize,
    pub data_shards_available: usize,
    pub parity_shards_available: usize,
    pub corruption_tolerance: usize,
    pub redundancy_factor: f32,
}

#[derive(Debug, Clone)]
pub struct IntegrityStatus {
    pub total_shards: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_verified: chrono::DateTime<chrono::Utc>,
    pub verification_count: u64,
    pub corruption_detected: bool,
}
