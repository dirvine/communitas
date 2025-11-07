// Copyright (c) 2025 Saorsa Labs Limited
//
// Network Integration Test Utilities
//
// Provides infrastructure for testing Sites protocol over real QUIC networks

use communitas_core::gossip::{GossipContext, SiteId, SiteManifest};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

/// Get a random available port for testing
pub fn get_random_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .expect("Failed to get local addr")
        .port()
}

/// Get random IPv6 port
pub fn get_random_ipv6_port() -> u16 {
    TcpListener::bind("[::1]:0")
        .expect("Failed to bind to IPv6 random port")
        .local_addr()
        .expect("Failed to get local addr")
        .port()
}

/// Test node wrapper with cleanup and metrics
pub struct TestNode {
    pub name: String,
    pub ctx: Arc<GossipContext>,
    pub gossip_port: u16,
    pub sites_port: u16,
    pub created_at: Instant,
}

impl TestNode {
    /// Create a new test node with random ports (IPv4)
    pub async fn new(name: &str) -> Self {
        let port = get_random_port();
        Self::new_with_port(name, port).await
    }

    /// Create a new test node with specific port
    pub async fn new_with_port(name: &str, port: u16) -> Self {
        let four_words = format!("{}-{}-test-node", name, port);

        info!("Creating test node '{}' on port {}", name, port);

        let ctx = GossipContext::initialize(
            four_words,
            name.to_string(),
            "test-device".to_string(),
            Some(port),
        )
        .await
        .expect("Failed to initialize test node");

        let sites_port = port + 1; // Sites uses main_port + 1

        info!(
            "Test node '{}': gossip={}, sites={}, peer_id={:?}",
            name,
            port,
            sites_port,
            ctx.peer_id()
        );

        Self {
            name: name.to_string(),
            ctx: Arc::new(ctx),
            gossip_port: port,
            sites_port,
            created_at: Instant::now(),
        }
    }

    /// Create IPv6 test node
    pub async fn new_ipv6(name: &str) -> Self {
        let port = get_random_ipv6_port();
        let four_words = format!("{}-ipv6-{}-test", name, port);

        info!("Creating IPv6 test node '{}' on port {}", name, port);

        let ctx = GossipContext::initialize(
            four_words,
            name.to_string(),
            "test-device".to_string(),
            Some(port),
        )
        .await
        .expect("Failed to initialize IPv6 test node");

        Self {
            name: name.to_string(),
            ctx: Arc::new(ctx),
            gossip_port: port,
            sites_port: port + 1,
            created_at: Instant::now(),
        }
    }

    /// Get peer ID
    pub fn peer_id(&self) -> saorsa_gossip_types::PeerId {
        self.ctx.peer_id()
    }

    /// Publish a site with signed manifest
    ///
    /// Returns (SiteId, signed SiteManifest, total bytes)
    pub async fn publish_site(
        &self,
        files: &[(&str, &[u8])], // (path, content)
    ) -> Result<(SiteId, SiteManifest, usize), String> {
        let start = Instant::now();

        let publisher = self.ctx.site_publisher.as_ref().ok_or("No publisher")?;

        // Get signing keys via type conversion
        let (public_key, private_key) = self
            .ctx
            .get_sites_signing_keys()
            .map_err(|e| format!("Failed to get signing keys: {}", e))?;

        let site_id = SiteId::from_public_key(&public_key);

        let mut total_bytes = 0;
        let mut asset_paths = vec![];

        // Add all assets
        for (path, content) in files.iter() {
            total_bytes += content.len();
            let hash = publisher
                .add_asset(path.to_string(), content.to_vec())
                .await
                .map_err(|e| format!("Failed to add asset {}: {}", path, e))?;
            asset_paths.push((path.to_string(), hash));
        }

        // Build manifest
        let mut manifest = publisher
            .build_manifest(&public_key, 1, asset_paths)
            .await
            .map_err(|e| format!("Failed to build manifest: {}", e))?;

        // CRITICAL: Sign with ML-DSA-87
        manifest
            .sign(&private_key)
            .map_err(|e| format!("Failed to sign manifest: {}", e))?;

        // Verify locally
        manifest
            .verify()
            .map_err(|e| format!("Local verification failed: {}", e))?;

        // Store signed manifest
        publisher
            .set_manifest(manifest.clone())
            .await
            .map_err(|e| format!("Failed to store manifest: {}", e))?;

        let elapsed = start.elapsed();
        info!(
            "Node '{}': Published {} files ({} bytes) in {:?}",
            self.name,
            files.len(),
            total_bytes,
            elapsed
        );

        Ok((site_id, manifest, total_bytes))
    }

