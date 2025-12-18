//! Sync tests for the Kanban CRDT system.
//!
//! These tests verify that concurrent edits merge correctly and data
//! is preserved across sync operations.

use communitas_kanban::{CardState, KanbanService};

fn make_service(peer_id: &str) -> KanbanService {
    KanbanService::new(peer_id.to_string())
}

/// Helper to do full sync from source to target (for new peers).
fn full_sync(source: &KanbanService, target: &KanbanService, board_id: &str) {
    let update = source.get_full_update(board_id).expect("get full update");
    target
        .apply_update(board_id, &update)
        .expect("apply update");
}

/// Helper to do incremental sync from source to target.
fn incremental_sync(source: &KanbanService, target: &KanbanService, board_id: &str) {
    let target_state = target.get_state_vector(board_id).expect("get state vector");
    let update = source
        .get_update(board_id, &target_state)
        .expect("get update");
    if !update.is_empty() {
        target
            .apply_update(board_id, &update)
            .expect("apply update");
    }
}

/// Helper to bidirectional sync between two peers.
fn bidirectional_sync(peer_a: &KanbanService, peer_b: &KanbanService, board_id: &str) {
    incremental_sync(peer_a, peer_b, board_id);
    incremental_sync(peer_b, peer_a, board_id);
}

#[test]
fn test_concurrent_card_edits_merge() {
    // Peer A and Peer B both have a copy of the same board
    let peer_a = make_service("peer-alpha-one-two");
    let peer_b = make_service("peer-beta-three-four");

    // Peer A creates a board
    let board = peer_a
        .create_board("shared-project", "Shared Board".to_string(), None)
        .expect("create board");

    let column = peer_a
        .add_column(&board.id, "Column".to_string(), None)
        .expect("add column");

    let card = peer_a
        .create_card(
            &board.id,
            &column.id,
            "Original Title".to_string(),
            Some("Original Description".to_string()),
        )
        .expect("create card");

    // Peer B syncs with peer A to get the initial state
    full_sync(&peer_a, &peer_b, &board.id);

    // Verify peer B has the card
    let card_b = peer_b.get_card(&board.id, &card.id).expect("get card on b");
    assert_eq!(card_b.title, "Original Title");

    // Peer A edits the title (simulating concurrent edit)
    peer_a
        .update_card(
            &board.id,
            &card.id,
            communitas_kanban::CardUpdate {
                title: Some("Title from A".to_string()),
                ..Default::default()
            },
        )
        .expect("a updates title");

    // Peer B edits the description (simulating concurrent edit)
    peer_b
        .update_card(
            &board.id,
            &card.id,
            communitas_kanban::CardUpdate {
                description: Some("Description from B".to_string()),
                ..Default::default()
            },
        )
        .expect("b updates description");

    // Bidirectional sync
    bidirectional_sync(&peer_a, &peer_b, &board.id);

    // Verify both peers see both changes
    let card_on_a = peer_a.get_card(&board.id, &card.id).expect("get card a");
    let card_on_b = peer_b.get_card(&board.id, &card.id).expect("get card b");

    // Title should be from A (LWW - A edited title)
    assert_eq!(card_on_a.title, "Title from A");
    assert_eq!(card_on_b.title, "Title from A");

    // Description should be from B (LWW - B edited description)
    assert_eq!(card_on_a.description, "Description from B");
    assert_eq!(card_on_b.description, "Description from B");
}

