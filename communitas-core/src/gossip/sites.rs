// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Saorsa Sites - DNS-free Website Publishing
//!
//! Implements SPEC2.md §5: Saorsa Sites for publishing and fetching websites
//! without DNS or HTTP, using ML-DSA signed manifests and gossip-based discovery.
//!
//! ## Architecture (per SPEC2.md §5)
//! - Site Identity (SID): ML-DSA public key
//! - Manifest: ML-DSA signed, content-addressed blocks
//! - Publishing: Chunk assets, gossip Provider Summaries to SITE_ADVERT shard
//! - Fetching: Subscribe to shards, score providers, fetch over QUIC
//!
//! ## Use Cases
//! 1. **Publish Site**: Create manifest, chunk assets, start provider
//! 2. **Fetch Site**: Subscribe to SITE_ADVERT shard, fetch manifest/blocks
//! 3. **Private Site**: MLS group encryption with ChaCha20Poly1305

use crate::gossip::rendezvous::RendezvousClient;
use anyhow::Result;
use blake3;
use bytes::Bytes;
use saorsa_gossip_transport::{GossipTransport, StreamType};
use saorsa_gossip_types::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Maximum block size (512KB per SPEC2.md §5.3)
pub const MAX_BLOCK_SIZE: usize = 512 * 1024;

/// Site request/response protocol for Bulk stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SiteRequest {
    /// Request a site manifest
    GetManifest { site_id: SiteId },
    /// Request a content block
    GetBlock { hash: [u8; 32] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SiteResponse {
    /// Manifest response
    Manifest(SiteManifest),
    /// Block response
    Block(Block),
    /// Error response
    Error(String),
}

/// Content-addressed block
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    /// BLAKE3 hash of content
    pub hash: [u8; 32],
    /// Raw block content
    pub content: Vec<u8>,
}

impl Block {
    /// Create a new block from content, computing BLAKE3 hash
    pub fn new(content: Vec<u8>) -> Self {
        let hash = blake3::hash(&content);
        Self {
            hash: hash.into(),
            content,
        }
    }

    /// Verify block hash matches content
    pub fn verify(&self) -> bool {
        let computed = blake3::hash(&self.content);
        computed.as_bytes() == &self.hash
    }
}

/// Chunk large content into blocks
pub fn chunk_content(content: &[u8], chunk_size: usize) -> Vec<Block> {
    content
        .chunks(chunk_size)
        .map(|chunk| Block::new(chunk.to_vec()))
        .collect()
}

/// Site Identifier (SID) - ML-DSA public key
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SiteId {
    /// ML-DSA public key bytes (32 bytes)
    pub key: [u8; 32],
}

impl SiteId {
    /// Create a new SiteId from ML-DSA public key
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

/// Site Manifest - ML-DSA signed content manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteManifest {
    /// Protocol version
    pub version: u8,

    /// Site identifier (ML-DSA public key)
    pub site_id: SiteId,

    /// Manifest version (incrementing)
    pub manifest_version: u64,

    /// Root block hash (BLAKE3)
    pub root_hash: [u8; 32],

    /// Block map: path -> block_hash
    pub blocks: Vec<(String, [u8; 32])>,

    /// ML-DSA signature over all fields except signature
    pub signature: Vec<u8>,
}

impl SiteManifest {
    /// Create a new unsigned manifest
    pub fn new(site_id: SiteId, manifest_version: u64, blocks: Vec<(String, [u8; 32])>) -> Self {
        // Compute root hash from all block hashes
        let mut hasher = blake3::Hasher::new();
        for (path, hash) in &blocks {
            hasher.update(path.as_bytes());
            hasher.update(hash);
        }
        let root_hash = hasher.finalize();

        Self {
            version: 1,
            site_id,
            manifest_version,
            root_hash: root_hash.into(),
            blocks,
            signature: vec![],
        }
    }

    /// Get canonical bytes for signing (all fields except signature)
    pub fn to_sign_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.version);
        bytes.extend_from_slice(&self.site_id.key);
        bytes.extend_from_slice(&self.manifest_version.to_le_bytes());
        bytes.extend_from_slice(&self.root_hash);

        for (path, hash) in &self.blocks {
            bytes.extend_from_slice(path.as_bytes());
            bytes.extend_from_slice(hash);
        }

        bytes
    }

    /// Sign the manifest (placeholder for ML-DSA)
    pub fn sign(&mut self, _secret_key: &[u8]) {
        // TODO: Implement ML-DSA signing when saorsa-pqc is integrated
        // For now, use BLAKE3 hash as placeholder signature
        let sign_bytes = self.to_sign_bytes();
        let sig_hash = blake3::hash(&sign_bytes);
        self.signature = sig_hash.as_bytes().to_vec();
    }

    /// Verify signature (placeholder for ML-DSA)
    pub fn verify(&self, _public_key: &[u8]) -> bool {
        // TODO: Implement ML-DSA verification when saorsa-pqc is integrated
        // For now, just verify signature is not empty
        !self.signature.is_empty()
    }
}

