/**
 * Saorsa Storage System - Content Addressing and Chunking
 * Implements BLAKE3 content addressing with 256KB chunking and integrity verification
 */
use crate::saorsa_storage::errors::*;
use blake3::{Hash, Hasher};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Standard chunk size: 256KB
pub const CHUNK_SIZE: usize = 256 * 1024;

/// Maximum number of chunks for a single content item
pub const MAX_CHUNKS: u32 = 40960; // ~10GB max file size

/// Content chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentChunk {
    pub data: Vec<u8>,
    pub index: u32,
    pub total_chunks: u32,
    pub chunk_hash: String,
    pub content_hash: String,
    pub size: usize,
}

/// Content metadata for addressing and verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub content_hash: String,
    pub size: u64,
    pub chunk_count: u32,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
    pub chunk_hashes: Vec<String>,
    pub compression_ratio: Option<f32>,
}

/// Content addressing result
#[derive(Debug, Clone)]
pub struct ContentAddress {
    pub hash: String,
    pub chunks: Vec<ContentChunk>,
    pub metadata: ContentMetadata,
}

/// Content reconstruction state
#[derive(Debug)]
pub struct ReconstructionState {
    pub total_chunks: u32,
    pub received_chunks: std::collections::HashMap<u32, ContentChunk>,
    pub content_hash: String,
    pub total_size: u64,
    pub is_complete: bool,
}

/// Content addressing manager
pub struct ContentAddressing {
    compression_enabled: bool,
    verify_integrity: bool,
}

impl ContentAddressing {
    /// Create a new content addressing manager
    pub fn new() -> Self {
        Self {
            compression_enabled: true,
            verify_integrity: true,
        }
    }

    /// Create content addressing manager with configuration
    pub fn with_config(compression_enabled: bool, verify_integrity: bool) -> Self {
        Self {
            compression_enabled,
            verify_integrity,
        }
    }

    /// Generate content address from data
    pub fn address_content(
        &self,
        data: &[u8],
        content_type: &str,
    ) -> ContentResult<ContentAddress> {
        if data.is_empty() {
            return Err(ContentError::ContentSizeValidationFailed);
        }

        // Calculate total content hash first
        let content_hash = self.hash_content(data);
        let content_hash_str = hex::encode(content_hash.as_bytes());

        // Compress data if enabled and beneficial
        let (processed_data, compression_ratio) = if self.compression_enabled {
            self.compress_if_beneficial(data)?
        } else {
            (data.to_vec(), None)
        };

        // Split into chunks
        let chunks = self.create_chunks(&processed_data, &content_hash_str)?;

        if chunks.len() > MAX_CHUNKS as usize {
            return Err(ContentError::ChunkingFailed {
                reason: format!(
                    "Too many chunks: {} exceeds limit {}",
                    chunks.len(),
                    MAX_CHUNKS
                ),
            });
        }

        // Collect chunk hashes for metadata
        let chunk_hashes: Vec<String> = chunks
            .iter()
            .map(|chunk| chunk.chunk_hash.clone())
            .collect();

        let metadata = ContentMetadata {
            content_hash: content_hash_str.clone(),
            size: data.len() as u64,
            chunk_count: chunks.len() as u32,
            content_type: content_type.to_string(),
            created_at: Utc::now(),
            chunk_hashes,
            compression_ratio,
        };

        Ok(ContentAddress {
            hash: content_hash_str,
            chunks,
            metadata,
        })
    }

    /// Verify content integrity against expected hash
    pub fn verify_content(&self, data: &[u8], expected_hash: &str) -> ContentResult<bool> {
        if !self.verify_integrity {
            return Ok(true);
        }

        let actual_hash = self.hash_content(data);
        let actual_hash_str = hex::encode(actual_hash.as_bytes());

        Ok(actual_hash_str == expected_hash)
    }

