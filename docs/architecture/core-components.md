# Core Components

Detailed technical overview of all Communitas system components and their interactions.

## Component Overview

Communitas is built with a modular architecture where each component has a specific responsibility:

```
┌─────────────────────────────────────────────────────────────┐
│                     FRONTEND LAYER                          │
│  React + TypeScript + Vite                                  │
│  - UI Components (Material-UI)                              │
│  - State Management (React Context)                         │
│  - Routing (React Router)                                   │
└─────────────────────────────────────────────────────────────┘
                           ↓ Tauri IPC
┌─────────────────────────────────────────────────────────────┐
│                  APPLICATION LAYER                          │
│  communitas-desktop (Tauri v2)                              │
│  - Tauri Commands                                           │
│  - Platform Integration                                     │
│  - Security & Keyring                                       │
└─────────────────────────────────────────────────────────────┘
                           ↓ Core API
┌─────────────────────────────────────────────────────────────┐
│                     CORE LAYER                              │
│  communitas-core                                            │
│  - AuthService                                              │
│  - GossipContext                                            │
│  - MemberManager                                            │
│  - CRDTManager                                              │
│  - EntityStorage                                            │
└─────────────────────────────────────────────────────────────┘
                           ↓ P2P Protocol
┌─────────────────────────────────────────────────────────────┐
│                   NETWORKING LAYER                          │
│  saorsa-gossip                                              │
│  - Membership (HyParView)                                   │
│  - PubSub (Plumtree)                                        │
│  - Presence (SWIM)                                          │
│  - Rendezvous (DHT-free discovery)                          │
└─────────────────────────────────────────────────────────────┘
```

## Frontend Components

### Technology Stack

**Core Framework**:
- React 18 with functional components and hooks
- TypeScript for type safety
- Vite for fast builds and HMR (Hot Module Replacement)

**UI Library**:
- Material-UI (MUI) v5 for component library
- Custom theme with light/dark mode support
- Responsive design for all screen sizes

**State Management**:
- React Context API for global state
- Custom hooks for local state
- No external state management library (Redux, Zustand)

### Directory Structure

```
src/
├── components/          # Reusable UI components
│   ├── auth/           # Authentication components
│   ├── chat/           # Messaging and channels
│   ├── common/         # Shared utilities
│   └── prototype/      # Experimental features
├── contexts/           # React Context providers
│   ├── AuthContext.tsx
│   ├── NavigationContext.tsx
│   └── ThemeContext.tsx
├── services/           # API service layer
│   ├── api/           # Backend communication
│   ├── storage/       # Offline storage
│   └── network/       # Network status
├── types/             # TypeScript type definitions
├── utils/             # Helper functions
└── App.tsx            # Root component
```

### Key Frontend Components

#### 1. Authentication System

**Location**: `src/components/auth/`

**Components**:
- `LoginDialog.tsx` - Login form with four-word address and password
- `PasskeyRegistration.tsx` - WebAuthn/passkey registration
- `UnifiedAuthFlow.tsx` - Combined registration/login UI

**Features**:
- Four-word address validation
- Password strength meter
- Passkey/WebAuthn support (when available)
- Session persistence with encrypted storage

**Example Usage**:
```typescript
import { AuthContext } from '@/contexts/AuthContext';

function MyComponent() {
  const { session, login, logout } = useContext(AuthContext);

  const handleLogin = async () => {
    await login({
      fourWords: 'ocean-forest-moon-star',
      password: 'secure-password'
    });
  };
}
```

#### 2. Navigation System

**Location**: `src/contexts/NavigationContext.tsx`

**Features**:
- Entity navigation (users, groups, channels)
- Breadcrumb trail management
- Back/forward navigation history
- Deep linking support

**Example Usage**:
```typescript
const {
  currentEntity,
  navigateToEntity,
  goBack
} = useNavigation();

// Navigate to a channel
await navigateToEntity({
  type: 'channel',
  id: 'channel-123',
  fourWords: 'bright-river-wind-star'
});
```

#### 3. Chat & Messaging

**Location**: `src/components/chat/`

**Components**:
- `ChannelView.tsx` - Channel message display
- `MessageComposer.tsx` - Message input with formatting
- `ThreadView.tsx` - Threaded conversations
- `MessageList.tsx` - Virtual scrolling message list

**Features**:
- Real-time message updates via CRDT sync
- Markdown support with preview
- File attachments and drag-and-drop
- Emoji picker and reactions
- Thread replies

#### 4. Offline Storage Service

**Location**: `src/services/storage/OfflineStorageService.ts`

