# Communitas TUI

Terminal User Interface for testing and interacting with the Communitas backend.

## Overview

Communitas TUI provides a comprehensive text-based interface that mirrors all functionality available in the desktop application. It enables developers and testers to:

- Test all backend operations without a graphical interface
- Verify four-word identity system and networking
- Test channel/message operations with real saorsa-core integration
- Debug and monitor network connections and DHT operations
- Test project management and issue tracking (Linear-style)
- Validate CRDT synchronization

## Features

### ✅ **Phase 1 Completed (Foundation)**
- ✅ CLI argument parsing with clap
- ✅ Event loop with crossterm input handling
- ✅ Ratatui-based UI with dashboard and status bar
- ✅ Backend integration via CoreContext wrapper
- ✅ Navigation system with view stack
- ✅ Key binding handlers
- ✅ Network status monitoring
- ✅ Four-word identity initialization

### ✅ **Phase 2 Completed (Organizations & Channels)**
- ✅ Channel list view with sorting by unread count
- ✅ Channel detail sidebar with member count
- ✅ Message view with scrolling
- ✅ Message composer with input handling
- ✅ Message loading from backend storage
- ✅ Support for different message types (text, rich text, system, encrypted)
- ✅ Real-time message sending via backend
- ✅ Enter key to open channels
- ✅ ESC key to go back to channel list

### 🚧 **Phase 3-6 (Pending)**
- ⏳ Thread creation and replies
- ⏳ Message reactions and emoji picker
- ⏳ Project/issue management UI
- ⏳ Groups and contacts
- ⏳ CRDT sync visualization
- ⏳ Search and filtering

## Offline Capabilities

Communitas TUI features **automatic offline handling** - operations work transparently whether online or offline:

### ✅ **Transparent Operation Queueing**
- Operations execute immediately when network is available
- Operations queue automatically when network is unavailable
- No manual "offline mode" - everything is automatic

### ✅ **Persistent Queue**
- Queued operations survive app restarts
- Operations persist across device shutdowns
- Automatic sync when network returns

### ✅ **Smart Error Handling**
- Network errors → automatic queueing
- Validation errors → immediate feedback
- Duplicate detection during sync

### 📚 **Documentation**
See [docs/architecture/offline-handling.md](../../docs/architecture/offline-handling.md) for comprehensive details on:
- Architecture and design philosophy
- Smart operations API
- Testing strategies
- Performance considerations

## Installation

### Prerequisites
- Rust 1.85+ (edition 2024)
- Tokio async runtime
- saorsa-core 0.5.7+

### Build
```bash
# From project root
cargo build -p communitas-tui

# Or with release optimizations
cargo build -p communitas-tui --release
```

## Usage

### Basic Usage
```bash
# Run TUI with auto-generated identity
cargo run -p communitas-tui

# Specify identity
cargo run -p communitas-tui -- --identity ocean-forest-moon-star --name "Alice" --device "Laptop"

# Offline mode (skip network initialization)
cargo run -p communitas-tui -- --offline

# Debug logging
cargo run -p communitas-tui -- --debug

# Custom data directory
cargo run -p communitas-tui -- --data-dir ~/.communitas-tui-data
```

### CLI Arguments
```
Options:
  -i, --identity <IDENTITY>    Four-word identity (e.g., ocean-forest-moon-star)
  -n, --name <NAME>            Display name [default: TUI User]
  -d, --device <DEVICE>        Device name [default: TUI Device]
      --data-dir <DATA_DIR>    Data directory for storage
      --debug                  Enable debug logging
      --offline                Skip network initialization
  -h, --help                   Print help
  -V, --version                Print version
```

## Key Bindings

### Global
- `q` - Quit application
- `Esc` - Go back / Close modal
- `Tab` - Switch focus between panels
- `Ctrl+C` - Force quit
- `?` or `F1` - Show help

### Navigation
- `↑/k` - Move up
- `↓/j` - Move down
- `←/h` - Move left / Previous panel
- `→/l` - Move right / Next panel
- `Enter` - Select / Open item

### Dashboard Shortcuts
- `o` - Open Organizations (channels)
- `p` - Open Projects (issues)
- `g` - Open Groups
- `c` - Open Contacts
- `n` - Check network status
- `i` - Initialize identity (shows instructions)

### Channel/Message View
- `Enter` - Activate message composer
- `ESC` - Go back to channel list
- Type message and press `Enter` to send

**Coming Soon:**
- `t` - Create thread from message
- `r` - Add reaction to message
- `e` - Edit message (if yours)
- `d` - Delete message (if yours)
- `m` - Mention user (@user)

### Project/Issue View (Coming Soon)
- `n` - Create new issue
- `s` - Change issue status
- `a` - Assign/reassign issue
- `p` - Change priority
- `f` - Filter issues

## UI Layout

### Dashboard
```
┌─────────────────────────────────────────────┐
│ Communitas TUI v0.1.17                      │
├─────────────────────────────────────────────┤
│  Select Entity Type:                        │
│                                             │
│  → 🏢 Organizations  (Press 'o')            │
│    📁 Projects       (Press 'p')            │
│    👥 Groups         (Press 'g')            │
│    👤 Contacts       (Press 'c')            │
│                                             │
│  i: Initialize identity                     │
│  n: Check network status                    │
│  q: Quit                                    │
├─────────────────────────────────────────────┤
│ Identity: ocean-forest-moon-star │ Network:●│
└─────────────────────────────────────────────┘
```

