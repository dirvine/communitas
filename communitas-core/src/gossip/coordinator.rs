// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Coordinator Advertisements for NAT Traversal
//!
//! Implements SPEC2.md §2 (step 4) and §9: Coordinator Adverts for seedless bootstrap.
//!
//! ## Flow (per SPEC2.md §2):
//! 1. If peer cache is cold, request Coordinator Adverts via FOAF `FIND_COORDINATOR` (TTL=3, fanout=3)
//! 2. Connect to a coordinator for address reflection and hole punching
//! 3. Update peer cache with coordinator metadata (roles, NAT class, etc.)
//!
//! ## Coordinator Roles:
//! - **Bootstrap**: Help new peers join the network
//! - **Reflector**: Provide address observation for NAT detection
//! - **Rendezvous**: Coordinate peer introductions
//! - **Relay**: Forward messages (optional)

use anyhow::Result;
use bytes::Bytes;
use saorsa_gossip_coordinator::{
    AddrHint, CoordinatorAdvert, CoordinatorRoles, FindCoordinatorQuery, NatClass,
};
use saorsa_gossip_membership::Membership;
use saorsa_gossip_transport::{GossipTransport, StreamType};
use saorsa_gossip_types::PeerId;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Coordinator client wrapper for Communitas integration
pub struct CoordinatorClient {
    /// Our peer ID
    peer_id: PeerId,

    /// Transport layer for network communication
    transport: Arc<RwLock<Box<dyn GossipTransport>>>,

    /// Membership layer for peer list access
    membership: Arc<RwLock<Box<dyn Membership>>>,

    /// Cached coordinator adverts
    cached_adverts: Arc<RwLock<Vec<CoordinatorAdvert>>>,
}

