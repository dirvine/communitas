# Communitas API Documentation

Complete API reference for the Communitas local-first collaboration platform.

## Overview

Communitas provides multiple API layers for different integration scenarios:

```
┌─────────────────────────────────────────────────────────────┐
│                   Frontend APIs (TypeScript)                 │
│  - React Context APIs (Auth, Navigation, Encryption)        │
│  - Service APIs (Network, Storage, Messaging)               │
│  - WebRTC & Collaboration APIs                              │
└─────────────────────────────────────────────────────────────┘
                            ↓ Tauri IPC
┌─────────────────────────────────────────────────────────────┐
│                 Tauri Commands (Desktop API)                 │
│  - Authentication (40+ commands)                             │
│  - Organization & Channels (20+ commands)                    │
│  - Projects & Issues (15+ commands)                          │
│  - Member Management (10+ commands)                          │
│  - Document Sync (8+ commands)                               │
└─────────────────────────────────────────────────────────────┘
                            ↓ Core API
┌─────────────────────────────────────────────────────────────┐
│                Core Library API (Rust)                       │
│  - communitas-core: Identity, Groups, Storage                │
│  - saorsa-pqc: Post-quantum cryptography                     │
│  - ant-quic: QUIC transport                                  │
└─────────────────────────────────────────────────────────────┘
                            ↓ HTTP/REST
┌─────────────────────────────────────────────────────────────┐
│              Bridge Server API (Testing/Integration)         │
│  - HTTP/REST endpoints for browser testing                   │
│  - Real P2P integration via saorsa-core                      │
│  - Chrome DevTools MCP compatible                            │
└─────────────────────────────────────────────────────────────┘
```

## API Layers

### 1. [Frontend APIs](frontend-api.md) (TypeScript/React)

**Purpose**: Client-side application development in the React frontend

**Key Components**:
- **React Contexts**: `AuthContext`, `NavigationContext`, `EncryptionContext`
- **Service APIs**: Network, Storage, Messaging, WebRTC
- **Utilities**: Input sanitization, cryptography, logging

**Use Cases**:
- Building UI components
- Managing application state
- Handling user interactions
- Real-time collaboration

**Example**:
```typescript
import { useAuth } from '@/contexts/AuthContext';
import { invoke } from '@tauri-apps/api/tauri';

function MyComponent() {
  const { user, login, logout } = useAuth();

  const handleLogin = async () => {
    await login('ocean-forest-moon-star', 'password');
  };

  return <button onClick={handleLogin}>Login</button>;
}
```

### 2. [Tauri Commands](tauri-commands.md) (Rust → TypeScript)

**Purpose**: Desktop application integration via Tauri IPC

**Key Modules**:
- **Authentication** (40+ commands): Vault management, login/logout, passkeys, sessions
- **Organizations** (20+ commands): Channels, messages, threads, members
- **Projects** (15+ commands): Issues, comments, status updates
- **Members** (10+ commands): Member management, invitations
- **Documents** (8+ commands): CRDT sync, state vectors, diffs

**Use Cases**:
- Secure authentication and session management
- Real-time messaging and collaboration
- Project and issue tracking
- Member management and permissions

**Example**:
```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Initialize authentication
await invoke('auth_initialize');

// Create a vault
const vaultId = await invoke('auth_create_vault', {
  fourWords: 'ocean-forest-moon-star',
  password: 'secure-password',
  displayName: 'Alice'
});

// Login
const session = await invoke('auth_login', {
  fourWords: 'ocean-forest-moon-star',
  password: 'secure-password'
});
```

### 3. [Core Library API](core-api.md) (Rust)

**Purpose**: Low-level Rust library for identity, storage, and networking

**Key Crates**:
- **communitas-core**: Identity, groups, messaging, storage
- **saorsa-pqc**: Post-quantum cryptography (ML-DSA, ML-KEM)
- **ant-quic**: QUIC transport with NAT traversal
- **four-word-networking**: Four-word address encoding

**Use Cases**:
- Building custom Rust applications
- Extending Communitas functionality
- Headless/server deployments
- Custom protocol implementations

**Example**:
```rust
use communitas_core::{CoreContext, types::DeviceType};

// Initialize core context
let mut context = CoreContext::new(
    "ocean-forest-moon-star",
    "Alice",
    "MacBook Pro",
    DeviceType::Desktop,
).await?;

// Create a group
let group = context.create_group("Engineering Team").await?;

// Send a message
context.send_message(
    &group.id,
    "Hello, team!",
    None, // No thread
).await?;
```

