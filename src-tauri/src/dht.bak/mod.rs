// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


//! DHT (Distributed Hash Table) Module for Communitas
//!
//! This module implements a production-ready Kademlia-based DHT with:
//! - K=8 replication for high availability
//! - BLAKE3 content addressing for integrity
//! - Fault tolerance and self-healing network topology
//! - Mobile/battery optimizations
//! - Comprehensive security measures
//!
//! ## Architecture
//!
//! The DHT is built around several core components:
//! - **Kademlia**: Core routing algorithm with XOR-based distance metric
//! - **RoutingTable**: Binary tree structure with 160 buckets for efficient lookups
//! - **ContentStore**: BLAKE3-based storage with integrity validation
//! - **PeerManager**: Dynamic peer discovery and connection management
//! - **NetworkTopology**: Real-time network health monitoring and partition detection
//! - **FaultTolerance**: Automatic failure detection and self-healing mechanisms
//! - **Performance**: Advanced optimizations for mobile and battery constraints

use anyhow::{Result};
use serde::{Deserialize, Serialize};
use std::collections::{VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error, instrument};

// Core modules
pub mod kademlia;
pub mod routing_table;
pub mod content_store;
pub mod peer_manager;
pub mod network_topology;
pub mod fault_tolerance;
pub mod performance;

// Re-export main types
pub use kademlia::{KademliaNode, KademliaConfig};
pub use routing_table::{RoutingTable, RoutingEntry};
pub use content_store::{ContentStore, ContentId, DhtEntry};
pub use peer_manager::{PeerManager, PeerInfo, PeerStatus};
pub use network_topology::{NetworkTopology, TopologyInfo};
pub use fault_tolerance::{FaultTolerance, FailureDetector};
pub use performance::{PerformanceMonitor, Metrics};

/// 160-bit Node ID derived from BLAKE3 hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId([u8; 20]);

impl NodeId {
    /// Create new NodeId from bytes
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }
    
    /// Create NodeId from BLAKE3 hash
    pub fn from_hash(hash: blake3::Hash) -> Self {
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&hash.as_bytes()[..20]);
        Self(bytes)
    }
    
    /// Generate random NodeId (for testing)
    pub fn random() -> Self {
        let mut bytes = [0u8; 20];
        getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");
        Self(bytes)
    }
    
    /// Calculate XOR distance to another NodeId
    pub fn distance(&self, other: &NodeId) -> Distance {
        let mut result = [0u8; 20];
        for i in 0..20 {
            result[i] = self.0[i] ^ other.0[i];
        }
        Distance(result)
    }
    
    /// Get bytes representation
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
    
    /// Convert to hex string for display
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

/// XOR distance between two NodeIds
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Distance([u8; 20]);

impl Distance {
    /// Get the bit position of the most significant bit
    pub fn leading_zeros(&self) -> u32 {
        for (i, byte) in self.0.iter().enumerate() {
            if *byte != 0 {
                return (i * 8) as u32 + byte.leading_zeros();
            }
        }
        160 // All zeros
    }
    
    /// Get bucket index for this distance (0-159)
    pub fn bucket_index(&self) -> usize {
        let zeros = self.leading_zeros();
        if zeros >= 160 {
            159 // Edge case: distance is 0
        } else {
            (159 - zeros) as usize
        }
    }
}

/// DHT configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtConfig {
    /// Replication factor (K in Kademlia)
    pub replication_factor: usize,
    /// Maximum entries per routing table bucket
    pub bucket_size: usize,
    /// Lookup parallelism factor (α in Kademlia)
    pub lookup_parallelism: usize,
    /// Bootstrap nodes for initial connection
    pub bootstrap_nodes: Vec<SocketAddr>,
    /// Network timeouts
    pub timeouts: TimeoutConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
    /// Mobile optimizations
    pub mobile: MobileConfig,
    /// Security settings
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub ping_timeout: Duration,
    pub lookup_timeout: Duration,
    pub store_timeout: Duration,
    pub bootstrap_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_requests: usize,
    pub connection_pool_size: usize,
    pub cache_size: usize,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileConfig {
    pub power_aware_mode: bool,
    pub adaptive_polling: bool,
    pub background_mode_timeout: Duration,
    pub battery_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub max_peers_per_bucket: usize,
    pub reputation_threshold: f64,
    pub rate_limit_per_second: u32,
    pub max_content_size: usize,
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            replication_factor: 8, // K=8 for high availability
            bucket_size: 20,       // Standard Kademlia bucket size
            lookup_parallelism: 3, // α=3 for efficient lookups
            bootstrap_nodes: vec![],
            timeouts: TimeoutConfig {
                ping_timeout: Duration::from_secs(5),
                lookup_timeout: Duration::from_secs(30),
                store_timeout: Duration::from_secs(10),
                bootstrap_timeout: Duration::from_secs(60),
            },
            performance: PerformanceConfig {
                max_concurrent_requests: 100,
                connection_pool_size: 50,
                cache_size: 1000,
                batch_size: 10,
            },
            mobile: MobileConfig {
                power_aware_mode: true,
                adaptive_polling: true,
                background_mode_timeout: Duration::from_secs(300),
                battery_threshold: 0.2,
            },
            security: SecurityConfig {
                max_peers_per_bucket: 25,
                reputation_threshold: 0.5,
                rate_limit_per_second: 100,
                max_content_size: 1024 * 1024, // 1MB
            },
        }
    }
}