## Architecture

### Crate Structure
```
communitas-tui/
├── src/
│   ├── main.rs           # Entry point + CLI
│   ├── app.rs            # Main App + event loop
│   ├── backend/          # CoreContext integration
│   │   ├── core.rs       # Backend wrapper
│   │   ├── channels.rs   # Channel operations
│   │   ├── messages.rs   # Message operations
│   │   ├── projects.rs   # Project operations
│   │   └── issues.rs     # Issue operations
│   ├── handlers/         # Event handlers
│   │   └── mod.rs        # Key/action handlers
│   ├── state/            # Application state
│   │   ├── app_state.rs  # Global state
│   │   ├── entities.rs   # Entity data structures
│   │   ├── navigation.rs # View navigation
│   │   └── network.rs    # Network state
│   ├── ui/               # UI rendering
│   │   ├── dashboard.rs  # Dashboard view
│   │   ├── layout.rs     # Root layout
│   │   └── status_bar.rs # Status bar
│   └── utils/            # Utilities
│       └── logger.rs     # Tracing setup
```

### Tech Stack
- **ratatui** (0.29): TUI framework
- **crossterm** (0.28): Terminal control
- **clap** (4.5): CLI argument parsing
- **tokio**: Async runtime
- **communitas-core**: Business logic
- **saorsa-core** (0.5.7): P2P networking

## Backend Integration

The TUI integrates directly with `communitas-core::CoreContext`, providing:

- Four-word identity initialization via `DeviceType::Desktop`
- Real DHT connection checking
- Channel creation and message sending
- Bootstrap node management
- Network status monitoring

### Example Backend Operations
```rust
// Initialize identity
backend.initialize(
    "ocean-forest-moon-star".to_string(),
    "Alice".to_string(),
    "TUI Device".to_string(),
).await?;

// Create channel
let channel = backend.create_channel(
    "general".to_string(),
    "General discussion".to_string(),
).await?;

// Send message
let msg_id = backend.send_message_to_channel(
    channel.id,
    "Hello from TUI!".to_string(),
).await?;
```

## Development

### Adding New Views
1. Define view in `state/navigation.rs` (`View` enum)
2. Add rendering function in `ui/layout.rs`
3. Add handler in `handlers/mod.rs`
4. Wire up key bindings in `app.rs`

### Testing
```bash
# Run with debug logging
RUST_LOG=debug cargo run -p communitas-tui -- --debug

# Test with specific identity
cargo run -p communitas-tui -- --identity test-user-four-words

# Test offline mode
cargo run -p communitas-tui -- --offline
```

## Roadmap

### Phase 2: Organizations & Channels (Week 2-3)
- [ ] Channel list view with unread counts
- [ ] Message view with scrolling
- [ ] Message composer with input handling
- [ ] Thread creation and replies
- [ ] Reactions (emoji picker)
- [ ] Member sidebar

### Phase 3: Projects & Issues (Week 3-4)
- [ ] Project list view
- [ ] Issue swim lanes (Backlog/Todo/InProgress/Done/Canceled)
- [ ] Issue detail view
- [ ] Comments and assignee management
- [ ] Status/priority updates
- [ ] Filtering and search

### Phase 4: Groups & Contacts (Week 4-5)
- [ ] Groups list with unread counts
- [ ] WhatsApp-style group messaging
- [ ] Contacts list (four-word address book)
- [ ] Direct messaging

### Phase 5: Advanced Features (Week 5-6)
- [ ] CRDT sync status visualization
- [ ] Network management (bootstrap nodes)
- [ ] Search and filtering across entities
- [ ] Slash commands (e.g., `/create`, `/join`)
- [ ] Command palette

### Phase 6: Polish & Testing (Week 6+)
- [ ] Comprehensive test coverage
- [ ] Performance optimization (1000+ messages)
- [ ] Error handling improvements
- [ ] Documentation and examples
- [ ] CI/CD integration

## Troubleshooting

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build -p communitas-tui
```

### Network Issues
- Check `--offline` mode for testing without network
- Verify bootstrap nodes are reachable
- Enable debug logging to see network activity

### Identity Initialization
- Use `--identity` flag for consistent testing
- Identity is persisted in data directory
- Check logs for initialization errors

## Contributing

When adding features:
1. Follow existing patterns (handlers → state → UI)
2. Update this README with new key bindings
3. Add comprehensive logging
4. Test with both online and offline modes
5. Ensure zero panics in production code

## License

Dual-licensed under AGPL-3.0-or-later and commercial license.
See LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md for details.

## See Also

- [Communitas Core](../communitas-core/README.md) - Core library documentation
- [Communitas Desktop](../communitas-desktop/README.md) - Desktop application
- [Communitas Headless](../communitas-headless/README.md) - Headless daemon
- [Architecture Documentation](../docs/architecture/) - System architecture
- [DESIGN.md](../DESIGN.md) - Complete system design
- [Development Guide](../docs/development/) - Development resources
- [API Reference](../docs/AGENTS_API.md) - Complete API documentation
- [Communitas Project](https://communitas.life) - Official website