    /// Verify chunk integrity
    pub fn verify_chunk(&self, chunk: &ContentChunk) -> ContentResult<bool> {
        if !self.verify_integrity {
            return Ok(true);
        }

        let actual_hash = self.hash_content(&chunk.data);
        let actual_hash_str = hex::encode(actual_hash.as_bytes());

        if actual_hash_str != chunk.chunk_hash {
            return Err(ContentError::ChecksumMismatch);
        }

        if chunk.data.len() != chunk.size {
            return Err(ContentError::ContentSizeValidationFailed);
        }

        Ok(true)
    }

    /// Start content reconstruction from chunks
    pub fn start_reconstruction(
        &self,
        content_hash: &str,
        total_size: u64,
        total_chunks: u32,
    ) -> ReconstructionState {
        ReconstructionState {
            total_chunks,
            received_chunks: std::collections::HashMap::new(),
            content_hash: content_hash.to_string(),
            total_size,
            is_complete: false,
        }
    }

    /// Add chunk to reconstruction state
    pub fn add_chunk_to_reconstruction(
        &self,
        state: &mut ReconstructionState,
        chunk: ContentChunk,
    ) -> ContentResult<()> {
        // Verify chunk integrity
        self.verify_chunk(&chunk)?;

        // Verify chunk belongs to this content
        if chunk.content_hash != state.content_hash {
            return Err(ContentError::InvalidAddress {
                address: chunk.content_hash,
            });
        }

        // Verify chunk index is valid
        if chunk.index >= state.total_chunks {
            return Err(ContentError::InvalidChunkIndex { index: chunk.index });
        }

        // Verify total chunks match
        if chunk.total_chunks != state.total_chunks {
            return Err(ContentError::ReconstructionFailed);
        }

        // Add chunk to state
        state.received_chunks.insert(chunk.index, chunk);

        // Check if reconstruction is complete
        state.is_complete = state.received_chunks.len() == state.total_chunks as usize;

        Ok(())
    }

    /// Reconstruct content from chunks
    pub fn reconstruct_content(&self, state: &ReconstructionState) -> ContentResult<Vec<u8>> {
        if !state.is_complete {
            let missing_count = state.total_chunks as usize - state.received_chunks.len();
            return Err(ContentError::MissingChunks {
                missing_count: missing_count as u32,
                total_count: state.total_chunks,
            });
        }

        // Sort chunks by index and reconstruct
        let mut reconstructed = Vec::with_capacity(state.total_size as usize);

        for i in 0..state.total_chunks {
            if let Some(chunk) = state.received_chunks.get(&i) {
                reconstructed.extend_from_slice(&chunk.data);
            } else {
                return Err(ContentError::MissingChunks {
                    missing_count: 1,
                    total_count: state.total_chunks,
                });
            }
        }

        // Verify reconstructed content hash
        if self.verify_integrity {
            let reconstructed_hash = self.hash_content(&reconstructed);
            let reconstructed_hash_str = hex::encode(reconstructed_hash.as_bytes());

            if reconstructed_hash_str != state.content_hash {
                return Err(ContentError::ChecksumMismatch);
            }
        }

        // Decompress if needed (detect from first chunk or metadata)
        let final_content = self.decompress_if_needed(&reconstructed)?;

        Ok(final_content)
    }

    /// Generate unique content address for deduplication
    pub fn generate_content_id(&self, data: &[u8], context: &str) -> String {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.update(context.as_bytes());

        let hash = hasher.finalize();
        hex::encode(hash.as_bytes())
    }

    /// Calculate content fingerprint for similarity detection
    pub fn calculate_fingerprint(&self, data: &[u8]) -> ContentResult<Vec<u8>> {
        // Use rolling hash for fuzzy matching
        const WINDOW_SIZE: usize = 64;
        let mut fingerprint = Vec::new();

        if data.len() < WINDOW_SIZE {
            // For small content, use full hash
            let hash = self.hash_content(data);
            return Ok(hash.as_bytes().to_vec());
        }

        // Calculate rolling hashes for larger content
        for window in data.windows(WINDOW_SIZE) {
            let hash = self.hash_content(window);
            // Take first 8 bytes for fingerprint
            fingerprint.extend_from_slice(&hash.as_bytes()[..8]);
        }

        Ok(fingerprint)
    }

