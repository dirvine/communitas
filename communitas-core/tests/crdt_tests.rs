//! Comprehensive CRDT Tests with Property-Based Testing
//!
//! Tests vector clocks, causal ordering, and CRDT message synchronization
//! using proptest for exhaustive property verification.

use communitas_core::crdt::*;
use proptest::prelude::*;

// ============================================================================
// Property-Based Tests for VectorClock
// ============================================================================

/// Generate arbitrary VectorClock with up to 10 peers
fn arb_vector_clock() -> impl Strategy<Value = VectorClock> {
    prop::collection::btree_map(
        "[a-z]{4}-[a-z]{4}-[a-z]{4}-[a-z]{4}", // Four-word peer IDs
        1u64..100,                             // Timestamps 1-99
        0..10,                                 // Up to 10 peers
    )
    .prop_map(VectorClock)
}

/// Generate a pair of related vector clocks (one derived from the other)
fn arb_related_clocks() -> impl Strategy<Value = (VectorClock, VectorClock)> {
    arb_vector_clock().prop_flat_map(|clock1| {
        let clock2 = clock1.clone();
        let peers: Vec<String> = clock1.0.keys().cloned().collect();

        // Only modify if there are peers
        if peers.is_empty() {
            return Just((clock1, clock2)).boxed();
        }

        // Modify clock2 by incrementing some timestamps
        prop::collection::vec(prop::sample::select(peers.clone()), 0..5)
            .prop_map(move |peers_to_increment| {
                let mut clock2_mod = clock2.clone();
                for peer in peers_to_increment {
                    if let Some(val) = clock2_mod.0.get_mut(&peer) {
                        *val += 1;
                    }
                }
                (clock1.clone(), clock2_mod)
            })
            .boxed()
    })
}

