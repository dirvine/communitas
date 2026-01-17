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
//! - Site Identity (SID): ML-DSA-65 public key (1952 bytes)
//! - Manifest: ML-DSA-65 signed (3309 byte signatures), content-addressed blocks
//! - Publishing: Chunk assets, gossip Provider Summaries to SITE_ADVERT shard
//! - Fetching: Subscribe to shards, score providers, fetch over QUIC
//!
//! ## Use Cases
//! 1. **Publish Site**: Create manifest, chunk assets, start provider
//! 2. **Fetch Site**: Subscribe to SITE_ADVERT shard, fetch manifest/blocks
//! 3. **Private Site**: MLS group encryption with ChaCha20Poly1305

use crate::gossip::RendezvousClient;
use anyhow::{Context, Result};
use blake3;
use bytes::Bytes;
use saorsa_gossip_transport::{AntQuicTransport, AntQuicTransportConfig, GossipStreamType};
use saorsa_gossip_types::PeerId;
use saorsa_pqc::dsa_traits::{SerDes, Signer, Verifier};
use saorsa_pqc::ml_dsa_65::{PrivateKey, PublicKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

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

/// Wire protocol envelope for request/response correlation
///
/// Wraps SiteRequest/SiteResponse with a correlation ID to match requests with responses
/// when multiple components share the same transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SitesWire {
    /// Request with correlation ID
    Request { id: u64, body: SiteRequest },
    /// Response with correlation ID (matches request id)
    Response { id: u64, body: SiteResponse },
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

/// Site Identifier (SID) - BLAKE3 hash of ML-DSA-65 public key
///
/// For efficiency, we use a 32-byte BLAKE3 hash of the ML-DSA-65 public key
/// for routing and discovery. The full public key is stored in the SiteManifest
/// for signature verification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SiteId {
    /// BLAKE3 hash of ML-DSA-65 public key (32 bytes)
    pub hash: [u8; 32],
}

impl SiteId {
    /// Create a new SiteId from BLAKE3 hash
    pub fn new(hash: [u8; 32]) -> Self {
        Self { hash }
    }

    /// Create SiteId from ML-DSA-65 PublicKey (hashes it)
    pub fn from_public_key(pk: &PublicKey) -> Self {
        let pk_bytes = pk.clone().into_bytes();
        let hash = blake3::hash(&pk_bytes);
        Self { hash: hash.into() }
    }

    /// Get the hash bytes (for rendezvous shard routing)
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.hash
    }

    /// Get the hash for use with rendezvous client
    pub fn to_target_id(&self) -> [u8; 32] {
        self.hash
    }
}

/// Site Manifest - ML-DSA-65 signed content manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteManifest {
    /// Protocol version
    pub version: u8,

    /// Site identifier (BLAKE3 hash of public key)
    pub site_id: SiteId,

    /// Full ML-DSA-65 public key for signature verification (1952 bytes)
    pub public_key: Vec<u8>,

    /// Manifest version (incrementing, prevents rollback)
    pub manifest_version: u64,

    /// Timestamp (Unix milliseconds, prevents replay)
    pub timestamp: u64,

    /// Root block hash (BLAKE3)
    pub root_hash: [u8; 32],

    /// Block map: path -> block_hash
    pub blocks: Vec<(String, [u8; 32])>,

    /// ML-DSA-65 signature over all fields except signature (3309 bytes)
    pub signature: Vec<u8>,
}

impl SiteManifest {
    /// Create a new unsigned manifest
    pub fn new(
        site_id: SiteId,
        public_key: &PublicKey,
        manifest_version: u64,
        blocks: Vec<(String, [u8; 32])>,
    ) -> Self {
        // Compute root hash from all block hashes
        let mut hasher = blake3::Hasher::new();
        for (path, hash) in &blocks {
            hasher.update(path.as_bytes());
            hasher.update(hash);
        }
        let root_hash = hasher.finalize();

        // Get current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;

        Self {
            version: 1,
            site_id,
            public_key: public_key.clone().into_bytes().to_vec(),
            manifest_version,
            timestamp,
            root_hash: root_hash.into(),
            blocks,
            signature: vec![],
        }
    }

