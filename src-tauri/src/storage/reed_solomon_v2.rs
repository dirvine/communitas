// Copyright 2024 Saorsa Labs
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Enhanced Reed Solomon v2 with 60% availability target and dynamic membership

#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Target availability percentage (60% of members need to be online)
const TARGET_AVAILABILITY: f32 = 0.60;
const AVAILABILITY_TOLERANCE: f32 = 0.05; // ±5% tolerance
const REBALANCE_THRESHOLD: f32 = 0.20; // Rebalance if size changes by 20%
const GRACE_PERIOD_SECONDS: i64 = 300; // 5 minutes before rebalancing

/// Hybrid Logical Clock for causal ordering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct HybridLogicalClock {
    pub physical_time: i64,
    pub logical_counter: u32,
    pub node_id: String,
}

impl HybridLogicalClock {
    pub fn new(node_id: String) -> Self {
        Self {
            physical_time: Utc::now().timestamp_millis(),
            logical_counter: 0,
            node_id,
        }
    }

    pub fn tick(&mut self) -> Self {
        let now = Utc::now().timestamp_millis();
        if now > self.physical_time {
            self.physical_time = now;
            self.logical_counter = 0;
        } else {
            self.logical_counter += 1;
        }
        self.clone()
    }

    pub fn update(&mut self, other: &HybridLogicalClock) {
        let now = Utc::now().timestamp_millis();
        let max_time = now.max(self.physical_time).max(other.physical_time);

        if max_time > self.physical_time {
            self.physical_time = max_time;
            self.logical_counter = 0;
        } else if max_time == self.physical_time && other.physical_time == max_time {
            self.logical_counter = self.logical_counter.max(other.logical_counter) + 1;
        }
    }
}

/// Enhanced Reed Solomon configuration optimized for 60% availability
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OptimalReedSolomonConfig {
    pub data_shards: usize,      // k - minimum shards needed
    pub parity_shards: usize,    // m - redundancy shards
    pub shard_size: usize,       // bytes per shard
    pub member_count: usize,     // current group size
    pub availability_ratio: f32, // k/(k+m) - should be ~0.6
    pub generation: u64,         // configuration version
}

impl OptimalReedSolomonConfig {
    /// Calculate optimal k,m values for 60% availability target
    pub fn for_group_size(member_count: usize) -> Self {
        let (k, m) = Self::calculate_optimal_shards(member_count);

        Self {
            data_shards: k,
            parity_shards: m,
            shard_size: Self::calculate_shard_size(member_count),
            member_count,
            availability_ratio: k as f32 / (k + m) as f32,
            generation: 0,
        }
    }

    fn calculate_optimal_shards(member_count: usize) -> (usize, usize) {
        // Calculate total shards based on group size
        // We want each member to hold 1-3 shards ideally
        let total_shards = match member_count {
            1..=3 => 3,     // Minimum viable
            4..=6 => 5,     // Small group
            7..=10 => 7,    // Medium group
            11..=20 => 12,  // Large group
            21..=50 => 20,  // XL group
            51..=100 => 30, // XXL group
            _ => 50,        // Massive group
        };

        // Calculate k to achieve ~60% availability
        let k = ((total_shards as f32 * TARGET_AVAILABILITY).round() as usize).max(1);
        let m = total_shards - k;

        // Ensure we have at least some redundancy
        if m == 0 && total_shards > 1 {
            (k - 1, 1)
        } else {
            (k, m)
        }
    }

    fn calculate_shard_size(member_count: usize) -> usize {
        match member_count {
            1..=10 => 4096,  // 4KB for small groups
            11..=50 => 8192, // 8KB for medium groups
            _ => 16384,      // 16KB for large groups
        }
    }

    pub fn total_shards(&self) -> usize {
        self.data_shards + self.parity_shards
    }

    pub fn minimum_members_required(&self) -> usize {
        self.data_shards
    }

    pub fn maximum_failures_tolerated(&self) -> usize {
        self.parity_shards
    }

    pub fn is_within_tolerance(&self) -> bool {
        let ratio = self.availability_ratio;
        ratio >= (TARGET_AVAILABILITY - AVAILABILITY_TOLERANCE)
            && ratio <= (TARGET_AVAILABILITY + AVAILABILITY_TOLERANCE)
    }
}

/// Member reliability tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberReliability {
    pub member_id: String,
    pub uptime_percentage: f32,
    pub average_response_time_ms: u32,
    pub successful_retrievals: u64,
    pub failed_retrievals: u64,
    pub last_seen: DateTime<Utc>,
    pub reliability_score: f32, // 0.0 to 1.0
}

