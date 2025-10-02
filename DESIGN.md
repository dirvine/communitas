# Communitas — Design & Architecture

**Version**: 1.1 • **Date**: 2025-09-30 • **Status**: Current Implementation

Communitas is a local-first, post-quantum collaboration platform that unifies messaging, file sharing, voice/video calling, and web publishing into a single decentralized experience using Four-Word addressing.

---

## **🏗️ System Architecture**

### **Component Stack**
```
┌─────────────────────────────────────────────────────────┐
│              React + TypeScript Frontend                │
│              (Material-UI, Vite bundled)               │
├─────────────────────────────────────────────────────────┤
│                   Tauri v2 IPC Layer                   │
├─────────────────────────────────────────────────────────┤
│              Rust Backend (Communitas)                 │
│  ┌─────────────┬──────────────┬──────────────────────┐ │
│  │Core Context │Container     │Four-Word Generation  │ │
│  │& Identity   │Engine        │& Validation          │ │
│  └─────────────┴──────────────┴──────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│              Saorsa-Core Foundation                     │
│  ┌─────────────┬──────────────┬──────────────────────┐ │
│  │DHT Network  │QUIC Transport│Post-Quantum Crypto   │ │
│  │(Kademlia)   │(ant-quic)    │(ML-DSA/ML-KEM)       │ │
│  └─────────────┴──────────────┴──────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### **Crate Organization**
- **`communitas-core/`**: Business logic, saorsa-core integration, re-exports
- **`communitas-desktop/`**: Tauri v2 app with IPC commands
- **`communitas-headless/`**: Bootstrap/seed nodes for network
- **`communitas-bridge/`**: HTTP/REST testing bridge for browser-based testing (NEW: 2025-09-30)
- **`communitas-container/`**: FEC/seal content operations
- **`apps/communitas/`**: Next-gen React console (future migration target)
- **`src/`**: Legacy React SPA (maintained for regression coverage)

---

## **🔗 Four-Word Addressing System**

### **Universal Entity Locator**
Four-word addresses serve as the universal addressing system for all entities:

**Uses:**
- **IP4/IP6 Encoding**: Network address encoding (IP + port encoded together into 4+ words)
- **DNS Replacement**: 4 valid dictionary words for entity discovery
- **Entity Identity**: Users, websites, storage disks, groups, organizations
- **Storage Discovery**: Group four-words → storage disk location

**Critical Implementation Notes:**

### **For Network Addresses (IP + Port)**
The `four_word_networking` crate **correctly encodes IP and port together**:
```rust
// CORRECT: Encode IP+port together (as designed by the crate)
let socket_addr = SocketAddr::from((ipv4_addr, port));
let encoding = FourWordEncoder::new().encode(socket_addr)?;
// Result: 4 words that encode BOTH IP and port

// INCORRECT: DO NOT try to encode IP separately from port
// The crate is designed to handle them together
```

### **For Entity Identities**
Four-word identities must use **valid dictionary words only**:
```rust
// Validate all four words are in the dictionary
let words: [String; 4] = parse_four_words(input)?;
let valid = saorsa_core::fwid::fw_check(words);
if !valid {
    return Err("Words not in four-word-networking dictionary");
}

// Convert to entity key for DHT storage
let entity_key = fw_to_key(words_array)?;
```

**Key Distinctions:**
- **Network Encoding**: IP+port → 4 words (encodes actual network address)
- **Identity Validation**: 4 words → must be valid dictionary words (human-memorable)
- **Never separate IP from port** when encoding network addresses
- **Always validate** identity words against the dictionary

### **Entity Creation Flow**
1. **DHT Connection Check**: Validate network connectivity
2. **Four-Word Generation**: Use saorsa-core validation  
3. **Entity Storage**: Store metadata on DHT with four-word key
4. **Discovery**: Other users can find entities by four-words

---

## **🎨 User Experience Design**

### **Core Entities**
- **👤 Individuals**: Personal identity, private storage, direct messaging
- **👥 Groups**: Medium teams, shared virtual disks, voice calls
- **🏢 Organizations**: Large-scale hubs, multi-channel, admin controls  
- **📁 Projects**: Structured workspaces, version control, task management
- **📢 Channels**: Topic-focused streams, threaded conversations

### **Collaboration Capabilities**
- **💬 Messaging**: Real-time chat, threads, reactions, @mentions
- **🎥 Voice/Video**: HD calls, screen sharing, recording
- **💾 Virtual Disks**: Per-entity encrypted storage with collaborative editing
- **🌐 Web Publishing**: DNS-free websites using Four-Word addresses
- **🔐 Security**: Post-quantum cryptography, end-to-end encryption

### **Navigation Flow**
```
Personal Dashboard → Select Entity Type → Entity-Specific Interface
     ↓                       ↓                      ↓