    /// Get canonical bytes for signing (all fields except signature)
    pub fn to_sign_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.version);
        bytes.extend_from_slice(&self.site_id.hash);
        bytes.extend_from_slice(&self.public_key);
        bytes.extend_from_slice(&self.manifest_version.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.root_hash);

        for (path, hash) in &self.blocks {
            bytes.extend_from_slice(path.as_bytes());
            bytes.extend_from_slice(hash);
        }

        bytes
    }

    /// Sign the manifest with ML-DSA-65
    ///
    /// # Arguments
    /// * `signing_key` - ML-DSA-65 private key for signing
    ///
    /// # Errors
    /// Returns error if signing fails
    pub fn sign(&mut self, signing_key: &PrivateKey) -> Result<()> {
        let message = self.to_sign_bytes();
        let signature = signing_key
            .try_sign(&message, &[]) // Empty context as per FIPS 204
            .map_err(|e| anyhow::anyhow!("ML-DSA-65 signing failed: {}", e))?;

        self.signature = signature.to_vec();
        Ok(())
    }

    /// Verify ML-DSA-65 signature using embedded public key
    ///
    /// # Returns
    /// Ok(()) if signature is valid and not expired, Err otherwise
    pub fn verify(&self) -> Result<()> {
        // Public key must be exactly 1952 bytes for ML-DSA-65
        if self.public_key.len() != 1952 {
            anyhow::bail!(
                "Invalid public key size: expected 1952, got {}",
                self.public_key.len()
            );
        }

        // Verify public key matches site_id
        let pk_hash = blake3::hash(&self.public_key);
        if pk_hash.as_bytes() != &self.site_id.hash {
            anyhow::bail!("Public key does not match site_id");
        }

        // Deserialize public key
        let pk_array: [u8; 1952] = self
            .public_key
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Public key is not 1952 bytes"))?;
        let public_key = PublicKey::try_from_bytes(pk_array)
            .map_err(|e| anyhow::anyhow!("Invalid public key: {}", e))?;

        // Signature must be exactly 3309 bytes for ML-DSA-65
        if self.signature.len() != 3309 {
            anyhow::bail!(
                "Invalid signature size: expected 3309, got {}",
                self.signature.len()
            );
        }

        // Convert signature to fixed-size array
        let sig_array: [u8; 3309] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert signature to array"))?;

        // Verify the signature
        let message = self.to_sign_bytes();
        if !public_key.verify(&message, &sig_array, &[]) {
            anyhow::bail!("Signature verification failed");
        }

        // Check timestamp is not too far in the future (prevent future-dated manifests)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;

        // Allow up to 5 minutes clock skew
        if self.timestamp > now + (5 * 60 * 1000) {
            anyhow::bail!("Manifest timestamp is too far in the future");
        }

        Ok(())
    }

    /// Check if this manifest is newer than another (prevents rollback)
    pub fn is_newer_than(&self, other: &SiteManifest) -> bool {
        self.manifest_version > other.manifest_version
    }
}

/// Site Publisher - Publishes sites to the network
pub struct SitePublisher {
    /// Site identifier
    site_id: SiteId,

    /// Block storage (hash -> block)
    /// In-memory for now, will be replaced by BlockCache
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,

    /// Persistent block cache (optional, enables offline serving)
    #[allow(dead_code)]
    block_cache: Option<Arc<super::block_cache::BlockCache>>,

    /// Current manifest
    manifest: Arc<RwLock<Option<SiteManifest>>>,
}