### 4. [Bridge Server API](bridge-api.md) (HTTP/REST)

**Purpose**: Browser-based testing and integration via HTTP/REST

**Endpoints**:
- `GET /health` - Health check
- `POST /api/core/initialize` - Initialize with four-word identity
- `POST /api/channels` - Create channel
- `GET /api/channels` - List channels
- `POST /api/channels/:id/messages` - Send message
- `POST /api/threads/create` - Create thread

**Use Cases**:
- Browser-based testing with Chrome DevTools MCP
- Integration testing without Tauri
- API prototyping and development
- Cross-platform testing

**Example**:
```javascript
// Initialize core
await fetch('http://localhost:3030/api/core/initialize', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    four_words: 'ocean-forest-moon-star',
    display_name: 'Alice',
    device_name: 'Browser'
  })
});

// Create channel
const response = await fetch('http://localhost:3030/api/channels', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'General',
    description: 'General discussion'
  })
});
```

## Authentication & Security

### Authentication Flow

Communitas supports multiple authentication methods:

1. **Password Authentication** (Primary)
2. **Passkey/WebAuthn** (Biometric)
3. **Touch ID/Face ID** (macOS)
4. **Windows Hello** (Windows)

**Standard Login Flow**:
```typescript
// 1. Initialize auth service
await invoke('auth_initialize');

// 2. Check for existing vaults
const vaults = await invoke('auth_list_vaults');

// 3. Login with credentials
const session = await invoke('auth_login', {
  fourWords: 'ocean-forest-moon-star',
  password: 'secure-password'
});

// 4. Verify session
const isValid = await invoke('auth_check_session', {
  sessionId: session.session_id
});
```

**Biometric Login Flow** (macOS):
```typescript
// 1. Register Touch ID
await invoke('auth_touchid_register', {
  fourWords: 'ocean-forest-moon-star',
  deviceName: 'MacBook Pro',
  password: 'secure-password',
  reason: 'Register Touch ID for quick access'
});

// 2. Login with Touch ID
const session = await invoke('auth_touchid_authenticate', {
  fourWords: 'ocean-forest-moon-star',
  reason: 'Sign in to Communitas'
});
```

### Security Model

**Encryption**:
- **Vault Encryption**: ChaCha20-Poly1305 AEAD with PBKDF2 key derivation (100,000 iterations)
- **Post-Quantum**: ML-DSA-65 signatures, ML-KEM-768 key exchange
- **Transport**: QUIC with TLS 1.3

**Session Management**:
- **Session Expiration**: 24-hour default, configurable
- **Max Sessions**: 3 per device
- **Secure Storage**: Encrypted session data with memory zeroing

**Key Storage**:
- **macOS**: Keychain
- **Windows**: DPAPI
- **Linux**: Secret Service API

See [Security Architecture](../architecture/security.md) for complete details.

## Quick Start Examples

### Example 1: Complete Authentication Flow

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function authenticateUser() {
  // Initialize
  await invoke('auth_initialize');

  // List existing vaults
  const vaults = await invoke('auth_list_vaults');

  if (vaults.length === 0) {
    // Create new vault
    const vaultId = await invoke('auth_create_vault', {
      fourWords: 'ocean-forest-moon-star',
      password: 'secure-password',
      displayName: 'Alice'
    });
    console.log('Vault created:', vaultId);
  }

  // Login
  const session = await invoke('auth_login', {
    fourWords: 'ocean-forest-moon-star',
    password: 'secure-password'
  });

  console.log('Logged in:', session);
  return session;
}
```

### Example 2: Create Channel and Send Message

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function createChannelAndSendMessage() {
  // Create a channel
  const channel = await invoke('create_channel', {
    orgId: 'org-123',
    name: 'General',
    description: 'General discussion',
    createdBy: 'user-456'
  });

  // Send a message
  const message = await invoke('send_message', {
    channelId: channel.id,
    authorId: 'user-456',
    content: 'Hello, everyone!',
    threadId: null
  });

  // Create a thread from the message
  const thread = await invoke('create_thread', {
    parentMessageId: message.id
  });

  // Reply to the thread
  const reply = await invoke('send_message', {
    channelId: channel.id,
    authorId: 'user-789',
    content: 'Great to be here!',
    threadId: thread.id
  });

  return { channel, message, thread, reply };
}
```