Individual View         Organization View     Project/Group View
     ↓                       ↓                      ↓
Direct Messages         Multi-Channel Chat    Collaborative Workspace
Private Storage         Shared Resources      Project Files & Tasks
```

---

## **🆔 DHT Identity System**

### **Three-Layer Architecture**
Communitas implements a sophisticated DHT identity system that stores identity packets, connection information, and web content using a three-layer design:

```
┌─────────────────────────────────────────────────────┐
│              DHT Layer (≤512B Records)              │
│  ┌─────────────┬─────────────┬─────────────────────┐ │
│  │IdentityRoot │ConnectionRec│SiteManifestRecord   │ │
│  │Record       │ord          │                     │ │
│  └─────────────┴─────────────┴─────────────────────┘ │
├─────────────────────────────────────────────────────┤
│            Content-Addressed Blobs                  │
│  ┌─────────────┬─────────────┬─────────────────────┐ │
│  │Identity     │Connection   │SiteManifestBlob     │ │
│  │DescriptorBlob│Blob        │                     │ │
│  └─────────────┴─────────────┴─────────────────────┘ │
├─────────────────────────────────────────────────────┤
│              Erasure-Coded Storage                  │
│       Web Pages • Media Assets • Large Content     │
└─────────────────────────────────────────────────────┘
```

### **DHT Record Types**
All DHT records use domain-separated BLAKE3 keys and stay within saorsa-core's 512B limit:

**IdentityRootRecord** (`K_id = blake3("communitas:id:v1:" || four_words)`)
- **Size**: 215-490 bytes
- **Purpose**: Main identity pointer with cryptographic hashes
- **Fields**: Sequence, timestamps, PQC key hashes, transport ID, content pointers
- **Update Frequency**: Infrequent (identity changes only)

**ConnectionRecord** (`K_conn = blake3("communitas:conn:v1:" || four_words)`)  
- **Size**: 106-166 bytes
- **Purpose**: Fast-updating NAT traversal coordination
- **Fields**: Transport ID, rendezvous node pointers, TTL
- **Update Frequency**: High (network changes)

**SiteManifestRecord** (`K_site = blake3("communitas:site:v1:" || four_words)`)
- **Size**: 67-96 bytes  
- **Purpose**: Website content manifest pointer
- **Fields**: Site manifest CID, sequence, TTL
- **Update Frequency**: Medium (content publishing)

### **Content-Addressed Blobs**

**IdentityDescriptorBlob** (ML-DSA-65 Signed)
- **Security**: Post-quantum digital signatures (ML-DSA-65)
- **Contents**: PQC public keys, display name, transport keys, continuity proofs
- **Binding**: Contains `root_core_digest` for cryptographic binding to root record
- **Size**: ~8KB (due to PQC key sizes)

**ConnectionBlob** (Pure QUIC NAT Traversal)
- **NAT Strategy**: Rendezvous/coordination via bootstrap nodes (no STUN)
- **Contents**: Bootstrap node lists, relay hints, connection policies
- **Integration**: Direct integration with ant-quic's NAT traversal system
- **Privacy**: No direct IP addresses stored in DHT

**SiteManifestBlob** (5MB Content Limit)
- **Purpose**: Website hosting with content integrity
- **Limit**: 5MB total content per identity
- **Structure**: Page manifest with content IDs and integrity hashes
- **Storage**: Content chunks via saorsa-core's erasure coding

### **Security Model: TOFU + PQC**

**Trust-On-First-Use (TOFU)**
- **First Contact**: Pin ML-DSA public key and ant-quic transport ID
- **Key Rotation**: Require continuity signatures from previous key
- **Transport Binding**: Cryptographic binding between DHT identity and QUIC transport
- **Audit Trail**: Security events logged for suspicious activity

**Post-Quantum Cryptography**
- **Signatures**: ML-DSA-65 (NIST Level 3, ~128-bit quantum security)
- **Key Exchange**: ML-KEM-768 (NIST Level 3, ~128-bit quantum security)  
- **Integration**: Seamless with ant-quic's hybrid PQC implementation
- **Future-Proof**: Quantum-resistant from day one

### **Circular Dependency Resolution**
The system elegantly resolves the circular dependency between root records and descriptor blobs:

- **Core Hash**: `blake3("root-core-v1" || core_fields)` (excludes descriptor_cid)
- **Full Hash**: `blake3("root-bind-v1" || core_hash || descriptor_cid)`  
- **Binding**: Descriptor stores `core_hash`, root stores `descriptor_cid`
- **Security**: Strong cryptographic binding without circular dependencies

### **Implementation Status**
✅ **Complete** (51 passing tests):
- Domain-separated key derivation
- CBOR-serialized DHT records  
- ML-DSA signed descriptor blobs
- TOFU validation and pinning
- NAT traversal integration
- Website content management
- Comprehensive test coverage

---

## **🛠️ Technical Implementation**

### **Development Workflow**

**Standard Development:**
```bash
# 1. Build frontend assets
npm run build

