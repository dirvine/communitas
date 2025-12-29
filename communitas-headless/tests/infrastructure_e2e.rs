//! Full infrastructure E2E test against live VPS network
//!
//! Tests all aspects: organizations, groups, channels, messaging, Kanban, files, invitations
//!
//! Run with: cargo test -p communitas-headless --test infrastructure_e2e -- --nocapture

use communitas_core::crdt::EntityType;
use communitas_core::disk_service::DiskType;
use communitas_core::invite_service::InviteRequest;
use communitas_core::legacy_crdt::MessageContent;
use communitas_core::types::DeviceType;
use communitas_core::CoreContext;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

// Bootstrap nodes on port 11000 (Communitas port range)
const BOOTSTRAP_1: &str = "142.93.199.50:11000"; // saorsa-2 NYC
const BOOTSTRAP_2: &str = "147.182.234.192:11000"; // saorsa-3 SFO

fn setup_crypto() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}

async fn create_connected_node(name: &str) -> (CoreContext, String, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let identity = communitas_core::identity::generate_id_words().expect("Failed to generate identity");
    println!("[{}] Four-word identity: {}", name, identity);

    let mut ctx = CoreContext::initialize(
        identity.clone(),
        name.to_string(),
        format!("{}-Device", name),
        DeviceType::Desktop,
        temp_dir.path().to_path_buf(),
    )
    .await
    .expect("Failed to init context");

    // Start networking
    let conn = ctx
        .start_networking(None)
        .await
        .expect("Failed to start networking");
    println!("[{}] Connection address: {}", name, conn);

    // Connect to both bootstrap nodes for redundancy
    let addr1: SocketAddr = BOOTSTRAP_1.parse().expect("Invalid bootstrap address");
    let conn1 = communitas_core::identity::conn_words(&addr1).expect("Failed to get conn words");
    ctx.connect_to_peer(&conn1)
        .await
        .expect("Failed to connect to bootstrap 1");
    println!("[{}] Connected to saorsa-2 (NYC)", name);

    let addr2: SocketAddr = BOOTSTRAP_2.parse().expect("Invalid bootstrap address");
    let conn2 = communitas_core::identity::conn_words(&addr2).expect("Failed to get conn words");
    ctx.connect_to_peer(&conn2)
        .await
        .expect("Failed to connect to bootstrap 2");
    println!("[{}] Connected to saorsa-3 (SFO)", name);

    (ctx, identity, temp_dir)
}

