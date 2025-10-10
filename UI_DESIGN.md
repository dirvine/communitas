# Communitas UI Design Document

**Version:** 1.0
**Date:** 2025-10-09
**Status:** Implementation & Testing Phase

---

## Table of Contents

1. [Overview](#overview)
2. [Design Principles](#design-principles)
3. [Authentication Flow](#authentication-flow)
4. [Main Interface Structure](#main-interface-structure)
5. [Entity Types & Management](#entity-types--management)
6. [Communication Features](#communication-features)
7. [File & Document System](#file--document-system)
8. [Website Publishing System](#website-publishing-system)
9. [Collaborative CRDT Editing](#collaborative-crdt-editing)
10. [Testing Strategy](#testing-strategy)
11. [Multi-Instance Testing](#multi-instance-testing)

---

## Overview

Communitas is a local-first, PQC-ready collaboration platform that merges WhatsApp, Dropbox, Zoom, and Slack functionality into a unified decentralized application. It uses Four-Word identities for human-verifiable addressing and provides per-entity virtual disks with CRDT-based collaborative editing.

### Technology Stack

- **Frontend**: React 18 + TypeScript + Material-UI
- **Backend**: Tauri v2 + Rust (saorsa-core v0.3.17)
- **Networking**: QUIC via ant-quic, IPv4-first with fallback
- **Storage**: Virtual disks with FEC, BLAKE3 content addressing
- **Cryptography**: Post-quantum (ML-DSA/ML-KEM) + ChaCha20-Poly1305
- **CRDT**: Automerge for collaborative editing
- **Testing**: Chrome DevTools MCP + communitas-bridge

---

## Design Principles

### 1. **Local-First with Network Sync**
- All operations work offline via IndexedDB
- Automatic sync when network returns
- Optimistic updates with conflict resolution

### 2. **Security & Privacy**
- End-to-end encryption by default
- Four-word identity system for human verification
- Post-quantum cryptography ready
- No central servers required

### 3. **Unified Interface**
- Single application for all collaboration needs
- Consistent design language across features
- Context-aware navigation

### 4. **Progressive Enhancement**
- Desktop app with full native features
- Web version with browser capabilities
- Graceful degradation for missing features

### 5. **Collaborative by Default**
- Real-time CRDT-based editing
- Presence indicators for all collaborators
- Conflict-free merging

---

## Authentication Flow

### Registration (Passkey-First)

```
┌─────────────────────────────────────────┐
│     Welcome to Communitas               │
│                                         │
│  ┌───────────────────────────────────┐ │
│  │ Name *                            │ │
│  │ [Your Name]                       │ │
│  └───────────────────────────────────┘ │
│                                         │
│  ┌───────────────────────────────────┐ │
│  │ Password (Optional)               │ │
│  │ [Leave empty for auto-password]   │ │
│  └───────────────────────────────────┘ │
│                                         │
│  ℹ️  Leave password empty to auto-      │
│     generate a secure password and      │
│     enable Touch ID/Face ID             │
│                                         │
│  ┌───────────────────────────────────┐ │
│  │ [Create Identity]                 │ │
│  └───────────────────────────────────┘ │
│                                         │
│  Or scan QR to import identity          │
│  ┌───────────────────────────────────┐ │
│  │ [📷 Import from QR Code]          │ │
│  └───────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

**Flow:**
1. User enters name only (password optional)
2. System generates four-word identity (e.g., "ocean-forest-moon-star")
3. If password empty:
   - Auto-generate 32-char secure password
   - Store in system keyring (macOS Keychain, Windows Credential Manager)
   - Desktop: Show keyring confirmation
   - Browser: Offer Touch ID/Face ID enrollment

**Success Dialog (Desktop):**
```
┌─────────────────────────────────────────┐
│  ✅ Identity Created                     │
│                                         │
│  Your four-word identity:               │
│  ocean-forest-moon-star                 │
│                                         │
│  ✅ Secure Password Created             │
│  We've created a secure password and    │
│  stored it in your system keyring.      │
│  You won't need to type it again.       │
│                                         │
│  [💾 Download Identity Backup]          │
│  [Continue to App]                      │
└─────────────────────────────────────────┘
```

**Success Dialog (Browser):**
```
┌─────────────────────────────────────────┐
│  ✅ Identity Created                     │
│                                         │
│  Your four-word identity:               │
│  ocean-forest-moon-star                 │
│                                         │
│  🔐 Recommended: Set Up Biometric Login │
│  Enable Touch ID or Face ID for the     │
│  fastest, most secure login experience. │
│                                         │
│  [🔐 Set Up Touch ID / Face ID]         │
│  [💾 Download Identity Backup]          │
│  [Continue to App]                      │
└─────────────────────────────────────────┘
```

### Login (Identity Picker)

```
┌─────────────────────────────────────────┐
│     Welcome Back to Communitas          │
│                                         │
│  Select your identity to continue       │
│                                         │
│  [🔍 Search identities...]              │
│                                         │
│  ┌───────────────────────────────────┐ │
│  │ [A] Alice                         │ │
│  │     ocean-forest-moon-star        │ │
│  │     Last used: 5m ago             │ │
│  │     [🔐 Passkey] [🔑 Touch ID]    │ │
│  └───────────────────────────────────┘ │
│                                         │
│  ┌───────────────────────────────────┐ │
│  │ [B] Bob (Work)                    │ │
│  │     mountain-river-star-tree      │ │
│  │     Last used: 2d ago             │ │
│  │     [→ Sign in with password]     │ │
│  └───────────────────────────────────┘ │
│                                         │
│  ─────────── or ───────────             │
│                                         │
│  Enter your four-word identity          │
│  [ocean-forest-moon-star]               │
│  [Sign In with Four Words]              │
│                                         │
│  ─────────── or ───────────             │
│                                         │
│  [📷 Import Identity from QR Code]      │
│  [+ Create New Identity]                │
└─────────────────────────────────────────┘
```

**Features:**
- Recent identities with avatars (color-coded by four-words hash)
- Passkey badge for biometric-enabled identities
- Single-click biometric authentication
- Manual four-word entry for new devices
- QR code import/export for easy identity transfer

---

## Main Interface Structure

### Layout Overview

```
┌────────────────────────────────────────────────────────────────┐
│ [☰] Communitas    [🔍 Search]    [🌐 Network: Connected] [👤]  │ ← Header
├────────────────────────────────────────────────────────────────┤
│ ┌──────────────┬────────────────────────────────────────────┐ │
│ │              │                                            │ │
│ │  SIDEBAR     │           MAIN CONTENT AREA               │ │
│ │              │                                            │ │
│ │  Channels    │  [Active Channel/Project/Document View]   │ │
│ │  Projects    │                                            │ │
│ │  Files       │                                            │ │
│ │  Contacts    │                                            │ │
│ │  Groups      │                                            │ │
│ │  Websites    │                                            │ │
│ │              │                                            │ │
│ │              │                                            │ │
│ │              │                                            │ │
│ └──────────────┴────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

### Header Bar

**Components:**
- **Menu Button** (☰): Opens navigation drawer
- **App Title**: "Communitas" with logo
- **Global Search**: Quick search across all entities
- **Network Status Indicator**:
  - 🟢 Connected (shows peer count)
  - 🟡 Connecting/Local mode
  - 🔴 Offline/Error (click to retry)
- **User Avatar**: Dropdown menu with:
  - Profile settings
  - Network status details
  - Active identity (four-words)
  - Switch identity
  - Settings
  - Logout

### Sidebar Navigation

**Primary Navigation Sections:**

```
┌──────────────────────┐
│ CONVERSATIONS        │
│ ├─ 💬 Channels       │
│ ├─ 📁 Projects       │
│ └─ 👥 Direct         │
│                      │
│ WORKSPACE            │
│ ├─ 📄 Files          │
│ ├─ 🌐 Websites       │
│ └─ 🎥 Meetings       │
│                      │
│ NETWORK              │
│ ├─ 👤 Contacts       │
│ ├─ 👥 Groups         │
│ └─ 🏢 Organizations  │
│                      │
│ SETTINGS             │
│ └─ ⚙️  Preferences   │
└──────────────────────┘
```

**Each Section Expandable:**
- Shows recent/pinned items
- Click to expand full list
- Right-click for context menu
- Drag to reorder

---

## Entity Types & Management

### 1. Channels (💬)

**Purpose**: Topic-based group conversations, like Slack channels

**Channel View:**
```
┌────────────────────────────────────────────────────────────┐
│ # general                         [📌 Pin] [🔔] [⋮ More]  │
├────────────────────────────────────────────────────────────┤
│ Description: Main discussion channel                       │
│ Members: 12 • Created: 2 weeks ago                        │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  [Messages Stream]                                         │
│                                                            │
│  Alice • 10:30 AM                                          │
│  Hey everyone! Check out the new design docs              │
│  📎 design-spec.md                                         │
│                                                            │
│  Bob • 10:32 AM                                            │
│  Looks great! 👍                                           │
│                                                            │
│  You • 10:35 AM                                            │
│  I'll review this afternoon                                │
│  🧵 2 replies                                              │
│                                                            │
├────────────────────────────────────────────────────────────┤
│ [Type a message...]              [📎] [😊] [🎤] [➤]       │
└────────────────────────────────────────────────────────────┘
```

**Features:**
- Real-time messaging with CRDT sync
- Thread support for organized discussions
- File attachments from virtual disk
- Emoji reactions
- Voice/video call initiation
- Message search
- Pin important messages
- Channel member management

**Create Channel:**
```
┌─────────────────────────────────────────┐
│  Create New Channel                     │
│                                         │
│  Channel Name *                         │
│  [general]                              │
│                                         │
│  Description                            │
│  [Main discussion channel]              │
│                                         │
│  Privacy                                │
│  ○ Public (visible to all org members) │
│  ● Private (invite only)                │
│                                         │
│  Add Members                            │
│  [Search contacts...]                   │
│  Selected: Alice, Bob, Carol            │
│                                         │
│  [Cancel]        [Create Channel]       │
└─────────────────────────────────────────┘
```

### 2. Projects (📁)

**Purpose**: Structured workspaces with channels, files, and tasks

**Project Structure:**
```
Project: "Website Redesign"
├─ 💬 Channels
│  ├─ #general
│  ├─ #design
│  └─ #development
├─ 📄 Files
│  ├─ /docs
│  ├─ /designs
│  └─ /code
├─ ✅ Tasks (future)
└─ 📊 Dashboard
```

**Project View:**
```
┌────────────────────────────────────────────────────────────┐
│ 📁 Website Redesign        [⭐ Star] [👥 Team] [⋮ More]   │
├────────────────────────────────────────────────────────────┤
│ Tabs: [Overview] [Channels] [Files] [Members] [Settings]  │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  OVERVIEW                                                  │
│                                                            │
│  Description:                                              │
│  Complete redesign of the company website                 │
│                                                            │
│  Recent Activity                                           │
│  • Alice uploaded design-mockups.pdf                       │
│  • Bob created #development channel                        │
│  • Carol invited 3 new members                            │
│                                                            │
│  Quick Access                                              │
│  [💬 General Channel]  [📄 Project Files]                 │
│                                                            │
│  Team (8 members)                                          │
│  [A] [B] [C] [D] [E] ...                                   │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### 3. Files (📄)

**Purpose**: Document storage and collaborative editing with virtual disks

**Files View:**
```
┌────────────────────────────────────────────────────────────┐
│ 📄 Files                         [🔍] [↕️ Sort] [⋮ View]  │
├────────────────────────────────────────────────────────────┤
│ Navigation: My Files > Projects > Website Redesign         │
│                                                            │
│ [📁] Shared with Me                                        │
│ [📁] Recent                                                │
│ [📁] Starred                                               │
│ [📁] Trash                                                 │
│                                                            │
│ ├─ 📁 Projects                                             │
│ │  ├─ 📁 Website Redesign                                 │
│ │  │  ├─ 📄 home.md (CRDT)                                │
│ │  │  ├─ 📄 about.md (CRDT)                               │
│ │  │  ├─ 🎨 design-mockups.pdf                            │
│ │  │  └─ 📊 analytics.csv                                 │
│ │  └─ 📁 Mobile App                                       │
│ ├─ 📁 Personal                                             │
│ └─ 📁 Templates                                            │
│                                                            │
│ [+ New File] [+ New Folder] [📤 Upload]                   │
└────────────────────────────────────────────────────────────┘
```

**File Context Menu:**
- Open in editor
- Open in browser (for web files)
- Share with contacts/channels
- Download
- Rename
- Move to folder
- Add to favorites
- View history
- Delete

**Virtual Disk Structure:**
- **Private**: Encrypted, local-only storage
- **Public**: Content-addressed, distributed
- **Shared**: Group-accessible with shared encryption

### 4. Contacts (👤)

**Purpose**: Manage individual connections

**Contacts View:**
```
┌────────────────────────────────────────────────────────────┐
│ 👤 Contacts                      [🔍 Search] [+ Add]       │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  [A]  Alice                                      🟢 Online│
│       ocean-forest-moon-star                              │
│       [💬 Message] [📞 Call] [⋮ More]                     │
│                                                            │
│  [B]  Bob                                        🟡 Away  │
│       mountain-river-star-tree                            │
│       [💬 Message] [📞 Call] [⋮ More]                     │
│                                                            │
│  [C]  Carol                                      ⚫ Offline│
│       desert-ocean-wind-cloud                             │
│       [💬 Message] [📞 Call] [⋮ More]                     │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**Add Contact Dialog:**
```
┌─────────────────────────────────────────┐
│  Add Contact                            │
│                                         │
│  Four-Word Address *                    │
│  [ocean-forest-moon-star]               │
│                                         │
│  Or scan QR code                        │
│  [📷 Scan QR Code]                      │
│                                         │
│  Display Name                           │
│  [Alice]                                │
│                                         │
│  [Cancel]              [Add Contact]    │
└─────────────────────────────────────────┘
```

### 5. Groups (👥)

**Purpose**: Collections of users for shared access and permissions

**Groups View:**
```
┌────────────────────────────────────────────────────────────┐
│ 👥 Groups                        [🔍 Search] [+ Create]    │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Engineering Team (12 members)                             │
│  ├─ Members: Alice, Bob, Carol, +9                        │
│  ├─ Created: 3 months ago                                 │
│  └─ [👥 Manage] [💬 Message All] [⋮ More]                 │
│                                                            │
│  Design Team (5 members)                                   │
│  ├─ Members: Alice, David, Eve, +2                        │
│  ├─ Created: 2 months ago                                 │
│  └─ [👥 Manage] [💬 Message All] [⋮ More]                 │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### 6. Websites (🌐)

**Purpose**: DNS-free website publishing with identity-bound website roots

**Websites View:**
```
┌────────────────────────────────────────────────────────────┐
│ 🌐 Websites                      [🔍 Search] [+ Publish]   │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  My Portfolio Site                            🟢 Published│
│  ├─ Root: home.md                                         │
│  ├─ Published: 2 days ago                                 │
│  ├─ Visitors: 245                                         │
│  └─ [✏️ Edit] [👁️ Preview] [📊 Analytics] [⋮ More]        │
│                                                            │
│  Company Website                              🟢 Published│
│  ├─ Root: index.md                                        │
│  ├─ Published: 1 week ago                                 │
│  ├─ Visitors: 1,842                                       │
│  └─ [✏️ Edit] [👁️ Preview] [📊 Analytics] [⋮ More]        │
│                                                            │
│  [+ Publish New Website]                                   │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**Publish Website Dialog:**
```
┌─────────────────────────────────────────┐
│  Publish Website                        │
│                                         │
│  Website Name *                         │
│  [My Portfolio]                         │
│                                         │
│  Home Document (root) *                 │
│  [Select from Files...]                 │
│  Selected: home.md                      │
│                                         │
│  ℹ️  Your website will be accessible via│
│     your four-word identity:            │
│     ocean-forest-moon-star.communitas   │
│                                         │
│  [Cancel]            [Publish Website]  │
└─────────────────────────────────────────┘
```

---

## Communication Features

### Text Messaging

**Message Composer:**
```
┌────────────────────────────────────────────────────────────┐
│ [Type a message...]                                        │
│                                                            │
│ [📎 Attach File] [😊 Emoji] [🎤 Voice Message]           │
│                                                            │
│ Keyboard shortcuts:                                        │
│ • Enter: Send message                                     │
│ • Shift+Enter: New line                                   │
│ • @mention: Mention user                                  │
│ • #channel: Link to channel                               │
│ • :emoji: Insert emoji                                    │
└────────────────────────────────────────────────────────────┘
```

**Features:**
- Rich text formatting (bold, italic, code, links)
- File attachments from virtual disk
- Inline image/video preview
- Code syntax highlighting
- Emoji reactions
- Thread replies
- Edit/delete messages
- Message search

### Voice Calls (🎤)

**Call Interface:**
```
┌─────────────────────────────────────────┐
│             Voice Call                  │
│                                         │
│  [🔴] Alice                             │
│       ocean-forest-moon-star            │
│                                         │
│  [🟢] You (Muted)                       │
│                                         │
│  Duration: 05:23                        │
│                                         │
│  [🎤 Unmute] [🔊 Speaker] [📞 Hangup]  │
│                                         │
└─────────────────────────────────────────┘
```

**Features:**
- P2P audio via QUIC
- Automatic NAT traversal
- Mute/unmute
- Speaker selection
- Call recording (with consent)
- Background mode

### Video Calls (🎥)

**Video Call Interface:**
```
┌────────────────────────────────────────────────────────────┐
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │                                                      │ │
│  │           [Alice's Video Feed]                       │ │
│  │                                                      │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐     │
│  │ [Your Video] │ │ [Bob's Video]│ │ [Carol]      │     │
│  └──────────────┘ └──────────────┘ └──────────────┘     │
│                                                            │
│  [🎤] [🎥] [🖥️ Share] [💬 Chat] [👥 Participants] [📞]   │
└────────────────────────────────────────────────────────────┘
```

**Features:**
- Multi-party video conferencing
- Screen sharing with annotations
- Virtual backgrounds
- Recording with consent
- Picture-in-picture mode
- Grid/speaker view toggle
- Bandwidth adaptation

### Screen Sharing (🖥️)

**Screen Share Options:**
```
┌─────────────────────────────────────────┐
│  Share Your Screen                      │
│                                         │
│  [🖥️ Entire Screen]                    │
│  Share your entire desktop              │
│                                         │
│  [🪟 Application Window]                │
│  Share a specific application           │
│                                         │
│  [📑 Browser Tab]                       │
│  Share a browser tab (web only)         │
│                                         │
│  Options:                               │
│  ☑ Share system audio                   │
│  ☑ Enable annotations                   │
│  ☑ Show cursor                          │
│                                         │
│  [Cancel]              [Start Sharing]  │
└─────────────────────────────────────────┘
```

**Features:**
- Full screen or window sharing
- Audio sharing
- Remote annotations
- Pause/resume sharing
- Quality adjustment

---

## File & Document System

### Markdown Editor (CRDT-Enabled)

**Editor Interface:**
```
┌────────────────────────────────────────────────────────────┐
│ 📄 home.md                    [👁️ Preview] [⋮ More]        │
├────────────────────────────────────────────────────────────┤
│ Editing • Auto-saving • 2 collaborators active             │
│ Alice (editing line 15) • Bob (viewing)                    │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  # Welcome to My Portfolio                                 │
│                                                            │
│  I'm a software engineer passionate about...               │
│  ← Alice                                                   │
│                                                            │
│  ## Projects                                               │
│                                                            │
│  - **Communitas**: Decentralized collaboration platform    │
│  - **DataViz**: Data visualization toolkit                 │
│                                                            │
│  ## Contact                                                │
│                                                            │
│  Feel free to reach out!                                   │
│                                                            │
├────────────────────────────────────────────────────────────┤
│ Markdown toolbar: [B] [I] [H1] [H2] [H3] [•] [1] [>] [```]│
└────────────────────────────────────────────────────────────┘
```

**CRDT Features:**
- **Real-time collaboration**: See cursors and selections from other users
- **Conflict-free merging**: Automerge CRDT automatically resolves conflicts
- **Presence indicators**: Shows who's editing what
- **Version history**: Track all changes with timestamps
- **Offline editing**: Changes sync when reconnected
- **Undo/redo**: Per-user undo history

**Collaborator Indicators:**
```
[Alice] ← Blue cursor and selection
[Bob]   ← Green cursor and selection
[You]   ← Your cursor
```

### Markdown Viewer (Browser-Like)

**Rendered View:**
```
┌────────────────────────────────────────────────────────────┐
│ 🌐 Preview: home.md              [✏️ Edit] [🔄 Refresh]    │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  # Welcome to My Portfolio                                 │
│                                                            │
│  I'm a software engineer passionate about building         │
│  decentralized systems and privacy-focused tools.          │
│                                                            │
│  ## Projects                                               │
│                                                            │
│  • Communitas: Decentralized collaboration platform        │
│  • DataViz: Data visualization toolkit                     │
│                                                            │
│  ## Contact                                                │
│                                                            │
│  Feel free to reach out via my [four-word identity]!       │
│                                                            │
│  Navigation: [Home] [About] [Projects] [Contact]           │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**Features:**
- GitHub-flavored markdown rendering
- Syntax highlighting for code blocks
- Embedded images and videos
- Internal links between documents
- External link handling
- Table rendering
- Math equations (KaTeX)
- Mermaid diagrams

### File Operations

**Upload Flow:**
```
1. Click [📤 Upload] button
2. Select file(s) from local system
3. System:
   - Encrypts file with ChaCha20-Poly1305
   - Chunks and applies FEC (Forward Error Correction)
   - Generates BLAKE3 content hash
   - Stores in appropriate virtual disk (Private/Public/Shared)
   - Syncs to network via QUIC
4. File appears in Files list
```

**Download Flow:**
```
1. Click file to download
2. System:
   - Fetches chunks from network (if not local)
   - Verifies BLAKE3 hash
   - Decrypts with user's keys
   - Reassembles file
3. File opens or downloads to user's system
```

**Sharing Flow:**
```
1. Right-click file → [Share]
2. Select recipients:
   - Individual contacts
   - Groups
   - Channels
3. System:
   - Grants access via group key
   - Updates ACL on virtual disk
   - Notifies recipients
4. Recipients see file in "Shared with Me"
```

---

## Website Publishing System

### Markdown-Based Website

**Concept**: Use `home.md` as the root document (like `index.html`) and render the entire website from interconnected markdown files.

**Directory Structure:**
```
/website
├─ home.md          ← Root document (website entry point)
├─ about.md         ← About page
├─ projects.md      ← Projects listing
├─ /blog
│  ├─ index.md      ← Blog index
│  ├─ post-1.md     ← Blog post
│  └─ post-2.md     ← Blog post
└─ /assets
   ├─ logo.png
   └─ styles.css    ← Optional custom CSS
```

**home.md Example:**
```markdown
# Welcome to My Portfolio

I'm Alice, a software engineer building decentralized systems.

## Navigation
- [About Me](about.md)
- [Projects](projects.md)
- [Blog](/blog/index.md)
- [Contact](#contact)

## Featured Projects

### Communitas
A decentralized collaboration platform combining WhatsApp, Dropbox,
Zoom, and Slack.

[Learn More](projects.md#communitas)

## Contact

Reach me at: `ocean-forest-moon-star`

---

*Last updated: 2025-10-09*
```

### Website Browser Interface

**Website Viewer:**
```
┌────────────────────────────────────────────────────────────┐
│ 🌐 ocean-forest-moon-star.communitas    [🔄] [⋮]          │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Welcome to My Portfolio                                   │
│  ================================                           │
│                                                            │
│  I'm Alice, a software engineer building decentralized     │
│  systems.                                                  │
│                                                            │
│  Navigation                                                │
│  • About Me                                                │
│  • Projects                                                │
│  • Blog                                                    │
│  • Contact                                                 │
│                                                            │
│  Featured Projects                                         │
│  ─────────────────                                         │
│                                                            │
│  Communitas                                                │
│  A decentralized collaboration platform...                 │
│                                                            │
│  [Learn More →]                                            │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**Navigation Features:**
- Click links to navigate between pages
- Browser-like back/forward buttons
- Bookmark favorite pages
- Share page URLs
- View page source (markdown)
- Edit page (if you have permission)

### Publishing Flow

**Step 1: Select Root Document**
```
┌─────────────────────────────────────────┐
│  Publish Website                        │
│                                         │
│  Select your home document:             │
│                                         │
│  📄 Files > Personal > Portfolio >      │
│      ☑ home.md                          │
│                                         │
│  This document will be your website's   │
│  entry point (like index.html)          │
│                                         │
│  [Back]                      [Next →]   │
└─────────────────────────────────────────┘
```

**Step 2: Configure Website**
```
┌─────────────────────────────────────────┐
│  Website Configuration                  │
│                                         │
│  Website Name                           │
│  [Alice's Portfolio]                    │
│                                         │
│  Description                            │
│  [Personal portfolio and blog]          │
│                                         │
│  Your website will be available at:     │
│  ocean-forest-moon-star.communitas      │
│                                         │
│  Access Control                         │
│  ● Public (anyone can view)             │
│  ○ Private (only you)                   │
│  ○ Shared (specific users/groups)       │
│                                         │
│  [← Back]                  [Publish →]  │
└─────────────────────────────────────────┘
```

**Step 3: Publish**
```
┌─────────────────────────────────────────┐
│  Publishing Website...                  │
│                                         │
│  ✅ Validating markdown files           │
│  ✅ Processing linked documents         │
│  ✅ Generating content hashes           │
│  ✅ Encrypting content                  │
│  ✅ Publishing to network               │
│  ✅ Updating identity record            │
│                                         │
│  Done! Your website is now live at:     │
│  ocean-forest-moon-star.communitas      │
│                                         │
│  [View Website]        [Done]           │
└─────────────────────────────────────────┘
```

### Website Management

**Website Dashboard:**
```
┌────────────────────────────────────────────────────────────┐
│ 🌐 Alice's Portfolio                                       │
├────────────────────────────────────────────────────────────┤
│ Status: 🟢 Published • Last updated: 2 hours ago           │
│ URL: ocean-forest-moon-star.communitas                     │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  PAGES (5)                                                 │
│  ├─ 📄 home.md (root)                                     │
│  ├─ 📄 about.md                                           │
│  ├─ 📄 projects.md                                        │
│  ├─ 📄 blog/index.md                                      │
│  └─ 📄 blog/post-1.md                                     │
│                                                            │
│  ANALYTICS (Last 30 days)                                  │
│  • Total Visits: 1,245                                     │
│  • Unique Visitors: 892                                    │
│  • Popular Pages:                                          │
│    1. home.md (478 visits)                                 │
│    2. projects.md (234 visits)                             │
│    3. about.md (189 visits)                                │
│                                                            │
│  ACTIONS                                                   │
│  [✏️ Edit Pages] [👁️ Preview] [📊 View Full Analytics]    │
│  [⚙️ Settings]   [🗑️ Unpublish]                           │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

---

## Collaborative CRDT Editing

### CRDT Architecture

**Implementation**: Using Automerge CRDT library

**Data Structure:**
```typescript
// Each document has an Automerge document
interface CRDTDocument {
  id: string;                    // Document ID
  type: 'markdown' | 'text';     // Document type
  content: Automerge.Text;       // CRDT text
  metadata: {
    title: string;
    createdAt: number;
    updatedAt: number;
    createdBy: string;           // Four-word identity
  };
  collaborators: {
    [fourWords: string]: {
      name: string;
      cursor: { line: number; column: number };
      selection: { start: number; end: number };
      lastSeen: number;
      color: string;             // Assigned color for UI
    };
  };
}
```

### Real-Time Editing Flow

**1. User Opens Document**
```
User → Open home.md
   ↓
System loads local Automerge document
   ↓
Connects to network via QUIC
   ↓
Syncs with other peers' changes
   ↓
Shows document with collaborator cursors
```

**2. User Makes Edit**
```
User types: "Hello World"
   ↓
Automerge records change as operation
   ↓
Update local state (optimistic)
   ↓
Broadcast change to all connected peers
   ↓
Other users see change in real-time
```

**3. Conflict Resolution**
```
Alice types "A" at position 10
Bob types "B" at position 10 (simultaneously)
   ↓
Both changes recorded as CRDT operations
   ↓
Automerge merges automatically
   ↓
Result: "AB" at position 10 (deterministic order)
   ↓
All users converge to same state
```

### Presence System

**Cursor Broadcasting:**
```typescript
// Every 100ms, broadcast cursor position
setInterval(() => {
  const cursor = editor.getCursor();
  const selection = editor.getSelection();

  broadcastPresence({
    fourWords: myIdentity,
    cursor: { line: cursor.line, col: cursor.ch },
    selection: selection,
    timestamp: Date.now()
  });
}, 100);
```

**Rendering Collaborators:**
```
Editor shows:
┌────────────────────────────────────────┐
│ # Document Title                       │
│                                        │
│ Some text here ← Alice                 │
│ More text ← Bob                        │
│                                        │
└────────────────────────────────────────┘

Colors:
- Alice: Blue cursor/selection
- Bob: Green cursor/selection
- You: Default cursor
```

### Offline Editing

**Scenario**: User edits document while offline

```
User offline
   ↓
Makes edits locally
   ↓
Automerge records all operations
   ↓
Operations stored in local queue
   ↓
User reconnects
   ↓
Sync queued operations with network
   ↓
Merge with any concurrent changes
   ↓
Document converges to consistent state
```

### Version History

**History View:**
```
┌────────────────────────────────────────────────────────────┐
│ 📄 home.md - Version History                               │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  📅 October 9, 2025                                        │
│                                                            │
│  • 16:45 - You edited line 23 (Added contact section)     │
│  • 16:30 - Alice edited line 15 (Updated projects)        │
│  • 15:20 - Bob edited line 8 (Fixed typo)                 │
│                                                            │
│  📅 October 8, 2025                                        │
│                                                            │
│  • 14:10 - You created document                           │
│                                                            │
│  [View Diff] [Restore Version] [Export History]           │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**Diff View:**
```
┌────────────────────────────────────────────────────────────┐
│ Comparing: v12 (16:30) ↔ v13 (16:45)                      │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  # Welcome to My Portfolio                                 │
│                                                            │
│  ## Projects                                               │
│                                                            │
│  - **Communitas**: Decentralized... ← UNCHANGED           │
│  + **DataViz**: Data visualization  ← ADDED (You, 16:45)  │
│                                                            │
│  ## Contact                                                │
│                                                            │
│  - Email: alice@example.com        ← REMOVED (You, 16:45) │
│  + Four-words: ocean-forest-...    ← ADDED (You, 16:45)   │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### CRDT Sync Protocol

**Message Types:**
```typescript
// Sync messages between peers
interface SyncMessage {
  type: 'sync-request' | 'sync-response' | 'change';
  documentId: string;
  sender: string;        // Four-word identity
  payload: {
    changes?: Automerge.Change[];
    state?: Automerge.SyncState;
  };
}
```

**Sync Flow:**
```
Peer A                          Peer B
   |                               |
   |--- sync-request ------------->|
   |    (my current state)         |
   |                               |
   |<-- sync-response -------------|
   |    (missing changes)          |
   |                               |
   |--- change ------------------->|
   |    (new edit)                 |
   |                               |
   |<-- change --------------------|
   |    (concurrent edit)          |
   |                               |
   v                               v
Both converge to same state
```

---

## Testing Strategy

### Testing Objectives

1. **Verify UI functionality**: All components work as designed
2. **Test authentication flow**: Registration, login, passkey enrollment
3. **Validate entity management**: Create, read, update, delete entities
4. **Confirm communication**: Messaging, calls, screen sharing
5. **Test file operations**: Upload, download, share, edit
6. **Verify CRDT sync**: Multi-user collaborative editing
7. **Test website publishing**: Publish, view, update websites
8. **Check network connectivity**: P2P connections via saorsa-core

### Testing Tools

**Chrome DevTools MCP:**
- Direct browser access and automation
- Console inspection
- Network monitoring
- Performance profiling
- DOM manipulation
- Screenshot capture

**Communitas Bridge:**
- HTTP/REST interface for browser testing
- Connects browser to saorsa-core P2P network
- Endpoints for all core operations
- Real network integration testing

### Test Environment Setup

**Architecture:**
```
┌─────────────────────┐       ┌─────────────────────┐
│   Browser Instance  │       │   Browser Instance  │
│   (User A - Alice)  │       │   (User B - Bob)    │
└─────────┬───────────┘       └─────────┬───────────┘
          │ HTTP/REST                   │ HTTP/REST
          │                             │
┌─────────▼───────────┐       ┌─────────▼───────────┐
│  Bridge Server A    │       │  Bridge Server B    │
│  (localhost:3030)   │       │  (localhost:3031)   │
└─────────┬───────────┘       └─────────┬───────────┘
          │ Rust IPC                    │ Rust IPC
          │                             │
┌─────────▼───────────┐       ┌─────────▼───────────┐
│   Saorsa Core A     │◄──────┤   Saorsa Core B     │
│   (P2P Network)     │ QUIC  │   (P2P Network)     │
└─────────────────────┘       └─────────────────────┘
```

**Setup Steps:**

1. **Start Bridge Servers**
```bash
# Terminal 1: Bridge for Alice
cd communitas-bridge
BRIDGE_PORT=3030 cargo run

# Terminal 2: Bridge for Bob
cd communitas-bridge
BRIDGE_PORT=3031 cargo run
```

2. **Build Frontend**
```bash
npm run build
```

3. **Serve Frontend**
```bash
# Terminal 3: Serve for Alice
cd dist
python3 -m http.server 8080

# Terminal 4: Serve for Bob
cd dist
python3 -m http.server 8081
```

4. **Open Chrome DevTools MCP**
```bash
# Terminal 5: Chrome DevTools MCP
# Automated via test scripts
```

---

## Multi-Instance Testing

### Test Scenarios

#### Scenario 1: User Registration & Connection

**Objective**: Create two users and establish P2P connection

**Test Steps:**

1. **Alice Registers (Browser 1: localhost:8080)**
```javascript
// Chrome DevTools MCP - Browser 1
await page.goto('http://localhost:8080');
await page.click('[data-testid="create-identity"]');
await page.fill('[name="name"]', 'Alice');
await page.click('[data-testid="register-button"]');

// Verify four-word identity displayed
const fourWords = await page.textContent('[data-testid="four-words"]');
console.log('Alice identity:', fourWords); // e.g., "ocean-forest-moon-star"

// Initialize via bridge
await fetch('http://localhost:3030/api/core/initialize', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    four_words: fourWords,
    display_name: 'Alice',
    device_name: 'Browser Test A'
  })
});
```

2. **Bob Registers (Browser 2: localhost:8081)**
```javascript
// Chrome DevTools MCP - Browser 2
await page.goto('http://localhost:8081');
await page.click('[data-testid="create-identity"]');
await page.fill('[name="name"]', 'Bob');
await page.click('[data-testid="register-button"]');

const fourWords = await page.textContent('[data-testid="four-words"]');
console.log('Bob identity:', fourWords); // e.g., "mountain-river-star-tree"

// Initialize via bridge
await fetch('http://localhost:3031/api/core/initialize', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    four_words: fourWords,
    display_name: 'Bob',
    device_name: 'Browser Test B'
  })
});
```

3. **Verify P2P Connection**
```javascript
// Check Alice's network status
const statusA = await fetch('http://localhost:3030/api/network/status');
const dataA = await statusA.json();
console.log('Alice peers:', dataA.connected_peers); // Should include Bob

// Check Bob's network status
const statusB = await fetch('http://localhost:3031/api/network/status');
const dataB = await statusB.json();
console.log('Bob peers:', dataB.connected_peers); // Should include Alice
```

**Expected Results:**
- ✅ Both users registered successfully
- ✅ Four-word identities generated
- ✅ P2P connection established via QUIC
- ✅ Network status shows 1 connected peer each

---

#### Scenario 2: Create Channel & Send Messages

**Objective**: Alice creates a channel, adds Bob, and they exchange messages

**Test Steps:**

1. **Alice Creates Channel**
```javascript
// Browser 1 (Alice)
await page.click('[data-testid="new-channel"]');
await page.fill('[name="channelName"]', 'general');
await page.fill('[name="description"]', 'General discussion');
await page.click('[data-testid="create-channel-button"]');

// Via bridge
const channelResp = await fetch('http://localhost:3030/api/channels', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'general',
    description: 'General discussion'
  })
});
const channel = await channelResp.json();
console.log('Channel created:', channel.id);
```

2. **Alice Invites Bob**
```javascript
// Get Bob's four-word identity
const bobFourWords = 'mountain-river-star-tree';

// Invite Bob to channel
await fetch(`http://localhost:3030/api/channels/${channel.id}/members`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    four_words: bobFourWords
  })
});
```

3. **Bob Accepts & Views Channel**
```javascript
// Browser 2 (Bob) - Check for invitation
const channelsResp = await fetch('http://localhost:3031/api/channels');
const channels = await channelsResp.json();
console.log('Bob sees channels:', channels); // Should include 'general'

// Bob clicks channel in UI
await page.click(`[data-channel-id="${channel.id}"]`);
```

4. **Alice Sends Message**
```javascript
// Browser 1 (Alice)
await page.fill('[data-testid="message-input"]', 'Hello Bob!');
await page.click('[data-testid="send-button"]');

// Via bridge
await fetch(`http://localhost:3030/api/channels/${channel.id}/messages`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    content: 'Hello Bob!',
    recipients: [bobFourWords]
  })
});
```

5. **Bob Receives & Replies**
```javascript
// Browser 2 (Bob) - Verify message received
const messagesResp = await fetch(`http://localhost:3031/api/channels/${channel.id}/messages`);
const messages = await messagesResp.json();
console.log('Messages:', messages); // Should include Alice's message

// Verify in UI
const messageText = await page.textContent('[data-message-id]:last-child .message-content');
console.log('Bob sees:', messageText); // "Hello Bob!"

// Bob replies
await page.fill('[data-testid="message-input"]', 'Hi Alice!');
await page.click('[data-testid="send-button"]');
```

6. **Alice Receives Reply**
```javascript
// Browser 1 (Alice) - Verify reply received
await page.waitForSelector('[data-message-content="Hi Alice!"]');
const replyText = await page.textContent('[data-message-id]:last-child .message-content');
console.log('Alice sees:', replyText); // "Hi Alice!"
```

**Expected Results:**
- ✅ Channel created by Alice
- ✅ Bob invited and sees channel
- ✅ Alice's message delivered to Bob
- ✅ Bob's reply delivered to Alice
- ✅ Both users see message history

---

#### Scenario 3: Collaborative Document Editing (CRDT)

**Objective**: Alice and Bob collaboratively edit the same markdown document in real-time

**Test Steps:**

1. **Alice Creates Document**
```javascript
// Browser 1 (Alice)
await page.click('[data-testid="new-file"]');
await page.fill('[name="fileName"]', 'home.md');
await page.select('[name="fileType"]', 'markdown');
await page.click('[data-testid="create-file-button"]');

// Open editor
await page.click('[data-file-name="home.md"]');

// Via bridge
const fileResp = await fetch('http://localhost:3030/api/files', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'home.md',
    type: 'markdown',
    content: '# Welcome\n\nThis is a test document.'
  })
});
const file = await fileResp.json();
console.log('File created:', file.id);
```

2. **Alice Shares Document with Bob**
```javascript
// Share file
await fetch(`http://localhost:3030/api/files/${file.id}/share`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    recipients: ['mountain-river-star-tree']
  })
});
```

3. **Bob Opens Shared Document**
```javascript
// Browser 2 (Bob)
await page.click('[data-testid="shared-files"]');
await page.click('[data-file-name="home.md"]');

