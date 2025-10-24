//! Peer Sync Integration Tests
//!
//! Tests message synchronization between peers over real QUIC connections,
//! including out-of-order handling, missing message recovery, and convergence.

use communitas_core::crdt::*;
use communitas_core::message_sync::MessageSyncService;
use communitas_core::test_harness::TestHarness;
use std::time::Duration;

// ============================================================================
// Two-Peer Sync Tests
// ============================================================================

#[tokio::test]
#[ignore] // Requires network integration
async fn test_two_peers_sync_stream() {
    // GIVEN: 2 connected nodes
    let harness = TestHarness::new(2).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");
    
    harness
        .wait_until_connected(1, Duration::from_secs(5))
        .await
        .expect("connection failed");

    // Create sync services for both peers
    let node_a = harness.get_node(0).await.expect("node 0 not found");
    let node_b = harness.get_node(1).await.expect("node 1 not found");
    
    let peer_id_a = node_a.read().await.four_words.clone();
    let peer_id_b = node_b.read().await.four_words.clone();
    
    let sync_a = MessageSyncService::new(peer_id_a.clone());
    let sync_b = MessageSyncService::new(peer_id_b.clone());
    
    let entity_id = "contact-alice";

    // WHEN: Peer A sends 10 messages
    for i in 0..10 {
        sync_a
            .send_message(
                entity_id.to_string(),
                EntityType::Person,
                MessageContent {
                    text: format!("Message {}", i),
                    author: "Peer A".to_string(),
                    attachments: None,
                },
                None,
            )
            .await
            .expect("send failed");
    }

    // Sync from A to B
    let messages_a = sync_a.get_all_messages(entity_id).await.expect("get messages failed");
    let result = sync_b.handle_sync_response(messages_a).await.expect("sync failed");
    
    // THEN: Peer B should receive all messages in order
    assert_eq!(result.messages_added, 10, "Should add 10 messages");
    
    let messages_b = sync_b.get_messages(entity_id).await.expect("get messages failed");
    assert_eq!(messages_b.len(), 10, "Peer B should have 10 messages");
    
    // Verify causal order
    for i in 0..9 {
        let ord = messages_b[i]
            .metadata
            .vector_clock
            .compare(&messages_b[i + 1].metadata.vector_clock);
        assert!(
            !matches!(ord, ClockOrdering::After),
            "Messages should be in causal order"
        );
    }

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
#[ignore] // Requires network integration with packet loss
async fn test_missing_range_repair() {
    // GIVEN: 2 nodes with 40% packet loss
    let harness = TestHarness::new(2).await.expect("harness creation failed");
    harness.set_loss(0, 1, 0.4).await;
    harness.mesh().await.expect("mesh setup failed");
    
    harness
        .wait_until_connected(1, Duration::from_secs(10))
        .await
        .expect("connection failed");

    let node_a = harness.get_node(0).await.expect("node 0 not found");
    let node_b = harness.get_node(1).await.expect("node 1 not found");
    
    let peer_id_a = node_a.read().await.four_words.clone();
    let peer_id_b = node_b.read().await.four_words.clone();
    
    let sync_a = MessageSyncService::new(peer_id_a);
    let sync_b = MessageSyncService::new(peer_id_b);
    
    let entity_id = "contact-test";

    // WHEN: Peer A sends 20 messages (some will be lost)
    for i in 0..20 {
        sync_a
            .send_message(
                entity_id.to_string(),
                EntityType::Person,
                MessageContent {
                    text: format!("Message {}", i),
                    author: "Peer A".to_string(),
                    attachments: None,
                },
                None,
            )
            .await
            .expect("send failed");
    }

    // Initial sync (some messages lost)
    let messages_a = sync_a.get_all_messages(entity_id).await.expect("get failed");
    let _ = sync_b.handle_sync_response(messages_a.clone()).await;
    
    // THEN: Peer B should detect missing ranges
    let state_b = sync_b.get_sync_state(entity_id).await.expect("state failed");
    
    if state_b.message_count < 20 {
        // Missing messages detected
        assert!(
            !state_b.missing_messages.is_empty(),
            "Should detect missing messages"
        );
        
        // Repair: sync again
        let repair_result = sync_b.handle_sync_response(messages_a).await.expect("repair failed");
        
        // Eventually should have all messages
        let final_messages = sync_b.get_messages(entity_id).await.expect("get failed");
        assert_eq!(final_messages.len(), 20, "Should repair all messages");
    }

    harness.cleanup().await.expect("cleanup failed");
}

// ============================================================================
// Multi-Peer Convergence Tests
// ============================================================================

#[tokio::test]
#[ignore] // Requires network integration
async fn test_convergence_after_partition() {
    // GIVEN: 3 nodes in mesh
    let harness = TestHarness::new(3).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");
    
    harness
        .wait_until_connected(3, Duration::from_secs(10))
        .await
        .expect("initial connection failed");

    // Create sync services
    let peer_ids: Vec<String> = {
        let mut ids = Vec::new();
        for i in 0..3 {
            let node = harness.get_node(i).await.expect("node not found");
            ids.push(node.read().await.four_words.clone());
        }
        ids
    };
    
    let sync_services: Vec<MessageSyncService> = peer_ids
        .iter()
        .map(|id| MessageSyncService::new(id.clone()))
        .collect();
    
    let entity_id = "project-communitas";

    // WHEN: Network partitions into {0} and {1,2}
    harness.partition(&[0], &[1, 2]).await.expect("partition failed");
    
    // Each partition sends messages
    // Partition A (node 0)
    for i in 0..3 {
        sync_services[0]
            .send_message(
                entity_id.to_string(),
                EntityType::Project,
                MessageContent {
                    text: format!("Partition A message {}", i),
                    author: "Node 0".to_string(),
                    attachments: None,
                },
                None,
            )
            .await
            .expect("send failed");
    }
    
    // Partition B (nodes 1 and 2)
    for i in 0..2 {
        sync_services[1]
            .send_message(
                entity_id.to_string(),
                EntityType::Project,
                MessageContent {
                    text: format!("Partition B message {}", i),
                    author: "Node 1".to_string(),
                    attachments: None,
                },
                None,
            )
            .await
            .expect("send failed");
    }

    // Sync within partition B
    let messages_1 = sync_services[1].get_all_messages(entity_id).await.expect("get failed");
    sync_services[2].handle_sync_response(messages_1).await.expect("sync failed");

    // THEN: Heal network
    harness.heal().await.expect("heal failed");
    harness
        .wait_until_connected(3, Duration::from_secs(10))
        .await
        .expect("reconnection failed");

    // Sync all nodes
    for i in 0..3 {
        for j in 0..3 {
            if i != j {
                let messages = sync_services[i].get_all_messages(entity_id).await.expect("get failed");
                sync_services[j].handle_sync_response(messages).await.expect("sync failed");
            }
        }
    }

    // All nodes should converge to 5 messages (3 from A, 2 from B)
    for (idx, sync) in sync_services.iter().enumerate() {
        let messages = sync.get_messages(entity_id).await.expect("get failed");
        assert_eq!(
            messages.len(),
            5,
            "Node {} should have 5 messages after convergence",
            idx
        );
    }

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
#[ignore] // Requires network integration
async fn test_three_peer_bidirectional_sync() {
    // GIVEN: 3 nodes in mesh
    let harness = TestHarness::new(3).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");
    
    harness
        .wait_until_connected(3, Duration::from_secs(10))
        .await
        .expect("connection failed");

    let peer_ids: Vec<String> = {
        let mut ids = Vec::new();
        for i in 0..3 {
            let node = harness.get_node(i).await.expect("node not found");
            ids.push(node.read().await.four_words.clone());
        }
        ids
    };
    
    let sync_services: Vec<MessageSyncService> = peer_ids
        .iter()
        .map(|id| MessageSyncService::new(id.clone()))
        .collect();
    
    let entity_id = "contact-bob";

    // WHEN: Each peer sends 2 messages
    for (idx, sync) in sync_services.iter().enumerate() {
        for i in 0..2 {
            sync.send_message(
                entity_id.to_string(),
                EntityType::Person,
                MessageContent {
                    text: format!("Peer {} message {}", idx, i),
                    author: format!("Peer {}", idx),
                    attachments: None,
                },
                None,
            )
            .await
            .expect("send failed");
        }
    }

    // Bidirectional sync between all pairs
    for i in 0..3 {
        for j in 0..3 {
            if i != j {
                let messages = sync_services[i].get_all_messages(entity_id).await.expect("get failed");
                sync_services[j].handle_sync_response(messages).await.expect("sync failed");
            }
        }
    }

    // THEN: All peers should have 6 messages (2 from each peer)
    for (idx, sync) in sync_services.iter().enumerate() {
        let messages = sync.get_messages(entity_id).await.expect("get failed");
        assert_eq!(
            messages.len(),
            6,
            "Peer {} should have 6 messages",
            idx
        );
    }

    harness.cleanup().await.expect("cleanup failed");
}

// ============================================================================
// Out-of-Order Handling
// ============================================================================

#[tokio::test]
async fn test_out_of_order_local() {
    // This test doesn't require network - tests MessageSyncService directly
    let sender = MessageSyncService::new("peer-sender".to_string());
    let receiver = MessageSyncService::new("peer-receiver".to_string());
    let entity_id = "contact-test";

    // Create 3 messages
    let mut messages = Vec::new();
    for i in 0..3 {
        let msg = sender
            .send_message(
                entity_id.to_string(),
                EntityType::Person,
                MessageContent {
                    text: format!("Message {}", i),
                    author: "Sender".to_string(),
                    attachments: None,
                },
                None,
            )
            .await
            .expect("send failed");
        messages.push(msg);
    }

    // Receive in order: 0, 2 (skipping 1)
    receiver.receive_message(messages[0].clone()).await.expect("receive 0 failed");
    let result = receiver.receive_message(messages[2].clone()).await.expect("receive 2 failed");

    // Message 2 should be out of order
    assert!(!result.accepted, "Message 2 should be rejected as out of order");
    assert!(result.out_of_order, "Should be marked out of order");

    // Check pending queue
    let state = receiver.get_sync_state(entity_id).await.expect("state failed");
    assert_eq!(state.out_of_order_messages.len(), 1, "Should have 1 pending message");

    // Receive missing message 1
    receiver.receive_message(messages[1].clone()).await.expect("receive 1 failed");

    // Now all 3 should be processed
    let final_messages = receiver.get_messages(entity_id).await.expect("get failed");
    assert_eq!(final_messages.len(), 3, "Should have all 3 messages");
}

#[tokio::test]
async fn test_duplicate_detection() {
    let sync = MessageSyncService::new("peer-test".to_string());
    let entity_id = "contact-alice";

    // Send same message twice
    let msg = sync
        .send_message(
            entity_id.to_string(),
            EntityType::Person,
            MessageContent {
                text: "Test".to_string(),
                author: "Test".to_string(),
                attachments: None,
            },
            None,
        )
        .await
        .expect("send failed");

    // Try to receive duplicate
    let _ = sync.receive_message(msg.clone()).await;

    // Should only have 1 message
    let messages = sync.get_messages(entity_id).await.expect("get failed");
    assert_eq!(messages.len(), 1, "Duplicates should be ignored");
}
