# Communitas Application Specification

<!--
╔══════════════════════════════════════════════════════════════════════════════╗
║                        FOR AI/LLM AGENTS                                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  This is the CANONICAL specification for the Communitas application.         ║
║                                                                              ║
║  IMPORTANT INSTRUCTIONS:                                                     ║
║  1. If you make ANY changes to this spec, you MUST:                          ║
║     - Increment the version number in the header below                       ║
║     - Add an entry to the CHANGELOG section at the end                       ║
║     - Clearly describe what changed and why                                  ║
║                                                                              ║
║  2. Use this spec as the source of truth for implementing Communitas         ║
║     in ANY language (Swift, Kotlin, Flutter, React Native, etc.)             ║
║                                                                              ║
║  3. The Rust bindings (communitas-bindings) define the CORE API.             ║
║     UI implementations must wrap these bindings.                             ║
║                                                                              ║
║  4. Four-Word Identities are FUNDAMENTAL - all addresses use them.           ║
║                                                                              ║
║  Version: 1.0.0                                                              ║
║  Last Updated: 2024-11-27                                                    ║
║  Updated By: Claude (AI Assistant)                                           ║
╚══════════════════════════════════════════════════════════════════════════════╝
-->

## Table of Contents

1. [Overview](#1-overview)
2. [Core Concepts](#2-core-concepts)
3. [Architecture](#3-architecture)
4. [Data Models](#4-data-models)
5. [Screens & Navigation](#5-screens--navigation)
6. [User Flows](#6-user-flows)
7. [API Reference](#7-api-reference)
8. [Debug/Testing Interface](#8-debugtesting-interface)
9. [Security Model](#9-security-model)
10. [Platform Requirements](#10-platform-requirements)
11. [Changelog](#11-changelog)

---

## 1. Overview

### 1.1 Product Description

**Communitas** is a local-first, decentralized collaboration platform that combines:
- **WhatsApp** - Encrypted messaging (1:1 and group)
- **Dropbox** - Per-entity virtual disks with file sync
- **Zoom** - Voice/video calls with screen sharing
- **Slack** - Channels, threads, and organization structure

All functionality operates **offline-first** with **peer-to-peer synchronization** when network is available.

### 1.2 Key Differentiators

| Feature | Description |
|---------|-------------|
| **Four-Word Identities** | Human-readable addresses like `ocean-forest-moon-star` instead of UUIDs or emails |
| **Zero Central Servers** | All data stored locally and synced P2P via gossip protocol |
| **Per-Entity Storage** | Each org/group/channel/project has isolated Private/Public/Shared disks |
| **PQC-Ready** | Post-quantum cryptography (ML-DSA, ML-KEM) for future-proof security |
| **CRDT Documents** | Real-time collaborative editing with conflict-free sync |

### 1.3 Target Platforms

| Platform | Minimum Version | Framework |
|----------|-----------------|-----------|
| macOS | 14.0+ | SwiftUI + Tauri |
| iOS | 17.0+ | SwiftUI |
| Android | API 26+ | Kotlin/Jetpack Compose |
| Windows | 10+ | Tauri |
| Linux | Ubuntu 22.04+ | Tauri |
| Web | Modern browsers | React/TypeScript |

---

## 2. Core Concepts

### 2.1 Four-Word Identity System

Every user, address, and endpoint in Communitas uses **Four-Word Identities**:

```
ocean-forest-moon-star
```

**Properties:**
- Generated from the `four-word-networking` dictionary (~8000 words)
- Provides ~48 bits of entropy (sufficient for user identity)
- Human-verifiable and memorable
- Can encode IPv4/IPv6 addresses and ports
- Checksum validation prevents typos

**Generation:**
```rust
// Rust
let identity = generate_id_words(); // Returns "ocean-forest-moon-star"

// Swift
let identity = try generateIdWords() // Returns "ocean-forest-moon-star"
```

**Validation Rules:**
- Exactly 4 words separated by hyphens
- Each word must exist in the dictionary
- Words are case-insensitive (normalized to lowercase)

### 2.2 Entity Types

Communitas organizes content around **Entities**:

| Entity Type | Description | Has Disk | Has Chat |
|-------------|-------------|----------|----------|
| **Organisation** | Top-level container (company, team) | Yes | Yes |
| **Project** | Work container within an org | Yes | Yes |
| **Group** | Private group of users | Yes | Yes |
| **Channel** | Public/semi-public communication channel | No | Yes |
| **Person** | Individual user entity | Yes | No |

**Entity Hierarchy:**
```
Organisation
├── Project
│   ├── Channel (general)
│   └── Channel (dev)
├── Project
└── Group (private team)
```

### 2.3 Virtual Disk System

Each entity has three **Virtual Disks**:

| Disk Type | Access | Encryption | Use Case |
|-----------|--------|------------|----------|
| **Private** | Owner only | AES-256-GCM | Personal files, drafts |
| **Public** | World-readable | None (content-addressed) | Published content, websites |
| **Shared** | Entity members | Group key | Collaborative files |

**Disk Operations:**
- Write file (with BLAKE3 hash verification)
- Read file (binary or base64)
- List directory
- Create directory
- Delete file
- Get storage stats

### 2.4 Presence System

Real-time online status via **Presence Beacons**:

| Status | Description |
|--------|-------------|
| **Online** | User is active |
| **Away** | User is idle (auto-detected) |
| **Busy** | Do not disturb mode |
| **Offline** | User is not connected |

Beacons are broadcast every 30 seconds via gossip protocol.

### 2.5 CRDT Documents

Collaborative documents use **Yrs** (Rust port of Yjs) for conflict-free sync:

- **Insert text** at position
- **Delete text** range
- **Get full text**
- **Export CRDT update** (binary)
- **Apply CRDT update** from peer

Documents sync automatically when peers connect.

---

## 3. Architecture

### 3.1 System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         UI Layer                                 │
│  (SwiftUI / Kotlin / Flutter / React)                           │
├─────────────────────────────────────────────────────────────────┤
│                    UniFFI Bindings                               │
│  (communitas-bindings crate - generates Swift/Kotlin)           │
├─────────────────────────────────────────────────────────────────┤
│                      Core Library                                │
│  (communitas-core crate)                                        │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐       │
│  │   Auth   │ Entities │ Messaging│ Documents│  Disks   │       │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘       │
├─────────────────────────────────────────────────────────────────┤
│                   Gossip Network Layer                           │
│  (saorsa-gossip crate - P2P pubsub)                             │
├─────────────────────────────────────────────────────────────────┤
│                    Transport Layer                               │
│  (QUIC via ant-quic, UDP)                                       │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Component Responsibilities

#### 3.2.1 CommunitasClient (Main Interface)

The single entry point for all app functionality:

```swift
let client = try CommunitasClient(
    fourWords: "ocean-forest-moon-star",
    displayName: "Alice",
    deviceName: "MacBook Pro",
    storagePath: "~/.communitas"
)
```

**Sub-clients:**
- **Auth** - Vault management, login, passkeys
- **Entity** - Create/manage orgs, groups, channels
- **Message** - Send/receive messages
- **Document** - CRDT document operations
- **Gossip** - P2P networking, contacts
- **Presence** - Online status
- **Disk** - Virtual disk file operations
- **WebRTC** - Voice/video calls

#### 3.2.2 AppState (UI State Management)

Observable state container for reactive UI:

```swift
@MainActor
class AppState: ObservableObject {
    @Published var fourWords: String = ""
    @Published var displayName: String = ""
    @Published var isAuthenticated: Bool = false
    @Published var isNetworking: Bool = false
    @Published var entities: [EntityItem] = []
    @Published var contacts: [ContactItem] = []
    @Published var activeView: ActiveView = .home
    // ... more published properties
}
```

#### 3.2.3 Services

| Service | Purpose |
|---------|---------|
| **CallStateManager** | Manages active call state, participants, quality |
| **WebRTCEventBridge** | Bridges Rust WebRTC events to UI |
| **MediaDeviceService** | Camera/mic permissions and enumeration |
| **KeychainHelper** | Secure credential storage (passkeys) |

### 3.3 Data Flow

```
User Action → UI View → AppState → CommunitasClient → Core → Gossip → Network
                ↑                                              ↓
                └──────────────── Response ←───────────────────┘
```

**Offline Flow:**
```
User Action → UI View → AppState → Local Storage
                                        ↓
                              (When online) → Gossip Sync
```

---

## 4. Data Models

### 4.1 User & Authentication

#### SwiftUserProfile
```typescript
interface UserProfile {
    fourWords: string;      // "ocean-forest-moon-star"
    displayName: string;    // "Alice"
    deviceName: string;     // "MacBook Pro"
    deviceType: string;     // "Desktop"
}
```

#### SwiftSessionInfo
```typescript
interface SessionInfo {
    sessionId: string;      // UUID
    fourWords: string;      // User's four-word identity
    displayName: string;    // Display name
}
```

#### SwiftVaultInfo
```typescript
interface VaultInfo {
    fourWords: string;      // Identity
    displayName: string;    // Display name
    createdAt: number;      // Unix timestamp (ms)
    lastAccessed: number;   // Unix timestamp (ms)
    sizeBytes: number;      // Vault size
}
```

#### SwiftRecentIdentity
```typescript
interface RecentIdentity {
    fourWords: string;      // Identity
    displayName: string;    // Display name
    lastUsed: number;       // Unix timestamp (ms)
    hasPasskey: boolean;    // Biometric auth available
}
```

#### SwiftPasskeyInfo
```typescript
interface PasskeyInfo {
    credentialId: string;   // Credential ID
    deviceName: string;     // Device name
    createdAt: number;      // Unix timestamp (ms)
    lastUsed?: number;      // Last authentication (ms)
}
```

### 4.2 Entities

#### SwiftEntity
```typescript
interface Entity {
    id: string;                     // Hex entity ID
    name: string;                   // Display name
    entityType: EntityType;         // group | channel | project | organisation | person
    description?: string;           // Optional description
    createdBy: string;              // Creator's four-words
    createdAt: number;              // Unix timestamp (seconds)
    members: string[];              // Member four-word identities
    parentOrgId?: string;           // Parent org ID
}

enum EntityType {
    Group = "group",
    Channel = "channel",
    Project = "project",
    Organisation = "organisation",
    Person = "person"
}
```

#### SwiftMemberInfo
```typescript
interface MemberInfo {
    fourWords: string;          // Member's identity
    displayName?: string;       // Display name
    role: string;               // "admin" | "member" | "guest"
    joinedAt: number;           // Unix timestamp (seconds)
}
```

### 4.3 Messaging

#### SwiftMessage
```typescript
interface Message {
    id: string;                 // Message ID
    text: string;               // Message content
    author: string;             // Author's four-words
    createdAt: number;          // Unix timestamp (seconds)
    replyToId?: string;         // Parent message (for threads)
    entityId: string;           // Entity ID
    reactions: Reaction[];      // Emoji reactions
}

interface Reaction {
    emoji: string;              // Emoji character
    count: number;              // Reaction count
    users: string[];            // Four-word identities
}
```

#### SwiftSyncState
```typescript
interface SyncState {
    entityId: string;           // Entity being synced
    messageCount: number;       // Messages synced
    lastSyncTime: number;       // Unix timestamp (seconds)
    isSyncing: boolean;         // Currently syncing
}
```

### 4.4 Contacts & Presence

#### SwiftContactInfo
```typescript
interface ContactInfo {
    fourWords: string;              // Contact's identity
    displayName?: string;           // Display name
    isFavourite: boolean;           // Marked as favorite
    online: boolean;                // Currently online
    lastSeenEndpoint?: string;      // Four-word encoded IP:port
    endpointUpdatedAt?: number;     // Unix timestamp (ms)
    endpointSuccessCount: number;   // Successful connections
    endpointFailureCount: number;   // Failed connections
}
```

#### SwiftPresenceInfo
```typescript
interface PresenceInfo {
    fourWords: string;              // User's identity
    status: PresenceStatus;         // online | away | busy | offline
    lastSeen: number;               // Unix timestamp (seconds)
    deviceName?: string;            // Device name
}

enum PresenceStatus {
    Online = "online",
    Away = "away",
    Busy = "busy",
    Offline = "offline"
}
```

#### SwiftNetworkInfo
```typescript
interface NetworkInfo {
    isActive: boolean;              // Networking active
    connectionIdentity?: string;    // Four-word encoded address
    listenAddress?: string;         // "192.168.1.100:49152"
    port?: number;                  // UDP port
    fourWords: string;              // User's identity
    isLocalOnlyMode: boolean;       // No WAN connectivity
}
```

### 4.5 Documents

#### SwiftDocumentInfo
```typescript
interface DocumentInfo {
    id: string;                     // Document ID
    name: string;                   // Document name
    entityId: string;               // Associated entity
    createdAt: number;              // Unix timestamp (seconds)
    modifiedAt: number;             // Unix timestamp (seconds)
    storageMode: StorageMode;       // filesonly | webonly | both
}

enum StorageMode {
    FilesOnly = "filesonly",
    WebOnly = "webonly",
    Both = "both"
}
```

### 4.6 Virtual Disks

#### SwiftDiskFileInfo
```typescript
interface DiskFileInfo {
    path: string;                   // "/docs/readme.md"
    name: string;                   // "readme.md"
    isDirectory: boolean;           // true if directory
    sizeBytes: number;              // File size (0 for dirs)
    modifiedAt: number;             // Unix timestamp (seconds)
    contentHash: string;            // BLAKE3 hash
}

enum DiskType {
    Private = "private",
    Public = "public",
    Shared = "shared"
}
```

#### SwiftDiskStats
```typescript
interface DiskStats {
    entityId: string;               // Entity ID
    diskType: DiskType;             // private | public | shared
    usedBytes: number;              // Total bytes used
    fileCount: number;              // Total files
    dirCount: number;               // Total directories
    lastModified: number;           // Unix timestamp (seconds)
}
```

### 4.7 WebRTC/Calls

#### SwiftCallState
```typescript
interface CallState {
    callId: string;                 // UUID
    targetFourWords: string;        // Target peer
    isVideoEnabled: boolean;        // Video on/off
    isAudioEnabled: boolean;        // Audio on/off
    isScreenSharing: boolean;       // Screen share active
    state: string;                  // "initiating" | "ringing" | "connected" | "ended"
}

interface MediaConstraints {
    hasAudio: boolean;              // Enable audio
    hasVideo: boolean;              // Enable video
}

interface MediaDevice {
    deviceId: string;               // Device ID
    label: string;                  // Human name
    kind: string;                   // "audioinput" | "audiooutput" | "videoinput"
}
```

#### SwiftCallEvent
```typescript
type CallEvent =
    | { type: "IncomingCall"; callId: string; fromFourWords: string; hasVideo: boolean }
    | { type: "CallInitiated"; callId: string; toFourWords: string }
    | { type: "CallConnected"; callId: string }
    | { type: "CallRejected"; callId: string }
    | { type: "CallEnded"; callId: string }
    | { type: "RemoteVideoChanged"; callId: string; enabled: boolean }
    | { type: "RemoteAudioChanged"; callId: string; enabled: boolean }
    | { type: "RemoteScreenShareChanged"; callId: string; active: boolean };
```

### 4.8 Errors

```typescript
enum ClientError {
    InitError = "InitError",           // Core initialization failed
    AuthError = "AuthError",           // Authentication error
    NetworkError = "NetworkError",     // P2P/gossip error
    EntityError = "EntityError",       // Entity operation error
    MessageError = "MessageError",     // Messaging error
    StorageError = "StorageError",     // Disk operation error
    DocumentError = "DocumentError",   // CRDT document error
    PresenceError = "PresenceError",   // Presence beacon error
    IoError = "IoError",               // General I/O error
    NotFound = "NotFound",             // Resource not found
    WebRtcError = "WebRtcError"        // Voice/video error
}
```

---

## 5. Screens & Navigation

### 5.1 Navigation Architecture

```
RootView
├── AuthenticationView (unauthenticated)
│   ├── WelcomeView
│   ├── LoginView
│   ├── CreateIdentityView
│   ├── VaultSelectionView
│   └── VaultManagementView
│
└── ContentView (authenticated)
    ├── ProfileHeader
    ├── SidebarView
    └── DetailPane
        ├── WelcomePane
        ├── ContactChatView
        ├── ChatView (entity)
        ├── DriveView
        ├── DocumentListView
        └── CallView
```

### 5.2 Authentication Screens

#### 5.2.1 WelcomeView
**Purpose:** Entry point for new/returning users

**Elements:**
- App logo and branding
- "Sign In" button → LoginView
- "Create Identity" button → CreateIdentityView
- "Touch ID / Face ID" button (if passkey exists) → Auto-login

#### 5.2.2 LoginView
**Purpose:** Password-based authentication

**Elements:**
- Four-word identity input (4 text fields with autocomplete)
- Password field
- "Sign In" button
- "Forgot Password" link
- "Back" button

**Validation:**
- Four words must be valid dictionary words
- Password minimum 8 characters

#### 5.2.3 CreateIdentityView
**Purpose:** Create new four-word identity

**Elements:**
- "Generate Random Identity" button
- Four-word display (editable)
- Display name input
- Password input with strength indicator
- Confirm password input
- "Create" button
- Optional: "Enable Touch ID" checkbox

**Password Strength:**
- Weak: < 8 chars
- Fair: 8-11 chars
- Good: 12-15 chars, mixed case
- Strong: 16+ chars, mixed case + numbers + symbols

#### 5.2.4 VaultSelectionView
**Purpose:** Select from stored identities

**Elements:**
- List of stored vaults with:
  - Four-word identity
  - Display name
  - Last accessed date
  - Touch ID indicator (if enabled)
- "Add New Identity" button
- "Manage Vaults" button

#### 5.2.5 VaultManagementView
**Purpose:** Manage stored vaults

**Elements:**
- List of vaults with:
  - Four-word identity
  - Size (bytes)
  - "Enable/Disable Touch ID" toggle
  - "Delete Vault" button (with confirmation)
- "Back" button

### 5.3 Main Application Screens

#### 5.3.1 ContentView (Main Container)
**Purpose:** Primary app interface after authentication

**Layout:**
```
┌─────────────────────────────────────────────────────┐
│ ProfileHeader                                        │
├────────────────┬────────────────────────────────────┤
│                │                                     │
│   SidebarView  │         DetailPane                  │
│   (250px)      │         (flexible)                  │
│                │                                     │
│                │                                     │
└────────────────┴────────────────────────────────────┘
```

#### 5.3.2 ProfileHeader
**Purpose:** User info and global controls

**Elements:**
- User avatar (generated from four-words)
- Display name
- Four-word identity (copyable)
- Network status indicator:
  - Green dot: Connected
  - Yellow dot: Local only / Connecting
  - Red dot: Error
- Settings menu dropdown:
  - Network settings
  - Account settings
  - Logout

#### 5.3.3 SidebarView
**Purpose:** Navigation tree for entities and contacts

**Sections:**
1. **Favorites** (collapsible)
   - Favorite contacts with online indicator

2. **Organisations** (collapsible)
   - Each org with nested projects/channels

3. **Groups** (collapsible)
   - Private groups

4. **Contacts** (collapsible)
   - All contacts with online status

**Actions:**
- Click entity → Open in DetailPane
- Right-click → Context menu (Edit, Delete, Leave)
- "+" button → Create new entity
- Search bar at top

#### 5.3.4 ContactChatView
**Purpose:** 1:1 direct messaging

**Elements:**
- Contact header:
  - Avatar
  - Display name
  - Online status
  - Call buttons (audio/video)
- Message list (scrollable, newest at bottom)
- Message composer:
  - Text input
  - Attachment button
  - Send button

**Message Bubble:**
- Author avatar (for received)
- Message text
- Timestamp
- Reactions (emoji row)
- Reply indicator (if thread)

#### 5.3.5 ChatView (Entity)
**Purpose:** Group/channel messaging

**Elements:**
- Entity header:
  - Entity icon (based on type)
  - Entity name
  - Member count
  - Call button (for groups)
- Tabs: Chat | Files | Documents
- Message list
- Message composer with @mentions

#### 5.3.6 DriveView
**Purpose:** Virtual disk file browser

**Elements:**
- Breadcrumb navigation: `/docs/subdir`
- Toolbar:
  - Upload button
  - New folder button
  - View toggle (list/grid)
  - Search
- File list with columns:
  - Icon (folder/file type)
  - Name
  - Size
  - Modified date
  - Actions (download, delete)
- Disk type selector: Private | Public | Shared
- Storage usage bar

#### 5.3.7 DocumentListView
**Purpose:** CRDT collaborative documents

**Elements:**
- Document list:
  - Document name
  - Last modified
  - Author
- "New Document" button
- Click → DocumentEditorView

#### 5.3.8 DocumentEditorView
**Purpose:** Real-time collaborative text editing

**Elements:**
- Document title (editable)
- Rich text editor with:
  - Bold, italic, underline
  - Headers (H1-H3)
  - Lists (bullet, numbered)
  - Code blocks
- Collaborator cursors (when online)
- Sync status indicator
- Last saved timestamp

#### 5.3.9 CallView
**Purpose:** Voice/video call interface

**Elements:**
- Video area:
  - Remote video (large)
  - Local video (small, corner)
  - Screen share (when active)
- Participant list (for group calls)
- Control bar:
  - Mute/unmute button
  - Video on/off button
  - Screen share button
  - End call button
- Call quality indicator
- Call duration

#### 5.3.10 IncomingCallView
**Purpose:** Incoming call notification

**Elements:**
- Caller avatar
- Caller name and four-words
- "Voice Call" or "Video Call" label
- "Accept" button (green)
- "Decline" button (red)
- Ringtone (audio)

### 5.4 Modal Dialogs

#### CreateEntityDialog
**Purpose:** Create new entity

**Fields:**
- Entity type selector (Organisation, Project, Group, Channel)
- Name input
- Description input (optional)
- Parent organisation selector (for Project/Channel)
- "Create" button

#### EntitySettingsDialog
**Purpose:** Edit entity settings

**Tabs:**
- General: Name, description
- Members: Add/remove members, set roles
- Permissions: Access controls
- Danger Zone: Delete entity

#### NetworkSettingsDialog
**Purpose:** Configure P2P networking

**Fields:**
- Bootstrap node address
- Listen port (0 = auto)
- "Start/Stop Networking" toggle
- Connection status display
- Peer list

---

## 6. User Flows

### 6.1 First-Time User Flow

```
1. Launch app
2. WelcomeView displayed
3. User taps "Create Identity"
4. CreateIdentityView:
   a. Tap "Generate" → Random four-words generated
   b. Enter display name
   c. Enter password (twice)
   d. Optionally enable Touch ID
   e. Tap "Create"
5. Vault created, user logged in
6. ContentView displayed with WelcomePane
7. Prompt to:
   - Add contacts
   - Create first organisation
   - Start networking
```

### 6.2 Returning User Flow (Password)

```
1. Launch app
2. VaultSelectionView displayed (if multiple vaults)
   - Or LoginView directly (if single vault)
3. Select vault / enter four-words
4. Enter password
5. Tap "Sign In"
6. ContentView displayed
7. Auto-connect to network (if enabled)
```

### 6.3 Returning User Flow (Biometric)

```
1. Launch app
2. VaultSelectionView displayed
3. Vault with Touch ID shows biometric icon
4. Tap vault
5. System biometric prompt appears
6. Authenticate with Touch ID / Face ID
7. ContentView displayed
```

### 6.4 Send Direct Message Flow

```
1. From SidebarView, select contact
2. ContactChatView opens
3. Type message in composer
4. Tap send
5. Message appears in list (optimistic)
6. Message synced via gossip (when online)
7. Recipient sees message
```

### 6.5 Create Organisation Flow

```
1. In SidebarView, tap "+"
2. CreateEntityDialog opens
3. Select "Organisation"
4. Enter name and description
5. Tap "Create"
6. Organisation appears in sidebar
7. Virtual disks created (Private, Public, Shared)
```

### 6.6 Add Contact Flow

```
1. From Contacts section, tap "Add Contact"
2. Enter contact's four-word identity
3. Optionally enter display name
4. Tap "Add"
5. Contact added to list
6. When both online, connection established
7. Direct messaging enabled
```

### 6.7 Upload File Flow

```
1. Navigate to entity's DriveView
2. Select disk type (Private/Public/Shared)
3. Tap "Upload"
4. File picker opens
5. Select file(s)
6. Upload progress shown
7. File appears in list with hash
8. File synced to peers (for Shared disk)
```

### 6.8 Start Call Flow

```
1. Open ContactChatView or ChatView
2. Tap audio or video call button
3. CallView opens
4. Call state: "Initiating..."
5. Peer receives IncomingCallView
6. Peer accepts
7. Call state: "Connected"
8. Media streams established
9. Either party can end call
```

### 6.9 Collaborative Document Flow

```
1. Navigate to entity's Documents tab
2. Tap "New Document"
3. Enter document name
4. DocumentEditorView opens
5. Type content
6. Edits saved locally via CRDT
7. When peers online, updates synced
8. Multiple users can edit simultaneously
9. Cursors show collaborator positions
```

---

## 7. API Reference

### 7.1 Namespace Functions

```typescript
// Generate random four-word identity
function generateIdWords(): string  // throws ClientError
```

### 7.2 CommunitasClient Constructor

```typescript
constructor(
    fourWords: string,      // User's four-word identity
    displayName: string,    // Display name
    deviceName: string,     // Device name
    storagePath: string     // Local storage path
): CommunitasClient  // throws ClientError
```

### 7.3 Core Methods

```typescript
// Get current user profile
getProfile(): UserProfile

// Check if networking is active
isNetworkingActive(): boolean

// Get connection identity (four-word encoded address)
getConnectionIdentity(): string | null
```

### 7.4 Auth Sub-Client (15 methods)

```typescript
// Create encrypted vault
authCreateVault(fourWords: string, password: string, displayName: string): string

// Login with password
authLogin(fourWords: string, password: string): SessionInfo

// Logout current session
authLogout(): void

// Get current session (nullable, non-throwing)
authGetCurrentSession(): SessionInfo | null

// List all vaults
authListVaults(): VaultInfo[]

// Check if vault exists
authVaultExists(fourWords: string): boolean

// Delete vault (requires password)
authDeleteVault(fourWords: string, password: string): void

// Register biometric authentication
authRegisterPasskey(fourWords: string, deviceName: string): PasskeyInfo

// Authenticate with biometric
authAuthenticateWithPasskey(fourWords: string): SessionInfo

// Check if identity has passkey
authHasPasskey(fourWords: string): boolean

// Remove passkey
authDeletePasskey(fourWords: string): void

// Get recently used identities
authGetRecentIdentities(): RecentIdentity[]

// Attempt auto-login with last identity
authTryAutoLogin(): SessionInfo | null
```

### 7.5 Entity Sub-Client (10 methods)

```typescript
// Create entity
entityCreate(
    name: string,
    entityType: EntityType,
    description: string | null,
    parentOrgId: string | null
): Entity

// Get entity by ID
entityGet(entityId: string): Entity

// List all entities
entityList(): Entity[]

// List entities by type
entityListByType(entityType: EntityType): Entity[]

// Add member to entity
entityAddMember(entityId: string, memberFourWords: string, role: string): void

// Remove member
entityRemoveMember(entityId: string, memberFourWords: string): void

// List members
entityListMembers(entityId: string): MemberInfo[]

// Set parent organisation
entitySetParentOrg(entityId: string, orgId: string): void
```

### 7.6 Messaging Sub-Client (7 methods)

```typescript
// Send message to entity
messageSend(entityId: string, text: string, replyToId: string | null): string

// Send direct message
messageSendDirect(recipientFourWords: string, text: string): string

// Get entity messages (paginated)
messageGetForEntity(entityId: string, limit: number | null, beforeId: string | null): Message[]

// Get thread replies
messageGetThread(entityId: string, parentMessageId: string): Message[]

// Get direct messages with peer
messageGetDirect(peerFourWords: string): Message[]

// Get sync state
messageGetSyncState(entityId: string): SyncState
```

### 7.7 Document Sub-Client (9 methods)

```typescript
// Create CRDT document
documentCreate(entityId: string, name: string, storageMode: StorageMode): string

// Get document info
documentGetInfo(docId: string): DocumentInfo

// List documents
documentList(entityId: string | null): DocumentInfo[]

// Delete document
documentDelete(docId: string): void

// Insert text at position
documentInsertText(docId: string, position: number, text: string): void

// Delete text range
documentDeleteText(docId: string, position: number, length: number): void

// Get full text
documentGetText(docId: string): string

// Export CRDT update (binary)
documentGetCrdtUpdate(docId: string): Uint8Array

// Apply CRDT update from peer
documentApplyCrdtUpdate(docId: string, update: Uint8Array): void
```

### 7.8 Gossip/Network Sub-Client (18 methods)

```typescript
// Start P2P networking
gossipStart(port: number | null): string

// Stop networking
gossipStop(): void

// Connect to peer by identity
gossipConnectToPeer(fourWords: string): void

// Find contact
gossipFindContact(fourWords: string): string

// Add contact
gossipAddContact(fourWords: string): void

// Get all contacts
gossipGetContacts(): ContactInfo[]

// Remove contact
gossipRemoveContact(fourWords: string): void

// Add to favorites
gossipAddFavouriteContact(fourWords: string): void

// Get favorites
gossipGetFavouriteContacts(): string[]

// Get cached endpoint (non-throwing)
gossipContactGetEndpoint(fourWords: string): string | null

// Update endpoint
gossipContactUpdateEndpoint(fourWords: string, endpoint: string): void

// Record successful connection
gossipContactRecordSuccess(fourWords: string, endpoint: string): void

// Record failed connection
gossipContactRecordFailure(fourWords: string): void

// Join entity topic
gossipJoinEntity(entityId: string, entityType: string): void

// Leave entity topic
gossipLeaveEntity(entityId: string): void

// Publish to entity
gossipPublishToEntity(entityId: string, message: Uint8Array): void

// Send P2P message
gossipSendP2pMessage(peerFourWords: string, message: Uint8Array): void

// Dial specific address
gossipDialAddress(address: string): void

// Get bound port (non-throwing)
gossipGetBoundPort(): number | null

// Get network info (non-throwing)
gossipGetNetworkInfo(): NetworkInfo
```

### 7.9 Presence Sub-Client (5 methods)

```typescript
// Start presence beacons
presenceStartBeacons(): void

// Stop beacons
presenceStopBeacons(): void

// Get user status
presenceGetStatus(fourWords: string): PresenceStatus

// Check if online
presenceIsOnline(fourWords: string): boolean

// Get online users in entity
presenceGetOnlineInEntity(entityId: string): PresenceInfo[]
```

### 7.10 Disk Sub-Client (9 methods)

```typescript
// Write file
diskWriteFile(entityId: string, diskType: DiskType, path: string, data: Uint8Array): DiskFileInfo

// Read file
diskReadFile(entityId: string, diskType: DiskType, path: string): Uint8Array

// List directory
diskListFiles(entityId: string, diskType: DiskType, path: string): DiskFileInfo[]

// Delete file
diskDeleteFile(entityId: string, diskType: DiskType, path: string): void

// Get storage stats
diskGetStats(entityId: string, diskType: DiskType): DiskStats

// Create directory
diskCreateDirectory(entityId: string, diskType: DiskType, path: string): DiskFileInfo

// Check file exists (non-throwing)
diskFileExists(entityId: string, diskType: DiskType, path: string): boolean

// Get file info without reading
diskGetFileInfo(entityId: string, diskType: DiskType, path: string): DiskFileInfo
```

### 7.11 WebRTC Sub-Client (11 methods)

```typescript
// Check WebRTC availability (non-throwing)
webrtcIsAvailable(): boolean

// Initiate call
webrtcInitiateCall(targetFourWords: string, constraints: MediaConstraints): string

// Accept incoming call
webrtcAcceptCall(callId: string, constraints: MediaConstraints): void

// Reject call
webrtcRejectCall(callId: string): void

// End call
webrtcEndCall(callId: string): void

// Toggle video
webrtcSetVideoEnabled(callId: string, enabled: boolean): void

// Toggle audio
webrtcSetAudioEnabled(callId: string, enabled: boolean): void

// Start screen share
webrtcStartScreenShare(callId: string): void

// Stop screen share
webrtcStopScreenShare(callId: string): void

// Get media devices
webrtcGetMediaDevices(): MediaDevice[]
```

---

## 8. Debug/Testing Interface

### 8.1 Debug Server

**Available in DEBUG builds only**

- **Default Port:** 9999 (configurable via `DEBUG_PORT` env var)
- **Protocol:** HTTP/REST with JSON
- **CORS:** Enabled (Access-Control-Allow-Origin: *)

### 8.2 Health Check

```http
GET /health
```

**Response:**
```json
{
    "status": "ok",
    "handlers": ["state", "contacts", "sendMessage", ...],
    "port": 9999
}
```

### 8.3 Debug Handlers (55 total)

#### State & Query
| Handler | Method | Purpose |
|---------|--------|---------|
| `state` | GET/POST | Get full app state |
| `contacts` | GET/POST | List all contacts |
| `entities` | GET/POST | List all entities |
| `messages` | POST | Get direct messages |
| `contactEndpoint` | POST | Get contact endpoint info |

#### Initialization
| Handler | Method | Purpose |
|---------|--------|---------|
| `setIdentity` | POST | Set four-words and display name |
| `initialize` | POST | Initialize core client |
| `debugLogin` | POST | Skip passkey auth (DEBUG only) |

#### Networking
| Handler | Method | Purpose |
|---------|--------|---------|
| `startNetworking` | POST | Start P2P networking |
| `stopNetworking` | POST | Stop networking |
| `setBootstrap` | POST | Set bootstrap node |
| `connectToPeer` | POST | Connect by four-words |
| `dialAddress` | POST | Dial specific IP:port |
| `networkInfo` | GET/POST | Get network state |

#### Contacts
| Handler | Method | Purpose |
|---------|--------|---------|
| `addContact` | POST | Add contact |
| `removeContact` | POST | Remove contact |
| `updateContactEndpoint` | POST | Update endpoint |
| `recordConnectionSuccess` | POST | Record success |
| `recordConnectionFailure` | POST | Record failure |

#### Messaging
| Handler | Method | Purpose |
|---------|--------|---------|
| `sendMessage` | POST | Send direct message |
| `loadMessages` | POST | Load message history |
| `sendEntityMessage` | POST | Send to entity |
| `getEntityMessages` | POST | Get entity messages |

#### Entities
| Handler | Method | Purpose |
|---------|--------|---------|
| `createEntity` | POST | Create entity |
| `addEntityMember` | POST | Add member |
| `removeEntityMember` | POST | Remove member |
| `listEntityMembers` | POST | List members |

#### Documents
| Handler | Method | Purpose |
|---------|--------|---------|
| `documentCreate` | POST | Create document |
| `documentList` | POST | List documents |
| `documentGetInfo` | POST | Get document info |
| `documentGetContent` | POST | Get document text |
| `documentEdit` | POST | Insert/delete text |
| `documentDelete` | POST | Delete document |

#### Virtual Disks
| Handler | Method | Purpose |
|---------|--------|---------|
| `diskWriteFile` | POST | Write file |
| `diskReadFile` | POST | Read file |
| `diskListFiles` | POST | List directory |
| `diskDeleteFile` | POST | Delete file |
| `diskGetStats` | POST | Get storage stats |
| `diskCreateDirectory` | POST | Create directory |
| `diskFileExists` | POST | Check existence |
| `diskGetFileInfo` | POST | Get file metadata |

#### Calls/WebRTC
| Handler | Method | Purpose |
|---------|--------|---------|
| `callState` | GET/POST | Get call state |
| `callInitiate` | POST | Start 1:1 call |
| `callInitiateEntity` | POST | Start group call |
| `callIncoming` | POST | Simulate incoming call |
| `callAccept` | POST | Accept call |
| `callReject` | POST | Reject call |
| `callEnd` | POST | End call |
| `callSetVideo` | POST | Toggle video |
| `callSetAudio` | POST | Toggle audio |
| `callWebRTCEvent` | POST | Simulate WebRTC event |

#### Utilities
| Handler | Method | Purpose |
|---------|--------|---------|
| `generateIdentity` | GET/POST | Generate random four-words |
| `navigate` | POST | Navigate to view |
| `pollContacts` | GET/POST | Poll contact requests |
| `clearError` | GET/POST | Clear error state |

### 8.4 Debug Handler Examples

**Generate Identity:**
```bash
curl -X POST http://localhost:9999/debug/generateIdentity
# Response: {"success":true,"fourWords":"ocean-forest-moon-star"}
```

**Send Message:**
```bash
curl -X POST http://localhost:9999/debug/sendMessage \
  -H "Content-Type: application/json" \
  -d '{"to":"ocean-forest-moon-star","text":"Hello!"}'
```

**Create Entity:**
```bash
curl -X POST http://localhost:9999/debug/createEntity \
  -H "Content-Type: application/json" \
  -d '{"name":"My Team","entityType":"group","description":"Test group"}'
```

**Write File:**
```bash
curl -X POST http://localhost:9999/debug/diskWriteFile \
  -H "Content-Type: application/json" \
  -d '{
    "entityId":"abc123",
    "diskType":"private",
    "path":"/readme.md",
    "contentBase64":"SGVsbG8gV29ybGQh"
  }'
```

---

## 9. Security Model

### 9.1 Cryptographic Primitives

| Component | Algorithm | Purpose |
|-----------|-----------|---------|
| Identity Signing | ML-DSA (Dilithium) | PQC digital signatures |
| Key Exchange | ML-KEM (Kyber) | PQC key encapsulation |
| Symmetric Encryption | ChaCha20-Poly1305 | Authenticated encryption |
| Hashing | BLAKE3 | Fast cryptographic hashing |
| Password Derivation | Argon2id | Memory-hard KDF |

### 9.2 Vault Security

- Vaults encrypted with AES-256-GCM
- Master key derived from password via Argon2id
- Optional biometric unlock via platform keychain
- Each device has unique vault encryption

### 9.3 Network Security

- All P2P traffic encrypted via QUIC (TLS 1.3)
- Perfect forward secrecy via ephemeral keys
- Peer identity verified via four-word signature
- No central servers = no MITM opportunity

### 9.4 Storage Security

| Disk Type | At-Rest Encryption | In-Transit Encryption |
|-----------|-------------------|----------------------|
| Private | AES-256-GCM (user key) | QUIC/TLS |
| Public | None (content-addressed) | QUIC/TLS |
| Shared | AES-256-GCM (group key) | QUIC/TLS |

### 9.5 Code Quality Requirements

**Production Code (Rust):**
- No `unwrap()` or `expect()`
- No `panic!()`
- All errors return `Result<T, ClientError>`
- Zero clippy warnings

**Test Code (Rust):**
- `unwrap()` and `expect()` permitted for test clarity

---

## 10. Platform Requirements

### 10.1 macOS

| Requirement | Value |
|-------------|-------|
| Minimum Version | 14.0 (Sonoma) |
| Architecture | arm64 (Apple Silicon) |
| Framework | SwiftUI |
| Permissions | Camera, Microphone, Screen Recording |

### 10.2 iOS

| Requirement | Value |
|-------------|-------|
| Minimum Version | 17.0 |
| Architecture | arm64 |
| Framework | SwiftUI |
| Permissions | Camera, Microphone |
| Background Modes | Audio, Remote notifications |

### 10.3 Android

| Requirement | Value |
|-------------|-------|
| Minimum SDK | 26 (Android 8.0) |
| Target SDK | 34 |
| Framework | Jetpack Compose |
| Permissions | CAMERA, RECORD_AUDIO, INTERNET |

### 10.4 Dependencies

**Rust Core:**
- `communitas-core` - Core business logic
- `communitas-bindings` - UniFFI bridge
- `saorsa-gossip` - P2P networking
- `four-word-networking` - Identity system
- `yrs` - CRDT documents

**Swift:**
- SwiftUI (system)
- Network.framework (system)
- AVFoundation (system)
- ScreenCaptureKit (system)

---

## 11. Changelog

### Version 1.0.0 (2024-11-27)
- Initial specification document created
- Documented all 98 API methods across 8 sub-clients
- Documented all 55 debug handlers
- Documented all 15 screens and navigation flows
- Documented all 25+ data models
- Documented security model with PQC support
- Added platform requirements for macOS, iOS, Android

---

<!--
╔══════════════════════════════════════════════════════════════════════════════╗
║                           END OF SPECIFICATION                               ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Remember: If you modify this spec, update the version number and changelog! ║
╚══════════════════════════════════════════════════════════════════════════════╝
-->