// Verify Bob sees Alice's presence
const presenceIndicator = await page.textContent('[data-testid="collaborators"]');
console.log('Collaborators:', presenceIndicator); // Should show "Alice"
```

4. **Both Users Edit Simultaneously**
```javascript
// Browser 1 (Alice) - Edit line 3
await page.click('.editor-line[data-line="3"]');
await page.keyboard.type('Alice was here!');

// Via bridge - Alice's edit
await fetch(`http://localhost:3030/api/files/${file.id}/edit`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    operation: 'insert',
    position: { line: 3, column: 0 },
    text: 'Alice was here!\n'
  })
});

// Browser 2 (Bob) - Edit line 5 (different location)
await page.click('.editor-line[data-line="5"]');
await page.keyboard.type('Bob was here too!');

// Via bridge - Bob's edit
await fetch(`http://localhost:3031/api/files/${file.id}/edit`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    operation: 'insert',
    position: { line: 5, column: 0 },
    text: 'Bob was here too!\n'
  })
});
```

5. **Verify CRDT Sync**
```javascript
// Browser 1 (Alice) - Should see Bob's edit
await page.waitForSelector('.editor-line[data-line="5"]:has-text("Bob was here too!")');
const aliceView = await page.textContent('.editor-content');
console.log('Alice sees:', aliceView);

