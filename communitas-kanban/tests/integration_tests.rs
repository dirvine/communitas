// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration tests for the Kanban system.
//!
//! These tests verify complete workflows and interactions between components.

use communitas_kanban::{CardFilter, CardState, KanbanService};

fn make_service(peer_id: &str) -> KanbanService {
    KanbanService::new(peer_id.to_string())
}

#[test]
fn test_full_workflow_create_board_columns_cards() {
    let service = make_service("ocean-forest-moon-star");

    // Create a board
    let board = service
        .create_board("project-1", "Sprint Board".to_string(), None)
        .expect("create board");
    assert_eq!(board.name, "Sprint Board");
    assert_eq!(board.project_id, "project-1");

    // Add columns
    let col_todo = service
        .add_column(&board.id, "To Do".to_string(), None)
        .expect("add todo column");
    let col_doing = service
        .add_column(&board.id, "In Progress".to_string(), None)
        .expect("add doing column");
    let col_done = service
        .add_column(&board.id, "Done".to_string(), None)
        .expect("add done column");

    // Verify columns
    let columns = service.list_columns(&board.id).expect("list columns");
    assert_eq!(columns.len(), 3);
    assert_eq!(columns[0].name, "To Do");
    assert_eq!(columns[1].name, "In Progress");
    assert_eq!(columns[2].name, "Done");

    // Create cards in To Do column
    let card1 = service
        .create_card(
            &board.id,
            &col_todo.id,
            "Implement feature A".to_string(),
            Some("Description for feature A".to_string()),
        )
        .expect("create card1");

    let card2 = service
        .create_card(&board.id, &col_todo.id, "Fix bug B".to_string(), None)
        .expect("create card2");

    // Verify cards in To Do
    let cards_todo = service
        .list_cards_in_column(&board.id, &col_todo.id)
        .expect("list cards");
    assert_eq!(cards_todo.len(), 2);

    // Move card1 to In Progress
    service
        .move_card(&board.id, &card1.id, &col_doing.id, 0)
        .expect("move card");

    // Verify move
    let cards_todo = service
        .list_cards_in_column(&board.id, &col_todo.id)
        .expect("list todo");
    assert_eq!(cards_todo.len(), 1);
    assert_eq!(cards_todo[0].id, card2.id);

    let cards_doing = service
        .list_cards_in_column(&board.id, &col_doing.id)
        .expect("list doing");
    assert_eq!(cards_doing.len(), 1);
    assert_eq!(cards_doing[0].id, card1.id);

    // Complete card1 (close it)
    service
        .change_card_state(&board.id, &card1.id, CardState::Closed)
        .expect("close card");

    // Move to done
    service
        .move_card(&board.id, &card1.id, &col_done.id, 0)
        .expect("move to done");

    // Verify final state
    let cards_done = service
        .list_cards_in_column(&board.id, &col_done.id)
        .expect("list done");
    assert_eq!(cards_done.len(), 1);
    assert_eq!(cards_done[0].state, CardState::Closed);
}