**Features**:
- IndexedDB for offline data persistence
- Automatic sync when network returns
- Encryption for sensitive data
- TTL-based cache invalidation

**API**:
```typescript
class OfflineStorage {
  // Store data with optional encryption
  async store(key: string, data: any, options?: {
    ttl?: number,
    encrypt?: boolean,
    syncOnline?: boolean
  }): Promise<void>

  // Retrieve data (cache-first)
  async get(key: string): Promise<any>

  // Queue operation for sync
  async queueForSync(operation: SyncOperation): Promise<void>
}
```

#### 5. Network Connection Service

**Location**: `src/services/network/NetworkConnectionService.ts`

**Features**:
- Auto-connect on startup
- Retry logic with exponential backoff
- Graceful degradation to local mode
- Real-time status monitoring

**States**:
- `connecting` - Attempting to establish connection
- `connected` - Successfully connected to P2P network
- `local` - Operating in offline mode
- `offline` - Intentionally disconnected
- `error` - Connection error occurred

**API**:
```typescript
class NetworkConnectionService {
  // Connect to network
  async connect(): Promise<void>

  // Disconnect from network
  async disconnect(): Promise<void>

  // Get current state
  getState(): NetworkState

  // Subscribe to state changes
  subscribe(callback: (state: NetworkState) => void): () => void
}
```

### Frontend Build Configuration

**Vite Config** (`vite.config.ts`):
```typescript
export default defineConfig({
  plugins: [react()],
  server: {
    port: 1420,
    strictPort: true
  },
  build: {
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: true
  },
  resolve: {
    alias: {
      '@': '/src'
    }
  }
});
```

**TypeScript Config** (`tsconfig.json`):
```json
{
  "compilerOptions": {
    "target": "ESNext",
    "lib": ["ESNext", "DOM"],
    "jsx": "react-jsx",
    "strict": true,
    "noImplicitAny": true,
    "moduleResolution": "bundler"
  }
}
```

## Desktop Application (Tauri v2)

### Architecture

**Location**: `communitas-desktop/`

**Purpose**: Native desktop application wrapping the React frontend with Rust backend

**Key Features**:
- Cross-platform: Windows, macOS, Linux
- Native system integration (notifications, system tray)
- Secure credential storage (platform keyring)
- Auto-updater for seamless updates
- WebView with IPC bridge

### Tauri Command Structure

Commands are organized by domain in `communitas-desktop/src/`:

```
communitas-desktop/src/
├── core_cmds.rs          # Core application commands
├── core_groups.rs        # Group management
├── core_storage.rs       # Virtual disk operations
├── member_commands.rs    # Member operations
├── message_sync_commands.rs  # Message sync
├── crdt_manager.rs       # CRDT document management
├── entity_storage.rs     # Entity persistence
├── update_manager.rs     # Auto-updater
├── sync.rs              # Sync coordinator
└── security/            # Security and keyring
    ├── mod.rs
    └── raw_spki.rs
```

### Core Tauri Commands

#### Authentication Commands

```rust
// Register new user
#[tauri::command]
async fn register_user(
    four_words: String,
    display_name: String,
    device_name: String,
    password: String
) -> Result<SessionInfo, String>

// Login existing user
#[tauri::command]
async fn login_user(
    four_words: String,
    password: String
) -> Result<SessionInfo, String>

// Logout current session
#[tauri::command]
async fn logout() -> Result<(), String>
```

#### Identity Commands

```rust
// Get current user identity
#[tauri::command]
async fn get_current_identity() -> Result<Identity, String>

// Initialize four-word identity
#[tauri::command]
async fn core_initialize(
    four_words: String,
    display_name: String,
    device_name: String
) -> Result<(), String>
```

#### Channel Commands

```rust
// Create new channel
#[tauri::command]
async fn create_channel(
    name: String,
    description: String,
    members: Vec<String>
) -> Result<Channel, String>

// List all channels
#[tauri::command]
async fn list_channels() -> Result<Vec<Channel>, String>

// Send message to channel
#[tauri::command]
async fn send_message(
    channel_id: String,
    content: String
) -> Result<Message, String>
```

#### Group Commands

```rust
// Create group
#[tauri::command]
async fn create_group(
    name: String,
    members: Vec<String>
) -> Result<Group, String>

// Add member to group
#[tauri::command]
async fn add_group_member(
    group_id: String,
    four_words: String
) -> Result<(), String>
```

#### Virtual Disk Commands

