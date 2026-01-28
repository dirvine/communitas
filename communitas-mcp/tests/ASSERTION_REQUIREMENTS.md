# MCP Test Assertion Requirements

This document defines the minimum assertion requirements for each tool category.

## Available Assertions

| Method | Purpose | Example |
|--------|---------|---------|
| `assert_success()` | Verify call succeeded | Always required for happy path |
| `assert_error()` | Verify call failed | For error case tests |
| `assert_has(key)` | Verify field exists | `assert_has("id")` |
| `assert_non_empty(key)` | Verify non-empty string | `assert_non_empty("id")` |
| `assert_str_eq(key, val)` | Verify string value | `assert_str_eq("name", "Alice")` |
| `assert_bool(key)` | Verify is boolean | `assert_bool("is_active")` |
| `assert_bool_eq(key, val)` | Verify boolean value | `assert_bool_eq("success", true)` |
| `assert_num_gt(key, n)` | Verify number > n | `assert_num_gt("count", 0)` |
| `assert_num_gte(key, n)` | Verify number >= n | `assert_num_gte("total", 1)` |
| `assert_one_of(key, vals)` | Verify string in list | `assert_one_of("status", &["online", "offline"])` |
| `assert_array_min(key, n)` | Verify array >= n items | `assert_array_min("items", 1)` |
| `assert_array_len(key, n)` | Verify array exactly n | `assert_array_len("results", 5)` |
| `assert_array_empty(key)` | Verify array is empty | `assert_array_empty("errors")` |
| `assert_null(key)` | Verify field is null | `assert_null("deleted_at")` |
| `assert_not_null(key)` | Verify not null | `assert_not_null("created_at")` |
| `assert_contains(pattern)` | Verify content contains | `assert_contains("success")` |

---

## Category Requirements

### identity (9 tools)

| Tool | Required Assertions |
|------|---------------------|
| `authenticate` | `assert_has("session_id")`, `assert_non_empty("session_id")` |
| `create_vault` | `assert_has("vault_id")`, `assert_non_empty("vault_id")` |
| `get_identity` | `assert_has("pubkey_hex")`, `assert_has("display_name")` |
| `get_preferences` | `assert_has("theme")`, `assert_has("notifications")` |
| `get_profile` | `assert_has("display_name")`, `assert_has("pubkey_hex")` |
| `list_vaults` | `assert_has("vaults")`, `assert_array_min("vaults", 0)` |
| `set_preference` | `assert_success()`, `assert_bool_eq("updated", true)` |
| `update_profile` | `assert_has("display_name")` |
| `validate_recovery_phrase` | `assert_has("valid")`, `assert_bool("valid")` |

### entities (11 tools)

| Tool | Required Assertions |
|------|---------------------|
| `create_entity` | `assert_has("id")`, `assert_has("name")`, `assert_has("entity_type")` |
| `get_entity` | `assert_has("id")`, `assert_has("name")` |
| `list_entities` | `assert_has("entities")` |
| `update_entity` | `assert_has("id")`, `assert_has("name")` |
| `delete_entity` | `assert_bool_eq("deleted", true)` |
| `archive_entity` | `assert_bool_eq("archived", true)` |
| `get_entity_settings` | `assert_has("settings")` |
| `set_entity_settings` | `assert_success()` |
| `list_entity_members` | `assert_has("members")` |
| `get_entity_stats` | `assert_has("member_count")` |
| `search_entities` | `assert_has("results")` |

### members (8 tools)

| Tool | Required Assertions |
|------|---------------------|
| `add_member` | `assert_has("member_id")` |
| `remove_member` | `assert_bool_eq("removed", true)` |
| `update_member_role` | `assert_has("role")` |
| `list_members` | `assert_has("members")` |
| `get_member` | `assert_has("id")`, `assert_has("role")` |
| `invite_member` | `assert_has("invite_id")` |
| `accept_invite` | `assert_has("member_id")` |
| `decline_invite` | `assert_bool_eq("declined", true)` |

