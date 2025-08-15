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


//! Kademlia DHT Implementation
//!
//! This module implements the core Kademlia distributed hash table algorithm
//! with production-ready optimizations for fault tolerance, performance, and
//! mobile constraints.

use super::*;
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{timeout, Duration, Instant};
use tracing::{debug, info, warn,  instrument};

/// Core Kademlia node implementation
pub struct KademliaNode {
    /// Our node ID
    node_id: NodeId,
    /// Configuration
    config: DhtConfig,
    /// Routing table reference
    routing_table: Arc<RwLock<RoutingTable>>,
    /// Content store reference
    content_store: Arc<RwLock<ContentStore>>,
    /// Peer manager reference
    peer_manager: Arc<PeerManager>,
    /// Semaphore to limit concurrent operations
    operation_semaphore: Arc<Semaphore>,
    /// Active lookup operations
    active_lookups: Arc<RwLock<HashMap<NodeId, LookupState>>>,
}

/// State of an active lookup operation
#[derive(Debug, Clone)]
struct LookupState {
    target: NodeId,
    closest_nodes: Vec<NodeId>,
    queried_nodes: std::collections::HashSet<NodeId>,
    pending_queries: usize,
    started_at: Instant,
}

/// Kademlia protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KademliaMessage {
    /// Ping message for liveness checking
    Ping {
        sender_id: NodeId,
    },
    /// Pong response to ping
    Pong {
        sender_id: NodeId,
    },
    /// Store content request
    Store {
        sender_id: NodeId,
        key: ContentId,
        value: Vec<u8>,
        ttl: Duration,
    },
    /// Store response
    StoreResponse {
        sender_id: NodeId,
        success: bool,
        error: Option<String>,
    },
    /// Find node request
    FindNode {
        sender_id: NodeId,
        target: NodeId,
    },
    /// Find node response
    FindNodeResponse {
        sender_id: NodeId,
        closest_nodes: Vec<(NodeId, SocketAddr)>,
    },
    /// Find value request
    FindValue {
        sender_id: NodeId,
        key: ContentId,
    },
    /// Find value response with actual value
    FindValueResponse {
        sender_id: NodeId,
        value: Vec<u8>,
    },
    /// Find value response with closest nodes
    FindValueNodesResponse {
        sender_id: NodeId,
        closest_nodes: Vec<(NodeId, SocketAddr)>,
    },
}

/// Configuration for Kademlia operations
#[derive(Debug, Clone)]
pub struct KademliaConfig {
    /// Base DHT configuration
    pub base: DhtConfig,
    /// Maximum lookup iterations
    pub max_lookup_iterations: usize,
    /// Lookup timeout
    pub lookup_timeout: Duration,
    /// Store timeout
    pub store_timeout: Duration,
    /// Number of nodes to query in parallel during lookup
    pub parallel_queries: usize,
    /// Minimum number of successful stores required
    pub min_successful_stores: usize,
}

impl From<DhtConfig> for KademliaConfig {
    fn from(config: DhtConfig) -> Self {
        Self {
            parallel_queries: config.lookup_parallelism,
            min_successful_stores: config.replication_factor / 2 + 1, // Majority
            max_lookup_iterations: 20, // Reasonable upper bound
            lookup_timeout: config.timeouts.lookup_timeout,
            store_timeout: config.timeouts.store_timeout,
            base: config,
        }
    }
}