# 2. Run Tauri with built assets
npm run tauri dev

# 3. Test entity creation flow
npm run test:run
```

**Browser-Based Testing (Bridge Server):**
```bash
# Terminal 1: Start bridge server
cargo run -p communitas-bridge

# Terminal 2: Start frontend dev server
npm run dev

# Bridge server exposes HTTP/REST API at localhost:3030
# Enables testing with Chrome DevTools MCP and browser automation
```

### **Bridge Server Architecture (NEW: 2025-09-30)**

The bridge server provides an HTTP/REST interface for testing Communitas with real P2P networking:

```
┌─────────────────────────────────────────────────────────┐
│              Browser Testing Layer                      │
│         (Chrome DevTools MCP, automation)              │
├─────────────────────────────────────────────────────────┤
│              HTTP/REST Bridge Server                    │
│              (localhost:3030, CORS enabled)            │
├─────────────────────────────────────────────────────────┤
│              CoreContext Integration                    │
│         (Same as Tauri desktop application)            │
├─────────────────────────────────────────────────────────┤
│              Saorsa-Core Foundation                     │
│  ┌─────────────┬──────────────┬──────────────────────┐ │
│  │DHT Network  │QUIC Transport│Post-Quantum Crypto   │ │
│  │(Kademlia)   │(ant-quic)    │(ML-DSA/ML-KEM)       │ │
│  └─────────────┴──────────────┴──────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

**Key Features:**
- **Real P2P Integration**: Uses actual saorsa-core networking (not mocked)
- **HTTP/REST API**: Standard REST endpoints for easy testing
- **CORS Enabled**: Supports browser-based testing tools
- **Chrome DevTools MCP**: Enables AI-assisted testing workflows
- **Testing Only**: Not for production use (security warning)

**Available Endpoints:**
- `GET /health` - Health check
- `POST /api/core/initialize` - Initialize with four-word identity
- `POST /api/channels` - Create channel
- `GET /api/channels` - List channels
- `POST /api/channels/:id/messages` - Send message
- `POST /api/threads/create` - Create thread from message

See `docs/BRIDGE_TESTING.md` for comprehensive testing guide.

### **Core Agent Flow**
1. **Identity Claim**: `core_claim(words: [String; 4])` + keyring persistence
2. **Presence Advertise**: `core_advertise(addr, storage_gb)` → endpoint discovery
3. **Runtime Initialize**: `core_initialize` → CoreContext + services
4. **Entity Operations**: Create/find entities using four-word addressing
5. **Messaging**: End-to-end encrypted channels with MLS
6. **Storage**: Virtual disks per entity with FEC protection

### **Messaging State & Automerge CRDTs**
- **Local-first cache**: Every conversation (channels, groups, DM threads) owns an Automerge document stored in IndexedDB via `offlineStorageService`.
- **Optimistic delivery**: Messages are appended locally with UUIDv7 identifiers and merged with remote payloads once the backend responds.
- **Conflict resolution**: Automerge handles concurrent edits/merges without manual conflict handling; metadata tracks version/updatedAt for deterministic replay.
- **Thread-aware hydration**: Threads reuse the same Automerge machinery under the `thread:<threadId>` namespace. We hydrate cached replies first and then merge the latest payload from `core_thread_get_messages` to guarantee stable ordering.
- **Thread Reply Composer (NEW: 2025-09-30)**: Threads now support in-drawer reply composition with full Automerge integration. Replies are optimistically added to the thread document, persisted to IndexedDB, and queued for backend sync under the parent entity context.
- **Bridging to backend**: The environment-aware `BackendService` now fronts all message IPC (`core_channel_get_messages`, `core_thread_get_messages`, `core_create_thread`, `core_send_message_to_channel`), normalises saorsa-core DTOs, and funnels them into Automerge.
- **Queue integration**: Failed sync attempts mark messages as `failed` in the CRDT, keeping UI state and persistence aligned with offline retries.

