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


//! Diagnostics engine implementation

use super::{NetworkHealth, NetworkMetrics, PeerMetrics, StorageMetrics};
use crate::network::NetworkIntegration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

/// Main diagnostics engine
pub struct DiagnosticsEngine {
    /// Network integration
    network: Arc<NetworkIntegration>,
    /// Metrics storage
    network_metrics: Arc<RwLock<NetworkMetrics>>,
    storage_metrics: Arc<RwLock<StorageMetrics>>,
    peer_metrics: Arc<RwLock<PeerMetrics>>,
    /// Collection state
    collecting: AtomicBool,
}

impl std::fmt::Debug for DiagnosticsEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiagnosticsEngine")
            .field("network", &"Arc<NetworkIntegration>")
            .field("network_metrics", &"Arc<RwLock<NetworkMetrics>>")
            .field("storage_metrics", &"Arc<RwLock<StorageMetrics>>")
            .field("peer_metrics", &"Arc<RwLock<PeerMetrics>>")
            .field("collecting", &self.collecting.load(Ordering::Relaxed))
            .finish()
    }
}

impl DiagnosticsEngine {
    /// Create new diagnostics engine
    pub fn new(network: Arc<NetworkIntegration>) -> Self {
        Self {
            network,
            network_metrics: Arc::new(RwLock::new(NetworkMetrics::default())),
            storage_metrics: Arc::new(RwLock::new(StorageMetrics::default())),
            peer_metrics: Arc::new(RwLock::new(PeerMetrics::default())),
            collecting: AtomicBool::new(false),
        }
    }

    /// Start metrics collection
    pub fn start_collection(&self) {
        if self.collecting.swap(true, Ordering::SeqCst) {
            return; // Already collecting
        }

        let network_metrics = self.network_metrics.clone();
        let _storage_metrics = self.storage_metrics.clone();
        let _peer_metrics = self.peer_metrics.clone();
        let network = self.network.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));

            loop {
                interval.tick().await;

                // Update metrics
                if let Ok(stats) = network.get_network_stats().await {
                    let mut metrics = network_metrics.write().await;
                    metrics.update(stats);
                }
            }
        });
    }

    /// Get current network health
    pub async fn get_network_health(&self) -> NetworkHealth {
        let network_metrics = self.network_metrics.read().await;
        let peer_metrics = self.peer_metrics.read().await;

        NetworkHealth {
            status: if peer_metrics.connected_peers > 0 {
                "Connected".to_string()
            } else {
                "Disconnected".to_string()
            },
            peer_count: peer_metrics.connected_peers,
            nat_type: network_metrics.nat_type,
            bandwidth_kbps: network_metrics.bandwidth_usage_kbps,
            avg_latency_ms: network_metrics.avg_latency_ms,
        }
    }
}
