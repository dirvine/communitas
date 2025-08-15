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


//! Network Topology Monitoring
//!
//! This module monitors the health and structure of the DHT network with:
//! - Real-time topology analysis
//! - Network partition detection
//! - Performance metrics collection
//! - Health scoring and alerts

use super::*;
use anyhow::{Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{debug, info, warn, error, instrument};

/// Network topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyInfo {
    /// Our position in the network
    pub our_node_id: NodeId,
    /// Total known nodes
    pub total_nodes: usize,
    /// Connected nodes
    pub connected_nodes: usize,
    /// Network diameter (maximum distance between any two nodes)
    pub network_diameter: usize,
    /// Clustering coefficient (how interconnected neighbors are)
    pub clustering_coefficient: f64,
    /// Average path length
    pub average_path_length: f64,
    /// Health score (0.0 to 1.0)
    pub health_score: f64,
    /// Network uptime
    pub uptime: Duration,
    /// Detected partitions
    pub partitions: Vec<NetworkPartition>,
    /// Performance metrics
    pub performance: NetworkPerformance,
}

/// Information about a network partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPartition {
    /// Partition ID
    pub id: String,
    /// Nodes in this partition
    pub nodes: Vec<NodeId>,
    /// When this partition was detected
    pub detected_at: SystemTime,
    /// Whether this partition is isolated
    pub is_isolated: bool,
    /// Bridge nodes connecting to other partitions
    pub bridge_nodes: Vec<NodeId>,
}

/// Network performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPerformance {
    /// Average lookup latency
    pub avg_lookup_latency: Duration,
    /// Average store latency
    pub avg_store_latency: Duration,
    /// Success rate for operations (0.0 to 1.0)
    pub operation_success_rate: f64,
    /// Network throughput (operations per second)
    pub throughput: f64,
    /// Bandwidth utilization
    pub bandwidth_utilization: f64,
    /// Memory usage
    pub memory_usage: u64,
}

/// Node connectivity information
#[derive(Debug, Clone)]
struct NodeConnectivity {
    node_id: NodeId,
    direct_connections: HashSet<NodeId>,
    reachable_nodes: HashSet<NodeId>,
    last_seen: SystemTime,
    latency: Duration,
    reliability: f64,
}

/// Network event for monitoring
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Node joined the network
    NodeJoined {
        node_id: NodeId,
        timestamp: SystemTime,
    },
    /// Node left the network
    NodeLeft {
        node_id: NodeId,
        timestamp: SystemTime,
        reason: String,
    },
    /// Connection established
    ConnectionEstablished {
        from: NodeId,
        to: NodeId,
        timestamp: SystemTime,
        latency: Duration,
    },
    /// Connection lost
    ConnectionLost {
        from: NodeId,
        to: NodeId,
        timestamp: SystemTime,
        reason: String,
    },
    /// Partition detected
    PartitionDetected {
        partition: NetworkPartition,
        timestamp: SystemTime,
    },
    /// Partition healed
    PartitionHealed {
        partition_id: String,
        timestamp: SystemTime,
    },
    /// Performance degradation
    PerformanceDegradation {
        metric: String,
        old_value: f64,
        new_value: f64,
        timestamp: SystemTime,
    },
}