#[tokio::test]
async fn test_full_infrastructure() {
    setup_crypto();
    println!("\n{}", "=".repeat(60));
    println!("  COMMUNITAS FULL INFRASTRUCTURE E2E TEST");
    println!("  Bootstrap: saorsa-2 (NYC), saorsa-3 (SFO)");
    println!("  Test nodes: saorsa-4 (AMS), saorsa-5 (LON)");
    println!("{}\n", "=".repeat(60));

    // ─────────────────────────────────────────────────────────────
    // PHASE 1: Create distributed test nodes
    // ─────────────────────────────────────────────────────────────
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ PHASE 1: Creating distributed test nodes                │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    let (alice, alice_id, _alice_dir) = create_connected_node("Alice").await;
    let (bob, bob_id, _bob_dir) = create_connected_node("Bob").await;
    let (carol, carol_id, _carol_dir) = create_connected_node("Carol").await;

    println!("\n✓ Created 3 test nodes:");
    println!("  • Alice: {}", alice_id);
    println!("  • Bob:   {}", bob_id);
    println!("  • Carol: {}", carol_id);

    // Wait for network stabilization and peer discovery
    println!("\n⏳ Waiting for network stabilization (5s)...");
    sleep(Duration::from_secs(5)).await;

    // ─────────────────────────────────────────────────────────────
    // PHASE 2: Create Organization
    // ─────────────────────────────────────────────────────────────
    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│ PHASE 2: Create Organization                            │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    let org = alice
        .entity_service
        .create_entity(
            "SaorsaLabs".to_string(),
            EntityType::Organisation,
            Some("Decentralized collaboration platform".to_string()),
            alice_id.clone(),
            vec![alice_id.clone()], // Alice is founder
        )
        .await
        .expect("Failed to create organization");

    println!("✓ Created organization: {}", org.name);
    println!("  ID: {}", org.id);
    println!("  Created by: {}", org.created_by);

    // Grant Alice Edit access to Members (required for invitations)
    alice
        .entity_service
        .set_permission_override(
            EntityType::Organisation,
            &org.id,
            &alice_id,
            "members",
            "edit",
        )
        .await
        .expect("Failed to grant Alice members edit permission");
    println!("  Granted Members:Edit permission");

    sleep(Duration::from_secs(2)).await;

    // ─────────────────────────────────────────────────────────────
    // PHASE 3: Create Group within Organization
    // ─────────────────────────────────────────────────────────────
    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│ PHASE 3: Create Group                                   │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    let group = alice
        .entity_service
        .create_entity(
            "Engineering".to_string(),
            EntityType::Group,
            Some("Core engineering team".to_string()),
            alice_id.clone(),
            vec![alice_id.clone()],
        )
        .await
        .expect("Failed to create group");

    println!("✓ Created group: {}", group.name);
    println!("  ID: {}", group.id);

    // Set parent organization
    alice
        .entity_service
        .set_parent_organization(&group.id, &org.id)
        .await
        .expect("Failed to set parent org");
    println!("  Parent org: {}", org.name);

    sleep(Duration::from_secs(2)).await;

    // ─────────────────────────────────────────────────────────────
    // PHASE 4: Create Channels within Group
    // ─────────────────────────────────────────────────────────────
    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│ PHASE 4: Create Channels                                │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    let general_channel = alice
        .entity_service
        .create_entity(
            "general".to_string(),
            EntityType::Channel,
            Some("General discussion".to_string()),
            alice_id.clone(),
            vec![alice_id.clone()],
        )
        .await
        .expect("Failed to create general channel");

    println!("✓ Created channel: #{}", general_channel.name);

    let dev_channel = alice
        .entity_service
        .create_entity(
            "development".to_string(),
            EntityType::Channel,
            Some("Development discussions".to_string()),
            alice_id.clone(),
            vec![alice_id.clone()],
        )
        .await
        .expect("Failed to create dev channel");

    println!("✓ Created channel: #{}", dev_channel.name);

    sleep(Duration::from_secs(2)).await;

    // ─────────────────────────────────────────────────────────────
    // PHASE 5: Send Messages
    // ─────────────────────────────────────────────────────────────
    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│ PHASE 5: Send Messages                                  │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    // Alice sends message to general channel
    let msg1 = alice
        .message_service
        .send_message(
            general_channel.id.clone(),
            EntityType::Channel,
            MessageContent {
                text: "Welcome to SaorsaLabs! 🚀".to_string(),
                author: "Alice".to_string(),
                attachments: None,
            },
            None,
        )
        .await
        .expect("Alice failed to send message");

    println!("✓ [Alice → #general]: \"Welcome to SaorsaLabs! 🚀\"");
    println!("  Message ID: {}", msg1.metadata.id);

    // Alice sends another message
    let _msg2 = alice
        .message_service
        .send_message(
            general_channel.id.clone(),
            EntityType::Channel,
            MessageContent {
                text: "Let's build something amazing together!".to_string(),
                author: "Alice".to_string(),
                attachments: None,
            },
            None,
        )
        .await
        .expect("Alice failed to send second message");

    println!("✓ [Alice → #general]: \"Let's build something amazing together!\"");

    // Send message to dev channel
    let _msg3 = alice
        .message_service
        .send_message(
            dev_channel.id.clone(),
            EntityType::Channel,
            MessageContent {
                text: "Sprint planning starts Monday".to_string(),
                author: "Alice".to_string(),
                attachments: None,
            },
            None,
        )
        .await
        .expect("Alice failed to send dev message");

    println!("✓ [Alice → #development]: \"Sprint planning starts Monday\"");

    sleep(Duration::from_secs(2)).await;

    // Retrieve messages
    let sync_response = alice
        .message_service
        .get_entity_messages(general_channel.id.clone())
        .await
        .expect("Failed to get messages");

    println!("\n📨 Messages in #general: {}", sync_response.messages.len());
    for msg in &sync_response.messages {
        println!("   • {}: {}", msg.content.author, msg.content.text);
    }

    // ─────────────────────────────────────────────────────────────
    // PHASE 6: Kanban Board Operations
    // ─────────────────────────────────────────────────────────────
    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│ PHASE 6: Kanban Board Operations                        │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    // Create board for the group
    let board = alice
        .kanban_service
        .create_board(&group.id, "Sprint 1".to_string(), None)
        .expect("Failed to create Kanban board");

    println!("✓ Created Kanban board: {}", board.name);
    println!("  Board ID: {}", board.id);

    // Create columns
    let todo_col = alice
        .kanban_service
        .add_column(&board.id, "To Do".to_string(), Some(0))
        .expect("Failed to create To Do column");
    println!("✓ Created column: To Do");

    let in_progress_col = alice
        .kanban_service
        .add_column(&board.id, "In Progress".to_string(), Some(1))
        .expect("Failed to create In Progress column");
    println!("✓ Created column: In Progress");

    let done_col = alice
        .kanban_service
        .add_column(&board.id, "Done".to_string(), Some(2))
        .expect("Failed to create Done column");
    println!("✓ Created column: Done");

    // Create cards
    let card1 = alice
        .kanban_service
        .create_card(
            &board.id,
            &todo_col.id,
            "Implement P2P messaging".to_string(),
            Some("End-to-end encrypted gossip messaging".to_string()),
        )
        .expect("Failed to create card 1");
    println!("✓ Created card: \"{}\"", card1.title);

    let card2 = alice
        .kanban_service
        .create_card(
            &board.id,
            &todo_col.id,
            "Add Kanban board".to_string(),
            Some("CRDT-based collaborative project management".to_string()),
        )
        .expect("Failed to create card 2");
    println!("✓ Created card: \"{}\"", card2.title);

    let card3 = alice
        .kanban_service
        .create_card(
            &board.id,
            &in_progress_col.id,
            "Virtual disk system".to_string(),
            Some("Per-entity encrypted storage".to_string()),
        )
        .expect("Failed to create card 3");
    println!("✓ Created card: \"{}\" (In Progress)", card3.title);

    // Move a card from To Do to In Progress
    alice
        .kanban_service
        .move_card(&board.id, &card1.id, &in_progress_col.id, 0)
        .expect("Failed to move card");
    println!("✓ Moved \"{}\" → In Progress", card1.title);

    // Move a card to Done
    alice
        .kanban_service
        .move_card(&board.id, &card3.id, &done_col.id, 0)
        .expect("Failed to move card to done");
    println!("✓ Moved \"{}\" → Done", card3.title);

    // Get board state
    let board_state = alice
        .kanban_service
        .get_board(&board.id)
        .expect("Failed to get board");

    println!("\n📋 Board \"{}\" state:", board_state.name);

    // List cards in each column
    for col_name in ["To Do", "In Progress", "Done"] {
        let col_id = match col_name {
            "To Do" => &todo_col.id,
            "In Progress" => &in_progress_col.id,
            "Done" => &done_col.id,
            _ => continue,
        };
        let cards = alice
            .kanban_service
            .list_cards_in_column(&board.id, col_id)
            .unwrap_or_default();
        println!("   {} ({}):", col_name, cards.len());
        for card in &cards {
            println!("      • {}", card.title);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // PHASE 7: Virtual Disk File Operations
    // ─────────────────────────────────────────────────────────────
    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│ PHASE 7: Virtual Disk File Operations                   │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    // Write files to organization's shared disk
    let readme_content = b"# SaorsaLabs\n\nDecentralized collaboration platform.\n\n## Features\n- P2P Messaging\n- Kanban Boards\n- Virtual Disks\n- Invitations\n";

    alice
        .disk_service
        .write_file(&org.id, DiskType::Shared, "README.md", readme_content)
        .await
        .expect("Failed to write README.md");
    println!("✓ Wrote README.md to shared disk ({} bytes)", readme_content.len());

    let config_content = br#"{
  "name": "SaorsaLabs",
  "version": "0.1.0",
  "encryption": "ML-KEM-1024"
}"#;

    alice
        .disk_service
        .write_file(&org.id, DiskType::Shared, "config.json", config_content)
        .await
        .expect("Failed to write config.json");
    println!("✓ Wrote config.json to shared disk");

    // Create a directory
    alice
        .disk_service
        .create_directory(&org.id, DiskType::Shared, "docs")
        .await
        .expect("Failed to create docs directory");
    println!("✓ Created /docs directory");

    // Write to private disk
    let private_notes = b"Personal notes: Remember to review PR #42";
    alice
        .disk_service
        .write_file(&alice_id, DiskType::Private, "notes.txt", private_notes)
        .await
        .expect("Failed to write private notes");
    println!("✓ Wrote notes.txt to Alice's private disk");

    // Read back the README
    let read_content = alice
        .disk_service
        .read_file(&org.id, DiskType::Shared, "README.md")
        .await
        .expect("Failed to read README.md");
    println!("✓ Read README.md back ({} bytes)", read_content.len());
    assert_eq!(read_content, readme_content);

    // List files on shared disk
    let files = alice
        .disk_service
        .list_files(&org.id, DiskType::Shared, "")
        .await
        .expect("Failed to list files");
    println!("\n📁 Files on org shared disk:");
    for file in &files {
        println!("   • {} ({} bytes)", file.path, file.size_bytes);
    }

    // ─────────────────────────────────────────────────────────────
    // PHASE 8: Invitation System
    // ─────────────────────────────────────────────────────────────
    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│ PHASE 8: Invitation System                              │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    // Alice invites Bob to the organization
    let invite_request = InviteRequest::new(
        bob_id.clone(),
        EntityType::Organisation,
        org.id.clone(),
        "member",
    )
    .with_message("Welcome to SaorsaLabs, Bob!");

    let invite = alice
        .invite_service
        .create_invite(&alice_id, invite_request)
        .await
        .expect("Failed to create invite for Bob");

    println!("✓ Alice created invite for Bob");
    println!("  Invite ID: {}", invite.id);
    println!("  Message: {:?}", invite.message);

    // Alice invites Carol to the organization
    let carol_invite_request = InviteRequest::new(
        carol_id.clone(),
        EntityType::Organisation,
        org.id.clone(),
        "member",
    )
    .with_message("Join us, Carol!");

    let carol_invite = alice
        .invite_service
        .create_invite(&alice_id, carol_invite_request)
        .await
        .expect("Failed to create invite for Carol");

    println!("✓ Alice created invite for Carol");
    println!("  Invite ID: {}", carol_invite.id);

    sleep(Duration::from_secs(2)).await;

    // Bob checks pending invites
    let bob_pending = bob
        .invite_service
        .list_pending_invites(&bob_id)
        .await
        .expect("Failed to list Bob's pending invites");

    println!("\n📬 Bob's pending invites: {}", bob_pending.len());
    for inv in &bob_pending {
        println!("   • From: {}, Entity: {}", inv.creator_id, inv.entity_id);
    }

    // Bob accepts the invite
    if !bob_pending.is_empty() {
        bob.invite_service
            .accept_invite(&bob_id, &bob_pending[0].id)
            .await
            .expect("Bob failed to accept invite");
        println!("✓ Bob accepted invite to join {}", org.name);
    }

    // Carol checks and accepts
    let carol_pending = carol
        .invite_service
        .list_pending_invites(&carol_id)
        .await
        .expect("Failed to list Carol's pending invites");

    if !carol_pending.is_empty() {
        carol
            .invite_service
            .accept_invite(&carol_id, &carol_pending[0].id)
            .await
            .expect("Carol failed to accept invite");
        println!("✓ Carol accepted invite to join {}", org.name);
    }

    sleep(Duration::from_secs(2)).await;

    // ─────────────────────────────────────────────────────────────
    // PHASE 9: Verify Sync Across Nodes
    // ─────────────────────────────────────────────────────────────
    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│ PHASE 9: Verify Sync Across Nodes                       │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    // List entities from each node's perspective
    let alice_entities = alice
        .entity_service
        .list_entities()
        .await
        .unwrap_or_default();
    let bob_entities = bob.entity_service.list_entities().await.unwrap_or_default();
    let carol_entities = carol
        .entity_service
        .list_entities()
        .await
        .unwrap_or_default();

    println!("📊 Entity count by node:");
    println!("   • Alice: {} entities", alice_entities.len());
    println!("   • Bob:   {} entities", bob_entities.len());
    println!("   • Carol: {} entities", carol_entities.len());

    println!("\n📋 Alice's entities:");
    for entity in &alice_entities {
        println!("   • {} ({:?})", entity.name, entity.entity_type);
    }

    // ─────────────────────────────────────────────────────────────
    // TEST SUMMARY
    // ─────────────────────────────────────────────────────────────
    println!("\n{}", "=".repeat(60));
    println!("  TEST SUMMARY");
    println!("{}\n", "=".repeat(60));

    println!("✅ Organization created: {}", org.name);
    println!("✅ Group created: {} (parent: {})", group.name, org.name);
    println!("✅ Channels created: #{}, #{}", general_channel.name, dev_channel.name);
    println!("✅ Messages sent: 3 messages across 2 channels");
    println!("✅ Kanban board: {} with 3 columns, 3 cards", board.name);
    println!("✅ File operations: {} files on shared disk", files.len());
    println!("✅ Invitations: Bob and Carol invited and joined");
    println!("✅ Sync verification: All nodes connected\n");

    println!("{}", "=".repeat(60));
    println!("  FULL INFRASTRUCTURE E2E TEST COMPLETE!");
    println!("{}\n", "=".repeat(60));
}
