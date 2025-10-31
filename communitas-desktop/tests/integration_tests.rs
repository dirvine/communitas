//! Integration Tests for Tauri IPC Commands
//!
//! Tests actual command execution with real CoreContext and state.
//! Uses TestFixture for proper setup and cleanup.

mod fixtures;

use fixtures::test_harness::TestFixture;
use fixtures::test_helpers::*;

// ============================================================================
// Identity & Initialization Tests (TIER 1: CRITICAL)
// ============================================================================

#[tokio::test]
async fn test_core_initialize_creates_context() {
    // GIVEN: A test fixture
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    // WHEN: We access the CoreContext
    let ctx = fixture.core_context();
    let ctx_guard = ctx.read().await;

    // THEN: CoreContext should be properly initialized
    assert_eq!(ctx_guard.four_words, "ocean-forest-moon-star");
    assert_eq!(ctx_guard.display_name, "Test User");
    assert_eq!(ctx_guard.device_name, "Test Device");

    // AND: Services should be initialized
    assert!(ctx_guard.entity_service.list_entities().await.is_ok());
    assert!(
        ctx_guard
            .message_service
            .get_channel_messages("test".to_string())
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn test_core_initialize_with_custom_identity() {
    // GIVEN: Custom identity parameters
    let four_words = "river-mountain-cloud-tree";
    let display_name = "Alice";
    let device_name = "Alice-Desktop";

    // WHEN: Initializing with custom identity
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context_custom(
            four_words.to_string(),
            display_name.to_string(),
            device_name.to_string(),
        )
        .await
        .unwrap();

    let ctx = fixture.core_context();
    let ctx_guard = ctx.read().await;

    // THEN: Context should use custom identity
    assert_eq!(ctx_guard.four_words, four_words);
    assert_eq!(ctx_guard.display_name, display_name);
    assert_eq!(ctx_guard.device_name, device_name);
}

#[tokio::test]
async fn test_core_initialize_invalid_four_words() {
    use communitas_core::{CoreContext, types::DeviceType};
    use std::path::PathBuf;

    // GIVEN: Invalid four-word format (only 2 words)
    let invalid_four_words = "ocean-forest"; // Only 2 words

    let temp_fixture = TestFixture::new().unwrap();
    let storage_dir = temp_fixture.temp_path().join("test_storage");

    // WHEN: Attempting to initialize with invalid format
    let result = CoreContext::initialize(
        invalid_four_words.to_string(),
        "Test".to_string(),
        "Device".to_string(),
        DeviceType::Desktop,
        storage_dir,
    )
    .await;

    // THEN: Should fail with validation error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("4 words") || error.contains("expected 4"));
}

// ============================================================================
// Channel & Messaging Tests (TIER 2: HIGH)
// ============================================================================

#[tokio::test]
async fn test_create_channel_succeeds() {
    // GIVEN: An initialized CoreContext
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    let ctx = fixture.core_context();

    // WHEN: Creating a test channel
    let channel_name = test_channel_name();
    let channel_id = create_test_channel(&ctx, &channel_name)
        .await
        .expect("Failed to create channel");

    // THEN: Channel should exist
    assert!(!channel_id.is_empty());
    assert_channel_exists(&ctx, &channel_id)
        .await
        .expect("Channel should exist");
}

#[tokio::test]
async fn test_send_message_to_channel() {
    // GIVEN: A channel exists
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    let ctx = fixture.core_context();
    let channel_name = test_channel_name();
    let channel_id = create_test_channel(&ctx, &channel_name)
        .await
        .expect("Failed to create channel");

    // WHEN: Sending a message to the channel
    let message_content = "Hello, Communitas!";
    let message_id = send_test_message(&ctx, &channel_id, message_content)
        .await
        .expect("Failed to send message");

    // THEN: Message should exist in channel
    assert!(!message_id.is_empty());
    assert_message_in_channel(&ctx, &channel_id, message_content)
        .await
        .expect("Message should exist in channel");
}

#[tokio::test]
async fn test_send_empty_message_fails() {
    // GIVEN: A channel exists
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    let ctx = fixture.core_context();
    let channel_name = test_channel_name();
    let channel_id = create_test_channel(&ctx, &channel_name)
        .await
        .expect("Failed to create channel");

    // WHEN: Attempting to send empty message
    let result = send_test_message(&ctx, &channel_id, "").await;

    // THEN: Should fail (empty messages not allowed)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_channels() {
    // GIVEN: Multiple channels exist
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    let ctx = fixture.core_context();

    let channel1 = create_test_channel(&ctx, &test_channel_name())
        .await
        .expect("Failed to create channel 1");
    let channel2 = create_test_channel(&ctx, &test_channel_name())
        .await
        .expect("Failed to create channel 2");

    // WHEN: Listing all channels
    let channels = get_channels(&ctx).await.expect("Failed to list channels");

    // THEN: Both channels should be in the list
    assert!(channels.len() >= 2);
    let channel_ids: Vec<_> = channels.iter().map(|ch| ch.id.as_str()).collect();

    assert!(channel_ids.contains(&channel1.as_str()));
    assert!(channel_ids.contains(&channel2.as_str()));
}

// ============================================================================
// Group Management Tests (TIER 4: MEDIUM)
// ============================================================================

#[tokio::test]
async fn test_create_group_succeeds() {
    // GIVEN: An initialized CoreContext
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    let ctx = fixture.core_context();

    // WHEN: Creating a test group
    let group_name = test_group_name();
    let group_id = create_test_group(&ctx, &group_name)
        .await
        .expect("Failed to create group");

    // THEN: Group should exist
    assert!(!group_id.is_empty());
}

#[tokio::test]
async fn test_add_member_to_group() {
    // GIVEN: A group exists
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    let ctx = fixture.core_context();
    let group_name = test_group_name();
    let group_id = create_test_group(&ctx, &group_name)
        .await
        .expect("Failed to create group");

    // WHEN: Adding a member to the group
    let member_id = "river-mountain-cloud-tree";
    add_group_member(&ctx, &group_id, member_id, "member")
        .await
        .expect("Failed to add member");

    // THEN: Member should be in the group
    let members = get_group_members(&ctx, &group_id)
        .await
        .expect("Failed to get members");

    assert!(members.contains(&member_id.to_string()));
}

#[tokio::test]
async fn test_remove_member_from_group() {
    // GIVEN: A group with a member
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    let ctx = fixture.core_context();
    let group_name = test_group_name();
    let group_id = create_test_group(&ctx, &group_name)
        .await
        .expect("Failed to create group");

    let member_id = "river-mountain-cloud-tree";
    add_group_member(&ctx, &group_id, member_id, "member")
        .await
        .expect("Failed to add member");

    // WHEN: Removing the member
    remove_group_member(&ctx, &group_id, member_id)
        .await
        .expect("Failed to remove member");

    // THEN: Member should not be in the group
    let members = get_group_members(&ctx, &group_id)
        .await
        .expect("Failed to get members");

    assert!(!members.contains(&member_id.to_string()));
}

// ============================================================================
// Multi-step Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_complete_messaging_workflow() {
    // GIVEN: An initialized system
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    let ctx = fixture.core_context();

    // WHEN: Executing complete workflow
    // Step 1: Create channel
    let channel_name = test_channel_name();
    let channel_id = create_test_channel(&ctx, &channel_name)
        .await
        .expect("Failed to create channel");

    // Step 2: Send first message
    let msg1 = send_test_message(&ctx, &channel_id, "First message")
        .await
        .expect("Failed to send first message");

    // Step 3: Send second message
    let msg2 = send_test_message(&ctx, &channel_id, "Second message")
        .await
        .expect("Failed to send second message");

    // Step 4: Retrieve all messages
    let messages = get_channel_messages(&ctx, &channel_id)
        .await
        .expect("Failed to get messages");

    // THEN: Complete workflow should succeed
    assert_eq!(messages.len(), 2);

    let msg_ids: Vec<_> = messages.iter().map(|m| m.metadata.id.as_str()).collect();

    assert!(msg_ids.contains(&msg1.as_str()));
    assert!(msg_ids.contains(&msg2.as_str()));
}

#[tokio::test]
async fn test_concurrent_operations() {
    // GIVEN: An initialized system
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    let ctx = fixture.core_context();
    let ctx_clone = ctx.clone();

    // WHEN: Performing concurrent operations
    let name1 = test_channel_name();
    let name2 = test_channel_name();
    let (channel1_result, channel2_result) = tokio::join!(
        create_test_channel(&ctx, &name1),
        create_test_channel(&ctx_clone, &name2)
    );

    // THEN: Both operations should succeed
    assert!(channel1_result.is_ok());
    assert!(channel2_result.is_ok());
    assert_ne!(channel1_result.unwrap(), channel2_result.unwrap());
}