```rust
// Write to virtual disk
#[tauri::command]
async fn disk_write(
    entity_id: String,
    disk_type: String,  // "Private" | "Public" | "Shared"
    path: String,
    content_base64: String
) -> Result<(), String>

// Read from virtual disk
#[tauri::command]
async fn disk_read(
    entity_id: String,
    disk_type: String,
    path: String
) -> Result<String, String>  // Returns base64

// List directory contents
#[tauri::command]
async fn disk_list_dir(
    entity_id: String,
    disk_type: String,
    path: String
) -> Result<Vec<FileInfo>, String>
```

### Platform Integration

#### macOS

**Keychain Integration**:
```rust
use security_framework::item::*;

pub fn store_password(
    service: &str,
    account: &str,
    password: &[u8]
) -> Result<()> {
    let query = SecGenericPassword::new(service, account);
    query.set_password(password)?;
    Ok(())
}
```

**Touch ID Authentication**:
```rust
use security_framework::os::macos::keychain::SecKeychain;

pub async fn authenticate_touch_id() -> Result<bool> {
    let context = LAContext::new()?;
    let result = context.evaluate_policy(
        LAPolicy::DeviceOwnerAuthenticationWithBiometrics,
        "Authenticate to access Communitas"
    ).await?;
    Ok(result)
}
```

#### Windows

**Credential Manager**:
```rust
use windows::Security::Credentials::*;

pub fn store_credential(
    resource: &str,
    username: &str,
    password: &str
) -> Result<()> {
    let vault = PasswordVault::new()?;
    let credential = PasswordCredential::CreatePasswordCredential(
        resource,
        username,
        password
    )?;
    vault.Add(&credential)?;
    Ok(())
}
```

**Windows Hello**:
```rust
use windows::Security::Credentials::UI::*;

pub async fn authenticate_windows_hello() -> Result<bool> {
    let result = UserConsentVerifier::RequestVerificationAsync(
        "Authenticate to access Communitas"
    )?.await?;

    Ok(result.status() == UserConsentVerificationResult::Verified)
}
```

#### Linux

**Secret Service (libsecret)**:
```rust
use libsecret::*;

pub async fn store_credential(
    four_words: &str,
    password: &str
) -> Result<()> {
    let schema = Schema::new(
        "com.saorsalabs.communitas",
        SchemaFlags::NONE,
        vec![(\"four_words\", SchemaAttributeType::String)]
    );

    password_store_sync(
        Some(&schema),
        vec![(\"four_words\", four_words)],
        Some(\"Communitas\"),
        password,
        None
    )?;

    Ok(())
}
```

### Auto-Updater

**Location**: `communitas-desktop/src/update_manager.rs`

**Features**:
- Automatic update checking
- Background download
- Signature verification with ML-DSA
- Silent installation on restart

**Configuration** (`tauri.conf.json`):
```json
{
  "updater": {
    "active": true,
    "endpoints": [
      "https://updates.communitas.life/{{target}}/{{current_version}}"
    ],
    "dialog": true,
    "pubkey": "ML-DSA public key here"
  }
}
```

## Core Library (communitas-core)

### Purpose

Shared Rust business logic used by all applications (desktop, headless, bridge, TUI).

**Location**: `communitas-core/`

### Module Structure

```
communitas-core/src/
├── lib.rs              # Public API exports
├── auth_service.rs     # Authentication
├── gossip/             # Gossip networking
│   └── context.rs
├── member_manager.rs   # Member operations
├── entity_storage.rs   # Entity persistence
├── encrypted_storage/  # Encrypted storage
│   ├── mod.rs
│   └── passkey.rs
└── error.rs           # Error types
```

### Key Services

#### 1. AuthService

**Purpose**: User authentication and session management

**Features**:
- Password-based authentication with Argon2
- Passkey/WebAuthn support
- Session token generation and validation
- Multi-device session management

**API**:
```rust
pub struct AuthService {
    storage: Arc<EntityStorage>,
    sessions: Arc<Mutex<HashMap<String, SessionInfo>>>
}

impl AuthService {
    // Register new user
    pub async fn register(
        &self,
        four_words: &str,
        display_name: &str,
        device_name: &str,
        password: &str
    ) -> Result<SessionInfo>

    // Login existing user
    pub async fn login(
        &self,
        four_words: &str,
        password: &str
    ) -> Result<SessionInfo>

    // Validate session token
    pub async fn validate_session(
        &self,
        token: &str
    ) -> Result<SessionInfo>

    // Logout (invalidate session)
    pub async fn logout(&self, token: &str) -> Result<()>
}
```

#### 2. GossipContext

