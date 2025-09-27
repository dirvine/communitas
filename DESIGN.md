# Communitas — Design & Architecture

**Version**: 1.0 • **Date**: 2025-09-27 • **Status**: Current Implementation

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
- **`communitas-container/`**: FEC/seal content operations
- **`apps/communitas/`**: Next-gen React console (future migration target)
- **`src/`**: Legacy React SPA (maintained for regression coverage)

---

## **🔗 Four-Word Addressing System**

### **Universal Entity Locator**
Four-word addresses serve as the universal addressing system for all entities:

**Uses:**
- **IP4/IP6 Encoding**: Network address encoding (more than 4 words)
- **DNS Replacement**: 4 valid dictionary words + hash for entity discovery
- **Entity Identity**: Users, websites, storage disks, groups, organizations
- **Storage Discovery**: Group four-words → storage disk location

**Implementation:**
```rust
// Generate valid four-words using saorsa-core
let four_words = generate_four_word_identity().await?;

// Validate using dictionary
let valid = fw_check(words_array);

// Convert to entity key for DHT storage
let entity_key = fw_to_key(words_array)?;
```

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

## **🛠️ Technical Implementation**

### **Development Workflow**
```bash
# 1. Build frontend assets
npm run build

# 2. Run Tauri with built assets  
npm run tauri dev

# 3. Test entity creation flow
npm run test:run
```

### **Core Agent Flow**
1. **Identity Claim**: `core_claim(words: [String; 4])` + keyring persistence
2. **Presence Advertise**: `core_advertise(addr, storage_gb)` → endpoint discovery
3. **Runtime Initialize**: `core_initialize` → CoreContext + services
4. **Entity Operations**: Create/find entities using four-word addressing
5. **Messaging**: End-to-end encrypted channels with MLS
6. **Storage**: Virtual disks per entity with FEC protection

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

### **🚧 In Progress**
- Real-time collaboration (Yjs CRDTs)
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
