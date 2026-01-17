//! FOAF (Friend-of-a-Friend) Discovery Tests
//!
//! Test-Driven Development approach for implementing SPEC.md §3
//! DHT replacement with presence-based FOAF discovery.
//!
//! Tests verify:
//! 1. Local contact cache operations
//! 2. Presence-based discovery in shared groups
//! 3. 2-hop FOAF queries through contact network
//! 4. Introducer node cold start
//! 5. Complete discovery flow without DHT

use communitas_core::gossip::discovery::{FoafDiscovery, IntroducerConfig, cold_start_discovery};
use saorsa_gossip_types::PeerId;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Helper: Create test peer ID from byte array
fn test_peer_id(byte: u8) -> PeerId {
    PeerId::new([byte; 32])
}

// ============================================================================
// PHASE 1: Local Cache Tests (Already Passing)
// ============================================================================

#[tokio::test]
async fn test_local_cache_add_find() {
    let discovery = FoafDiscovery::new();
    let peer_id = test_peer_id(1);

    // Add contact
    discovery
        .add_contact("ocean-forest-moon-star".to_string(), peer_id)
        .await;

    // Find in cache
    let found = discovery.find_contact("ocean-forest-moon-star").await;
    assert!(found.is_ok());
    assert_eq!(found.expect("should find"), peer_id);
}

#[tokio::test]
async fn test_local_cache_not_found() {
    let discovery = FoafDiscovery::new();

    // Try to find non-existent contact
    let found = discovery.find_contact("river-mountain-cloud-light").await;
    assert!(found.is_err());
}