/// Site Publisher - Publishes sites to the network
pub struct SitePublisher {
    /// Site identifier
    site_id: SiteId,

    /// Block storage (hash -> block)
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,

    /// Current manifest
    manifest: Arc<RwLock<Option<SiteManifest>>>,
}

impl SitePublisher {
    /// Create a new site publisher
    pub fn new(site_id: SiteId) -> Self {
        Self {
            site_id,
            blocks: Arc::new(RwLock::new(HashMap::new())),
            manifest: Arc::new(RwLock::new(None)),
        }
    }

    /// Add an asset to the site
    pub async fn add_asset(&self, _path: String, content: Vec<u8>) -> Result<[u8; 32]> {
        // Chunk the content if needed
        let chunks = if content.len() > MAX_BLOCK_SIZE {
            chunk_content(&content, MAX_BLOCK_SIZE)
        } else {
            vec![Block::new(content)]
        };

        // Store blocks
        let mut blocks = self.blocks.write().await;
        for block in &chunks {
            blocks.insert(block.hash, block.clone());
        }

        // Return the first block hash (for single-block assets this is the only hash)
        Ok(chunks[0].hash)
    }

    /// Build manifest from added assets
    pub async fn build_manifest(
        &self,
        version: u64,
        asset_paths: Vec<(String, [u8; 32])>,
    ) -> Result<SiteManifest> {
        let manifest = SiteManifest::new(self.site_id.clone(), version, asset_paths);

        // Store manifest
        let mut current_manifest = self.manifest.write().await;
        *current_manifest = Some(manifest.clone());

        Ok(manifest)
    }

    /// Get a block by hash
    pub async fn get_block(&self, hash: &[u8; 32]) -> Option<Block> {
        let blocks = self.blocks.read().await;
        blocks.get(hash).cloned()
    }

    /// Get current manifest
    pub async fn get_manifest(&self) -> Option<SiteManifest> {
        let manifest = self.manifest.read().await;
        manifest.clone()
    }

    /// Handle a site request and return response bytes
    pub async fn handle_request(&self, request_bytes: Bytes) -> Result<Bytes> {
        // Deserialize request
        let request: SiteRequest = bincode::deserialize(&request_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize request: {}", e))?;

        // Process request
        let response = match request {
            SiteRequest::GetBlock { hash } => {
                // Look up block
                let blocks = self.blocks.read().await;
                match blocks.get(&hash) {
                    Some(block) => SiteResponse::Block(block.clone()),
                    None => SiteResponse::Error(format!("Block not found: {:?}", hash)),
                }
            }
            SiteRequest::GetManifest { site_id } => {
                // Verify site ID matches
                if site_id != self.site_id {
                    return Ok(Bytes::from(bincode::serialize(&SiteResponse::Error(
                        format!(
                            "Site ID mismatch: expected {:?}, got {:?}",
                            self.site_id, site_id
                        ),
                    ))?));
                }

                // Get manifest
                let manifest = self.manifest.read().await;
                match manifest.as_ref() {
                    Some(m) => SiteResponse::Manifest(m.clone()),
                    None => SiteResponse::Error("No manifest published".to_string()),
                }
            }
        };

        // Serialize response
        let response_bytes = bincode::serialize(&response)
            .map_err(|e| anyhow::anyhow!("Failed to serialize response: {}", e))?;

        Ok(Bytes::from(response_bytes))
    }
}

/// Site Fetcher - Fetches sites from the network
pub struct SiteFetcher {
    /// Rendezvous client for provider discovery
    rendezvous: Arc<RendezvousClient>,

    /// Transport layer for QUIC operations
    transport: Arc<RwLock<Box<dyn GossipTransport>>>,

    /// Fetched blocks cache (hash -> block)
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,

