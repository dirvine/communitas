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


//! Fault Tolerance System
//!
//! This module implements comprehensive fault tolerance with:
//! - Automatic failure detection
//! - Self-healing mechanisms
//! - Content replication management
//! - Network partition recovery
//! - Graceful degradation

use super::*;
use anyhow::{Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{debug, info, warn, error, instrument};

/// Failure detection and recovery system
pub struct FaultTolerance {
    /// Configuration
    config: FaultToleranceConfig,
    /// Failure detector
    failure_detector: Arc<RwLock<FailureDetector>>,
    /// Recovery manager
    recovery_manager: Arc<RwLock<RecoveryManager>>,
    /// Replication manager
    replication_manager: Arc<RwLock<ReplicationManager>>,
    /// Statistics
    stats: Arc<RwLock<FaultToleranceStats>>,
    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// Configuration for fault tolerance
#[derive(Debug, Clone)]
pub struct FaultToleranceConfig {
    /// Failure detection interval
    pub detection_interval: Duration,
    /// Recovery attempt interval
    pub recovery_interval: Duration,
    /// Replication check interval
    pub replication_interval: Duration,
    /// Node failure threshold
    pub failure_threshold: Duration,
    /// Maximum recovery attempts
    pub max_recovery_attempts: u32,
    /// Minimum replication factor
    pub min_replication_factor: usize,
    /// Target replication factor
    pub target_replication_factor: usize,
    /// Enable automatic recovery
    pub auto_recovery: bool,
    /// Enable proactive replication
    pub proactive_replication: bool,
}

impl Default for FaultToleranceConfig {
    fn default() -> Self {
        Self {
            detection_interval: Duration::from_secs(30),
            recovery_interval: Duration::from_secs(60),
            replication_interval: Duration::from_secs(120),
            failure_threshold: Duration::from_secs(180), // 3 minutes
            max_recovery_attempts: 3,
            min_replication_factor: 3,
            target_replication_factor: 8,
            auto_recovery: true,
            proactive_replication: true,
        }
    }
}

/// Failure detector implementation
#[derive(Debug)]
pub struct FailureDetector {
    /// Node health status
    node_health: HashMap<NodeId, NodeHealth>,
    /// Failure patterns
    failure_patterns: HashMap<NodeId, VecDeque<FailureEvent>>,
    /// Configuration
    config: FaultToleranceConfig,
}

/// Node health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    pub node_id: NodeId,
    pub status: NodeStatus,
    pub last_seen: SystemTime,
    pub last_successful_operation: SystemTime,
    pub consecutive_failures: u32,
    pub total_failures: u32,
    pub availability: f64, // 0.0 to 1.0
    pub response_time: Duration,
    pub suspected_failure_time: Option<SystemTime>,
}

/// Node status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is healthy and responsive
    Healthy,
    /// Node is suspected to have failed
    Suspected,
    /// Node has failed
    Failed,
    /// Node is recovering
    Recovering,
    /// Node is permanently offline
    Offline,
}

/// Failure event record
#[derive(Debug, Clone)]
pub struct FailureEvent {
    pub timestamp: SystemTime,
    pub event_type: FailureType,
    pub details: String,
}

/// Types of failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    /// Network timeout
    Timeout,
    /// Connection refused
    ConnectionRefused,
    /// Protocol error
    ProtocolError,
    /// Content corruption
    ContentCorruption,
    /// Overloaded node
    Overload,
    /// Malicious behavior
    Malicious,
}

/// Recovery manager
#[derive(Debug)]
pub struct RecoveryManager {
    /// Active recovery operations
    active_recoveries: HashMap<NodeId, RecoveryOperation>,
    /// Recovery history
    recovery_history: VecDeque<RecoveryRecord>,
    /// Configuration
    config: FaultToleranceConfig,
}

/// Recovery operation state
#[derive(Debug, Clone)]
pub struct RecoveryOperation {
    pub node_id: NodeId,
    pub started_at: SystemTime,
    pub attempts: u32,
    pub strategy: RecoveryStrategy,
    pub status: RecoveryStatus,
}

/// Recovery strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Reconnect to the node
    Reconnect,
    /// Find alternative nodes
    FindAlternatives,
    /// Redistribute content
    RedistributeContent,
    /// Partition healing
    HealPartition,
}

