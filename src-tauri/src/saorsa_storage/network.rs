use crate::dht_facade::DhtFacade;
/**
 * Saorsa Storage System - P2P Network Integration
 * Implements DHT operations and geographic routing for content distribution
 */
use crate::saorsa_storage::errors::*;
// use crate::error::AppResult;  // Currently unused
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
// use async_trait::async_trait;  // Currently unused

/// Network operation timeout configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub operation_timeout: Duration,
    pub retry_attempts: u32,
    pub retry_backoff: Duration,
    pub max_concurrent_operations: usize,
    pub enable_geographic_routing: bool,
    pub peer_discovery_interval: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            operation_timeout: Duration::from_secs(30),
            retry_attempts: 3,
            retry_backoff: Duration::from_millis(500),
            max_concurrent_operations: 10,
            enable_geographic_routing: true,
            peer_discovery_interval: Duration::from_secs(60),
        }
    }
}

/// Network peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub location: Option<(f64, f64)>, // (latitude, longitude)
    pub distance_km: Option<f64>,
    pub last_seen: std::time::SystemTime,
    pub response_time_ms: Option<u64>,
    pub reliability_score: f64, // 0.0 to 1.0
    pub available_storage: Option<u64>,
}

/// Network operation result
#[derive(Debug, Clone)]
pub struct NetworkOperation {
    pub operation_id: String,
    pub peer_id: String,
    pub operation_type: String,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub success: bool,
    pub error: Option<String>,
}

/// Network statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub avg_response_time_ms: f64,
    pub active_peers: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub geographic_routing_hits: u64,
}

/// Storage request for DHT operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequest {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub requester_id: String,
    pub ttl: Option<Duration>,
    pub priority: u8, // 0-255, higher = more priority
}

/// Geographic location for routing
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

impl Location {
    /// Calculate distance to another location in kilometers
    pub fn distance_km(&self, other: &Location) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let lat1_rad = self.latitude.to_radians();
        let lat2_rad = other.latitude.to_radians();
        let delta_lat = (other.latitude - self.latitude).to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_KM * c
    }
}

/// Network manager for P2P storage operations
pub struct NetworkManager<D: DhtFacade> {
    dht: Arc<D>,
    config: NetworkConfig,
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    operations: Arc<RwLock<Vec<NetworkOperation>>>,
    stats: Arc<RwLock<NetworkStats>>,
    local_location: Option<Location>,
    operation_semaphore: Arc<tokio::sync::Semaphore>,
}