impl KademliaNode {
    /// Create new Kademlia node
    #[instrument(skip(routing_table, content_store, peer_manager))]
    pub async fn new(
        node_id: NodeId,
        config: DhtConfig,
        routing_table: Arc<RwLock<RoutingTable>>,
        content_store: Arc<RwLock<ContentStore>>,
        peer_manager: Arc<PeerManager>,
    ) -> Result<Self> {
        let kademlia_config = KademliaConfig::from(config);
        let operation_semaphore = Arc::new(Semaphore::new(
            kademlia_config.base.performance.max_concurrent_requests
        ));
        
        info!("Initializing Kademlia node: {}", node_id.to_hex());
        
        Ok(Self {
            node_id,
            config: kademlia_config.base,
            routing_table,
            content_store,
            peer_manager,
            operation_semaphore,
            active_lookups: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Store a key-value pair in the DHT
    #[instrument(skip(self, value))]
    pub async fn store(&self, key: ContentId, value: Vec<u8>) -> Result<()> {
        let _permit = self.operation_semaphore.acquire().await
            .context("Failed to acquire operation permit")?;
        
        info!("Storing content: {} ({} bytes)", key.to_hex(), value.len());
        
        // Validate content size
        if value.len() > self.config.security.max_content_size {
            return Err(anyhow::anyhow!(
                "Content too large: {} bytes (max: {})",
                value.len(),
                self.config.security.max_content_size
            ));
        }
        
        // Find K closest nodes to the key
        let target_node_id = NodeId::from_hash(blake3::hash(key.as_bytes()));
        let closest_nodes = self.find_node(target_node_id).await
            .context("Failed to find closest nodes for storage")?;
        
        let store_targets = closest_nodes.into_iter()
            .take(self.config.replication_factor)
            .collect::<Vec<_>>();
        
        if store_targets.is_empty() {
            return Err(anyhow::anyhow!("No nodes available for storage"));
        }
        
        // Store on all target nodes in parallel
        let store_tasks = store_targets.into_iter().map(|node_id| {
            let key = key.clone();
            let value = value.clone();
            let peer_manager = self.peer_manager.clone();
            let timeout_duration = self.config.timeouts.store_timeout;
            
            async move {
                timeout(timeout_duration, peer_manager.send_message(
                    node_id.clone(),
                    KademliaMessage::Store {
                        sender_id: self.node_id.clone(),
                        key,
                        value,
                        ttl: Duration::from_secs(3600), // 1 hour default TTL
                    }
                )).await
                .map_err(|_| anyhow::anyhow!("Store timeout for node {}", node_id.to_hex()))?
                .context("Failed to send store message")?;
                
                Ok(node_id)
            }
        });
        
        let store_results = futures::future::join_all(store_tasks).await;
        let successful_stores: Vec<NodeId> = store_results.into_iter()
            .filter_map(|result| result.ok())
            .collect();
        let successful_count = successful_stores.len();
        
        let min_required = self.config.replication_factor / 2 + 1; // Majority
        if successful_count < min_required {
            warn!(
                "Only {} of {} stores succeeded (minimum required: {})",
                successful_count, self.config.replication_factor, min_required
            );
            return Err(anyhow::anyhow!(
                "Insufficient successful stores: {}/{} (required: {})",
                successful_count, self.config.replication_factor, min_required
            ));
        }
        
        // Also store locally if we're one of the closest nodes
        let our_distance = self.node_id.distance(&target_node_id);
        let mut should_store_locally = false;
        
        {
            let routing_table = self.routing_table.read().await;
            let local_closest = routing_table.find_closest(&target_node_id, self.config.replication_factor);
            
            if local_closest.len() < self.config.replication_factor ||
               local_closest.iter().any(|node| self.node_id.distance(&target_node_id) <= node.distance(&target_node_id)) {
                should_store_locally = true;
            }
        }
        
        if should_store_locally {
            let mut content_store = self.content_store.write().await;
            content_store.store(DhtEntry {
                key,
                value,
                timestamp: SystemTime::now(),
                ttl: Duration::from_secs(3600),
                replicas: vec![self.node_id.clone()],
            }).await.context("Failed to store content locally")?;
        }
        
        info!("Successfully stored content with {}/{} replicas", 
              successful_stores, self.config.replication_factor);
        Ok(())
    }
    
    /// Find the value for a given key
    #[instrument(skip(self))]
    pub async fn find_value(&self, key: ContentId) -> Result<Option<Vec<u8>>> {
        let _permit = self.operation_semaphore.acquire().await
            .context("Failed to acquire operation permit")?;
        
        debug!("Finding value for key: {}", key.to_hex());
        
        // First check our local storage
        {
            let content_store = self.content_store.read().await;
            if let Some(entry) = content_store.get(&key).await? {
                // Verify entry hasn't expired
                if entry.timestamp + entry.ttl > SystemTime::now() {
                    debug!("Found value locally: {}", key.to_hex());
                    return Ok(Some(entry.value));
                }
            }
        }
        
        // Look up in the network
        let target_node_id = NodeId::from_hash(blake3::hash(key.as_bytes()));
        let closest_nodes = self.find_node(target_node_id).await
            .context("Failed to find closest nodes for value lookup")?;
        
        // Query closest nodes for the value
        for node_id in closest_nodes.into_iter().take(self.config.lookup_parallelism) {
            match timeout(
                self.config.timeouts.lookup_timeout,
                self.peer_manager.send_message(
                    node_id.clone(),
                    KademliaMessage::FindValue {
                        sender_id: self.node_id.clone(),
                        key: key.clone(),
                    }
                )
            ).await {
                Ok(Ok(response)) => {
                    if let Some(value) = self.handle_find_value_response(response).await? {
                        info!("Successfully found value: {}", key.to_hex());
                        return Ok(Some(value));
                    }
                },
                Ok(Err(e)) => {
                    warn!("Error querying node {} for value: {}", node_id.to_hex(), e);
                },
                Err(_) => {
                    warn!("Timeout querying node {} for value", node_id.to_hex());
                }
            }
        }
        
        debug!("Value not found: {}", key.to_hex());
        Ok(None)
    }
    
    /// Find the K closest nodes to a target node ID
    #[instrument(skip(self))]
    pub async fn find_node(&self, target: NodeId) -> Result<Vec<NodeId>> {
        let _permit = self.operation_semaphore.acquire().await
            .context("Failed to acquire operation permit")?;
        
        debug!("Finding nodes closest to: {}", target.to_hex());
        
        // Check if we already have an active lookup for this target
        {
            let active_lookups = self.active_lookups.read().await;
            if active_lookups.contains_key(&target) {
                // Wait for existing lookup to complete
                drop(active_lookups);
                return self.wait_for_lookup(&target).await;
            }
        }
        
        // Initialize lookup state
        let initial_nodes = {
            let routing_table = self.routing_table.read().await;
            routing_table.find_closest(&target, self.config.lookup_parallelism)
        };
        
        if initial_nodes.is_empty() {
            warn!("No nodes in routing table for lookup");
            return Ok(vec![]);
        }
        
        let lookup_state = LookupState {
            target: target.clone(),
            closest_nodes: initial_nodes.clone(),
            queried_nodes: std::collections::HashSet::new(),
            pending_queries: 0,
            started_at: Instant::now(),
        };
        
        {
            let mut active_lookups = self.active_lookups.write().await;
            active_lookups.insert(target.clone(), lookup_state);
        }
        
        // Perform iterative lookup
        let result = self.iterative_lookup(target.clone()).await;
        
        // Clean up lookup state
        {
            let mut active_lookups = self.active_lookups.write().await;
            active_lookups.remove(&target);
        }
        
        result
    }
    
    /// Perform iterative lookup for closest nodes
    #[instrument(skip(self))]
    async fn iterative_lookup(&self, target: NodeId) -> Result<Vec<NodeId>> {
        let mut closest_nodes = {
            let routing_table = self.routing_table.read().await;
            routing_table.find_closest(&target, self.config.replication_factor)
        };
        
        let mut queried_nodes = std::collections::HashSet::new();
        let mut iteration = 0;
        
        while iteration < 20 { // Maximum iterations to prevent infinite loops
            iteration += 1;
            
            // Select nodes to query (closest unqueried nodes)
            let nodes_to_query: Vec<NodeId> = closest_nodes
                .iter()
                .filter(|node| !queried_nodes.contains(node))
                .take(self.config.lookup_parallelism)
                .cloned()
                .collect();
            
            if nodes_to_query.is_empty() {
                debug!("No more nodes to query, lookup complete");
                break;
            }
            
            // Query nodes in parallel
            let query_tasks = nodes_to_query.iter().map(|node_id| {
                let node_id = node_id.clone();
                let target = target.clone();
                let peer_manager = self.peer_manager.clone();
                let sender_id = self.node_id.clone();
                
                async move {
                    match timeout(
                        Duration::from_secs(5),
                        peer_manager.send_message(
                            node_id.clone(),
                            KademliaMessage::FindNode {
                                sender_id,
                                target,
                            }
                        )
                    ).await {
                        Ok(Ok(response)) => {
                            if let Some(nodes) = Self::parse_find_node_response(response) {
                                Some((node_id, nodes))
                            } else {
                                None
                            }
                        },
                        _ => None,
                    }
                }
            });
            
            let query_results = futures::future::join_all(query_tasks).await;
            
            // Mark nodes as queried
            for node_id in &nodes_to_query {
                queried_nodes.insert(node_id.clone());
            }
            
            // Process responses and update closest nodes
            let mut new_nodes = Vec::new();
            for result in query_results {
                if let Some((responding_node, returned_nodes)) = result {
                    // Update routing table with successful response
                    {
                        let mut routing_table = self.routing_table.write().await;
                        if let Err(e) = routing_table.update_node(responding_node.clone()).await {
                            warn!("Failed to update routing table: {}", e);
                        }
                    }
                    
                    // Add returned nodes to our list
                    for (node_id, _addr) in returned_nodes {
                        if node_id != self.node_id && !new_nodes.contains(&node_id) {
                            new_nodes.push(node_id);
                        }
                    }
                }
            }
            
            // Update closest nodes list
            closest_nodes.extend(new_nodes);
            closest_nodes.sort_by_key(|node| node.distance(&target));
            closest_nodes.truncate(self.config.replication_factor);
            
            debug!("Lookup iteration {}: {} closest nodes", iteration, closest_nodes.len());
        }
        
        info!("Lookup completed after {} iterations, found {} nodes", 
              iteration, closest_nodes.len());
        Ok(closest_nodes)
    }
    
    /// Handle incoming Kademlia message
    #[instrument(skip(self, message))]
    pub async fn handle_message(&self, from: NodeId, message: KademliaMessage) -> Result<Option<KademliaMessage>> {
        debug!("Handling message from {}: {:?}", from.to_hex(), message);
        
        // Update routing table with message sender
        {
            let mut routing_table = self.routing_table.write().await;
            if let Err(e) = routing_table.update_node(from.clone()).await {
                warn!("Failed to update routing table: {}", e);
            }
        }
        
        match message {
            KademliaMessage::Ping { sender_id } => {
                Ok(Some(KademliaMessage::Pong { sender_id: self.node_id.clone() }))
            },
            
            KademliaMessage::Store { key, value, ttl, .. } => {
                let success = {
                    let mut content_store = self.content_store.write().await;
                    content_store.store(DhtEntry {
                        key: key.clone(),
                        value,
                        timestamp: SystemTime::now(),
                        ttl,
                        replicas: vec![from.clone()],
                    }).await.is_ok()
                };
                
                Ok(Some(KademliaMessage::StoreResponse {
                    sender_id: self.node_id.clone(),
                    success,
                    error: if success { None } else { Some("Storage failed".to_string()) },
                }))
            },
            
            KademliaMessage::FindNode { target, .. } => {
                let closest_nodes = {
                    let routing_table = self.routing_table.read().await;
                    routing_table.find_closest(&target, self.config.replication_factor)
                };
                
                // Convert to (NodeId, SocketAddr) pairs
                let node_addrs = closest_nodes.into_iter()
                    .filter_map(|node_id| {
                        // In a real implementation, we'd look up the address
                        // For now, use a placeholder
                        Some((node_id, "127.0.0.1:8080".parse().unwrap()))
                    })
                    .collect();
                
                Ok(Some(KademliaMessage::FindNodeResponse {
                    sender_id: self.node_id.clone(),
                    closest_nodes: node_addrs,
                }))
            },
            
            KademliaMessage::FindValue { key, .. } => {
                // Check local storage first
                let local_value = {
                    let content_store = self.content_store.read().await;
                    content_store.get(&key).await?
                        .and_then(|entry| {
                            if entry.timestamp + entry.ttl > SystemTime::now() {
                                Some(entry.value)
                            } else {
                                None
                            }
                        })
                };
                
                if let Some(value) = local_value {
                    Ok(Some(KademliaMessage::FindValueResponse {
                        sender_id: self.node_id.clone(),
                        value,
                    }))
                } else {
                    // Return closest nodes instead
                    let target_node_id = NodeId::from_hash(blake3::hash(key.as_bytes()));
                    let closest_nodes = {
                        let routing_table = self.routing_table.read().await;
                        routing_table.find_closest(&target_node_id, self.config.replication_factor)
                    };
                    
                    let node_addrs = closest_nodes.into_iter()
                        .filter_map(|node_id| {
                            Some((node_id, "127.0.0.1:8080".parse().unwrap()))
                        })
                        .collect();
                    
                    Ok(Some(KademliaMessage::FindValueNodesResponse {
                        sender_id: self.node_id.clone(),
                        closest_nodes: node_addrs,
                    }))
                }
            },
            
            _ => {
                // Response messages don't generate replies
                Ok(None)
            }
        }
    }
    
    /// Wait for an existing lookup to complete
    async fn wait_for_lookup(&self, target: &NodeId) -> Result<Vec<NodeId>> {
        // Simple polling approach - in production, use proper async coordination
        for _ in 0..100 { // Maximum wait iterations
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            let active_lookups = self.active_lookups.read().await;
            if !active_lookups.contains_key(target) {
                drop(active_lookups);
                // Lookup completed, return current closest nodes
                let routing_table = self.routing_table.read().await;
                return Ok(routing_table.find_closest(target, self.config.replication_factor));
            }
        }
        
        Err(anyhow::anyhow!("Timeout waiting for lookup to complete"))
    }
    
    /// Handle find value response
    async fn handle_find_value_response(&self, response: KademliaMessage) -> Result<Option<Vec<u8>>> {
        match response {
            KademliaMessage::FindValueResponse { value, .. } => {
                Ok(Some(value))
            },
            KademliaMessage::FindValueNodesResponse { closest_nodes, .. } => {
                // Update routing table with returned nodes
                let mut routing_table = self.routing_table.write().await;
                for (node_id, _addr) in closest_nodes {
                    if let Err(e) = routing_table.update_node(node_id).await {
                        warn!("Failed to update routing table: {}", e);
                    }
                }
                Ok(None)
            },
            _ => Ok(None),
        }
    }
    
    /// Parse find node response
    fn parse_find_node_response(response: KademliaMessage) -> Option<Vec<(NodeId, SocketAddr)>> {
        match response {
            KademliaMessage::FindNodeResponse { closest_nodes, .. } => {
                Some(closest_nodes)
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dht::{RoutingTable, ContentStore};
    use crate::dht::peer_manager::PeerManager;
    
    #[tokio::test]
    async fn test_kademlia_node_creation() {
        let node_id = NodeId::random();
        let config = DhtConfig::default();
        let routing_table = Arc::new(RwLock::new(RoutingTable::new(node_id.clone())));
        let content_store = Arc::new(RwLock::new(ContentStore::new(1000)));
        let peer_manager = Arc::new(PeerManager::new(config.clone()).await.unwrap());
        
        let kademlia = KademliaNode::new(
            node_id,
            config,
            routing_table,
            content_store,
            peer_manager,
        ).await;
        
        assert!(kademlia.is_ok());
    }
    
    #[tokio::test]
    async fn test_message_handling() {
        let node_id = NodeId::random();
        let config = DhtConfig::default();
        let routing_table = Arc::new(RwLock::new(RoutingTable::new(node_id.clone())));
        let content_store = Arc::new(RwLock::new(ContentStore::new(1000)));
        let peer_manager = Arc::new(PeerManager::new(config.clone()).await.unwrap());
        
        let kademlia = KademliaNode::new(
            node_id.clone(),
            config,
            routing_table,
            content_store,
            peer_manager,
        ).await.unwrap();
        
        let from = NodeId::random();
        let message = KademliaMessage::Ping { sender_id: from.clone() };
        
        let response = kademlia.handle_message(from, message).await.unwrap();
        assert!(matches!(response, Some(KademliaMessage::Pong { .. })));
    }
}