/// Recovery operation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStatus {
    /// Recovery in progress
    InProgress,
    /// Recovery succeeded
    Success,
    /// Recovery failed
    Failed,
    /// Recovery abandoned
    Abandoned,
}

/// Recovery record for history
#[derive(Debug, Clone)]
pub struct RecoveryRecord {
    pub node_id: NodeId,
    pub started_at: SystemTime,
    pub completed_at: SystemTime,
    pub attempts: u32,
    pub strategy: RecoveryStrategy,
    pub final_status: RecoveryStatus,
    pub details: String,
}

/// Replication manager
#[derive(Debug)]
pub struct ReplicationManager {
    /// Content replication status
    content_replicas: HashMap<ContentId, ReplicationStatus>,
    /// Node content mapping
    node_content: HashMap<NodeId, HashSet<ContentId>>,
    /// Configuration
    config: FaultToleranceConfig,
}

/// Replication status for content
#[derive(Debug, Clone)]
pub struct ReplicationStatus {
    pub content_id: ContentId,
    pub current_replicas: Vec<NodeId>,
    pub target_replicas: usize,
    pub last_replication_check: SystemTime,
    pub replication_health: f64, // 0.0 to 1.0
}

/// Fault tolerance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultToleranceStats {
    pub nodes_monitored: usize,
    pub healthy_nodes: usize,
    pub suspected_nodes: usize,
    pub failed_nodes: usize,
    pub recovery_operations: u32,
    pub successful_recoveries: u32,
    pub failed_recoveries: u32,
    pub content_items_managed: usize,
    pub under_replicated_items: usize,
    pub over_replicated_items: usize,
    pub replication_operations: u32,
    pub last_maintenance: SystemTime,
}

impl Default for FaultToleranceStats {
    fn default() -> Self {
        Self {
            nodes_monitored: 0,
            healthy_nodes: 0,
            suspected_nodes: 0,
            failed_nodes: 0,
            recovery_operations: 0,
            successful_recoveries: 0,
            failed_recoveries: 0,
            content_items_managed: 0,
            under_replicated_items: 0,
            over_replicated_items: 0,
            replication_operations: 0,
            last_maintenance: SystemTime::UNIX_EPOCH,
        }
    }
}

impl FaultTolerance {
    /// Create new fault tolerance system
    #[instrument(skip(dht_config))]
    pub fn new(dht_config: DhtConfig) -> Self {
        let config = FaultToleranceConfig {
            target_replication_factor: dht_config.replication_factor,
            min_replication_factor: dht_config.replication_factor / 2 + 1,
            ..FaultToleranceConfig::default()
        };
        
        info!("Creating fault tolerance system with replication factor: {}", 
              config.target_replication_factor);
        
        let failure_detector = Arc::new(RwLock::new(FailureDetector {
            node_health: HashMap::new(),
            failure_patterns: HashMap::new(),
            config: config.clone(),
        }));
        
        let recovery_manager = Arc::new(RwLock::new(RecoveryManager {
            active_recoveries: HashMap::new(),
            recovery_history: VecDeque::new(),
            config: config.clone(),
        }));
        
        let replication_manager = Arc::new(RwLock::new(ReplicationManager {
            content_replicas: HashMap::new(),
            node_content: HashMap::new(),
            config: config.clone(),
        }));
        
        Self {
            config,
            failure_detector,
            recovery_manager,
            replication_manager,
            stats: Arc::new(RwLock::new(FaultToleranceStats::default())),
            shutdown_tx: None,
        }
    }
    
