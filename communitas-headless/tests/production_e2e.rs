use communitas_core::types::DeviceType;
use communitas_core::crdt::EntityType;
use communitas_core::CoreContext;
use std::time::Duration;
use tokio::time::sleep;

const DO_BOOTSTRAP_NODE: &str = "ocean-forest-moon-star";

#[tokio::test]
async fn test_production_network_gossip_sync() {
    // 1. Setup Alice
    let temp_dir_alice = tempfile::tempdir().unwrap();
    let alice_identity = communitas_core::identity::generate_id_words().unwrap();
    println!("Alice Identity: {}", alice_identity);

    let mut alice = CoreContext::initialize(
        alice_identity.clone(),
        "Alice".to_string(),
        "Alice-Device".to_string(),
        DeviceType::Desktop,
        temp_dir_alice.path().to_path_buf(),
    )
    .await
    .expect("Failed to init Alice");

    // Start networking for Alice
    let alice_conn = alice.start_networking(None).await.expect("Failed to start networking for Alice");
    println!("Alice connected as {}", alice_conn);

    // Connect Alice to DO
    alice.connect_to_peer(DO_BOOTSTRAP_NODE).await.expect("Alice failed to connect to DO");
    
    // Wait for connection stability
    sleep(Duration::from_secs(5)).await;

    // 2. Setup Bob
    let temp_dir_bob = tempfile::tempdir().unwrap();
    let bob_identity = communitas_core::identity::generate_id_words().unwrap();
    println!("Bob Identity: {}", bob_identity);

    let mut bob = CoreContext::initialize(
        bob_identity.clone(),
        "Bob".to_string(),
        "Bob-Device".to_string(),
        DeviceType::Mobile,
        temp_dir_bob.path().to_path_buf(),
    )
    .await
    .expect("Failed to init Bob");

    // Start networking for Bob
    let bob_conn = bob.start_networking(None).await.expect("Failed to start networking for Bob");
    println!("Bob connected as {}", bob_conn);

    // Connect Bob to DO
    bob.connect_to_peer(DO_BOOTSTRAP_NODE).await.expect("Bob failed to connect to DO");

    // Wait for connection stability
    sleep(Duration::from_secs(5)).await;

    // 3. Alice gossips a message
    // We use the low-level gossip context to verify network transport and anti-entropy
    let message = b"Hello Production Network!".to_vec();
    
    if let Some(gossip) = &alice.gossip {
        gossip.store_message(message.clone()).await.expect("Alice failed to store message");
        println!("Alice stored message in local CRDT");
    } else {
        panic!("Alice gossip context not initialized");
    }

    // 4. Bob waits to receive the message via gossip sync
    // AE interval is 60s. Two hops (Alice->DO->Bob) could take up to 120s+ depending on scheduling.
    // We wait up to 3 minutes.
    let mut found = false;
    for i in 0..90 { // 90 attempts * 2s = 180s timeout
        if let Some(gossip) = &bob.gossip {
            let messages = gossip.get_all_messages().await.expect("Bob failed to get messages");
            if messages.iter().any(|m| m == &message) {
                println!("Bob received the message! ({})", i);
                found = true;
                break;
            }
        }
        if i % 5 == 0 {
            println!("Bob waiting for sync... ({}s)", i * 2);
        }
        sleep(Duration::from_secs(2)).await;
    }

    if !found {
        panic!("Bob failed to receive gossip message from Alice via DO network");
    }

    // Cleanup
    alice.stop_networking().await.ok();
    bob.stop_networking().await.ok();
}