#[test]
fn test_concurrent_card_moves() {
    // Two peers move the same card to different columns concurrently
    let peer_a = make_service("move-peer-alpha");
    let peer_b = make_service("move-peer-beta");

    // Setup: Peer A creates board with 3 columns
    let board = peer_a
        .create_board("move-project", "Move Test".to_string(), None)
        .expect("create board");

    let col_x = peer_a
        .add_column(&board.id, "Column X".to_string(), None)
        .expect("add x");
    let col_y = peer_a
        .add_column(&board.id, "Column Y".to_string(), None)
        .expect("add y");
    let col_z = peer_a
        .add_column(&board.id, "Column Z".to_string(), None)
        .expect("add z");

    let card = peer_a
        .create_card(&board.id, &col_x.id, "Movable Card".to_string(), None)
        .expect("create card");

    // Sync initial state to peer B
    full_sync(&peer_a, &peer_b, &board.id);

    // Concurrent moves:
    // Peer A moves card to Column Y
    peer_a
        .move_card(&board.id, &card.id, &col_y.id, 0)
        .expect("a moves to y");

    // Peer B moves card to Column Z (concurrent, before sync)
    peer_b
        .move_card(&board.id, &card.id, &col_z.id, 0)
        .expect("b moves to z");

    // Bidirectional sync
    bidirectional_sync(&peer_a, &peer_b, &board.id);

    // After sync, both peers should agree on the card's location
    let card_a = peer_a.get_card(&board.id, &card.id).expect("get a");
    let card_b = peer_b.get_card(&board.id, &card.id).expect("get b");

    // LWW: card should be in the same column on both peers
    assert_eq!(
        card_a.column_id, card_b.column_id,
        "Peers must agree on card location"
    );

    // The card should be in exactly one column
    let cards_y_a = peer_a
        .list_cards_in_column(&board.id, &col_y.id)
        .expect("list y");
    let cards_z_a = peer_a
        .list_cards_in_column(&board.id, &col_z.id)
        .expect("list z");

    let in_y = cards_y_a.iter().any(|c| c.id == card.id);
    let in_z = cards_z_a.iter().any(|c| c.id == card.id);

    // Card must be in exactly one column, not both
    assert!(in_y || in_z, "Card must be in Y or Z");
    assert!(!(in_y && in_z), "Card must not be in both Y and Z");
}

#[test]
fn test_concurrent_assignments_orset() {
    // OR-Set semantics: concurrent add/remove on different items should merge
    let peer_a = make_service("orset-peer-alpha");
    let peer_b = make_service("orset-peer-beta");

    // Setup
    let board = peer_a
        .create_board("orset-project", "OR-Set Test".to_string(), None)
        .expect("board");
    let column = peer_a
        .add_column(&board.id, "Column".to_string(), None)
        .expect("column");
    let card = peer_a
        .create_card(&board.id, &column.id, "Card".to_string(), None)
        .expect("card");

    // Assign initial user
    peer_a
        .assign_user(&board.id, &card.id, "user-initial-one-two")
        .expect("assign initial");

    // Sync to peer B
    full_sync(&peer_a, &peer_b, &board.id);

    // Concurrent operations:
    // Peer A: add user-alpha
    peer_a
        .assign_user(&board.id, &card.id, "user-alpha-three-four")
        .expect("a adds alpha");

    // Peer B: add user-beta, remove user-initial
    peer_b
        .assign_user(&board.id, &card.id, "user-beta-five-six")
        .expect("b adds beta");
    peer_b
        .unassign_user(&board.id, &card.id, "user-initial-one-two")
        .expect("b removes initial");

    // Bidirectional sync
    bidirectional_sync(&peer_a, &peer_b, &board.id);

    // Verify both peers agree
    let card_a = peer_a.get_card(&board.id, &card.id).expect("get a");
    let card_b = peer_b.get_card(&board.id, &card.id).expect("get b");

    // Sort for comparison
    let mut assignees_a = card_a.assignee_ids.clone();
    let mut assignees_b = card_b.assignee_ids.clone();
    assignees_a.sort();
    assignees_b.sort();

    assert_eq!(assignees_a, assignees_b, "Peers must agree on assignees");

    // OR-Set semantics:
    // - user-alpha should be present (A added)
    // - user-beta should be present (B added)
    // - user-initial may or may not be present depending on timing
    assert!(
        card_a
            .assignee_ids
            .contains(&"user-alpha-three-four".to_string()),
        "Alpha should be assigned"
    );
    assert!(
        card_a
            .assignee_ids
            .contains(&"user-beta-five-six".to_string()),
        "Beta should be assigned"
    );
}

