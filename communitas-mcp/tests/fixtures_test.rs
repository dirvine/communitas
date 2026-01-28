// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Test fixtures validation

mod fixtures;

use fixtures::*;

#[test]
fn test_contact_fixtures() {
    let contact = contact_fixture("Test User");
    assert_eq!(contact["name"], "Test User");
    assert!(contact["email"].as_str().unwrap().contains("test.user"));

    let contacts = contact_fixtures(5);
    assert_eq!(contacts.len(), 5);

    assert_eq!(alice_contact()["name"], "Alice Anderson");
    assert_eq!(bob_contact()["name"], "Bob Baker");
    assert_eq!(charlie_contact()["name"], "Charlie Chen");
}

#[test]
fn test_entity_fixtures() {
    let group = entity_fixture("My Group", EntityType::Group);
    assert_eq!(group["name"], "My Group");
    assert_eq!(group["type"], "group");

    assert_eq!(test_group()["type"], "group");
    assert_eq!(test_channel()["type"], "channel");
    assert_eq!(test_project()["type"], "project");
}

#[test]
fn test_kanban_fixtures() {
    let board = board_fixture("My Board");
    assert_eq!(board["name"], "My Board");

    let column = column_fixture("board-123", "To Do", 1);
    assert_eq!(column["board_id"], "board-123");
    assert_eq!(column["name"], "To Do");

    let card = card_fixture("column-123", "Test Task");
    assert_eq!(card["column_id"], "column-123");
    assert_eq!(card["title"], "Test Task");

    let setup = standard_board_setup();
    let columns = setup["columns"].as_array().unwrap();
    assert_eq!(columns.len(), 5);

    let tags = common_tags("board-123");
    assert_eq!(tags.len(), 5);
    assert!(tags.iter().any(|t| t["name"] == "bug"));
}

#[test]
fn test_messaging_fixtures() {
    let msg = message_fixture("Hello, World!");
    assert_eq!(msg["content"], "Hello, World!");

    let thread = thread_with_participants("Team Discussion", &["user1", "user2", "user3"]);
    let participants = thread["participants"].as_array().unwrap();
    assert_eq!(participants.len(), 3);

    let reactions = common_reactions();
    assert!(!reactions.is_empty());
    assert!(reactions.contains(&"thumbsup"));

    let reaction = reaction_fixture("msg-123", "thumbsup");
    assert_eq!(reaction["message_id"], "msg-123");
    assert_eq!(reaction["emoji"], "thumbsup");
}

#[test]
fn test_drive_fixtures() {
    let file = file_fixture("test.txt", "Hello");
    assert_eq!(file["name"], "test.txt");
    assert_eq!(file["content"], "Hello");

    let dir = directory_fixture("/path/to/dir");
    assert_eq!(dir["path"], "/path/to/dir");

    assert_eq!(DiskType::Private.as_str(), "private");
    assert_eq!(DiskType::Public.as_str(), "public");
    assert_eq!(DiskType::Shared.as_str(), "shared");

    let content = sample_binary_content();
    assert!(!content.is_empty());
    assert_eq!(content[0], 0x89); // PNG header

    let text = sample_text_content();
    assert!(!text.is_empty());
}
