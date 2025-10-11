# TUI Control API - Testing Branch

This worktree contains the HTTP Control API for communitas-tui, enabling MCP-driven automated testing of P2P gossip and QUIC networking.

## Status: Phase 3 Partially Complete ⚡

- ✅ Phase 0: Fixed CoreContext API mismatches
- ✅ Phase 1: HTTP Control API Foundation
- ✅ Phase 2: Authentication & Entity Persistence
  - ✅ Vault creation and login endpoints
  - ✅ EntityManager with JSON persistence
  - ✅ Entities persist across restarts
- ⚡ Phase 3: Multi-Instance CRDT Testing (Offline Mode Testing Complete)
  - ✅ Multi-instance startup and authentication
  - ✅ Entity creation and message sending
  - ✅ MessageSyncService integration verified
  - ⚠️ Network CRDT sync requires removing --offline flag
- ⏳ Phase 4: Website Publishing
- ⏳ Phase 5: NAT Traversal & Cloud Testing

## Quick Start

### Start the Control API

```bash
# API-only mode (no TUI, can run in background)
cargo run -p communitas-tui -- --control-port 3040 --offline --api-only

# With TUI (for interactive testing)
cargo run -p communitas-tui -- --control-port 3040 --offline
```

### Test Endpoints

```bash
# Health check
curl http://localhost:3040/health

# Get current identity
curl http://localhost:3040/api/identity/current

# Network status
curl http://localhost:3040/api/network/status

# List entities
curl http://localhost:3040/api/entities

# Run full test suite
./test_control_api.sh
```

## Available Endpoints

### System
- `GET /health` - Health check with version

### Authentication
- `POST /api/auth/vault` - Create new vault and login (auto-generates identity if not provided)
- `POST /api/auth/login` - Login with existing vault
- `POST /api/auth/logout` - Logout current session

### Identity
- `GET /api/identity/current` - Get current identity and login status

### Network
- `GET /api/network/status` - Check network connection status

### Entities (Contacts, Groups, Channels)
- `POST /api/entities` - Create new entity
- `GET /api/entities` - List all entities
- `GET /api/entities/:id/messages` - Get messages for entity

### Messages
- `POST /api/messages/send` - Send message to entity

## Architecture

### Backend Separation
The control API creates a separate Backend instance from the TUI, simulating multiple nodes:
- TUI Backend: `data_dir/` (for interactive use)
- Control API Backend: `data_dir/control/` (for HTTP automation)

This allows testing P2P interactions between the TUI and control API instances.

### Message Flow
```
MCP/Browser/curl
    ↓ HTTP REST
Control API (localhost:3040)
    ↓ Backend
MessageSyncService (CRDT)
    ↓ Gossip
P2P Network
```

## Implementation Details

### Files Modified
- `communitas-tui/Cargo.toml` - Added axum dependencies
- `communitas-tui/src/main.rs` - Added CLI flags and API server
- `communitas-tui/src/backend/core.rs` - Fixed CoreContext API
- `communitas-tui/src/backend/messages.rs` - Updated to MessageSyncService
- `communitas-tui/src/backend/channels.rs` - Entity-based design
- `communitas-tui/src/handlers/mod.rs` - Stubbed for HTTP control

### Files Created
- `communitas-tui/src/control_api/` - Complete HTTP API module
  - `mod.rs` - Module exports and documentation
  - `types.rs` - Request/response types
  - `handlers.rs` - Endpoint handlers
  - `routes.rs` - Route configuration
  - `server.rs` - HTTP server setup
- `test_control_api.sh` - Testing script
- `IMPLEMENTATION_PLAN.md` - Detailed implementation plan
- `TUI_CONTROL_API.md` - This file

## Phase 3 Testing Results

### What Works ✅
- **Multi-Instance Startup**: Successfully running two independent TUI instances
- **Authentication**: Vault creation and login working on both instances
- **Entity Management**: Creating channels/groups with member lists
- **Message Storage**: MessageSyncService storing messages with CRDT metadata
- **Message Retrieval**: GET endpoint returning messages for entities
- **Separate Data**: Each instance maintains independent vault and entity storage