impl<D: DhtFacade> NetworkManager<D> {
    /// Create a new network manager
    pub fn new(dht: Arc<D>) -> Self {
        let config = NetworkConfig::default();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            config.max_concurrent_operations,
        ));

        Self {
            dht,
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            operations: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(NetworkStats {
                total_operations: 0,
                successful_operations: 0,
                failed_operations: 0,
                avg_response_time_ms: 0.0,
                active_peers: 0,
                total_bytes_sent: 0,
                total_bytes_received: 0,
                geographic_routing_hits: 0,
            })),
            local_location: None,
            operation_semaphore: semaphore,
        }
    }

    /// Create network manager with configuration and location
    pub fn with_config(dht: Arc<D>, config: NetworkConfig, location: Option<Location>) -> Self {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            config.max_concurrent_operations,
        ));

        Self {
            dht,
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            operations: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(NetworkStats {
                total_operations: 0,
                successful_operations: 0,
                failed_operations: 0,
                avg_response_time_ms: 0.0,
                active_peers: 0,
                total_bytes_sent: 0,
                total_bytes_received: 0,
                geographic_routing_hits: 0,
            })),
            local_location: location,
            operation_semaphore: semaphore,
        }
    }

    /// Store content in DHT with retry logic
    pub async fn store_content(&self, key: Vec<u8>, value: Vec<u8>) -> NetworkResult<()> {
        let operation_id = self.generate_operation_id();
        let start_time = Instant::now();

        // Acquire semaphore to limit concurrent operations
        let _permit = self.operation_semaphore.acquire().await.map_err(|_| {
            NetworkError::ConnectionFailed {
                address: "semaphore".to_string(),
                reason: "Failed to acquire operation permit".to_string(),
            }
        })?;

        let mut last_error = None;

        // Retry logic
        for attempt in 0..self.config.retry_attempts {
            match tokio::time::timeout(
                self.config.operation_timeout,
                self.dht.put(key.clone(), value.clone()),
            )
            .await
            {
                Ok(Ok(_)) => {
                    // Success
                    self.record_operation_success(&operation_id, "store", start_time)
                        .await;
                    self.update_stats_success(start_time.elapsed(), value.len())
                        .await;
                    return Ok(());
                }
                Ok(Err(app_error)) => {
                    last_error = Some(NetworkError::ProtocolError {
                        reason: format!("DHT store error: {}", app_error),
                    });
                }
                Err(_) => {
                    last_error = Some(NetworkError::RequestTimeout {
                        timeout: self.config.operation_timeout,
                    });
                }
            }

            // Wait before retry (exponential backoff)
            if attempt < self.config.retry_attempts - 1 {
                let backoff = self.config.retry_backoff * (2_u32.pow(attempt));
                tokio::time::sleep(backoff).await;
            }
        }

        // All retries failed
        let error = last_error.unwrap_or(NetworkError::ProtocolError {
            reason: "Unknown error during store operation".to_string(),
        });

        self.record_operation_failure(&operation_id, "store", start_time, &error)
            .await;
        self.update_stats_failure().await;

        Err(error)
    }

    /// Retrieve content from DHT with retry logic
    pub async fn retrieve_content(&self, key: Vec<u8>) -> NetworkResult<Option<Vec<u8>>> {
        let operation_id = self.generate_operation_id();
        let start_time = Instant::now();

        // Acquire semaphore to limit concurrent operations
        let _permit = self.operation_semaphore.acquire().await.map_err(|_| {
            NetworkError::ConnectionFailed {
                address: "semaphore".to_string(),
                reason: "Failed to acquire operation permit".to_string(),
            }
        })?;

        let mut last_error = None;

        // Retry logic
        for attempt in 0..self.config.retry_attempts {
            match tokio::time::timeout(self.config.operation_timeout, self.dht.get(key.clone()))
                .await
            {
                Ok(Ok(result)) => {
                    // Success
                    self.record_operation_success(&operation_id, "retrieve", start_time)
                        .await;

                    let data_size = result.as_ref().map(|v| v.len()).unwrap_or(0);
                    self.update_stats_success(start_time.elapsed(), data_size)
                        .await;

                    return Ok(result);
                }
                Ok(Err(app_error)) => {
                    last_error = Some(NetworkError::ProtocolError {
                        reason: format!("DHT retrieve error: {}", app_error),
                    });
                }
                Err(_) => {
                    last_error = Some(NetworkError::RequestTimeout {
                        timeout: self.config.operation_timeout,
                    });
                }
            }

            // Wait before retry (exponential backoff)
            if attempt < self.config.retry_attempts - 1 {
                let backoff = self.config.retry_backoff * (2_u32.pow(attempt));
                tokio::time::sleep(backoff).await;
            }
        }

        // All retries failed
        let error = last_error.unwrap_or(NetworkError::ProtocolError {
            reason: "Unknown error during retrieve operation".to_string(),
        });

        self.record_operation_failure(&operation_id, "retrieve", start_time, &error)
            .await;
        self.update_stats_failure().await;

        Err(error)
    }

    /// Send message to specific peer
    pub async fn send_to_peer(
        &self,
        peer_id: &str,
        topic: &str,
        payload: Vec<u8>,
    ) -> NetworkResult<Vec<u8>> {
        let operation_id = self.generate_operation_id();
        let start_time = Instant::now();

        let _permit = self.operation_semaphore.acquire().await.map_err(|_| {
            NetworkError::ConnectionFailed {
                address: peer_id.to_string(),
                reason: "Failed to acquire operation permit".to_string(),
            }
        })?;

        match tokio::time::timeout(
            self.config.operation_timeout,
            self.dht
                .send(peer_id.to_string(), topic.to_string(), payload.clone()),
        )
        .await
        {
            Ok(Ok(response)) => {
                self.record_operation_success(&operation_id, "send", start_time)
                    .await;
                self.update_peer_success(peer_id, start_time.elapsed())
                    .await;
                self.update_stats_success(start_time.elapsed(), payload.len() + response.len())
                    .await;
                Ok(response)
            }
            Ok(Err(app_error)) => {
                let error = NetworkError::PeerRejected {
                    reason: format!("Peer {} rejected request: {}", peer_id, app_error),
                };
                self.record_operation_failure(&operation_id, "send", start_time, &error)
                    .await;
                self.update_peer_failure(peer_id).await;
                self.update_stats_failure().await;
                Err(error)
            }
            Err(_) => {
                let error = NetworkError::RequestTimeout {
                    timeout: self.config.operation_timeout,
                };
                self.record_operation_failure(&operation_id, "send", start_time, &error)
                    .await;
                self.update_peer_failure(peer_id).await;
                self.update_stats_failure().await;
                Err(error)
            }
        }
    }

    /// Discover and update peer list
    pub async fn discover_peers(&self) -> NetworkResult<Vec<String>> {
        match self.dht.peers().await {
            Ok(peer_list) => {
                // Update peer information
                let mut peers = self.peers.write().await;
                let current_time = std::time::SystemTime::now();

                for peer_id in &peer_list {
                    peers
                        .entry(peer_id.clone())
                        .or_insert_with(|| PeerInfo {
                            peer_id: peer_id.clone(),
                            location: None,
                            distance_km: None,
                            last_seen: current_time,
                            response_time_ms: None,
                            reliability_score: 0.5, // Start with neutral score
                            available_storage: None,
                        })
                        .last_seen = current_time;
                }

                // Update stats
                let mut stats = self.stats.write().await;
                stats.active_peers = peer_list.len();

                Ok(peer_list)
            }
            Err(app_error) => Err(NetworkError::ProtocolError {
                reason: format!("Peer discovery failed: {}", app_error),
            }),
        }
    }

    /// Get optimal peers for storage based on location and reliability
    pub async fn get_optimal_peers(&self, count: usize) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        let mut peer_list: Vec<PeerInfo> = peers.values().cloned().collect();

        // Calculate distances if we have location info
        if let Some(local_loc) = self.local_location {
            for peer in &mut peer_list {
                if let Some(peer_loc) = peer.location {
                    let location = Location {
                        latitude: peer_loc.0,
                        longitude: peer_loc.1,
                    };
                    peer.distance_km = Some(local_loc.distance_km(&location));
                }
            }
        }

        // Score peers based on reliability, distance, and response time
        peer_list.sort_by(|a, b| {
            let score_a = self.calculate_peer_score(a);
            let score_b = self.calculate_peer_score(b);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        peer_list.truncate(count);
        peer_list
    }

    /// Update local geographic location
    pub fn set_location(&mut self, location: Location) {
        self.local_location = Some(location);
    }

    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkStats {
        self.stats.read().await.clone()
    }

    /// Get active peer count
    pub async fn get_active_peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Check network health
    pub async fn check_health(&self) -> NetworkResult<bool> {
        let stats = self.stats.read().await;

        // Consider network healthy if:
        // - We have some active peers
        // - Success rate is above 70%
        // - Average response time is reasonable

        let success_rate = if stats.total_operations > 0 {
            stats.successful_operations as f64 / stats.total_operations as f64
        } else {
            1.0 // No operations yet, assume healthy
        };

        let is_healthy =
            stats.active_peers > 0 && success_rate >= 0.7 && stats.avg_response_time_ms < 5000.0; // 5 seconds max

        Ok(is_healthy)
    }

    // Private helper methods

    fn generate_operation_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("op_{}", timestamp)
    }

    async fn record_operation_success(&self, op_id: &str, op_type: &str, start_time: Instant) {
        let operation = NetworkOperation {
            operation_id: op_id.to_string(),
            peer_id: "dht".to_string(), // Generic for DHT operations
            operation_type: op_type.to_string(),
            started_at: start_time,
            completed_at: Some(Instant::now()),
            success: true,
            error: None,
        };

        let mut operations = self.operations.write().await;
        operations.push(operation);

        // Keep only recent operations (last 1000)
        if operations.len() > 1000 {
            operations.drain(0..100); // Remove oldest 100
        }
    }

    async fn record_operation_failure(
        &self,
        op_id: &str,
        op_type: &str,
        start_time: Instant,
        error: &NetworkError,
    ) {
        let operation = NetworkOperation {
            operation_id: op_id.to_string(),
            peer_id: "dht".to_string(),
            operation_type: op_type.to_string(),
            started_at: start_time,
            completed_at: Some(Instant::now()),
            success: false,
            error: Some(error.to_string()),
        };

        let mut operations = self.operations.write().await;
        operations.push(operation);

        // Keep only recent operations
        if operations.len() > 1000 {
            operations.drain(0..100);
        }
    }

    async fn update_stats_success(&self, duration: Duration, bytes_transferred: usize) {
        let mut stats = self.stats.write().await;
        stats.total_operations += 1;
        stats.successful_operations += 1;
        stats.total_bytes_sent += bytes_transferred as u64;

        // Update moving average response time
        let response_time_ms = duration.as_secs_f64() * 1000.0;
        if stats.total_operations == 1 {
            stats.avg_response_time_ms = response_time_ms;
        } else {
            stats.avg_response_time_ms = (stats.avg_response_time_ms
                * (stats.total_operations - 1) as f64
                + response_time_ms)
                / stats.total_operations as f64;
        }
    }

    async fn update_stats_failure(&self) {
        let mut stats = self.stats.write().await;
        stats.total_operations += 1;
        stats.failed_operations += 1;
    }

    async fn update_peer_success(&self, peer_id: &str, response_time: Duration) {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(peer_id) {
            peer.response_time_ms = Some(response_time.as_millis() as u64);
            peer.last_seen = std::time::SystemTime::now();

            // Improve reliability score
            peer.reliability_score = (peer.reliability_score * 0.9 + 0.1).min(1.0);
        }
    }

    async fn update_peer_failure(&self, peer_id: &str) {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(peer_id) {
            // Decrease reliability score
            peer.reliability_score = (peer.reliability_score * 0.9).max(0.0);
        }
    }

    fn calculate_peer_score(&self, peer: &PeerInfo) -> f64 {
        let mut score = peer.reliability_score;

        // Factor in distance (closer is better)
        if let Some(distance) = peer.distance_km {
            let distance_factor = 1.0 / (1.0 + distance / 1000.0); // Normalize by 1000km
            score *= distance_factor;
        }

        // Factor in response time (faster is better)
        if let Some(response_ms) = peer.response_time_ms {
            let response_factor = 1.0 / (1.0 + response_ms as f64 / 1000.0); // Normalize by 1 second
            score *= response_factor;
        }

        score
    }
}

