//! Forward Error Correction Storage
//!
//! Provides resilient storage using Reed-Solomon erasure coding with
//! per-shard checksums and on-demand repair.

use anyhow::{Context, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, warn};

/// Target shard size (256KB) for balancing repair granularity and overhead.
const TARGET_SHARD_SIZE: usize = 256 * 1024;
/// Cap data shards to limit total shard count and encode cost.
const MAX_DATA_SHARDS: usize = 16;

/// FEC storage manager for resilient data storage
pub struct FecStorage {
    base_path: PathBuf,
    default_redundancy: f32,
}

impl FecStorage {
    /// Create a new FEC storage manager
    pub async fn new(base_path: &Path, default_redundancy: f32) -> Result<Self> {
        let fec_path = base_path.join("fec");
        fs::create_dir_all(&fec_path).await?;

        Ok(Self {
            base_path: fec_path,
            default_redundancy,
        })
    }

    /// Store data with Forward Error Correction
    pub async fn store_with_fec(
        &self,
        key: &str,
        data: &[u8],
        redundancy: f32,
    ) -> Result<Vec<PathBuf>> {
        let key_dir = self.base_path.join(key);
        fs::create_dir_all(&key_dir).await?;

        let redundancy = if redundancy <= 0.0 {
            self.default_redundancy
        } else {
            redundancy
        };

        let (data_shards, parity_shards, shard_size) = Self::compute_layout(data.len(), redundancy);
        let total_shards = data_shards + parity_shards;

        let mut shards: Vec<Vec<u8>> = vec![vec![0u8; shard_size]; total_shards];
        for (i, shard) in shards.iter_mut().enumerate().take(data_shards) {
            let start = i * shard_size;
            let end = std::cmp::min(start + shard_size, data.len());
            if start < data.len() {
                shard[..end - start].copy_from_slice(&data[start..end]);
            }
        }

        if parity_shards > 0 && shard_size > 0 {
            let rs = ReedSolomon::new(data_shards, parity_shards)
                .context("Failed to create Reed-Solomon encoder")?;
            rs.encode(&mut shards)
                .context("Failed to encode Reed-Solomon shards")?;
        }

        let mut shard_paths = Vec::with_capacity(total_shards);
        let mut shard_hashes = Vec::with_capacity(total_shards);

        for (index, shard) in shards.iter().enumerate() {
            let path = key_dir.join(Self::shard_file_name(index));
            fs::write(&path, shard).await?;
            shard_paths.push(path);
            shard_hashes.push(blake3::hash(shard).as_bytes().to_vec());
        }

        let metadata = FecMetadata {
            original_size: data.len(),
            checksum: blake3::hash(data).as_bytes().to_vec(),
            data_shards,
            parity_shards,
            shard_size,
            shard_hashes,
        };

        let metadata_path = key_dir.join("metadata.json");
        let metadata_json = serde_json::to_vec(&metadata)?;
        fs::write(metadata_path, metadata_json).await?;

        Ok(shard_paths)
    }

    /// Retrieve data from FEC shards
    pub async fn retrieve_from_fec(&self, shard_paths: &[PathBuf]) -> Result<Vec<u8>> {
        let (key_dir, metadata) = Self::load_metadata_from_shards(shard_paths).await?;
        if metadata.original_size == 0 {
            return Ok(Vec::new());
        }

        let total_shards = metadata.data_shards + metadata.parity_shards;
        let (mut shards, missing_indices) =
            Self::load_shards(&key_dir, &metadata, total_shards).await?;

        if metadata.parity_shards > 0 && !missing_indices.is_empty() {
            let rs = ReedSolomon::new(metadata.data_shards, metadata.parity_shards)
                .context("Failed to create Reed-Solomon decoder")?;
            rs.reconstruct(&mut shards)
                .context("Failed to reconstruct missing shards")?;
        }

        if shards
            .iter()
            .take(metadata.data_shards)
            .any(|s| s.is_none())
        {
            return Err(anyhow::anyhow!(
                "Insufficient shards to recover data for {}",
                key_dir.display()
            ));
        }

        let mut data = Vec::with_capacity(metadata.original_size);
        for shard in shards.into_iter().take(metadata.data_shards) {
            let shard = shard.ok_or_else(|| anyhow::anyhow!("Missing data shard"))?;
            data.extend_from_slice(&shard);
        }
        data.truncate(metadata.original_size);

        let checksum = blake3::hash(&data);
        if checksum.as_bytes() != metadata.checksum.as_slice() {
            return Err(anyhow::anyhow!("Data integrity check failed"));
        }

        Ok(data)
    }