impl MemberReliability {
    pub fn new(member_id: String) -> Self {
        Self {
            member_id,
            uptime_percentage: 100.0,
            average_response_time_ms: 0,
            successful_retrievals: 0,
            failed_retrievals: 0,
            last_seen: Utc::now(),
            reliability_score: 1.0,
        }
    }

    pub fn update_reliability(&mut self, success: bool, response_time_ms: Option<u32>) {
        if success {
            self.successful_retrievals += 1;
            if let Some(time) = response_time_ms {
                // Rolling average
                self.average_response_time_ms =
                    ((self.average_response_time_ms as u64 * self.successful_retrievals
                        + time as u64)
                        / (self.successful_retrievals + 1)) as u32;
            }
        } else {
            self.failed_retrievals += 1;
        }

        self.last_seen = Utc::now();
        self.calculate_reliability_score();
    }

    fn calculate_reliability_score(&mut self) {
        let total_requests = self.successful_retrievals + self.failed_retrievals;
        if total_requests == 0 {
            self.reliability_score = 1.0;
            return;
        }

        let success_rate = self.successful_retrievals as f32 / total_requests as f32;
        let response_factor = 1.0 - (self.average_response_time_ms as f32 / 5000.0).min(1.0);
        let recency_factor = {
            let hours_since_seen = (Utc::now() - self.last_seen).num_hours();
            1.0 - (hours_since_seen as f32 / 168.0).min(1.0) // 168 hours = 1 week
        };

        self.reliability_score =
            (success_rate * 0.5 + response_factor * 0.3 + recency_factor * 0.2)
                .max(0.0)
                .min(1.0);
    }
}

