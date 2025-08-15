// Copyright 2024 Saorsa Labs
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Shard distribution system for real P2P network communication

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context, bail};
use tracing::{debug, info, warn, error};
use tokio::sync::RwLock;
// Note: timeout import removed as it's not used
// use tokio::time::timeout;

use saorsa_core::dht::skademlia::SKademlia;
use super::reed_solomon_manager::{Shard, ShardDistributionPlan, EnhancedReedSolomonManager};

/// Message types for shard communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShardMessage {
    StoreShardRequest {
        shard: Shard,
        group_id: String,
        sender_id: String,
    },
    StoreShardResponse {
        success: bool,
        message: String,
        storage_available: bool,
    },
    RetrieveShardRequest {
        group_id: String,
        data_id: String,
        shard_index: usize,
        requester_id: String,
    },
    RetrieveShardResponse {
        shard: Option<Shard>,
        success: bool,
        message: String,
    },
    ShardHealthCheck {
        group_id: String,
        data_id: String,
        shard_indices: Vec<usize>,
    },
    ShardHealthResponse {
        available_shards: Vec<usize>,
        corrupted_shards: Vec<usize>,
    },
}

/// Tracks the status of shard distribution operations
#[derive(Debug, Clone)]
pub struct DistributionStatus {
    pub group_id: String,
    pub total_shards: usize,
    pub successful_distributions: usize,
    pub failed_distributions: usize,
    pub member_responses: HashMap<String, DistributionResult>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub enum DistributionResult {
    Success,
    Failed(String),
    Timeout,
    MemberOffline,
}

/// Manages shard distribution across group members via P2P network
#[derive(Debug)]
pub struct ShardDistributor {
    #[allow(dead_code)]
    dht: Arc<SKademlia>,
    reed_solomon: Arc<EnhancedReedSolomonManager>,
    active_distributions: Arc<RwLock<HashMap<String, DistributionStatus>>>,
    shard_cache: Arc<RwLock<HashMap<String, Vec<Shard>>>>,
    #[allow(dead_code)]
    network_timeout: Duration,
    max_retries: usize,
}

impl ShardDistributor {
    pub fn new(
        dht: Arc<SKademlia>,
        reed_solomon: Arc<EnhancedReedSolomonManager>,
    ) -> Self {
        Self {
            dht,
            reed_solomon,
            active_distributions: Arc::new(RwLock::new(HashMap::new())),
            shard_cache: Arc::new(RwLock::new(HashMap::new())),
            network_timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }

    /// Create a distribution plan for group shards
    pub async fn create_distribution_plan(
        &self,
        group_id: &str,
        shards: &[Shard],
        group_members: &[String],
    ) -> Result<ShardDistributionPlan> {
        self.reed_solomon.create_distribution_plan(group_id, shards, group_members).await
    }

    /// Distribute shards to group members via P2P network
    pub async fn distribute_shards(
        &self,
        distribution_plan: &ShardDistributionPlan,
    ) -> Result<DistributionStatus> {
        let distribution_id = format!("{}:{}", distribution_plan.group_id, chrono::Utc::now().timestamp());
        
        let mut status = DistributionStatus {
            group_id: distribution_plan.group_id.clone(),
            total_shards: distribution_plan.total_shards,
            successful_distributions: 0,
            failed_distributions: 0,
            member_responses: HashMap::new(),
            started_at: chrono::Utc::now(),
            completed_at: None,
        };

        // Track this distribution
        {
            let mut active = self.active_distributions.write().await;
            active.insert(distribution_id.clone(), status.clone());
        }

        info!(
            "Starting shard distribution for group {} to {} members",
            distribution_plan.group_id,
            distribution_plan.member_assignments.len()
        );

        // Distribute shards to each member
        let mut distribution_tasks = Vec::new();

        for (member_id, member_shards) in &distribution_plan.member_assignments {
            for shard in member_shards {
                let task = self.distribute_shard_to_member(
                    member_id.clone(),
                    shard.clone(),
                    distribution_id.clone(),
                );
                distribution_tasks.push(task);
            }
        }

        // Execute all distributions concurrently
        let results = futures::future::join_all(distribution_tasks).await;

        // Process results
        for (member_id, result) in results.iter().enumerate() {
            let member_key = distribution_plan.member_assignments.keys()
                .nth(member_id)
                .unwrap();
                
            match result {
                Ok(_) => {
                    status.successful_distributions += 1;
                    status.member_responses.insert(
                        member_key.clone(),
                        DistributionResult::Success,
                    );
                }
                Err(e) => {
                    status.failed_distributions += 1;
                    status.member_responses.insert(
                        member_key.clone(),
                        DistributionResult::Failed(e.to_string()),
                    );
                    warn!("Failed to distribute shard to {}: {}", member_key, e);
                }
            }
        }

        status.completed_at = Some(chrono::Utc::now());

        // Update tracked distribution
        {
            let mut active = self.active_distributions.write().await;
            active.insert(distribution_id, status.clone());
        }

        // Store distribution success/failure metrics
        let success_rate = (status.successful_distributions as f32 / status.total_shards as f32) * 100.0;
        
        if success_rate >= 75.0 {
            info!(
                "Shard distribution completed for group {} with {:.1}% success rate",
                distribution_plan.group_id, success_rate
            );
        } else {
            error!(
                "Shard distribution failed for group {} with only {:.1}% success rate",
                distribution_plan.group_id, success_rate
            );
        }

        Ok(status)
    }

    /// Collect available shards from group members
    pub async fn collect_available_shards(
        &self,
        group_id: &str,
        data_id: &str,
        group_members: &[String],
    ) -> Result<Vec<Shard>> {
        debug!(
            "Collecting shards for group {} data {} from {} members",
            group_id, data_id, group_members.len()
        );

        // First check local cache
        let cache_key = format!("{}:{}", group_id, data_id);
        {
            let cache = self.shard_cache.read().await;
            if let Some(cached_shards) = cache.get(&cache_key) {
                if !cached_shards.is_empty() {
                    debug!("Found {} cached shards for {}", cached_shards.len(), cache_key);
                    return Ok(cached_shards.clone());
                }
            }
        }

        // Collect shards from group members concurrently
        let mut collection_tasks = Vec::new();
        
        for member_id in group_members {
            let task = self.request_shards_from_member(
                member_id.clone(),
                group_id.to_string(),
                data_id.to_string(),
            );
            collection_tasks.push(task);
        }

        let results = futures::future::join_all(collection_tasks).await;
        let mut available_shards = Vec::new();
        let mut successful_requests = 0;

        for (member_id, result) in group_members.iter().zip(results.iter()) {
            match result {
                Ok(member_shards) => {
                    available_shards.extend(member_shards.iter().cloned());
                    successful_requests += 1;
                    debug!("Collected {} shards from member {}", member_shards.len(), member_id);
                }
                Err(e) => {
                    warn!("Failed to collect shards from member {}: {}", member_id, e);
                }
            }
        }

        // Remove duplicate shards (same index)
        available_shards = self.deduplicate_shards(available_shards);

        // Cache the results
        {
            let mut cache = self.shard_cache.write().await;
            cache.insert(cache_key, available_shards.clone());
        }

        info!(
            "Collected {} unique shards from {}/{} members for group {}",
            available_shards.len(), successful_requests, group_members.len(), group_id
        );

        Ok(available_shards)
    }

    /// Perform health check on distributed shards
    pub async fn check_shard_health(
        &self,
        group_id: &str,
        data_id: &str,
        group_members: &[String],
    ) -> Result<ShardHealthReport> {
        debug!("Performing health check for group {} data {}", group_id, data_id);

        let mut health_tasks = Vec::new();
        
        for member_id in group_members {
            let task = self.check_member_shard_health(
                member_id.clone(),
                group_id.to_string(),
                data_id.to_string(),
            );
            health_tasks.push(task);
        }

        let results = futures::future::join_all(health_tasks).await;
        let mut health_report = ShardHealthReport {
            group_id: group_id.to_string(),
            data_id: data_id.to_string(),
            total_members: group_members.len(),
            responsive_members: 0,
            total_shards_found: 0,
            corrupted_shards: HashSet::new(),
            missing_members: Vec::new(),
            reconstruction_possible: false,
        };

        for (member_id, result) in group_members.iter().zip(results.iter()) {
            match result {
                Ok(member_health) => {
                    health_report.responsive_members += 1;
                    health_report.total_shards_found += member_health.available_shards.len();
                    health_report.corrupted_shards.extend(&member_health.corrupted_shards);
                }
                Err(_) => {
                    health_report.missing_members.push(member_id.clone());
                }
            }
        }

        // Check if reconstruction is still possible
        let reconstruction_status = self.reed_solomon.get_reconstruction_status(
            group_id,
            data_id,
            &[], // We'll need to collect actual shards for this
        ).await?;

        health_report.reconstruction_possible = health_report.total_shards_found >= reconstruction_status.required_shards;

        info!(
            "Health check completed for group {}: {}/{} members responsive, {} shards found, reconstruction {}",
            group_id,
            health_report.responsive_members,
            health_report.total_members,
            health_report.total_shards_found,
            if health_report.reconstruction_possible { "possible" } else { "impossible" }
        );

        Ok(health_report)
    }

    /// Get status of active distributions
    pub async fn get_distribution_status(&self, distribution_id: &str) -> Option<DistributionStatus> {
        let active = self.active_distributions.read().await;
        active.get(distribution_id).cloned()
    }

    // Private helper methods

    async fn distribute_shard_to_member(
        &self,
        member_id: String,
        shard: Shard,
        _distribution_id: String,
    ) -> Result<()> {
        debug!(
            "Distributing shard {} to member {} for group {}",
            shard.index, member_id, shard.group_id
        );

        let message = ShardMessage::StoreShardRequest {
            shard: shard.clone(),
            group_id: shard.group_id.clone(),
            sender_id: "self".to_string(), // TODO: Get actual node ID
        };

        // Serialize message for network transmission
        let message_data = serde_json::to_vec(&message)
            .context("Failed to serialize shard storage message")?;

        // Send via DHT with retry logic
        for attempt in 0..self.max_retries {
            match self.send_message_to_member(&member_id, &message_data).await {
                Ok(response_data) => {
                    // Parse response
                    let response: ShardMessage = serde_json::from_slice(&response_data)
                        .context("Failed to parse shard storage response")?;

                    match response {
                        ShardMessage::StoreShardResponse { success, message, .. } => {
                            if success {
                                debug!("Successfully stored shard {} with member {}", shard.index, member_id);
                                return Ok(());
                            } else {
                                bail!("Member {} rejected shard storage: {}", member_id, message);
                            }
                        }
                        _ => bail!("Unexpected response type from member {}", member_id),
                    }
                }
                Err(e) => {
                    warn!(
                        "Attempt {}/{} failed to send shard to {}: {}",
                        attempt + 1, self.max_retries, member_id, e
                    );
                    
                    if attempt == self.max_retries - 1 {
                        return Err(e);
                    }
                    
                    // Exponential backoff
                    let delay = Duration::from_millis(100 * (1 << attempt));
                    tokio::time::sleep(delay).await;
                }
            }
        }

        unreachable!()
    }

    async fn request_shards_from_member(
        &self,
        member_id: String,
        group_id: String,
        data_id: String,
    ) -> Result<Vec<Shard>> {
        debug!("Requesting shards from member {} for group {}", member_id, group_id);

        let message = ShardMessage::RetrieveShardRequest {
            group_id: group_id.clone(),
            data_id: data_id.clone(),
            shard_index: usize::MAX, // Request all shards
            requester_id: "self".to_string(), // TODO: Get actual node ID
        };

        let message_data = serde_json::to_vec(&message)
            .context("Failed to serialize shard retrieval message")?;

        let response_data = self.send_message_to_member(&member_id, &message_data).await?;
        
        let response: ShardMessage = serde_json::from_slice(&response_data)
            .context("Failed to parse shard retrieval response")?;

        match response {
            ShardMessage::RetrieveShardResponse { shard, success, message } => {
                if success {
                    Ok(shard.into_iter().collect())
                } else {
                    bail!("Member {} failed to provide shards: {}", member_id, message);
                }
            }
            _ => bail!("Unexpected response type from member {}", member_id),
        }
    }

    async fn check_member_shard_health(
        &self,
        member_id: String,
        group_id: String,
        data_id: String,
    ) -> Result<MemberShardHealth> {
        let message = ShardMessage::ShardHealthCheck {
            group_id: group_id.clone(),
            data_id: data_id.clone(),
            shard_indices: vec![], // Check all shards
        };

        let message_data = serde_json::to_vec(&message)
            .context("Failed to serialize health check message")?;

        let response_data = self.send_message_to_member(&member_id, &message_data).await?;
        
        let response: ShardMessage = serde_json::from_slice(&response_data)
            .context("Failed to parse health check response")?;

        match response {
            ShardMessage::ShardHealthResponse { available_shards, corrupted_shards } => {
                Ok(MemberShardHealth {
                    member_id,
                    available_shards,
                    corrupted_shards,
                })
            }
            _ => bail!("Unexpected response type from member"),
        }
    }

    async fn send_message_to_member(
        &self,
        member_id: &str,
        message_data: &[u8],
    ) -> Result<Vec<u8>> {
        // TODO: Implement proper DHT messaging once send_message method is available
        debug!("Would send message to member {}: {} bytes", member_id, message_data.len());
        Ok(vec![]) // Placeholder response
    }

    fn deduplicate_shards(&self, shards: Vec<Shard>) -> Vec<Shard> {
        let mut seen_indices = HashSet::new();
        let mut unique_shards = Vec::new();

        for shard in shards {
            if seen_indices.insert(shard.index) {
                unique_shards.push(shard);
            }
        }

        unique_shards
    }
}

#[derive(Debug, Clone)]
pub struct ShardHealthReport {
    pub group_id: String,
    pub data_id: String,
    pub total_members: usize,
    pub responsive_members: usize,
    pub total_shards_found: usize,
    pub corrupted_shards: HashSet<usize>,
    pub missing_members: Vec<String>,
    pub reconstruction_possible: bool,
}

#[derive(Debug, Clone)]
pub struct MemberShardHealth {
    pub member_id: String,
    pub available_shards: Vec<usize>,
    pub corrupted_shards: Vec<usize>,
}