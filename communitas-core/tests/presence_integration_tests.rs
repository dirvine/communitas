//! Presence Integration Tests
//!
//! Tests presence beacon advertising, discovery, TTL expiration,
//! and multi-group scenarios over real network.

use communitas_core::test_harness::TestHarness;
use saorsa_gossip_presence::PresenceStatus;
use saorsa_gossip_types::{PresenceRecord, TopicId};
use std::time::Duration;

// ============================================================================
// Presence Advertising & Discovery
// ============================================================================

#[tokio::test]
#[ignore] // Requires presence integration
async fn test_advertise_and_discover() {
    // GIVEN: 2 nodes in same group
    let harness = TestHarness::new(2).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");

    harness
        .wait_until_connected(1, Duration::from_secs(5))
        .await
        .expect("connection failed");

    let topic_id = TopicId::new([1u8; 32]);

    // Both join same group
    let node_a = harness.get_node(0).await.expect("node 0 not found");
    let node_b = harness.get_node(1).await.expect("node 1 not found");

    {
        let node_a_guard = node_a.read().await;
        node_a_guard
            .join_group(topic_id, "test-group")
            .await
            .expect("join failed");
    }

    {
        let node_b_guard = node_b.read().await;
        node_b_guard
            .join_group(topic_id, "test-group")
            .await
            .expect("join failed");
    }

    // WHEN: Node A advertises presence
    let presence_a = {
        let node = node_a.read().await;
        node.presence
            .as_ref()
            .expect("presence not initialized")
            .clone()
    };

    let four_words_a = node_a.read().await.four_words.clone();
    let peer_id_a = node_a.read().await.peer_id;

    let beacon = PresenceRecord::with_four_words(
        [0u8; 32],
        vec![node_a.read().await.bootstrap_addr()],
        900, // 15 min TTL
        four_words_a.clone(),
    );

    {
        let presence_guard = presence_a.write().await;
        presence_guard
            .handle_beacon(topic_id, peer_id_a, beacon)
            .await
            .expect("handle beacon failed");
    }

    // THEN: Node B should discover Node A via presence
    let presence_b = {
        let node = node_b.read().await;
        node.presence
            .as_ref()
            .expect("presence not initialized")
            .clone()
    };

    {
        let presence_guard = presence_b.write().await;
        presence_guard
            .handle_beacon(topic_id, peer_id_a, beacon.clone())
            .await
            .expect("handle beacon failed");
    }

    tokio::time::sleep(Duration::from_millis(200)).await;

    {
        let presence_guard = presence_b.read().await;
        let status = presence_guard.get_status(peer_id_a, topic_id).await;
        assert_eq!(status, PresenceStatus::Online);
    }

    let records = presence_guard.get_group_presence(topic_id).await;
    let record = records.get(&peer_id_a).expect("record missing");
    assert_eq!(record.four_words.as_deref(), Some(four_words_a.as_str()));

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
#[ignore] // Requires presence integration
async fn test_presence_ttl_expiry() {
    // GIVEN: Node with short TTL beacon
    let harness = TestHarness::new(2).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");

    let topic_id = TopicId::new([1u8; 32]);

    let node_a = harness.get_node(0).await.expect("node 0 not found");
    let node_b = harness.get_node(1).await.expect("node 1 not found");

    {
        let node_guard = node_a.read().await;
        node_guard
            .join_group(topic_id, "test-group")
            .await
            .expect("join failed");
    }

    {
        let node_guard = node_b.read().await;
        node_guard
            .join_group(topic_id, "test-group")
            .await
            .expect("join failed");
    }

    // WHEN: Node A advertises with 1 second TTL
    let presence_a = {
        let node = node_a.read().await;
        node.presence
            .as_ref()
            .expect("presence not initialized")
            .clone()
    };

    let four_words_a = node_a.read().await.four_words.clone();
    let peer_id_a = node_a.read().await.peer_id;

    let beacon = PresenceRecord::with_four_words(
        [0u8; 32],
        vec![node_a.read().await.bootstrap_addr()],
        1, // 1 second TTL
        four_words_a.clone(),
    );

    {
        let presence_guard = presence_a.write().await;
        presence_guard
            .handle_beacon(topic_id, peer_id_a, beacon)
            .await
            .expect("handle beacon failed");
    }

    let presence_b = {
        let node = node_b.read().await;
        node.presence
            .as_ref()
            .expect("presence not initialized")
            .clone()
    };

    {
        let presence_guard = presence_b.write().await;
        presence_guard
            .handle_beacon(topic_id, peer_id_a, beacon.clone())
            .await
            .expect("handle beacon failed");
    }

    let presence_guard = presence_b.read().await;
    let status = presence_guard.get_status(peer_id_a, topic_id).await;
    assert_eq!(status, PresenceStatus::Online);

    // THEN: After TTL expires, should not be discoverable
    tokio::time::sleep(Duration::from_secs(2)).await;

    let presence_guard = presence_b.read().await;
    let status = presence_guard.get_status(peer_id_a, topic_id).await;
    assert_eq!(status, PresenceStatus::Offline);

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
#[ignore] // Requires presence integration
async fn test_multi_group_presence() {
    // GIVEN: User present in multiple groups
    let harness = TestHarness::new(2).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");

    let topic1 = TopicId::new([1u8; 32]);
    let topic2 = TopicId::new([2u8; 32]);

    let node_a = harness.get_node(0).await.expect("node 0 not found");

    // Join both groups
    {
        let node_guard = node_a.read().await;
        node_guard
            .join_group(topic1, "group-1")
            .await
            .expect("join 1 failed");
        node_guard
            .join_group(topic2, "group-2")
            .await
            .expect("join 2 failed");
    }

    // Advertise in both groups
    let presence_a = {
        let node = node_a.read().await;
        node.presence
            .as_ref()
            .expect("presence not initialized")
            .clone()
    };

    let four_words_a = node_a.read().await.four_words.clone();
    let peer_id_a = node_a.read().await.peer_id;
    let addr_a = node_a.read().await.bootstrap_addr();

    for topic in [topic1, topic2] {
        let beacon = PresenceRecord::with_four_words(
            [0u8; 32],
            vec![addr_a.clone()],
            900,
            four_words_a.clone(),
        );

        let presence_guard = presence_a.write().await;
        presence_guard
            .handle_beacon(topic, peer_id_a, beacon)
            .await
            .expect("handle beacon failed");
    }

    let presence_guard = presence_a.read().await;
    let records_1 = presence_guard.get_group_presence(topic1).await;
    let records_2 = presence_guard.get_group_presence(topic2).await;

    assert!(records_1.contains_key(&peer_id_a));
    assert!(records_2.contains_key(&peer_id_a));

    harness.cleanup().await.expect("cleanup failed");
}

// ============================================================================
// Presence with Network Chaos
// ============================================================================

#[tokio::test]
#[ignore] // Requires presence integration and network
async fn test_presence_with_packet_loss() {
    // GIVEN: 3 nodes with 30% packet loss
    let harness = TestHarness::new(3).await.expect("harness creation failed");

    // Set packet loss between all pairs
    for i in 0..3 {
        for j in (i + 1)..3 {
            harness.set_loss(i, j, 0.3).await;
        }
    }

    harness.mesh().await.expect("mesh setup failed");

    let topic_id = TopicId::new([1u8; 32]);

    // All join same group
    for i in 0..3 {
        let node = harness.get_node(i).await.expect("node not found");
        let node_guard = node.read().await;
        node_guard
            .join_group(topic_id, "test-group")
            .await
            .expect("join failed");
    }

    // WHEN: All advertise presence
    for i in 0..3 {
        let node = harness.get_node(i).await.expect("node not found");
        let presence = {
            let node_guard = node.read().await;
            node_guard
                .presence
                .as_ref()
                .expect("presence not initialized")
                .clone()
        };

        let four_words = node.read().await.four_words.clone();
        let peer_id = node.read().await.peer_id;
        let addr = node.read().await.bootstrap_addr();

        let beacon = PresenceRecord::with_four_words([0u8; 32], vec![addr], 900, four_words);

        let presence_guard = presence.write().await;
        presence_guard
            .handle_beacon(topic_id, peer_id, beacon)
            .await
            .expect("handle beacon failed");
    }

    // THEN: Eventually all should discover each other despite packet loss
    tokio::time::sleep(Duration::from_millis(200)).await;

    for i in 0..3 {
        let node = harness.get_node(i).await.expect("node not found");
        let presence = {
            let node_guard = node.read().await;
            node_guard
                .presence
                .as_ref()
                .expect("presence not initialized")
                .clone()
        };
        let presence_guard = presence.read().await;
        let records = presence_guard.get_group_presence(topic_id).await;
        assert_eq!(records.len(), 3, "node {} should see all peers", i);
    }

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
#[ignore] // Requires presence integration
async fn test_presence_during_partition() {
    // GIVEN: 4 nodes, partitioned into [0,1] and [2,3]
    let harness = TestHarness::new(4).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");

    harness
        .wait_until_connected(6, Duration::from_secs(10))
        .await
        .expect("connection failed");

    let topic_id = TopicId::new([1u8; 32]);

    // All join group
    for i in 0..4 {
        let node = harness.get_node(i).await.expect("node not found");
        let node_guard = node.read().await;
        node_guard
            .join_group(topic_id, "test-group")
            .await
            .expect("join failed");
    }

    // Partition network
    harness
        .partition(&[0, 1], &[2, 3])
        .await
        .expect("partition failed");

    // WHEN: Nodes advertise in their partitions
    let partitions = vec![vec![0usize, 1], vec![2usize, 3]];
    for partition in &partitions {
        for &sender_id in partition {
            let sender = harness.get_node(sender_id).await.expect("node not found");
            let sender_guard = sender.read().await;
            let beacon = PresenceRecord::with_four_words(
                [0u8; 32],
                vec![sender_guard.bootstrap_addr()],
                900,
                sender_guard.four_words.clone(),
            );
            let peer_id = sender_guard.peer_id;
            drop(sender_guard);

            for &receiver_id in partition {
                let receiver = harness.get_node(receiver_id).await.expect("node not found");
                let presence = {
                    let receiver_guard = receiver.read().await;
                    receiver_guard
                        .presence
                        .as_ref()
                        .expect("presence not initialized")
                        .clone()
                };
                let presence_guard = presence.write().await;
                presence_guard
                    .handle_beacon(topic_id, peer_id, beacon.clone())
                    .await
                    .expect("handle beacon failed");
            }
        }
    }

    // THEN: Partition A should only see [0,1], Partition B only [2,3]
    for (idx, partition) in partitions.iter().enumerate() {
        for &node_id in partition {
            let node = harness.get_node(node_id).await.expect("node not found");
            let presence = {
                let node_guard = node.read().await;
                node_guard
                    .presence
                    .as_ref()
                    .expect("presence not initialized")
                    .clone()
            };
            let records = presence.read().await.get_group_presence(topic_id).await;
            assert_eq!(
                records.len(),
                partition.len(),
                "partition {} node {} should only see its partition",
                idx,
                node_id
            );
        }
    }

    // Heal network
    harness.heal().await.expect("heal failed");

    // Re-advertise after healing
    for sender_id in 0..4 {
        let sender = harness.get_node(sender_id).await.expect("node not found");
        let sender_guard = sender.read().await;
        let beacon = PresenceRecord::with_four_words(
            [0u8; 32],
            vec![sender_guard.bootstrap_addr()],
            900,
            sender_guard.four_words.clone(),
        );
        let peer_id = sender_guard.peer_id;
        drop(sender_guard);

        for receiver_id in 0..4 {
            let receiver = harness.get_node(receiver_id).await.expect("node not found");
            let presence = {
                let receiver_guard = receiver.read().await;
                receiver_guard
                    .presence
                    .as_ref()
                    .expect("presence not initialized")
                    .clone()
            };
            let presence_guard = presence.write().await;
            presence_guard
                .handle_beacon(topic_id, peer_id, beacon.clone())
                .await
                .expect("handle beacon failed");
        }
    }

    // THEN: All should see all after healing
    for node_id in 0..4 {
        let node = harness.get_node(node_id).await.expect("node not found");
        let presence = {
            let node_guard = node.read().await;
            node_guard
                .presence
                .as_ref()
                .expect("presence not initialized")
                .clone()
        };
        let records = presence.read().await.get_group_presence(topic_id).await;
        assert_eq!(records.len(), 4, "node {} should see all peers", node_id);
    }

    harness.cleanup().await.expect("cleanup failed");
}