// Browser 2 (Bob) - Should see Alice's edit
await page.waitForSelector('.editor-line[data-line="3"]:has-text("Alice was here!")');
const bobView = await page.textContent('.editor-content');
console.log('Bob sees:', bobView);

// Verify both have identical content
const aliceContent = await fetch(`http://localhost:3030/api/files/${file.id}/content`);
const bobContent = await fetch(`http://localhost:3031/api/files/${file.id}/content`);
const aliceText = await aliceContent.text();
const bobText = await bobContent.text();

console.assert(aliceText === bobText, 'CRDT sync successful - content matches!');
```

6. **Test Presence Indicators**
```javascript
// Browser 1 (Alice) - Verify Bob's cursor visible
const bobCursor = await page.$('[data-collaborator="mountain-river-star-tree"] .cursor');
console.log('Alice sees Bob cursor:', bobCursor !== null); // Should be true

// Browser 2 (Bob) - Verify Alice's cursor visible
const aliceCursor = await page.$('[data-collaborator="ocean-forest-moon-star"] .cursor');
console.log('Bob sees Alice cursor:', aliceCursor !== null); // Should be true
```

**Expected Results:**
- ✅ Document created and shared
- ✅ Both users can edit simultaneously
- ✅ Edits sync in real-time via CRDT
- ✅ No conflicts - both see identical content
- ✅ Presence indicators show collaborators
- ✅ Cursor positions visible

---

#### Scenario 4: Website Publishing & Viewing

**Objective**: Alice publishes a website from markdown, Bob views it

**Test Steps:**

1. **Alice Creates Website Structure**
```javascript
// Browser 1 (Alice)
// Create home.md (already created in Scenario 3)

