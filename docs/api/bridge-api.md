# Bridge Server API Reference

HTTP/REST API reference for the Communitas bridge server (`communitas-bridge`).

## Overview

The bridge server provides an HTTP/REST interface for browser-based testing and integration. It bridges HTTP requests to the Rust `saorsa-core` P2P network, enabling testing with Chrome DevTools MCP and other HTTP clients.

**Base URL**: `http://localhost:3030`

**Purpose**:
- Browser-based testing with Chrome DevTools MCP
- Integration testing without Tauri
- API prototyping and development
- Cross-platform testing

---

## Quick Start

```bash
# Terminal 1: Start bridge server
cargo run -p communitas-bridge

# Terminal 2: Make requests
curl http://localhost:3030/health
```

---

## Authentication

The bridge server does not require authentication for local development. In production deployments, use:
- API keys in `Authorization` header
- IP whitelist
- TLS/SSL certificates

---

## Endpoints

### Health Check

#### `GET /health`

Check if the bridge server is running.

**Response**:
```json
{
  "status": "ok",
  "version": "0.1.0",
  "uptime": 12345
}
```

**Example**:
```bash
curl http://localhost:3030/health
```

---

### Core Initialization

#### `POST /api/core/initialize`

Initialize the core context with a four-word identity.

**Request Body**:
```json
{
  "four_words": "ocean-forest-moon-star",
  "display_name": "Alice",
  "device_name": "Browser Test"
}
```

**Response**:
```json
{
  "success": true,
  "four_words": "ocean-forest-moon-star",
  "peer_id": "peer-uuid-here"
}
```

**Example**:
```bash
curl -X POST http://localhost:3030/api/core/initialize \
  -H "Content-Type: application/json" \
  -d '{
    "four_words": "ocean-forest-moon-star",
    "display_name": "Alice",
    "device_name": "Browser"
  }'
```

**Errors**:
- `400 Bad Request` - Invalid four-word address or missing fields
- `409 Conflict` - Core already initialized

---

### Channel Management

#### `POST /api/channels`

Create a new channel.

**Request Body**:
```json
{
  "name": "General",
  "description": "General discussion"
}
```

**Response**:
```json
{
  "id": "channel-uuid",
  "name": "General",
  "description": "General discussion",
  "created_at": 1699876543,
  "member_count": 1
}
```

**Example**:
```bash
curl -X POST http://localhost:3030/api/channels \
  -H "Content-Type: application/json" \
  -d '{
    "name": "General",
    "description": "General discussion"
  }'
```

---

#### `GET /api/channels`

List all channels.

**Response**:
```json
{
  "channels": [
    {
      "id": "channel-1",
      "name": "General",
      "description": "General discussion",
      "created_at": 1699876543,
      "member_count": 5
    },
    {
      "id": "channel-2",
      "name": "Engineering",
      "description": null,
      "created_at": 1699876600,
      "member_count": 3
    }
  ]
}
```

**Example**:
```bash
curl http://localhost:3030/api/channels
```

---

#### `GET /api/channels/:id`

Get a specific channel by ID.

**Response**:
```json
{
  "id": "channel-1",
  "name": "General",
  "description": "General discussion",
  "created_at": 1699876543,
  "member_count": 5
}
```

**Example**:
```bash
curl http://localhost:3030/api/channels/channel-1
```

**Errors**:
- `404 Not Found` - Channel doesn't exist

---

### Messaging

#### `POST /api/channels/:id/messages`

Send a message to a channel.

**Request Body**:
```json
{
  "content": "Hello, everyone!",
  "recipients": ["ocean-forest-moon-star"]
}
```

**Response**:
```json
{
  "id": "message-uuid",
  "channel_id": "channel-1",
  "author_id": "user-uuid",
  "content": "Hello, everyone!",
  "created_at": 1699876543,
  "thread_id": null
}
```

**Example**:
```bash
curl -X POST http://localhost:3030/api/channels/channel-1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello from the bridge!",
    "recipients": ["ocean-forest-moon-star"]
  }'
```