### contacts (13 tools)

| Tool | Required Assertions |
|------|---------------------|
| `create_contact` | `assert_has("id")`, `assert_has("name")` |
| `get_contact_by_id` | `assert_has("id")`, `assert_has("name")` |
| `list_contacts` | `assert_has("contacts")` |
| `update_contact` | `assert_has("id")` |
| `delete_contact` | `assert_bool_eq("deleted", true)` |
| `search_contacts` | `assert_has("results")` |
| `block_contact` | `assert_bool_eq("blocked", true)` |
| `unblock_contact` | `assert_bool_eq("unblocked", true)` |
| `favorite_contact` | `assert_bool_eq("favorited", true)` |
| `unfavorite_contact` | `assert_bool_eq("unfavorited", true)` |
| `list_blocked_contacts` | `assert_has("contacts")` |
| `list_favorite_contacts` | `assert_has("contacts")` |
| `get_contact_presence` | `assert_has("status")` |

### presence (13 tools)

| Tool | Required Assertions |
|------|---------------------|
| `set_presence` | `assert_has("status")` |
| `get_presence` | `assert_has("status")`, `assert_one_of("status", &["online", "offline", "away", "busy", "dnd"])` |
| `subscribe_presence` | `assert_has("subscription_id")` |
| `unsubscribe_presence` | `assert_bool_eq("unsubscribed", true)` |
| `list_presence` | `assert_has("presences")` |
| Other tools | `assert_success()`, `assert_has("status")` |

### messaging (20 tools)

| Tool | Required Assertions |
|------|---------------------|
| `send_message` | `assert_has("message_id")` |
| `list_threads` | `assert_has("threads")` |
| `get_thread` | `assert_has("id")`, `assert_has("participants")` |
| `get_thread_messages` | `assert_has("messages")` |
| `add_reaction` | `assert_bool_eq("added", true)` |
| `remove_reaction` | `assert_bool_eq("removed", true)` |
| `mark_read` | `assert_success()` |
| `mark_unread` | `assert_success()` |
| `search_messages` | `assert_has("results")` |
| `pin_message` | `assert_bool_eq("pinned", true)` |
| `unpin_message` | `assert_bool_eq("unpinned", true)` |
| Other tools | Verify key response fields exist |

### kanban (22 tools)

| Tool | Required Assertions |
|------|---------------------|
| `create_board` | `assert_has("id")`, `assert_has("name")` |
| `get_board` | `assert_has("id")`, `assert_has("columns")` |
| `list_boards` | `assert_has("boards")` |
| `create_column` | `assert_has("id")`, `assert_has("board_id")` |
| `create_card` | `assert_has("id")`, `assert_has("column_id")` |
| `move_card` | `assert_has("id")`, `assert_has("column_id")` |
| `update_card` | `assert_has("id")` |
| `delete_card` | `assert_bool_eq("deleted", true)` |
| Other tools | Verify key response fields exist |

### drive (11 tools)

| Tool | Required Assertions |
|------|---------------------|
| `list_files` | `assert_has("files")` |
| `get_file` | `assert_has("id")`, `assert_has("name")`, `assert_has("size")` |
| `create_directory` | `assert_has("id")`, `assert_has("path")` |
| `delete_file` | `assert_bool_eq("deleted", true)` |
| `rename_file` | `assert_has("id")`, `assert_has("name")` |
| `move_file` | `assert_has("id")`, `assert_has("path")` |
| `copy_file` | `assert_has("id")` |
| Other tools | Verify key response fields exist |

### drive_staging (11 tools)

| Tool | Required Assertions |
|------|---------------------|
| `stage_file` | `assert_has("staging_id")` |
| `commit_staged` | `assert_has("file_id")` |
| `cancel_staged` | `assert_bool_eq("cancelled", true)` |
| `list_staged` | `assert_has("staged")` |
| Other tools | Verify operation succeeded |