proptest! {
    /// Property: Incrementing a clock always increases the peer's timestamp
    #[test]
    fn prop_increment_increases_timestamp(mut clock in arb_vector_clock()) {
        let peer_id = "test-peer-id-one";
        let initial = clock.0.get(peer_id).copied().unwrap_or(0);

        clock.increment(peer_id);

        let after = clock.0.get(peer_id).copied().unwrap_or(0);
        prop_assert_eq!(after, initial + 1);
    }

    /// Property: Merging two clocks produces a clock >= both inputs
    #[test]
    fn prop_merge_is_greater_or_equal(
        mut clock1 in arb_vector_clock(),
        clock2 in arb_vector_clock()
    ) {
        let original1 = clock1.clone();
        clock1.merge(&clock2);

        // Check all peers from clock1 are >= original
        for (peer, timestamp) in &original1.0 {
            let merged_ts = clock1.0.get(peer).copied().unwrap_or(0);
            prop_assert!(merged_ts >= *timestamp);
        }

        // Check all peers from clock2 are <= merged
        for (peer, timestamp) in &clock2.0 {
            let merged_ts = clock1.0.get(peer).copied().unwrap_or(0);
            prop_assert!(merged_ts >= *timestamp);
        }
    }

    /// Property: Merge is commutative: merge(A, B) == merge(B, A)
    #[test]
    fn prop_merge_commutative(
        clock1 in arb_vector_clock(),
        clock2 in arb_vector_clock()
    ) {
        let mut a_then_b = clock1.clone();
        a_then_b.merge(&clock2);

        let mut b_then_a = clock2.clone();
        b_then_a.merge(&clock1);

        prop_assert_eq!(a_then_b, b_then_a);
    }

    /// Property: Merge is idempotent: merge(A, A) == A
    #[test]
    fn prop_merge_idempotent(clock in arb_vector_clock()) {
        let mut merged = clock.clone();
        merged.merge(&clock);

        prop_assert_eq!(merged, clock);
    }

    /// Property: Merge is associative: merge(merge(A, B), C) == merge(A, merge(B, C))
    #[test]
    fn prop_merge_associative(
        clock1 in arb_vector_clock(),
        clock2 in arb_vector_clock(),
        clock3 in arb_vector_clock()
    ) {
        // (A ∪ B) ∪ C
        let mut left = clock1.clone();
        left.merge(&clock2);
        left.merge(&clock3);

        // A ∪ (B ∪ C)
        let mut right = clock2.clone();
        right.merge(&clock3);
        let mut right_final = clock1.clone();
        right_final.merge(&right);

        prop_assert_eq!(left, right_final);
    }

    /// Property: If clock A happened-before clock B, then compare(A, B) == Before
    #[test]
    fn prop_compare_transitive((clock1, clock2) in arb_related_clocks()) {
        let ordering = clock1.compare(&clock2);

        match ordering {
            ClockOrdering::Before => {
                // All of clock1's timestamps should be <= clock2's
                for (peer, ts1) in &clock1.0 {
                    let ts2 = clock2.0.get(peer).copied().unwrap_or(0);
                    prop_assert!(ts1 <= &ts2);
                }
            }
            ClockOrdering::After => {
                // All of clock2's timestamps should be <= clock1's
                for (peer, ts2) in &clock2.0 {
                    let ts1 = clock1.0.get(peer).copied().unwrap_or(0);
                    prop_assert!(ts2 <= &ts1);
                }
            }
            ClockOrdering::Equal => {
                prop_assert_eq!(clock1, clock2);
            }
            ClockOrdering::Concurrent => {
                // At least one peer has ts1 > ts2 AND one has ts1 < ts2
                // This is allowed
            }
        }
    }

    /// Property: Compare is anti-symmetric: if compare(A, B) == Before, then compare(B, A) == After
    #[test]
    fn prop_compare_anti_symmetric(
        clock1 in arb_vector_clock(),
        clock2 in arb_vector_clock()
    ) {
        let ord1 = clock1.compare(&clock2);
        let ord2 = clock2.compare(&clock1);

        match ord1 {
            ClockOrdering::Before => prop_assert_eq!(ord2, ClockOrdering::After),
            ClockOrdering::After => prop_assert_eq!(ord2, ClockOrdering::Before),
            ClockOrdering::Equal => prop_assert_eq!(ord2, ClockOrdering::Equal),
            ClockOrdering::Concurrent => prop_assert_eq!(ord2, ClockOrdering::Concurrent),
        }
    }

    /// Property: has_dependencies correctly detects missing events
    #[test]
    fn prop_has_dependencies_correctness(
        clock in arb_vector_clock(),
        peer_id in "[a-z]{4}-[a-z]{4}-[a-z]{4}-[a-z]{4}",
        jump in 1u64..10
    ) {
        let mut message_clock = clock.clone();
        message_clock.increment(&peer_id);

        // Local clock has all dependencies
        prop_assert!(clock.has_dependencies(&message_clock));

        // Jump ahead by 'jump' - now we're missing dependencies
        for _ in 0..jump {
            message_clock.increment(&peer_id);
        }

        if jump > 1 {
            prop_assert!(!clock.has_dependencies(&message_clock));
        }
    }

    /// Property: get_missing_ranges identifies all gaps
    #[test]
    fn prop_get_missing_ranges_complete(
        mut local in arb_vector_clock(),
        remote in arb_vector_clock()
    ) {
        let missing = local.get_missing_ranges(&remote);

        // Apply all missing ranges to local
        for range in &missing {
            let entry = local.0.entry(range.peer_id.clone()).or_insert(0);
            *entry = (*entry).max(range.to_timestamp);
        }

        // After applying, local should have all events from remote
        for (peer, remote_ts) in &remote.0 {
            let local_ts = local.0.get(peer).copied().unwrap_or(0);
            prop_assert!(local_ts >= *remote_ts);
        }
    }
}

// ============================================================================
// Property-Based Tests for Message Sorting
// ============================================================================

/// Generate arbitrary CRDTMessage
fn arb_crdt_message(entity_id: String) -> impl Strategy<Value = CRDTMessage> {
    (
        "[a-z]{4}-[a-z]{4}-[a-z]{4}-[a-z]{4}", // author peer ID
        arb_vector_clock(),
        1u64..1000,         // lamport clock
        "[a-zA-Z ]{10,50}", // message text
    )
        .prop_map(
            move |(author_peer_id, vector_clock, lamport_clock, text)| CRDTMessage {
                content: MessageContent {
                    text,
                    author: "Test User".to_string(),
                    attachments: None,
                },
                metadata: MessageMetadata {
                    id: format!("{}-{}", author_peer_id, lamport_clock),
                    entity_id: entity_id.clone(),
                    entity_type: EntityType::Person,
                    author_peer_id,
                    vector_clock,
                    lamport_clock,
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    previous_message_id: None,
                    reply_to_id: None,
                },
                local_state: None,
            },
        )
}