impl CoordinatorClient {
    /// Create a new coordinator client
    ///
    /// # Arguments
    /// * `peer_id` - Our peer ID
    /// * `transport` - Transport layer for network communication
    /// * `membership` - Membership layer for peer list access
    pub fn new(
        peer_id: PeerId,
        transport: Arc<RwLock<Box<dyn GossipTransport>>>,
        membership: Arc<RwLock<Box<dyn Membership>>>,
    ) -> Self {
        Self {
            peer_id,
            transport,
            membership,
            cached_adverts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Publish our coordinator advert if we're acting as a coordinator
    ///
    /// Per SPEC2.md §9: Implement Coordinator Adverts publish/cache UI toggles
    ///
    /// # Arguments
    /// * `roles` - Coordinator roles we support
    /// * `endpoints` - Our listen endpoints
    /// * `nat_class` - Our detected NAT classification
    /// * `validity_ms` - How long this advert is valid (milliseconds)
    pub async fn publish_coordinator_advert(
        &self,
        roles: CoordinatorRoles,
        endpoints: Vec<SocketAddr>,
        nat_class: NatClass,
        validity_ms: u64,
    ) -> Result<()> {
        info!("Publishing coordinator advert for peer {:?}", self.peer_id);

        // Convert SocketAddr to AddrHint with current timestamp
        let addr_hints: Vec<AddrHint> = endpoints.into_iter().map(AddrHint::new).collect();

        // Create coordinator advert
        let advert =
            CoordinatorAdvert::new(self.peer_id, roles, addr_hints, nat_class, validity_ms);

        // Cache locally
        self.cached_adverts.write().await.push(advert.clone());

        // Serialize advert for network transmission
        let mut advert_bytes = Vec::new();
        ciborium::ser::into_writer(&advert, &mut advert_bytes)
            .map_err(|e| anyhow::anyhow!("CBOR encoding failed: {:?}", e))?;

        // Get active peers from membership layer
        let membership = self.membership.read().await;
        let active_peers = membership.active_view();

        if active_peers.is_empty() {
            debug!("No active peers to broadcast advert to, cached locally only");
            return Ok(());
        }

        // Broadcast advert to all active peers via PubSub stream
        let transport = self.transport.read().await;
        let mut broadcast_count = 0;
        let mut failed_count = 0;

        for peer_id in active_peers {
            match transport
                .send_to_peer(
                    peer_id,
                    StreamType::PubSub,
                    Bytes::from(advert_bytes.clone()),
                )
                .await
            {
                Ok(_) => {
                    broadcast_count += 1;
                    debug!("Broadcasted coordinator advert to peer {:?}", peer_id);
                }
                Err(e) => {
                    failed_count += 1;
                    warn!("Failed to broadcast advert to peer {:?}: {}", peer_id, e);
                }
            }
        }

        info!(
            "Coordinator advert broadcast complete: {} succeeded, {} failed out of {} peers",
            broadcast_count,
            failed_count,
            broadcast_count + failed_count
        );

        Ok(())
    }

    /// Find coordinators via FOAF discovery
    ///
    /// Per SPEC2.md §2 step 4: Request Coordinator Adverts via FOAF `FIND_COORDINATOR` (TTL=3, fanout=3)
    ///
    /// # Arguments
    /// * `ttl` - Time-to-live for FOAF query (default: 3)
    /// * `fanout` - Number of peers to query per hop (default: 3)
    ///
    /// # Returns
    /// Vec of discovered coordinator adverts
    pub async fn find_coordinators_via_foaf(
        &self,
        ttl: u8,
        fanout: u8,
    ) -> Result<Vec<CoordinatorAdvert>> {
        debug!(
            "Finding coordinators via FOAF (TTL={}, fanout={})",
            ttl, fanout
        );

        // Check cache first for fast return
        let cached = self.cached_adverts.read().await.clone();
        if !cached.is_empty() {
            info!("Returning {} cached coordinator adverts", cached.len());
            return Ok(cached);
        }

        // Get active peers from membership layer
        let membership = self.membership.read().await;
        let active_peers = membership.active_view();

        if active_peers.is_empty() {
            debug!("No active peers available for FOAF query, returning empty result");
            return Ok(Vec::new());
        }

        // Select up to `fanout` peers randomly
        use rand::SeedableRng;
        use rand::seq::SliceRandom;
        let mut rng = rand::rngs::StdRng::from_entropy();
        let selected_peers: Vec<_> = active_peers
            .choose_multiple(&mut rng, fanout as usize)
            .cloned()
            .collect();

        info!(
            "Sending FOAF query to {} peers (fanout={})",
            selected_peers.len(),
            fanout
        );

        // Create and serialize FIND_COORDINATOR query
        let query = FindCoordinatorQuery::new(self.peer_id);
        let mut query_bytes = Vec::new();
        ciborium::ser::into_writer(&query, &mut query_bytes)
            .map_err(|e| anyhow::anyhow!("CBOR encoding failed: {:?}", e))?;

        // Send query to selected peers
        let transport = self.transport.read().await;
        let mut query_count = 0;
        let mut failed_count = 0;

        for peer_id in &selected_peers {
            match transport
                .send_to_peer(
                    *peer_id,
                    StreamType::Membership,
                    Bytes::from(query_bytes.clone()),
                )
                .await
            {
                Ok(_) => {
                    query_count += 1;
                    debug!("Sent FOAF query to peer {:?}", peer_id);
                }
                Err(e) => {
                    failed_count += 1;
                    warn!("Failed to send FOAF query to peer {:?}: {}", peer_id, e);
                }
            }
        }

        info!(
            "FOAF queries sent: {} succeeded, {} failed",
            query_count, failed_count
        );

        if query_count == 0 {
            debug!("No queries succeeded, returning empty result");
            return Ok(Vec::new());
        }

        // Collect responses from peers with 10 second timeout
        let adverts = self
            .collect_coordinator_responses(&selected_peers, 10)
            .await?;

        // Cache discovered adverts
        if !adverts.is_empty() {
            let mut cache = self.cached_adverts.write().await;
            for advert in &adverts {
                // Only add if not already cached
                if !cache.iter().any(|a| a.peer == advert.peer) {
                    cache.push(advert.clone());
                }
            }
        }

        Ok(adverts)
    }

    /// Request address reflection from a coordinator
    ///
    /// This is used for NAT detection - coordinator tells us our public address
    ///
    /// # Arguments
    /// * `coordinator_peer_id` - PeerId of the coordinator
    ///
    /// # Returns
    /// Our observed public address
    pub async fn request_address_reflection(
        &self,
        coordinator_peer_id: PeerId,
    ) -> Result<SocketAddr> {
        debug!(
            "Requesting address reflection from coordinator {:?}",
            coordinator_peer_id
        );

        // Create reflection request (simple ping to coordinator)
        let request = b"ADDR_REFLECT_REQUEST";
        let request_bytes = Bytes::from(request.to_vec());

        // Send request to coordinator via membership stream
        let transport = self.transport.read().await;
        transport
            .send_to_peer(coordinator_peer_id, StreamType::Membership, request_bytes)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send reflection request: {}", e))?;

        // Wait for response with timeout (5 seconds)
        let response_future = async {
            loop {
                match transport.receive_message().await {
                    Ok((peer, stream_type, data)) => {
                        if peer == coordinator_peer_id && stream_type == StreamType::Membership {
                            // Parse response - expected format: "ADDR_REFLECT_RESPONSE:<ip>:<port>"
                            if let Ok(response_str) = String::from_utf8(data.to_vec())
                                && let Some(addr_str) =
                                    response_str.strip_prefix("ADDR_REFLECT_RESPONSE:")
                                && let Ok(addr) = addr_str.parse::<SocketAddr>()
                            {
                                return Ok(addr);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Error receiving reflection response: {}", e);
                        break;
                    }
                }
            }
            Err(anyhow::anyhow!("No valid reflection response received"))
        };

        match timeout(Duration::from_secs(5), response_future).await {
            Ok(Ok(addr)) => {
                info!("Received reflected address: {}", addr);
                Ok(addr)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow::anyhow!("Address reflection request timed out")),
        }
    }

    /// Get cached coordinator adverts
    pub async fn get_cached_adverts(&self) -> Vec<CoordinatorAdvert> {
        self.cached_adverts.read().await.clone()
    }

    /// Collect coordinator responses from network
    ///
    /// Listens for FindCoordinatorResponse messages from peers and aggregates results
    ///
    /// # Arguments
    /// * `expected_peers` - PeerIds we sent queries to
    /// * `timeout_secs` - How long to wait for responses (seconds)
    ///
    /// # Returns
    /// Deduplicated vec of coordinator adverts
    async fn collect_coordinator_responses(
        &self,
        expected_peers: &[PeerId],
        timeout_secs: u64,
    ) -> Result<Vec<CoordinatorAdvert>> {
        use std::collections::HashMap;

        debug!(
            "Collecting coordinator responses from {} peers (timeout: {}s)",
            expected_peers.len(),
            timeout_secs
        );

        let transport = self.transport.read().await;

        let response_future = async {
            let mut adverts_map: HashMap<PeerId, CoordinatorAdvert> = HashMap::new();
            loop {
                match transport.receive_message().await {
                    Ok((peer_id, stream_type, data)) => {
                        // Only process Membership stream responses from expected peers
                        if stream_type != StreamType::Membership {
                            continue;
                        }
                        if !expected_peers.contains(&peer_id) {
                            continue;
                        }

                        // Try to deserialize as coordinator advert
                        match ciborium::de::from_reader::<CoordinatorAdvert, _>(&data[..]) {
                            Ok(advert) => {
                                // Deduplicate by coordinator peer
                                let coord_peer = advert.peer;
                                adverts_map.entry(coord_peer).or_insert_with(|| {
                                    debug!(
                                        "Received coordinator advert from {:?} via {:?}",
                                        coord_peer, peer_id
                                    );
                                    advert
                                });
                            }
                            Err(e) => {
                                debug!(
                                    "Failed to deserialize coordinator advert from {:?}: {}",
                                    peer_id, e
                                );
                            }
                        }

                        // Stop if we've heard from all expected peers
                        if adverts_map.len() >= expected_peers.len() {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Error receiving coordinator response: {}", e);
                        break;
                    }
                }
            }
            adverts_map.into_values().collect::<Vec<_>>()
        };

        match timeout(Duration::from_secs(timeout_secs), response_future).await {
            Ok(adverts) => {
                info!("Collected {} coordinator adverts", adverts.len());
                Ok(adverts)
            }
            Err(_) => {
                info!("Response collection timed out, returning empty result");
                Ok(Vec::new())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_gossip_transport::{QuicTransport, TransportConfig};
    use std::net::Ipv4Addr;

    fn create_test_peer_id(seed: u8) -> PeerId {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        PeerId::new(bytes)
    }

    async fn create_test_coordinator_client() -> CoordinatorClient {
        let peer_id = create_test_peer_id(1);

        // Create a test transport (using QuicTransport like context.rs does)
        let config = TransportConfig::default();
        let transport = QuicTransport::new(config);
        let transport: Arc<RwLock<Box<dyn GossipTransport>>> =
            Arc::new(RwLock::new(Box::new(transport)));

        // Create a test membership layer
        let membership: Arc<RwLock<Box<dyn Membership>>> = Arc::new(RwLock::new(Box::new(
            saorsa_gossip_membership::HyParViewMembership::new(
                5,  // active_degree
                15, // passive_degree
                Arc::new(QuicTransport::new(TransportConfig::default())),
            ),
        )));

        CoordinatorClient::new(peer_id, transport, membership)
    }

    #[tokio::test]
    async fn test_coordinator_client_creation() {
        // RED: This test should pass immediately but establishes the structure
        let client = create_test_coordinator_client().await;

        // Verify client was created
        assert_eq!(client.peer_id.as_bytes()[0], 1);

        // Verify empty cache initially
        let cached = client.get_cached_adverts().await;
        assert_eq!(cached.len(), 0);
    }

    #[tokio::test]
    async fn test_publish_coordinator_advert() {
        // GREEN: Now implements actual caching
        let client = create_test_coordinator_client().await;

        let roles = CoordinatorRoles {
            coordinator: true,
            reflector: true,
            rendezvous: false,
            relay: false,
        };
        let endpoints = vec![SocketAddr::from((Ipv4Addr::LOCALHOST, 9000))];
        let nat_class = NatClass::Eim; // Published type
        let validity_ms = 3_600_000; // 1 hour

        // Should succeed and cache locally
        let result = client
            .publish_coordinator_advert(roles.clone(), endpoints.clone(), nat_class, validity_ms)
            .await;

        assert!(result.is_ok());

        // Verify cached locally
        let cached = client.get_cached_adverts().await;
        assert_eq!(cached.len(), 1);
    }

    #[tokio::test]
    async fn test_find_coordinators_via_foaf_cold_cache() {
        let client = create_test_coordinator_client().await;

        let ttl = 3;
        let fanout = 3;

        let coordinators = client
            .find_coordinators_via_foaf(ttl, fanout)
            .await
            .expect("FOAF discovery should not fail");

        // Returns empty since no active peers in test
        assert_eq!(coordinators.len(), 0);
    }

    #[tokio::test]
    async fn test_request_address_reflection() {
        // GREEN: Now implements actual reflection request with timeout
        let client = create_test_coordinator_client().await;

        let coordinator_peer_id = create_test_peer_id(100);

        let result = client.request_address_reflection(coordinator_peer_id).await;

        // Should timeout since there's no real coordinator responding
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("timed out") || err_msg.contains("No valid reflection response"),
            "Expected timeout or no response error, got: {}",
            err_msg
        );

        // Note: With a real coordinator, we would verify:
        // - Request sent to coordinator
        // - Received our public address
        // - Address is valid SocketAddr
    }

    #[tokio::test]
    async fn test_coordinator_roles_default() {
        let roles = CoordinatorRoles::default();

        // Published crate defaults: coordinator=true, reflector=true, others=false
        assert!(roles.coordinator);
        assert!(roles.reflector);
        assert!(!roles.rendezvous);
        assert!(!roles.relay);
    }
}