### Example 3: Real-Time CRDT Synchronization

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function syncChannel(channelId: string) {
  // Get local state vector
  const localStateVector = await invoke('get_channel_state_vector', {
    channelId
  });

  // Send state vector to remote peer
  // (In real app, this would be sent via P2P network)
  const remoteStateVector = await sendToPeer(localStateVector);

  // Get diff based on remote state
  const diff = await invoke('get_channel_diff', {
    channelId,
    remoteStateVector
  });

  // Send diff to remote peer
  await sendDiffToPeer(diff);

  // Apply incoming diff from remote
  const incomingDiff = await receiveFromPeer();
  const result = await invoke('apply_channel_diff', {
    channelId,
    diff: incomingDiff
  });

  console.log('Sync complete:', result);
}
```

### Example 4: Project Management with Issues

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function manageProject() {
  // Create project
  const project = await invoke('create_project', {
    orgId: 'org-123',
    name: 'Q1 Launch',
    description: 'Q1 product launch tracking',
    icon: '🚀',
    color: '#4285f4',
    createdBy: 'user-456'
  });

  // Create issue
  const issue = await invoke('create_issue', {
    projectId: project.id,
    title: 'Implement user authentication',
    description: 'Add password and biometric authentication',
    priority: 'high',
    reporterId: 'user-456'
  });

  // Assign issue
  await invoke('assign_issue', {
    issueId: issue.id,
    assigneeId: 'user-789'
  });

  // Update status
  await invoke('update_issue_status', {
    issueId: issue.id,
    newStatus: 'in-progress'
  });

  // Add comment
  const comment = await invoke('add_issue_comment', {
    issueId: issue.id,
    authorId: 'user-789',
    content: 'Working on this now. Will have PR ready tomorrow.'
  });

  return { project, issue, comment };
}
```

### Example 5: Member Management

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function manageMember() {
  // Create member
  const member = await invoke('create_member', {
    fourWords: 'valley-river-cloud-wind',
    displayName: 'Bob',
    publicKey: 'base64_public_key',
    capabilities: {
      canInvite: true,
      canManageRoles: false
    }
  });

  // Get member
  const memberData = await invoke('get_member', {
    memberId: member.id
  });

  // Update member role
  await invoke('update_member_role', {
    memberId: member.id,
    newRole: 'admin'
  });

  // List all members
  const members = await invoke('list_members');

  return { member, memberData, members };
}
```

## Error Handling

All Tauri commands return `Result<T, String>` where errors are formatted as strings:

```typescript
try {
  const session = await invoke('auth_login', {
    fourWords: 'ocean-forest-moon-star',
    password: 'wrong-password'
  });
} catch (error) {
  // error is a string with description
  console.error('Login failed:', error);

  // Handle specific errors
  if (error.includes('Invalid password')) {
    // Show password error
  } else if (error.includes('not initialized')) {
    // Need to initialize first
    await invoke('auth_initialize');
  }
}
```

**Common Error Patterns**:

```typescript
// Service not initialized
"Auth service not initialized. Call auth_initialize first."

// Invalid credentials
"Login failed: Invalid password"

// Session expired
"Session not found or expired"

// Four-word validation
"Invalid four-word address: word not in dictionary"

// Network errors
"Failed to connect: connection timeout"

// Storage errors
"Failed to write to disk: permission denied"
```

## API Conventions

### Naming

- **Tauri Commands**: `snake_case` (Rust convention)
- **TypeScript APIs**: `camelCase` (JavaScript convention)
- **Rust Types**: `PascalCase` for structs, `snake_case` for fields

### Identifiers

- **Four-Word Addresses**: Hyphen-separated lowercase (e.g., `ocean-forest-moon-star`)
- **UUIDs**: Hexadecimal strings (e.g., `550e8400-e29b-41d4-a716-446655440000`)
- **Session IDs**: UUID format
- **Entity IDs**: UUID format

### Timestamps

- **Format**: Unix timestamp (seconds since epoch) as `u64`
- **Fields**: `created_at`, `updated_at`, `last_accessed`, `registered_at`

### Binary Data

- **Encoding**: Base64 for transport, raw bytes for storage
- **Fields**: `content_base64`, `public_key`, `signature_hex`

## Testing APIs

### Unit Testing (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_vault() {
        let state = AppState::new();
        // ... setup ...

        let result = auth_create_vault(
            State::new(&state),
            "test-vault".to_string(),
            "password".to_string(),
            "Test User".to_string(),
        ).await;

        assert!(result.is_ok());
    }
}
```

### Integration Testing (TypeScript)

