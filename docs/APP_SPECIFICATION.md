# Communitas Application Specification

**Version**: 1.0
**Status**: Definitive
**Last Updated**: 2025-12-31

This document is the **definitive source of truth** for all Communitas GUI implementations (Flutter, Swift).

---

## Table of Contents

1. [Product Overview](#1-product-overview)
2. [Entity Model](#2-entity-model)
3. [Authentication & Identity](#3-authentication--identity)
4. [Access Control (RBAC)](#4-access-control-rbac)
5. [Screen Inventory](#5-screen-inventory)
6. [Messaging System](#6-messaging-system)
7. [Virtual Disk System](#7-virtual-disk-system)
8. [Kanban System](#8-kanban-system)
9. [Voice/Video Calls](#9-voicevideo-calls)
10. [Network & P2P](#10-network--p2p)
11. [User Stories](#11-user-stories)
12. [Demo Mode](#12-demo-mode)
13. [Error States](#13-error-states)
14. [Theme & Design](#14-theme--design)

---

## 1. Product Overview

### 1.1 Vision

Communitas is a **local-first, PQC-ready collaboration platform** that merges the functionality of WhatsApp, Dropbox, Zoom, and Slack into one decentralized application.

### 1.2 Core Principles

- **Local-first**: All operations work offline; sync when network available
- **Decentralized**: No central server; peer-to-peer gossip protocol
- **Privacy-first**: End-to-end encryption with post-quantum cryptography
- **Human-verifiable**: Four-word identities for anti-phishing

### 1.3 Supported Platforms

| Platform | GUI Framework | Status |
|----------|---------------|--------|
| macOS | Swift/SwiftUI | Active |
| macOS | Flutter | Planned |
| iOS | Flutter | Planned |
| Android | Flutter | Planned |
| Linux | Flutter | Planned |
| Windows | Flutter | Planned |
| Web | Flutter (demo mode) | Planned |

---

## 2. Entity Model

### 2.1 Entity Types

| Type | Icon | Description | Can Have Children |
|------|------|-------------|-------------------|
| **Organisation** | Building | Top-level container for teams/companies | Yes (Projects, Channels, Groups) |
| **Project** | Folder | Work container with Kanban board | Yes (Channels) |
| **Channel** | Hash (#) | Topic-based messaging space | No |
| **Group** | Users | Team of people | No |
| **Person** | User | Individual contact | No |

### 2.2 Entity Hierarchy

```
Organisation
├── Project
│   ├── Channel (project-scoped)
│   └── [Kanban Board - automatic]
├── Channel (org-scoped)
└── Group
```

### 2.3 Entity Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | UUID | Yes | Unique identifier |
| `fourWords` | String? | No | Network identity (null = local-only) |
| `name` | String | Yes | Display name (1-100 chars) |
| `description` | String? | No | Optional description |
| `entityType` | Enum | Yes | Organisation/Project/Channel/Group |
| `parentOrgId` | UUID? | No | Parent organization (if nested) |
| `createdBy` | String | Yes | Creator's four-word identity |
| `createdAt` | Timestamp | Yes | Creation time |
| `memberCount` | Int | Yes | Number of members |
| `isLocalOnly` | Bool | Yes | Not synced to network |
| `isPersonal` | Bool | Yes | Personal space (not org-affiliated) |

### 2.4 Entity Colors

| Entity Type | Color | Hex |
|-------------|-------|-----|
| Organisation | Teal | `#3E8E7E` |
| Project | Gold | `#DAA520` |
| Channel | Jade | `#4CAF83` |
| Group | Purple | `#9B59B6` |
| Person | Coral | `#FF7F7F` |

---

## 3. Authentication & Identity

### 3.1 Four-Word Identity System

**Format**: `word-word-word-word` (e.g., `ocean-forest-moon-star`)

| Property | Value |
|----------|-------|
| Word count | 4 |
| Entropy | 48+ bits |
| Dictionary | four-word-networking crate |
| Validation | Checksum built-in |
| Case | Lowercase normalized |

### 3.2 Vault Structure

Each identity has a local vault containing:

| Component | Encryption | Location |
|-----------|------------|----------|
| Private key (ML-DSA) | AES-256-GCM | Platform keychain |
| Session data | AES-256-GCM | Encrypted file |
| Preferences | None | Local file |
| Backup key | Password-derived | Exportable file |

### 3.3 Authentication Methods

| Method | Platforms | Flow |
|--------|-----------|------|
| Password | All | Enter 4-word identity + password (min 8 chars) |
| Biometric | macOS, iOS, Android | Touch ID / Face ID / Fingerprint |
| Passkey | macOS, iOS | Platform passkey with biometric |

### 3.4 Identity Lifecycle

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Generate  │ ──► │   Create    │ ──► │   Login     │
│   4-words   │     │   Vault     │     │   Session   │
└─────────────┘     └─────────────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │   Export    │
                    │   Backup    │
                    └─────────────┘
```

### 3.5 Password Requirements

| Requirement | Value |
|-------------|-------|
| Minimum length | 8 characters |
| Strength indicator | Weak / Fair / Good / Strong |
| Biometric fallback | Always available |

### 3.6 Session Management

| Property | Value |
|----------|-------|
| Session ID | UUID v4 |
| Persistence | Until explicit logout |
| Auto-login | Disabled (require explicit selection) |
| Multi-device | Supported (same identity) |

---

## 4. Access Control (RBAC)

### 4.1 Role Definitions

| Role | Level | Description |
|------|-------|-------------|
| **Owner** | 4 | Full control, can delete entity |
| **Admin** | 3 | Manage members, settings |
| **Member** | 2 | Read and write access |
| **Guest** | 1 | Read-only access |

### 4.2 Permissions Matrix

| Action | Owner | Admin | Member | Guest |
|--------|-------|-------|--------|-------|
| View entity | ✅ | ✅ | ✅ | ✅ |
| Send message | ✅ | ✅ | ✅ | ❌ |
| Edit own message | ✅ | ✅ | ✅ | ❌ |
| Delete own message | ✅ | ✅ | ✅ | ❌ |
| Delete any message | ✅ | ✅ | ❌ | ❌ |
| Add reaction | ✅ | ✅ | ✅ | ❌ |
| Upload file | ✅ | ✅ | ✅ | ❌ |
| Delete file | ✅ | ✅ | ❌ | ❌ |
| Create document | ✅ | ✅ | ✅ | ❌ |
| Edit document | ✅ | ✅ | ✅ | ❌ |
| Create Kanban card | ✅ | ✅ | ✅ | ❌ |
| Move Kanban card | ✅ | ✅ | ✅ | ❌ |
| Delete Kanban card | ✅ | ✅ | ❌ | ❌ |
| Add member | ✅ | ✅ | ❌ | ❌ |
| Remove member | ✅ | ✅ | ❌ | ❌ |
| Change member role | ✅ | ✅ | ❌ | ❌ |
| Edit entity settings | ✅ | ✅ | ❌ | ❌ |
| Delete entity | ✅ | ❌ | ❌ | ❌ |
| Transfer ownership | ✅ | ❌ | ❌ | ❌ |

### 4.3 Role Badges

| Role | Color | Icon |
|------|-------|------|
| Owner | Amber `#E59933` | Crown |
| Admin | Blue `#4D80E6` | Shield |
| Member | Gray `#808080` | User |
| Guest | Light Gray `#9999B3` | Eye |

### 4.4 Read-Only Indicator

When user lacks edit permissions, show:
- Grayed-out input fields
- "Read Only" banner at top
- Disabled action buttons

---

## 5. Screen Inventory

### 5.1 Authentication Screens

#### 5.1.1 Welcome Screen
- **Route**: `/welcome`
- **Purpose**: Entry point for unauthenticated users
- **Components**:
  - App logo and tagline
  - "Sign In" button → Login Screen
  - "Create Identity" button → Create Identity Screen
  - "Use Biometric" button (if vault with passkey exists)

#### 5.1.2 Login Screen
- **Route**: `/login`
- **Purpose**: Authenticate with existing identity
- **Components**:
  - Vault selector (list of stored identities)
  - Password input field
  - "Forgot Password" link
  - "Login" button
  - "Use Biometric" option (if available)
- **Validation**:
  - Password required, min 8 chars
  - Show error on invalid credentials

#### 5.1.3 Create Identity Screen
- **Route**: `/create-identity`
- **Purpose**: Generate new four-word identity
- **Components**:
  - Generated four-word identity (read-only, with copy button)
  - "Regenerate" button
  - Display name input (required)
  - Password input with strength indicator
  - Confirm password input
  - "Enable Biometric" checkbox
  - "Create" button
- **Validation**:
  - Display name: 1-50 chars
  - Password: min 8 chars
  - Passwords must match

#### 5.1.4 Vault Management Screen
- **Route**: `/vault-management`
- **Purpose**: Manage stored identities
- **Components**:
  - List of vaults (four-word, display name, last accessed)
  - Enable/disable biometric per vault
  - Delete vault (requires password confirmation)

### 5.2 Main Application Layout

#### 5.2.1 Adaptive Scaffold
- **Desktop (≥1024px)**: Three-column layout
  - Left: Sidebar (280px)
  - Center: Detail pane (flex)
  - Right: Thread panel (320px, collapsible)
- **Tablet (768-1023px)**: Two-column with drawer sidebar
- **Mobile (<768px)**: Single column with bottom navigation

#### 5.2.2 Sidebar Sections

| Section | Content | Collapsible |
|---------|---------|-------------|
| Profile Header | Avatar, name, four-word, network status | No |
| My Organizations | Entities where role = Owner | Yes |
| My Communities | Entities where role != Owner | Yes |
| Personal | Personal entities (isPersonal = true) | Yes |
| Direct Messages | Contacts with presence status | Yes |

#### 5.2.3 Sidebar Item Structure
```
[Icon] [Name]          [Role Badge] [Unread Count]
       [Four-word (truncated)]
```

### 5.3 Entity Screens

#### 5.3.1 Home Screen
- **Route**: `/`
- **Purpose**: Dashboard when no entity selected
- **Components**:
  - Welcome message with user's name
  - Quick statistics (entities count, messages today, etc.)
  - Recent activity feed
  - Quick actions (create org, add contact)

#### 5.3.2 Entity Detail Screen
- **Route**: `/entity/:type/:id`
- **Purpose**: Main content area for selected entity
- **Tabs** (conditional):

| Tab | Available For | Description |
|-----|---------------|-------------|
| Board | Project only | Kanban board |
| Chat | All | Messaging |
| Drive | All | File storage |
| Documents | All | CRDT docs |
| Details | All | Metadata |

#### 5.3.3 Contact Chat Screen
- **Route**: `/contact/:fourWords/chat`
- **Purpose**: 1:1 direct messaging
- **Components**:
  - Contact header (name, presence, call buttons)
  - Message list
  - Compose bar
  - Thread panel (side)

### 5.4 Modal Screens

#### 5.4.1 Create Entity Sheet
- **Trigger**: Sidebar "+" button
- **Fields**:
  - Entity type selector (Org/Project/Channel/Group)
  - Name input
  - Description input (optional)
  - Parent organization (if applicable)
  - "Create as local-only" checkbox
- **Validation**:
  - Name: 1-100 chars, required
  - Type: required

#### 5.4.2 Add Contact Sheet
- **Trigger**: Direct Messages "+" button
- **Fields**:
  - Four-word identity input OR
  - Display name (for local-only contact)
  - "Create as local-only" checkbox
- **Validation**:
  - Four-word: valid dictionary words
  - Display name: 1-50 chars if local-only

#### 5.4.3 Card Detail Sheet
- **Trigger**: Kanban card tap
- **Sections**:
  - Title (editable)
  - Description (editable, markdown)
  - Priority selector
  - Assignee selector
  - Column (read-only, current column)
  - Comments list
  - Delete button (if authorized)

---

## 6. Messaging System

### 6.1 Message Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | UUID | Yes | Unique identifier |
| `entityId` | UUID | Yes | Parent entity |
| `author` | String | Yes | Sender's four-word |
| `authorDisplayName` | String | Yes | Sender's display name |
| `text` | String | Yes | Message content (markdown) |
| `timestamp` | Int64 | Yes | Unix timestamp (seconds) |
| `replyToId` | UUID? | No | Parent message (for threads) |
| `reactions` | Map<String, List<String>> | No | emoji → list of reactor four-words |
| `isEdited` | Bool | No | Has been modified |
| `isDeleted` | Bool | No | Soft deleted |

### 6.2 Message Features

#### 6.2.1 Markdown Support
| Element | Syntax | Rendered |
|---------|--------|----------|
| Bold | `**text**` | **text** |
| Italic | `*text*` | *text* |
| Code inline | `` `code` `` | `code` |
| Code block | ``` ```code``` ``` | Code block |
| Link | `[text](url)` | Hyperlink |
| Image | `![alt](url)` | Embedded image |
| Heading | `# Heading` | Large text |
| List | `- item` | Bullet list |
| Quote | `> quote` | Block quote |

#### 6.2.2 Threading
- Click message → Open thread panel (right side)
- Thread panel shows:
  - Parent message
  - All replies chronologically
  - Reply compose bar
- Thread indicator on parent message: "X replies"

#### 6.2.3 Reactions
- Hover/tap message → Reaction button
- Emoji picker (common emojis + search)
- Show reaction counts on message
- Tap reaction to add/remove own

#### 6.2.4 @Mentions
- Type `@` → Show member list
- Select member → Insert `@four-word`
- Mentioned users receive notification
- Render as clickable link

#### 6.2.5 Message Actions

| Action | Permission | Gesture |
|--------|------------|---------|
| Reply | Member+ | Click reply button |
| React | Member+ | Click reaction button |
| Edit | Author | Click edit (or long-press) |
| Delete | Author or Admin+ | Click delete |
| Copy | All | Click copy |

### 6.3 Message Composer

| Component | Description |
|-----------|-------------|
| Text input | Multi-line, supports markdown |
| Send button | Disabled when empty |
| Attach button | Open file picker |
| Emoji button | Open emoji picker |
| @mention | Auto-complete on `@` |

---

## 7. Virtual Disk System

### 7.1 Disk Types

| Type | Encryption | Access | Use Case |
|------|------------|--------|----------|
| **Private** | AES-256-GCM (owner key) | Owner only | Personal drafts, notes |
| **Public** | None (content-addressed) | World-readable | Published content, websites |
| **Shared** | AES-256-GCM (group key) | Entity members | Collaborative files |

### 7.2 File Properties

| Property | Type | Description |
|----------|------|-------------|
| `path` | String | Full path (e.g., `/docs/report.pdf`) |
| `name` | String | File name |
| `size` | Int64 | Size in bytes |
| `mimeType` | String | MIME type |
| `modifiedAt` | Timestamp | Last modification |
| `hash` | String | BLAKE3 content hash |
| `isDirectory` | Bool | True if folder |

### 7.3 File Operations

| Operation | API | Permission |
|-----------|-----|------------|
| List directory | `diskListDirectory` | Member+ |
| Read file | `diskReadFile` | Member+ |
| Write file | `diskWriteFile` | Member+ |
| Create directory | `diskCreateDirectory` | Member+ |
| Delete file | `diskDeleteFile` | Admin+ |
| Get stats | `diskStats` | Member+ |

### 7.4 Drive Browser UI

| Component | Description |
|-----------|-------------|
| Breadcrumb | Path navigation (clickable) |
| Disk type tabs | Private / Public / Shared |
| View toggle | List / Grid |
| Sort options | Name / Size / Date |
| Upload button | Trigger file picker |
| New folder button | Create directory dialog |
| File list/grid | File items with actions |
| Storage bar | Usage vs available |

### 7.5 File Item Display

```
[Icon] [Name]                    [Size] [Modified Date]
       [MIME type]               [Actions: Download, Delete]
```

---

## 8. Kanban System

### 8.1 Default Columns

| Column | Color | Order |
|--------|-------|-------|
| Backlog | Gray `#6B7280` | 0 |
| To Do | Blue `#3B82F6` | 1 |
| In Progress | Amber `#F59E0B` | 2 |
| Review | Purple `#8B5CF6` | 3 |
| Done | Green `#10B981` | 4 |

### 8.2 Card Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | UUID | Yes | Unique identifier |
| `projectId` | UUID | Yes | Parent project |
| `column` | String | Yes | Current column ID |
| `title` | String | Yes | Card title (1-200 chars) |
| `description` | String? | No | Markdown description |
| `assignee` | String? | No | Assigned member's four-word |
| `priority` | Enum | Yes | Low/Normal/High/Urgent |
| `position` | Int | Yes | Order within column |
| `commentCount` | Int | Yes | Number of comments |
| `createdAt` | Timestamp | Yes | Creation time |

### 8.3 Priority Levels

| Priority | Color | Icon |
|----------|-------|------|
| Low | Gray `#6B7280` | Arrow down |
| Normal | Blue `#3B82F6` | Minus |
| High | Amber `#F59E0B` | Arrow up |
| Urgent | Red `#EF4444` | Exclamation |

### 8.4 Kanban Operations

| Operation | Permission | UI |
|-----------|------------|-----|
| Create card | Member+ | "+" button in column |
| Edit card | Member+ | Click card → detail sheet |
| Move card | Member+ | Drag-and-drop between columns |
| Delete card | Admin+ | Delete button in detail sheet |
| Assign card | Member+ | Assignee picker in detail sheet |

### 8.5 Board UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│ [Backlog]     [To Do]     [In Progress]  [Review]   [Done]  │
│ ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐ ┌─────────┐
│ │ Card 1  │  │ Card 3  │  │ Card 5  │  │ Card 7  │ │ Card 9  │
│ └─────────┘  └─────────┘  └─────────┘  └─────────┘ └─────────┘
│ ┌─────────┐  ┌─────────┐  ┌─────────┐
│ │ Card 2  │  │ Card 4  │  │ Card 6  │
│ └─────────┘  └─────────┘  └─────────┘
│ [+ Add]      [+ Add]      [+ Add]      [+ Add]     [+ Add]   │
└─────────────────────────────────────────────────────────────┘
```

---

## 9. Voice/Video Calls

### 9.1 Call States

| State | Description | UI |
|-------|-------------|-----|
| `Idle` | No active call | Normal view |
| `Initiating` | Outgoing call starting | Calling overlay |
| `Ringing` | Outgoing call ringing | Ring animation |
| `Incoming` | Received call invitation | Incoming call overlay |
| `Connecting` | WebRTC negotiating | Connecting spinner |
| `Connected` | Active call | Call view |
| `Reconnecting` | Connection interrupted | Reconnecting indicator |
| `OnHold` | Call paused | Hold indicator |
| `Ending` | Call terminating | Ending indicator |
| `Ended` | Call finished | Return to previous view |

### 9.2 Call Properties

| Property | Type | Description |
|----------|------|-------------|
| `callId` | UUID | Unique call identifier |
| `participants` | List | Participant info list |
| `hasVideo` | Bool | Video enabled |
| `hasAudio` | Bool | Audio enabled |
| `isScreenSharing` | Bool | Screen share active |
| `duration` | Duration | Call duration |
| `startedAt` | Timestamp | Call start time |

### 9.3 Call Controls

| Control | Icon | Action |
|---------|------|--------|
| Mute | Microphone | Toggle audio |
| Video | Camera | Toggle video |
| Screen Share | Monitor | Start/stop screen share |
| End Call | Phone (red) | Terminate call |
| Settings | Gear | Open device selector |

### 9.4 Device Selection

| Device Type | API | Platform Support |
|-------------|-----|------------------|
| Audio Input | Microphone list | All |
| Audio Output | Speaker list | All |
| Video | Camera list | All |

### 9.5 Incoming Call Overlay

```
┌─────────────────────────────┐
│                             │
│     [Caller Avatar]         │
│     Caller Display Name     │
│     caller-four-words       │
│                             │
│  [Reject (Red)] [Accept (Green)]  │
│                             │
└─────────────────────────────┘
```

### 9.6 Active Call View

```
┌─────────────────────────────────────┐
│                                     │
│     ┌───────────────────────────┐   │
│     │                           │   │
│     │     Remote Video          │   │
│     │                           │   │
│     └───────────────────────────┘   │
│                   ┌─────┐           │
│                   │Local│           │
│                   └─────┘           │
│                                     │
│  [Duration: 00:05:23]               │
│                                     │
│  [Mute] [Video] [Share] [End]       │
└─────────────────────────────────────┘
```

---

## 10. Network & P2P

### 10.1 Network States

| State | Indicator | Description |
|-------|-----------|-------------|
| Online | Green dot | Connected to peers |
| Connecting | Yellow dot | Establishing connections |
| Offline | Gray dot | No network connectivity |
| Local Only | Blue dot | Local mode, no sync |

### 10.2 Network Info

| Property | Type | Description |
|----------|------|-------------|
| `isNetworking` | Bool | Network subsystem active |
| `listenAddress` | String | Local listening address |
| `externalAddress` | String? | NAT-reflected external address |
| `peerCount` | Int | Connected peer count |

### 10.3 Peer Info

| Property | Type | Description |
|----------|------|-------------|
| `fourWords` | String | Peer identity |
| `displayName` | String? | Peer display name |
| `endpoint` | String | IP:Port |
| `lastSeen` | Timestamp | Last activity |
| `isBootstrap` | Bool | Is bootstrap node |

### 10.4 Network Panel UI

```
┌─────────────────────────────────────┐
│ Network Status: [Online]            │
│ Listen: 0.0.0.0:54321               │
│ External: 203.0.113.42:54321        │
│                                     │
│ Bootstrap Nodes:                    │
│ ┌─────────────────────────────────┐ │
│ │ ● saorsa-2.saorsalabs.com:11000│ │
│ │ ● saorsa-3.saorsalabs.com:11000│ │
│ └─────────────────────────────────┘ │
│                                     │
│ Connected Peers (5):                │
│ ┌─────────────────────────────────┐ │
│ │ ocean-forest-moon-star          │ │
│ │   198.51.100.1:50000 (2m ago)   │ │
│ │ bright-river-swift-eagle        │ │
│ │   198.51.100.2:50001 (5m ago)   │ │
│ └─────────────────────────────────┘ │
│                                     │
│ [Start/Stop Networking]             │
└─────────────────────────────────────┘
```

---

## 11. User Stories

### 11.1 Identity & Authentication

#### US-001: Create New Identity
**As a** new user
**I want to** generate a four-word identity
**So that** I can join the network with a memorable, verifiable address

**Acceptance Criteria:**
- [ ] System generates valid four-word identity from dictionary
- [ ] User can regenerate until satisfied
- [ ] User must set display name (1-50 chars)
- [ ] User must set password (min 8 chars)
- [ ] Password strength indicator shows Weak/Fair/Good/Strong
- [ ] Biometric enrollment optional
- [ ] Vault created with encrypted private key
- [ ] User redirected to main app after creation

#### US-002: Login with Password
**As a** returning user
**I want to** login with my password
**So that** I can access my account

**Acceptance Criteria:**
- [ ] Show list of available vaults
- [ ] Password field with show/hide toggle
- [ ] Error message on invalid password
- [ ] Redirect to main app on success
- [ ] Session persists until explicit logout

#### US-003: Login with Biometric
**As a** user with biometric enabled
**I want to** login with Touch ID/Face ID
**So that** I can quickly access my account

**Acceptance Criteria:**
- [ ] Biometric button visible if vault has passkey
- [ ] Prompt system biometric dialog
- [ ] Auto-login on successful verification
- [ ] Fallback to password on failure

#### US-004: Export Identity Backup
**As a** user
**I want to** export my identity to a file
**So that** I can recover on another device

**Acceptance Criteria:**
- [ ] Export button in settings
- [ ] Require password confirmation
- [ ] Generate encrypted backup file
- [ ] Show file save dialog
- [ ] Clear instructions for import

### 11.2 Entity Management

#### US-010: Create Organization
**As a** user
**I want to** create an organization
**So that** I can collaborate with my team

**Acceptance Criteria:**
- [ ] Click "+" in sidebar → Create Entity sheet
- [ ] Select "Organization" type
- [ ] Enter name (required)
- [ ] Enter description (optional)
- [ ] Option for local-only
- [ ] User becomes Owner
- [ ] Org appears in "My Organizations" section

#### US-011: Create Project in Organization
**As an** organization owner/admin
**I want to** create a project
**So that** I can organize work with Kanban boards

**Acceptance Criteria:**
- [ ] Select organization first
- [ ] Click "+" → Create Entity sheet
- [ ] Select "Project" type
- [ ] Enter name
- [ ] Kanban board auto-created with 5 columns
- [ ] Project appears under organization in sidebar

#### US-012: Invite Member to Entity
**As an** entity owner/admin
**I want to** invite members by their four-word identity
**So that** they can collaborate

**Acceptance Criteria:**
- [ ] Open entity details → Members tab
- [ ] Click "Invite Member"
- [ ] Enter four-word identity
- [ ] Select role (Admin/Member/Guest)
- [ ] Invitation sent via network
- [ ] Member added on acceptance

#### US-013: Remove Member from Entity
**As an** entity owner/admin
**I want to** remove a member
**So that** they no longer have access

**Acceptance Criteria:**
- [ ] Open entity details → Members tab
- [ ] Click remove button on member
- [ ] Confirmation dialog
- [ ] Member removed from entity
- [ ] Member's access revoked immediately

### 11.3 Messaging

#### US-020: Send Message to Channel
**As a** member
**I want to** send a message
**So that** I can communicate with the team

**Acceptance Criteria:**
- [ ] Type in compose bar
- [ ] Click send or press Enter
- [ ] Message appears in list immediately
- [ ] Message synced to other members
- [ ] Markdown rendered correctly

#### US-021: Reply in Thread
**As a** user
**I want to** reply to a specific message
**So that** I can keep discussions organized

**Acceptance Criteria:**
- [x] Click message → Thread panel opens
- [ ] See parent message at top
- [x] Reply via dialog from message menu
- [x] Reply appears in thread
- [x] Parent message shows "X replies"

#### US-022: React to Message
**As a** member
**I want to** react with an emoji
**So that** I can quickly acknowledge messages

**Acceptance Criteria:**
- [x] Hover/tap message → Reaction button
- [x] Emoji picker appears
- [x] Select emoji → Reaction added
- [x] Reaction count shown on message
- [x] Click again to remove own reaction

#### US-023: Edit Own Message
**As a** message author
**I want to** edit my message
**So that** I can correct mistakes

**Acceptance Criteria:**
- [x] Click edit button on own message
- [x] Edit dialog activated
- [x] Save changes → Message updated
- [x] "(edited)" indicator shown
- [x] Edit synced to others

#### US-024: Delete Message
**As a** message author or admin
**I want to** delete a message
**So that** I can remove inappropriate content

**Acceptance Criteria:**
- [x] Click delete button
- [x] Confirmation dialog
- [x] Message removed from list
- [x] Deletion synced to others

### 11.4 File Storage

#### US-030: Upload File
**As a** member
**I want to** upload a file
**So that** I can share with the team

**Acceptance Criteria:**
- [ ] Click upload button in Drive tab
- [ ] File picker opens
- [ ] Select file(s)
- [ ] Upload progress shown
- [ ] File appears in list on completion

#### US-031: Download File
**As a** member
**I want to** download a file
**So that** I can use it locally

**Acceptance Criteria:**
- [ ] Click file → Download button
- [ ] Save dialog or auto-download
- [ ] File downloaded to device

#### US-032: Create Folder
**As a** member
**I want to** create a folder
**So that** I can organize files

**Acceptance Criteria:**
- [ ] Click "New Folder" button
- [ ] Enter folder name dialog
- [ ] Folder created in current path
- [ ] Navigation updates to show folder

### 11.5 Kanban

#### US-040: Create Kanban Card
**As a** project member
**I want to** create a card
**So that** I can track a task

**Acceptance Criteria:**
- [ ] Click "+" in any column
- [ ] Enter card title
- [ ] Card created in that column
- [ ] Card detail sheet opens for more fields

#### US-041: Move Card Between Columns
**As a** project member
**I want to** drag a card to another column
**So that** I can update task status

**Acceptance Criteria:**
- [ ] Drag card from one column
- [ ] Drop in target column
- [ ] Card position updated
- [ ] Change synced via CRDT

#### US-042: Assign Card to Member
**As a** project member
**I want to** assign a card to someone
**So that** ownership is clear

**Acceptance Criteria:**
- [ ] Open card detail sheet
- [ ] Click assignee picker
- [ ] Select member from list
- [ ] Assignee shown on card
- [ ] Assigned user notified

### 11.6 Calls

#### US-050: Start Voice Call
**As a** user
**I want to** call a contact
**So that** we can talk in real-time

**Acceptance Criteria:**
- [ ] Click phone icon on contact
- [ ] Call initiated
- [ ] Ring on recipient
- [ ] On accept, call connects
- [ ] Audio works both ways

#### US-051: Start Video Call
**As a** user
**I want to** start a video call
**So that** we can see each other

**Acceptance Criteria:**
- [ ] Click video icon on contact
- [ ] Call initiated with video
- [ ] Video streams both ways
- [ ] Can toggle video off/on

#### US-052: Share Screen
**As a** call participant
**I want to** share my screen
**So that** I can present content

**Acceptance Criteria:**
- [ ] Click screen share button in call
- [ ] Select screen/window
- [ ] Screen visible to others
- [ ] Stop sharing when done

---

## 12. Demo Mode

### 12.1 Activation

| Method | How |
|--------|-----|
| Compile flag | `--dart-define=DEMO_MODE=true` |
| Web default | FFI unavailable → auto demo mode |
| Runtime toggle | Settings → Demo Mode switch |

### 12.2 Pre-populated Data

#### Identity
```
Four Words: ocean-forest-moon-star
Display Name: Demo User
```

#### Organizations

| Name | Type | Role | Members |
|------|------|------|---------|
| Saorsa Labs | Organisation | Owner | 12 |
| Open Source Collective | Organisation | Member | 150 |

#### Projects

| Name | Parent | Role |
|------|--------|------|
| Communitas Flutter | Saorsa Labs | Owner |
| Documentation | Saorsa Labs | Member |

#### Channels

| Name | Parent | Role |
|------|--------|------|
| general | Saorsa Labs | Member |
| engineering | Saorsa Labs | Member |
| design | Communitas Flutter | Member |

#### Contacts

| Name | Four Words | Status |
|------|------------|--------|
| Alice Chen | bright-ocean-swift-eagle | Online |
| Bob Smith | calm-river-ancient-oak | Away |
| Local Contact | (none) | Offline |

#### Messages

Pre-populate each channel with 5-10 sample messages:
- Mix of authors
- Include reactions
- Include one thread with 3 replies
- Include markdown examples

#### Kanban Cards

| Title | Column | Priority | Assignee |
|-------|--------|----------|----------|
| Implement sidebar | In Progress | High | Demo User |
| Add theme system | To Do | Normal | Alice |
| Write tests | Backlog | Low | - |
| Review PR #42 | Review | Urgent | Bob |
| Deploy v1.0 | Done | High | Demo User |

#### Network (Mock)

```
Status: Online (demo)
Peers: 3 connected
Bootstrap: 2/2 connected
```

### 12.3 Demo Mode Behavior

- All CRUD operations work locally (not synced)
- Network operations simulated with delays
- Calls simulated (no actual WebRTC)
- Presence statuses cycle randomly
- No persistence (reset on restart)

---

## 13. Error States

### 13.1 Authentication Errors

| Error | Display | Recovery |
|-------|---------|----------|
| Invalid password | "Incorrect password" | Retry |
| Vault not found | "Identity not found on this device" | Create or import |
| Biometric failed | "Authentication failed" | Fall back to password |
| Vault corrupted | "Unable to read vault" | Delete and recreate |

### 13.2 Network Errors

| Error | Display | Recovery |
|-------|---------|----------|
| No connectivity | "Offline - changes will sync when online" | Auto-retry |
| Peer unreachable | "Unable to reach [four-word]" | Retry button |
| Bootstrap failed | "Cannot connect to network" | Check connection |

### 13.3 Entity Errors

| Error | Display | Recovery |
|-------|---------|----------|
| Name taken | "Name already in use" | Choose different name |
| Permission denied | "You don't have permission" | Request access |
| Entity not found | "Entity no longer exists" | Remove from list |

### 13.4 File Errors

| Error | Display | Recovery |
|-------|---------|----------|
| Upload failed | "Upload failed - tap to retry" | Retry button |
| File too large | "File exceeds maximum size" | Choose smaller file |
| Download failed | "Download failed" | Retry button |
| File not found | "File no longer exists" | Remove from list |

### 13.5 Call Errors

| Error | Display | Recovery |
|-------|---------|----------|
| No microphone | "Microphone access required" | Settings link |
| No camera | "Camera access required" | Settings link |
| Call failed | "Call could not connect" | Retry button |
| Peer unavailable | "[Name] is offline" | Try again later |

---

## 14. Theme & Design

### 14.1 Theme: Warm Digital Commons

A forest-inspired aesthetic with deep greens, earth tones, and jade accents.

### 14.2 Color Palette

#### Foundation Colors
| Name | Hex | Usage |
|------|-----|-------|
| Deep Forest | `#1A241F` | Primary background |
| Moss | `#2D3D36` | Secondary background |
| Fern | `#394C43` | Tertiary/hover |
| Jade | `#4CAF83` | Primary accent |
| Amber | `#E0B265` | Secondary accent |
| Cream | `#F2EEE7` | Light text |

#### Status Colors
| Name | Hex | Usage |
|------|-----|-------|
| Online | `#4CAF50` | Connected/available |
| Away | `#FFC107` | Idle/away |
| Offline | `#9E9E9E` | Disconnected |
| Error | `#E74C3C` | Errors/destructive |
| Success | `#4CAF83` | Success states |
| Warning | `#E0B265` | Warnings |

#### Entity Colors
| Entity | Hex |
|--------|-----|
| Organisation | `#3E8E7E` |
| Project | `#DAA520` |
| Channel | `#4CAF83` |
| Group | `#9B59B6` |
| Person | `#FF7F7F` |

#### Role Colors
| Role | Hex |
|------|-----|
| Owner | `#E59933` |
| Admin | `#4D80E6` |
| Member | `#808080` |
| Guest | `#9999B3` |

### 14.3 Typography

| Element | Font | Size | Weight |
|---------|------|------|--------|
| H1 | System | 28px | Bold |
| H2 | System | 24px | Semi-bold |
| H3 | System | 20px | Semi-bold |
| Body | System | 16px | Regular |
| Caption | System | 14px | Regular |
| Four-word | Monospace | 14px | Regular |

### 14.4 Spacing Scale

| Name | Value |
|------|-------|
| xs | 4px |
| sm | 8px |
| md | 16px |
| lg | 24px |
| xl | 32px |
| xxl | 48px |

### 14.5 Border Radius

| Element | Radius |
|---------|--------|
| Buttons | 8px |
| Cards | 12px |
| Modals | 16px |
| Avatars | 50% |

### 14.6 Shadows

| Level | Shadow |
|-------|--------|
| Low | `0 1px 2px rgba(0,0,0,0.1)` |
| Medium | `0 4px 8px rgba(0,0,0,0.15)` |
| High | `0 8px 16px rgba(0,0,0,0.2)` |

---

## Appendix A: API Reference

See `communitas-core/src/command.rs` for complete Command enum.
See `communitas-core/src/query.rs` for complete Query enum.
See `communitas-core/src/event.rs` for complete Event enum.

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| Four-word identity | Human-readable network address (e.g., ocean-forest-moon-star) |
| Vault | Encrypted local storage for identity and credentials |
| Entity | Generic term for Organisation/Project/Channel/Group |
| CRDT | Conflict-free Replicated Data Type for offline-first sync |
| PQC | Post-Quantum Cryptography (ML-DSA, ML-KEM) |
| Gossip | Peer-to-peer message dissemination protocol |

---

*Document Version: 1.0*
*This specification is the definitive source of truth for all Communitas GUI implementations.*