**Errors**:
- `404 Not Found` - Channel doesn't exist
- `400 Bad Request` - Invalid message content

---

#### `GET /api/channels/:id/messages`

Get messages from a channel.

**Query Parameters**:
- `limit` (optional): Maximum messages to return (default: 50)
- `offset` (optional): Offset for pagination (default: 0)

**Response**:
```json
{
  "messages": [
    {
      "id": "msg-1",
      "channel_id": "channel-1",
      "author_id": "user-1",
      "content": "Hello!",
      "created_at": 1699876543,
      "thread_id": null
    },
    {
      "id": "msg-2",
      "channel_id": "channel-1",
      "author_id": "user-2",
      "content": "Hi there!",
      "created_at": 1699876600,
      "thread_id": null
    }
  ],
  "total": 2
}
```

**Example**:
```bash
curl "http://localhost:3030/api/channels/channel-1/messages?limit=50&offset=0"
```

---

### Threads

#### `POST /api/threads/create`

Create a thread from a message.

**Request Body**:
```json
{
  "parent_message_id": "message-uuid"
}
```

**Response**:
```json
{
  "id": "thread-uuid",
  "channel_id": "channel-1",
  "parent_message_id": "message-uuid",
  "created_at": 1699876543,
  "reply_count": 0
}
```

**Example**:
```bash
curl -X POST http://localhost:3030/api/threads/create \
  -H "Content-Type: application/json" \
  -d '{
    "parent_message_id": "msg-1"
  }'
```

---

#### `GET /api/threads/:id/replies`

Get all replies in a thread.

**Response**:
```json
{
  "replies": [
    {
      "id": "reply-1",
      "channel_id": "channel-1",
      "author_id": "user-2",
      "content": "Great point!",
      "created_at": 1699876600,
      "thread_id": "thread-1"
    }
  ],
  "total": 1
}
```

**Example**:
```bash
curl http://localhost:3030/api/threads/thread-1/replies
```

---

### Network Status

#### `GET /api/network/status`

Get current P2P network status.

**Response**:
```json
{
  "status": "connected",
  "peer_count": 5,
  "peer_id": "peer-uuid",
  "listen_addresses": [
    "/ip4/192.168.1.100/tcp/8080"
  ]
}
```

**Example**:
```bash
curl http://localhost:3030/api/network/status
```

---

#### `GET /api/network/peers`

List connected peers.

**Response**:
```json
{
  "peers": [
    {
      "peer_id": "peer-1",
      "address": "192.168.1.101:8080",
      "four_words": "valley-river-cloud-wind",
      "connected_at": 1699876543
    },
    {
      "peer_id": "peer-2",
      "address": "192.168.1.102:8080",
      "four_words": "mountain-lake-tree-bird",
      "connected_at": 1699876600
    }
  ],
  "total": 2
}
```

**Example**:
```bash
curl http://localhost:3030/api/network/peers
```

---

## Error Responses

All errors follow a consistent format:

```json
{
  "error": "error_code",
  "message": "Human-readable error description",
  "details": {
    "field": "Additional context"
  }
}
```

### HTTP Status Codes

- `200 OK` - Success
- `201 Created` - Resource created
- `400 Bad Request` - Invalid input
- `401 Unauthorized` - Authentication required
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `409 Conflict` - Conflict with existing resource
- `500 Internal Server Error` - Server error

### Error Examples

**Invalid four-word address**:
```json
{
  "error": "invalid_address",
  "message": "Invalid four-word address: word not in dictionary",
  "details": {
    "invalid_word": "invalid"
  }
}
```

**Channel not found**:
```json
{
  "error": "not_found",
  "message": "Channel not found",
  "details": {
    "channel_id": "channel-123"
  }
}
```

**Core not initialized**:
```json
{
  "error": "not_initialized",
  "message": "Core context not initialized. Call /api/core/initialize first."
}
```

---

## Rate Limiting

The bridge server implements rate limiting to prevent abuse:

- **Global**: 100 requests per minute
- **Per-IP**: 50 requests per minute
- **Endpoint-specific**: 10 message sends per second

**Rate Limit Headers**:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1699876543
```

**Rate Limit Exceeded**:
```json
{
  "error": "rate_limit_exceeded",
  "message": "Rate limit exceeded. Try again in 30 seconds.",
  "retry_after": 30
}
```

---

## CORS

The bridge server supports CORS for browser-based testing:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
```

For production, configure specific origins in `bridge-config.toml`:

```toml
[cors]
allowed_origins = ["https://app.communitas.life"]
allowed_methods = ["GET", "POST"]
```

---

## Configuration

### bridge-config.toml

```toml
[server]
host = "127.0.0.1"
port = 3030
workers = 4

[cors]
enabled = true
allowed_origins = ["*"]

[rate_limit]
global_per_minute = 100
per_ip_per_minute = 50

[p2p]
bootstrap_nodes = [
  "ocean-forest-moon-star",  # 192.168.1.100:8080
  "valley-river-cloud-wind"  # 10.0.1.50:8080
]
```

---

## Testing with Chrome DevTools MCP

### Setup

1. Start bridge server:
```bash
cargo run -p communitas-bridge
```

2. Launch Chrome DevTools MCP:
```bash
npx chrome-devtools-mcp@latest
```

3. Navigate to `http://localhost:5173` (frontend)

### Example Test Flow

```javascript
// 1. Initialize core
await fetch('http://localhost:3030/api/core/initialize', {
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
    name: 'Test Channel',
    description: 'Created from browser'
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

// 4. Get messages
const messagesResp = await fetch(
  `http://localhost:3030/api/channels/${channel.id}/messages`
);
const messages = await messagesResp.json();
console.log('Messages:', messages);
```

---

## Complete Example

```javascript
// Complete channel creation and messaging flow
async function testChannelFlow() {
  const baseUrl = 'http://localhost:3030';

  // 1. Initialize
  console.log('Initializing...');
  await fetch(`${baseUrl}/api/core/initialize`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      four_words: 'ocean-forest-moon-star',
      display_name: 'Alice',
      device_name: 'Browser'
    })
  });

  // 2. Create channel
  console.log('Creating channel...');
  const channelResp = await fetch(`${baseUrl}/api/channels`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      name: 'General',
      description: 'General discussion'
    })
  });
  const channel = await channelResp.json();
  console.log('Channel created:', channel.id);

  // 3. Send messages
  console.log('Sending messages...');
  for (let i = 1; i <= 5; i++) {
    await fetch(`${baseUrl}/api/channels/${channel.id}/messages`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        content: `Message ${i}`,
        recipients: ['ocean-forest-moon-star']
      })
    });
  }

  // 4. Create thread
  console.log('Creating thread...');
  const messagesResp = await fetch(
    `${baseUrl}/api/channels/${channel.id}/messages`
  );
  const { messages } = await messagesResp.json();
  const firstMessage = messages[0];

  const threadResp = await fetch(`${baseUrl}/api/threads/create`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      parent_message_id: firstMessage.id
    })
  });
  const thread = await threadResp.json();
  console.log('Thread created:', thread.id);

  // 5. Reply to thread
  console.log('Replying to thread...');
  await fetch(`${baseUrl}/api/channels/${channel.id}/messages`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      content: 'Thread reply',
      recipients: ['ocean-forest-moon-star'],
      thread_id: thread.id
    })
  });

  console.log('Test complete!');
}

// Run test
testChannelFlow().catch(console.error);
```

---

## See Also

- [Tauri Commands API](tauri-commands.md) - Desktop IPC interface
- [Core API](core-api.md) - Rust library API
- [Frontend API](frontend-api.md) - TypeScript/React APIs
- [Testing Guide](../guides/testing.md) - Complete testing guide
- [Bridge README](../../communitas-bridge/README.md) - Bridge server setup and usage

---

**Bridge API**: HTTP/REST interface for browser testing. 🌉🔬