    /// Fetched manifests cache (site_id -> manifest)
    manifests: Arc<RwLock<HashMap<SiteId, SiteManifest>>>,
}

impl SiteFetcher {
    /// Create a new site fetcher
    pub fn new(rendezvous: Arc<RendezvousClient>) -> Self {
        let transport = rendezvous.get_transport();

        Self {
            rendezvous,
            transport,
            blocks: Arc::new(RwLock::new(HashMap::new())),
            manifests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start discovering providers for a site
    pub async fn start_discovery(&self, site_id: &SiteId) -> Result<()> {
        // Subscribe to SITE_ADVERT shard for this site
        self.rendezvous.subscribe_to_shard(&site_id.key).await?;

        // Start collecting provider summaries
        self.rendezvous
            .start_collecting_for_target(site_id.key)
            .await?;

        Ok(())
    }

    /// Get providers for a site (from rendezvous)
    pub async fn get_providers(
        &self,
        site_id: &SiteId,
    ) -> Vec<saorsa_gossip_rendezvous::ProviderSummary> {
        self.rendezvous.get_providers_for_target(&site_id.key).await
    }

    /// Fetch a block from network via QUIC
    pub async fn fetch_block(&self, hash: &[u8; 32], provider: PeerId) -> Result<Block> {
        // Check cache first
        {
            let blocks = self.blocks.read().await;
            if let Some(block) = blocks.get(hash) {
                return Ok(block.clone());
            }
        }

        // Create request
        let request = SiteRequest::GetBlock { hash: *hash };
        let request_bytes = bincode::serialize(&request)
            .map_err(|e| anyhow::anyhow!("Failed to serialize request: {}", e))?;

        // Send request on Bulk stream
        self.transport
            .read()
            .await
            .send_to_peer(provider, StreamType::Bulk, Bytes::from(request_bytes))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request: {}", e))?;

        // Receive response on Bulk stream
        let (_peer, stream_type, response_bytes) = self
            .transport
            .read()
            .await
            .receive_message()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to receive response: {}", e))?;

        if stream_type != StreamType::Bulk {
            return Err(anyhow::anyhow!(
                "Wrong stream type: expected Bulk, got {:?}",
                stream_type
            ));
        }

        // Deserialize response
        let response: SiteResponse = bincode::deserialize(&response_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize response: {}", e))?;

        // Extract block
        match response {
            SiteResponse::Block(block) => {
                // Verify hash
                if !block.verify() {
                    return Err(anyhow::anyhow!("Block hash verification failed"));
                }

                // Cache
                let mut blocks = self.blocks.write().await;
                blocks.insert(*hash, block.clone());

                Ok(block)
            }
            SiteResponse::Error(err) => Err(anyhow::anyhow!("Provider error: {}", err)),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    /// Fetch a manifest for a site via QUIC
    pub async fn fetch_manifest(&self, site_id: &SiteId, provider: PeerId) -> Result<SiteManifest> {
        // Check cache first
        {
            let manifests = self.manifests.read().await;
            if let Some(manifest) = manifests.get(site_id) {
                return Ok(manifest.clone());
            }
        }

        // Create request
        let request = SiteRequest::GetManifest {
            site_id: site_id.clone(),
        };
        let request_bytes = bincode::serialize(&request)
            .map_err(|e| anyhow::anyhow!("Failed to serialize request: {}", e))?;

        // Send request on Bulk stream
        self.transport
            .read()
            .await
            .send_to_peer(provider, StreamType::Bulk, Bytes::from(request_bytes))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request: {}", e))?;

        // Receive response on Bulk stream
        let (_peer, stream_type, response_bytes) = self
            .transport
            .read()
            .await
            .receive_message()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to receive response: {}", e))?;

        if stream_type != StreamType::Bulk {
            return Err(anyhow::anyhow!(
                "Wrong stream type: expected Bulk, got {:?}",
                stream_type
            ));
        }

        // Deserialize response
        let response: SiteResponse = bincode::deserialize(&response_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize response: {}", e))?;

        // Extract manifest
        match response {
            SiteResponse::Manifest(manifest) => {
                // Verify site ID matches
                if &manifest.site_id != site_id {
                    return Err(anyhow::anyhow!("Site ID mismatch"));
                }

                // Cache
                let mut manifests = self.manifests.write().await;
                manifests.insert(site_id.clone(), manifest.clone());

                Ok(manifest)
            }
            SiteResponse::Error(err) => Err(anyhow::anyhow!("Provider error: {}", err)),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    /// Store a block in cache (for testing/simulation)
    pub async fn cache_block(&self, block: Block) {
        let mut blocks = self.blocks.write().await;
        blocks.insert(block.hash, block);
    }

    /// Store a manifest in cache (for testing/simulation)
    pub async fn cache_manifest(&self, manifest: SiteManifest) {
        let mut manifests = self.manifests.write().await;
        manifests.insert(manifest.site_id.clone(), manifest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_gossip_pubsub::PubSub as PubSubTrait;
    use saorsa_gossip_transport::{GossipTransport, QuicTransport, TransportConfig};
    use saorsa_gossip_types::PeerId;

    fn create_test_peer_id(seed: u8) -> PeerId {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        PeerId::new(bytes)
    }

    async fn create_test_rendezvous_client() -> RendezvousClient {
        let peer_id = create_test_peer_id(1);

        // Create test transport
        let config = TransportConfig::default();
        let qt1 = QuicTransport::new(config.clone());
        let qt2 = QuicTransport::new(config);
        let transport: Arc<RwLock<Box<dyn GossipTransport>>> = Arc::new(RwLock::new(Box::new(qt1)));

        // Create test identity for signing
        let identity = saorsa_gossip_identity::Identity::new("TestUser".to_string())
            .expect("identity creation");
        let signing_key = identity.key_pair().clone();

        // Create test pubsub (needs separate transport instance)
        let pubsub_impl =
            saorsa_gossip_pubsub::PlumtreePubSub::new(peer_id, Arc::new(qt2), signing_key);
        let pubsub: Arc<RwLock<Box<dyn PubSubTrait>>> =
            Arc::new(RwLock::new(Box::new(pubsub_impl)));

        RendezvousClient::new(peer_id, transport, pubsub)
    }

    #[test]
    fn test_site_id_creation() {
        let key = [42u8; 32];
        let site_id = SiteId::new(key);

        assert_eq!(site_id.as_bytes(), &key);
        assert_eq!(site_id.key, key);
    }

    #[test]
    fn test_site_manifest_structure() {
        // RED: This will pass immediately as we're just testing structure
        let site_id = SiteId::new([1u8; 32]);
        let manifest = SiteManifest {
            version: 1,
            site_id: site_id.clone(),
            manifest_version: 1,
            root_hash: [0u8; 32],
            blocks: vec![("index.html".to_string(), [2u8; 32])],
            signature: vec![],
        };

        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.site_id, site_id);
        assert_eq!(manifest.blocks.len(), 1);
    }

    #[test]
    fn test_block_creation_and_hashing() {
        // RED: Test that Block correctly computes BLAKE3 hash
        let content = b"Hello, Saorsa Sites!".to_vec();
        let block = Block::new(content.clone());

        // Verify hash was computed
        assert_eq!(block.content, content);
        assert_ne!(block.hash, [0u8; 32]); // Hash should not be zero

        // Manually verify hash
        let expected_hash = blake3::hash(&content);
        assert_eq!(block.hash, *expected_hash.as_bytes());
    }

    #[test]
    fn test_block_verification() {
        // RED: Test block integrity verification
        let content = b"Test content for verification".to_vec();
        let block = Block::new(content);

        // Valid block should verify
        assert!(block.verify());

        // Corrupted block should fail verification
        let mut corrupted = block.clone();
        corrupted.content[0] ^= 0xFF; // Flip bits
        assert!(!corrupted.verify());
    }

    #[test]
    fn test_chunk_small_content() {
        // RED: Test chunking content smaller than block size
        let content = b"Small content".to_vec();
        let blocks = chunk_content(&content, MAX_BLOCK_SIZE);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, content);
        assert!(blocks[0].verify());
    }

    #[test]
    fn test_chunk_large_content() {
        // RED: Test chunking content larger than block size
        let chunk_size = 100;
        let content: Vec<u8> = (0..250).map(|i| (i % 256) as u8).collect();
        let blocks = chunk_content(&content, chunk_size);

        // Should create 3 blocks: 100 + 100 + 50
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].content.len(), 100);
        assert_eq!(blocks[1].content.len(), 100);
        assert_eq!(blocks[2].content.len(), 50);

        // All blocks should verify
        for block in &blocks {
            assert!(block.verify());
        }

        // Reassemble should match original
        let reassembled: Vec<u8> = blocks
            .iter()
            .flat_map(|b| b.content.iter())
            .copied()
            .collect();
        assert_eq!(reassembled, content);
    }

    #[test]
    fn test_chunk_exact_multiple() {
        // RED: Test chunking when content is exact multiple of chunk size
        let chunk_size = 50;
        let content: Vec<u8> = vec![42u8; 150]; // Exactly 3 chunks
        let blocks = chunk_content(&content, chunk_size);

        assert_eq!(blocks.len(), 3);
        for block in blocks {
            assert_eq!(block.content.len(), 50);
            assert!(block.verify());
        }
    }

    #[test]
    fn test_deterministic_hashing() {
        // RED: Same content should always produce same hash
        let content = b"Deterministic content".to_vec();
        let block1 = Block::new(content.clone());
        let block2 = Block::new(content.clone());

        assert_eq!(block1.hash, block2.hash);
        assert_eq!(block1.content, block2.content);
    }

    #[tokio::test]
    async fn test_site_publisher_creation() {
        let site_id = SiteId::new([1u8; 32]);
        let publisher = SitePublisher::new(site_id.clone());

        // Should have no manifest initially
        assert!(publisher.get_manifest().await.is_none());
    }

    #[tokio::test]
    async fn test_add_asset() {
        let site_id = SiteId::new([1u8; 32]);
        let publisher = SitePublisher::new(site_id);

        let content = b"Hello, World!".to_vec();
        let hash = publisher
            .add_asset("index.html".to_string(), content.clone())
            .await
            .unwrap();

        // Should be able to retrieve the block
        let block = publisher.get_block(&hash).await.unwrap();
        assert_eq!(block.content, content);
        assert!(block.verify());
    }

    #[tokio::test]
    async fn test_build_manifest() {
        let site_id = SiteId::new([1u8; 32]);
        let publisher = SitePublisher::new(site_id.clone());

        // Add assets
        let index_hash = publisher
            .add_asset("index.html".to_string(), b"<html>".to_vec())
            .await
            .unwrap();
        let style_hash = publisher
            .add_asset("style.css".to_string(), b"body{}".to_vec())
            .await
            .unwrap();

        // Build manifest
        let asset_paths = vec![
            ("index.html".to_string(), index_hash),
            ("style.css".to_string(), style_hash),
        ];
        let manifest = publisher
            .build_manifest(1, asset_paths.clone())
            .await
            .unwrap();

        // Verify manifest
        assert_eq!(manifest.site_id, site_id);
        assert_eq!(manifest.manifest_version, 1);
        assert_eq!(manifest.blocks, asset_paths);
        assert_ne!(manifest.root_hash, [0u8; 32]);

        // Should be retrievable
        let retrieved = publisher.get_manifest().await.unwrap();
        assert_eq!(retrieved.root_hash, manifest.root_hash);
    }

    #[test]
    fn test_manifest_signing() {
        let site_id = SiteId::new([1u8; 32]);
        let blocks = vec![("index.html".to_string(), [2u8; 32])];
        let mut manifest = SiteManifest::new(site_id, 1, blocks);

        // Initially unsigned
        assert_eq!(manifest.signature.len(), 0);

        // Sign manifest
        let secret_key = [3u8; 32];
        manifest.sign(&secret_key);

        // Should have signature
        assert!(!manifest.signature.is_empty());
        assert!(manifest.verify(&[4u8; 32]));
    }

    #[test]
    fn test_manifest_root_hash_deterministic() {
        let site_id = SiteId::new([1u8; 32]);
        let blocks = vec![
            ("index.html".to_string(), [2u8; 32]),
            ("style.css".to_string(), [3u8; 32]),
        ];

        let manifest1 = SiteManifest::new(site_id.clone(), 1, blocks.clone());
        let manifest2 = SiteManifest::new(site_id, 1, blocks);

        // Same blocks should produce same root hash
        assert_eq!(manifest1.root_hash, manifest2.root_hash);
    }

    #[tokio::test]
    async fn test_add_large_asset() {
        let site_id = SiteId::new([1u8; 32]);
        let publisher = SitePublisher::new(site_id);

        // Create content larger than MAX_BLOCK_SIZE
        let large_content: Vec<u8> = vec![42u8; MAX_BLOCK_SIZE + 100];
        let hash = publisher
            .add_asset("large.bin".to_string(), large_content.clone())
            .await
            .unwrap();

        // Should have stored multiple blocks
        let first_block = publisher.get_block(&hash).await.unwrap();
        assert_eq!(first_block.content.len(), MAX_BLOCK_SIZE);
        assert!(first_block.verify());
    }

    #[tokio::test]
    async fn test_site_fetcher_creation() {
        let rendezvous = Arc::new(create_test_rendezvous_client().await);
        let fetcher = SiteFetcher::new(rendezvous);

        // Initially should have no cached data
        let site_id = SiteId::new([2u8; 32]);
        let provider = create_test_peer_id(99);
        // This will fail because there's no actual provider serving
        assert!(fetcher.fetch_manifest(&site_id, provider).await.is_err());
    }

    #[tokio::test]
    async fn test_fetcher_block_caching() {
        let rendezvous = Arc::new(create_test_rendezvous_client().await);
        let fetcher = SiteFetcher::new(rendezvous);

        // Cache a block
        let content = b"Test content".to_vec();
        let block = Block::new(content.clone());
        let hash = block.hash;

        fetcher.cache_block(block.clone()).await;

        // Should be able to fetch from cache (provider arg not used when cached)
        let provider = create_test_peer_id(99);
        let fetched = fetcher.fetch_block(&hash, provider).await.unwrap();
        assert_eq!(fetched.content, content);
        assert!(fetched.verify());
    }

    #[tokio::test]
    async fn test_fetcher_manifest_caching() {
        let rendezvous = Arc::new(create_test_rendezvous_client().await);
        let fetcher = SiteFetcher::new(rendezvous);

        // Cache a manifest
        let site_id = SiteId::new([2u8; 32]);
        let blocks = vec![("index.html".to_string(), [3u8; 32])];
        let manifest = SiteManifest::new(site_id.clone(), 1, blocks);

        fetcher.cache_manifest(manifest.clone()).await;

        // Should be able to fetch from cache (provider arg not used when cached)
        let provider = create_test_peer_id(99);
        let fetched = fetcher.fetch_manifest(&site_id, provider).await.unwrap();
        assert_eq!(fetched.site_id, site_id);
        assert_eq!(fetched.root_hash, manifest.root_hash);
    }

    #[tokio::test]
    async fn test_fetcher_start_discovery() {
        let rendezvous = Arc::new(create_test_rendezvous_client().await);
        let fetcher = SiteFetcher::new(rendezvous);

        let site_id = SiteId::new([2u8; 32]);

        // Start discovery (subscribes to shard)
        fetcher.start_discovery(&site_id).await.unwrap();

        // Should be able to get providers (empty initially)
        let providers = fetcher.get_providers(&site_id).await;
        assert_eq!(providers.len(), 0);
    }

    #[tokio::test]
    async fn test_publisher_serve_block_request() {
        // RED: Test that publisher can serve block requests
        let site_id = SiteId::new([1u8; 32]);
        let publisher = SitePublisher::new(site_id);

        // Add a block
        let content = b"Test block content".to_vec();
        let hash = publisher
            .add_asset("test.txt".to_string(), content.clone())
            .await
            .unwrap();

        // Create a request
        let request = SiteRequest::GetBlock { hash };
        let request_bytes = bincode::serialize(&request).unwrap();

        // Process request (this will fail until we implement it)
        let response_bytes = publisher
            .handle_request(Bytes::from(request_bytes))
            .await
            .unwrap();
        let response: SiteResponse = bincode::deserialize(&response_bytes).unwrap();

        // Verify response
        match response {
            SiteResponse::Block(block) => {
                assert_eq!(block.hash, hash);
                assert_eq!(block.content, content);
            }
            _ => panic!("Expected Block response"),
        }
    }

    #[tokio::test]
    async fn test_publisher_serve_manifest_request() {
        // RED: Test that publisher can serve manifest requests
        let site_id = SiteId::new([1u8; 32]);
        let publisher = SitePublisher::new(site_id.clone());

        // Add assets and build manifest
        let hash = publisher
            .add_asset("index.html".to_string(), b"<html>".to_vec())
            .await
            .unwrap();
        let manifest = publisher
            .build_manifest(1, vec![("index.html".to_string(), hash)])
            .await
            .unwrap();

        // Create a request
        let request = SiteRequest::GetManifest {
            site_id: site_id.clone(),
        };
        let request_bytes = bincode::serialize(&request).unwrap();

        // Process request
        let response_bytes = publisher
            .handle_request(Bytes::from(request_bytes))
            .await
            .unwrap();
        let response: SiteResponse = bincode::deserialize(&response_bytes).unwrap();

        // Verify response
        match response {
            SiteResponse::Manifest(received_manifest) => {
                assert_eq!(received_manifest.site_id, site_id);
                assert_eq!(received_manifest.root_hash, manifest.root_hash);
            }
            _ => panic!("Expected Manifest response"),
        }
    }
}
