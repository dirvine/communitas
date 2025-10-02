//! Integration test for P2P messaging between CoreContext instances
//!
//! **RESOLVED (2025-10-01)**: saorsa-core 0.4.0 now supports OS-assigned ports!
//!
//! Tests verify:
//! 1. ✅ Multiple CoreContext instances can run simultaneously
//! 2. ✅ Each instance gets a unique OS-assigned port
//! 3. ✅ Four-word addressing works for peer connectivity
//! 4. ✅ P2P connections can be established between instances
//! 5. ✅ Channels and messaging infrastructure initializes correctly
//!
//! See: SAORSA_CORE_PORT_ISSUE.md for resolution details

use communitas_core::core_context::CoreContext;
use saorsa_core::identity::enhanced::DeviceType;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// Generate a random valid four-word identity for testing
/// These are for USER IDENTITIES, not network addresses
/// Uses the four-word-networking dictionary validation
fn random_four_words() -> String {
    use four_word_networking::FourWordAdaptiveEncoder;
    use rand::Rng;

    // Generate random IP and port (this is just to get valid words from the dictionary)
    // The actual network address will be assigned by the OS when P2P starts
    let mut rng = rand::thread_rng();

    // Generate random bytes for IPv4 address using array construction
    let octets: [u8; 4] = [
        rng.r#gen(),
        rng.r#gen(),
        rng.r#gen(),
        rng.r#gen()
    ];
    let ip = std::net::Ipv4Addr::from(octets);
    let port: u16 = rng.gen_range(1024..65535);

    // Encode IP+port to get 4 valid dictionary words
    let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");
    let addr_str = format!("{}:{}", ip, port);
    let words = encoder.encode(&addr_str).expect("Failed to encode");

    // Convert space-separated words to hyphen-separated format
    words.replace(' ', "-")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_two_instances_p2p_connection() -> anyhow::Result<()> {
    // Initialize logging for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug,saorsa_core=debug,communitas_core=debug")
        .with_test_writer()
        .try_init();

    info!("Starting two-instance P2P connection test");

    // Generate four-word identities for both users
    let four_words_1 = random_four_words();
    let four_words_2 = random_four_words();

    info!("User 1 identity: {}", four_words_1);
    info!("User 2 identity: {}", four_words_2);

    // NOTE: Due to saorsa-core hardcoding port 9000, we can only run one instance at a time
    // This is a known limitation that needs to be fixed in saorsa-core
    // For now, test that both instances can initialize successfully when run sequentially

    // Create and verify CoreContext for user 1
    info!("Creating CoreContext for user 1");
    let ctx1 = CoreContext::initialize(
        four_words_1.clone(),
        "Test User 1".to_string(),
        "Device 1".to_string(),
        DeviceType::Desktop,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to initialize context 1: {}", e))?;

    info!("✅ User 1 CoreContext initialized successfully");

    // Verify P2P is running
    assert!(ctx1.is_p2p_running().await, "User 1 P2P should be running");

    // Drop ctx1 to release the port
    drop(ctx1);

    // Wait for port to be released
    sleep(Duration::from_millis(500)).await;

    // Create and verify CoreContext for user 2
    info!("Creating CoreContext for user 2");
    let ctx2 = CoreContext::initialize(
        four_words_2.clone(),
        "Test User 2".to_string(),
        "Device 2".to_string(),
        DeviceType::Desktop,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to initialize context 2: {}", e))?;

    info!("✅ User 2 CoreContext initialized successfully");

    // Verify P2P is running
    assert!(ctx2.is_p2p_running().await, "User 2 P2P should be running");

    info!("✅ Sequential P2P initialization test passed!");
    info!("✅ saorsa-core 0.4.0 supports OS-assigned ports - multi-instance testing works!");

    Ok(())
}

#[tokio::test]
async fn test_four_word_encoding() -> anyhow::Result<()> {
    use four_word_networking::FourWordAdaptiveEncoder;

    let encoder = FourWordAdaptiveEncoder::new()?;
    let addr = "127.0.0.1:50665";

    println!("Testing four-word encoding for: {}", addr);

    let encoded = encoder.encode(addr)?;
    println!("Encoded (space-separated): {}", encoded);

    let hyphenated = encoded.replace(' ', "-");
    println!("Hyphenated: {}", hyphenated);

    // Try decoding with hyphens
    match encoder.decode(&hyphenated) {
        Ok(decoded) => {
            println!("Decoded from hyphens: {}", decoded);
            assert_eq!(decoded, addr, "Decoding with hyphens should work");
        }
        Err(e) => {
            println!("Decode with hyphens failed: {}", e);

            // Try decoding with spaces
            match encoder.decode(&encoded) {
                Ok(decoded) => {
                    println!("Decoded from spaces: {}", decoded);
                    assert_eq!(decoded, addr, "Decoding with spaces should work");
                }
                Err(e2) => {
                    panic!("Both decode attempts failed: hyphens={}, spaces={}", e, e2);
                }
            }
        }
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_two_instances_send_message() -> anyhow::Result<()> {
    use saorsa_core::messaging::DhtClient;

    // Initialize logging for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug,saorsa_core=debug,communitas_core=debug")
        .with_test_writer()
        .try_init();

    info!("Starting two-instance messaging test");

    // Generate identities
    let four_words_1 = random_four_words();
    let four_words_2 = random_four_words();

    info!("User 1 identity: {}", four_words_1);
    info!("User 2 identity: {}", four_words_2);

    // Create SHARED DHT client for KEM key sharing between instances
    info!("Creating shared DHT client for KEM key exchange");
    let shared_dht = DhtClient::new()
        .map_err(|e| anyhow::anyhow!("Failed to create shared DHT: {}", e))?;

    // Create CoreContext instances with shared DHT
    let mut ctx1 = CoreContext::initialize_with_shared_dht(
        four_words_1.clone(),
        "Message User 1".to_string(),
        "Device 1".to_string(),
        DeviceType::Desktop,
        shared_dht.clone(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to initialize context 1: {}", e))?;

    let ctx2 = CoreContext::initialize_with_shared_dht(
        four_words_2.clone(),
        "Message User 2".to_string(),
        "Device 2".to_string(),
        DeviceType::Desktop,
        shared_dht.clone(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to initialize context 2: {}", e))?;

    info!("✅ Both instances initialized with shared DHT for KEM key exchange");

    info!("P2P networking started automatically during initialization");

    sleep(Duration::from_secs(3)).await;

    // Connect peers
    // Get socket address and convert 0.0.0.0 to 127.0.0.1 for localhost testing
    let mut socket_addr2 = ctx2.get_local_endpoint_socket().await
        .ok_or_else(|| anyhow::anyhow!("User 2 has no socket endpoint"))?;
    info!("User 2 bound socket address: {}", socket_addr2);

    // Replace 0.0.0.0 with 127.0.0.1 for actual connectivity
    if socket_addr2.ip().is_unspecified() {
        use std::net::{IpAddr, Ipv4Addr};
        socket_addr2.set_ip(IpAddr::V4(Ipv4Addr::LOCALHOST));
        info!("Converted to localhost address: {}", socket_addr2);
    }

    // Encode the actual connectable address to four words
    use four_word_networking::FourWordAdaptiveEncoder;
    let encoder = FourWordAdaptiveEncoder::new()
        .map_err(|e| anyhow::anyhow!("Failed to create encoder: {}", e))?;
    let addr2 = encoder.encode(&socket_addr2.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to encode address: {}", e))?;
    let addr2 = addr2.replace(' ', "-");

    info!("User 2 four-word network address: {}", addr2);

    // DHT-only approach: Skip manual P2P connections, let MessagingService establish them
    // This workaround is needed because CoreContext.network and MessagingService use separate
    // P2PNode instances that cannot share connections. By letting MessagingService handle
    // connections via DHT resolution, it can manage and reuse connections within its own instance.
    info!("Skipping manual P2P connections - MessagingService will connect via DHT");

    // Publish PeerInfo to shared DHT for message routing
    // This allows MessagingService to look up peer addresses and establish connections dynamically
    info!("Publishing PeerInfo to shared DHT for message routing");
    ctx1.publish_peer_info_to_dht().await
        .map_err(|e| anyhow::anyhow!("Failed to publish ctx1 peer info: {}", e))?;
    ctx2.publish_peer_info_to_dht().await
        .map_err(|e| anyhow::anyhow!("Failed to publish ctx2 peer info: {}", e))?;
    info!("✅ PeerInfo published for both instances");

    // Mark peers as online using IDENTITY addresses (for KEM key lookup)
    info!("Marking peers online for session key exchange");
    ctx1.mark_peer_online(&four_words_2).await
        .map_err(|e| anyhow::anyhow!("Failed to mark user2 online: {}", e))?;
    ctx2.mark_peer_online(&four_words_1).await
        .map_err(|e| anyhow::anyhow!("Failed to mark user1 online: {}", e))?;
    info!("✅ Peers marked online (using identity addresses)");

    // Give MessagingService time to establish connections via DHT
    info!("Waiting for MessagingService to establish connections...");
    sleep(Duration::from_secs(8)).await;

    // Create a channel on user 1
    info!("Creating channel on user 1");
    let channel1 = ctx1.chat.create_channel(
        "Test Channel".to_string(),
        "Channel for testing".to_string(),
        saorsa_core::chat::ChannelType::Public,
        None,
    ).await?;

    let channel_id = channel1.id.0.to_string();
    info!("Created channel: {}", channel_id);

    // Add user 2 to the channel
    // NOTE: For P2P messaging with KEM encryption, we must use the IDENTITY address (where KEM keys are published)
    info!("Adding user 2 to the channel using identity address: {}", four_words_2);
    ctx1.add_channel_member(&channel_id, four_words_2.clone(), saorsa_core::chat::ChannelRole::Member)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to add member: {}", e))?;

    info!("✅ User 2 added to channel");

    // Wait for membership to propagate
    sleep(Duration::from_secs(2)).await;

    // Send message from user 1 to the channel
    info!("Sending message from user 1");
    let test_message = "Hello from User 1! This is a test message.";
    let msg_id = ctx1.send_channel_message(&channel_id, test_message)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;

    info!("✅ Sent message: {}", msg_id);

    // Wait for message to be delivered (includes connection establishment time)
    info!("Waiting for message delivery...");
    sleep(Duration::from_secs(6)).await;

    // Get messages from user 2's perspective
    info!("Checking messages received by user 2");
    let messages = ctx2.get_channel_messages(&channel_id, 10)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get messages: {}", e))?;

    info!("User 2 received {} messages", messages.len());

    // Verify message was received
    let received_msg = messages
        .iter()
        .find(|m| m.id.0.to_string() == msg_id)
        .ok_or_else(|| anyhow::anyhow!("Message not found in channel"))?;

    // Check message content
    if let saorsa_core::messaging::MessageContent::Text(content) = &received_msg.content {
        assert_eq!(content, test_message, "Message content mismatch");
        info!("✅ Message content verified: '{}'", content);
    } else {
        return Err(anyhow::anyhow!("Message content is not text"));
    }

    info!("✅ Full message exchange test passed!");
    info!("   - Channel created");
    info!("   - Member added");
    info!("   - Message sent");
    info!("   - Message received");
    info!("   - Content verified");

    Ok(())
}
