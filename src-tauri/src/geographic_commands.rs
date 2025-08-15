// Copyright (c) 2025 Saorsa Labs Limited
//
// Geographic Routing Tauri Commands
//
// This module provides Tauri commands for controlling and monitoring
// geographic routing in the Communitas application.

use crate::geographic_routing::{
    GeographicRoutingManager, GeographicPeerInfo, GeographicNetworkOverview,
    GeographicStatus, RegionalPeerStats,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Get geographic routing status
#[tauri::command]
pub async fn get_geographic_status(
    manager: tauri::State<'_, Arc<RwLock<Option<GeographicRoutingManager>>>>,
) -> Result<GeographicStatus, String> {
    let manager_guard = manager.read().await;
    
    if let Some(geo_manager) = manager_guard.as_ref() {
        let overview = geo_manager.get_network_overview().await
            .map_err(|e| format!("Failed to get overview: {}", e))?;
        
        let cross_region_ratio = if overview.total_peers > 0 {
            overview.cross_region_connections as f64 / overview.total_peers as f64
        } else {
            0.0
        };
        
        let avg_latency = if !overview.avg_regional_latency.is_empty() {
            overview.avg_regional_latency.values().sum::<f64>() / 
            overview.avg_regional_latency.len() as f64
        } else {
            0.0
        };
        
        Ok(GeographicStatus {
            enabled: true,
            local_region: overview.local_region,
            peer_count: overview.total_peers,
            regions_covered: overview.geographic_coverage,
            cross_region_ratio,
            avg_latency_ms: avg_latency,
            routing_efficiency: overview.routing_efficiency,
        })
    } else {
        Ok(GeographicStatus {
            enabled: false,
            local_region: "Unknown".to_string(),
            peer_count: 0,
            regions_covered: vec![],
            cross_region_ratio: 0.0,
            avg_latency_ms: 0.0,
            routing_efficiency: 0.0,
        })
    }
}

/// Get all peers with geographic information
#[tauri::command]
pub async fn get_geographic_peers(
    manager: tauri::State<'_, Arc<RwLock<Option<GeographicRoutingManager>>>>,
) -> Result<Vec<GeographicPeerInfo>, String> {
    let manager_guard = manager.read().await;
    
    if let Some(geo_manager) = manager_guard.as_ref() {
        Ok(geo_manager.get_all_peers().await)
    } else {
        Ok(vec![])
    }
}

/// Get geographic network overview
#[tauri::command]
pub async fn get_geographic_overview(
    manager: tauri::State<'_, Arc<RwLock<Option<GeographicRoutingManager>>>>,
) -> Result<GeographicNetworkOverview, String> {
    let manager_guard = manager.read().await;
    
    if let Some(geo_manager) = manager_guard.as_ref() {
        geo_manager.get_network_overview().await
            .map_err(|e| format!("Failed to get overview: {}", e))
    } else {
        Err("Geographic routing not initialized".to_string())
    }
}

/// Get peers by region
#[tauri::command]
pub async fn get_peers_by_region(
    region: String,
    manager: tauri::State<'_, Arc<RwLock<Option<GeographicRoutingManager>>>>,
) -> Result<Vec<GeographicPeerInfo>, String> {
    let manager_guard = manager.read().await;
    
    if let Some(geo_manager) = manager_guard.as_ref() {
        geo_manager.get_peers_by_region(&region).await
            .map_err(|e| format!("Failed to get peers: {}", e))
    } else {
        Ok(vec![])
    }
}

/// Get regional statistics
#[tauri::command]
pub async fn get_regional_stats(
    manager: tauri::State<'_, Arc<RwLock<Option<GeographicRoutingManager>>>>,
) -> Result<Vec<RegionalPeerStats>, String> {
    let manager_guard = manager.read().await;
    
    if let Some(geo_manager) = manager_guard.as_ref() {
        let overview = geo_manager.get_network_overview().await
            .map_err(|e| format!("Failed to get overview: {}", e))?;
        
        let mut stats = Vec::new();
        
        for (region, count) in overview.regional_distribution {
            let avg_latency = overview.avg_regional_latency.get(&region).copied();
            
            // Calculate average reliability for the region
            let peers = geo_manager.get_peers_by_region(&region).await
                .unwrap_or_default();
            
            let avg_reliability = if !peers.is_empty() {
                peers.iter().map(|p| p.reliability_score).sum::<f64>() / peers.len() as f64
            } else {
                0.0
            };
            
            stats.push(RegionalPeerStats {
                region: region.clone(),
                peer_count: count,
                avg_latency_ms: avg_latency,
                avg_reliability,
                is_local_region: region == overview.local_region,
            });
        }
        
        // Sort by peer count
        stats.sort_by(|a, b| b.peer_count.cmp(&a.peer_count));
        
        Ok(stats)
    } else {
        Ok(vec![])
    }
}

/// Update peer quality metrics
#[tauri::command]
pub async fn update_peer_quality(
    peer_id: String,
    success: bool,
    latency_ms: Option<f64>,
    manager: tauri::State<'_, Arc<RwLock<Option<GeographicRoutingManager>>>>,
) -> Result<(), String> {
    let manager_guard = manager.read().await;
    
    if let Some(geo_manager) = manager_guard.as_ref() {
        geo_manager.update_peer_quality(&peer_id, success, latency_ms).await
            .map_err(|e| format!("Failed to update peer quality: {}", e))
    } else {
        Err("Geographic routing not initialized".to_string())
    }
}

/// Select optimal peers for operation
#[tauri::command]
pub async fn select_optimal_peers(
    operation: String,
    count: usize,
    manager: tauri::State<'_, Arc<RwLock<Option<GeographicRoutingManager>>>>,
) -> Result<Vec<String>, String> {
    let manager_guard = manager.read().await;
    
    if let Some(geo_manager) = manager_guard.as_ref() {
        geo_manager.select_peers_for_operation(&operation, count).await
            .map_err(|e| format!("Failed to select peers: {}", e))
    } else {
        Ok(vec![])
    }
}

/// Configuration for geographic routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicConfig {
    pub enabled: bool,
    pub local_region: Option<String>,
    pub latency_weight: f64,
    pub reliability_weight: f64,
    pub region_preference_weight: f64,
    pub max_cross_region_ratio: f64,
}

