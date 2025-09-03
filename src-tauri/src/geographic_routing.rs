// Copyright (c) 2025 Saorsa Labs Limited
//
// Geographic Routing Integration for Communitas
//
// This module integrates the geographic-aware DHT routing capabilities
// into the Communitas P2P application.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// Import geographic routing components from saorsa-core
use saorsa_core::{
    Multiaddr,
    dht::{
        geographic_network_integration::{DhtOperationType, GeographicNetworkIntegration},
        geographic_routing::GeographicRegion,
    },
    geographic_enhanced_network::{
        GeographicNetworkConfig, GeographicNetworkService, GeographicNetworkStats,
    },
};

/// Geographic peer information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicPeerInfo {
    pub peer_id: String,
    pub address: String,
    pub region: String,
    pub country: Option<String>,
    pub city: Option<String>,
    pub latency_ms: Option<f64>,
    pub reliability_score: f64,
    pub last_seen: SystemTime,
    pub connection_quality: String,
    pub is_bootstrap: bool,
    pub cross_region: bool,
}

/// Geographic network overview for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicNetworkOverview {
    pub local_region: String,
    pub total_peers: usize,
    pub regional_distribution: HashMap<String, usize>,
    pub cross_region_connections: usize,
    pub avg_regional_latency: HashMap<String, f64>,
    pub routing_efficiency: f64,
    pub geographic_coverage: Vec<String>,
}

/// Geographic routing manager for Communitas
pub struct GeographicRoutingManager {
    /// Geographic network service
    network_service: Arc<RwLock<GeographicNetworkService>>,
    /// Geographic network integration
    network_integration: Arc<GeographicNetworkIntegration>,
    /// Local node region
    local_region: GeographicRegion,
    /// Peer geographic information cache
    peer_info_cache: Arc<RwLock<HashMap<String, GeographicPeerInfo>>>,
    /// Statistics
    stats: Arc<RwLock<GeographicNetworkStats>>,
}

