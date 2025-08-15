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


//! Metrics structures

use super::NatType;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Network health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealth {
    /// Connection status
    pub status: String,
    /// Number of connected peers
    pub peer_count: usize,
    /// Detected NAT type
    pub nat_type: NatType,
    /// Current bandwidth usage
    pub bandwidth_kbps: f64,
    /// Average message latency
    pub avg_latency_ms: f64,
}

/// Network metrics
#[derive(Debug, Default)]
pub struct NetworkMetrics {
    /// Connected peers count
    pub connected_peers: usize,
    /// Message latency samples (last 100)
    pub latency_samples: VecDeque<f64>,
    /// Bandwidth usage in KB/s
    pub bandwidth_usage_kbps: f64,
    /// NAT type
    pub nat_type: NatType,
    /// Packet loss rate
    pub packet_loss_rate: f64,
    /// Average latency
    pub avg_latency_ms: f64,
}

/// Storage metrics
#[derive(Debug, Default)]
pub struct StorageMetrics {
    /// Total DHT operations
    pub dht_operations_total: u64,
    /// Successful DHT operations
    pub dht_operations_success: u64,
    /// Average operation latency
    pub avg_dht_latency_ms: f64,
    /// Storage size in bytes
    pub storage_bytes: u64,
    /// Number of stored records
    pub record_count: u64,
    /// Replication factor achieved
    pub avg_replication_factor: f64,
}

/// Peer metrics
#[derive(Debug, Default)]
pub struct PeerMetrics {
    /// Connected peers
    pub connected_peers: usize,
    /// Total peers discovered
    pub total_peers_discovered: usize,
    /// Peer churn rate
    pub churn_rate: f64,
    /// Average peer uptime
    pub avg_peer_uptime_seconds: f64,
}

impl NetworkMetrics {
    /// Update metrics with new stats
    pub fn update(&mut self, stats: NetworkStats) {
        self.connected_peers = stats.connected_peers;
        self.bandwidth_usage_kbps = stats.bandwidth_kbps;

        // Update latency samples
        self.latency_samples.push_back(stats.latest_latency_ms);
        if self.latency_samples.len() > 100 {
            self.latency_samples.pop_front();
        }

        // Calculate average latency
        if !self.latency_samples.is_empty() {
            self.avg_latency_ms =
                self.latency_samples.iter().sum::<f64>() / self.latency_samples.len() as f64;
        }
    }
}

/// Network statistics from the P2P layer
#[derive(Debug)]
pub struct NetworkStats {
    /// Connected peers
    pub connected_peers: usize,
    /// Bandwidth in KB/s
    pub bandwidth_kbps: f64,
    /// Latest latency measurement
    pub latest_latency_ms: f64,
}
