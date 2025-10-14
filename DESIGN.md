# Communitas — Design & Architecture

**Version**: 2.0 • **Date**: 2025-10-11 • **Status**: Current Implementation

**Communitas is a desktop and mobile-first, post-quantum collaboration platform** that enables secure communication, file sharing, and collaboration even during internet disruptions. Using gossip-based P2P networking and mesh capabilities, users can connect to their contacts on any reachable network, making the system highly partition-tolerant and resilient.

> **Implementation Guide**: See `STORYBOARD.md` for complete UI components and `STORYBOARD_V2.md` for day-by-day implementation checklist.
> **Interactive Prototype**: Open `storyboard-canvas-v2.html` in a browser to see the full UI design.

---

## **🎯 Core Principles**

### **Desktop & Mobile Native**
Communitas is **NOT a web application**. It is a **native desktop and mobile application** built with:
- **Desktop**: Tauri v2 framework with React UI
- **Mobile**: React Native with native P2P bridges (future)
- **Architecture**: Rust backend with TypeScript frontend
- **Distribution**: Self-contained binaries with no web dependencies

### **Gossip-Based P2P Mesh Networking**
The platform uses **saorsa-gossip** overlay networking for decentralized, partition-tolerant communication:
- **No Central Servers**: Direct peer-to-peer connections via QUIC
- **Mesh Capability**: Connect to contacts on ANY reachable network (local, internet, cellular, Bluetooth)
- **Partition Tolerance**: Continue operating during internet outages
- **Offline-First**: All data stored locally with eventual consistency
- **Contact-Based Discovery**: Find peers through friend-of-a-friend (FOAF) network

### **Internet Disruption Resilience**
Communitas is designed for **high partition tolerance**:
- **Local Network Operation**: Full functionality on isolated LANs
- **Mini Mesh Networks**: Create ad-hoc networks without internet
- **Bluetooth Mesh**: Connect via BLE when WiFi/cellular unavailable (mobile)
- **Automatic Reconnection**: Seamlessly rejoin mesh when connectivity returns
- **No Single Point of Failure**: No dependency on centralized infrastructure

---

## **🏗️ System Architecture**

### **Component Stack**
```
┌─────────────────────────────────────────────────────────┐
│           Desktop Application (Tauri v2)                │
│        React + TypeScript UI (Material-UI)             │
├─────────────────────────────────────────────────────────┤
│                 Tauri v2 IPC Layer                      │
│          (Secure bridge: JS ↔ Rust)                    │
├─────────────────────────────────────────────────────────┤
│            Communitas Rust Backend                      │
│  ┌──────────────┬───────────────┬───────────────────┐  │
│  │CoreContext   │Entity Manager │Four-Word System   │  │
│  │& Identity    │& Storage      │Generation & Valid │  │
│  └──────────────┴───────────────┴───────────────────┘  │
├─────────────────────────────────────────────────────────┤
│         Saorsa-Gossip Mesh Overlay                      │
│  ┌──────────────┬───────────────┬───────────────────┐  │
│  │Membership    │Pub/Sub        │Presence Service   │  │
│  │(HyParView)   │(Plumtree)     │(Group-Scoped)     │  │
│  │SWIM Failure  │Topic Routing  │Rendezvous Shards  │  │
│  └──────────────┴───────────────┴───────────────────┘  │
├─────────────────────────────────────────────────────────┤
│              Transport & Crypto Layer                   │
│  ┌──────────────┬───────────────┬───────────────────┐  │
│  │QUIC Transport│NAT Traversal  │Post-Quantum Crypto│  │
│  │(ant-quic)    │(Hole Punching)│(ML-DSA/ML-KEM)    │  │
│  │IPv4/IPv6     │Relay Fallback │ChaCha20Poly1305   │  │
│  └──────────────┴───────────────┴───────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### **Partition Tolerance Architecture**

The system maintains full functionality across network partitions:

```
Internet Disruption Scenario:

┌─────────── Local Network A ───────────┐  ┌─── Local Network B ───┐
│                                        │  │                       │
│  🖥️  Alice ←→ 🖥️  Bob                 │  │  🖥️  Charlie          │
│  │                  │                  │  │                       │
│  └── Direct QUIC ──┘                   │  │  (Disconnected)       │
│                                        │  │                       │
│  Full messaging, file sharing,        │  │  Local storage only   │
│  collaborative editing continues       │  │                       │
└────────────────────────────────────────┘  └───────────────────────┘