### What Requires Network Testing ⚠️
- **CRDT Synchronization**: Messages don't sync between instances in offline mode
- **Entity Discovery**: Entity metadata is local-only, not shared via P2P
- **Vector Clocks**: Can't test conflict resolution without network sync
- **Anti-Entropy**: Requires active gossip connections for catchup

### Key Architectural Findings
1. **Offline Mode Limitation**: `--offline` flag prevents all P2P connections
2. **Two-Layer Architecture**:
   - **Entity Layer**: Local metadata (EntityManager → entities.json)
   - **Message Layer**: CRDT sync via MessageSyncService over gossip
3. **Network Required For**:
   - Message synchronization between instances
   - Entity discovery and sharing
   - CRDT conflict resolution testing

## Next Steps (See IMPLEMENTATION_PLAN.md)

### Complete Phase 3: Network CRDT Testing
- Remove `--offline` flag to enable P2P networking
- Test message synchronization via gossip overlay
- Verify CRDT vector clocks and conflict resolution
- Test entity discovery mechanisms
- Verify partition tolerance and anti-entropy

### Phase 4: Website Publishing
- Add website publishing endpoints
- Integrate DocReplicator with StorageMode::Web
- Test markdown publishing via gossip

### Phase 5: NAT Traversal & Cloud Testing
- Deploy to Digital Ocean
- Test local → cloud connections
- Verify QUIC NAT hole punching
- Test partition tolerance

## Testing Strategy

### Local Testing
```bash
# Terminal 1: Start instance 1
cargo run -p communitas-tui -- --control-port 3040 --offline --api-only

# Terminal 2: Start instance 2
cargo run -p communitas-tui -- --control-port 3041 --offline --api-only

# Terminal 3: Test with curl/MCP
curl http://localhost:3040/health
curl http://localhost:3041/health
```

### LAN Testing
```bash
# Machine 1 (192.168.1.100)
cargo run -p communitas-tui -- --control-port 3040

# Machine 2 (192.168.1.101)
cargo run -p communitas-tui -- --control-port 3040

# Test from either machine
curl http://192.168.1.100:3040/api/network/status
curl http://192.168.1.101:3040/api/network/status
```

### Cloud Testing (Digital Ocean)
```bash
# Cloud instance (Droplet)
ssh droplet
cd communitas
cargo run -p communitas-tui -- --control-port 3040

# Local machine
curl http://DROPLET_IP:3040/health
```

## MCP Integration

The HTTP Control API can be controlled via MCP servers:

### Chrome DevTools MCP
For browser-based testing:
```javascript
// Via Chrome DevTools MCP
await fetch('http://localhost:3040/api/entities', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'Test Channel',
    entity_type: 'Channel',
    members: []
  })
});
```

### Custom MCP Server
For automated test scenarios:
```python
# Python MCP server
import requests

def test_messaging():
    # Create entity
    entity = requests.post('http://localhost:3040/api/entities', json={
        'name': 'Test Group',
        'entity_type': 'Group',
        'members': ['ocean-forest-moon-star']
    }).json()

    # Send message
    requests.post('http://localhost:3040/api/messages/send', json={
        'entity_id': entity['id'],
        'entity_type': 'Group',
        'text': 'Hello from MCP!'
    })
```

## Troubleshooting

### Port Already in Use
```bash
# Find process using port 3040
lsof -i :3040

# Kill process
pkill -f "communitas-tui.*control-port"
```

### Terminal Device Error
Use `--api-only` flag to run without TUI:
```bash
cargo run -p communitas-tui -- --control-port 3040 --api-only
```

### Network Connection Issues
Start in offline mode for testing:
```bash
cargo run -p communitas-tui -- --control-port 3040 --offline --api-only
```

## Commit History
- Initial: Fixed CoreContext API mismatches (Phase 0)
- Added: HTTP Control API with axum (Phase 1)
- Added: CLI flags --control-port and --api-only
- Added: Test script and documentation

## Related Documentation
- `IMPLEMENTATION_PLAN.md` - Detailed 5-phase plan
- `CLAUDE.md` - Project overview and architecture
- `docs/BRIDGE_TESTING.md` - Browser bridge server docs
- `AGENTS_API.md` - Complete API surface documentation