### drive_upload / drive_download (9 tools)

| Tool | Required Assertions |
|------|---------------------|
| `upload_file` | `assert_has("file_id")` |
| `download_file` | `assert_has("content")` or content validation |
| `get_upload_progress` | `assert_has("progress")`, `assert_num_gte("progress", 0)` |
| `get_download_progress` | `assert_has("progress")`, `assert_num_gte("progress", 0)` |
| Other tools | Verify operation succeeded |

### canvas (9 tools)

| Tool | Required Assertions |
|------|---------------------|
| `create_canvas` | `assert_has("id")`, `assert_has("name")` |
| `get_canvas` | `assert_has("id")`, `assert_has("elements")` |
| `add_element` | `assert_has("element_id")` |
| `update_element` | `assert_has("element_id")` |
| `delete_element` | `assert_bool_eq("deleted", true)` |
| Other tools | Verify key response fields exist |

### canvas_history (6 tools)

| Tool | Required Assertions |
|------|---------------------|
| `canvas_undo` | `assert_success()` or `assert_has("action")` |
| `canvas_redo` | `assert_success()` or `assert_has("action")` |
| `get_canvas_history` | `assert_has("history")` |
| Other tools | Verify operation succeeded |

### call (11 tools)

| Tool | Required Assertions |
|------|---------------------|
| `start_call` | `assert_has("call_id")` |
| `join_call` | `assert_has("call_id")`, `assert_has("participant_id")` |
| `end_call` | `assert_bool_eq("ended", true)` |
| `get_call_participants` | `assert_has("participants")` |
| `get_call_quality` | `assert_has("rtt")`, `assert_has("packet_loss")` |
| `toggle_mute` | `assert_has("muted")`, `assert_bool("muted")` |
| `toggle_video` | `assert_has("video_enabled")`, `assert_bool("video_enabled")` |
| Other tools | Verify key response fields exist |

### network (9 tools)

| Tool | Required Assertions |
|------|---------------------|
| `get_connection_status` | `assert_has("connected")`, `assert_bool("connected")` |
| `list_peers` | `assert_has("peers")` |
| `connect_peer` | `assert_has("peer_id")` |
| `disconnect_peer` | `assert_bool_eq("disconnected", true)` |
| Other tools | Verify operation succeeded |

### offline_queue (6 tools)

| Tool | Required Assertions |
|------|---------------------|
| `list_queued_operations` | `assert_has("operations")` |
| `retry_operation` | `assert_success()` |
| `skip_operation` | `assert_success()` |
| `queue_operation` | `assert_has("operation_id")` |
| Other tools | Verify operation succeeded |

---

## Test Pattern

Every test should follow this pattern:

```rust
#[tokio::test]
async fn test_tool_name() {
    let client = McpTestClient::new().await;

    // Arrange - set up any prerequisites
    // (e.g., create contact before testing get_contact)

    // Act - call the tool
    let result = client
        .call_tool("tool_name", json!({
            "param1": "value1"
        }))
        .await;

    // Assert - verify the response
    result
        .assert_success()
        .assert_has("id")
        .assert_non_empty("id")
        .assert_has("expected_field");
}
```

---

## Priority Order

Complete stubs in this order (highest impact first):

1. **call** (7 stubs) - Core real-time functionality
2. **canvas** (9 stubs) - Collaboration feature
3. **drive_staging** (12 stubs) - File management flow
4. **messaging** (10 stubs) - Core communication
5. **presence** (10 stubs) - User status
6. **drive** (5 stubs) - File operations
7. **canvas_history** (6 stubs) - Undo/redo
8. **canvas_view** (4 stubs) - View management
9. **network** (3 stubs) - Connectivity
10. **offline_queue** (5 stubs) - Offline support
11. **Remaining** (22 stubs) - Other categories