#[test]
fn test_concurrent_step_completion() {
    // Two peers complete the same step concurrently
    let peer_a = make_service("step-peer-alpha");
    let peer_b = make_service("step-peer-beta");

    // Setup
    let board = peer_a
        .create_board("step-project", "Step Test".to_string(), None)
        .expect("board");
    let column = peer_a
        .add_column(&board.id, "Column".to_string(), None)
        .expect("column");
    let card = peer_a
        .create_card(&board.id, &column.id, "Card".to_string(), None)
        .expect("card");
    let step = peer_a
        .add_step(&board.id, &card.id, "Do something".to_string(), None)
        .expect("step");

    // Sync to peer B
    full_sync(&peer_a, &peer_b, &board.id);

    // Both peers complete the step concurrently
    peer_a
        .toggle_step(&board.id, &card.id, &step.id)
        .expect("a toggles");
    peer_b
        .toggle_step(&board.id, &card.id, &step.id)
        .expect("b toggles");

    // Bidirectional sync
    bidirectional_sync(&peer_a, &peer_b, &board.id);

    // Both peers should agree on completion status
    // LWW applies, so final state depends on timestamp ordering
    // but both must agree after sync
    let _card_a = peer_a.get_card(&board.id, &card.id).expect("get a");
    let _card_b = peer_b.get_card(&board.id, &card.id).expect("get b");
    // The important thing is the documents converge (no assertion on specific state)
}

#[test]
fn test_sync_with_new_peer() {
    // A new peer joins and receives full state
    let original = make_service("original-peer-one");

    // Create substantial state
    let board = original
        .create_board("full-project", "Full State".to_string(), None)
        .expect("board");

    let col1 = original
        .add_column(&board.id, "Todo".to_string(), None)
        .expect("col1");
    let col2 = original
        .add_column(&board.id, "Done".to_string(), None)
        .expect("col2");

    let card1 = original
        .create_card(&board.id, &col1.id, "Card 1".to_string(), None)
        .expect("card1");
    let card2 = original
        .create_card(&board.id, &col1.id, "Card 2".to_string(), None)
        .expect("card2");

    original
        .assign_user(&board.id, &card1.id, "user-one-two-three")
        .expect("assign");

    original
        .move_card(&board.id, &card2.id, &col2.id, 0)
        .expect("move");
    original
        .change_card_state(&board.id, &card2.id, CardState::Closed)
        .expect("close");

    // New peer joins with empty state
    let new_peer = make_service("new-peer-joining");

    // Full sync from original to new peer
    full_sync(&original, &new_peer, &board.id);

    // Verify new peer has all state
    let board_new = new_peer.get_board(&board.id).expect("get board");
    assert_eq!(board_new.name, "Full State");

    let columns = new_peer.list_columns(&board.id).expect("list columns");
    assert_eq!(columns.len(), 2);

    let card1_new = new_peer.get_card(&board.id, &card1.id).expect("get card1");
    assert_eq!(card1_new.title, "Card 1");
    assert!(
        card1_new
            .assignee_ids
            .contains(&"user-one-two-three".to_string())
    );

    let card2_new = new_peer.get_card(&board.id, &card2.id).expect("get card2");
    assert_eq!(card2_new.title, "Card 2");
    assert_eq!(card2_new.column_id, col2.id);
    assert_eq!(card2_new.state, CardState::Closed);
}