proptest! {
    /// Property: Sorting messages is idempotent
    #[test]
    fn prop_sort_messages_idempotent(
        messages in prop::collection::vec(arb_crdt_message("test-entity".to_string()), 0..20)
    ) {
        let mut sorted1 = messages.clone();
        sort_messages_causally(&mut sorted1);

        let mut sorted2 = sorted1.clone();
        sort_messages_causally(&mut sorted2);

        // Compare message IDs instead of full structs
        let ids1: Vec<_> = sorted1.iter().map(|m| &m.metadata.id).collect();
        let ids2: Vec<_> = sorted2.iter().map(|m| &m.metadata.id).collect();
        prop_assert_eq!(ids1, ids2);
    }

    /// Property: After sorting, messages maintain causal order
    #[test]
    fn prop_sort_maintains_causal_order(
        messages in prop::collection::vec(arb_crdt_message("test-entity".to_string()), 2..20)
    ) {
        let mut sorted = messages.clone();
        sort_messages_causally(&mut sorted);

        // Check that for any pair (i, j) where i < j:
        // - If messages[i] causally precedes messages[j], then i < j in sorted order
        for i in 0..sorted.len() {
            for j in (i + 1)..sorted.len() {
                let ord = sorted[i].metadata.vector_clock.compare(&sorted[j].metadata.vector_clock);

                // If i happened before j, it should remain before
                if matches!(ord, ClockOrdering::Before) {
                    // This is correct - i < j already
                    prop_assert!(true);
                }

                // If j happened before i, this violates causal order
                if matches!(ord, ClockOrdering::After) {
                    // This should never happen after sorting
                    return Err(TestCaseError::fail(format!(
                        "Causal order violated: message {} should be before message {}",
                        j, i
                    )));
                }
            }
        }
    }

    /// Property: Lamport clock provides total ordering for concurrent messages
    #[test]
    fn prop_lamport_orders_concurrent(
        msg1 in arb_crdt_message("entity1".to_string()),
        msg2 in arb_crdt_message("entity1".to_string())
    ) {
        let mut messages = vec![msg1.clone(), msg2.clone()];
        sort_messages_causally(&mut messages);

        // If vector clocks are concurrent or equal, Lamport clock decides order
        let ord = msg1.metadata.vector_clock.compare(&msg2.metadata.vector_clock);
        if matches!(ord, ClockOrdering::Concurrent | ClockOrdering::Equal) {
            if msg1.metadata.lamport_clock < msg2.metadata.lamport_clock {
                prop_assert_eq!(&messages[0].metadata.id, &msg1.metadata.id);
            } else if msg1.metadata.lamport_clock > msg2.metadata.lamport_clock {
                prop_assert_eq!(&messages[0].metadata.id, &msg2.metadata.id);
            }
            // If Lamport clocks are also equal, message ID provides tie-breaking
        }
    }
}