### **Security Model**
- **Post-Quantum**: ML-DSA signatures, ML-KEM key exchange
- **Zero-Trust**: All entities cryptographically verified
- **Anti-Phishing**: Four-word checksum validation prevents spoofing
- **Forward Secrecy**: Regular key rotation with perfect forward secrecy

### **Authentication & Identity Management**

#### **Four-Word Identity System**
The four-word address serves as the **permanent, universal identity** for users across all devices:
- **Primary Identifier**: Required for first-time login on any new device
- **Human-Memorable**: Four dictionary words are easier to remember than cryptographic keys
- **Cross-Device Portability**: Your identity travels with you to any device
- **DHT-Backed**: Identity metadata stored on the distributed network

#### **Smart Local Authentication**
Communitas provides intelligent authentication that adapts to user context:

**Password-Only Login (Familiar Devices)**
- When logging in on a previously-used device, only password is required
- System automatically scans all local encrypted vaults
- Uses PBKDF2-derived password hash as lookup key
- Matches password against all stored accounts, auto-populating four-word address
- Supports multiple accounts per device with account switching

**Full Login (New Devices)**
- Four-word address + password required for first login on new device
- Creates local encrypted vault for future password-only access
- Validates identity against DHT network when online

**Passkey Support**
- WebAuthn/FIDO2 integration for passwordless authentication
- Platform authenticators (Touch ID, Face ID, Windows Hello)
- Bound to specific device for enhanced security
- Optional backup method alongside password authentication

#### **Local Vault Architecture**
Each device maintains encrypted vaults for authenticated accounts:
```
IndexedDB Storage Structure:
├── encrypted-vaults/
│   ├── [fourWordAddress1] → Encrypted vault with PBKDF2+AES-GCM
│   ├── [fourWordAddress2] → Additional account vault
│   └── password-locators/  → Password hash → fourWordAddress mapping
```

**Vault Security:**
- PBKDF2 with 100,000 iterations for key derivation
- AES-256-GCM for vault encryption
- Per-vault salt and IV generation
- SHA-256 checksums for integrity verification

#### **Account Recovery Options**
1. **Local Recovery**: Password-based vault decryption on familiar devices
2. **Network Recovery**: Four-word address validation via DHT peers
3. **Passkey Recovery**: Device-bound authentication if configured
4. **Export/Import**: Encrypted vault backup and restoration

#### **User Experience Principles**
- **Clear Communication**: Users MUST understand four-word identity importance at registration
- **Progressive Disclosure**: Simple password login when possible, full credentials when needed
- **Multi-Account Support**: Seamless switching between identities on same device
- **Security Transparency**: Clear indicators of authentication method and security level

### **Network Architecture**
- **Transport**: QUIC over IPv4/IPv6 with PQC security
- **Discovery**: DHT-based with trust-weighted routing
- **Addressing**: Four-word human-verifiable endpoints
- **Connectivity**: Direct peer-to-peer, no central servers

---

## **📊 Distribution & Deployment**

### **Desktop Application**
- **Platform**: Tauri v2 with React frontend
- **Distribution**: Self-contained binary with embedded assets
- **Updates**: Automatic with cryptographic signature verification
- **Storage**: Local SQLite + platform keyring integration

### **Headless Nodes**
- **Bootstrap Nodes**: 6-region DigitalOcean deployment
- **Personal Nodes**: Home/server installations for network participation
- **Rewards**: Future incentive system for storage providers

### **Build Process**
```bash
# Development
npm run build && npm run tauri dev

# Production Distribution
npm run build && npm run tauri build
```

---

## **🔄 Development Status**

### **✅ Completed Features**
- Four-word addressing with saorsa-core integration
- Entity creation with DHT validation
- Tauri v2 desktop application framework
- Post-quantum cryptography integration
- Virtual disk storage architecture
- MLS messaging scaffolding
- Automerge-based messaging cache with offline-first sync
- Thread reply composer with optimistic updates (2025-09-30)
- HTTP/REST bridge server for browser-based testing (2025-09-30)
- TypeScript strict mode with zero errors (2025-09-30)

### **🚧 In Progress**
- Real-time document collaboration parity (graduating editors from Yjs to Automerge)
- Voice/video calling infrastructure
- Advanced UI component library
- Mobile application development

### **🎯 Next Priorities**
1. Complete MLS messaging with real DHT storage
2. Implement collaborative document editing
3. Add voice/video WebRTC integration
4. Deploy testnet infrastructure
5. Mobile client development

---

This design serves as the authoritative guide for all development decisions and architectural choices in the Communitas project.