    /// Validate content type and size constraints
    pub fn validate_content(
        &self,
        data: &[u8],
        content_type: &str,
        max_size: Option<u64>,
    ) -> ContentResult<()> {
        // Size validation
        if let Some(max) = max_size {
            if data.len() as u64 > max {
                return Err(ContentError::ContentSizeValidationFailed);
            }
        }

        // Basic content type validation
        if content_type.is_empty() {
            return Err(ContentError::ContentTypeValidationFailed);
        }

        // Content should not be empty
        if data.is_empty() {
            return Err(ContentError::ContentSizeValidationFailed);
        }

        // Check for reasonable maximum size (10GB)
        if data.len() > 10 * 1024 * 1024 * 1024 {
            return Err(ContentError::ContentSizeValidationFailed);
        }

        Ok(())
    }

    /// Get optimal chunk size for content
    pub fn get_optimal_chunk_size(&self, content_size: u64) -> usize {
        match content_size {
            0..=1048576 => 64 * 1024,          // 64KB for small files
            1048577..=104857600 => CHUNK_SIZE, // 256KB for medium files
            _ => 512 * 1024,                   // 512KB for large files
        }
    }

    // Private helper methods

    fn hash_content(&self, data: &[u8]) -> Hash {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }

    fn create_chunks(&self, data: &[u8], content_hash: &str) -> ContentResult<Vec<ContentChunk>> {
        let chunk_size = self.get_optimal_chunk_size(data.len() as u64);
        let total_chunks = (data.len() + chunk_size - 1) / chunk_size;
        let mut chunks = Vec::with_capacity(total_chunks);

        for (index, chunk_data) in data.chunks(chunk_size).enumerate() {
            let chunk_hash = self.hash_content(chunk_data);
            let chunk_hash_str = hex::encode(chunk_hash.as_bytes());

            let chunk = ContentChunk {
                data: chunk_data.to_vec(),
                index: index as u32,
                total_chunks: total_chunks as u32,
                chunk_hash: chunk_hash_str,
                content_hash: content_hash.to_string(),
                size: chunk_data.len(),
            };

            chunks.push(chunk);
        }

        Ok(chunks)
    }

    fn compress_if_beneficial(&self, data: &[u8]) -> ContentResult<(Vec<u8>, Option<f32>)> {
        // Only compress if data is large enough to benefit
        if data.len() < 1024 {
            return Ok((data.to_vec(), None));
        }

        use flate2::Compression;
        use flate2::write::GzEncoder;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(data)
            .map_err(|e| ContentError::ChunkingFailed {
                reason: format!("Compression failed: {}", e),
            })?;

        let compressed = encoder.finish().map_err(|e| ContentError::ChunkingFailed {
            reason: format!("Compression finalization failed: {}", e),
        })?;

        let compression_ratio = compressed.len() as f32 / data.len() as f32;

        // Only use compression if it provides significant benefit
        if compression_ratio < 0.9 {
            Ok((compressed, Some(compression_ratio)))
        } else {
            Ok((data.to_vec(), None))
        }
    }

    fn decompress_if_needed(&self, data: &[u8]) -> ContentResult<Vec<u8>> {
        // Simple heuristic: check if data starts with gzip magic number
        if data.len() > 2 && data[0] == 0x1f && data[1] == 0x8b {
            use flate2::read::GzDecoder;

            let mut decoder = GzDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|_e| ContentError::ReconstructionFailed)?;

            Ok(decompressed)
        } else {
            Ok(data.to_vec())
        }
    }
}

impl Default for ContentAddressing {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-safe implementations
unsafe impl Send for ContentAddressing {}
unsafe impl Sync for ContentAddressing {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_addressing() {
        let addressing = ContentAddressing::new();
        let data = b"Hello, world! This is test content for chunking.";

