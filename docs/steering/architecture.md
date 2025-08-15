# Communitas Architecture

## Overview

Communitas is a desktop application built with Tauri, combining a React frontend with a Rust backend for cross-platform desktop deployment. The architecture supports distributed storage with Forward Error Correction, entity-based isolation, and integrated web publishing capabilities.

## Architecture Pattern

```
┌─────────────────────────────────────────────────────────────┐
│                    Desktop Application                       │
├─────────────────────────┬───────────────────────────────────┤
│      Frontend           │           Backend                 │
│      (React)            │           (Rust)                  │
│                         │                                   │
│  ┌─────────────────┐   │   ┌─────────────────────────────┐ │
│  │ React Components │   │   │      Tauri Core             │ │
│  │ - UI Logic       │   │   │  - System Integration       │ │
│  │ - State Mgmt     │   │   │  - Storage & FEC            │ │
│  │ - Markdown UI    │   │   │  - P2P Networking           │ │
│  └─────────────────┘   │   └─────────────────────────────┘ │
│           │             │              │                   │
│           └─────────────┼──────────────┘                   │
│                         │      IPC Bridge                  │
└─────────────────────────┴───────────────────────────────────┘
```

## Technical Architecture

### Desktop Application Architecture

Communitas is built as a desktop application using the Tauri framework:

```
┌─────────────────────────────────────────┐
│         React Frontend (UI)             │
│    TypeScript + Material UI + Vite      │
│    Markdown Editor/Browser + Yjs        │
├─────────────────────────────────────────┤
│         Tauri IPC Bridge                │
│    (Secure Command Interface)           │
├─────────────────────────────────────────┤
│         Rust Backend                    │
│    (Business Logic & System APIs)       │
│    Storage + FEC + Encryption           │
├─────────────────────────────────────────┤
│         System Integration              │
│    (OS APIs, File System, Keyring)      │
└─────────────────────────────────────────┘
```

### Component Architecture

#### Frontend Components
- **React UI Layer**: Component-based user interface
- **State Management**: React Context for application state
- **Routing**: React Router for navigation
- **IPC Communication**: Tauri API for backend interaction
- **Markdown Browser**: Integrated markdown viewer with four-word address navigation
- **Markdown Editor**: Collaborative editing with Yjs CRDTs
- **Storage UI**: File management and web publishing interface

#### Backend Components
- **Tauri Core**: Application framework and window management
- **Rust Services**: Business logic and data processing
- **System Integration**: OS-specific APIs and features
- **Security Layer**: Multi-layer encryption (pre-FEC, post-FEC, DHT)
- **Storage Engine**: Entity-based storage with FEC redundancy
- **Saorsa FEC**: Reed-Solomon error correction (60% recovery threshold)
- **Web Publishing**: Markdown-based website hosting per entity

### Data Flow

#### Standard Operations
```
User Interaction
      ↓
React Component
      ↓
Tauri Command (IPC)
      ↓
Rust Handler
      ↓
Business Logic
      ↓
Data Storage/Network
      ↓
Response
      ↓
UI Update
```

#### Storage Flow with FEC
```
File Upload
      ↓
[First Encryption Layer]
      ↓
Encrypted Data
      ↓
[Forward Error Correction]
(6 data + 4 parity shards)
      ↓
FEC Shards
      ↓
[Second Encryption Layer]
      ↓
Encrypted Shards
      ↓
[DHT Distribution]
      ↓
Distributed Storage
```

#### Web Publishing Flow
```
Markdown Content
      ↓
Entity Web Directory
      ↓
Encryption + FEC
      ↓
DHT Storage
      ↓
Four-Word Address Access
(entity-address/web/path)
      ↓
Markdown Browser Rendering
```

## Storage Architecture

### Entity-Based Storage Hierarchy
```
Storage System
├── Individual Users (Personal Storage)
│   ├── Files
│   ├── Documents
│   └── Web Content
├── Groups (Shared Storage with FEC)
│   ├── Group Files
│   ├── Collaborative Documents
│   └── Group Website
├── Organizations
│   ├── Organization Storage
│   ├── Organizational Groups
│   ├── Channels
│   └── Projects
└── All Entities Support:
    ├── Voice/Video Calling
    ├── Messaging
    ├── File Storage
    └── Web Publishing
```

### Forward Error Correction Configuration
- **Group Storage**: 60% recovery threshold (6 data + 4 parity shards)
- **Individual Storage**: Direct storage without FEC
- **Distribution**: Shards distributed across all group members
- **Recovery**: Only 6 out of 10 shards needed for full recovery