/// Main DHT interface
pub struct Dht {
    /// Our node ID
    node_id: NodeId,
    /// Configuration
    config: DhtConfig,
    /// Core Kademlia implementation
    kademlia: Arc<RwLock<KademliaNode>>,
    /// Routing table
    routing_table: Arc<RwLock<RoutingTable>>,
    /// Content storage
    content_store: Arc<RwLock<ContentStore>>,
    /// Peer management
    peer_manager: Arc<PeerManager>,
    /// Network topology monitoring
    network_topology: Arc<NetworkTopology>,
    /// Fault tolerance system
    fault_tolerance: Arc<FaultTolerance>,
    /// Performance monitoring
    performance_monitor: Arc<PerformanceMonitor>,
}

impl Dht {
    /// Create new DHT instance
    #[instrument(skip(config))]
    pub async fn new(node_id: NodeId, config: DhtConfig) -> Result<Self> {
        info!("Initializing DHT with node ID: {}", node_id.to_hex());
        
        // Initialize components
        let routing_table = Arc::new(RwLock::new(RoutingTable::new(node_id.clone())));
        let content_store = Arc::new(RwLock::new(ContentStore::new(config.performance.cache_size)));
        let peer_manager = Arc::new(PeerManager::new(config.clone()).await?);
        let network_topology = Arc::new(NetworkTopology::new(node_id.clone()));
        let fault_tolerance = Arc::new(FaultTolerance::new(config.clone()));
        let performance_monitor = Arc::new(PerformanceMonitor::new());
        
        let kademlia = Arc::new(RwLock::new(
            KademliaNode::new(
                node_id.clone(),
                config.clone(),
                routing_table.clone(),
                content_store.clone(),
                peer_manager.clone(),
            ).await?
        ));
        
        Ok(Self {
            node_id,
            config,
            kademlia,
            routing_table,
            content_store,
            peer_manager,
            network_topology,
            fault_tolerance,
            performance_monitor,
        })
    }
    
    /// Start the DHT (connect to bootstrap nodes, begin maintenance)
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting DHT...");
        
        // Start background tasks
        self.peer_manager.start().await?;
        self.network_topology.start().await?;
        self.fault_tolerance.start().await?;
        self.performance_monitor.start().await?;
        
        // Connect to bootstrap nodes
        self.bootstrap().await?;
        
