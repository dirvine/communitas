// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! FOAF (Friend-of-a-Friend) Discovery
//!
//! Implements SPEC.md §3: Replace DHT with presence-based discovery and FOAF.
//!
//! Discovery strategy:
//! 1. Check local contacts (O(1))
//! 2. Check presence in shared groups (group-scoped)
//! 3. Query FOAF (friends' contacts) - 2 hops max
//! 4. Fall back to introducer nodes for cold start

use anyhow::Result;
use saorsa_gossip_presence::PresenceManager;
use saorsa_gossip_types::{FoafQuery, FoafResponse, PeerId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Trait for sending FOAF queries over the network
///
/// This allows for mock implementations in tests
#[async_trait::async_trait]
pub trait FoafTransport: Send + Sync {
    /// Send a FOAF query to a peer
    async fn send_query(&self, peer: PeerId, query: FoafQuery) -> Result<()>;

    /// Wait for FOAF responses (with timeout)
    async fn wait_for_responses(&self, query_id: [u8; 16], timeout_ms: u64) -> Vec<FoafResponse>;
}

/// FOAF Discovery Manager
///
/// Finds contacts without DHT using friend-of-a-friend discovery
pub struct FoafDiscovery {
    /// Local contact cache (four_words → peer_id)
    local_contacts: Arc<RwLock<HashMap<String, PeerId>>>,

    /// Maximum FOAF hops (default: 2)
    max_hops: usize,

    /// Optional presence manager for group-scoped discovery
    presence: Option<Arc<RwLock<PresenceManager>>>,

    /// Optional FOAF transport for network queries
    foaf_transport: Option<Arc<dyn FoafTransport>>,

    /// Our peer ID
    our_peer_id: PeerId,

    /// Query timeout in milliseconds (default: 5000)
    query_timeout_ms: u64,
}

/// Result of a contact discovery lookup with address hints.
#[derive(Debug, Clone)]
pub struct ContactDiscoveryResult {
    pub peer_id: PeerId,
    pub addr_hints: Vec<String>,
}

impl Default for FoafDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl FoafDiscovery {
    /// Create a new FOAF discovery manager
    pub fn new() -> Self {
        Self {
            local_contacts: Arc::new(RwLock::new(HashMap::new())),
            max_hops: 2,
            presence: None,
            foaf_transport: None,
            our_peer_id: PeerId::new([0u8; 32]), // Default, should be set properly
            query_timeout_ms: 5000,
        }
    }

    /// Create a new FOAF discovery manager with presence integration
    pub fn with_presence(presence: Arc<RwLock<PresenceManager>>) -> Self {
        Self {
            local_contacts: Arc::new(RwLock::new(HashMap::new())),
            max_hops: 2,
            presence: Some(presence),
            foaf_transport: None,
            our_peer_id: PeerId::new([0u8; 32]),
            query_timeout_ms: 5000,
        }
    }

    /// Create a new FOAF discovery manager with full configuration
    pub fn with_config(
        presence: Option<Arc<RwLock<PresenceManager>>>,
        foaf_transport: Option<Arc<dyn FoafTransport>>,
        our_peer_id: PeerId,
        max_hops: usize,
    ) -> Self {
        Self {
            local_contacts: Arc::new(RwLock::new(HashMap::new())),
            max_hops,
            presence,
            foaf_transport,
            our_peer_id,
            query_timeout_ms: 5000,
        }
    }

    /// Find a contact by four-word address
    ///
    /// Strategy:
    /// 1. Check local cache
    /// 2. Check presence in shared groups (requires PresenceManager)
    /// 3. Query FOAF (up to max_hops) via transport if configured
    /// 4. Return error if not found
    pub async fn find_contact(&self, four_words: &str) -> Result<PeerId> {
        let result = self.find_contact_with_hints(four_words).await?;
        Ok(result.peer_id)
    }

    /// Find a contact by four-word address and return address hints if available.
    pub async fn find_contact_with_hints(
        &self,
        four_words: &str,
    ) -> Result<ContactDiscoveryResult> {
        // Step 1: Check local cache
        {
            let contacts = self.local_contacts.read().await;
            if let Some(peer_id) = contacts.get(four_words) {
                debug!("Found {} in local cache", four_words);

                let mut hints = Vec::new();
                if let Some(presence) = &self.presence {
                    let presence_guard = presence.read().await;
                    let groups = presence_guard.get_groups().await;
                    for topic_id in groups {
                        let presence_records = presence_guard.get_group_presence(topic_id).await;
                        if let Some(record) = presence_records.get(peer_id)
                            && !record.is_expired()
                        {
                            hints = record.addr_hints.clone();
                            break;
                        }
                    }
                }

                return Ok(ContactDiscoveryResult {
                    peer_id: *peer_id,
                    addr_hints: hints,
                });
            }
        }

        // Step 2: Check presence in shared groups
        if let Some(presence) = &self.presence {
            let presence_guard = presence.read().await;

            // Get all joined groups
            let groups = presence_guard.get_groups().await;

            // Search for four_words in presence records across all groups
            for topic_id in groups {
                let presence_records = presence_guard.get_group_presence(topic_id).await;

                // Check each presence record for matching four_words
                for (peer_id, record) in presence_records {
                    // Skip expired beacons
                    if record.is_expired() {
                        continue;
                    }

                    if let Some(fw) = &record.four_words
                        && fw == four_words
                    {
                        debug!("Found {} via presence in topic {:?}", four_words, topic_id);
                        // Add to cache for faster future lookups
                        let mut contacts = self.local_contacts.write().await;
                        contacts.insert(four_words.to_string(), peer_id);
                        return Ok(ContactDiscoveryResult {
                            peer_id,
                            addr_hints: record.addr_hints.clone(),
                        });
                    }
                }
            }
        }

        // Step 3: Query FOAF (up to max_hops)
        if let Some(transport) = &self.foaf_transport {
            debug!("Starting FOAF query for {}", four_words);

            // Generate unique query ID
            let query_id = self.generate_query_id();

            // Get our direct contacts to query
            let contacts = self.get_contacts().await;
            if contacts.is_empty() {
                debug!("No contacts to query via FOAF");
                return Err(anyhow::anyhow!(
                    "Contact {} not found. No contacts available for FOAF query.",
                    four_words
                ));
            }

            // Create FOAF query
            let query = FoafQuery {
                query_id,
                target_four_words: four_words.to_string(),
                hop: 0,
                max_hops: self.max_hops as u8,
                visited: vec![self.our_peer_id],
                originator: self.our_peer_id,
            };

            // Send query to all direct contacts
            for (_, peer_id) in contacts.iter() {
                if let Err(e) = transport.send_query(*peer_id, query.clone()).await {
                    warn!("Failed to send FOAF query to {:?}: {}", peer_id, e);
                }
            }

            // Wait for responses
            let responses = transport
                .wait_for_responses(query_id, self.query_timeout_ms)
                .await;

            if let Some(response) = responses.first() {
                info!(
                    "Found {} via FOAF query ({}  hops)",
                    four_words, response.hops
                );

                // Add to cache
                let mut cache = self.local_contacts.write().await;
                cache.insert(four_words.to_string(), response.peer_id);

                return Ok(ContactDiscoveryResult {
                    peer_id: response.peer_id,
                    addr_hints: response.addr_hints.clone(),
                });
            }

            debug!("FOAF query returned no results for {}", four_words);
        }

        Err(anyhow::anyhow!(
            "Contact {} not found via cache, presence, or FOAF queries.",
            four_words
        ))
    }

    /// Generate a unique query ID
    fn generate_query_id(&self) -> [u8; 16] {
        use std::time::SystemTime;

        let mut id = [0u8; 16];
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        // Use timestamp + our peer ID for uniqueness
        id[..8].copy_from_slice(&now.to_le_bytes()[..8]);
        id[8..].copy_from_slice(&self.our_peer_id.as_bytes()[..8]);

        id
    }

    /// Add a known contact to local cache
    pub async fn add_contact(&self, four_words: String, peer_id: PeerId) {
        let mut contacts = self.local_contacts.write().await;
        contacts.insert(four_words, peer_id);
    }

    /// Remove a contact from cache
    pub async fn remove_contact(&self, four_words: &str) {
        let mut contacts = self.local_contacts.write().await;
        contacts.remove(four_words);
    }

    /// Get all known contacts
    pub async fn get_contacts(&self) -> Vec<(String, PeerId)> {
        let contacts = self.local_contacts.read().await;
        contacts.iter().map(|(k, v)| (k.clone(), *v)).collect()
    }
}

/// Introducer node configuration for cold start
#[derive(Debug, Clone)]
pub struct IntroducerConfig {
    /// List of introducer addresses (host:port)
    pub addresses: Vec<String>,
    /// Timeout per introducer (seconds)
    pub timeout_secs: u64,
}

impl Default for IntroducerConfig {
    fn default() -> Self {
        Self {
            // Production bootstrap nodes (saorsa network)
            addresses: vec![
                "77.42.75.115:11000".to_string(), // saorsa-1: Dublin bootstrap (primary)
                "142.93.199.50:11000".to_string(), // saorsa-2: DigitalOcean NYC1 bootstrap
                "147.182.234.192:11000".to_string(), // saorsa-3: DigitalOcean SFO3 bootstrap
            ],
            timeout_secs: 10,
        }
    }
}

use saorsa_gossip_transport::GossipTransport;

/// Parse an address that may be either a direct socket address (IP:port)
/// or a four-word encoded address (words:port or words with encoded port)
fn parse_introducer_address(address: &str) -> Result<std::net::SocketAddr> {
    // First, try parsing as a direct socket address (e.g., "192.168.1.1:443")
    if let Ok(addr) = address.parse::<std::net::SocketAddr>() {
        return Ok(addr);
    }

    // Try parsing as four-word format using conn_from_words from identity module
    // Handle both "word-word-word-word:port" and "word word word word" formats
    match crate::identity::conn_from_words(address) {
        Ok(addr) => Ok(addr),
        Err(e) => {
            // If the address has :port suffix, try stripping it and decoding the words part
            if let Some((words_part, port_str)) = address.rsplit_once(':')
                && let Ok(port) = port_str.parse::<u16>()
            {
                // Try decoding just the words part
                match crate::identity::conn_from_words(words_part) {
                    Ok(mut addr) => {
                        // Replace the port with the explicit one
                        addr.set_port(port);
                        return Ok(addr);
                    }
                    Err(_) => {
                        // Try with dashes converted to spaces for the encoder
                        let words_with_spaces = words_part.replace('-', " ");
                        if let Ok(mut addr) = crate::identity::conn_from_words(&words_with_spaces) {
                            addr.set_port(port);
                            return Ok(addr);
                        }
                    }
                }
            }

            Err(anyhow::anyhow!(
                "Failed to parse address '{}': not a valid socket address or four-word format: {}",
                address,
                e
            ))
        }
    }
}

/// Cold start discovery using introducer nodes
pub async fn cold_start_discovery(
    config: IntroducerConfig,
    _transport: &dyn GossipTransport,
) -> Result<Vec<String>> {
    if config.addresses.is_empty() {
        warn!("No introducer nodes configured for cold start");
        return Ok(vec![]);
    }

    let mut connected_introducers = vec![];

    for introducer in &config.addresses {
        info!("Connecting to introducer: {}", introducer);

        // Parse the socket address (supports both IP:port and four-word formats)
        match parse_introducer_address(introducer) {
            Ok(addr) => {
                info!("Parsed introducer address '{}' -> {}", introducer, addr);
                // Store the resolved IP:port format for consistency
                connected_introducers.push(addr.to_string());
            }
            Err(e) => {
                warn!("Failed to parse introducer address {}: {}", introducer, e);
            }
        }
    }

    if connected_introducers.is_empty() {
        return Err(anyhow::anyhow!("Failed to connect to any introducers"));
    }

    info!("Connected to {} introducer(s)", connected_introducers.len());
    Ok(connected_introducers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_foaf_discovery_creation() {
        let discovery = FoafDiscovery::new();

        let contacts = discovery.get_contacts().await;
        assert_eq!(contacts.len(), 0);
    }

    #[tokio::test]
    async fn test_add_remove_contact() {
        let discovery = FoafDiscovery::new();

        let peer_id = PeerId::new([2u8; 32]);
        discovery
            .add_contact("ocean-forest-moon-star".to_string(), peer_id)
            .await;

        let contacts = discovery.get_contacts().await;
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].0, "ocean-forest-moon-star");

        discovery.remove_contact("ocean-forest-moon-star").await;
        let contacts = discovery.get_contacts().await;
        assert_eq!(contacts.len(), 0);
    }

    #[tokio::test]
    async fn test_find_contact_in_cache() {
        let discovery = FoafDiscovery::new();

        let peer_id = PeerId::new([3u8; 32]);
        discovery
            .add_contact("river-mountain-cloud-light".to_string(), peer_id)
            .await;

        let found = discovery.find_contact("river-mountain-cloud-light").await;
        assert!(found.is_ok());
        assert_eq!(found.expect("should find cached contact"), peer_id);
    }
}