## P2P Network Architecture

### Saorsa Core Integration
- **DHT-based Discovery**: Kademlia distributed hash table
- **Geographic Routing**: Location-aware peer selection
- **NAT Traversal**: Automatic hole punching
- **Bootstrap Nodes**: Initial network connection points

### Storage Distribution
- **Entity-Based Isolation**: Each entity has independent storage
- **FEC Distribution**: Shards distributed across group members
- **Recovery Threshold**: 60% of members needed for data recovery
- **Redundancy**: 40% of nodes can be offline without data loss

## Security Architecture

### Multi-Layer Encryption
1. **Pre-FEC Encryption**: Protects data before splitting
   - ChaCha20-Poly1305 for content encryption
   - Entity-specific keys

2. **Post-FEC Encryption**: Protects individual shards
   - Per-shard encryption keys
   - Prevents unauthorized shard access

3. **Transport Security**: TLS for network communication
   - Secure shard distribution
   - Protected DHT operations

4. **At-Rest Encryption**: Local storage protection
   - Platform keyring integration
   - Secure credential storage

### Authentication & Access Control
- **Digital Signatures**: Ed25519 for authentication
- **Permission System**: Read/Write/Admin/WebPublish roles
- **Entity Isolation**: Cryptographic separation between entities

## Web Publishing System

### Markdown-Based Websites
Each entity can publish a website using markdown files:
- **Home Page**: `home.md` serves as the index page
- **Navigation**: Internal links between markdown files
- **Media Support**: Images embedded in markdown (video support planned)
- **Access**: Via four-word addresses (e.g., `ocean-forest-moon-star/web/home.md`)

### Markdown Browser Features
- **Address Bar**: Four-word address navigation
- **Rendering**: Markdown to HTML conversion
- **Link Resolution**: Internal and cross-entity links
- **Media Loading**: Secure image retrieval from DHT
- **Navigation**: Back/forward/refresh functionality

## Technology Integration Points

### Frontend Integration
- React components with TypeScript
- Material UI design system
- Tauri API for IPC communication
- WebView for rendering
- Markdown editor/browser components
- Yjs for collaborative editing
- File management UI for storage operations

### Backend Integration
- Rust async/await patterns
- Tokio runtime for concurrency
- SQLite for local storage
- System keyring for credentials
- Saorsa FEC for error correction
- Blake3 for content addressing
- DHT for distributed storage
- Reed-Solomon codec implementation

### IPC Communication

Communication between frontend and backend uses Tauri's IPC system:

```typescript
// Frontend calls backend
import { invoke } from '@tauri-apps/api/tauri';
const result = await invoke('store_file', { 
  entityId: 'ocean-forest-moon-star',
  content: fileData 
});
```

```rust
// Backend command handler
#[tauri::command]
async fn store_file(
    entity_id: String,
    content: Vec<u8>
) -> Result<FileId, String> {
    // Apply encryption, FEC, and DHT storage
    storage_service.store_with_fec(entity_id, content).await
}
```

## Platform Targets

Supported platforms via Tauri:
- Windows (x64, ARM64)
- macOS (Intel, Apple Silicon)
- Linux (x64, ARM64)

## State Management

- **Frontend**: React hooks and context for UI state
- **Backend**: Rust ownership model for data integrity
- **Storage State**: Entity-based isolation with FEC redundancy
- **Communication**: Async IPC via Tauri bridge
- **Persistence**: Local SQLite + distributed DHT storage

## Security Boundaries

1. **Web Content**: Sandboxed React application
2. **IPC Layer**: Controlled command interface with validation
3. **Storage Layer**: Multi-layer encryption with FEC
4. **System Layer**: Rust backend with secure OS integration
5. **Network Layer**: P2P communication with end-to-end encryption

## Performance Characteristics

### Storage Performance
- **Upload**: Parallel shard distribution to group members
- **Download**: Parallel shard collection (60% threshold)
- **Cache**: LRU caching for frequently accessed content
- **Recovery**: Reed-Solomon enables data recovery with 40% node loss

### Web Publishing Performance
- **Page Load**: <500ms local, <2s remote
- **Content Caching**: Aggressive caching for home.md
- **Predictive Loading**: Preload linked pages
- **Bandwidth Management**: Rate limiting for large files

This architecture provides a robust foundation for decentralized community collaboration with secure storage, web publishing, and resilient data distribution through Forward Error Correction.