// Create about.md
await fetch('http://localhost:3030/api/files', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'about.md',
    type: 'markdown',
    content: '# About Me\n\nI\'m Alice, a software engineer.\n\n[Back to Home](home.md)'
  })
});

// Create projects.md
await fetch('http://localhost:3030/api/files', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'projects.md',
    type: 'markdown',
    content: '# Projects\n\n- **Communitas**: Collaboration platform\n\n[Back to Home](home.md)'
  })
});

// Update home.md with navigation
await fetch(`http://localhost:3030/api/files/${file.id}/edit`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    operation: 'replace',
    content: `# Welcome to My Portfolio

I'm Alice, a software engineer.

## Navigation
- [About Me](about.md)
- [Projects](projects.md)
- [Contact](#contact)

## Contact
Reach me at: \`ocean-forest-moon-star\``
  })
});
```

2. **Alice Publishes Website**
```javascript
// Browser 1 (Alice)
await page.click('[data-testid="publish-website"]');
await page.fill('[name="websiteName"]', 'Alice\'s Portfolio');
await page.click('[data-testid="select-root-document"]');
await page.click('[data-file-name="home.md"]');
await page.click('[data-testid="publish-button"]');

// Via bridge
const publishResp = await fetch('http://localhost:3030/api/websites/publish', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'Alice\'s Portfolio',
    root_document: file.id,
    access: 'public'
  })
});
const website = await publishResp.json();
console.log('Website published:', website.url); // ocean-forest-moon-star.communitas
```

3. **Bob Views Website**
```javascript
// Browser 2 (Bob)
const websiteUrl = 'http://localhost:3031/api/websites/view/ocean-forest-moon-star';
await page.goto(websiteUrl);

