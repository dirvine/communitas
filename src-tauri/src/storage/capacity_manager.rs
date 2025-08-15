// Copyright 2024 Saorsa Labs
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Capacity management for the 1:1:2 storage allocation policy

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use super::{StorageAllocation, StorageUsage};

/// Manages storage capacity according to the 1:1:2 allocation policy
/// Personal:DHT:Public = 1:1:2 ratio
#[derive(Debug)]
pub struct CapacityManager {
    allocation: StorageAllocation,
    current_usage: Arc<RwLock<StorageUsage>>,
    safety_margin: f32, // Percentage to keep free (default 10%)
}

impl CapacityManager {
    pub fn new(allocation: StorageAllocation) -> Self {
        Self {
            allocation,
            current_usage: Arc::new(RwLock::new(StorageUsage {
                personal_local: 0,
                personal_dht: 0,
                group_shards: 0,
                public_dht_used: 0,
                last_updated: chrono::Utc::now(),
            })),
            safety_margin: 0.10, // 10% safety margin
        }
    }

    /// Check if we can store personal data (requires local + DHT space)
    pub async fn can_store_personal(&self, size: usize) -> bool {
        let usage = self.current_usage.read().await;
        let available_personal = self.available_personal_capacity(&usage);
        
        // Need space for both local and DHT copy
        let required_space = size * 2;
        
        debug!(
            "Personal storage check: need {} bytes, available {} bytes", 
            required_space, available_personal
        );
        
        available_personal >= required_space
    }

    /// Check if we can accept a group shard
    pub async fn can_accept_group_shard(&self, shard_size: usize) -> bool {
        let usage = self.current_usage.read().await;
        let available_shard_space = self.available_group_shard_capacity(&usage);
        
        debug!(
            "Group shard check: need {} bytes, available {} bytes", 
            shard_size, available_shard_space
        );
        
        available_shard_space >= shard_size
    }

    /// Check if we can accept DHT data from other nodes
    pub async fn can_accept_dht_data(&self, data_size: usize) -> bool {
        let usage = self.current_usage.read().await;
        let available_dht_space = self.available_public_dht_capacity(&usage);
        
        debug!(
            "DHT storage check: need {} bytes, available {} bytes", 
            data_size, available_dht_space
        );
        
        available_dht_space >= data_size
    }

    /// Update usage statistics
    pub async fn update_usage(&self, usage_update: StorageUsageUpdate) {
        let mut usage = self.current_usage.write().await;
        
        match usage_update {
            StorageUsageUpdate::PersonalStored { size } => {
                usage.personal_local += size;
                usage.personal_dht += size; // Replicated
            }
            StorageUsageUpdate::GroupShardStored { size } => {
                usage.group_shards += size;
            }
            StorageUsageUpdate::DHTDataStored { size } => {
                usage.public_dht_used += size;
            }
            StorageUsageUpdate::PersonalRemoved { size } => {
                usage.personal_local = usage.personal_local.saturating_sub(size);
                usage.personal_dht = usage.personal_dht.saturating_sub(size);
            }
            StorageUsageUpdate::GroupShardRemoved { size } => {
                usage.group_shards = usage.group_shards.saturating_sub(size);
            }
            StorageUsageUpdate::DHTDataRemoved { size } => {
                usage.public_dht_used = usage.public_dht_used.saturating_sub(size);
            }
        }
        
        usage.last_updated = chrono::Utc::now();
        
        // Log warnings if approaching capacity limits
        self.check_capacity_warnings(&usage);
    }

    /// Get current capacity status
    pub async fn get_capacity_status(&self) -> CapacityStatus {
        let usage = self.current_usage.read().await;
        
        CapacityStatus {
            allocation: self.allocation.clone(),
            usage: usage.clone(),
            personal_utilization: self.calculate_personal_utilization(&usage),
            group_shard_utilization: self.calculate_group_shard_utilization(&usage),
            dht_utilization: self.calculate_dht_utilization(&usage),
            overall_utilization: self.calculate_overall_utilization(&usage),
            is_healthy: self.is_healthy(&usage),
            recommendations: self.generate_recommendations(&usage),
        }
    }

    /// Calculate storage efficiency metrics
    pub async fn get_efficiency_metrics(&self) -> EfficiencyMetrics {
        let usage = self.current_usage.read().await;
        
        // Calculate Reed Solomon efficiency
        let total_personal_and_groups = usage.personal_local + usage.group_shards;
        let total_capacity_used = total_personal_and_groups + usage.public_dht_used;
        
        let storage_efficiency = if self.allocation.total_capacity > 0 {
            (total_capacity_used as f32 / self.allocation.total_capacity as f32) * 100.0
        } else {
            0.0
        };

        // Calculate DHT participation ratio
        let expected_dht_participation = self.allocation.public_dht_allocation;
        let actual_dht_participation = usage.public_dht_used;
        let dht_participation_ratio = if expected_dht_participation > 0 {
            (actual_dht_participation as f32 / expected_dht_participation as f32) * 100.0
        } else {
            0.0
        };

        EfficiencyMetrics {
            storage_efficiency_percent: storage_efficiency,
            dht_participation_ratio_percent: dht_participation_ratio,
            reed_solomon_overhead_percent: self.calculate_reed_solomon_overhead(),
            deduplication_savings_percent: 0.0, // TODO: Implement deduplication
            compression_ratio: 1.0, // TODO: Implement compression
        }
    }