Internet Restored:

┌────────── Gossip Mesh Network ─────────────────────────┐
│                                                         │
│  🖥️  Alice ←─→ 🖥️  Bob ←─→ 🖥️  Charlie               │
│     ↑              ↑              ↑                    │
│     └──── FOAF Discovery ─────────┘                    │
│                                                         │
│  Automatic sync, CRDTs resolve conflicts,              │
│  presence restored, mesh reformed                      │
└─────────────────────────────────────────────────────────┘
```

**Key Mechanisms**:
- **Peer Cache**: Remembers successful peer connections for fast reconnection
- **Bootstrap Nodes**: Optional introducers for cold start (not required after first connection)
- **FOAF Discovery**: Find contacts through mutual friends when network changes
- **Coordinator Adverts**: Self-organize NAT traversal without central servers
- **Rendezvous Shards**: Distributed meeting points for finding peers globally
- **CRDT Conflict Resolution**: Automatic merge of concurrent updates after reconnection

---

## **🔗 Four-Word Addressing System**

### **Universal Entity Locator**
Four-word addresses serve as the universal addressing system for all entities:

**Primary Uses:**
- **Human-Memorable Identities**: Users, groups, organizations (e.g., "ocean-forest-moon-star")
- **Network Address Encoding**: IP + port encoded together into 4+ words for peer discovery
- **Entity Discovery**: DNS-free lookup via DHT or gossip overlay
- **Storage Location**: Map four-words to encrypted storage disks
- **Website Publishing**: DNS-free web hosting using four-word addresses

### **Network Address Encoding**
The `four_word_networking` crate encodes network addresses (IP + port together):

```rust
// CORRECT: Encode SocketAddr (IP+port) together
let socket = SocketAddr::from((ipv4_addr, port));
let four_words = FourWordEncoder::new().encode(socket)?;
// Result: 4-6 words encoding BOTH IP and port

// For peer discovery in mesh network
let addr = FourWordEncoder::new().decode(&four_words)?;
gossip_context.connect_to_peer(addr).await?;
```

### **Identity Validation**
Four-word identities must use **valid dictionary words only**:

```rust
// Validate identity uses dictionary words
let words: [String; 4] = parse_four_words(input)?;
let valid = validate_four_word_identity(words).await?;
if !valid {
    return Err("Words not in four-word-networking dictionary");
}

// Store identity in gossip overlay
let peer_id = create_gossip_identity(words).await?;
```

**Key Distinctions:**
- **Network Encoding**: IP+port → 4-6 words (for peer connections)
- **Identity**: 4 valid dictionary words (for human-memorable addresses)
- **Always validate** identity words against the dictionary
- **Never separate** IP from port when encoding network addresses

---

## **🎨 User Experience Design**

### **Entity-Centric Interface**
The UI is organized around **entities** (not channels or threads):

```
Navigation Hierarchy:

Personal Dashboard
├── 👤 My Identity (ocean-forest-moon-star)
│   ├── 💾 Private Storage (encrypted vault)
│   ├── 📁 Public Storage (shared files)
│   ├── 🌐 Personal Website (published at ocean-forest-moon-star.site)
│   └── ⚙️ Settings & Preferences
│
├── 👥 Groups
│   ├── Family Group (family-gather-sunset-wave)
│   │   ├── 💬 Group Chat
│   │   ├── 💾 Shared Storage
│   │   ├── 📹 Voice/Video Calls
│   │   └── 🌐 Group Website
│   └── Dev Team (code-sprint-build-ship)
│
├── 🏢 Organizations
│   ├── Saorsa Labs (saorsa-build-future-tech)
│   │   ├── 📢 Channels (#general, #engineering, #design)
│   │   ├── 📁 Projects (communitas-v2, saorsa-core)
│   │   ├── 💾 Organization Storage
│   │   └── 🌐 Company Website
│   └── Open Source Community (open-collab-code-free)
│
└── 📁 Projects
    └── Communitas v2 (comm-v2-launch-ready)
        ├── 💬 Project Chat
        ├── 📋 Tasks & Issues
        ├── 💾 Project Files
        └── 📊 Metrics & Progress
```

### **Core Entity Types**

**👤 Individual Identity**
- **Four-Word Address**: Your permanent identity across all devices
- **Private Storage**: Encrypted vault only you can access
- **Public Storage**: Files you share with contacts
- **Personal Website**: Publish content at your four-word address
- **Settings**: Profile, security, backup preferences

**👥 Group**
- **Four-Word Address**: Group identity for discovery
- **Members**: Add/remove contacts via four-word addresses
- **Shared Storage**: Collaborative workspace
- **Group Chat**: Real-time messaging with threads
- **Voice/Video**: HD calls and screen sharing
- **Mesh Connectivity**: Works offline if members are on same network

**🏢 Organization**
- **Multi-Channel**: Separate channels for topics (#general, #engineering, etc.)
- **Projects**: Structured workspaces with task tracking
- **Roles & Permissions**: Admin, member, guest access levels
- **Organization Storage**: Company-wide file repository
- **Website Publishing**: Professional site at org's four-word address

**📁 Project**
- **Four-Word Address**: Project identity
- **Team Members**: Assign leads and contributors
- **Project Chat**: Dedicated discussion space
- **File Versioning**: Track changes to project files
- **Task Management**: Issues, milestones, deadlines
- **Progress Tracking**: Metrics and status dashboard

**📢 Channel**
- **Topic-Focused**: Dedicated conversation streams
- **Threaded Discussions**: Keep conversations organized
- **Members**: Subset of organization members
- **Persistent History**: Searchable message archive
- **File Attachments**: Share documents in-channel

### **Virtual Disk System**
Each entity has its own virtual disk providing a unified view of markdown-based data with different access policies:

```
Entity Storage Model (Backend-Focused):

ocean-forest-moon-star (Your Identity)
├── 💾 Private Disk (encrypted, local-only markdown files)
│   └── Accessible only by you on authorized devices
│   └── Synced via Yrs CRDT for collaborative file editing
│
├── 📁 Public Disk (replicated markdown files)
│   └── Readable by anyone with the link
│   └── Full file replication across gossip network
│
└── 🔗 Shared Disk (group-encrypted markdown files)
    └── Accessible by group members with permission
    └── Synchronized via Automerge CRDT for entities/chats

Access Control:
- Private: ML-DSA key + device authorization required
- Public: Full file replication with signature verification
- Shared: MLS group membership + derived encryption keys
- Implementation: Backend-focused with markdown file storage
```

### **Website Publishing**
Publish DNS-free websites using four-word addresses:

```
Website Publishing Flow:

1. Create website in entity's public storage
2. Build site manifest (index.html, assets, etc.)
3. Generate content IDs (BLAKE3 hashes)
4. Sign manifest with ML-DSA private key
5. Publish to gossip overlay (Saorsa Sites)
6. Visitors browse to: ocean-forest-moon-star.site

Website Access:
- Visitor subscribes to site's rendezvous shard
- Discovers providers (peers hosting the site)
- Fetches manifest and content chunks via QUIC
- Verifies signatures and content integrity
- Renders site locally

Private Websites:
- Create MLS group for authorized viewers
- Encrypt content with group-derived keys
- Share four-word address with group members only
```

---

## **🔐 Security & Privacy**

### **Post-Quantum Cryptography**
All cryptographic operations use quantum-resistant algorithms:

- **Signatures**: ML-DSA-87 (NIST Level 5, ~256-bit quantum security)
- **Key Exchange**: ML-KEM-1024 (NIST Level 5, ~256-bit quantum security)
- **Symmetric Encryption**: ChaCha20-Poly1305 AEAD (preferred over AES-GCM)
- **Hashing**: BLAKE3 for content addressing and key derivation

### **Trust-On-First-Use (TOFU) Security Model**

```
First Contact with New Peer:

1. User initiates contact with "ocean-forest-moon-star"
2. Resolve four-words to peer via gossip overlay
3. Establish QUIC connection with TLS 1.3
4. Exchange ML-DSA public keys
5. Pin peer's public key and transport ID
6. Display four-word checksum for out-of-band verification
7. Mark as trusted after user confirms

Subsequent Connections:
- Verify transport ID matches pinned value
- Validate ML-DSA signature with pinned public key
- Alert user if keys change (potential MITM)
- Require continuity signature from previous key for rotation

Anti-Phishing:
- Four-word checksums displayed prominently
- Users verify checksums via secondary channel
- No way to forge valid four-word identity
```

### **End-to-End Encryption**
All communication is encrypted end-to-end:

- **Direct Messages**: Point-to-point QUIC with ML-KEM key exchange
- **Group Chats**: MLS (Messaging Layer Security) group encryption
- **File Storage**: ChaCha20-Poly1305 with per-entity keys
- **Presence Beacons**: MLS-encrypted rotating announcements

### **Privacy by Design**
- **No IP Address Storage**: Only four-word addresses in DHT/gossip
- **No Metadata Leakage**: Message timing obscured via batching
- **No Central Logging**: All logs are local and user-controlled
- **No Telemetry by Default**: Opt-in anonymous usage stats only

---

## **🛠️ Development Workflow**

### **Standard Desktop Development**
```bash
# 1. Install dependencies
npm install
cargo build

# 2. Build frontend assets
npm run build

# 3. Run Tauri desktop app
npm run tauri dev

# 4. Run tests
npm test                    # Frontend tests
cargo test --workspace      # Backend tests
```

### **Bridge Server for Testing**
For browser-based testing and MCP integration:

```bash
# Terminal 1: Start bridge server
cargo run -p communitas-bridge

# Terminal 2: Start frontend dev server
npm run dev

# Bridge exposes HTTP/REST API at localhost:3030
# See docs/BRIDGE_TESTING.md for full testing guide
```

### **MCP Testing Strategy**
The Chrome DevTools MCP integration enables AI-assisted testing:

**Setup:**
```bash
# Configure in .mcp.json (already set up)
{
  "mcpServers": {
    "chrome-devtools": {
      "command": "npx",
      "args": ["chrome-devtools-mcp@latest"]
    }
  }
}

# Start testing
npm run dev                              # Terminal 1: Frontend
cargo run -p communitas-bridge           # Terminal 2: Bridge server
npx chrome-devtools-mcp@latest \
  --browserUrl http://127.0.0.1:1420    # Terminal 3: MCP inspector
```

**Test Scenarios:**
1. **Entity Creation**: Create groups/orgs with four-word validation
2. **P2P Connection**: Connect two instances via gossip overlay
3. **Mesh Formation**: Test 3+ peer mesh network
4. **Partition Tolerance**: Simulate network split and rejoin
5. **Offline Operation**: Test local-only functionality
6. **Storage Operations**: Create/read/update entity storage
7. **Website Publishing**: Publish and browse sites via four-words
8. **Voice/Video**: Test WebRTC calls with screen sharing

---

## **📦 Crate Organization**

### **Active Crates**
```
communitas/
├── communitas-core/              # Business logic & saorsa-gossip integration
│   ├── src/
│   │   ├── core_context.rs      # Main application context
│   │   ├── identity.rs           # Four-word identity system
│   │   ├── encrypted_storage/   # Vault management
│   │   ├── gossip/              # P2P mesh overlay
│   │   ├── crdt.rs              # Conflict-free replication
│   │   └── doc_replicator.rs    # Document synchronization
│   └── tests/                   # Integration tests
│
├── communitas-desktop/           # Tauri v2 desktop application
│   ├── src/
│   │   ├── main.rs              # Tauri entry point
│   │   ├── core_commands.rs     # IPC command handlers
│   │   ├── core_groups.rs       # Group management
│   │   └── entity_storage.rs    # Entity storage interface
│   └── tauri.conf.json          # Tauri configuration
│
├── crates/communitas-container/  # Content addressing & storage
│   └── src/
│       ├── lib.rs               # Container abstraction
│       └── seal.rs              # Content sealing
│
├── src/                          # React frontend (TypeScript)
│   ├── components/
│   │   ├── auth/                # Authentication UI
│   │   ├── entities/            # Entity management
│   │   ├── storage/             # File browser
│   │   └── websites/            # Site publisher
│   ├── services/
│   │   ├── DocumentService.ts   # CRDT document ops
│   │   ├── network/             # P2P connection mgmt
│   │   └── storage/             # Offline-first cache
│   └── contexts/
│       ├── AuthContext.tsx      # Authentication state
│       └── EntityDirectoryContext.tsx  # Entity catalog
│
└── docs/                        # Documentation
    ├── BRIDGE_TESTING.md        # Testing guide
    ├── AGENTS_API.md            # Complete API reference
    └── archive/                 # Outdated specs
```

### **Temporarily Excluded (API Mismatches)**
These crates need API updates to match restored architecture:

- `communitas-bridge/`: HTTP test interface (references old API)
- `communitas-headless/`: Bootstrap nodes (missing imports)
- `communitas-tui/`: Terminal UI (old CoreContext API)

---

## **🔄 Current Implementation Status**

### **✅ Fully Implemented**
- Four-word identity generation and validation
- ML-DSA/ML-KEM post-quantum cryptography
- QUIC transport with ant-quic
- Gossip overlay with membership (HyParView + SWIM)
- Pub/sub messaging (Plumtree)
- CRDT-based document collaboration (Yrs)
- Encrypted vault storage (ChaCha20-Poly1305)
- Tauri v2 desktop application framework
- Entity creation and management
- Offline-first storage with IndexedDB

### **🚧 In Progress**
- MLS group encryption for multi-party chats
- Voice/video calling (WebRTC integration)
- Website publishing UI (Saorsa Sites)
- Mobile application (React Native)
- Bluetooth mesh bridge (for mobile)

### **🎯 Next Priorities**

**Phase 1: API Restoration (This Week)**
1. Fix CoreContext API mismatches in bridge/headless/tui
2. Test entity creation end-to-end
3. Verify storage operations (private/public/shared)
4. Test website publishing flow

**Phase 2: Mesh Testing (Next Week)**
1. Multi-node P2P connection tests
2. Partition tolerance verification
3. Offline operation validation
4. FOAF discovery testing
5. MCP-based automated test suite

**Phase 3: Feature Completion (2 Weeks)**
1. Complete MLS group encryption
2. Implement voice/video calling
3. Finish website publishing UI
4. Add mobile application scaffold

---

## **📊 Architecture Diagrams**

### **Data Flow: Entity Creation**

```
User Action: Create Group "family-gather-sunset-wave"
                    ↓
┌─────────────────────────────────────────────────────┐
│              Frontend (React)                       │
│  User fills form → Click "Create Group"            │
└────────────────────┬────────────────────────────────┘
                     ↓ (Tauri IPC)
┌─────────────────────────────────────────────────────┐
│              Backend (Rust)                         │
│  1. Validate four-words via saorsa-gossip          │
│  2. Generate ML-DSA keypair for group              │
│  3. Create MLS group for members                   │
│  4. Initialize entity storage disk                 │
│  5. Store group metadata in local DB              │
└────────────────────┬────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────────┐
│              Gossip Overlay                         │
│  1. Publish group existence to topic               │
│  2. Advertise group's four-word address            │
│  3. Join rendezvous shard for discovery            │
└────────────────────┬────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────────┐
│              Mesh Network                           │
│  Other peers can now find and join group via:     │
│  - FOAF queries through mutual contacts            │
│  - Rendezvous shard subscription                   │
│  - Direct invitation with four-word address        │
└─────────────────────────────────────────────────────┘
```

### **Data Flow: P2P Message Delivery**

```
Alice sends message to Bob in "dev-team" group:
                    ↓
┌─────────────────────────────────────────────────────┐
│              Alice's Device                         │
│  1. Compose message in UI                          │
│  2. Encrypt with MLS group key                     │
│  3. Sign with Alice's ML-DSA key                   │
│  4. Store in local CRDT document                   │
│  5. Add to send queue                              │
└────────────────────┬────────────────────────────────┘
                     ↓ (Gossip Pub/Sub)
┌─────────────────────────────────────────────────────┐
│              Gossip Overlay (Plumtree)              │
│  1. Publish to group topic                         │
│  2. Fanout via optimized tree topology            │
│  3. Hop through mesh to reach Bob                  │
└────────────────────┬────────────────────────────────┘
                     ↓ (QUIC Transport)
┌─────────────────────────────────────────────────────┐
│              Bob's Device                           │
│  1. Receive encrypted message                      │
│  2. Verify Alice's ML-DSA signature                │
│  3. Decrypt with MLS group key                     │
│  4. Merge into local CRDT document                 │
│  5. Display in UI                                  │
└─────────────────────────────────────────────────────┘

If Bob is offline:
- Alice's message stored in CRDT (optimistic update)
- Other group members relay message via gossip
- Bob receives on reconnection via CRDT anti-entropy
- CRDTs automatically resolve any conflicts
```

---

## **🎨 UI Implementation**

### **Modern WhatsApp/Slack-Style Interface**

The UI follows a **three-panel layout** inspired by modern collaboration tools:

```
┌─────────────────────────────────────────────────────────┐
│  Entity Sidebar (340px)  │  Main Content (flex)  │  Info  │
│───────────────────────────┼───────────────────────┼────────┤
│ [👤 Ocean-Forest-Moon] │ [🏢 ACME CORP]      │ Details│
│───────────────────────────│  Organisation view    │        │
│ [All Spaces] [Orgs] [...]  │───────────────────────│        │
│───────────────────────────│ ┌─Members────┬─Projects┐ │        │
│ 🔍 Search (Cmd+K)         │ │ DA • Owner│ 📁 Lumos │ │        │
│───────────────────────────│ │ LM • Admin│ 📁 Boot. │ │        │
│ ▼ 🏢 ACME CORPORATION     │ ├───────────┴──────────┤ │        │
│   # general • Updates      │ │ Channels   │ Storage   │ │        │
│   # engineering • 3 new    │ │ #general   │ 420GB/1TB │ │        │
│   📁 Website Redesign     │ │ #eng       │ [███░░] 42%│ │        │
│   👥 Development Team     │ └───────────┴──────────┘ │        │
│ ▶ 🏢 TECH STARTUP        │                       │        │
└───────────────────────────┴───────────────────────┴────────┘
```

### **Component Architecture**

**Left Sidebar (Entity List):**
- **Identity Selector**: Shows current four-word identity
- **Filters**: Two-tier filtering (Space type + Entity type)
- **Search**: Command palette with ⌘K shortcut
- **Organization Tree**: Expandable hierarchy with channels/projects/teams

**Main Content Area:**
- **Dynamic Views**: Changes based on selected entity
- **Organization Dashboard**: 2x2 grid (Members, Projects, Channels, Storage)
- **Chat Interface**: WhatsApp-style messaging with threads
- **Storage Management**: Visual meters and file browser

**Design System:**
```scss
// Dark Theme Colors
$background-primary: #161C20;    // Main content
$background-secondary: #1a1f24;  // Sidebar
$accent-green: #2EB67D;          // Primary (online, success)
$accent-blue: #1E88E5;           // Secondary
$text-primary: #F4F6F8;          // Main text
$text-secondary: #9AA2AB;        // Muted text
```

### **Key UI Features**

1. **Organizations First**: Orgs are the primary grouping mechanism
2. **Visual Storage Meters**: Progress bars show vault usage
3. **Member Avatars**: Gradient backgrounds with initials
4. **Status Indicators**: Green/yellow/gray dots for presence
5. **Filter Chips**: Quick filtering by space and entity type
6. **Hover States**: Interactive feedback on all elements

**Implementation Files:**
- `storyboard-canvas-v2.html` - Interactive prototype
- `STORYBOARD.md` - Complete component specifications
- `STORYBOARD_V2.md` - Day-by-day implementation guide

---

## **🎯 Key Takeaways**

**This is NOT a web app. It's a desktop/mobile P2P collaboration platform.**

**Core Capabilities:**
- ✅ Native desktop application (Tauri v2)
- ✅ Gossip-based P2P mesh networking
- ✅ Works during internet disruptions
- ✅ Connect to contacts on any reachable network
- ✅ Partition-tolerant and offline-capable
- ✅ Post-quantum cryptography everywhere
- ✅ Four-word human-memorable addresses
- ✅ Entity-centric UI (individuals, groups, orgs, projects)
- ✅ Virtual disks for encrypted storage
- ✅ DNS-free website publishing

**What Makes It Different:**
- **No central servers** - True peer-to-peer architecture
- **No web browser required** - Native application only
- **No internet required** - Works on local networks
- **No DNS** - Four-word addressing replaces domains
- **No single point of failure** - Highly distributed
- **No plaintext metadata** - Everything encrypted

**For Developers:**
- Build with `cargo build && npm run tauri dev`
- Test with MCP: `npm run dev` + `cargo run -p communitas-bridge`
- Deploy as self-contained desktop binary
- Plan for mobile with React Native bridge

---

**This design document is the authoritative source for all architectural decisions in Communitas.**
