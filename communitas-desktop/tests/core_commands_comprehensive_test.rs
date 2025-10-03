//! Comprehensive tests for core commands (channels, messages, bootstrap)
//!
//! Tests all core Tauri commands with various scenarios and edge cases.

use communitas_core::CoreContext;
use communitas_desktop::core_commands::*;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Helper to create test state with initialized core
async fn create_test_core() -> Arc<RwLock<Option<CoreContext>>> {
    Arc::new(RwLock::new(None))
}

#[tokio::test]
async fn test_core_initialize_success() {
    let shared = create_test_core().await;

    let result = core_initialize(
        State::from(&shared),
        "ocean-forest-moon-star".to_string(),
        "Test User".to_string(),
        Some("Test Device".to_string()),
        Some("Desktop".to_string()),
    )
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    // Verify core is initialized
    let guard = shared.read().await;
    assert!(guard.is_some());
}

#[tokio::test]
async fn test_core_initialize_invalid_device_type() {
    let shared = create_test_core().await;

    let result = core_initialize(
        State::from(&shared),
        "ocean-forest-moon-star".to_string(),
        "Test User".to_string(),
        Some("Device".to_string()),
        Some("InvalidType".to_string()),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_core_initialize_default_device_type() {
    let shared = create_test_core().await;

    let result = core_initialize(
        State::from(&shared),
        "river-mountain-sun-cloud".to_string(),
        "User".to_string(),
        Some("Device".to_string()),
        None, // No device type specified
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_channel_creation_valid() {
    let shared = create_test_core().await;

    // Initialize core
    core_initialize(
        State::from(&shared),
        "test-channel-user".to_string(),
        "Channel User".to_string(),
        Some("Device".to_string()),
        Some("Desktop".to_string()),
    )
    .await
    .unwrap();

    // Create channel
    let result = core_create_channel(
        State::from(&shared),
        "General Discussion".to_string(),
        "Main channel for general topics".to_string(),
    )
    .await;

    assert!(result.is_ok());
    let channel = result.unwrap();
    assert_eq!(channel.name, "General Discussion");
    assert_eq!(channel.description, "Main channel for general topics");
}

#[tokio::test]
async fn test_channel_creation_empty_name() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_create_channel(
        State::from(&shared),
        "".to_string(), // Empty name
        "Description".to_string(),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_channel_creation_name_too_long() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let long_name = "a".repeat(101); // Exceeds 100 character limit

    let result = core_create_channel(
        State::from(&shared),
        long_name,
        "Description".to_string(),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too long"));
}

#[tokio::test]
async fn test_channel_creation_description_too_long() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let long_desc = "a".repeat(501); // Exceeds 500 character limit

    let result = core_create_channel(
        State::from(&shared),
        "Valid Name".to_string(),
        long_desc,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too long"));
}

#[tokio::test]
async fn test_channel_creation_invalid_characters() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_create_channel(
        State::from(&shared),
        "Invalid@Name#".to_string(), // Invalid characters
        "Description".to_string(),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("invalid characters"));
}

#[tokio::test]
async fn test_get_channels_empty() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_get_channels(State::from(&shared)).await;

    assert!(result.is_ok());
    let channels = result.unwrap();
    assert_eq!(channels.len(), 0);
}

#[tokio::test]
async fn test_get_channels_multiple() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    // Create multiple channels
    for i in 1..=3 {
        core_create_channel(
            State::from(&shared),
            format!("Channel {}", i),
            format!("Description {}", i),
        )
        .await
        .unwrap();
    }

    let result = core_get_channels(State::from(&shared)).await;

    assert!(result.is_ok());
    let channels = result.unwrap();
    assert_eq!(channels.len(), 3);
}

#[tokio::test]
async fn test_send_message_to_recipients_valid() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "sender-user".to_string(),
        "Sender".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let channel = core_create_channel(
        State::from(&shared),
        "Test Channel".to_string(),
        "Description".to_string(),
    )
    .await
    .unwrap();

    let result = core_send_message_to_recipients(
        State::from(&shared),
        channel.id.0.clone(),
        vec!["ocean-forest-moon-star".to_string()],
        "Hello, world!".to_string(),
    )
    .await;

    assert!(result.is_ok());
    let message_id = result.unwrap();
    assert!(!message_id.is_empty());
}

#[tokio::test]
async fn test_send_message_empty_text() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_send_message_to_recipients(
        State::from(&shared),
        "channel-id".to_string(),
        vec!["recipient".to_string()],
        "".to_string(), // Empty message
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_send_message_too_large() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let large_message = "a".repeat(11 * 1024); // Exceeds 10KB limit

    let result = core_send_message_to_recipients(
        State::from(&shared),
        "channel-id".to_string(),
        vec!["recipient".to_string()],
        large_message,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too long"));
}

#[tokio::test]
async fn test_send_message_no_recipients() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_send_message_to_recipients(
        State::from(&shared),
        "channel-id".to_string(),
        vec![], // No recipients
        "Message".to_string(),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at least one recipient"));
}

#[tokio::test]
async fn test_send_message_too_many_recipients() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let recipients: Vec<String> = (0..101)
        .map(|i| format!("user-{}-test-addr", i))
        .collect();

    let result = core_send_message_to_recipients(
        State::from(&shared),
        "channel-id".to_string(),
        recipients,
        "Message".to_string(),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Too many recipients"));
}

#[tokio::test]
async fn test_send_message_invalid_recipient_format() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_send_message_to_recipients(
        State::from(&shared),
        "channel-id".to_string(),
        vec!["invalid-format".to_string()], // Not four words
        "Message".to_string(),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid recipient format"));
}

#[tokio::test]
async fn test_add_reaction_without_init() {
    let shared = create_test_core().await;

    let result = core_add_reaction(
        State::from(&shared),
        "channel-id".to_string(),
        "message-id".to_string(),
        "👍".to_string(),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not initialized"));
}

#[tokio::test]
async fn test_create_thread_without_init() {
    let shared = create_test_core().await;

    let result = core_create_thread(
        State::from(&shared),
        "channel-id".to_string(),
        "message-id".to_string(),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_bootstrap_nodes_operations() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "bootstrap-test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    // Get initial bootstrap nodes
    let nodes = core_get_bootstrap_nodes(State::from(&shared))
        .await
        .unwrap();

    assert!(!nodes.is_empty());

    // Add a bootstrap node
    let result = core_add_bootstrap_node(
        State::from(&shared),
        "custom-node-addr".to_string(),
    )
    .await;

    assert!(result.is_ok());

    // Get stats
    let stats = core_get_bootstrap_stats(State::from(&shared))
        .await
        .unwrap();

    assert!(stats.get("total_nodes").is_some());
}

#[tokio::test]
async fn test_add_bootstrap_node_empty() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_add_bootstrap_node(
        State::from(&shared),
        "".to_string(), // Empty node
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_add_bootstrap_node_too_long() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let long_node = "a".repeat(256); // Exceeds 255 limit

    let result = core_add_bootstrap_node(
        State::from(&shared),
        long_node,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too long"));
}

#[tokio::test]
async fn test_clear_custom_nodes() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    // Add custom node
    core_add_bootstrap_node(
        State::from(&shared),
        "custom-node".to_string(),
    )
    .await
    .unwrap();

    // Clear custom nodes
    let result = core_clear_custom_nodes(State::from(&shared)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_private_storage_put_get() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "storage-test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let key = "test-key";
    let data = b"test data content".to_vec();

    // Store data
    let put_result = core_private_put(
        State::from(&shared),
        key.to_string(),
        data.clone(),
    )
    .await;

    assert!(put_result.is_ok());

    // Retrieve data
    let get_result = core_private_get(
        State::from(&shared),
        key.to_string(),
    )
    .await;

    assert!(get_result.is_ok());
    assert_eq!(get_result.unwrap(), data);
}

#[tokio::test]
async fn test_private_storage_get_nonexistent() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_private_get(
        State::from(&shared),
        "nonexistent-key".to_string(),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_channel_members_list() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "member-test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let channel = core_create_channel(
        State::from(&shared),
        "Members Test".to_string(),
        "Test channel for members".to_string(),
    )
    .await
    .unwrap();

    let result = core_channel_list_members(
        State::from(&shared),
        channel.id.0.clone(),
    )
    .await;

    assert!(result.is_ok());
    let members = result.unwrap();
    // Creator should be a member
    assert!(!members.is_empty());
}

#[tokio::test]
async fn test_sync_status() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "sync-test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = get_sync_status(State::from(&shared)).await;

    assert!(result.is_ok());
    let status = result.unwrap();
    assert!(status.get("connected").is_some());
}

#[tokio::test]
async fn test_subscribe_entity_offline_mode() {
    let shared = create_test_core().await;

    // Don't initialize core (simulates offline mode)

    let result = subscribe_to_entity(
        State::from(&shared),
        "entity-id".to_string(),
        "user-id".to_string(),
    )
    .await;

    // Should succeed gracefully in offline mode
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_unsubscribe_entity_offline_mode() {
    let shared = create_test_core().await;

    // Don't initialize core (simulates offline mode)

    let result = unsubscribe_from_entity(
        State::from(&shared),
        "entity-id".to_string(),
        "user-id".to_string(),
    )
    .await;

    // Should succeed gracefully in offline mode
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_messages_list_placeholder() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_messages_list(
        State::from(&shared),
        "entity-id".to_string(),
        10,
        0,
    )
    .await;

    // Currently returns empty array (placeholder implementation)
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_entity_permissions_placeholder() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_entity_get_permissions(
        State::from(&shared),
        "entity-id".to_string(),
    )
    .await;

    // Currently returns default permissions (placeholder)
    assert!(result.is_ok());
    let perms = result.unwrap();
    assert!(perms.get("canRead").is_some());
}

#[tokio::test]
async fn test_entity_encryption_status_placeholder() {
    let shared = create_test_core().await;

    core_initialize(
        State::from(&shared),
        "test-user".to_string(),
        "User".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let result = core_entity_get_encryption_status(
        State::from(&shared),
        "entity-id".to_string(),
    )
    .await;

    // Currently returns default status (placeholder)
    assert!(result.is_ok());
    let status = result.unwrap();
    assert!(status.get("encrypted").is_some());
    assert_eq!(status.get("algorithm").and_then(|v| v.as_str()), Some("ML-DSA-65"));
}