#[tokio::test]
async fn test_local_cache_remove() {
    let discovery = FoafDiscovery::new();
    let peer_id = test_peer_id(2);

    // Add and verify
    discovery
        .add_contact("winter-summer-spring-fall".to_string(), peer_id)
        .await;
    assert!(
        discovery
            .find_contact("winter-summer-spring-fall")
            .await
            .is_ok()
    );

    // Remove
    discovery.remove_contact("winter-summer-spring-fall").await;

    // Verify removed
    assert!(
        discovery
            .find_contact("winter-summer-spring-fall")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_local_cache_list_contacts() {
    let discovery = FoafDiscovery::new();

    // Add multiple contacts
    discovery
        .add_contact("north-south-east-west".to_string(), test_peer_id(10))
        .await;
    discovery
        .add_contact("fire-water-earth-air".to_string(), test_peer_id(20))
        .await;
    discovery
        .add_contact("alpha-beta-gamma-delta".to_string(), test_peer_id(30))
        .await;

    // Get all contacts
    let contacts = discovery.get_contacts().await;
    assert_eq!(contacts.len(), 3);

    // Verify all present
    let four_words: Vec<String> = contacts.iter().map(|(fw, _)| fw.clone()).collect();
    assert!(four_words.contains(&"north-south-east-west".to_string()));
    assert!(four_words.contains(&"fire-water-earth-air".to_string()));
    assert!(four_words.contains(&"alpha-beta-gamma-delta".to_string()));
}

// ============================================================================
// PHASE 2: Presence-Based Discovery Tests (TDD - Write Tests First)
// ============================================================================

#[tokio::test]
async fn test_presence_discovery_in_shared_group() {
    // GIVEN: Two users in the same group with presence beacons
    let (presence, groups) = create_mock_presence().await;
    let topic_id = saorsa_gossip_types::TopicId::new([1u8; 32]);

    // Add presence beacons to the group
    add_mock_presence(
        &presence,
        &groups,
        topic_id,
        test_peer_id(100),
        "alice-has-four-words",
    )
    .await;
    add_mock_presence(
        &presence,
        &groups,
        topic_id,
        test_peer_id(200),
        "bob-charlie-david-eve",
    )
    .await;

    // Create discovery with presence integration
    let discovery = FoafDiscovery::with_presence(presence);

    // WHEN: Looking for a user in the same group
    let found = discovery.find_contact("bob-charlie-david-eve").await;

    // THEN: Should find via presence beacon in shared group
    assert!(found.is_ok(), "Should find contact in presence");
    assert_eq!(found.expect("should find in presence"), test_peer_id(200));
}

#[tokio::test]
async fn test_presence_discovery_multiple_groups() {
    // GIVEN: User present in multiple groups
    let (presence, groups) = create_mock_presence().await;
    let topic1 = saorsa_gossip_types::TopicId::new([1u8; 32]);
    let topic2 = saorsa_gossip_types::TopicId::new([2u8; 32]);

    // Add user to both groups
    add_mock_presence(
        &presence,
        &groups,
        topic1,
        test_peer_id(150),
        "charlie-delta-echo-fox",
    )
    .await;
    add_mock_presence(
        &presence,
        &groups,
        topic2,
        test_peer_id(150),
        "charlie-delta-echo-fox",
    )
    .await;

    let discovery = FoafDiscovery::with_presence(presence);

    // WHEN: Looking for the user
    let found = discovery.find_contact("charlie-delta-echo-fox").await;

    // THEN: Should find in first matching group
    assert!(found.is_ok(), "Should find user in one of the groups");
    assert_eq!(found.expect("should find"), test_peer_id(150));
}

#[tokio::test]
async fn test_presence_discovery_expired_beacon() {
    // GIVEN: User with expired presence beacon (TTL = 0)
    let (presence, groups) = create_mock_presence().await;
    let topic_id = saorsa_gossip_types::TopicId::new([1u8; 32]);

    // Add the group to groups map
    {
        let group_ctx = GroupContext::from_entity("test-group");
        let mut groups_guard = groups.write().await;
        groups_guard.insert(topic_id, group_ctx);
    }

    // Add expired beacon (TTL = 0 means immediate expiration)
    let record = PresenceRecord::with_four_words(
        [0u8; 32],
        vec![],
        0, // TTL = 0, immediately expired
        "expired-user-old-beacon".to_string(),
    );

    let manager_guard = presence.write().await;
    manager_guard
        .handle_beacon(topic_id, test_peer_id(250), record)
        .await
        .expect("handle_beacon failed");
    drop(manager_guard);

    let discovery = FoafDiscovery::with_presence(presence);

    // Wait a tiny bit to ensure expiration
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // WHEN: Looking for user with expired beacon
    let found = discovery.find_contact("expired-user-old-beacon").await;

    // THEN: Should not find (beacon expired)
    assert!(found.is_err(), "Should not find expired beacon");
}

// ============================================================================
// PHASE 3: FOAF Query Protocol Tests (TDD - Write Tests First)
// ============================================================================

#[tokio::test]
async fn test_foaf_1hop_discovery() {
    // GIVEN: Contact network with 1-hop connection
    let me = test_peer_id(1);
    let alice = test_peer_id(10);
    let bob = test_peer_id(20);

    // Network topology:
    // Me -> Alice (direct contact)
    // Alice -> Bob (Alice's contact)
    //
    // I want to find Bob (1-hop through Alice)

    // Setup mock FOAF transport
    let transport = Arc::new(MockFoafTransport::new());

    // Configure Alice's knowledge: Alice knows Bob
    transport
        .add_knowledge(alice, "bob-bravo-charlie-delta".to_string(), bob)
        .await;

    // Create discovery with FOAF transport
    let discovery = FoafDiscovery::with_config(
        None, // No presence
        Some(transport),
        me,
        2, // max_hops
    );

    // Add Alice as direct contact
    discovery
        .add_contact("alice-alpha-beta-gamma".to_string(), alice)
        .await;

    // WHEN: Looking for Bob (not in my cache)
    let found = discovery.find_contact("bob-bravo-charlie-delta").await;

    // THEN: Should find via 1-hop FOAF query through Alice
    assert!(found.is_ok(), "Should find Bob via 1-hop FOAF");
    assert_eq!(found.expect("should find via FOAF"), bob);
}

#[tokio::test]
async fn test_foaf_2hop_discovery() {
    // GIVEN: Contact network with 2-hop connection
    let me = test_peer_id(1);
    let alice = test_peer_id(10);
    let bob = test_peer_id(20);
    let charlie = test_peer_id(30);

    // Network topology:
    // Me -> Alice (direct)
    // Alice -> Bob (Alice's contact)
    // Bob -> Charlie (Bob's contact)
    //
    // I want to find Charlie (2-hop: Me->Alice->Bob->Charlie)

    // Setup mock FOAF transport
    let transport = Arc::new(MockFoafTransport::new());

    // Configure network knowledge:
    // Alice knows Bob (but not Charlie)
    transport
        .add_knowledge(alice, "bob-bravo-charlie-delta".to_string(), bob)
        .await;

    // Bob knows Charlie
    transport
        .add_knowledge(bob, "charlie-cesar-delta-echo".to_string(), charlie)
        .await;

    // Create discovery with FOAF transport
    let discovery = FoafDiscovery::with_config(
        None, // No presence
        Some(transport),
        me,
        2, // max_hops (allows 2-hop)
    );

    // Add Alice as direct contact
    discovery
        .add_contact("alice-alpha-beta-gamma".to_string(), alice)
        .await;

    // WHEN: Looking for Charlie
    let found = discovery.find_contact("charlie-cesar-delta-echo").await;

    // THEN: Should find via 2-hop FOAF
    assert!(found.is_ok(), "Should find Charlie via 2-hop FOAF");
    assert_eq!(found.expect("should find via 2-hop"), charlie);
}

#[tokio::test]
async fn test_foaf_max_hops_limit() {
    // GIVEN: Contact network with 3-hop connection (exceeds max_hops=2)
    let me = test_peer_id(1);
    let alice = test_peer_id(10);
    let bob = test_peer_id(20);
    let charlie = test_peer_id(30);
    let david = test_peer_id(40);

    // Network topology:
    // Me -> Alice (direct)
    // Alice -> Bob (1 hop)
    // Bob -> Charlie (2 hops)
    // Charlie -> David (3 hops - exceeds limit!)

    // Setup mock FOAF transport
    let transport = Arc::new(MockFoafTransport::new());

    // Configure network knowledge:
    transport
        .add_knowledge(alice, "bob-bravo-charlie-delta".to_string(), bob)
        .await;
    transport
        .add_knowledge(bob, "charlie-cesar-delta-echo".to_string(), charlie)
        .await;
    transport
        .add_knowledge(charlie, "david-delta-epsilon-foxtrot".to_string(), david)
        .await;

    // Create discovery with max_hops=2
    let discovery = FoafDiscovery::with_config(
        None,
        Some(transport),
        me,
        2, // max_hops=2 (should stop at Charlie, won't reach David)
    );

    // Add Alice as direct contact
    discovery
        .add_contact("alice-alpha-beta-gamma".to_string(), alice)
        .await;

    // WHEN: Looking for David (3 hops, exceeds limit)
    let found = discovery.find_contact("david-delta-epsilon-foxtrot").await;

    // THEN: Should NOT find (exceeds max_hops=2)
    assert!(
        found.is_err(),
        "Should NOT find David beyond max_hops limit"
    );
}

#[tokio::test]
async fn test_foaf_query_timeout() {
    // GIVEN: FOAF query that doesn't respond in time
    let me = test_peer_id(1);
    let alice = test_peer_id(10);

    // Setup mock FOAF transport (no knowledge configured)
    let transport = Arc::new(MockFoafTransport::new());

    // Create discovery
    let discovery = FoafDiscovery::with_config(
        None,
        Some(transport),
        me,
        2, // max_hops
    );

    // Add Alice as direct contact
    discovery
        .add_contact("alice-alpha-beta-gamma".to_string(), alice)
        .await;

    // WHEN: Looking for contact that nobody knows (no response)
    let found = discovery.find_contact("slow-response-user-timeout").await;

    // THEN: Should timeout and return error (no responses after timeout period)
    assert!(
        found.is_err(),
        "Should return error when no responses received"
    );
}

#[tokio::test]
async fn test_foaf_cycle_detection() {
    // GIVEN: Contact network with circular references
    let me = test_peer_id(1);
    let alice = test_peer_id(10);
    let bob = test_peer_id(20);

    // Network topology (cycle):
    // Me -> Alice
    // Alice -> Bob
    // Bob -> Alice (cycle!)

    // Setup mock FOAF transport
    let transport = Arc::new(MockFoafTransport::new());

    // Configure circular network:
    // Alice knows Bob
    transport
        .add_knowledge(alice, "bob-bravo-charlie-delta".to_string(), bob)
        .await;

    // Bob knows Alice (creates cycle)
    transport
        .add_knowledge(bob, "alice-alpha-beta-gamma".to_string(), alice)
        .await;

    // Create discovery
    let discovery = FoafDiscovery::with_config(
        None,
        Some(transport),
        me,
        2, // max_hops
    );

    // Add Alice as direct contact
    discovery
        .add_contact("alice-alpha-beta-gamma".to_string(), alice)
        .await;

    // WHEN: Looking for non-existent contact (would trigger cycle traversal)
    let found = discovery.find_contact("non-existent-creates-cycle").await;

    // THEN: Should not infinite loop, should return error
    // The cycle detection prevents infinite forwarding
    assert!(
        found.is_err(),
        "Should not find non-existent contact (cycle handled gracefully)"
    );
}

// ============================================================================
// PHASE 4: Introducer Node Tests
// ============================================================================

#[tokio::test]
async fn test_introducer_node_connection() {
    // GIVEN: Introducer node address with mock transport
    let _config = IntroducerConfig {
        addresses: vec!["127.0.0.1:9000".to_string()],
        timeout_secs: 10,
    };

    // Setup mock transport that simulates successful connection
    let transport = Arc::new(MockFoafTransport::new());
    let introducer_peer = test_peer_id(99);

    // Introducer knows some peers
    transport
        .add_knowledge(
            introducer_peer,
            "alice-beta-gamma-delta".to_string(),
            test_peer_id(10),
        )
        .await;
    transport
        .add_knowledge(
            introducer_peer,
            "bob-charlie-delta-echo".to_string(),
            test_peer_id(20),
        )
        .await;

    // Discovery with introducer configured
    let discovery = FoafDiscovery::with_config(None, Some(transport), test_peer_id(1), 2);

    // Simulate having introducer as a contact
    discovery
        .add_contact("introducer-node-test".to_string(), introducer_peer)
        .await;

    // WHEN: Attempting discovery via introducer
    let found = discovery.find_contact("alice-beta-gamma-delta").await;

    // THEN: Should find via introducer
    assert!(found.is_ok(), "Should connect via introducer");
    assert_eq!(found.unwrap(), test_peer_id(10));
}

#[tokio::test]
async fn test_introducer_config_empty_addresses() {
    // GIVEN: Empty introducer config
    let config = IntroducerConfig {
        addresses: vec![],
        timeout_secs: 10,
    };

    let bind: std::net::SocketAddr = "127.0.0.1:0".parse().expect("valid addr");
    let transport = AntQuicTransport::new(bind, vec![])
        .await
        .expect("transport");

    // WHEN: Attempting cold start with no introducers
    let peers = cold_start_discovery(config, &transport)
        .await
        .expect("cold start");

    // THEN: Should return empty list (not an error - graceful degradation)
    assert_eq!(peers.len(), 0);
}

#[tokio::test]
async fn test_introducer_node_timeout() {
    // GIVEN: Introducer that doesn't respond
    let _config = IntroducerConfig {
        addresses: vec!["192.0.2.1:9000".to_string()], // TEST-NET address (should timeout)
        timeout_secs: 1,
    };

    // Mock transport with no knowledge (simulates no response)
    let transport = Arc::new(MockFoafTransport::new());
    let introducer_peer = test_peer_id(99);

    let discovery = FoafDiscovery::with_config(None, Some(transport), test_peer_id(1), 2);

    discovery
        .add_contact("slow-introducer".to_string(), introducer_peer)
        .await;

    // WHEN: Attempting cold start (no responses)
    let start = std::time::Instant::now();
    let result = discovery.find_contact("non-existent-peer").await;
    let elapsed = start.elapsed();

    // THEN: Should fail gracefully and quickly (mock has 10ms wait)
    assert!(result.is_err(), "Should return error when not found");
    assert!(
        elapsed < std::time::Duration::from_secs(2),
        "Should fail quickly without blocking"
    );
}

// ============================================================================
// PHASE 5: Integration Tests (Complete Discovery Flow)
// ============================================================================

#[tokio::test]
async fn test_complete_discovery_flow() {
    let (presence_mgr, groups_map) = create_mock_presence().await;
    let topic_id = TopicId::new([1u8; 32]);
    let alice_peer = test_peer_id(10);
    let bob_peer = test_peer_id(11);

    add_mock_presence(
        &presence_mgr,
        &groups_map,
        topic_id,
        alice_peer,
        "alice-alpha-beta-gamma",
    )
    .await;

    let transport = Arc::new(MockFoafTransport::new());
    transport
        .add_knowledge(alice_peer, "bob-bravo-charlie-delta".to_string(), bob_peer)
        .await;

    let discovery =
        FoafDiscovery::with_config(Some(presence_mgr), Some(transport), test_peer_id(1), 2);

    discovery
        .add_contact("alice-alpha-beta-gamma".to_string(), alice_peer)
        .await;

    let found_alice = discovery
        .find_contact("alice-alpha-beta-gamma")
        .await
        .expect("find alice");
    assert_eq!(found_alice, alice_peer);

    let found_bob = discovery
        .find_contact("bob-bravo-charlie-delta")
        .await
        .expect("find bob");
    assert_eq!(found_bob, bob_peer);

    let cached_bob = discovery
        .find_contact("bob-bravo-charlie-delta")
        .await
        .expect("find bob from cache");
    assert_eq!(cached_bob, bob_peer);
}

#[tokio::test]
async fn test_discovery_fallback_chain() {
    let (presence_mgr, groups_map) = create_mock_presence().await;
    let topic_id = TopicId::new([2u8; 32]);
    let alice_peer = test_peer_id(12);
    let target_peer = test_peer_id(13);

    add_mock_presence(
        &presence_mgr,
        &groups_map,
        topic_id,
        alice_peer,
        "alice-alpha-beta-gamma",
    )
    .await;

    let transport = Arc::new(MockFoafTransport::new());
    transport
        .add_knowledge(
            alice_peer,
            "distant-contact-two-hops".to_string(),
            target_peer,
        )
        .await;

    let discovery =
        FoafDiscovery::with_config(Some(presence_mgr), Some(transport), test_peer_id(1), 2);

    discovery
        .add_contact("alice-alpha-beta-gamma".to_string(), alice_peer)
        .await;

    let found = discovery
        .find_contact("distant-contact-two-hops")
        .await
        .expect("find via foaf");
    assert_eq!(found, target_peer);
}

// ============================================================================
// Test Helpers
// ============================================================================

use communitas_core::gossip::discovery::FoafTransport;
use saorsa_gossip_groups::GroupContext;
use saorsa_gossip_presence::PresenceManager;
use saorsa_gossip_transport::AntQuicTransport;
use saorsa_gossip_types::{FoafQuery, FoafResponse, PresenceRecord, TopicId};
use std::collections::HashMap;
use std::net::SocketAddr;

/// Create a mock presence manager with test data
/// Returns both the manager and the groups map so tests can add groups
async fn create_mock_presence() -> (
    Arc<RwLock<PresenceManager>>,
    Arc<RwLock<HashMap<TopicId, GroupContext>>>,
) {
    let peer_id = test_peer_id(1);
    let bind: SocketAddr = "127.0.0.1:0".parse().expect("valid addr");
    let transport = Arc::new(
        AntQuicTransport::new(bind, vec![])
            .await
            .expect("transport"),
    );
    let groups_by_topic = Arc::new(RwLock::new(HashMap::new()));

    let manager = PresenceManager::new(peer_id, transport, groups_by_topic.clone());
    (Arc::new(RwLock::new(manager)), groups_by_topic)
}

/// Add a mock presence record to the manager
/// Also adds the group to the groups map
async fn add_mock_presence(
    manager: &Arc<RwLock<PresenceManager>>,
    groups_map: &Arc<RwLock<HashMap<TopicId, GroupContext>>>,
    topic_id: TopicId,
    peer_id: PeerId,
    four_words: &str,
) {
    // Add the group to the groups map so get_groups() will return it
    let group_ctx = GroupContext::from_entity("test-group");
    {
        let mut groups = groups_map.write().await;
        groups.entry(topic_id).or_insert(group_ctx);
    }

    // Add the presence beacon
    let record = PresenceRecord::with_four_words(
        [0u8; 32],
        vec!["127.0.0.1:8080".to_string()],
        900, // 15 min TTL
        four_words.to_string(),
    );

    let manager_guard = manager.write().await;
    manager_guard
        .handle_beacon(topic_id, peer_id, record)
        .await
        .expect("handle_beacon failed");
}

/// Mock FOAF transport for testing
///
/// Simulates a network of peers with known contacts
pub struct MockFoafTransport {
    /// Network topology: peer_id → HashMap<four_words, peer_id>
    /// Each peer knows some contacts by their four-word addresses
    network: Arc<RwLock<HashMap<PeerId, HashMap<String, PeerId>>>>,

    /// Pending responses collected during wait period
    responses: Arc<RwLock<HashMap<[u8; 16], Vec<FoafResponse>>>>,
}

impl Default for MockFoafTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl MockFoafTransport {
    pub fn new() -> Self {
        Self {
            network: Arc::new(RwLock::new(HashMap::new())),
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a peer's knowledge to the network
    /// peer knows that four_words maps to target_peer_id
    pub async fn add_knowledge(&self, peer: PeerId, four_words: String, target_peer_id: PeerId) {
        let mut network = self.network.write().await;
        network
            .entry(peer)
            .or_insert_with(HashMap::new)
            .insert(four_words, target_peer_id);
    }

    /// Process a FOAF query (simulates network behavior)
    async fn process_query(&self, peer: PeerId, query: FoafQuery) {
        // Check if this peer knows the target
        let network = self.network.read().await;

        if let Some(peer_knowledge) = network.get(&peer)
            && let Some(&target_peer_id) = peer_knowledge.get(&query.target_four_words)
        {
            // Found! Send response
            let response = FoafResponse {
                query_id: query.query_id,
                peer_id: target_peer_id,
                addr_hints: vec!["127.0.0.1:8080".to_string()],
                hops: query.hop + 1,
            };

            let mut responses = self.responses.write().await;
            responses
                .entry(query.query_id)
                .or_insert_with(Vec::new)
                .push(response);

            return;
        }
        // Not found at this peer, forward if within hop limit
        if query.hop + 1 < query.max_hops {
            // Check for cycles
            if query.visited.contains(&peer) {
                return; // Cycle detected, don't forward
            }

            // Forward to this peer's contacts
            if let Some(peer_contacts) = network.get(&peer) {
                let mut new_visited = query.visited.clone();
                new_visited.push(peer);

                let forwarded_query = FoafQuery {
                    hop: query.hop + 1,
                    visited: new_visited,
                    ..query
                };

                // Forward to all contacts (except those already visited)
                for (_, contact_peer_id) in peer_contacts.iter() {
                    if !forwarded_query.visited.contains(contact_peer_id) {
                        Box::pin(self.process_query(*contact_peer_id, forwarded_query.clone()))
                            .await;
                    }
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl FoafTransport for MockFoafTransport {
    async fn send_query(&self, peer: PeerId, query: FoafQuery) -> Result<(), anyhow::Error> {
        // Simulate sending query by processing it
        self.process_query(peer, query).await;
        Ok(())
    }

    async fn wait_for_responses(&self, query_id: [u8; 16], _timeout_ms: u64) -> Vec<FoafResponse> {
        // Give a tiny bit of time for async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let mut responses = self.responses.write().await;
        responses.remove(&query_id).unwrap_or_default()
    }
}