// Verify content loaded
const title = await page.textContent('h1');
console.log('Page title:', title); // "Welcome to My Portfolio"

// Verify navigation links work
const aboutLink = await page.$('a[href="about.md"]');
console.log('About link found:', aboutLink !== null);
```

4. **Bob Navigates Website**
```javascript
// Browser 2 (Bob) - Click About link
await page.click('a[href="about.md"]');
await page.waitForSelector('h1:has-text("About Me")');

// Verify About page loaded
const aboutTitle = await page.textContent('h1');
console.log('About page title:', aboutTitle); // "About Me"

// Click Projects link
await page.click('a[href="../projects.md"]');
await page.waitForSelector('h1:has-text("Projects")');

// Click Back to Home
await page.click('a[href="home.md"]');
await page.waitForSelector('h1:has-text("Welcome to My Portfolio")');
```

5. **Alice Updates Website (Live)**
```javascript
// Browser 1 (Alice) - Edit home.md while Bob is viewing
await page.click('[data-file-name="home.md"]');
await page.keyboard.type('\n\n## Latest News\n\nWebsite just launched!');

// Via bridge
await fetch(`http://localhost:3030/api/files/${file.id}/edit`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    operation: 'append',
    text: '\n\n## Latest News\n\nWebsite just launched!'
  })
});
```

6. **Bob Sees Update**
```javascript
// Browser 2 (Bob) - Refresh page
await page.reload();
await page.waitForSelector('h2:has-text("Latest News")');