    /// Fetch a site and measure performance
    ///
    /// Returns (fetched manifest, blocks, total_bytes, elapsed_time)
    pub async fn fetch_site(
        &self,
        site_id: &SiteId,
        provider_peer_id: saorsa_gossip_types::PeerId,
    ) -> Result<
        (
            SiteManifest,
            Vec<communitas_core::gossip::Block>,
            usize,
            std::time::Duration,
        ),
        String,
    > {
        let start = Instant::now();

        let fetcher = self.ctx.site_fetcher.as_ref().ok_or("No fetcher")?;

        // Fetch manifest
        let manifest = fetcher
            .fetch_manifest(site_id, provider_peer_id)
            .await
            .map_err(|e| format!("Failed to fetch manifest: {}", e))?;

        // CRITICAL: Verify signature after network fetch
        manifest
            .verify()
            .map_err(|e| format!("Manifest signature verification failed: {}", e))?;

        info!(
            "Node '{}': Fetched and verified manifest (v{})",
            self.name, manifest.manifest_version
        );

        // Fetch all blocks
        let mut blocks = vec![];
        let mut total_bytes = 0;

        for (path, hash) in &manifest.blocks {
            let block = fetcher
                .fetch_block(hash, provider_peer_id)
                .await
                .map_err(|e| format!("Failed to fetch block {}: {}", path, e))?;

            // Verify block hash
            if !block.verify() {
                return Err(format!("Block hash verification failed for {}", path));
            }

            total_bytes += block.content.len();
            blocks.push(block);
        }

        let elapsed = start.elapsed();

        let throughput_mbps = if elapsed.as_secs_f64() > 0.0 {
            (total_bytes as f64 / elapsed.as_secs_f64()) / (1024.0 * 1024.0)
        } else {
            0.0
        };

        info!(
            "Node '{}': Fetched {} blocks ({} bytes) in {:?} ({:.2} MB/s)",
            self.name,
            blocks.len(),
            total_bytes,
            elapsed,
            throughput_mbps
        );

        Ok((manifest, blocks, total_bytes, elapsed))
    }