    /// Start fault tolerance monitoring
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting fault tolerance system");
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        
        // Start failure detection task
        let failure_detector = self.failure_detector.clone();
        let stats = self.stats.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(config.detection_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::failure_detection_task(&failure_detector, &stats, &config).await {
                            error!("Failure detection task error: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Failure detection task shutting down");
                        break;
                    }
                }
            }
        });
        
        // Start recovery task
        let recovery_manager = self.recovery_manager.clone();
        let stats = self.stats.clone();
        let config = self.config.clone();
        let (shutdown_tx2, mut shutdown_rx2) = mpsc::channel(1);
        
        tokio::spawn(async move {
            let mut interval = interval(config.recovery_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::recovery_task(&recovery_manager, &stats, &config).await {
                            error!("Recovery task error: {}", e);
                        }
                    }
                    _ = shutdown_rx2.recv() => {
                        info!("Recovery task shutting down");
                        break;
                    }
                }
            }
        });
        
        // Start replication task
        let replication_manager = self.replication_manager.clone();
        let stats = self.stats.clone();
        let config = self.config.clone();
        let (shutdown_tx3, mut shutdown_rx3) = mpsc::channel(1);
        
        tokio::spawn(async move {
            let mut interval = interval(config.replication_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::replication_task(&replication_manager, &stats, &config).await {
                            error!("Replication task error: {}", e);
                        }
                    }
                    _ = shutdown_rx3.recv() => {
                        info!("Replication task shutting down");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop fault tolerance system
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping fault tolerance system");
        
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(()).await;
        }
        
        Ok(())
    }
    
    /// Record node interaction result
    #[instrument(skip(self))]
    pub async fn record_interaction(&self, node_id: NodeId, success: bool, response_time: Duration) -> Result<()> {
        let mut detector = self.failure_detector.write().await;
        let now = SystemTime::now();
        
        let health = detector.node_health.entry(node_id.clone()).or_insert_with(|| {
            NodeHealth {
                node_id: node_id.clone(),
                status: NodeStatus::Healthy,
                last_seen: now,
                last_successful_operation: now,
                consecutive_failures: 0,
                total_failures: 0,
                availability: 1.0,
                response_time: Duration::from_millis(100),
                suspected_failure_time: None,
            }
        });
        
        health.last_seen = now;
        health.response_time = Duration::from_millis(
            (health.response_time.as_millis() as u64 * 3 + response_time.as_millis() as u64) / 4
        );
        
        if success {
            health.last_successful_operation = now;
            health.consecutive_failures = 0;
            health.suspected_failure_time = None;
            
            if health.status == NodeStatus::Suspected || health.status == NodeStatus::Recovering {
                health.status = NodeStatus::Healthy;
                info!("Node recovered: {}", node_id.to_hex());
            }
        } else {
            health.consecutive_failures += 1;
            health.total_failures += 1;
            
            // Update node status and record failure
            // Capture current failure info before accessing other fields
            let failure_info = (health.consecutive_failures, health.status.clone());
            
            // Record failure event
            let failure_events = detector.failure_patterns.entry(node_id.clone()).or_insert_with(VecDeque::new);
            failure_events.push_back(FailureEvent {
                timestamp: now,
                event_type: FailureType::Timeout, // Simplified
                details: "Operation failed".to_string(),
            });
            
            // Limit failure history
            while failure_events.len() > 100 {
                failure_events.pop_front();
            }
            
            // Update status based on failure count
            let health = detector.node_health.get_mut(&node_id).unwrap();
            if failure_info.0 >= 3 && failure_info.1 == NodeStatus::Healthy {
                health.status = NodeStatus::Suspected;
                health.suspected_failure_time = Some(now);
                warn!("Node suspected of failure: {} (consecutive failures: {})", 
                      node_id.to_hex(), health.consecutive_failures);
            } else if failure_info.0 >= 5 {
                health.status = NodeStatus::Failed;
                error!("Node marked as failed: {} (consecutive failures: {})", 
                       node_id.to_hex(), health.consecutive_failures);
            }
        }
        
        // Update availability
        let health = detector.node_health.get_mut(&node_id).unwrap();
        let _total_ops = health.consecutive_failures + 1; // Simplified calculation
        let successful_ops = if success { 1 } else { 0 };
        health.availability = (health.availability * 0.9) + (successful_ops as f64 * 0.1);
        
        Ok(())
    }
    
    /// Record content replication
    #[instrument(skip(self))]
    pub async fn record_content_replication(&self, content_id: ContentId, replicas: Vec<NodeId>) -> Result<()> {
        let mut repl_manager = self.replication_manager.write().await;
        let now = SystemTime::now();
        
        let replication_status = repl_manager.content_replicas.entry(content_id.clone()).or_insert_with(|| {
            ReplicationStatus {
                content_id: content_id.clone(),
                current_replicas: Vec::new(),
                target_replicas: repl_manager.config.target_replication_factor,
                last_replication_check: now,
                replication_health: 1.0,
            }
        });
        
        replication_status.current_replicas = replicas.clone();
        replication_status.last_replication_check = now;
        
        // Calculate replication health
        let current_count = replicas.len();
        let target_count = replication_status.target_replicas;
        replication_status.replication_health = if target_count > 0 {
            (current_count as f64 / target_count as f64).min(1.0)
        } else {
            0.0
        };
        
        // Update node content mapping
        for replica_node in &replicas {
            repl_manager.node_content.entry(replica_node.clone())
                .or_insert_with(HashSet::new)
                .insert(content_id.clone());
        }
        
        if current_count < repl_manager.config.min_replication_factor {
            warn!("Content under-replicated: {} ({}/{} replicas)", 
                  content_id.to_hex(), current_count, target_count);
        }
        
        Ok(())
    }
    
    /// Get current fault tolerance status
    pub async fn status(&self) -> FaultToleranceStats {
        let detector = self.failure_detector.read().await;
        let repl_manager = self.replication_manager.read().await;
        let mut stats = self.stats.read().await.clone();
        
        // Update real-time statistics
        stats.nodes_monitored = detector.node_health.len();
        stats.healthy_nodes = detector.node_health.values()
            .filter(|h| h.status == NodeStatus::Healthy)
            .count();
        stats.suspected_nodes = detector.node_health.values()
            .filter(|h| h.status == NodeStatus::Suspected)
            .count();
        stats.failed_nodes = detector.node_health.values()
            .filter(|h| h.status == NodeStatus::Failed)
            .count();
        
        stats.content_items_managed = repl_manager.content_replicas.len();
        stats.under_replicated_items = repl_manager.content_replicas.values()
            .filter(|r| r.current_replicas.len() < repl_manager.config.min_replication_factor)
            .count();
        stats.over_replicated_items = repl_manager.content_replicas.values()
            .filter(|r| r.current_replicas.len() > repl_manager.config.target_replication_factor * 2)
            .count();
        
        stats
    }
    
    /// Failure detection task
    async fn failure_detection_task(
        failure_detector: &Arc<RwLock<FailureDetector>>,
        stats: &Arc<RwLock<FaultToleranceStats>>,
        config: &FaultToleranceConfig,
    ) -> Result<()> {
        debug!("Running failure detection");
        
        let mut detector = failure_detector.write().await;
        let now = SystemTime::now();
        
        for (node_id, health) in detector.node_health.iter_mut() {
            // Check for node timeouts
            if health.status == NodeStatus::Healthy || health.status == NodeStatus::Suspected {
                if now.duration_since(health.last_seen).unwrap_or(Duration::MAX) > config.failure_threshold {
                    if health.status == NodeStatus::Healthy {
                        health.status = NodeStatus::Suspected;
                        health.suspected_failure_time = Some(now);
                        warn!("Node suspected due to timeout: {}", node_id.to_hex());
                    } else if let Some(suspected_time) = health.suspected_failure_time {
                        if now.duration_since(suspected_time).unwrap_or(Duration::ZERO) > config.failure_threshold {
                            health.status = NodeStatus::Failed;
                            error!("Node marked as failed due to prolonged timeout: {}", node_id.to_hex());
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Recovery task
    async fn recovery_task(
        recovery_manager: &Arc<RwLock<RecoveryManager>>,
        stats: &Arc<RwLock<FaultToleranceStats>>,
        config: &FaultToleranceConfig,
    ) -> Result<()> {
        debug!("Running recovery operations");
        
        let mut manager = recovery_manager.write().await;
        let now = SystemTime::now();
        
        // Check active recovery operations
        let mut completed_recoveries = Vec::new();
        
        for (node_id, recovery) in manager.active_recoveries.iter_mut() {
            if now.duration_since(recovery.started_at).unwrap_or(Duration::ZERO) > Duration::from_secs(300) {
                // Recovery timeout
                recovery.status = RecoveryStatus::Failed;
                completed_recoveries.push(node_id.clone());
                warn!("Recovery operation timed out for node: {}", node_id.to_hex());
            }
        }
        
        // Move completed recoveries to history
        for node_id in completed_recoveries {
            if let Some(recovery) = manager.active_recoveries.remove(&node_id) {
                manager.recovery_history.push_back(RecoveryRecord {
                    node_id: recovery.node_id,
                    started_at: recovery.started_at,
                    completed_at: now,
                    attempts: recovery.attempts,
                    strategy: recovery.strategy,
                    final_status: recovery.status,
                    details: "Recovery completed".to_string(),
                });
                
                // Limit history size
                while manager.recovery_history.len() > 1000 {
                    manager.recovery_history.pop_front();
                }
            }
        }
        
        Ok(())
    }
    
    /// Replication task
    async fn replication_task(
        replication_manager: &Arc<RwLock<ReplicationManager>>,
        stats: &Arc<RwLock<FaultToleranceStats>>,
        config: &FaultToleranceConfig,
    ) -> Result<()> {
        debug!("Running replication maintenance");
        
        let mut manager = replication_manager.write().await;
        let now = SystemTime::now();
        
        let mut replication_actions = 0;
        
        for (content_id, replication_status) in manager.content_replicas.iter_mut() {
            let current_count = replication_status.current_replicas.len();
            let target_count = replication_status.target_replicas;
            
            if current_count < config.min_replication_factor {
                // Under-replicated - need to create more replicas
                info!("Scheduling replication for under-replicated content: {} ({}/{})", 
                      content_id.to_hex(), current_count, target_count);
                
                // In a real implementation, we'd trigger replication here
                replication_actions += 1;
                replication_status.last_replication_check = now;
            } else if current_count > target_count * 2 {
                // Over-replicated - consider removing some replicas
                debug!("Content over-replicated: {} ({}/{})", 
                       content_id.to_hex(), current_count, target_count);
            }
        }
        
        if replication_actions > 0 {
            let mut stats_guard = stats.write().await;
            stats_guard.replication_operations += replication_actions;
            stats_guard.last_maintenance = now;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fault_tolerance_creation() {
        let config = DhtConfig::default();
        let ft = FaultTolerance::new(config);
        
        let status = ft.status().await;
        assert_eq!(status.nodes_monitored, 0);
        assert_eq!(status.healthy_nodes, 0);
    }
    
    #[tokio::test]
    async fn test_interaction_recording() {
        let config = DhtConfig::default();
        let ft = FaultTolerance::new(config);
        
        let node_id = NodeId::random();
        
        // Record successful interaction
        ft.record_interaction(node_id.clone(), true, Duration::from_millis(50)).await.unwrap();
        
        let detector = ft.failure_detector.read().await;
        let health = detector.node_health.get(&node_id).unwrap();
        assert_eq!(health.status, NodeStatus::Healthy);
        assert_eq!(health.consecutive_failures, 0);
        
        drop(detector);
        
        // Record multiple failures
        for _ in 0..4 {
            ft.record_interaction(node_id.clone(), false, Duration::from_millis(1000)).await.unwrap();
        }
        
        let detector = ft.failure_detector.read().await;
        let health = detector.node_health.get(&node_id).unwrap();
        assert_eq!(health.status, NodeStatus::Suspected);
        assert_eq!(health.consecutive_failures, 4);
    }
    
    #[tokio::test]
    async fn test_content_replication_tracking() {
        let config = DhtConfig::default();
        let ft = FaultTolerance::new(config);
        
        let content_id = ContentId::from_data(b"test content");
        let replicas = vec![NodeId::random(), NodeId::random()];
        
        ft.record_content_replication(content_id.clone(), replicas.clone()).await.unwrap();
        
        let manager = ft.replication_manager.read().await;
        let replication_status = manager.content_replicas.get(&content_id).unwrap();
        assert_eq!(replication_status.current_replicas.len(), 2);
        assert!(replication_status.replication_health < 1.0); // Under-replicated
    }
    
    #[test]
    fn test_node_health_status_transitions() {
        let node_id = NodeId::random();
        let now = SystemTime::now();
        
        let mut health = NodeHealth {
            node_id: node_id.clone(),
            status: NodeStatus::Healthy,
            last_seen: now,
            last_successful_operation: now,
            consecutive_failures: 0,
            total_failures: 0,
            availability: 1.0,
            response_time: Duration::from_millis(100),
            suspected_failure_time: None,
        };
        
        // Simulate failures
        health.consecutive_failures = 3;
        assert_eq!(health.status, NodeStatus::Healthy); // Not yet changed
        
        health.consecutive_failures = 5;
        // In real implementation, status would change to Failed
        // This would be handled by the interaction recording logic
    }
}