```typescript
import { describe, it, expect, beforeAll } from 'vitest';
import { invoke } from '@tauri-apps/api/tauri';

describe('Authentication Flow', () => {
  beforeAll(async () => {
    await invoke('auth_initialize');
  });

  it('should create vault and login', async () => {
    const vaultId = await invoke('auth_create_vault', {
      fourWords: 'test-auth-flow',
      password: 'test-password',
      displayName: 'Test User'
    });

    expect(vaultId).toBeDefined();

    const session = await invoke('auth_login', {
      fourWords: 'test-auth-flow',
      password: 'test-password'
    });

    expect(session.four_words).toBe('test-auth-flow');
  });
});
```

### Bridge Server Testing (HTTP)

```javascript
describe('Bridge API', () => {
  const baseUrl = 'http://localhost:3030';

  it('should initialize and create channel', async () => {
    // Initialize
    const initResponse = await fetch(`${baseUrl}/api/core/initialize`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        four_words: 'test-bridge-api',
        display_name: 'Test User',
        device_name: 'Browser Test'
      })
    });

    expect(initResponse.ok).toBe(true);

    // Create channel
    const channelResponse = await fetch(`${baseUrl}/api/channels`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        name: 'Test Channel',
        description: 'Test channel from API'
      })
    });

    const channel = await channelResponse.json();
    expect(channel.name).toBe('Test Channel');
  });
});
```

## Performance Considerations

### Best Practices

1. **Batch Operations**: Group multiple Tauri commands when possible
2. **State Vectors**: Use CRDT state vectors for efficient synchronization
3. **Pagination**: Use `limit` and `offset` parameters for large result sets
4. **Caching**: Cache frequently accessed data in frontend
5. **Background Sync**: Use background tasks for non-critical updates

### Optimization Examples

**Bad** (Multiple round trips):
```typescript
for (const messageId of messageIds) {
  const message = await invoke('get_message', { messageId });
  messages.push(message);
}
```

**Good** (Batch query):
```typescript
const messages = await invoke('get_messages', {
  channelId,
  limit: 100,
  offset: 0
});
```

**Bad** (Full document sync):
```typescript
const fullUpdate = await invoke('get_channel_sync_update', { channelId });
await applyUpdate(fullUpdate);
```

**Good** (Diff-based sync):
```typescript
const stateVector = await invoke('get_channel_state_vector', { channelId });
const diff = await invoke('get_channel_diff', { channelId, remoteStateVector: stateVector });
await invoke('apply_channel_diff', { channelId, diff });
```

## Rate Limiting

Communitas implements rate limiting to prevent abuse:

- **Authentication**: 5 failed attempts per 15 minutes
- **API Calls**: 100 requests per minute per endpoint
- **Message Sending**: 10 messages per second per channel
- **File Upload**: 10 MB per minute per user

Rate limit headers (Bridge API):
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1699876543
```

## Versioning

Communitas follows semantic versioning (SemVer):

- **Major**: Breaking API changes
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes, backward compatible

Current version: **0.1.0** (Development)

API versioning:
- Tauri commands: Version matches desktop app version
- Core library: Independent versioning (communitas-core `0.3.17`)
- Bridge API: Version in path (`/api/v1/...`)

## Migration Guides

When upgrading between major versions, see:
- [Migration from v0.x to v1.x](../guides/migration-v1.md) (Coming soon)
- [Breaking Changes Log](../CHANGELOG.md)

## Additional Resources

### Documentation
- [Getting Started Guide](../guides/getting-started.md) - Setup and first steps
- [Authentication Guide](../guides/authentication.md) - Security and login
- [Four-Word Addresses](../guides/four-word-addresses.md) - Identity system
- [Testing Guide](../guides/testing.md) - Testing strategies

### Architecture
- [Core Components](../architecture/core-components.md) - System architecture
- [CRDT System](../architecture/crdt-system.md) - Real-time sync
- [Security](../architecture/security.md) - Cryptography and security
- [Storage](../architecture/storage.md) - Data persistence

### External Resources
- [Tauri v2 Documentation](https://v2.tauri.app/)
- [Yrs CRDT Documentation](https://docs.rs/yrs/)
- [ant-quic Transport](https://github.com/maidsafe/ant-quic)
- [four-word-networking](https://crates.io/crates/four-word-networking)

### Community
- [GitHub Repository](https://github.com/dirvine/communitas)
- [GitHub Discussions](https://github.com/dirvine/communitas/discussions)
- [Issue Tracker](https://github.com/dirvine/communitas/issues)

---

**Communitas API**: Local-first collaboration with post-quantum security. 🚀🔒
