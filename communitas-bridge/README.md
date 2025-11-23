# Communitas Bridge

HTTP/REST bridge server for browser-based testing and integration.

## Overview

The Communitas Bridge provides HTTP endpoints for testing Communitas functionality without Tauri. It's primarily used for:

- Browser-based testing via Chrome DevTools MCP
- Integration testing
- External service integrations
- Development and debugging

## Features

- **RESTful API**: Full HTTP/REST API for all core operations
- **WebSocket Support**: Real-time updates via WebSocket connections
- **Chrome DevTools MCP Integration**: Seamless testing with Chrome DevTools
- **Swagger/OpenAPI Documentation**: Interactive API documentation
- **CORS Support**: Configurable CORS for browser access

## Quick Start

### Installation

```bash
# From repository root
cargo run -p communitas-bridge
```

The server will start on `http://localhost:3030`

### Configuration

Create a `bridge.toml` file:

```toml
[server]
host = "127.0.0.1"
port = 3030
cors_origins = ["http://localhost:5173", "http://localhost:1420"]

[p2p]
auto_connect = true
bootstrap_nodes = ["bootstrap.communitas.network:8080"]

[storage]
path = "./data"
```

## API Endpoints

### Core Operations

#### Initialize Core
```http
POST /api/core/initialize
Content-Type: application/json

{
  "four_words": "ocean-forest-moon-star",
  "display_name": "Test User",
  "device_name": "Browser Test"
}
```

#### Get Status
```http
GET /api/core/status
```

#### Start Networking
```http
POST /api/network/start
Content-Type: application/json

{}
```

### Channel Operations

#### Create Channel
```http
POST /api/channels
Content-Type: application/json

{
  "name": "general",
  "description": "General discussion",
  "visibility": "public"
}
```

#### List Channels
```http
GET /api/channels
```

#### Get Channel
```http
GET /api/channels/:id
```

### Message Operations

#### Send Message
```http
POST /api/channels/:id/messages
Content-Type: application/json

{
  "content": "Hello, World!",
  "recipients": ["ocean-forest-moon-star"]
}
```

#### Get Messages
```http
GET /api/channels/:id/messages?limit=50&offset=0
```

### Thread Operations

#### Create Thread
```http
POST /api/threads/create
Content-Type: application/json

{
  "message_id": "msg_123",
  "title": "Thread Title"
}
```

#### Reply to Thread
```http
POST /api/threads/:id/reply
Content-Type: application/json

{
  "content": "Thread reply content"
}
```

## WebSocket API

### Connection

```javascript
const ws = new WebSocket('ws://localhost:3030/ws');

ws.onopen = () => {
  // Subscribe to events
  ws.send(JSON.stringify({
    type: 'subscribe',
    channels: ['message', 'presence']
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Event:', data);
};
```

### Event Types

- `message:new` - New message received
- `message:edited` - Message was edited
- `message:deleted` - Message was deleted
- `presence:online` - User came online
- `presence:offline` - User went offline
- `channel:created` - New channel created
- `channel:updated` - Channel was updated

## Testing with Chrome DevTools MCP

The bridge is designed to work seamlessly with Chrome DevTools MCP for comprehensive testing:

```javascript
// Example test flow
async function testCommunitas() {
  // 1. Initialize
  const initResp = await fetch('http://localhost:3030/api/core/initialize', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      four_words: 'ocean-forest-moon-star',
      display_name: 'Test User',
      device_name: 'Browser Test'
    })
  });

  // 2. Create channel
  const channelResp = await fetch('http://localhost:3030/api/channels', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      name: 'test-channel',
      description: 'Test Channel'
    })
  });
  const channel = await channelResp.json();

  // 3. Send message
  await fetch(`http://localhost:3030/api/channels/${channel.id}/messages`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      content: 'Hello from browser!',
      recipients: ['ocean-forest-moon-star']
    })
  });
}
```

See [../../docs/BRIDGE_TESTING.md](../../docs/BRIDGE_TESTING.md) for complete testing scenarios.

## Architecture

```
Browser/Chrome DevTools MCP
    ↓ HTTP/REST
Bridge Server (localhost:3030)
    ↓ Rust IPC
Saorsa Core (P2P Network)
```

### Components

- **HTTP Server**: Actix-web based REST API
- **WebSocket Server**: Real-time event streaming
- **Core Integration**: Direct integration with communitas-core
- **State Management**: Shared state via Arc<RwLock<T>>

## Development

### Building

```bash
cargo build -p communitas-bridge
```

### Testing

```bash
# Unit tests
cargo test -p communitas-bridge

# Integration tests
cargo test -p communitas-bridge --test '*'
```

### Running with Custom Config

```bash
cargo run -p communitas-bridge -- --config custom-bridge.toml
```

## API Documentation

Interactive Swagger UI available at:
```
http://localhost:3030/swagger-ui
```

OpenAPI spec at:
```
http://localhost:3030/api-docs/openapi.json
```

## Security Considerations

⚠️ **Development Only**: This bridge is intended for development and testing only. Do not expose it to the internet without proper security measures.

### Security Features

- CORS protection (configurable)
- Rate limiting per IP
- Input validation
- No authentication required (localhost only)

### Production Considerations

If deploying to production:

1. Enable authentication (JWT/OAuth2)
2. Use HTTPS with valid certificates
3. Implement proper rate limiting
4. Add request logging and monitoring
5. Use a reverse proxy (nginx/Caddy)

## Troubleshooting

### Port Already in Use

```bash
# Check what's using port 3030
lsof -i :3030

# Use a different port
cargo run -p communitas-bridge -- --port 3031
```

### CORS Errors

Update `bridge.toml`:
```toml
[server]
cors_origins = ["http://localhost:5173", "http://your-frontend:port"]
```

### Connection to Core Failed

- Verify saorsa-core is initialized
- Check that P2P network is accessible
- Review logs for connection errors

## Performance

- Single bridge can handle ~1,000 concurrent connections
- WebSocket connections are kept alive with ping/pong
- HTTP connections use connection pooling

## Contributing

See [../../docs/development/contributing.md](../../docs/development/contributing.md)

## License

Dual-licensed under AGPL-3.0-or-later and commercial license.

## See Also

- [Bridge Testing Guide](../../docs/BRIDGE_TESTING.md)
- [Communitas Core](../communitas-core/README.md)
- [Testing Guide](../../docs/guides/testing.md)