        let result = addressing.address_content(data, "text/plain");
        assert!(result.is_ok());

        let address = result.unwrap();
        assert!(!address.hash.is_empty());
        assert_eq!(address.chunks.len(), 1); // Small content should be 1 chunk
    }

    #[test]
    fn test_content_verification() {
        let addressing = ContentAddressing::new();
        let data = b"Test content for verification";

        let address = addressing.address_content(data, "text/plain").unwrap();
        let is_valid = addressing.verify_content(data, &address.hash).unwrap();
        assert!(is_valid);

        // Test with wrong data
        let wrong_data = b"Wrong content";
        let is_invalid = addressing
            .verify_content(wrong_data, &address.hash)
            .unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_content_reconstruction() {
        let addressing = ContentAddressing::new();
        let data = vec![42u8; CHUNK_SIZE * 3]; // 3 chunks worth of data

        let address = addressing
            .address_content(&data, "application/octet-stream")
            .unwrap();
        assert_eq!(address.chunks.len(), 3);

        // Test reconstruction
        let mut state = addressing.start_reconstruction(
            &address.hash,
            data.len() as u64,
            address.chunks.len() as u32,
        );

        // Add all chunks
        for chunk in address.chunks {
            addressing
                .add_chunk_to_reconstruction(&mut state, chunk)
                .unwrap();
        }

        assert!(state.is_complete);

        let reconstructed = addressing.reconstruct_content(&state).unwrap();
        assert_eq!(reconstructed, data);
    }

    #[test]
    fn test_chunk_verification() {
        let addressing = ContentAddressing::new();
        let data = b"Test chunk data";

        let address = addressing.address_content(data, "text/plain").unwrap();
        let chunk = &address.chunks[0];

        let is_valid = addressing.verify_chunk(chunk).unwrap();
        assert!(is_valid);

        // Test with corrupted chunk
        let mut corrupted_chunk = chunk.clone();
        corrupted_chunk.data[0] ^= 1; // Flip a bit

        let result = addressing.verify_chunk(&corrupted_chunk);
        assert!(result.is_err());
    }

    #[test]
    fn test_content_fingerprint() {
        let addressing = ContentAddressing::new();
        let data1 = b"This is some test content for fingerprinting";
        let data2 = b"This is some test content for fingerprinting with changes";

        let fp1 = addressing.calculate_fingerprint(data1).unwrap();
        let fp2 = addressing.calculate_fingerprint(data2).unwrap();

        assert_ne!(fp1, fp2);
        assert!(!fp1.is_empty());
        assert!(!fp2.is_empty());
    }

    #[test]
    fn test_content_validation() {
        let addressing = ContentAddressing::new();
        let data = b"Valid content";

        let result = addressing.validate_content(data, "text/plain", Some(1024));
        assert!(result.is_ok());

        // Test empty content
        let result = addressing.validate_content(&[], "text/plain", Some(1024));
        assert!(result.is_err());

        // Test oversized content
        let result = addressing.validate_content(data, "text/plain", Some(5));
        assert!(result.is_err());
    }

    #[test]
    fn test_optimal_chunk_size() {
        let addressing = ContentAddressing::new();

        assert_eq!(addressing.get_optimal_chunk_size(1024), 64 * 1024);
        assert_eq!(addressing.get_optimal_chunk_size(1024 * 1024), CHUNK_SIZE);
        assert_eq!(
            addressing.get_optimal_chunk_size(1024 * 1024 * 1024),
            512 * 1024
        );
    }

    #[test]
    fn test_content_id_generation() {
        let addressing = ContentAddressing::new();
        let data = b"Test content";

        let id1 = addressing.generate_content_id(data, "context1");
        let id2 = addressing.generate_content_id(data, "context2");
        let id3 = addressing.generate_content_id(data, "context1");

        assert_ne!(id1, id2); // Different contexts should produce different IDs
        assert_eq!(id1, id3); // Same data and context should produce same ID
    }
}