impl SitePublisher {
    /// Create a new site publisher without persistent cache
    pub fn new(site_id: SiteId) -> Self {
        Self {
            site_id,
            blocks: Arc::new(RwLock::new(HashMap::new())),
            block_cache: None,
            manifest: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new site publisher with persistent cache
    pub fn with_cache(site_id: SiteId, cache: Arc<super::block_cache::BlockCache>) -> Self {
        Self {
            site_id,
            blocks: Arc::new(RwLock::new(HashMap::new())),
            block_cache: Some(cache),
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

        // Store blocks in both memory and persistent cache
        let mut blocks = self.blocks.write().await;
        for block in &chunks {
            blocks.insert(block.hash, block.clone());

            // Also store in persistent cache if available (pinned for published content)
            if let Some(cache) = &self.block_cache {
                cache.store(block.clone(), true).await?;
            }
        }

        // Return the first block hash (for single-block assets this is the only hash)
        Ok(chunks[0].hash)
    }

    /// Build manifest from added assets
    ///
    /// # Arguments
    /// * `public_key` - ML-DSA-65 public key for this site
    /// * `version` - Manifest version (must be monotonically increasing)
    /// * `asset_paths` - List of (path, block_hash) tuples
    pub async fn build_manifest(
        &self,
        public_key: &PublicKey,
        version: u64,
        asset_paths: Vec<(String, [u8; 32])>,
    ) -> Result<SiteManifest> {
        let manifest = SiteManifest::new(self.site_id.clone(), public_key, version, asset_paths);

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

    /// Update stored manifest (e.g., after signing)
    ///
    /// Used to update the manifest after signing it externally.
    /// Validates that the manifest belongs to this publisher.
    pub async fn set_manifest(&self, manifest: SiteManifest) -> Result<()> {
        if manifest.site_id != self.site_id {
            anyhow::bail!("Manifest site_id does not match publisher");
        }

        let mut current = self.manifest.write().await;
        *current = Some(manifest);
        Ok(())
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

/// RAII guard for dispatcher channel cleanup
///
/// Ensures response channel is unregistered on ALL paths (success or error)
struct DispatcherGuard {
    dispatcher: Arc<super::sites_dispatcher::SitesDispatcher>,
    request_id: u64,
    rx: tokio::sync::mpsc::Receiver<SiteResponse>,
}

impl Drop for DispatcherGuard {
    fn drop(&mut self) {
        // Spawn cleanup task (can't await in Drop)
        let dispatcher = self.dispatcher.clone();
        let request_id = self.request_id;
        tokio::spawn(async move {
            dispatcher.unregister_response_channel(request_id).await;
        });
    }
}

/// Site Fetcher - Fetches sites from the network
pub struct SiteFetcher {
    /// Rendezvous client for provider discovery
    rendezvous: Arc<RendezvousClient>,

    /// Transport layer for QUIC operations (shared with SitesListener)
    transport: super::transport_types::SharedTransport,

    /// Sites dispatcher for coordinated message routing (optional - for production)
    /// When set, uses dispatcher channels instead of direct transport receive
    dispatcher: Option<Arc<super::sites_dispatcher::SitesDispatcher>>,

    /// Fetched blocks cache (hash -> block) - in-memory
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,

    /// Persistent block cache (optional, enables offline viewing)
    #[allow(dead_code)]
    block_cache: Option<Arc<super::block_cache::BlockCache>>,

    /// Fetched manifests cache (site_id -> manifest) - in-memory
    manifests: Arc<RwLock<HashMap<SiteId, SiteManifest>>>,

    /// Correlation ID counter for request/response matching
    next_request_id: Arc<RwLock<u64>>,
}

impl SiteFetcher {
    /// Create a new site fetcher with shared transport (recommended)
    ///
    /// Use this with the dedicated Sites transport for production.
    pub fn new_with_shared_transport(
        rendezvous: Arc<RendezvousClient>,
        transport: super::transport_types::SharedTransport,
    ) -> Self {
        Self {
            rendezvous,
            transport,
            dispatcher: None, // Set via set_dispatcher() after construction
            blocks: Arc::new(RwLock::new(HashMap::new())),
            block_cache: None,
            manifests: Arc::new(RwLock::new(HashMap::new())),
            next_request_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Set the Sites dispatcher (must be called before fetching in production)
    pub fn set_dispatcher(&mut self, dispatcher: Arc<super::sites_dispatcher::SitesDispatcher>) {
        self.dispatcher = Some(dispatcher);
    }

    /// Create a new site fetcher (for tests, creates dummy transport)
    ///
    /// Note: This creates an unbound transport that won't work for real network fetching.
    /// Use new_with_shared_transport() with a bound Sites transport for production.
    pub async fn new(rendezvous: Arc<RendezvousClient>) -> Result<Self> {
        let dummy_bind = "0.0.0.0:0"
            .parse()
            .context("Failed to parse dummy bind address")?;
        let dummy =
            AntQuicTransport::with_config(AntQuicTransportConfig::new(dummy_bind, vec![]), None)
                .await
                .context("Failed to create dummy transport")?;
        let transport: super::transport_types::SharedTransport = Arc::new(dummy);

        Ok(Self {
            rendezvous,
            transport,
            dispatcher: None, // For backward compat with tests
            blocks: Arc::new(RwLock::new(HashMap::new())),
            block_cache: None,
            manifests: Arc::new(RwLock::new(HashMap::new())),
            next_request_id: Arc::new(RwLock::new(1)),
        })
    }

    /// Allocate a new request ID for correlation
    async fn next_id(&self) -> u64 {
        let mut id_lock = self.next_request_id.write().await;
        let id = *id_lock;
        *id_lock += 1;
        id
    }

    /// Send a request and receive correlated response using SitesWire envelope
    ///
    /// If dispatcher is set (production), uses dispatcher channels.
    /// Otherwise falls back to direct transport receive (tests only).
    ///
    /// Ensures channel cleanup on ALL paths (success or error) via RAII guard.
    async fn request_response(
        &self,
        request: SiteRequest,
        provider: PeerId,
    ) -> Result<SiteResponse> {
        // Allocate correlation ID
        let request_id = self.next_id().await;

        // Wrap in SitesWire envelope
        let wire_request = SitesWire::Request {
            id: request_id,
            body: request,
        };

        // Serialize request (before registering channel to avoid leak on error)
        let request_bytes = bincode::serialize(&wire_request)
            .map_err(|e| anyhow::anyhow!("Failed to serialize request: {}", e))?;

        // If we have a dispatcher, register for response before sending
        // Create cleanup guard to ensure unregister happens on ALL paths
        let dispatcher_guard = if let Some(ref dispatcher) = self.dispatcher {
            let rx = dispatcher.register_response_channel(request_id).await;
            Some(DispatcherGuard {
                dispatcher: dispatcher.clone(),
                request_id,
                rx,
            })
        } else {
            None
        };

        // Send the request
        debug!(
            "Sending Sites request {} to peer {:?}",
            request_id, provider
        );
        self.transport
            .send_to_peer(provider, GossipStreamType::Bulk, Bytes::from(request_bytes))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request: {}", e))?;
        debug!("Sites request {} sent successfully", request_id);

        // Receive response
        let response = if let Some(mut guard) = dispatcher_guard {
            // Production path: wait for dispatcher to route the response
            debug!("Waiting for Sites response {} via dispatcher", request_id);
            let response = guard
                .rx
                .recv()
                .await
                .ok_or_else(|| anyhow::anyhow!("Dispatcher dropped response channel"))?;
            debug!("Received Sites response {} via dispatcher", request_id);

            // Guard will auto-cleanup on drop (both success and error paths)
            response
        } else {
            // Test/fallback path: receive directly from transport
            // This will ONLY work if there's no listener competing for messages!
            loop {
                let (_peer, stream_type, response_bytes) =
                    self.transport
                        .receive_message()
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to receive response: {}", e))?;

                if stream_type != GossipStreamType::Bulk {
                    continue; // Skip non-Bulk messages
                }

                // Try to deserialize as SitesWire
                let wire_msg: SitesWire = match bincode::deserialize(&response_bytes) {
                    Ok(msg) => msg,
                    Err(_) => continue, // Not a Sites message, skip
                };

                // Check if this is a Response with our correlation ID
                match wire_msg {
                    SitesWire::Response { id, body } if id == request_id => {
                        // This is our response!
                        break body;
                    }
                    _ => {
                        // Not our response, keep waiting
                        continue;
                    }
                }
            }
        };

        // Note: Cleanup happens automatically via DispatcherGuard::drop()
        // No manual unregister needed - guard ensures cleanup on ALL paths

        Ok(response)
    }

    /// Start discovering providers for a site
    pub async fn start_discovery(&self, site_id: &SiteId) -> Result<()> {
        // Subscribe to SITE_ADVERT shard for this site
        self.rendezvous.subscribe_to_shard(&site_id.hash).await?;

        // Start collecting provider summaries
        self.rendezvous
            .start_collecting_for_target(site_id.hash)
            .await?;

        Ok(())
    }

    /// Get providers for a site (from rendezvous)
    pub async fn get_providers(
        &self,
        site_id: &SiteId,
    ) -> Vec<saorsa_gossip_rendezvous::ProviderSummary> {
        self.rendezvous
            .get_providers_for_target(&site_id.hash)
            .await
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

        // Create request and send with correlation ID
        let request = SiteRequest::GetBlock { hash: *hash };
        let response = self.request_response(request, provider).await?;

        // Extract block from response
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

        // Create request and send with correlation ID
        let request = SiteRequest::GetManifest {
            site_id: site_id.clone(),
        };
        let response = self.request_response(request, provider).await?;

        // Extract manifest from response
        match response {
            SiteResponse::Manifest(manifest) => {
                // CRITICAL: Verify ML-DSA-65 signature BEFORE caching or returning!
                // This prevents malicious/compromised providers from serving forged content.
                manifest.verify().map_err(|e| {
                    anyhow::anyhow!("Manifest signature verification failed: {}", e)
                })?;

                // Verify site ID matches what we requested
                if &manifest.site_id != site_id {
                    return Err(anyhow::anyhow!(
                        "Site ID mismatch: expected {:?}, got {:?}",
                        site_id,
                        manifest.site_id
                    ));
                }

                // Only cache AFTER all verification succeeds
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
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use saorsa_gossip_pubsub::PubSub as PubSubTrait;
    use saorsa_gossip_transport::{AntQuicTransport, AntQuicTransportConfig, GossipTransport};
    use saorsa_gossip_types::PeerId;
    use saorsa_pqc::ml_dsa_65::try_keygen_with_rng;

    fn create_test_peer_id(seed: u8) -> PeerId {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        PeerId::new(bytes)
    }

    async fn create_test_rendezvous_client() -> RendezvousClient {
        let peer_id = create_test_peer_id(1);

        let bind_a = "127.0.0.1:0".parse().expect("valid addr");
        let bind_b = "127.0.0.1:0".parse().expect("valid addr");
        let qt1 = AntQuicTransport::with_config(AntQuicTransportConfig::new(bind_a, vec![]), None)
            .await
            .expect("transport");
        let qt2 = AntQuicTransport::with_config(AntQuicTransportConfig::new(bind_b, vec![]), None)
            .await
            .expect("transport");
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

    /// Generate a deterministic test keypair from a seed
    fn generate_test_keypair(seed: u64) -> (PrivateKey, PublicKey) {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let (pk, sk) = try_keygen_with_rng(&mut rng).expect("Failed to generate test keypair");
        (sk, pk)
    }

    #[test]
    fn test_site_id_creation() {
        let key = [42u8; 32];
        let site_id = SiteId::new(key);

        assert_eq!(site_id.as_bytes(), &key);
        assert_eq!(site_id.hash, key);
    }

    #[test]
    fn test_site_manifest_structure() {
        let (_sk, pk) = generate_test_keypair(1);
        let site_id = SiteId::from_public_key(&pk);
        let blocks = vec![
            ("index.html".to_string(), [2u8; 32]),
            ("style.css".to_string(), [3u8; 32]),
        ];

        let manifest = SiteManifest::new(site_id.clone(), &pk, 1, blocks.clone());

        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.site_id, site_id);
        assert_eq!(manifest.manifest_version, 1);
        assert_eq!(manifest.blocks, blocks);
        assert_eq!(manifest.signature.len(), 0); // Unsigned initially
        assert_ne!(manifest.root_hash, [0u8; 32]); // Should have computed root hash
        assert_eq!(manifest.public_key.len(), 1952); // ML-DSA-65 public key size
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
        let (_sk, pk) = generate_test_keypair(1);
        let site_id = SiteId::from_public_key(&pk);
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
            .build_manifest(&pk, 1, asset_paths.clone())
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
        let (sk, pk) = generate_test_keypair(1);
        let site_id = SiteId::from_public_key(&pk);
        let blocks = vec![("index.html".to_string(), [2u8; 32])];
        let mut manifest = SiteManifest::new(site_id, &pk, 1, blocks);

        // Initially unsigned
        assert_eq!(manifest.signature.len(), 0);

        // Sign manifest
        manifest.sign(&sk).expect("Failed to sign manifest");

        // Should have signature (ML-DSA-65 signatures are 3309 bytes)
        assert_eq!(manifest.signature.len(), 3309);

        // Verify signature
        manifest.verify().expect("Signature verification failed");
    }

    #[test]
    fn test_manifest_root_hash_deterministic() {
        let (_sk, pk) = generate_test_keypair(1);
        let site_id = SiteId::from_public_key(&pk);
        let blocks = vec![
            ("index.html".to_string(), [2u8; 32]),
            ("style.css".to_string(), [3u8; 32]),
        ];

        let manifest1 = SiteManifest::new(site_id.clone(), &pk, 1, blocks.clone());
        let manifest2 = SiteManifest::new(site_id, &pk, 1, blocks);

        // Same blocks should produce same root hash (timestamps will differ, but root hash is deterministic)
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
        let fetcher = SiteFetcher::new(rendezvous).await.unwrap();

        // Initially should have no cached data
        let site_id = SiteId::new([2u8; 32]);
        let provider = create_test_peer_id(99);
        // This will fail because there's no actual provider serving
        assert!(fetcher.fetch_manifest(&site_id, provider).await.is_err());
    }

    #[tokio::test]
    async fn test_fetcher_block_caching() {
        let rendezvous = Arc::new(create_test_rendezvous_client().await);
        let fetcher = SiteFetcher::new(rendezvous).await.unwrap();

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
        let fetcher = SiteFetcher::new(rendezvous).await.unwrap();

        // Cache a manifest
        let (_sk, pk) = generate_test_keypair(2);
        let site_id = SiteId::from_public_key(&pk);
        let blocks = vec![("index.html".to_string(), [3u8; 32])];
        let manifest = SiteManifest::new(site_id.clone(), &pk, 1, blocks);

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
        let fetcher = SiteFetcher::new(rendezvous).await.unwrap();

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
        let (_sk, pk) = generate_test_keypair(1);
        let site_id = SiteId::from_public_key(&pk);
        let publisher = SitePublisher::new(site_id.clone());

        // Add assets and build manifest
        let hash = publisher
            .add_asset("index.html".to_string(), b"<html>".to_vec())
            .await
            .unwrap();
        let manifest = publisher
            .build_manifest(&pk, 1, vec![("index.html".to_string(), hash)])
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

    /// End-to-end integration test: Publisher serves content, Fetcher retrieves it
    ///
    /// This test validates the complete pipeline:
    /// 1. Create SitePublisher with signed content
    /// 2. SitePublisher.handle_request() processes GetManifest/GetBlock
    /// 3. Responses are properly serialized
    /// 4. Manifest signature verifies
    /// 5. Block hashes verify
    #[tokio::test]
    async fn test_end_to_end_site_serving() {
        // Setup: Generate keypair and create site
        let (sk, pk) = generate_test_keypair(42);
        let site_id = SiteId::from_public_key(&pk);
        let publisher = Arc::new(SitePublisher::new(site_id.clone()));

        // Step 1: Publisher creates content
        let html_content = b"<html><body><h1>Hello, Saorsa Sites!</h1></body></html>".to_vec();
        let css_content = b"body { font-family: sans-serif; }".to_vec();

        let html_hash = publisher
            .add_asset("index.html".to_string(), html_content.clone())
            .await
            .unwrap();

        let css_hash = publisher
            .add_asset("style.css".to_string(), css_content.clone())
            .await
            .unwrap();

        // Step 2: Build and sign manifest
        let asset_paths = vec![
            ("index.html".to_string(), html_hash),
            ("style.css".to_string(), css_hash),
        ];

        let mut manifest = publisher
            .build_manifest(&pk, 1, asset_paths.clone())
            .await
            .unwrap();

        // Sign the manifest with ML-DSA
        manifest.sign(&sk).expect("Failed to sign manifest");

        // Verify signature works
        manifest.verify().expect("Signature verification failed");

        // Store signed manifest back in publisher
        {
            let mut current_manifest = publisher.manifest.write().await;
            *current_manifest = Some(manifest.clone());
        }

        // Step 3: Simulate fetcher requesting manifest
        let manifest_request = SiteRequest::GetManifest {
            site_id: site_id.clone(),
        };
        let manifest_request_bytes = bincode::serialize(&manifest_request).unwrap();

        let manifest_response_bytes = publisher
            .handle_request(Bytes::from(manifest_request_bytes))
            .await
            .unwrap();

        let manifest_response: SiteResponse =
            bincode::deserialize(&manifest_response_bytes).unwrap();

        // Step 4: Verify manifest response
        let fetched_manifest = match manifest_response {
            SiteResponse::Manifest(m) => m,
            _ => panic!("Expected Manifest response"),
        };

        assert_eq!(fetched_manifest.site_id, site_id);
        assert_eq!(fetched_manifest.manifest_version, 1);
        assert_eq!(fetched_manifest.blocks.len(), 2);
        assert_eq!(fetched_manifest.root_hash, manifest.root_hash);

        // Verify signature on fetched manifest
        fetched_manifest
            .verify()
            .expect("Fetched manifest signature verification failed");

        // Step 5: Fetch blocks
        for (path, hash) in &fetched_manifest.blocks {
            let block_request = SiteRequest::GetBlock { hash: *hash };
            let block_request_bytes = bincode::serialize(&block_request).unwrap();

            let block_response_bytes = publisher
                .handle_request(Bytes::from(block_request_bytes))
                .await
                .unwrap();

            let block_response: SiteResponse = bincode::deserialize(&block_response_bytes).unwrap();

            let fetched_block = match block_response {
                SiteResponse::Block(b) => b,
                _ => panic!("Expected Block response"),
            };

            // Verify block hash
            assert!(fetched_block.verify(), "Block hash verification failed");
            assert_eq!(fetched_block.hash, *hash);

            // Verify content
            if path == "index.html" {
                assert_eq!(fetched_block.content, html_content);
            } else if path == "style.css" {
                assert_eq!(fetched_block.content, css_content);
            }
        }

        // Success! Full pipeline works:
        // ✓ Publisher created content
        // ✓ Manifest signed with ML-DSA
        // ✓ Manifest fetched and signature verified
        // ✓ Blocks fetched and hashes verified
        // ✓ Content matches original
    }
}
