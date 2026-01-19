# MCP Messaging Parity Testing

This document describes the parity testing harness for MCP messaging and contacts tools, ensuring Dioxus UI and MCP return identical data.

## Overview

The `mcp_messaging.sh` script validates that MCP tools return data consistent with the canonical export binaries from `communitas-core`. This ensures AI agents using MCP see the same data that users see in the Dioxus UI.

## Test Coverage

| Test | MCP Tool | Export Binary | Comparison |
|------|----------|---------------|------------|
| Thread List | `list_threads` | `export_threads` | Thread IDs, count |
| Thread Filtering | `list_threads` (filter param) | `export_threads` (filter arg) | Filter results |
| Contacts with Presence | `list_contacts` (include_presence) | `export_contacts` | Contact IDs, count |
| Contact Filtering | `list_contacts` (filter param) | `export_contacts` (filter arg) | Filter results |

## Running Locally

```bash
# Run all messaging parity tests
./scripts/tests/mcp_messaging.sh

# Use custom port
MCP_HTTP_PORT=7333 ./scripts/tests/mcp_messaging.sh

# Use custom demo identity
MCP_DEMO_FOUR_WORDS="my-test-identity-name" ./scripts/tests/mcp_messaging.sh
```

## JSON Schema

### list_threads Response

```json
{
  "threads": [
    {
      "thread_id": "entity:abc123",
      "entity_id": "abc123",
      "entity_type": "channel",
      "contact_id": null,
      "display_name": "General Discussion",
      "last_message_preview": "Hello everyone!",
      "last_message_timestamp": 1705600000000,
      "unread_count": 3,
      "is_muted": false
    },
    {
      "thread_id": "contact:def456",
      "entity_id": null,
      "entity_type": null,
      "contact_id": "def456",
      "display_name": "Alice",
      "last_message_preview": "See you tomorrow",
      "last_message_timestamp": 1705590000000,
      "unread_count": 0,
      "is_muted": false
    }
  ],
  "total_count": 2,
  "filter": "all"
}
```

### list_contacts Response (with presence)

```json
{
  "contacts": [
    {
      "id": "abc123",
      "display_name": "Alice",
      "four_words": "alpha-bravo-charlie-delta",
      "is_favourite": true,
      "is_online": true,
      "last_seen": 1705600000000,
      "presence_status": "online"
    }
  ],
  "count": 1,
  "filter": "all",
  "include_presence": true
}
```

## CI Integration

The test is wired into `.github/workflows/rust.yml` in the `mcp-parity` job:

```yaml
- name: MCP Messaging Parity
  run: ./scripts/tests/mcp_messaging.sh
  env:
    MCP_HTTP_PORT: 7332
```

### Artifacts

When running in CI, JSON artifacts are saved to `$GITHUB_WORKSPACE/messaging-parity-artifacts/`:

- `mcp_threads_raw.json` - Raw MCP response for threads
- `mcp_threads.json` - Extracted threads array
- `cli_threads_snapshot.json` - Full CLI export output
- `cli_threads.json` - Extracted CLI threads array
- `mcp_contacts_raw.json` - Raw MCP response for contacts
- `mcp_contacts.json` - Extracted contacts array
- `cli_contacts_snapshot.json` - Full CLI export output
- `cli_contacts.json` - Extracted CLI contacts array
- `threads_diff.txt` - Diff output if threads diverge
- `contacts_diff.txt` - Diff output if contacts diverge

## Troubleshooting

### "jq is required"

Install jq:
```bash
# macOS
brew install jq

# Ubuntu/Debian
apt-get install jq
```

### Thread count mismatch

This may occur if:
1. The MCP server and CLI use different demo data seeds
2. There's a timing issue with async operations

Check the diff file in artifacts for specifics.

### Contact presence differs

Presence is time-sensitive. The `is_online` and `last_seen` fields may differ slightly between MCP and CLI calls due to timing.

## Adding New Tests

To add parity tests for new MCP tools:

1. Add export binary to `communitas-core/src/bin/` if needed
2. Add test section to `mcp_messaging.sh`:
   ```bash
   echo "[mcp-messaging] Test N: new_tool parity"
   MCP_RESPONSE=$(post '{"jsonrpc":"2.0","id":N,"method":"tools/call","params":{"name":"new_tool","arguments":{}}}')
   # Compare with CLI export
   ```
3. Update this documentation

## Related Documentation

- [MCP API Reference](../api/mcp-api.md)
- [MCP Nav/Auth Parity](mcp_nav_auth_parity.md)
- [Milestone 2 Architecture](../architecture/dioxus_milestone2_messaging_entities.md)