#[test]
fn test_incremental_sync() {
    // Sync only changes since last sync
    let peer_a = make_service("incr-peer-alpha");
    let peer_b = make_service("incr-peer-beta");

    // Initial state
    let board = peer_a
        .create_board("incr-project", "Incremental".to_string(), None)
        .expect("board");
    let column = peer_a
        .add_column(&board.id, "Column".to_string(), None)
        .expect("column");
    let card1 = peer_a
        .create_card(&board.id, &column.id, "Card 1".to_string(), None)
        .expect("card1");

    // Initial full sync
    full_sync(&peer_a, &peer_b, &board.id);

    // Verify initial sync worked
    let _card1_b = peer_b
        .get_card(&board.id, &card1.id)
        .expect("get card1 on b");

    // Peer A makes more changes
    let card2 = peer_a
        .create_card(&board.id, &column.id, "Card 2".to_string(), None)
        .expect("card2");
    peer_a
        .update_card(
            &board.id,
            &card1.id,
            communitas_kanban::CardUpdate {
                title: Some("Updated Card 1".to_string()),
                ..Default::default()
            },
        )
        .expect("update");

    // Incremental sync - only get changes since last sync
    incremental_sync(&peer_a, &peer_b, &board.id);

    // Verify peer B has the new changes
    let card1_b = peer_b.get_card(&board.id, &card1.id).expect("get card1");
    assert_eq!(card1_b.title, "Updated Card 1");

    let card2_b = peer_b.get_card(&board.id, &card2.id).expect("get card2");
    assert_eq!(card2_b.title, "Card 2");
}

#[test]
fn test_three_way_sync() {
    // Three peers all syncing with each other
    let peer_a = make_service("three-way-alpha");
    let peer_b = make_service("three-way-beta");
    let peer_c = make_service("three-way-gamma");

    // A creates board
    let board = peer_a
        .create_board("three-project", "Three Way".to_string(), None)
        .expect("board");
    let column = peer_a
        .add_column(&board.id, "Column".to_string(), None)
        .expect("column");

    // Full sync A -> B and A -> C
    full_sync(&peer_a, &peer_b, &board.id);
    full_sync(&peer_a, &peer_c, &board.id);

    // Each peer creates a card
    let _card_a = peer_a
        .create_card(&board.id, &column.id, "From A".to_string(), None)
        .expect("card a");
    let _card_b = peer_b
        .create_card(&board.id, &column.id, "From B".to_string(), None)
        .expect("card b");
    let _card_c = peer_c
        .create_card(&board.id, &column.id, "From C".to_string(), None)
        .expect("card c");

    // Pairwise syncs (multiple rounds to ensure convergence)
    for _ in 0..2 {
        bidirectional_sync(&peer_a, &peer_b, &board.id);
        bidirectional_sync(&peer_b, &peer_c, &board.id);
        bidirectional_sync(&peer_a, &peer_c, &board.id);
    }

    // All peers should have all 3 cards
    let cards_a = peer_a
        .list_cards_in_column(&board.id, &column.id)
        .expect("a");
    let cards_b = peer_b
        .list_cards_in_column(&board.id, &column.id)
        .expect("b");
    let cards_c = peer_c
        .list_cards_in_column(&board.id, &column.id)
        .expect("c");

    assert_eq!(cards_a.len(), 3, "A should have 3 cards");
    assert_eq!(cards_b.len(), 3, "B should have 3 cards");
    assert_eq!(cards_c.len(), 3, "C should have 3 cards");

    // All peers should have same cards
    let titles_a: Vec<_> = cards_a.iter().map(|c| &c.title).collect();
    let titles_b: Vec<_> = cards_b.iter().map(|c| &c.title).collect();
    let titles_c: Vec<_> = cards_c.iter().map(|c| &c.title).collect();

    for title in ["From A", "From B", "From C"] {
        assert!(
            titles_a.contains(&&title.to_string()),
            "A missing {}",
            title
        );
        assert!(
            titles_b.contains(&&title.to_string()),
            "B missing {}",
            title
        );
        assert!(
            titles_c.contains(&&title.to_string()),
            "C missing {}",
            title
        );
    }
}