    // Private helper methods

    fn available_personal_capacity(&self, usage: &StorageUsage) -> usize {
        let used = usage.personal_local;
        let allocated = self.allocation.personal_local;
        let safety_reserve = (allocated as f32 * self.safety_margin) as usize;
        
        allocated.saturating_sub(used).saturating_sub(safety_reserve)
    }

    fn available_group_shard_capacity(&self, usage: &StorageUsage) -> usize {
        let used = usage.group_shards;
        let allocated = self.allocation.group_shard_allocation;
        let safety_reserve = (allocated as f32 * self.safety_margin) as usize;
        
        allocated.saturating_sub(used).saturating_sub(safety_reserve)
    }

    fn available_public_dht_capacity(&self, usage: &StorageUsage) -> usize {
        let used = usage.public_dht_used;
        let allocated = self.allocation.public_dht_allocation;
        let safety_reserve = (allocated as f32 * self.safety_margin) as usize;
        
        allocated.saturating_sub(used).saturating_sub(safety_reserve)
    }

    fn calculate_personal_utilization(&self, usage: &StorageUsage) -> f32 {
        if self.allocation.personal_local == 0 {
            return 0.0;
        }
        (usage.personal_local as f32 / self.allocation.personal_local as f32) * 100.0
    }

    fn calculate_group_shard_utilization(&self, usage: &StorageUsage) -> f32 {
        if self.allocation.group_shard_allocation == 0 {
            return 0.0;
        }
        (usage.group_shards as f32 / self.allocation.group_shard_allocation as f32) * 100.0
    }

    fn calculate_dht_utilization(&self, usage: &StorageUsage) -> f32 {
        if self.allocation.public_dht_allocation == 0 {
            return 0.0;
        }
        (usage.public_dht_used as f32 / self.allocation.public_dht_allocation as f32) * 100.0
    }

    fn calculate_overall_utilization(&self, usage: &StorageUsage) -> f32 {
        let total_used = usage.personal_local + usage.group_shards + usage.public_dht_used;
        if self.allocation.total_capacity == 0 {
            return 0.0;
        }
        (total_used as f32 / self.allocation.total_capacity as f32) * 100.0
    }

    fn is_healthy(&self, usage: &StorageUsage) -> bool {
        let overall_utilization = self.calculate_overall_utilization(usage);
        let personal_utilization = self.calculate_personal_utilization(usage);
        let group_utilization = self.calculate_group_shard_utilization(usage);
        let dht_utilization = self.calculate_dht_utilization(usage);
        
        // Consider healthy if no category exceeds 90%
        overall_utilization < 90.0 && 
        personal_utilization < 90.0 && 
        group_utilization < 90.0 && 
        dht_utilization < 90.0
    }

    fn check_capacity_warnings(&self, usage: &StorageUsage) {
        let personal_utilization = self.calculate_personal_utilization(usage);
        let group_utilization = self.calculate_group_shard_utilization(usage);
        let dht_utilization = self.calculate_dht_utilization(usage);

        if personal_utilization > 80.0 {
            warn!("Personal storage utilization high: {:.1}%", personal_utilization);
        }
        if group_utilization > 80.0 {
            warn!("Group shard storage utilization high: {:.1}%", group_utilization);
        }
        if dht_utilization > 80.0 {
            warn!("DHT participation storage utilization high: {:.1}%", dht_utilization);
        }
    }

    fn generate_recommendations(&self, usage: &StorageUsage) -> Vec<String> {
        let mut recommendations = Vec::new();
        let personal_util = self.calculate_personal_utilization(usage);
        let group_util = self.calculate_group_shard_utilization(usage);
        let dht_util = self.calculate_dht_utilization(usage);

        if personal_util > 85.0 {
            recommendations.push("Consider cleaning up old personal data".to_string());
            recommendations.push("Archive infrequently accessed files".to_string());
        }

        if group_util > 85.0 {
            recommendations.push("Review group membership - some shards may be for inactive groups".to_string());
        }

        if dht_util < 20.0 {
            recommendations.push("Low DHT participation - consider accepting more public storage requests".to_string());
        }

        if dht_util > 95.0 {
            recommendations.push("DHT storage nearly full - may need to reject new storage requests".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Storage utilization is healthy".to_string());
        }

        recommendations
    }

    fn calculate_reed_solomon_overhead(&self) -> f32 {
        // With k=8, m=4 Reed Solomon, overhead is m/k = 4/8 = 50%
        50.0
    }
}

#[derive(Debug, Clone)]
pub enum StorageUsageUpdate {
    PersonalStored { size: usize },
    GroupShardStored { size: usize },
    DHTDataStored { size: usize },
    PersonalRemoved { size: usize },
    GroupShardRemoved { size: usize },
    DHTDataRemoved { size: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityStatus {
    pub allocation: StorageAllocation,
    pub usage: StorageUsage,
    pub personal_utilization: f32,
    pub group_shard_utilization: f32,
    pub dht_utilization: f32,
    pub overall_utilization: f32,
    pub is_healthy: bool,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub storage_efficiency_percent: f32,
    pub dht_participation_ratio_percent: f32,
    pub reed_solomon_overhead_percent: f32,
    pub deduplication_savings_percent: f32,
    pub compression_ratio: f32,
}