/// Main network topology monitor
pub struct NetworkTopology {
    /// Our node ID
    our_node_id: NodeId,
    /// Node connectivity map
    connectivity: Arc<RwLock<HashMap<NodeId, NodeConnectivity>>>,
    /// Current topology information
    topology_info: Arc<RwLock<TopologyInfo>>,
    /// Event history
    event_history: Arc<RwLock<VecDeque<NetworkEvent>>>,
    /// Configuration
    config: TopologyConfig,
    /// Start time for uptime calculation
    start_time: Instant,
    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// Configuration for topology monitoring
#[derive(Debug, Clone)]
pub struct TopologyConfig {
    /// How often to analyze topology
    pub analysis_interval: Duration,
    /// Maximum event history size
    pub max_event_history: usize,
    /// Threshold for partition detection
    pub partition_threshold: f64,
    /// Minimum health score threshold
    pub min_health_score: f64,
    /// Performance monitoring window
    pub performance_window: Duration,
    /// Node timeout threshold
    pub node_timeout: Duration,
}

impl Default for TopologyConfig {
    fn default() -> Self {
        Self {
            analysis_interval: Duration::from_secs(30),
            max_event_history: 1000,
            partition_threshold: 0.3,
            min_health_score: 0.5,
            performance_window: Duration::from_secs(300), // 5 minutes
            node_timeout: Duration::from_secs(120), // 2 minutes
        }
    }
}

impl NetworkTopology {
    /// Create new network topology monitor
    #[instrument(skip(our_node_id))]
    pub fn new(our_node_id: NodeId) -> Self {
        info!("Creating network topology monitor for node: {}", our_node_id.to_hex());
        
        let start_time = Instant::now();
        let topology_info = TopologyInfo {
            our_node_id: our_node_id.clone(),
            total_nodes: 0,
            connected_nodes: 0,
            network_diameter: 0,
            clustering_coefficient: 0.0,
            average_path_length: 0.0,
            health_score: 1.0,
            uptime: Duration::from_secs(0),
            partitions: Vec::new(),
            performance: NetworkPerformance {
                avg_lookup_latency: Duration::from_millis(100),
                avg_store_latency: Duration::from_millis(50),
                operation_success_rate: 1.0,
                throughput: 0.0,
                bandwidth_utilization: 0.0,
                memory_usage: 0,
            },
        };
        
        Self {
            our_node_id,
            connectivity: Arc::new(RwLock::new(HashMap::new())),
            topology_info: Arc::new(RwLock::new(topology_info)),
            event_history: Arc::new(RwLock::new(VecDeque::new())),
            config: TopologyConfig::default(),
            start_time,
            shutdown_tx: None,
        }
    }
    
    /// Start topology monitoring
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting network topology monitoring");
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        
        // Start analysis task
        let connectivity = self.connectivity.clone();
        let topology_info = self.topology_info.clone();
        let event_history = self.event_history.clone();
        let config = self.config.clone();
        let our_node_id = self.our_node_id.clone();
        let start_time = self.start_time;
        