/// Update geographic routing configuration
#[tauri::command]
pub async fn update_geographic_config(
    config: GeographicConfig,
    manager: tauri::State<'_, Arc<RwLock<Option<GeographicRoutingManager>>>>,
) -> Result<(), String> {
    info!("Updating geographic routing configuration: {:?}", config);
    
    if config.enabled {
        let mut manager_guard = manager.write().await;
        
        if manager_guard.is_none() {
            // Initialize new manager with configuration
            let local_region = config.local_region
                .and_then(|r| match r.as_str() {
                    "NorthAmerica" => Some(saorsa_core::dht::geographic_routing::GeographicRegion::NorthAmerica),
                    "Europe" => Some(saorsa_core::dht::geographic_routing::GeographicRegion::Europe),
                    "AsiaPacific" => Some(saorsa_core::dht::geographic_routing::GeographicRegion::AsiaPacific),
                    "SouthAmerica" => Some(saorsa_core::dht::geographic_routing::GeographicRegion::SouthAmerica),
                    "Africa" => Some(saorsa_core::dht::geographic_routing::GeographicRegion::Africa),
                    "Oceania" => Some(saorsa_core::dht::geographic_routing::GeographicRegion::Oceania),
                    _ => None,
                });
            
            let mut new_manager = GeographicRoutingManager::new(local_region).await
                .map_err(|e| format!("Failed to create manager: {}", e))?;
            
            new_manager.start().await
                .map_err(|e| format!("Failed to start manager: {}", e))?;
            
            *manager_guard = Some(new_manager);
            
            info!("Geographic routing enabled");
        }
    } else {
        let mut manager_guard = manager.write().await;
        
        if let Some(mut geo_manager) = manager_guard.take() {
            geo_manager.shutdown().await
                .map_err(|e| format!("Failed to shutdown: {}", e))?;
            
            info!("Geographic routing disabled");
        }
    }
    
    Ok(())
}

/// Test geographic routing connectivity
#[tauri::command]
pub async fn test_geographic_connectivity(
    target_ip: String,
    _manager: tauri::State<'_, Arc<RwLock<Option<GeographicRoutingManager>>>>,
) -> Result<serde_json::Value, String> {
    use std::net::{IpAddr, SocketAddr, TcpStream};
    use std::time::{Duration, Instant};
    
    debug!("Testing connectivity to {}", target_ip);
    
    // Parse IP address
    let ip: IpAddr = target_ip.parse()
        .map_err(|e| format!("Invalid IP: {}", e))?;
    
    // Test common P2P ports
    let test_ports = vec![22, 9001, 9110, 9120];
    let mut results = Vec::new();
    
    for port in test_ports {
        let socket_addr = SocketAddr::new(ip, port);
        let start = Instant::now();
        
        let (status, latency_ms) = match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5)) {
            Ok(_) => {
                let latency = start.elapsed();
                ("connected", Some(latency.as_millis() as f64))
            }
            Err(_) => ("failed", None),
        };
        
        results.push(serde_json::json!({
            "port": port,
            "status": status,
            "latency_ms": latency_ms,
        }));
    }
    
    // Detect region
    let region = match ip {
        IpAddr::V4(ipv4) if ipv4.octets()[0] == 159 && ipv4.octets()[1] == 89 => "Europe",
        _ => "Unknown",
    };
    
    Ok(serde_json::json!({
        "ip": target_ip,
        "region": region,
        "tests": results,
        "timestamp": SystemTime::now(),
    }))
}