**Purpose**: P2P networking coordinator

**Features**:
- Peer discovery and connection
- Message dissemination
- Presence tracking
- CRDT synchronization

**API**:
```rust
pub struct GossipContext {
    membership: Arc<Membership>,
    pubsub: Arc<PubSub>,
    presence: Arc<Presence>,
    rendezvous: Arc<Rendezvous>
}

impl GossipContext {
    // Initialize gossip network
    pub async fn new(config: GossipConfig) -> Result<Self>

    // Start networking
    pub async fn start(&self) -> Result<()>

    // Subscribe to topic
    pub async fn subscribe(
        &self,
        topic: &str,
        handler: Box<dyn MessageHandler>
    ) -> Result<()>

    // Publish message to topic
    pub async fn publish(
        &self,
        topic: &str,
        message: Vec<u8>
    ) -> Result<()>

    // Get list of connected peers
    pub async fn peers(&self) -> Vec<PeerInfo>
}
```

#### 3. MemberManager

**Purpose**: Entity membership management

**Features**:
- Add/remove members
- Role-based permissions
- Member presence tracking
- Membership CRDT synchronization

**API**:
```rust
pub struct MemberManager {
    storage: Arc<EntityStorage>,
    crdt: Arc<CRDTManager>
}

impl MemberManager {
    // Add member to entity
    pub async fn add_member(
        &self,
        entity_id: &str,
        four_words: &str,
        role: MemberRole
    ) -> Result<()>

    // Remove member from entity
    pub async fn remove_member(
        &self,
        entity_id: &str,
        four_words: &str
    ) -> Result<()>

    // List entity members
    pub async fn list_members(
        &self,
        entity_id: &str
    ) -> Result<Vec<Member>>

    // Check if member exists
    pub async fn is_member(
        &self,
        entity_id: &str,
        four_words: &str
    ) -> Result<bool>
}
```

#### 4. CRDTManager

**Purpose**: CRDT document management and synchronization

**Features**:
- Yrs document creation and updates
- State vector management
- Incremental sync
- SQL materialization for large documents

**API**:
```rust
pub struct CRDTManager {
    storage: Arc<EntityStorage>,
    docs: Arc<Mutex<HashMap<String, Doc>>>
}

impl CRDTManager {
    // Get or create CRDT document
    pub async fn get_doc(
        &self,
        doc_id: &str
    ) -> Result<Doc>

    // Apply update to document
    pub async fn apply_update(
        &self,
        doc_id: &str,
        update: Vec<u8>
    ) -> Result<()>

    // Get state vector for sync
    pub async fn get_state_vector(
        &self,
        doc_id: &str
    ) -> Result<Vec<u8>>

    // Generate diff from state vector
    pub async fn get_diff(
        &self,
        doc_id: &str,
        state_vector: Vec<u8>
    ) -> Result<Vec<u8>>
}
```

#### 5. EntityStorage

**Purpose**: Persistent storage for entities and CRDT documents

**Features**:
- libSQL database with embedded mode
- Entity metadata persistence
- CRDT state blob storage
- Indexing for fast queries

**Schema**:
```sql
-- Entities table
CREATE TABLE entities (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    four_words TEXT NOT NULL UNIQUE,
    name TEXT,
    created_at INTEGER,
    created_by TEXT
);

-- CRDT documents table
CREATE TABLE crdt_documents (
    doc_id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL,
    concern TEXT NOT NULL,
    state_blob BLOB NOT NULL,
    last_updated INTEGER,
    FOREIGN KEY (entity_id) REFERENCES entities(id)
);

-- Members table
CREATE TABLE members (
    entity_id TEXT NOT NULL,
    four_words TEXT NOT NULL,
    role TEXT,
    joined_at INTEGER,
    PRIMARY KEY (entity_id, four_words),
    FOREIGN KEY (entity_id) REFERENCES entities(id)
);
```

### Error Handling

**Location**: `communitas-core/src/error.rs`