    /// Repair missing or corrupted shards
    pub async fn repair_shards(&self, key: &str) -> Result<usize> {
        let key_dir = self.base_path.join(key);
        let metadata = Self::load_metadata(&key_dir).await?;

        if metadata.parity_shards == 0 {
            return Ok(0);
        }

        let total_shards = metadata.data_shards + metadata.parity_shards;
        let (mut shards, missing_indices) =
            Self::load_shards(&key_dir, &metadata, total_shards).await?;

        if missing_indices.is_empty() {
            return Ok(0);
        }

        let rs = ReedSolomon::new(metadata.data_shards, metadata.parity_shards)
            .context("Failed to create Reed-Solomon repairer")?;
        rs.reconstruct(&mut shards)
            .context("Failed to reconstruct shards")?;

        let mut repaired = 0usize;
        for index in missing_indices {
            if let Some(ref shard) = shards[index] {
                let path = key_dir.join(Self::shard_file_name(index));
                fs::write(&path, shard).await?;
                repaired += 1;
            }
        }

        if repaired > 0 {
            debug!("Repaired {} shards for {}", repaired, key);
        }

        Ok(repaired)
    }

    /// Delete FEC storage for a key
    pub async fn delete(&self, key: &str) -> Result<()> {
        let key_dir = self.base_path.join(key);
        if key_dir.exists() {
            fs::remove_dir_all(key_dir).await?;
        }
        Ok(())
    }

    /// Get storage statistics
    pub async fn get_stats(&self, key: &str) -> Result<FecStats> {
        let key_dir = self.base_path.join(key);
        let metadata = Self::load_metadata(&key_dir).await?;

        let total_shards = metadata.data_shards + metadata.parity_shards;
        let (_shards, missing_indices) =
            Self::load_shards(&key_dir, &metadata, total_shards).await?;

        let corrupted = missing_indices.len();
        let healthy = total_shards.saturating_sub(corrupted);
        let can_recover = healthy >= metadata.data_shards;

        let storage_overhead = if metadata.original_size == 0 {
            0.0
        } else {
            (metadata.shard_size * total_shards) as f32 / metadata.original_size as f32
        };

        Ok(FecStats {
            original_size: metadata.original_size,
            total_shards,
            healthy_shards: healthy,
            corrupted_shards: corrupted,
            redundancy: if metadata.data_shards == 0 {
                0.0
            } else {
                total_shards as f32 / metadata.data_shards as f32
            },
            storage_overhead,
            can_recover,
        })
    }

    fn compute_layout(data_len: usize, redundancy: f32) -> (usize, usize, usize) {
        if data_len == 0 {
            return (1, 0, 0);
        }

        let mut data_shards = data_len
            .div_ceil(TARGET_SHARD_SIZE)
            .clamp(1, MAX_DATA_SHARDS);

        let mut parity_shards = if redundancy > 1.0 {
            ((data_shards as f32) * (redundancy - 1.0)).ceil() as usize
        } else {
            0
        };

        if redundancy > 1.0 && parity_shards == 0 {
            parity_shards = 1;
        }

        if data_shards == 0 {
            data_shards = 1;
        }

        let shard_size = data_len.div_ceil(data_shards);
        (data_shards, parity_shards, shard_size)
    }

    fn shard_file_name(index: usize) -> String {
        format!("shard_{index}.bin")
    }