// Thread-safe implementations
unsafe impl<D: DhtFacade> Send for NetworkManager<D> {}
unsafe impl<D: DhtFacade> Sync for NetworkManager<D> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dht_facade::LocalDht;

    fn setup_network_manager() -> NetworkManager<LocalDht> {
        let dht = Arc::new(LocalDht::new("test_node".to_string()));
        NetworkManager::new(dht)
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let manager = setup_network_manager();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        // Store content
        let store_result = manager.store_content(key.clone(), value.clone()).await;
        assert!(store_result.is_ok());

        // Retrieve content
        let retrieve_result = manager.retrieve_content(key).await;
        assert!(retrieve_result.is_ok());
        assert_eq!(retrieve_result.unwrap(), Some(value));
    }

    #[tokio::test]
    async fn test_peer_discovery() {
        let manager = setup_network_manager();

        let peers = manager.discover_peers().await.unwrap();
        assert!(peers.is_empty()); // LocalDht starts with no peers

        let peer_count = manager.get_active_peer_count().await;
        assert_eq!(peer_count, 0);
    }

    #[test]
    fn test_location_distance() {
        let loc1 = Location {
            latitude: 40.7128,
            longitude: -74.0060,
        }; // NYC
        let loc2 = Location {
            latitude: 34.0522,
            longitude: -118.2437,
        }; // LA

        let distance = loc1.distance_km(&loc2);
        assert!(distance > 3900.0 && distance < 4000.0); // Approximately 3944 km
    }

    #[tokio::test]
    async fn test_network_health() {
        let manager = setup_network_manager();

        // Should be healthy initially (no operations)
        let health = manager.check_health().await.unwrap();
        assert!(health);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let manager = Arc::new(setup_network_manager());
        let mut handles = Vec::new();

        // Start multiple concurrent store operations
        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let key = format!("key_{}", i).into_bytes();
                let value = format!("value_{}", i).into_bytes();
                manager_clone.store_content(key, value).await
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let manager = setup_network_manager();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        // Perform some operations
        manager
            .store_content(key.clone(), value.clone())
            .await
            .unwrap();
        manager.retrieve_content(key).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.successful_operations, 2);
        assert_eq!(stats.failed_operations, 0);
    }
}