#[test]
fn test_card_with_steps_and_comments() {
    let service = make_service("alpha-beta-gamma-delta");

    // Setup board and card
    let board = service
        .create_board("project-2", "Task Board".to_string(), None)
        .expect("create board");
    let column = service
        .add_column(&board.id, "Tasks".to_string(), None)
        .expect("add column");
    let card = service
        .create_card(
            &board.id,
            &column.id,
            "Complex Task".to_string(),
            Some("Task with steps and comments".to_string()),
        )
        .expect("create card");

    // Add steps (checklist items)
    let step1 = service
        .add_step(&board.id, &card.id, "Research".to_string(), None)
        .expect("add step 1");
    let step2 = service
        .add_step(&board.id, &card.id, "Implementation".to_string(), None)
        .expect("add step 2");
    let step3 = service
        .add_step(&board.id, &card.id, "Testing".to_string(), None)
        .expect("add step 3");

    // Toggle step1 complete
    let step1_toggled = service
        .toggle_step(&board.id, &card.id, &step1.id)
        .expect("toggle step1");
    assert!(step1_toggled.completed);
    assert!(step1_toggled.completed_by.is_some());

    // Toggle step1 incomplete again
    let step1_untoggled = service
        .toggle_step(&board.id, &card.id, &step1.id)
        .expect("untoggle step1");
    assert!(!step1_untoggled.completed);
    assert!(step1_untoggled.completed_by.is_none());

    // Add comments
    let comment1 = service
        .add_comment(
            &board.id,
            &card.id,
            "Starting work on this".to_string(),
            None,
        )
        .expect("add comment1");
    assert!(comment1.reply_to_id.is_none());

    // Reply to comment
    let reply = service
        .add_comment(
            &board.id,
            &card.id,
            "Sounds good!".to_string(),
            Some(comment1.id.clone()),
        )
        .expect("add reply");
    assert_eq!(reply.reply_to_id, Some(comment1.id.clone()));

    // List comments
    let comments = service
        .list_comments(&board.id, &card.id)
        .expect("list comments");
    assert_eq!(comments.len(), 2);

    // Complete all steps and close card
    service.toggle_step(&board.id, &card.id, &step1.id).unwrap();
    service.toggle_step(&board.id, &card.id, &step2.id).unwrap();
    service.toggle_step(&board.id, &card.id, &step3.id).unwrap();

    service
        .change_card_state(&board.id, &card.id, CardState::Closed)
        .expect("close card");

    let updated_card = service.get_card(&board.id, &card.id).expect("get card");
    assert_eq!(updated_card.state, CardState::Closed);
    assert!(updated_card.completed_at.is_some());
}

#[test]
fn test_assignments_and_tags() {
    let service = make_service("one-two-three-four");

    // Setup
    let board = service
        .create_board("project-3", "Team Board".to_string(), None)
        .expect("create board");
    let column = service
        .add_column(&board.id, "Backlog".to_string(), None)
        .expect("add column");

    // Create tags
    let tag_bug = service
        .create_tag(&board.id, "Bug".to_string(), "#ff0000".to_string())
        .expect("create bug tag");
    let tag_feature = service
        .create_tag(&board.id, "Feature".to_string(), "#00ff00".to_string())
        .expect("create feature tag");

    // Create card
    let card = service
        .create_card(&board.id, &column.id, "Important task".to_string(), None)
        .expect("create card");

    // Assign users
    service
        .assign_user(&board.id, &card.id, "alice-bob-charlie-dave")
        .expect("assign alice");
    service
        .assign_user(&board.id, &card.id, "eve-frank-grace-henry")
        .expect("assign eve");

    // Tag card
    service
        .tag_card(&board.id, &card.id, &tag_bug.id)
        .expect("tag as bug");
    service
        .tag_card(&board.id, &card.id, &tag_feature.id)
        .expect("tag as feature");

    // Verify
    let updated_card = service.get_card(&board.id, &card.id).expect("get card");
    assert_eq!(updated_card.assignee_ids.len(), 2);
    assert!(
        updated_card
            .assignee_ids
            .contains(&"alice-bob-charlie-dave".to_string())
    );
    assert!(
        updated_card
            .assignee_ids
            .contains(&"eve-frank-grace-henry".to_string())
    );
    assert_eq!(updated_card.tag_ids.len(), 2);

    // Unassign one user
    service
        .unassign_user(&board.id, &card.id, "alice-bob-charlie-dave")
        .expect("unassign alice");

    let updated_card = service.get_card(&board.id, &card.id).expect("get card");
    assert_eq!(updated_card.assignee_ids.len(), 1);
    assert!(
        !updated_card
            .assignee_ids
            .contains(&"alice-bob-charlie-dave".to_string())
    );

    // Remove one tag
    service
        .untag_card(&board.id, &card.id, &tag_bug.id)
        .expect("untag bug");

    let updated_card = service.get_card(&board.id, &card.id).expect("get card");
    assert_eq!(updated_card.tag_ids.len(), 1);
}

