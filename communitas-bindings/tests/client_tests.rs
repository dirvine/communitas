use communitas_bindings::{CommunitasClient, SwiftEntityType};
use tempfile::tempdir;

#[test]
fn test_client_initialization() {
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().to_string_lossy().to_string();

    let client = CommunitasClient::new(
        "ocean-forest-moon-star".to_string(),
        "Test User".to_string(),
        "Test Device".to_string(),
        storage_path,
    );

    assert!(client.is_ok());
    let client = client.unwrap();

    let profile = client.get_profile();
    assert_eq!(profile.four_words, "ocean-forest-moon-star");
    assert_eq!(profile.display_name, "Test User");
    assert_eq!(profile.device_name, "Test Device");
    assert_eq!(profile.device_type, "Mobile");
}

#[test]
fn test_entity_creation_and_listing() {
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().to_string_lossy().to_string();

    let client = CommunitasClient::new(
        "ocean-forest-moon-star".to_string(),
        "Test User".to_string(),
        "Test Device".to_string(),
        storage_path,
    )
    .unwrap();

    // List entities should be empty initially
    let entities = client.entity_list().unwrap();
    assert!(entities.is_empty());

    // Create a group (name, type, description, parent_id)
    let group = client
        .entity_create(
            "My Test Group".to_string(),
            SwiftEntityType::Group,
            Some("Description".to_string()),
            None, // parent_id
        )
        .unwrap();

    assert_eq!(group.name, "My Test Group");
    assert!(matches!(group.entity_type, SwiftEntityType::Group));

    // List entities should now contain the group
    let entities = client.entity_list().unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].id, group.id);
}

#[test]
fn test_messaging_flow() {
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().to_string_lossy().to_string();

    let client = CommunitasClient::new(
        "ocean-forest-moon-star".to_string(),
        "Test User".to_string(),
        "Test Device".to_string(),
        storage_path,
    )
    .unwrap();

    // Create a channel to send messages to (name, type, description, parent_id)
    let channel = client
        .entity_create(
            "General".to_string(),
            SwiftEntityType::Channel,
            None,
            None,
        )
        .unwrap();

    // Send a message (entity_id, text, parent_id for thread)
    let msg_id_result = client.message_send(
        channel.id.clone(),
        "Hello World".to_string(),
        None, // parent_id for threading
    );

    assert!(msg_id_result.is_ok());
    let msg_id = msg_id_result.unwrap();

    // Retrieve messages (entity_id, limit, before_id)
    let messages = client
        .message_get_for_entity(channel.id.clone(), None, None)
        .unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, msg_id);
    assert_eq!(messages[0].text, "Hello World");
    assert_eq!(messages[0].author, "ocean-forest-moon-star");
}

#[test]
fn test_networking_status() {
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().to_string_lossy().to_string();

    let client = CommunitasClient::new(
        "ocean-forest-moon-star".to_string(),
        "Test User".to_string(),
        "Test Device".to_string(),
        storage_path,
    )
    .unwrap();

    // Should be false initially
    assert!(!client.is_networking_active());

    // Note: Starting networking requires CryptoProvider to be installed,
    // which is a process-level concern handled by the real app.
    // We verify the initial state only in unit tests.
}