/// Group membership state machine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupState {
    Stable {
        config: OptimalReedSolomonConfig,
        members: Vec<String>,
    },
    MemberJoining {
        config: OptimalReedSolomonConfig,
        current_members: Vec<String>,
        joining_member: String,
        initiated_at: DateTime<Utc>,
    },
    MemberLeaving {
        config: OptimalReedSolomonConfig,
        current_members: Vec<String>,
        leaving_member: String,
        initiated_at: DateTime<Utc>,
    },
    GracePeriod {
        config: OptimalReedSolomonConfig,
        pending_changes: Vec<MembershipChange>,
        grace_period_ends: DateTime<Utc>,
    },
    Rebalancing {
        old_config: OptimalReedSolomonConfig,
        new_config: OptimalReedSolomonConfig,
        migration_progress: f32,
        started_at: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MembershipChange {
    pub change_type: MembershipChangeType,
    pub member_id: String,
    pub timestamp: DateTime<Utc>,
    pub hlc: HybridLogicalClock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MembershipChangeType {
    Join,
    Leave,
}

/// Shard with generation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedShard {
    pub index: usize,
    pub shard_type: ShardType,
    pub data: Vec<u8>,
    pub group_id: String,
    pub data_id: String,
    pub generation: u64, // Configuration version
    pub integrity_hash: String,
    pub created_at: DateTime<Utc>,
    pub hlc: HybridLogicalClock,     // For causal ordering
    pub holder_id: String,           // Current holder
    pub backup_holders: Vec<String>, // Backup holders
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShardType {
    Data,
    Parity,
}

/// Distribution strategy with member affinity
#[derive(Debug, Clone)]
pub struct IntelligentShardDistributor {
    member_reliability: Arc<RwLock<HashMap<String, MemberReliability>>>,
    consistent_hash_ring: Arc<RwLock<ConsistentHashRing>>,
    distribution_history: Arc<RwLock<VecDeque<DistributionEvent>>>,
}

#[derive(Debug, Clone)]
pub struct ConsistentHashRing {
    ring: Vec<(u64, String)>, // (hash, member_id)
    virtual_nodes: usize,     // Virtual nodes per member for better distribution
}

impl ConsistentHashRing {
    pub fn new(members: &[String], virtual_nodes: usize) -> Self {
        let mut ring = Vec::new();

        for member in members {
            for i in 0..virtual_nodes {
                let key = format!("{}:{}", member, i);
                let hash = blake3::hash(key.as_bytes()).as_bytes()[0..8]
                    .try_into()
                    .map(u64::from_le_bytes)
                    .unwrap_or(0);
                ring.push((hash, member.clone()));
            }
        }

        ring.sort_by_key(|&(hash, _)| hash);

        Self {
            ring,
            virtual_nodes,
        }
    }

    pub fn get_nodes_for_shard(&self, shard_id: &str, count: usize) -> Vec<String> {
        if self.ring.is_empty() {
            return Vec::new();
        }

        let hash = blake3::hash(shard_id.as_bytes()).as_bytes()[0..8]
            .try_into()
            .map(u64::from_le_bytes)
            .unwrap_or(0);

        let start_pos = self
            .ring
            .binary_search_by_key(&hash, |&(h, _)| h)
            .unwrap_or_else(|x| x % self.ring.len());

        let mut selected = HashSet::new();
        let mut result = Vec::new();
        let mut pos = start_pos;

        while selected.len() < count.min(self.ring.len() / self.virtual_nodes) {
            let member = &self.ring[pos].1;
            if selected.insert(member.clone()) {
                result.push(member.clone());
            }
            pos = (pos + 1) % self.ring.len();
        }

        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub affected_shards: Vec<usize>,
    pub members_involved: Vec<String>,
    pub success: bool,
}

impl IntelligentShardDistributor {
    pub fn new() -> Self {
        Self {
            member_reliability: Arc::new(RwLock::new(HashMap::new())),
            consistent_hash_ring: Arc::new(RwLock::new(ConsistentHashRing::new(&[], 3))),
            distribution_history: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
        }
    }

    /// Create optimal shard distribution plan
    pub async fn create_distribution_plan(
        &self,
        config: &OptimalReedSolomonConfig,
        members: &[String],
        data_id: &str,
    ) -> Result<ShardDistributionPlan> {
        let reliability_scores = self.member_reliability.read().await;

        // Sort members by reliability score
        let mut sorted_members: Vec<_> = members
            .iter()
            .map(|id| {
                let score = reliability_scores
                    .get(id)
                    .map(|r| r.reliability_score)
                    .unwrap_or(0.5); // New members get neutral score
                (id.clone(), score)
            })
            .collect();

        sorted_members.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Distribute shards with preference for reliable members
        let mut assignments: HashMap<String, Vec<usize>> = HashMap::new();
        let total_shards = config.total_shards();
        let _shards_per_member = (total_shards as f32 / members.len() as f32).ceil() as usize;

        // Assign data shards to most reliable members
        for i in 0..config.data_shards {
            let member_idx = i % sorted_members.len();
            let member_id = &sorted_members[member_idx].0;
            assignments
                .entry(member_id.clone())
                .or_insert_with(Vec::new)
                .push(i);
        }

        // Assign parity shards to remaining members
        for i in 0..config.parity_shards {
            let shard_idx = config.data_shards + i;
            let member_idx = (i + config.data_shards) % sorted_members.len();
            let member_id = &sorted_members[member_idx].0;
            assignments
                .entry(member_id.clone())
                .or_insert_with(Vec::new)
                .push(shard_idx);
        }

        // Ensure minimum distribution (each member should have at least one shard if possible)
        for member in members {
            assignments.entry(member.clone()).or_insert_with(Vec::new);
        }

        Ok(ShardDistributionPlan {
            config: config.clone(),
            assignments,
            primary_holders: self.calculate_primary_holders(&sorted_members, total_shards),
            backup_holders: self.calculate_backup_holders(&sorted_members, total_shards),
            data_id: data_id.to_string(),
            generation: config.generation,
        })
    }

    fn calculate_primary_holders(
        &self,
        sorted_members: &[(String, f32)],
        total_shards: usize,
    ) -> HashMap<usize, String> {
        let mut primary = HashMap::new();

        for shard_idx in 0..total_shards {
            let member_idx = shard_idx % sorted_members.len();
            primary.insert(shard_idx, sorted_members[member_idx].0.clone());
        }

        primary
    }

    fn calculate_backup_holders(
        &self,
        sorted_members: &[(String, f32)],
        total_shards: usize,
    ) -> HashMap<usize, Vec<String>> {
        let mut backups = HashMap::new();

        for shard_idx in 0..total_shards {
            let primary_idx = shard_idx % sorted_members.len();
            let mut shard_backups = Vec::new();

            // Add 2 backup holders
            for i in 1..=2 {
                let backup_idx = (primary_idx + i) % sorted_members.len();
                if backup_idx != primary_idx {
                    shard_backups.push(sorted_members[backup_idx].0.clone());
                }
            }

            backups.insert(shard_idx, shard_backups);
        }

        backups
    }

    /// Update member reliability based on interaction
    pub async fn update_member_reliability(
        &self,
        member_id: &str,
        success: bool,
        response_time_ms: Option<u32>,
    ) {
        let mut reliability = self.member_reliability.write().await;

        reliability
            .entry(member_id.to_string())
            .and_modify(|r| r.update_reliability(success, response_time_ms))
            .or_insert_with(|| {
                let mut r = MemberReliability::new(member_id.to_string());
                r.update_reliability(success, response_time_ms);
                r
            });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardDistributionPlan {
    pub config: OptimalReedSolomonConfig,
    pub assignments: HashMap<String, Vec<usize>>, // member_id -> shard indices
    pub primary_holders: HashMap<usize, String>,  // shard_idx -> primary holder
    pub backup_holders: HashMap<usize, Vec<String>>, // shard_idx -> backup holders
    pub data_id: String,
    pub generation: u64,
}

/// Dynamic membership manager
#[derive(Debug)]
pub struct DynamicMembershipManager {
    group_states: Arc<RwLock<HashMap<String, GroupState>>>,
    pending_rebalances: Arc<RwLock<HashMap<String, RebalanceTask>>>,
    hlc: Arc<RwLock<HybridLogicalClock>>,
}

#[derive(Debug, Clone)]
pub struct RebalanceTask {
    pub group_id: String,
    pub old_config: OptimalReedSolomonConfig,
    pub new_config: OptimalReedSolomonConfig,
    pub scheduled_at: DateTime<Utc>,
    pub reason: String,
}

impl DynamicMembershipManager {
    pub fn new(node_id: String) -> Self {
        Self {
            group_states: Arc::new(RwLock::new(HashMap::new())),
            pending_rebalances: Arc::new(RwLock::new(HashMap::new())),
            hlc: Arc::new(RwLock::new(HybridLogicalClock::new(node_id))),
        }
    }

    /// Check if rebalancing is needed
    pub async fn should_rebalance(
        &self,
        group_id: &str,
        current_members: &[String],
    ) -> Result<bool> {
        let states = self.group_states.read().await;

        if let Some(state) = states.get(group_id) {
            match state {
                GroupState::Stable { config, members } => {
                    // Check if membership size changed significantly
                    let size_change = (current_members.len() as f32 - members.len() as f32).abs();
                    let change_ratio = size_change / members.len() as f32;

                    if change_ratio > REBALANCE_THRESHOLD {
                        info!(
                            "Group {} needs rebalancing: size changed from {} to {} ({}% change)",
                            group_id,
                            members.len(),
                            current_members.len(),
                            (change_ratio * 100.0) as i32
                        );
                        return Ok(true);
                    }

                    // Check if configuration is still optimal for current size
                    let optimal_config =
                        OptimalReedSolomonConfig::for_group_size(current_members.len());
                    if !config.is_within_tolerance()
                        || config.data_shards != optimal_config.data_shards
                    {
                        info!(
                            "Group {} needs rebalancing: configuration no longer optimal",
                            group_id
                        );
                        return Ok(true);
                    }
                }
                _ => {
                    // Already in transition, don't trigger another rebalance
                    return Ok(false);
                }
            }
        }

        Ok(false)
    }

    /// Handle member joining
    pub async fn handle_member_join(&self, group_id: &str, new_member: String) -> Result<()> {
        let mut states = self.group_states.write().await;
        let mut hlc = self.hlc.write().await;
        let timestamp = hlc.tick();

        let state = states
            .entry(group_id.to_string())
            .or_insert_with(|| GroupState::Stable {
                config: OptimalReedSolomonConfig::for_group_size(1),
                members: vec![],
            });

        match state {
            GroupState::Stable { config, members } => {
                *state = GroupState::MemberJoining {
                    config: config.clone(),
                    current_members: members.clone(),
                    joining_member: new_member.clone(),
                    initiated_at: Utc::now(),
                };
                info!("Member {} joining group {}", new_member, group_id);
            }
            GroupState::GracePeriod {
                pending_changes, ..
            } => {
                pending_changes.push(MembershipChange {
                    change_type: MembershipChangeType::Join,
                    member_id: new_member.clone(),
                    timestamp: Utc::now(),
                    hlc: timestamp,
                });
                info!(
                    "Member {} queued to join group {} during grace period",
                    new_member, group_id
                );
            }
            _ => {
                warn!(
                    "Cannot add member {} to group {} in current state",
                    new_member, group_id
                );
            }
        }

        Ok(())
    }

    /// Process grace period and decide on rebalancing
    pub async fn process_grace_period(&self, group_id: &str) -> Result<()> {
        let mut states = self.group_states.write().await;

        if let Some(state) = states.get_mut(group_id) {
            if let GroupState::GracePeriod {
                config,
                pending_changes,
                grace_period_ends,
            } = state
            {
                if Utc::now() >= *grace_period_ends {
                    // Apply all pending changes
                    let mut current_members: HashSet<String> = HashSet::new();

                    // Start with current config members
                    // Note: In grace period, we don't have access to previous members
                    // They should be tracked separately or passed in pending_changes

                    // Apply changes
                    for change in pending_changes.iter() {
                        match change.change_type {
                            MembershipChangeType::Join => {
                                current_members.insert(change.member_id.clone());
                            }
                            MembershipChangeType::Leave => {
                                current_members.remove(&change.member_id);
                            }
                        }
                    }

                    let member_vec: Vec<String> = current_members.into_iter().collect();
                    let new_config = OptimalReedSolomonConfig::for_group_size(member_vec.len());

                    // Check if rebalancing is actually needed
                    if config.data_shards != new_config.data_shards
                        || !new_config.is_within_tolerance()
                    {
                        *state = GroupState::Rebalancing {
                            old_config: config.clone(),
                            new_config,
                            migration_progress: 0.0,
                            started_at: Utc::now(),
                        };
                        info!("Starting rebalancing for group {}", group_id);
                    } else {
                        *state = GroupState::Stable {
                            config: config.clone(),
                            members: member_vec,
                        };
                        info!(
                            "No rebalancing needed for group {}, returning to stable",
                            group_id
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_60_percent_availability() {
        // Test various group sizes for ~60% availability
        let test_cases = vec![
            (5, 3, 2),   // 5 members: 3 data + 2 parity = 60% required
            (10, 4, 3),  // 10 members: 4 data + 3 parity = 57% required
            (20, 7, 5),  // 20 members: 7 data + 5 parity = 58% required
            (50, 12, 8), // 50 members: 12 data + 8 parity = 60% required
        ];

        for (members, expected_k, expected_m) in test_cases {
            let config = OptimalReedSolomonConfig::for_group_size(members);

            assert_eq!(
                config.data_shards, expected_k,
                "Failed for {} members: expected k={}",
                members, expected_k
            );
            assert_eq!(
                config.parity_shards, expected_m,
                "Failed for {} members: expected m={}",
                members, expected_m
            );

            let availability = config.data_shards as f32 / config.total_shards() as f32;
            assert!(
                (availability - TARGET_AVAILABILITY).abs() <= AVAILABILITY_TOLERANCE,
                "Availability {} not within tolerance of 60% for {} members",
                availability * 100.0,
                members
            );
        }
    }

    #[test]
    fn test_member_reliability_scoring() {
        let mut reliability = MemberReliability::new("member1".to_string());

        // Simulate successful operations
        for _ in 0..8 {
            reliability.update_reliability(true, Some(100));
        }

        // Simulate failures
        for _ in 0..2 {
            reliability.update_reliability(false, None);
        }

        // Score should be around 0.8 (80% success rate)
        assert!(reliability.reliability_score > 0.7 && reliability.reliability_score < 0.9);
    }

    #[test]
    fn test_consistent_hashing() {
        let members = vec![
            "member1".to_string(),
            "member2".to_string(),
            "member3".to_string(),
        ];

        let ring = ConsistentHashRing::new(&members, 3);

        // Should get consistent results for same shard
        let nodes1 = ring.get_nodes_for_shard("shard1", 2);
        let nodes2 = ring.get_nodes_for_shard("shard1", 2);

        assert_eq!(nodes1, nodes2);
        assert_eq!(nodes1.len(), 2);
    }

    #[tokio::test]
    async fn test_rebalancing_decision() {
        let manager = DynamicMembershipManager::new("node1".to_string());

        // Create initial group
        let group_id = "test_group";
        let initial_members = vec!["m1".to_string(), "m2".to_string(), "m3".to_string()];

        {
            let mut states = manager.group_states.write().await;
            states.insert(
                group_id.to_string(),
                GroupState::Stable {
                    config: OptimalReedSolomonConfig::for_group_size(3),
                    members: initial_members.clone(),
                },
            );
        }

        // Small change shouldn't trigger rebalancing
        let slightly_changed = vec![
            "m1".to_string(),
            "m2".to_string(),
            "m3".to_string(),
            "m4".to_string(),
        ];
        assert!(
            !manager
                .should_rebalance(group_id, &slightly_changed)
                .await
                .unwrap()
        );

        // Large change should trigger rebalancing
        let significantly_changed = vec![
            "m1".to_string(),
            "m2".to_string(),
            "m3".to_string(),
            "m4".to_string(),
            "m5".to_string(),
            "m6".to_string(),
            "m7".to_string(),
            "m8".to_string(),
        ];
        assert!(
            manager
                .should_rebalance(group_id, &significantly_changed)
                .await
                .unwrap()
        );
    }
}
