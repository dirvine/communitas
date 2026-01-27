# Connecting Communitas MCP Server to Claude Desktop

This guide shows how to connect your Communitas MCP server to Claude Desktop for AI-powered collaboration.

## What is MCP?

The Model Context Protocol (MCP) allows AI assistants like Claude to interact with external tools and services. Communitas provides an MCP server that exposes:

- **187 tools** for contacts, messaging, kanban, drive, canvas, and more
- **Interactive UI widgets** via MCP Apps extension
- **Real-time collaboration** through the Communitas platform

## Prerequisites

- [Claude Desktop](https://claude.ai/download) installed
- Communitas MCP server running (local or remote)

## Quick Setup

### Option 1: Local MCP Server (Development)

1. Start the Communitas MCP server:
```bash
cd communitas
cargo run -p communitas-mcp -- --demo
```

2. Configure Claude Desktop (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):
```json
{
  "mcpServers": {
    "communitas": {
      "command": "cargo",
      "args": ["run", "-p", "communitas-mcp", "--", "--demo"],
      "cwd": "/path/to/communitas"
    }
  }
}
```

3. Restart Claude Desktop

### Option 2: Remote MCP Server (Production)

1. Configure Claude Desktop to connect to the remote server:
```json
{
  "mcpServers": {
    "communitas": {
      "url": "https://mcp.saorsalabs.com/mcp",
      "transport": "http"
    }
  }
}
```

2. Restart Claude Desktop

## Configuration Options

### Full Configuration Example

```json
{
  "mcpServers": {
    "communitas": {
      "command": "cargo",
      "args": ["run", "-p", "communitas-mcp", "--", "--demo"],
      "cwd": "/Users/you/projects/communitas",
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Configuration Fields

| Field | Description |
|-------|-------------|
| `command` | The command to run (e.g., `cargo`) |
| `args` | Arguments to pass to the command |
| `cwd` | Working directory for the command |
| `env` | Environment variables |
| `url` | URL for HTTP transport (remote servers) |
| `transport` | Transport type: `stdio` (default) or `http` |

### Server Arguments

| Argument | Description |
|----------|-------------|
| `--demo` | Enable demo mode with sample data |
| `--http` | Enable HTTP transport instead of stdio |
| `--tls` | Enable TLS (requires certificates) |
| `--listen <addr>` | Listen address (default: `127.0.0.1:3040`) |
| `--no-client-auth` | Disable client certificate authentication |

## Verifying the Connection

1. Open Claude Desktop
2. Start a new conversation
3. Ask Claude: "What MCP tools are available from Communitas?"

Claude should respond with a list of available tools including:
- Contact management (list_contacts, create_contact, etc.)
- Messaging (list_threads, send_message, etc.)
- Kanban boards (list_kanban_boards, create_kanban_card, etc.)
- Drive/files (list_files, upload_file, etc.)
- Canvas (canvas_get_snapshot, etc.)

## Using Communitas Tools

### Example: Managing Contacts

```
You: List all my contacts
Claude: [Uses list_contacts tool]
Here are your contacts:
1. Alice Smith (alice@example.com)
2. Bob Johnson (bob@example.com)
...

You: Add a new contact for Carol Davis with email carol@example.com
Claude: [Uses create_contact tool]
I've created a new contact for Carol Davis.
```

### Example: Kanban Boards

```
You: Show me my Kanban boards
Claude: [Uses list_kanban_boards tool]
You have 2 Kanban boards:
1. Product Roadmap (12 cards)
2. Sprint Planning (8 cards)

You: Create a new card "Fix login bug" in Sprint Planning, To Do column
Claude: [Uses create_kanban_card tool]
Created card "Fix login bug" in the To Do column.
```

### Example: Messaging

```
You: Show me my recent message threads
Claude: [Uses list_threads tool]
Recent threads:
1. Team Chat (5 new messages)
2. Project Discussion (2 new messages)

You: Send "Meeting at 3pm" to Team Chat
Claude: [Uses send_message tool]
Message sent to Team Chat.
```

## MCP Apps (Interactive UIs)

Communitas supports the MCP Apps extension, which enables interactive UI widgets directly in Claude conversations.

### Available Widgets

| Widget | Description |
|--------|-------------|
| Contacts | Interactive contact list with search |
| Messages | Thread view with message composition |
| Kanban | Drag-drop project boards |
| Drive | File browser with preview |
| Canvas | Collaborative whiteboard viewer |

When Claude uses a Communitas tool, it may render an interactive widget showing the results. You can interact with these widgets directly in the conversation.

## Troubleshooting

### "MCP server not found"

1. Check the config file path is correct
2. Verify the `cwd` path exists
3. Check the server can start manually:
```bash
cd /path/to/communitas
cargo run -p communitas-mcp -- --demo
```

### "Connection refused"

1. For local servers, ensure the server is running
2. For remote servers, verify the URL is correct
3. Check firewall/network settings

### "Tool call failed"

1. Check Claude Desktop logs: `~/Library/Logs/Claude/`
2. Check server logs (if running with `RUST_LOG=debug`)
3. Verify the tool arguments are correct

### Logs Location

| Platform | Log Path |
|----------|----------|
| macOS | `~/Library/Logs/Claude/` |
| Windows | `%APPDATA%\Claude\logs\` |
| Linux | `~/.local/share/Claude/logs/` |

## Advanced Configuration

### Multiple MCP Servers

You can connect to multiple MCP servers:

```json
{
  "mcpServers": {
    "communitas": {
      "command": "cargo",
      "args": ["run", "-p", "communitas-mcp", "--", "--demo"],
      "cwd": "/path/to/communitas"
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/home/user/docs"]
    }
  }
}
```

### Environment Variables

```json
{
  "mcpServers": {
    "communitas": {
      "command": "cargo",
      "args": ["run", "-p", "communitas-mcp", "--", "--demo"],
      "cwd": "/path/to/communitas",
      "env": {
        "RUST_LOG": "communitas_mcp=debug",
        "COMMUNITAS_DATA_DIR": "/custom/data/path"
      }
    }
  }
}
```

## Security Considerations

- **Local servers**: Run with `--demo` for testing; in production, configure proper authentication
- **Remote servers**: Always use HTTPS; consider client certificates for sensitive deployments
- **Data access**: MCP servers can access contacts, messages, files. Review tool permissions carefully

## Related Documentation

- [MCP API Reference](../api/mcp-api.md)
- [MCP Server Deployment](../deployment/mcp-server.md)
- [Communitas Architecture](../architecture/README.md)

## Support

- GitHub Issues: https://github.com/saorsa-labs/communitas/issues
- Email: david@saorsalabs.com