**Error Types**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("CRDT error: {0}")]
    CRDTError(String),

    #[error("Invalid four-word address: {0}")]
    InvalidFourWords(String)
}
```

**Result Type**:
```rust
pub type Result<T> = std::result::Result<T, CoreError>;
```

## Testing & Development Tools

### MCP Server (communitas-mcp)

**Purpose**: Model Context Protocol server for AI agent control

**Location**: `communitas-mcp/`

**Use Case**: Enables AI agents (Claude, custom agents) to control Communitas

**Transport Options**:
- **stdio**: Default JSON-RPC 2.0 over stdin/stdout
- **HTTPS**: RFC 7250 Raw Public Keys with ML-DSA-65 (post-quantum)

**CLI Flags**:
```bash
--demo              # Auto-initialize with temporary identity
--http              # Enable HTTP transport
--tls               # Enable HTTPS with ML-DSA-65 raw public keys
--listen <addr>     # Listen address (default: 127.0.0.1:3040)
--no-client-auth    # Disable client certificate verification
```

See [communitas-mcp/README.md](../../communitas-mcp/README.md) for details.

### Terminal UI (communitas-tui)

**Purpose**: Terminal-based interface for development and testing

**Location**: `communitas-tui/`

**Features**:
- Full-featured TUI with keyboard navigation
- Real-time message updates
- Channel and group management
- Network status monitoring

### Headless Daemon (communitas-headless)

**Purpose**: Background service for servers and bots

**Location**: `communitas-headless/`

**Features**:
- systemd/launchd integration
- JSON-RPC API for remote control
- Webhook system for events
- Logging and monitoring

See [communitas-headless/README.md](../../communitas-headless/README.md) for details.

## Component Interactions

### Message Send Flow

```
User Types Message
       ↓
MessageComposer (React)
       ↓ invoke('send_message')
Tauri IPC Bridge
       ↓
send_message command (Rust)
       ↓
GossipContext.publish()
       ↓
PubSub (Plumtree)
       ↓
Connected Peers
       ↓
Peer's GossipContext
       ↓
Message Handler
       ↓
CRDTManager.apply_update()
       ↓
UI Update (React)
```

### Authentication Flow

```
User Enters Credentials
       ↓
LoginDialog (React)
       ↓ invoke('login_user')
Tauri IPC Bridge
       ↓
login_user command (Rust)
       ↓
AuthService.login()
       ↓
EntityStorage (verify password)
       ↓
Generate Session Token
       ↓
Store in Keyring
       ↓
Return SessionInfo
       ↓
AuthContext Update (React)
       ↓
Navigate to Main UI
```

### CRDT Sync Flow

```
User Edits Document
       ↓
CRDT Update (Yrs)
       ↓
CRDTManager.apply_update()
       ↓
EntityStorage (persist)
       ↓
GossipContext.publish()
       ↓
Peers Receive Update
       ↓
Peer's CRDTManager
       ↓
Merge with Local State
       ↓
UI Reflects Changes
```

## Performance Considerations

### Frontend Optimization

1. **Code Splitting**:
   - Route-based splitting
   - Lazy loading of heavy components
   - Dynamic imports for modals

2. **Virtual Scrolling**:
   - Message lists use react-window
   - Only render visible items
   - Efficient memory usage

3. **Debouncing & Throttling**:
   - Search input debounced (300ms)
   - Scroll events throttled
   - Resize listeners optimized

4. **Memoization**:
   - React.memo for expensive components
   - useMemo for computed values
   - useCallback for event handlers

### Backend Optimization

1. **Async Everywhere**:
   - Tokio async runtime
   - Non-blocking I/O
   - Concurrent task execution

2. **Connection Pooling**:
   - libSQL connection pool
   - QUIC connection reuse
   - Peer connection cache

3. **Efficient Serialization**:
   - Bincode for binary data
   - MessagePack for structured data
   - BLAKE3 for content addressing

## Security Best Practices

### Input Validation

All Tauri commands validate inputs:
```rust
#[tauri::command]
async fn send_message(
    channel_id: String,
    content: String
) -> Result<Message, String> {
    // Validate channel ID format
    if !is_valid_entity_id(&channel_id) {
        return Err("Invalid channel ID".to_string());
    }

    // Sanitize content
    let content = sanitize_html(&content);

    // Rate limiting
    if !check_rate_limit(&channel_id) {
        return Err("Rate limit exceeded".to_string());
    }

    // Proceed with sending
    // ...
}
```

### Secure Storage

- Passwords hashed with Argon2id
- Session tokens use secure random generation
- Sensitive data encrypted at rest
- Platform keyring for credential storage

### Network Security

- All P2P connections use QUIC with TLS 1.3
- Message encryption with ChaCha20-Poly1305
- Post-quantum signatures with ML-DSA
- No plaintext transmission

## See Also

- [Architecture Overview](README.md) - System architecture
- [CRDT System](crdt-system.md) - Collaborative editing
- [Gossip Protocol](gossip-protocol.md) - P2P networking
- [Security](security.md) - Security model
- [API Reference](../api/) - Complete API documentation

---

**Component architecture ensuring scalability, security, and maintainability.**