        tokio::spawn(async move {
            let mut interval = interval(config.analysis_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::analyze_topology(
                            &connectivity,
                            &topology_info,
                            &event_history,
                            &config,
                            &our_node_id,
                            start_time,
                        ).await {
                            error!("Topology analysis error: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Network topology monitoring shutting down");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop topology monitoring
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping network topology monitoring");
        
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(()).await;
        }
        
        Ok(())
    }
    
    /// Record node joining the network
    #[instrument(skip(self))]
    pub async fn node_joined(&self, node_id: NodeId) -> Result<()> {
        debug!("Node joined: {}", node_id.to_hex());
        
        let now = SystemTime::now();
        
        // Add to connectivity map
        {
            let mut connectivity = self.connectivity.write().await;
            connectivity.insert(node_id.clone(), NodeConnectivity {
                node_id: node_id.clone(),
                direct_connections: HashSet::new(),
                reachable_nodes: HashSet::new(),
                last_seen: now,
                latency: Duration::from_millis(100),
                reliability: 1.0,
            });
        }
        
        // Record event
        let event = NetworkEvent::NodeJoined {
            node_id: node_id.clone(),
            timestamp: now,
        };
        self.record_event(event).await;
        
        // Trigger immediate analysis
        self.trigger_analysis().await?;
        
        Ok(())
    }
    
    /// Record node leaving the network
    #[instrument(skip(self))]
    pub async fn node_left(&self, node_id: &NodeId, reason: String) -> Result<()> {
        debug!("Node left: {} (reason: {})", node_id.to_hex(), reason);
        
        let now = SystemTime::now();
        
        // Remove from connectivity map
        {
            let mut connectivity = self.connectivity.write().await;
            connectivity.remove(node_id);
            
            // Remove from other nodes' connection lists
            for (_, node_connectivity) in connectivity.iter_mut() {
                node_connectivity.direct_connections.remove(node_id);
                node_connectivity.reachable_nodes.remove(node_id);
            }
        }
        
        // Record event
        let event = NetworkEvent::NodeLeft {
            node_id: node_id.clone(),
            timestamp: now,
            reason,
        };
        self.record_event(event).await;
        
        // Trigger immediate analysis
        self.trigger_analysis().await?;
        
        Ok(())
    }
    
    /// Record connection establishment
    #[instrument(skip(self))]
    pub async fn connection_established(&self, from: NodeId, to: NodeId, latency: Duration) -> Result<()> {
        debug!("Connection established: {} -> {} (latency: {:?})", 
               from.to_hex(), to.to_hex(), latency);
        
        let now = SystemTime::now();
        
        // Update connectivity map
        {
            let mut connectivity = self.connectivity.write().await;
            
            if let Some(from_node) = connectivity.get_mut(&from) {
                from_node.direct_connections.insert(to.clone());
                from_node.reachable_nodes.insert(to.clone());
                from_node.last_seen = now;
                from_node.latency = latency;
            }
            
            if let Some(to_node) = connectivity.get_mut(&to) {
                to_node.direct_connections.insert(from.clone());
                to_node.reachable_nodes.insert(from.clone());
                to_node.last_seen = now;
            }
        }
        
        // Record event
        let event = NetworkEvent::ConnectionEstablished {
            from,
            to,
            timestamp: now,
            latency,
        };
        self.record_event(event).await;
        
        Ok(())
    }
    
    /// Record connection loss
    #[instrument(skip(self))]
    pub async fn connection_lost(&self, from: NodeId, to: NodeId, reason: String) -> Result<()> {
        debug!("Connection lost: {} -> {} (reason: {})", 
               from.to_hex(), to.to_hex(), reason);
        
        let now = SystemTime::now();
        
        // Update connectivity map
        {
            let mut connectivity = self.connectivity.write().await;
            
            if let Some(from_node) = connectivity.get_mut(&from) {
                from_node.direct_connections.remove(&to);
                // Keep in reachable_nodes - might be reachable through other paths
            }
            
            if let Some(to_node) = connectivity.get_mut(&to) {
                to_node.direct_connections.remove(&from);
            }
        }
        
        // Record event
        let event = NetworkEvent::ConnectionLost {
            from,
            to,
            timestamp: now,
            reason,
        };
        self.record_event(event).await;
        
        // Trigger analysis to check for partitions
        self.trigger_analysis().await?;
        
        Ok(())
    }
    
    /// Get current topology status
    pub async fn status(&self) -> TopologyInfo {
        let mut info = self.topology_info.read().await.clone();
        info.uptime = self.start_time.elapsed();
        info
    }
    
    /// Get recent network events
    pub async fn recent_events(&self, count: usize) -> Vec<NetworkEvent> {
        let events = self.event_history.read().await;
        events.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
    
    /// Trigger immediate topology analysis
    async fn trigger_analysis(&self) -> Result<()> {
        Self::analyze_topology(
            &self.connectivity,
            &self.topology_info,
            &self.event_history,
            &self.config,
            &self.our_node_id,
            self.start_time,
        ).await
    }
    
    /// Record network event
    async fn record_event(&self, event: NetworkEvent) {
        let mut history = self.event_history.write().await;
        history.push_back(event);
        
        // Maintain maximum history size
        while history.len() > self.config.max_event_history {
            history.pop_front();
        }
    }
    
    /// Main topology analysis task
    #[instrument(skip(connectivity, topology_info, event_history, _config))]
    async fn analyze_topology(
        connectivity: &Arc<RwLock<HashMap<NodeId, NodeConnectivity>>>,
        topology_info: &Arc<RwLock<TopologyInfo>>,
        event_history: &Arc<RwLock<VecDeque<NetworkEvent>>>,
        _config: &TopologyConfig,
        our_node_id: &NodeId,
        start_time: Instant,
    ) -> Result<()> {
        debug!("Analyzing network topology");
        
        let connectivity_map = connectivity.read().await;
        let mut info = topology_info.write().await;
        
        // Basic metrics
        info.total_nodes = connectivity_map.len();
        info.connected_nodes = connectivity_map.values()
            .filter(|node| node.last_seen + _config.node_timeout > SystemTime::now())
            .count();
        info.uptime = start_time.elapsed();
        
        // Calculate network diameter and average path length
        let (diameter, avg_path_length) = Self::calculate_graph_metrics(&connectivity_map);
        info.network_diameter = diameter;
        info.average_path_length = avg_path_length;
        
        // Calculate clustering coefficient
        info.clustering_coefficient = Self::calculate_clustering_coefficient(&connectivity_map);
        
        // Detect partitions
        let partitions = Self::detect_partitions(&connectivity_map, _config);
        let partition_detected = !partitions.is_empty() && info.partitions.is_empty();
        info.partitions = partitions;
        
        // Calculate health score
        info.health_score = Self::calculate_health_score(&info, &connectivity_map, _config);
        
        // Update performance metrics (simplified)
        info.performance.operation_success_rate = if info.connected_nodes > 0 {
            (info.connected_nodes as f64 / info.total_nodes as f64).min(1.0)
        } else {
            0.0
        };
        
        // Record partition detection event
        if partition_detected {
            let event = NetworkEvent::PartitionDetected {
                partition: info.partitions[0].clone(),
                timestamp: SystemTime::now(),
            };
            
            let mut history = event_history.write().await;
            history.push_back(event);
        }
        
        // Log health warnings
        if info.health_score < _config.min_health_score {
            warn!("Network health degraded: score={:.2}, connected={}/{}", 
                  info.health_score, info.connected_nodes, info.total_nodes);
        }
        
        debug!("Topology analysis complete: nodes={}, health={:.2}, partitions={}", 
               info.total_nodes, info.health_score, info.partitions.len());
        
        Ok(())
    }
    
    /// Calculate network diameter and average path length
    fn calculate_graph_metrics(connectivity: &HashMap<NodeId, NodeConnectivity>) -> (usize, f64) {
        if connectivity.is_empty() {
            return (0, 0.0);
        }
        
        // Simplified calculation - in a real implementation, use Floyd-Warshall or BFS
        let node_count = connectivity.len();
        let connection_count: usize = connectivity.values()
            .map(|node| node.direct_connections.len())
            .sum();
        
        let avg_connections = if node_count > 0 {
            connection_count as f64 / node_count as f64
        } else {
            0.0
        };
        
        // Estimate diameter based on network density
        let diameter = if avg_connections > 0.0 {
            ((node_count as f64).ln() / avg_connections.ln()).ceil() as usize
        } else {
            node_count
        };
        
        let avg_path_length = diameter as f64 * 0.6; // Rough approximation
        
        (diameter, avg_path_length)
    }
    
    /// Calculate clustering coefficient
    fn calculate_clustering_coefficient(connectivity: &HashMap<NodeId, NodeConnectivity>) -> f64 {
        if connectivity.len() < 3 {
            return 0.0;
        }
        
        let mut total_coefficient = 0.0;
        let mut node_count = 0;
        
        for node in connectivity.values() {
            if node.direct_connections.len() < 2 {
                continue;
            }
            
            let neighbors: Vec<&NodeId> = node.direct_connections.iter().collect();
            let possible_edges = neighbors.len() * (neighbors.len() - 1) / 2;
            
            if possible_edges == 0 {
                continue;
            }
            
            // Count actual edges between neighbors
            let mut actual_edges = 0;
            for i in 0..neighbors.len() {
                for j in (i + 1)..neighbors.len() {
                    if let Some(neighbor_node) = connectivity.get(neighbors[i]) {
                        if neighbor_node.direct_connections.contains(neighbors[j]) {
                            actual_edges += 1;
                        }
                    }
                }
            }
            
            let coefficient = actual_edges as f64 / possible_edges as f64;
            total_coefficient += coefficient;
            node_count += 1;
        }
        
        if node_count > 0 {
            total_coefficient / node_count as f64
        } else {
            0.0
        }
    }
    
    /// Detect network partitions
    fn detect_partitions(
        connectivity: &HashMap<NodeId, NodeConnectivity>,
        _config: &TopologyConfig,
    ) -> Vec<NetworkPartition> {
        let mut partitions = Vec::new();
        let mut visited = HashSet::new();
        
        for (node_id, _) in connectivity {
            if visited.contains(node_id) {
                continue;
            }
            
            // Find connected component using DFS
            let mut component = Vec::new();
            let mut stack = vec![node_id.clone()];
            
            while let Some(current) = stack.pop() {
                if visited.contains(&current) {
                    continue;
                }
                
                visited.insert(current.clone());
                component.push(current.clone());
                
                if let Some(node) = connectivity.get(&current) {
                    for neighbor in &node.direct_connections {
                        if !visited.contains(neighbor) {
                            stack.push(neighbor.clone());
                        }
                    }
                }
            }
            
            // Check if this is a significant partition
            let partition_ratio = component.len() as f64 / connectivity.len() as f64;
            if partition_ratio < _config.partition_threshold {
                // This is a partition (small disconnected component)
                partitions.push(NetworkPartition {
                    id: format!("partition_{}", partitions.len()),
                    nodes: component,
                    detected_at: SystemTime::now(),
                    is_isolated: true,
                    bridge_nodes: Vec::new(), // Would be calculated in real implementation
                });
            }
        }
        
        partitions
    }
    
    /// Calculate overall network health score
    fn calculate_health_score(
        info: &TopologyInfo,
        connectivity: &HashMap<NodeId, NodeConnectivity>,
        _config: &TopologyConfig,
    ) -> f64 {
        let mut score = 1.0;
        
        // Penalize for disconnected nodes
        if info.total_nodes > 0 {
            let connectivity_ratio = info.connected_nodes as f64 / info.total_nodes as f64;
            score *= connectivity_ratio;
        }
        
        // Penalize for network partitions
        if !info.partitions.is_empty() {
            score *= 0.5; // Severe penalty for partitions
        }
        
        // Penalize for poor reliability
        let avg_reliability: f64 = connectivity.values()
            .map(|node| node.reliability)
            .sum::<f64>() / connectivity.len().max(1) as f64;
        score *= avg_reliability;
        
        // Penalize for high latency
        let avg_latency: f64 = connectivity.values()
            .map(|node| node.latency.as_millis() as f64)
            .sum::<f64>() / connectivity.len().max(1) as f64;
        
        if avg_latency > 1000.0 { // > 1 second is very poor
            score *= 0.5;
        } else if avg_latency > 500.0 { // > 0.5 seconds is poor
            score *= 0.8;
        }
        
        score.max(0.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_network_topology_creation() {
        let node_id = NodeId::random();
        let topology = NetworkTopology::new(node_id.clone());
        
        let status = topology.status().await;
        assert_eq!(status.our_node_id, node_id);
        assert_eq!(status.total_nodes, 0);
        assert_eq!(status.health_score, 1.0);
    }
    
    #[tokio::test]
    async fn test_node_join_leave() {
        let our_node_id = NodeId::random();
        let topology = NetworkTopology::new(our_node_id);
        
        let peer_id = NodeId::random();
        topology.node_joined(peer_id.clone()).await.unwrap();
        
        let status = topology.status().await;
        assert_eq!(status.total_nodes, 1);
        
        topology.node_left(&peer_id, "Test".to_string()).await.unwrap();
        
        let status = topology.status().await;
        assert_eq!(status.total_nodes, 0);
    }
    
    #[tokio::test]
    async fn test_connection_tracking() {
        let our_node_id = NodeId::random();
        let topology = NetworkTopology::new(our_node_id);
        
        let peer1 = NodeId::random();
        let peer2 = NodeId::random();
        
        topology.node_joined(peer1.clone()).await.unwrap();
        topology.node_joined(peer2.clone()).await.unwrap();
        
        topology.connection_established(
            peer1.clone(),
            peer2.clone(),
            Duration::from_millis(50)
        ).await.unwrap();
        
        let connectivity = topology.connectivity.read().await;
        let peer1_info = connectivity.get(&peer1).unwrap();
        assert!(peer1_info.direct_connections.contains(&peer2));
    }
    
    #[test]
    fn test_clustering_coefficient_calculation() {
        let mut connectivity = HashMap::new();
        
        // Create a triangle (3 nodes, all connected)
        let node1 = NodeId::random();
        let node2 = NodeId::random();
        let node3 = NodeId::random();
        
        let mut connections1 = HashSet::new();
        connections1.insert(node2.clone());
        connections1.insert(node3.clone());
        
        let mut connections2 = HashSet::new();
        connections2.insert(node1.clone());
        connections2.insert(node3.clone());
        
        let mut connections3 = HashSet::new();
        connections3.insert(node1.clone());
        connections3.insert(node2.clone());
        
        connectivity.insert(node1.clone(), NodeConnectivity {
            node_id: node1,
            direct_connections: connections1,
            reachable_nodes: HashSet::new(),
            last_seen: SystemTime::now(),
            latency: Duration::from_millis(100),
            reliability: 1.0,
        });
        
        connectivity.insert(node2.clone(), NodeConnectivity {
            node_id: node2,
            direct_connections: connections2,
            reachable_nodes: HashSet::new(),
            last_seen: SystemTime::now(),
            latency: Duration::from_millis(100),
            reliability: 1.0,
        });
        
        connectivity.insert(node3.clone(), NodeConnectivity {
            node_id: node3,
            direct_connections: connections3,
            reachable_nodes: HashSet::new(),
            last_seen: SystemTime::now(),
            latency: Duration::from_millis(100),
            reliability: 1.0,
        });
        
        let coefficient = NetworkTopology::calculate_clustering_coefficient(&connectivity);
        assert!((coefficient - 1.0).abs() < 0.01); // Perfect triangle should have coefficient 1.0
    }
}