    async fn load_metadata(key_dir: &Path) -> Result<FecMetadata> {
        let metadata_path = key_dir.join("metadata.json");
        let metadata_json = fs::read(metadata_path).await?;
        let metadata: FecMetadata = serde_json::from_slice(&metadata_json)?;
        Ok(metadata)
    }

    async fn load_metadata_from_shards(shard_paths: &[PathBuf]) -> Result<(PathBuf, FecMetadata)> {
        let first = shard_paths
            .first()
            .ok_or_else(|| anyhow::anyhow!("No shard paths provided"))?;
        let key_dir = first
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid shard path"))?;
        let metadata = Self::load_metadata(key_dir).await?;
        Ok((key_dir.to_path_buf(), metadata))
    }

    async fn load_shards(
        key_dir: &Path,
        metadata: &FecMetadata,
        total_shards: usize,
    ) -> Result<(Vec<Option<Vec<u8>>>, Vec<usize>)> {
        let mut shards: Vec<Option<Vec<u8>>> = vec![None; total_shards];
        let mut missing = Vec::new();

        for (index, slot) in shards.iter_mut().enumerate() {
            let path = key_dir.join(Self::shard_file_name(index));
            match fs::read(&path).await {
                Ok(bytes) => {
                    let expected_hash = metadata.shard_hashes.get(index);
                    if Self::shard_valid(&bytes, metadata.shard_size, expected_hash) {
                        *slot = Some(bytes);
                    } else {
                        warn!("Shard {} failed validation", path.display());
                        missing.push(index);
                    }
                }
                Err(_) => {
                    missing.push(index);
                }
            }
        }

        Ok((shards, missing))
    }

    fn shard_valid(bytes: &[u8], expected_size: usize, expected_hash: Option<&Vec<u8>>) -> bool {
        if bytes.len() != expected_size {
            return false;
        }
        if let Some(hash) = expected_hash {
            blake3::hash(bytes).as_bytes() == hash.as_slice()
        } else {
            true
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FecMetadata {
    original_size: usize,
    checksum: Vec<u8>,
    data_shards: usize,
    parity_shards: usize,
    shard_size: usize,
    shard_hashes: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct FecStats {
    pub original_size: usize,
    pub total_shards: usize,
    pub healthy_shards: usize,
    pub corrupted_shards: usize,
    pub redundancy: f32,
    pub storage_overhead: f32,
    pub can_recover: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_fec_storage_basic() {
        let temp_dir = TempDir::new().unwrap();
        let fec = FecStorage::new(temp_dir.path(), 1.5).await.unwrap();

        let test_data = vec![42u8; 1000];

        // Store with FEC
        let shard_paths = fec
            .store_with_fec("test_key", &test_data, 1.5)
            .await
            .unwrap();
        assert!(!shard_paths.is_empty());

        // Retrieve data
        let recovered = fec.retrieve_from_fec(&shard_paths).await.unwrap();
        assert_eq!(recovered, test_data);

        // Get statistics
        let stats = fec.get_stats("test_key").await.unwrap();
        assert!(stats.can_recover);
        assert_eq!(stats.original_size, test_data.len());
    }

    #[tokio::test]
    async fn test_fec_repair_missing_shard() {
        let temp_dir = TempDir::new().unwrap();
        let fec = FecStorage::new(temp_dir.path(), 1.5).await.unwrap();

        let test_data = vec![7u8; 1024 * 8];
        let shard_paths = fec
            .store_with_fec("repair_key", &test_data, 1.5)
            .await
            .unwrap();

        // Corrupt first shard
        let corrupt_path = shard_paths[0].clone();
        fs::write(&corrupt_path, vec![0u8; 10]).await.unwrap();

        let repaired = fec.repair_shards("repair_key").await.unwrap();
        assert!(repaired >= 1);

        let recovered = fec.retrieve_from_fec(&shard_paths).await.unwrap();
        assert_eq!(recovered, test_data);
    }
}