// Verify new section visible
const newsSection = await page.textContent('h2:last-of-type');
console.log('Bob sees update:', newsSection); // "Latest News"
```

**Expected Results:**
- ✅ Website published with markdown files
- ✅ Bob can access website via Alice's identity URL
- ✅ Navigation between pages works
- ✅ Content renders correctly
- ✅ Live updates propagate to viewers
- ✅ No DNS required - identity-based addressing

---

#### Scenario 5: File Upload & Download

**Objective**: Test binary file handling (images, PDFs, etc.)

**Test Steps:**

1. **Alice Uploads Image**
```javascript
// Browser 1 (Alice)
const imageFile = await page.$('[data-testid="file-input"]');
await imageFile.setInputFiles('./test-assets/logo.png');
await page.click('[data-testid="upload-button"]');

// Via bridge - Upload image
const formData = new FormData();
formData.append('file', fs.createReadStream('./test-assets/logo.png'));
formData.append('name', 'logo.png');

const uploadResp = await fetch('http://localhost:3030/api/files/upload', {
  method: 'POST',
  body: formData
});
const uploadedFile = await uploadResp.json();
console.log('File uploaded:', uploadedFile.id);
```

2. **Alice Shares Image with Bob**
```javascript
// Browser 1 (Alice)
await page.rightClick('[data-file-name="logo.png"]');
await page.click('[data-action="share"]');
await page.fill('[data-testid="recipient-search"]', 'Bob');
await page.click('[data-user="mountain-river-star-tree"]');
await page.click('[data-testid="share-confirm"]');