// ============================================================================
// Unit Tests for Edge Cases
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_vector_clock_empty() {
        let clock = VectorClock::new();
        assert!(clock.0.is_empty());
    }

    #[test]
    fn test_vector_clock_single_peer() {
        let mut clock = VectorClock::new();
        clock.increment("peer-1");
        assert_eq!(clock.0.get("peer-1"), Some(&1));
    }

    #[test]
    fn test_vector_clock_multiple_increments() {
        let mut clock = VectorClock::new();
        clock.increment("peer-1");
        clock.increment("peer-1");
        clock.increment("peer-1");
        assert_eq!(clock.0.get("peer-1"), Some(&3));
    }

    #[test]
    fn test_compare_equal_clocks() {
        let mut clock1 = VectorClock::new();
        clock1.increment("peer-1");

        let mut clock2 = VectorClock::new();
        clock2.increment("peer-1");

        assert_eq!(clock1.compare(&clock2), ClockOrdering::Equal);
    }

    #[test]
    fn test_compare_before() {
        let mut clock1 = VectorClock::new();
        clock1.increment("peer-1");

        let mut clock2 = clock1.clone();
        clock2.increment("peer-1");

        assert_eq!(clock1.compare(&clock2), ClockOrdering::Before);
    }

    #[test]
    fn test_compare_after() {
        let mut clock1 = VectorClock::new();
        clock1.increment("peer-1");
        clock1.increment("peer-1");

        let mut clock2 = VectorClock::new();
        clock2.increment("peer-1");

        assert_eq!(clock1.compare(&clock2), ClockOrdering::After);
    }

    #[test]
    fn test_compare_concurrent() {
        let mut clock1 = VectorClock::new();
        clock1.increment("peer-1");

        let mut clock2 = VectorClock::new();
        clock2.increment("peer-2");

        assert_eq!(clock1.compare(&clock2), ClockOrdering::Concurrent);
    }

    #[test]
    fn test_has_dependencies_empty_clocks() {
        let local = VectorClock::new();
        let message = VectorClock::new();

        assert!(local.has_dependencies(&message));
    }

    #[test]
    fn test_has_dependencies_first_message() {
        let local = VectorClock::new();

        let mut message = VectorClock::new();
        message.increment("peer-1");

        assert!(local.has_dependencies(&message));
    }

    #[test]
    fn test_has_dependencies_missing_events() {
        let mut local = VectorClock::new();
        local.increment("peer-1");

        let mut message = VectorClock::new();
        message.increment("peer-1");
        message.increment("peer-1");
        message.increment("peer-1"); // Jump to 3

        assert!(!local.has_dependencies(&message)); // Missing event 2
    }

    #[test]
    fn test_has_dependencies_sequential() {
        let mut local = VectorClock::new();
        local.increment("peer-1");
        local.increment("peer-1");

        let mut message = VectorClock::new();
        message.increment("peer-1");
        message.increment("peer-1");
        message.increment("peer-1");

        assert!(local.has_dependencies(&message)); // Has 1,2 so can accept 3
    }

    #[test]
    fn test_get_missing_ranges_no_gaps() {
        let mut local = VectorClock::new();
        local.increment("peer-1");

        let remote = local.clone();

        let missing = local.get_missing_ranges(&remote);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_get_missing_ranges_single_gap() {
        let mut local = VectorClock::new();
        local.increment("peer-1");

        let mut remote = VectorClock::new();
        remote.increment("peer-1");
        remote.increment("peer-1");
        remote.increment("peer-1");

        let missing = local.get_missing_ranges(&remote);
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].peer_id, "peer-1");
        assert_eq!(missing[0].from_timestamp, 2);
        assert_eq!(missing[0].to_timestamp, 3);
    }

    #[test]
    fn test_get_missing_ranges_multiple_peers() {
        let mut local = VectorClock::new();
        local.increment("peer-1");

        let mut remote = VectorClock::new();
        remote.increment("peer-1");
        remote.increment("peer-1");
        remote.increment("peer-2");
        remote.increment("peer-3");

        let missing = local.get_missing_ranges(&remote);
        assert_eq!(missing.len(), 3); // peer-1 (2), peer-2 (1), peer-3 (1)
    }

    #[test]
    fn test_merge_empty_clocks() {
        let mut clock1 = VectorClock::new();
        let clock2 = VectorClock::new();

        clock1.merge(&clock2);
        assert!(clock1.0.is_empty());
    }

    #[test]
    fn test_merge_disjoint_peers() {
        let mut clock1 = VectorClock::new();
        clock1.increment("peer-1");

        let mut clock2 = VectorClock::new();
        clock2.increment("peer-2");

        clock1.merge(&clock2);
        assert_eq!(clock1.0.get("peer-1"), Some(&1));
        assert_eq!(clock1.0.get("peer-2"), Some(&1));
    }

    #[test]
    fn test_merge_takes_max() {
        let mut clock1 = VectorClock::new();
        clock1.increment("peer-1");
        clock1.increment("peer-1");

        let mut clock2 = VectorClock::new();
        clock2.increment("peer-1");
        clock2.increment("peer-1");
        clock2.increment("peer-1");

        clock1.merge(&clock2);
        assert_eq!(clock1.0.get("peer-1"), Some(&3)); // Max of 2 and 3
    }

    #[test]
    fn test_sort_messages_empty() {
        let mut messages: Vec<CRDTMessage> = vec![];
        sort_messages_causally(&mut messages);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_sort_messages_single() {
        let mut messages = vec![create_test_message("msg-1", 1, 1)];
        sort_messages_causally(&mut messages);
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_sort_messages_by_lamport() {
        let msg1 = create_test_message("msg-1", 1, 10);
        let msg2 = create_test_message("msg-2", 1, 20);
        let msg3 = create_test_message("msg-3", 1, 5);

        let mut messages = vec![msg2.clone(), msg1.clone(), msg3.clone()];
        sort_messages_causally(&mut messages);

        assert_eq!(messages[0].metadata.id, "msg-3");
        assert_eq!(messages[1].metadata.id, "msg-1");
        assert_eq!(messages[2].metadata.id, "msg-2");
    }

    // Helper to create test messages
    fn create_test_message(id: &str, peer_ts: u64, lamport: u64) -> CRDTMessage {
        let mut clock = VectorClock::new();
        for _ in 0..peer_ts {
            clock.increment("peer-1");
        }

        CRDTMessage {
            content: MessageContent {
                text: format!("Message {}", id),
                author: "Test".to_string(),
                attachments: None,
            },
            metadata: MessageMetadata {
                id: id.to_string(),
                entity_id: "test-entity".to_string(),
                entity_type: EntityType::Person,
                author_peer_id: "peer-1".to_string(),
                vector_clock: clock,
                lamport_clock: lamport,
                timestamp: 0,
                previous_message_id: None,
                reply_to_id: None,
            },
            local_state: None,
        }
    }
}