#[test]
fn test_filtering_cards() {
    let service = make_service("filter-test-peer-id");

    // Setup board with columns
    let board = service
        .create_board("project-4", "Filter Test".to_string(), None)
        .expect("create board");
    let col_todo = service
        .add_column(&board.id, "To Do".to_string(), None)
        .expect("add todo");
    let col_done = service
        .add_column(&board.id, "Done".to_string(), None)
        .expect("add done");

    // Create tag
    let tag_urgent = service
        .create_tag(&board.id, "Urgent".to_string(), "#ff0000".to_string())
        .expect("create tag");

    // Create cards
    let card1 = service
        .create_card(&board.id, &col_todo.id, "Task A".to_string(), None)
        .expect("card1");
    let _card2 = service
        .create_card(&board.id, &col_todo.id, "Task B".to_string(), None)
        .expect("card2");
    let card3 = service
        .create_card(&board.id, &col_done.id, "Task C".to_string(), None)
        .expect("card3");

    // Assign and tag card1
    service
        .assign_user(&board.id, &card1.id, "user-one-two-three")
        .expect("assign");
    service
        .tag_card(&board.id, &card1.id, &tag_urgent.id)
        .expect("tag");

    // Close card3
    service
        .change_card_state(&board.id, &card3.id, CardState::Closed)
        .expect("close card3");

    // Filter by state: Open only
    let filter = CardFilter::new().with_states(vec![CardState::Open]);
    let open_cards = service.filter_cards(&board.id, filter).expect("filter");
    assert_eq!(open_cards.len(), 2);
    assert!(open_cards.iter().all(|c| c.state == CardState::Open));

    // Filter by column
    let filter = CardFilter::new().with_columns(vec![col_todo.id.clone()]);
    let todo_cards = service.filter_cards(&board.id, filter).expect("filter");
    assert_eq!(todo_cards.len(), 2);

    // Filter by assignee
    let filter = CardFilter::new().with_assignees(vec!["user-one-two-three".to_string()]);
    let assigned_cards = service.filter_cards(&board.id, filter).expect("filter");
    assert_eq!(assigned_cards.len(), 1);
    assert_eq!(assigned_cards[0].id, card1.id);

    // Filter by tag
    let filter = CardFilter::new().with_tags(vec![tag_urgent.id.clone()]);
    let tagged_cards = service.filter_cards(&board.id, filter).expect("filter");
    assert_eq!(tagged_cards.len(), 1);
    assert_eq!(tagged_cards[0].id, card1.id);

    // Text search
    let filter = CardFilter::new().with_search("Task A".to_string());
    let search_cards = service.filter_cards(&board.id, filter).expect("filter");
    assert_eq!(search_cards.len(), 1);
    assert_eq!(search_cards[0].title, "Task A");

    // Combined filter
    let filter = CardFilter::new()
        .with_states(vec![CardState::Open])
        .with_columns(vec![col_todo.id.clone()]);
    let combined = service.filter_cards(&board.id, filter).expect("filter");
    assert_eq!(combined.len(), 2);
}

#[test]
fn test_multiple_boards_per_project() {
    let service = make_service("multi-board-test-peer");

    // Create multiple boards for same project
    let board1 = service
        .create_board("shared-project", "Sprint 1".to_string(), None)
        .expect("board1");
    let board2 = service
        .create_board("shared-project", "Sprint 2".to_string(), None)
        .expect("board2");
    let board3 = service
        .create_board("other-project", "Other Board".to_string(), None)
        .expect("board3");

    // Each board should have unique ID
    assert_ne!(board1.id, board2.id);
    assert_ne!(board1.id, board3.id);
    assert_ne!(board2.id, board3.id);

    // All boards should be accessible
    let retrieved1 = service.get_board(&board1.id).expect("get board1");
    let retrieved2 = service.get_board(&board2.id).expect("get board2");
    let retrieved3 = service.get_board(&board3.id).expect("get board3");

    assert_eq!(retrieved1.name, "Sprint 1");
    assert_eq!(retrieved2.name, "Sprint 2");
    assert_eq!(retrieved3.name, "Other Board");
}