        info!("DHT started successfully");
        Ok(())
    }
    
    /// Bootstrap DHT by connecting to known nodes
    #[instrument(skip(self))]
    pub async fn bootstrap(&self) -> Result<()> {
        if self.config.bootstrap_nodes.is_empty() {
            warn!("No bootstrap nodes configured");
            return Ok(());
        }
        
        info!("Bootstrapping DHT with {} nodes", self.config.bootstrap_nodes.len());
        
        let mut connected = 0;
        for addr in &self.config.bootstrap_nodes {
            match self.peer_manager.connect_to_peer(*addr).await {
                Ok(peer_id) => {
                    info!("Connected to bootstrap node: {} -> {}", addr, peer_id.to_hex());
                    connected += 1;
                },
                Err(e) => {
                    warn!("Failed to connect to bootstrap node {}: {}", addr, e);
                }
            }
        }
        
        if connected == 0 {
            return Err(anyhow::anyhow!("Failed to connect to any bootstrap nodes"));
        }
        
        // Perform initial node lookup to populate routing table
        let _ = self.find_node(self.node_id.clone()).await?;
        
        info!("DHT bootstrap completed, connected to {} nodes", connected);
        Ok(())
    }
    
    /// Store content in the DHT
    #[instrument(skip(self, value))]
    pub async fn store(&self, key: ContentId, value: Vec<u8>) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        debug!("Storing content with key: {}", key.to_hex());
        
        let result = self.kademlia.read().await
            .store(key.clone(), value).await;
            
        self.performance_monitor.record_store_latency(start_time.elapsed()).await;
        
        match result {
            Ok(_) => {
                info!("Successfully stored content: {}", key.to_hex());
                Ok(())
            },
            Err(e) => {
                error!("Failed to store content {}: {}", key.to_hex(), e);
                Err(e)
            }
        }
    }
    
    /// Retrieve content from the DHT
    #[instrument(skip(self))]
    pub async fn get(&self, key: ContentId) -> Result<Option<Vec<u8>>> {
        let start_time = std::time::Instant::now();
        
        debug!("Retrieving content with key: {}", key.to_hex());
        
        let result = self.kademlia.read().await
            .find_value(key.clone()).await;
            
        self.performance_monitor.record_get_latency(start_time.elapsed()).await;
        
        match result {
            Ok(value) => {
                if value.is_some() {
                    info!("Successfully retrieved content: {}", key.to_hex());
                } else {
                    debug!("Content not found: {}", key.to_hex());
                }
                Ok(value)
            },
            Err(e) => {
                error!("Failed to retrieve content {}: {}", key.to_hex(), e);
                Err(e)
            }
        }
    }
    
    /// Find the closest nodes to a given node ID
    #[instrument(skip(self))]
    pub async fn find_node(&self, target: NodeId) -> Result<Vec<NodeId>> {
        let start_time = std::time::Instant::now();
        
        debug!("Finding nodes closest to: {}", target.to_hex());
        
        let result = self.kademlia.read().await
            .find_node(target.clone()).await;
            
        self.performance_monitor.record_lookup_latency(start_time.elapsed()).await;
        
        match result {
            Ok(nodes) => {
                info!("Found {} nodes closest to {}", nodes.len(), target.to_hex());
                Ok(nodes)
            },
            Err(e) => {
                error!("Failed to find nodes for {}: {}", target.to_hex(), e);
                Err(e)
            }
        }
    }
    
    /// Get DHT status information
    pub async fn status(&self) -> DhtStatus {
        let routing_table = self.routing_table.read().await;
        let content_store = self.content_store.read().await;
        let network_info = self.network_topology.status().await;
        let performance_metrics = self.performance_monitor.metrics().await;
        
        DhtStatus {
            node_id: self.node_id.clone(),
            peer_count: routing_table.peer_count(),
            stored_items: content_store.item_count(),
            network_health: network_info.health_score,
            uptime: network_info.uptime,
            performance: performance_metrics,
        }
    }
    
    /// Shutdown the DHT gracefully
    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down DHT...");
        
        // Stop background tasks
        self.fault_tolerance.stop().await?;
        self.network_topology.stop().await?;
        self.peer_manager.stop().await?;
        self.performance_monitor.stop().await?;
        
        info!("DHT shutdown completed");
        Ok(())
    }
}

/// DHT status information for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtStatus {
    pub node_id: NodeId,
    pub peer_count: usize,
    pub stored_items: usize,
    pub network_health: f64,
    pub uptime: Duration,
    pub performance: Metrics,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_id_distance() {
        let id1 = NodeId::from_bytes([0; 20]);
        let id2 = NodeId::from_bytes([255; 20]);
        let distance = id1.distance(&id2);
        assert_eq!(distance.leading_zeros(), 0);
    }
    
    #[test]
    fn test_distance_bucket_index() {
        let mut bytes = [0u8; 20];
        bytes[0] = 0b10000000; // MSB set
        let distance = Distance(bytes);
        assert_eq!(distance.bucket_index(), 159);
        
        bytes[0] = 0b01000000; // Second bit set
        let distance = Distance(bytes);
        assert_eq!(distance.bucket_index(), 158);
    }
    
    #[tokio::test]
    async fn test_dht_creation() {
        let node_id = NodeId::random();
        let config = DhtConfig::default();
        let dht = Dht::new(node_id, config).await;
        assert!(dht.is_ok());
    }
}