// Via bridge
await fetch(`http://localhost:3030/api/files/${uploadedFile.id}/share`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    recipients: ['mountain-river-star-tree']
  })
});
```

3. **Bob Downloads Image**
```javascript
// Browser 2 (Bob)
await page.click('[data-testid="shared-files"]');
await page.click('[data-file-name="logo.png"]');
await page.click('[data-action="download"]');

// Via bridge - Download file
const downloadResp = await fetch(`http://localhost:3031/api/files/${uploadedFile.id}/download`);
const fileBlob = await downloadResp.blob();
console.log('File downloaded:', fileBlob.size, 'bytes');

// Verify file integrity (hash comparison)
const originalHash = uploadedFile.hash;
const downloadedHash = await computeHash(fileBlob);
console.assert(originalHash === downloadedHash, 'File integrity verified!');
```

4. **Bob Uses Image in Document**
```javascript
// Browser 2 (Bob) - Edit markdown to include image
await page.click('[data-file-name="home.md"]');
await page.keyboard.type('\n\n![Logo](logo.png)\n');

// Via bridge
await fetch(`http://localhost:3031/api/files/${file.id}/edit`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    operation: 'append',
    text: '\n\n![Logo](logo.png)\n'
  })
});

// Verify image renders in preview
await page.click('[data-testid="preview-button"]');
await page.waitForSelector('img[alt="Logo"]');
const imgSrc = await page.getAttribute('img[alt="Logo"]', 'src');
console.log('Image rendered in preview:', imgSrc);
```

**Expected Results:**
- ✅ Image uploaded successfully
- ✅ File encrypted and chunked
- ✅ Bob receives shared file notification
- ✅ Bob downloads file with correct hash
- ✅ Image embeds in markdown preview
- ✅ Binary file integrity preserved

---

#### Scenario 6: Voice/Video Call Testing

**Objective**: Establish voice/video call between Alice and Bob

**Test Steps:**

1. **Alice Initiates Call**
```javascript
// Browser 1 (Alice)
await page.click('[data-contact="mountain-river-star-tree"]');
await page.click('[data-action="start-call"]');
await page.click('[data-call-type="video"]');