#[test]
fn test_delete_operations() {
    let service = make_service("delete-test-peer-id");

    // Setup
    let board = service
        .create_board("project-5", "Delete Test".to_string(), None)
        .expect("board");
    let column = service
        .add_column(&board.id, "Column".to_string(), None)
        .expect("column");
    let card = service
        .create_card(&board.id, &column.id, "Card".to_string(), None)
        .expect("card");
    let step = service
        .add_step(&board.id, &card.id, "Step".to_string(), None)
        .expect("step");
    let comment = service
        .add_comment(&board.id, &card.id, "Comment".to_string(), None)
        .expect("comment");

    // Delete step
    service
        .delete_step(&board.id, &card.id, &step.id)
        .expect("delete step");

    // Delete comment
    service
        .delete_comment(&board.id, &card.id, &comment.id)
        .expect("delete comment");

    // Delete card
    service
        .delete_card(&board.id, &card.id)
        .expect("delete card");

    // Verify card is not returned in list
    let cards = service
        .list_cards_in_column(&board.id, &column.id)
        .expect("list");
    assert_eq!(cards.len(), 0);

    // Delete column
    service
        .delete_column(&board.id, &column.id)
        .expect("delete column");

    // Verify column is not returned
    let columns = service.list_columns(&board.id).expect("list");
    assert_eq!(columns.len(), 0);

    // Delete board
    service.delete_board(&board.id).expect("delete board");

    // Verify board returns not found
    assert!(service.get_board(&board.id).is_err());
}

#[test]
fn test_card_state_transitions_workflow() {
    let service = make_service("state-workflow-peer");

    let board = service
        .create_board("project-6", "State Test".to_string(), None)
        .expect("board");
    let column = service
        .add_column(&board.id, "Work".to_string(), None)
        .expect("column");
    let card = service
        .create_card(&board.id, &column.id, "Task".to_string(), None)
        .expect("card");

    // Initial state is Open
    assert_eq!(card.state, CardState::Open);

    // Open -> Postponed
    service
        .change_card_state(&board.id, &card.id, CardState::Postponed)
        .expect("postpone");
    let card = service.get_card(&board.id, &card.id).expect("get");
    assert_eq!(card.state, CardState::Postponed);

    // Postponed -> Open
    service
        .change_card_state(&board.id, &card.id, CardState::Open)
        .expect("reopen");
    let card = service.get_card(&board.id, &card.id).expect("get");
    assert_eq!(card.state, CardState::Open);

    // Open -> Closed
    service
        .change_card_state(&board.id, &card.id, CardState::Closed)
        .expect("close");
    let card = service.get_card(&board.id, &card.id).expect("get");
    assert_eq!(card.state, CardState::Closed);
    assert!(card.completed_at.is_some());

    // Closed -> Archived
    service
        .change_card_state(&board.id, &card.id, CardState::Archived)
        .expect("archive");
    let card = service.get_card(&board.id, &card.id).expect("get");
    assert_eq!(card.state, CardState::Archived);

    // Archived -> Open (restore)
    service
        .change_card_state(&board.id, &card.id, CardState::Open)
        .expect("restore");
    let card = service.get_card(&board.id, &card.id).expect("get");
    assert_eq!(card.state, CardState::Open);
    assert!(card.completed_at.is_none()); // Cleared on reopen

    // Test invalid transition: Closed -> Postponed should fail
    service
        .change_card_state(&board.id, &card.id, CardState::Closed)
        .expect("close");
    let result = service.change_card_state(&board.id, &card.id, CardState::Postponed);
    assert!(result.is_err());
}

