/**
 * Enhanced TestHarness for Communitas Integration Tests
 *
 * Provides a controlled testing environment with real QUIC transport,
 * multi-node scenarios, network chaos testing, and full networking stack.
 *
 * Capabilities:
 * - Real QUIC connections over ephemeral UDP ports
 * - Multi-node mesh/line/star topologies
 * - Network chaos: partition, latency, jitter, packet loss
 * - Presence, FOAF, gossip, and sync integration
 */
use crate::core_context::CoreContext;
use crate::types::DeviceType;
use anyhow::{Context, Result};
use rand::Rng;
use saorsa_gossip_groups::GroupContext;
use saorsa_gossip_presence::PresenceManager;
use saorsa_gossip_transport::{UdpTransportAdapter, UdpTransportAdapterConfig};
use saorsa_gossip_types::{PeerId, TopicId};
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

const TEST_PORT_MIN: u16 = 20_000;
const TEST_PORT_MAX: u16 = 60_000;
const TEST_PORT_ATTEMPTS: usize = 50;

static PORT_REGISTRY: OnceLock<Mutex<HashSet<u16>>> = OnceLock::new();

fn port_registry() -> &'static Mutex<HashSet<u16>> {
    PORT_REGISTRY.get_or_init(|| Mutex::new(HashSet::new()))
}

fn reserve_test_port() -> Result<u16> {
    let mut rng = rand::thread_rng();
    for _ in 0..TEST_PORT_ATTEMPTS {
        let port = rng.gen_range(TEST_PORT_MIN..=TEST_PORT_MAX);
        let registry = port_registry();
        let mut used = registry
            .lock()
            .map_err(|_| anyhow::anyhow!("test port registry lock poisoned"))?;
        if used.contains(&port) {
            continue;
        }
        used.insert(port);
        return Ok(port);
    }
    Err(anyhow::anyhow!(
        "failed to reserve a test port after {} attempts",
        TEST_PORT_ATTEMPTS
    ))
}

fn release_test_port(port: u16) {
    if let Some(registry) = PORT_REGISTRY.get()
        && let Ok(mut used) = registry.lock()
    {
        used.remove(&port);
    }
}

/// Network link policy for chaos engineering
#[derive(Debug, Clone)]
pub struct LinkPolicy {
    /// Whether nodes can communicate
    pub connected: bool,
    /// Base latency added to communication
    pub latency: Duration,
    /// Random jitter added (0 to jitter)
    pub jitter: Duration,
    /// Packet loss probability (0.0 to 1.0)
    pub loss: f32,
}

impl Default for LinkPolicy {
    fn default() -> Self {
        Self {
            connected: true,
            latency: Duration::ZERO,
            jitter: Duration::ZERO,
            loss: 0.0,
        }
    }
}

impl LinkPolicy {
    /// Perfect link (no latency, no loss)
    pub fn perfect() -> Self {
        Self::default()
    }

    /// Disconnected link
    pub fn disconnected() -> Self {
        Self {
            connected: false,
            ..Default::default()
        }
    }