impl GeographicRoutingManager {
    /// Create new geographic routing manager
    pub async fn new(local_region: Option<GeographicRegion>) -> Result<Self> {
        // Auto-detect region if not provided
        let region = local_region.unwrap_or_else(|| Self::auto_detect_region());

        info!(
            "Initializing Geographic Routing Manager for region: {:?}",
            region
        );

        // Create configuration with optimized settings
        let config = GeographicNetworkConfig {
            local_region: region,
            enable_geographic_routing: true,
            latency_weight: 0.4,
            reliability_weight: 0.3,
            region_preference_weight: 0.3,
            max_cross_region_ratio: 0.3,
            maintenance_interval: Duration::from_secs(300),
        };

        // Initialize services
        let network_service = Arc::new(RwLock::new(
            GeographicNetworkService::new(config.clone()).await?,
        ));

        let network_integration = Arc::new(GeographicNetworkIntegration::new(region)?);

        Ok(Self {
            network_service,
            network_integration,
            local_region: region,
            peer_info_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(GeographicNetworkStats::default())),
        })
    }

    /// Auto-detect geographic region based on system locale and network
    fn auto_detect_region() -> GeographicRegion {
        // Try to detect from system locale
        if let Ok(locale) = std::env::var("LANG") {
            if locale.contains("US") || locale.contains("CA") {
                return GeographicRegion::NorthAmerica;
            } else if locale.contains("GB") || locale.contains("DE") || locale.contains("FR") {
                return GeographicRegion::Europe;
            } else if locale.contains("CN") || locale.contains("JP") || locale.contains("KR") {
                return GeographicRegion::AsiaPacific;
            }
        }

        // Default to Unknown and let network detection refine it
        GeographicRegion::Unknown
    }

    /// Start geographic routing services
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Geographic Routing Services");

        // Start the network service
        self.network_service.write().await.start().await?;

        // Start maintenance task
        self.start_maintenance_task().await?;

        info!("Geographic Routing Services started successfully");
        Ok(())
    }

    /// Add a peer with geographic awareness
    pub async fn add_peer(&self, peer_id: String, address: String) -> Result<()> {
        debug!(
            "Adding peer {} at {} to geographic routing",
            peer_id, address
        );

        // Parse address to Multiaddr
        let multiaddr: Multiaddr = address.parse()?;

        // Add to network service
        self.network_service
            .write()
            .await
            .add_peer(peer_id.clone(), multiaddr.clone())
            .await?;

        // Detect region and cache peer info
        let region = self.network_integration.detect_region(&multiaddr).await;

        let peer_info = GeographicPeerInfo {
            peer_id: peer_id.clone(),
            address: address.clone(),
            region: format!("{:?}", region),
            country: Self::detect_country(&address),
            city: Self::detect_city(&address),
            latency_ms: None,
            reliability_score: 0.5,
            last_seen: SystemTime::now(),
            connection_quality: "Good".to_string(),
            is_bootstrap: address.contains("159.89.81.21"),
            cross_region: region != self.local_region,
        };

        self.peer_info_cache
            .write()
            .await
            .insert(peer_id, peer_info);

        Ok(())
    }

    /// Update peer quality metrics
    pub async fn update_peer_quality(
        &self,
        peer_id: &str,
        success: bool,
        latency_ms: Option<f64>,
    ) -> Result<()> {
        // Update network service
        let latency = latency_ms.map(|ms| Duration::from_millis(ms as u64));
        self.network_service
            .write()
            .await
            .update_peer_quality(peer_id, success, latency)
            .await?;

        // Update cache
        if let Some(peer_info) = self.peer_info_cache.write().await.get_mut(peer_id) {
            if let Some(ms) = latency_ms {
                peer_info.latency_ms = Some(ms);
            }

            // Update reliability score
            if success {
                peer_info.reliability_score = (peer_info.reliability_score * 0.9 + 0.1).min(1.0);
            } else {
                peer_info.reliability_score = (peer_info.reliability_score * 0.9).max(0.1);
            }

            // Update connection quality based on latency and reliability
            peer_info.connection_quality =
                match (peer_info.latency_ms, peer_info.reliability_score) {
                    (Some(lat), rel) if lat < 50.0 && rel > 0.8 => "Excellent",
                    (Some(lat), rel) if lat < 100.0 && rel > 0.6 => "Good",
                    (Some(lat), rel) if lat < 200.0 && rel > 0.4 => "Fair",
                    _ => "Poor",
                }
                .to_string();

            peer_info.last_seen = SystemTime::now();
        }

        Ok(())
    }

    /// Get geographic peer information for UI
    pub async fn get_peer_info(&self, peer_id: &str) -> Option<GeographicPeerInfo> {
        self.peer_info_cache.read().await.get(peer_id).cloned()
    }

    /// Get all peers with geographic information
    pub async fn get_all_peers(&self) -> Vec<GeographicPeerInfo> {
        self.peer_info_cache
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Get geographic network overview
    pub async fn get_network_overview(&self) -> Result<GeographicNetworkOverview> {
        let stats = self.network_service.read().await.get_stats().await?;
        let peers = self.peer_info_cache.read().await;

        // Calculate regional distribution
        let mut regional_distribution = HashMap::new();
        let mut avg_regional_latency = HashMap::new();
        let mut cross_region_connections = 0;

        for peer in peers.values() {
            *regional_distribution
                .entry(peer.region.clone())
                .or_insert(0) += 1;

            if peer.cross_region {
                cross_region_connections += 1;
            }

            if let Some(latency) = peer.latency_ms {
                let latencies = avg_regional_latency
                    .entry(peer.region.clone())
                    .or_insert_with(Vec::new);
                latencies.push(latency);
            }
        }

        // Calculate average latencies
        let avg_regional_latency: HashMap<String, f64> = avg_regional_latency
            .into_iter()
            .map(|(region, latencies)| {
                let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
                (region, avg)
            })
            .collect();

        // Calculate routing efficiency
        let routing_efficiency = if stats.successful_connections > 0 {
            stats.successful_connections as f64
                / (stats.successful_connections + stats.failed_connections) as f64
        } else {
            0.0
        };

        Ok(GeographicNetworkOverview {
            local_region: format!("{:?}", self.local_region),
            total_peers: peers.len(),
            regional_distribution,
            cross_region_connections,
            avg_regional_latency,
            routing_efficiency,
            geographic_coverage: peers
                .values()
                .map(|p| p.region.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
        })
    }

    /// Select optimal peers for DHT operation
    pub async fn select_peers_for_operation(
        &self,
        operation: &str,
        count: usize,
    ) -> Result<Vec<String>> {
        let op_type = match operation {
            "store" => DhtOperationType::Store,
            "retrieve" => DhtOperationType::Retrieve,
            "find_node" => DhtOperationType::FindNode,
            _ => DhtOperationType::Ping,
        };

        let target_key = vec![0u8; 32]; // Placeholder for actual key

        self.network_service
            .read()
            .await
            .select_peers_for_operation(&target_key, op_type, count)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to select peers: {}", e))
    }

    /// Get peers by geographic region
    pub async fn get_peers_by_region(&self, region: &str) -> Result<Vec<GeographicPeerInfo>> {
        let peers = self.peer_info_cache.read().await;
        Ok(peers
            .values()
            .filter(|p| p.region == region)
            .cloned()
            .collect())
    }

    /// Detect country from address (simplified)
    fn detect_country(address: &str) -> Option<String> {
        if address.contains("159.89.81.21") {
            Some("Netherlands".to_string()) // DigitalOcean EU
        } else {
            None
        }
    }

    /// Detect city from address (simplified)
    fn detect_city(address: &str) -> Option<String> {
        if address.contains("159.89.81.21") {
            Some("Amsterdam".to_string()) // DigitalOcean EU
        } else {
            None
        }
    }

    /// Start background maintenance task
    async fn start_maintenance_task(&self) -> Result<()> {
        let cache = Arc::clone(&self.peer_info_cache);
        let integration = Arc::clone(&self.network_integration);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Clean up stale peers
                let now = SystemTime::now();
                let mut cache_guard = cache.write().await;
                cache_guard.retain(|_, peer| {
                    now.duration_since(peer.last_seen)
                        .map(|d| d < Duration::from_secs(600))
                        .unwrap_or(false)
                });

                // Perform network maintenance
                if let Err(e) = integration.perform_maintenance().await {
                    warn!("Geographic routing maintenance error: {}", e);
                }

                debug!("Geographic routing maintenance completed");
            }
        });

        Ok(())
    }

    /// Shutdown geographic routing services
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Geographic Routing Services");
        self.network_service.write().await.shutdown().await?;
        Ok(())
    }
}

/// Tauri command responses
#[derive(Debug, Serialize, Deserialize)]
pub struct GeographicStatus {
    pub enabled: bool,
    pub local_region: String,
    pub peer_count: usize,
    pub regions_covered: Vec<String>,
    pub cross_region_ratio: f64,
    pub avg_latency_ms: f64,
    pub routing_efficiency: f64,
}

impl std::fmt::Debug for GeographicRoutingManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeographicRoutingManager")
            .field("local_region", &self.local_region)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegionalPeerStats {
    pub region: String,
    pub peer_count: usize,
    pub avg_latency_ms: Option<f64>,
    pub avg_reliability: f64,
    pub is_local_region: bool,
}