    /// Connect to another test node by exchanging peer addresses
    ///
    /// This establishes the necessary peer-to-peer routing for QUIC communication
    /// by adding each node's peer ID and address to the other's discovery cache.
    ///
    /// IMPORTANT: Sites protocol uses Sites ports (gossip_port + 1), not gossip ports
    pub async fn connect_to_peer(&self, other: &TestNode) -> Result<(), String> {
        use local_ip_address::local_ip;
        use std::net::SocketAddr;

        // Get local IP and construct socket addresses for both nodes
        // CRITICAL: Use Sites ports for Sites protocol communication
        let local_ip = local_ip().map_err(|e| format!("Failed to get local IP: {}", e))?;

        let self_sites_addr = SocketAddr::new(local_ip, self.sites_port);
        let other_sites_addr = SocketAddr::new(local_ip, other.sites_port);

        // Exchange peer information via discovery cache
        self.ctx
            .add_contact(other.ctx.four_words().to_string(), other.peer_id())
            .await
            .map_err(|e| format!("Failed to add peer to discovery: {}", e))?;

        other
            .ctx
            .add_contact(self.ctx.four_words().to_string(), self.peer_id())
            .await
            .map_err(|e| format!("Failed to add self to peer discovery: {}", e))?;

        // Also add to peer cache with Sites address hints for transport routing
        {
            let mut self_cache = self.ctx.peer_cache.write().await;
            self_cache
                .update_success(other.peer_id(), other_sites_addr)
                .await
                .map_err(|e| format!("Failed to update peer cache: {}", e))?;
        }

        {
            let mut other_cache = other.ctx.peer_cache.write().await;
            other_cache
                .update_success(self.peer_id(), self_sites_addr)
                .await
                .map_err(|e| format!("Failed to update peer cache: {}", e))?;
        }

        // CRITICAL: Establish direct connection between Sites transports
        // The Sites transport has its own peer routing that needs to be primed
        // We need to ensure the Sites transport can reach the peer's Sites port
        info!("Establishing direct Sites transport connection...");

        let self_peer_id = self.peer_id();
        let other_peer_id = other.peer_id();

        // For Sites transport: add peer address hints directly to peer cache
        // Sites uses sites_port (main_port + 1) for communication
        info!(
            "Priming Sites transport peer routing: {} -> {}",
            self_peer_id, other_sites_addr
        );

        // Use the context method to establish Sites peer routing
        // This sends a dummy ping to prime the Sites transport's routing table
        if let Err(e) = self.ctx.establish_sites_peer_routing(other_peer_id).await {
            warn!(
                "Failed to establish Sites peer routing from self to other: {}",
                e
            );
        }

        if let Err(e) = other.ctx.establish_sites_peer_routing(self_peer_id).await {
            warn!(
                "Failed to establish Sites peer routing from other to self: {}",
                e
            );
        }

        // Give the transports time to establish routes
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        info!("Sites transport peer routing established");

        // Give transports time to process the peer information
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        info!(
            "Node '{}' connected to '{}' (Sites: {} <-> {})",
            self.name, other.name, self_sites_addr, other_sites_addr
        );

        Ok(())
    }

    /// Advertise as provider via rendezvous
    pub async fn advertise_as_provider(
        &self,
        site_id: &SiteId,
        manifest_version: u64,
    ) -> Result<(), String> {
        use communitas_core::gossip::ProviderSummary;
        use saorsa_gossip_rendezvous::Capability;

        let rendezvous = &self.ctx.rendezvous;

        let mut summary = ProviderSummary::new(
            site_id.hash,
            self.ctx.peer_id(),
            vec![Capability::Site],
            3_600_000, // 1 hour
        )
        .with_manifest_version(manifest_version)
        .with_root(true);

        // Sign with identity keypair
        // ProviderSummary.sign() takes &MlDsaSecretKey, get from typed method
        let identity_kp = self.ctx.get_identity_keypair();
        let secret_key = identity_kp
            .get_secret_key_typed()
            .map_err(|e| format!("Failed to get secret key: {}", e))?;
        summary
            .sign(&secret_key)
            .map_err(|e| format!("Failed to sign ProviderSummary: {}", e))?;

        // Publish to rendezvous
        rendezvous
            .publish_provider_summary(summary)
            .await
            .map_err(|e| format!("Failed to publish to rendezvous: {}", e))?;

        info!(
            "Node '{}': Advertised as provider for site {:?}",
            self.name, site_id
        );

        Ok(())
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        let uptime = self.created_at.elapsed();
        info!(
            "Cleaning up test node '{}' (uptime: {:?}, ports: {}/{})",
            self.name, uptime, self.gossip_port, self.sites_port
        );
        // Transport cleanup handled by Arc drop
    }
}

/// Performance metrics for test validation
pub struct TestMetrics {
    pub publish_time_ms: u64,
    pub fetch_time_ms: u64,
    pub total_bytes: usize,
    pub throughput_mbps: f64,
    pub blocks_count: usize,
}

impl TestMetrics {
    pub fn print_summary(&self) {
        println!("\n=== Performance Metrics ===");
        println!("Publish time:  {} ms", self.publish_time_ms);
        println!("Fetch time:    {} ms", self.fetch_time_ms);
        println!(
            "Total bytes:   {} ({:.2} MB)",
            self.total_bytes,
            self.total_bytes as f64 / (1024.0 * 1024.0)
        );
        println!("Throughput:    {:.2} MB/s", self.throughput_mbps);
        println!("Blocks:        {}", self.blocks_count);
        println!("=========================\n");
    }
}