#[test]
fn test_column_reordering() {
    let service = make_service("reorder-test-peer-id");

    let board = service
        .create_board("project-7", "Reorder Test".to_string(), None)
        .expect("board");

    let col_a = service
        .add_column(&board.id, "A".to_string(), None)
        .expect("col a");
    let _col_b = service
        .add_column(&board.id, "B".to_string(), None)
        .expect("col b");
    let col_c = service
        .add_column(&board.id, "C".to_string(), None)
        .expect("col c");

    // Initial order: A(0), B(1), C(2)
    let columns = service.list_columns(&board.id).expect("list");
    assert_eq!(columns[0].name, "A");
    assert_eq!(columns[1].name, "B");
    assert_eq!(columns[2].name, "C");

    // Move C to position 0
    service
        .move_column(&board.id, &col_c.id, 0)
        .expect("move c");

    // New order: C(0), A(1), B(2)
    let columns = service.list_columns(&board.id).expect("list");
    assert_eq!(columns[0].name, "C");
    assert_eq!(columns[1].name, "A");
    assert_eq!(columns[2].name, "B");

    // Move A to position 2
    service
        .move_column(&board.id, &col_a.id, 2)
        .expect("move a");

    // New order: C(0), B(1), A(2)
    let columns = service.list_columns(&board.id).expect("list");
    assert_eq!(columns[0].name, "C");
    assert_eq!(columns[1].name, "B");
    assert_eq!(columns[2].name, "A");
}

#[test]
fn test_card_move_between_columns() {
    let service = make_service("card-move-test-peer");

    let board = service
        .create_board("project-8", "Move Test".to_string(), None)
        .expect("board");

    let col_todo = service
        .add_column(&board.id, "To Do".to_string(), None)
        .expect("todo");
    let col_doing = service
        .add_column(&board.id, "Doing".to_string(), None)
        .expect("doing");

    // Create cards in todo
    let card_a = service
        .create_card(&board.id, &col_todo.id, "A".to_string(), None)
        .expect("a");
    let card_b = service
        .create_card(&board.id, &col_todo.id, "B".to_string(), None)
        .expect("b");
    let card_c = service
        .create_card(&board.id, &col_todo.id, "C".to_string(), None)
        .expect("c");

    // Move B to doing at position 0
    service
        .move_card(&board.id, &card_b.id, &col_doing.id, 0)
        .expect("move b");

    // Verify todo has A, C
    let todo_cards = service
        .list_cards_in_column(&board.id, &col_todo.id)
        .expect("list todo");
    assert_eq!(todo_cards.len(), 2);
    assert!(todo_cards.iter().any(|c| c.id == card_a.id));
    assert!(todo_cards.iter().any(|c| c.id == card_c.id));

    // Verify doing has B
    let doing_cards = service
        .list_cards_in_column(&board.id, &col_doing.id)
        .expect("list doing");
    assert_eq!(doing_cards.len(), 1);
    assert_eq!(doing_cards[0].id, card_b.id);

    // Verify card's column_id is updated
    let card_b_updated = service.get_card(&board.id, &card_b.id).expect("get b");
    assert_eq!(card_b_updated.column_id, col_doing.id);
}

#[test]
fn test_large_board_performance() {
    let service = make_service("perf-test-peer-id");

    let board = service
        .create_board("project-perf", "Performance Test".to_string(), None)
        .expect("board");

    // Create 10 columns
    let mut columns = Vec::new();
    for i in 0..10 {
        let col = service
            .add_column(&board.id, format!("Column {}", i), None)
            .expect("add column");
        columns.push(col);
    }

    // Create 100 cards (10 per column)
    let mut cards = Vec::new();
    for (col_idx, column) in columns.iter().enumerate() {
        for card_idx in 0..10 {
            let card = service
                .create_card(
                    &board.id,
                    &column.id,
                    format!("Card {}-{}", col_idx, card_idx),
                    None,
                )
                .expect("create card");
            cards.push(card);
        }
    }

    // Verify all cards exist
    let filter = CardFilter::new();
    let all_cards = service.filter_cards(&board.id, filter).expect("filter");
    assert_eq!(all_cards.len(), 100);

    // Move some cards between columns
    for i in 0..20 {
        let card = &cards[i * 5];
        let target_col = &columns[(i + 3) % 10];
        service
            .move_card(&board.id, &card.id, &target_col.id, 0)
            .expect("move card");
    }

    // All cards should still exist
    let filter = CardFilter::new();
    let all_cards = service.filter_cards(&board.id, filter).expect("filter");
    assert_eq!(all_cards.len(), 100);
}