// Via bridge - Initiate call
const callResp = await fetch('http://localhost:3030/api/calls/initiate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    recipient: 'mountain-river-star-tree',
    type: 'video'
  })
});
const call = await callResp.json();
console.log('Call initiated:', call.id);
```

2. **Bob Receives Call**
```javascript
// Browser 2 (Bob) - Incoming call notification
await page.waitForSelector('[data-testid="incoming-call"]');
const callerName = await page.textContent('[data-caller]');
console.log('Incoming call from:', callerName); // "Alice"

// Bob answers
await page.click('[data-action="answer-call"]');

// Via bridge
await fetch(`http://localhost:3031/api/calls/${call.id}/answer`, {
  method: 'POST'
});
```

3. **Verify Media Streams**
```javascript
// Browser 1 (Alice) - Verify video/audio
await page.waitForSelector('[data-testid="remote-video"]');
const remoteVideo = await page.$('[data-testid="remote-video"]');
const isPlaying = await page.evaluate((video) => {
  return !video.paused && !video.ended && video.readyState > 2;
}, remoteVideo);
console.log('Alice sees Bob video:', isPlaying);

// Browser 2 (Bob) - Verify video/audio
await page.waitForSelector('[data-testid="remote-video"]');
const remoteVideoBob = await page.$('[data-testid="remote-video"]');
const isPlayingBob = await page.evaluate((video) => {
  return !video.paused && !video.ended && video.readyState > 2;
}, remoteVideoBob);
console.log('Bob sees Alice video:', isPlayingBob);
```

4. **Test Call Controls**
```javascript
// Browser 1 (Alice) - Mute audio
await page.click('[data-action="toggle-audio"]');
const isMuted = await page.getAttribute('[data-action="toggle-audio"]', 'data-muted');
console.log('Alice muted:', isMuted === 'true');

// Browser 2 (Bob) - Should see muted indicator
await page.waitForSelector('[data-user="ocean-forest-moon-star"][data-audio="muted"]');
const aliceMutedIndicator = await page.$('[data-user="ocean-forest-moon-star"][data-audio="muted"]');
console.log('Bob sees Alice muted:', aliceMutedIndicator !== null);

// Alice unmutes
await page.click('[data-action="toggle-audio"]');
```

5. **Test Screen Sharing**
```javascript
// Browser 1 (Alice) - Start screen share
await page.click('[data-action="share-screen"]');
await page.click('[data-share-option="window"]');

// Via bridge
await fetch(`http://localhost:3030/api/calls/${call.id}/screen-share`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    type: 'window'
  })
});

// Browser 2 (Bob) - Should see screen share
await page.waitForSelector('[data-testid="screen-share-video"]');
const screenShareVideo = await page.$('[data-testid="screen-share-video"]');
console.log('Bob sees Alice screen share:', screenShareVideo !== null);
```

6. **End Call**
```javascript
// Browser 1 (Alice) - Hang up
await page.click('[data-action="end-call"]');

// Via bridge
await fetch(`http://localhost:3030/api/calls/${call.id}/end`, {
  method: 'POST'
});

// Browser 2 (Bob) - Call ended
await page.waitForSelector('[data-testid="call-ended"]');
const callEndedMessage = await page.textContent('[data-testid="call-ended"]');
console.log('Call ended message:', callEndedMessage);
```

**Expected Results:**
- ✅ Call initiated successfully
- ✅ Bob receives call notification
- ✅ Both users connected via QUIC
- ✅ Audio/video streams established
- ✅ Mute controls work correctly
- ✅ Screen sharing functional
- ✅ Call terminates cleanly

---

### Test Execution Plan

**Phase 1: Setup (15 minutes)**
- Start bridge servers for Alice and Bob
- Build and serve frontend
- Initialize Chrome DevTools MCP
- Create test utilities

**Phase 2: Authentication Tests (10 minutes)**
- Scenario 1: User Registration & Connection
- Verify: Identity creation, keyring storage, P2P connection

**Phase 3: Messaging Tests (15 minutes)**
- Scenario 2: Create Channel & Send Messages
- Verify: Channel creation, invitations, message delivery

**Phase 4: CRDT Tests (20 minutes)**
- Scenario 3: Collaborative Document Editing
- Verify: Real-time sync, conflict resolution, presence

**Phase 5: Website Tests (15 minutes)**
- Scenario 4: Website Publishing & Viewing
- Verify: Markdown rendering, navigation, live updates

**Phase 6: File Tests (10 minutes)**
- Scenario 5: File Upload & Download
- Verify: Binary file handling, encryption, integrity

**Phase 7: Communication Tests (20 minutes)**
- Scenario 6: Voice/Video Call Testing
- Verify: WebRTC, QUIC transport, media streams

**Total Estimated Time: ~2 hours**

---

### Test Automation Script

**Example Test Runner:**
```javascript
// test-runner.js
const { chromium } = require('playwright');

async function runTests() {
  // Launch browsers
  const browserAlice = await chromium.launch();
  const browserBob = await chromium.launch();

  const contextAlice = await browserAlice.newContext();
  const contextBob = await browserBob.newContext();

  const pageAlice = await contextAlice.newPage();
  const pageBob = await contextBob.newPage();

  // Run test scenarios
  console.log('🧪 Running Scenario 1: User Registration & Connection');
  await testScenario1(pageAlice, pageBob);

  console.log('🧪 Running Scenario 2: Create Channel & Send Messages');
  await testScenario2(pageAlice, pageBob);

  console.log('🧪 Running Scenario 3: Collaborative Document Editing');
  await testScenario3(pageAlice, pageBob);

  console.log('🧪 Running Scenario 4: Website Publishing & Viewing');
  await testScenario4(pageAlice, pageBob);

  console.log('🧪 Running Scenario 5: File Upload & Download');
  await testScenario5(pageAlice, pageBob);

  console.log('🧪 Running Scenario 6: Voice/Video Call Testing');
  await testScenario6(pageAlice, pageBob);

  // Cleanup
  await browserAlice.close();
  await browserBob.close();

  console.log('✅ All tests completed!');
}

runTests().catch(console.error);
```

---

## Conclusion

This comprehensive UI design document covers:

✅ **Complete Authentication Flow** - Passkey-first with optional passwords
✅ **All Entity Types** - Channels, Projects, Files, Contacts, Groups, Websites
✅ **Communication Features** - Text, Voice, Video, Screen Sharing
✅ **File Management** - Upload, Download, Share, Collaborative Editing
✅ **Website Publishing** - Markdown-based, DNS-free, identity-bound
✅ **CRDT Collaboration** - Real-time editing with conflict-free merging
✅ **Testing Strategy** - Multi-instance testing via Chrome DevTools MCP
✅ **Bridge Integration** - Browser-to-network via communitas-bridge

### Next Steps

1. **Review & Approve** - Review this document and provide feedback
2. **Implement Components** - Build UI components according to design
3. **Integrate CRDT** - Add Automerge to existing file/website features
4. **Setup Testing** - Configure bridge servers and test environment
5. **Execute Tests** - Run all test scenarios and verify functionality
6. **Iterate** - Refine based on test results and user feedback

---

**Status**: Ready for Review
**Last Updated**: 2025-10-09
**Version**: 1.0