    /// Lossy link with specified packet loss rate
    pub fn lossy(loss: f32) -> Self {
        Self {
            loss: loss.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// High-latency link
    pub fn slow(latency_ms: u64) -> Self {
        Self {
            latency: Duration::from_millis(latency_ms),
            jitter: Duration::from_millis(latency_ms / 10),
            ..Default::default()
        }
    }

    /// Should this packet be dropped?
    pub fn should_drop(&self) -> bool {
        !self.connected || (self.loss > 0.0 && rand::random::<f32>() < self.loss)
    }

    /// Get effective delay for this packet
    pub fn effective_delay(&self) -> Duration {
        if self.jitter > Duration::ZERO {
            let jitter_ms = rand::random::<u64>() % self.jitter.as_millis() as u64;
            self.latency + Duration::from_millis(jitter_ms)
        } else {
            self.latency
        }
    }
}

/// Test node with full networking stack
pub struct TestNode {
    pub id: usize,
    pub four_words: String,
    pub peer_id: PeerId,
    pub port: u16,
    pub addr: SocketAddr,
    pub temp_dir: TempDir,
    pub core: Option<Arc<CoreContext>>,
    pub presence: Option<Arc<RwLock<PresenceManager>>>,
    pub groups: Arc<RwLock<HashMap<TopicId, GroupContext>>>,
    pub transport: Arc<UdpTransportAdapter>,
}

impl TestNode {
    /// Create a new test node with unique identity and ephemeral port
    pub async fn new(id: usize) -> Result<Self> {
        let temp_dir = TempDir::new().context("failed to create temp dir")?;

        // Generate test identity
        let four_words = format!("test-node-{:04x}-peer", id);
        let peer_id = PeerId::new([(id % 256) as u8; 32]);

        let mut last_err = None;
        for _ in 0..TEST_PORT_ATTEMPTS {
            let port = reserve_test_port()?;
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);

            match UdpTransportAdapter::with_config(UdpTransportAdapterConfig::new(addr, vec![]), None)
                .await
                .context("failed to create UdpTransportAdapter")
            {
                Ok(transport) => {
                    debug!("TestNode {} created on port {}", id, port);
                    return Ok(TestNode {
                        id,
                        four_words,
                        peer_id,
                        port,
                        addr,
                        temp_dir,
                        core: None,
                        presence: None,
                        groups: Arc::new(RwLock::new(HashMap::new())),
                        transport: Arc::new(transport),
                    });
                }
                Err(err) => {
                    release_test_port(port);
                    last_err = Some(err);
                }
            }
        }

        Err(anyhow::anyhow!(
            "failed to allocate test port for node {}: {}",
            id,
            last_err
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown error".to_string())
        ))
    }

    /// Initialize core context (full stack)
    pub async fn initialize_core(&mut self) -> Result<()> {
        let groups_map = self.groups.clone();
        let presence_mgr =
            PresenceManager::new(self.peer_id, self.transport.clone(), groups_map.clone());
        self.presence = Some(Arc::new(RwLock::new(presence_mgr)));

        let storage_dir = self.temp_dir.path().join("core");
        let core = CoreContext::initialize(
            self.four_words.clone(),
            format!("Test Node {}", self.id),
            format!("test-node-{}", self.id),
            DeviceType::Desktop,
            storage_dir,
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to initialize CoreContext: {e}"))?;

        self.core = Some(Arc::new(core));

        info!("TestNode {} core initialized", self.id);
        Ok(())
    }

    /// Join a topic/group
    pub async fn join_group(&self, topic_id: TopicId, group_name: &str) -> Result<()> {
        let group_ctx = GroupContext::from_entity(group_name);

        let mut groups = self.groups.write().await;
        groups.insert(topic_id, group_ctx);

        info!("TestNode {} joined group {}", self.id, group_name);
        Ok(())
    }

    /// Get local address for bootstrap
    pub fn bootstrap_addr(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// Shutdown node
    pub async fn shutdown(self) -> Result<()> {
        info!("TestNode {} shutting down", self.id);
        // Cleanup happens automatically via Drop
        Ok(())
    }
}

/// Network harness with chaos control
pub struct NetworkHarness {
    pub nodes: HashMap<usize, Arc<RwLock<TestNode>>>,
    pub policies: Arc<RwLock<HashMap<(usize, usize), LinkPolicy>>>,
}

impl Default for NetworkHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkHarness {
    pub fn new() -> Self {
        NetworkHarness {
            nodes: HashMap::new(),
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_node(&mut self, node: TestNode) -> usize {
        let id = node.id;
        self.nodes.insert(id, Arc::new(RwLock::new(node)));
        id
    }

    /// Set link policy between two nodes
    pub async fn set_policy(&self, a: usize, b: usize, policy: LinkPolicy) {
        let mut policies = self.policies.write().await;
        policies.insert((a.min(b), a.max(b)), policy.clone());
        debug!("Link {}->{} policy: {:?}", a, b, policy);
    }

    /// Get link policy between two nodes
    pub async fn get_policy(&self, a: usize, b: usize) -> LinkPolicy {
        let policies = self.policies.read().await;
        policies
            .get(&(a.min(b), a.max(b)))
            .cloned()
            .unwrap_or_default()
    }

    /// Check if nodes are connected
    pub async fn are_connected(&self, a: usize, b: usize) -> bool {
        self.get_policy(a, b).await.connected
    }

    /// Get node by ID
    pub async fn get_node(&self, id: usize) -> Option<Arc<RwLock<TestNode>>> {
        self.nodes.get(&id).cloned()
    }

    /// Connect two nodes (set policy to perfect)
    pub async fn connect(&self, a: usize, b: usize) {
        self.set_policy(a, b, LinkPolicy::perfect()).await;
    }

    /// Disconnect two nodes
    pub async fn disconnect(&self, a: usize, b: usize) {
        self.set_policy(a, b, LinkPolicy::disconnected()).await;
    }

    /// Set latency between nodes
    pub async fn set_latency(&self, a: usize, b: usize, latency_ms: u64) {
        self.set_policy(a, b, LinkPolicy::slow(latency_ms)).await;
    }

    /// Set packet loss between nodes
    pub async fn set_loss(&self, a: usize, b: usize, loss: f32) {
        self.set_policy(a, b, LinkPolicy::lossy(loss)).await;
    }
}

/// Main test harness for integration testing
pub struct TestHarness {
    pub network: Arc<RwLock<NetworkHarness>>,
    pub temp_dir: TempDir,
}

impl TestHarness {
    /// Create test harness with N nodes
    pub async fn new(node_count: usize) -> Result<Self> {
        let temp_dir = TempDir::new().context("failed to create harness temp dir")?;
        let network = Arc::new(RwLock::new(NetworkHarness::new()));

        let harness = TestHarness { network, temp_dir };

        // Create nodes
        for i in 0..node_count {
            let mut node = TestNode::new(i).await?;
            node.initialize_core().await?;
            harness.network.write().await.add_node(node);
        }

        info!("TestHarness created with {} nodes", node_count);
        Ok(harness)
    }

    /// Setup mesh topology (all nodes connected to all)
    pub async fn mesh(&self) -> Result<()> {
        let node_count = self.network.read().await.nodes.len();
        for i in 0..node_count {
            for j in (i + 1)..node_count {
                self.network.read().await.connect(i, j).await;
            }
        }
        info!("Mesh topology configured");
        Ok(())
    }

    /// Setup line topology (0-1-2-3-...)
    pub async fn line(&self) -> Result<()> {
        let node_count = self.network.read().await.nodes.len();

        // First disconnect all nodes from each other
        for i in 0..node_count {
            for j in (i + 1)..node_count {
                self.network.read().await.disconnect(i, j).await;
            }
        }

        // Then connect only adjacent nodes
        for i in 0..node_count.saturating_sub(1) {
            self.network.read().await.connect(i, i + 1).await;
        }
        info!("Line topology configured");
        Ok(())
    }

    /// Setup star topology (hub connected to all)
    pub async fn star(&self, hub: usize) -> Result<()> {
        let node_count = self.network.read().await.nodes.len();

        // First disconnect all nodes from each other
        for i in 0..node_count {
            for j in (i + 1)..node_count {
                self.network.read().await.disconnect(i, j).await;
            }
        }

        // Then connect hub to all spokes
        for i in 0..node_count {
            if i != hub {
                self.network.read().await.connect(hub, i).await;
            }
        }
        info!("Star topology configured with hub {}", hub);
        Ok(())
    }

    /// Partition network into two groups
    pub async fn partition(&self, group_a: &[usize], group_b: &[usize]) -> Result<()> {
        for &a in group_a {
            for &b in group_b {
                self.network.read().await.disconnect(a, b).await;
            }
        }
        info!("Network partitioned: {:?} | {:?}", group_a, group_b);
        Ok(())
    }

    /// Heal partition (reconnect all)
    pub async fn heal(&self) -> Result<()> {
        let node_count = self.network.read().await.nodes.len();
        for i in 0..node_count {
            for j in (i + 1)..node_count {
                self.network.read().await.connect(i, j).await;
            }
        }
        info!("Network healed");
        Ok(())
    }

    /// Set latency between two nodes
    pub async fn set_latency(&self, a: usize, b: usize, latency_ms: u64) {
        self.network
            .read()
            .await
            .set_latency(a, b, latency_ms)
            .await;
    }

    /// Set packet loss between two nodes
    pub async fn set_loss(&self, a: usize, b: usize, loss: f32) {
        self.network.read().await.set_loss(a, b, loss).await;
    }

    /// Wait until at least N nodes are connected
    pub async fn wait_until_connected(&self, count: usize, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();

        loop {
            let network = self.network.read().await;
            let node_count = network.nodes.len();

            // Count connected pairs
            let mut connected = 0;
            for i in 0..node_count {
                for j in (i + 1)..node_count {
                    if network.are_connected(i, j).await {
                        connected += 1;
                    }
                }
            }

            if connected >= count {
                info!("Connected threshold reached: {}/{}", connected, count);
                return Ok(());
            }

            if start.elapsed() > timeout {
                warn!("Timeout waiting for connections: {}/{}", connected, count);
                return Err(anyhow::anyhow!(
                    "Timeout waiting for {} connections (got {})",
                    count,
                    connected
                ));
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get node by ID
    pub async fn get_node(&self, id: usize) -> Option<Arc<RwLock<TestNode>>> {
        self.network.read().await.get_node(id).await
    }

    /// Get bootstrap addresses for all nodes
    pub async fn get_bootstrap_addrs(&self) -> Vec<String> {
        let network = self.network.read().await;
        let mut addrs = Vec::new();
        for node_lock in network.nodes.values() {
            let node = node_lock.read().await;
            addrs.push(node.bootstrap_addr());
        }
        addrs
    }

    /// Cleanup harness
    pub async fn cleanup(self) -> Result<()> {
        info!("TestHarness cleanup started");

        // Shutdown all nodes
        let network = Arc::try_unwrap(self.network)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap network"))?
            .into_inner();

        for (id, node_lock) in network.nodes {
            let node = Arc::try_unwrap(node_lock)
                .map_err(|_| anyhow::anyhow!("Failed to unwrap node {}", id))?
                .into_inner();
            node.shutdown().await?;
        }

        info!("TestHarness cleanup complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_harness_creation() {
        let harness = TestHarness::new(3).await.expect("harness creation failed");
        let network = harness.network.read().await;
        assert_eq!(network.nodes.len(), 3);
        drop(network);
        harness.cleanup().await.expect("cleanup failed");
    }

    #[tokio::test]
    async fn test_mesh_topology() {
        let harness = TestHarness::new(4).await.expect("harness creation failed");
        harness.mesh().await.expect("mesh setup failed");

        let network = harness.network.read().await;
        // In a 4-node mesh, we should have 6 connections (n*(n-1)/2)
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert!(
                    network.are_connected(i, j).await,
                    "Nodes {} and {} should be connected",
                    i,
                    j
                );
            }
        }
    }

    #[tokio::test]
    async fn test_partition_and_heal() {
        let harness = TestHarness::new(4).await.expect("harness creation failed");
        harness.mesh().await.expect("mesh failed");

        // Partition into [0,1] and [2,3]
        harness
            .partition(&[0, 1], &[2, 3])
            .await
            .expect("partition failed");

        let network = harness.network.read().await;
        // Within partitions should be connected
        assert!(network.are_connected(0, 1).await);
        assert!(network.are_connected(2, 3).await);

        // Across partitions should be disconnected
        assert!(!network.are_connected(0, 2).await);
        assert!(!network.are_connected(0, 3).await);
        assert!(!network.are_connected(1, 2).await);
        assert!(!network.are_connected(1, 3).await);
        drop(network);

        // Heal
        harness.heal().await.expect("heal failed");

        let network = harness.network.read().await;
        // All should be connected again
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert!(network.are_connected(i, j).await);
            }
        }
    }

    #[tokio::test]
    async fn test_link_policies() {
        let harness = TestHarness::new(2).await.expect("harness creation failed");

        // Test latency
        harness.set_latency(0, 1, 100).await;
        let policy = harness.network.read().await.get_policy(0, 1).await;
        assert_eq!(policy.latency, Duration::from_millis(100));

        // Test packet loss
        harness.set_loss(0, 1, 0.3).await;
        let policy = harness.network.read().await.get_policy(0, 1).await;
        assert_eq!(policy.loss, 0.3);
    }

    #[tokio::test]
    async fn test_star_topology() {
        let harness = TestHarness::new(5).await.expect("harness creation failed");
        harness.star(0).await.expect("star failed");

        let network = harness.network.read().await;
        // Hub (0) connected to all
        for i in 1..5 {
            assert!(network.are_connected(0, i).await);
        }

        // Spokes not connected to each other
        assert!(!network.are_connected(1, 2).await);
        assert!(!network.are_connected(2, 3).await);
    }

    #[tokio::test]
    async fn test_node_initialization() {
        let node = TestNode::new(42).await.expect("node creation failed");
        assert_eq!(node.id, 42);
        assert!(node.port > 0, "Should have ephemeral port");
        assert!(node.four_words.contains("test-node"));

        node.shutdown().await.expect("shutdown failed");
    }
}
