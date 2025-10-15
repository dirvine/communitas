# Communitas — Local-First Collaboration Platform

> **Post-quantum collaboration: messaging, virtual disks, DNS-free websites with Four-Word identities.**

Communitas is a local-first, PQC-ready collaboration platform that unifies messaging, file sharing, voice/video calling, and web publishing using human-verifiable Four-Word addressing. Built on saorsa-core v0.3.26 with Tauri v2.

---

## **🚀 Quick Start**

### **Prerequisites**
- Node.js 20+
- Rust 1.85+
- Platform dependencies for Tauri v2

### **Development Setup**
```bash
git clone https://github.com/dirvine/communitas.git
cd communitas
npm install

# Build frontend and run Tauri app
npm run build
npm run tauri dev
```

### **Testing**
```bash
# TypeScript validation
npm run typecheck

# Rust linting (strict policy)
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used

# Unit tests
npm run test:run
cargo test
```

---

## **📋 Key Features**

### **Four-Word Identity System**
- **Human-Verifiable**: `ocean-blue-eagle-star` instead of cryptographic hashes
- **Universal Addressing**: Users, organizations, websites, storage disks
- **Anti-Phishing**: Dictionary validation prevents typosquatting
- **DNS Replacement**: Cryptographically verified, decentralized naming

### **Entity-Based Collaboration**
- **👤 Individuals**: Personal identity, private storage, direct messaging
- **👥 Groups**: Team collaboration, shared virtual disks, voice calls
- **🏢 Organizations**: Multi-channel communication, admin controls
- **📁 Projects**: Structured workspaces, task management, version control
- **📢 Channels**: Topic-focused discussions, threaded conversations

### **Post-Quantum Security**
- **ML-DSA Signatures**: Quantum-resistant identity verification
- **ML-KEM Key Exchange**: Secure session establishment
- **End-to-End Encryption**: All communications encrypted by default
- **Forward Secrecy**: Perfect forward secrecy for all sessions

### **Local-First Architecture**
- **Offline Capable**: Core functionality works without network
- **Real-Time Sync**: Background synchronization when connected
- **Conflict Resolution**: Automatic merge with manual override options
- **Data Ownership**: All data stored locally with optional P2P sharing

---

## **📚 Documentation**

### **Getting Started**
- **[Getting Started Guide](docs/guides/getting-started.md)**: Complete setup and first steps *(coming soon)*
- **[Authentication Guide](docs/guides/authentication.md)**: Login, passkeys, and security *(coming soon)*
- **[Four-Word Addresses](docs/guides/four-word-addresses.md)**: Understanding identity system *(coming soon)*

### **Architecture & Design**
- **[DESIGN.md](DESIGN.md)**: System architecture and technical design
- **[Architecture Overview](docs/architecture/)**: Detailed architecture documentation
- **[CRDT System](docs/CRDT_ARCHITECTURE.md)**: Conflict-free replicated data types
- **[Gossip Protocol](docs/GOSSIP_OVERLAY.md)**: P2P communication layer

### **API Reference**
- **[Tauri Commands API](docs/AGENTS_API.md)**: Complete Tauri command reference
- **[Core API](docs/api/)**: Rust core library API *(coming soon)*
- **[Frontend API](docs/api/)**: React/TypeScript interface *(coming soon)*

### **Deployment Guides**

- **[Headless Service](communitas-headless/README.md)**: systemd, launchd, JSON-RPC API
- **[Bootstrap Node](bootstrap-node/README.md)**: Network bootstrap deployment
- **[Testnet Deployment](finalise/DEPLOY_TESTNET.md)**: Complete network deployment

### **Development**
- **[Contributing Guide](docs/development/)**: How to contribute *(coming soon)*
- **[Coding Standards](docs/development/)**: Code style and quality *(coming soon)*
- **[Testing Guide](docs/guides/testing.md)**: Test strategy and examples *(coming soon)*
- **[Troubleshooting](docs/development/)**: Common issues and solutions *(coming soon)*

### **Operations**
- **[Monitoring](docs/operations/)**: Prometheus, Grafana, metrics *(coming soon)*
- **[Security Policy](docs/operations/)**: Security guidelines *(coming soon)*
- **[Incident Response](docs/operations/)**: Emergency procedures *(coming soon)*

### **For AI Assistants**
- **[CLAUDE.md](CLAUDE.md)**: Project context for LLM helpers
- **[Agent Automation](docs/development/AGENTS.md)**: Automated development workflows

---

## **🏗️ Project Structure**

### **Applications**
- **[communitas-desktop/](communitas-desktop/)**: Tauri v2 desktop application with React frontend
- **[communitas-headless/](communitas-headless/)**: Headless daemon for system services ([README](communitas-headless/README.md))
- **[communitas-bridge/](communitas-bridge/)**: HTTP/REST bridge for browser testing ([README](communitas-bridge/README.md))

### **Core Libraries**
- **[communitas-core/](communitas-core/)**: Shared Rust business logic and P2P networking
- **[bootstrap-node/](bootstrap-node/)**: Network bootstrap and discovery service ([README](bootstrap-node/README.md))

### **Container & Deployment**


### **Frontend**
- **[src/](src/)**: React frontend with TypeScript
- **[dist/](dist/)**: Built frontend assets (served by Tauri)

### **Documentation**
- **[docs/](docs/)**: Comprehensive project documentation
  - **[guides/](docs/guides/)**: User and developer guides
  - **[architecture/](docs/architecture/)**: System architecture documentation
  - **[api/](docs/api/)**: API reference documentation
  - **[development/](docs/development/)**: Development setup and standards
  - **[operations/](docs/operations/)**: Deployment and operations guides
  - **[archive/](docs/archive/)**: Historical documentation

### **Key Commands**
```bash
# Development
npm run build && npm run tauri dev

# Production Build
npm run build && npm run tauri build

# Quality Checks
npm run typecheck && cargo clippy --all-features

# Run as system service (see communitas-headless/README.md)
communitas-headless --config /etc/communitas/headless.toml


```

---

## **🚢 Deployment Options**

Communitas supports multiple deployment scenarios for different use cases:

### **Desktop Application** (End Users)
Full-featured native application with UI for Windows, macOS, and Linux.
```bash
npm run build && npm run tauri build
```
See [communitas-desktop/](communitas-desktop/) for details.

### **Headless Daemon** (Servers & Bots)
Run as a system service for automated operations, bots, or server infrastructure.
```bash
# Install and run as systemd service
sudo systemctl enable communitas
sudo systemctl start communitas
```
Complete guide: [communitas-headless/README.md](communitas-headless/README.md)



### **Bootstrap Nodes** (Network Infrastructure)
DHT bootstrap and discovery nodes for network infrastructure.
```bash
cargo run -p bootstrap-node -- --config config.toml
```
Complete guide: [bootstrap-node/README.md](bootstrap-node/README.md)

### **Testing Bridge** (Development)
HTTP/REST bridge for browser-based testing with Chrome DevTools MCP.
```bash
cargo run -p communitas-bridge
```
Complete guide: [communitas-bridge/README.md](communitas-bridge/README.md)

---

## **🌐 Network & Identity**

### **Four-Word Addressing Example**
```typescript
// Create new organization with just display name
const result = await createOrganization({ 
  displayName: "Acme Corporation" 
});
// → { fourWords: "ocean-blue-eagle-star", entityId: "org-abc123" }

// Others can find it using the four-words
const entity = await findEntity("ocean-blue-eagle-star");
// → Access organization workspace, channels, shared storage
```

### **Network Participation**
- **Desktop Nodes**: Full participants with UI
- **Headless Nodes**: Bootstrap/seed nodes for network infrastructure  
- **Mobile Nodes**: Future lightweight clients
- **Browser Bridge**: WebRTC bridge for web access

---

## **🔐 Security Model**

### **Zero-Trust Architecture**
- **Everything Encrypted**: All data encrypted at rest and in transit
- **Cryptographic Verification**: Every entity verified by signature
- **No Central Authority**: Fully decentralized with gossip overlay consensus
- **Quantum-Safe**: Post-quantum cryptography throughout

### **Privacy Features**
- **Local-First**: Data stays on your devices unless explicitly shared
- **Selective Sharing**: Granular control over what gets shared with whom
- **Anonymous Discovery**: Find public entities without revealing identity
- **Plausible Deniability**: Private messages indistinguishable from noise

---

## **📄 License**

**AGPL-3.0** for open collaboration. Commercial licensing available via [Saorsa Labs](mailto:saorsalabs@gmail.com).

---

## **🤝 Contributing**

1. **Code Style**: Follow existing patterns and conventions
2. **Commit Format**: Conventional commits (`feat:`, `fix:`, `docs:`)
3. **Quality Gates**: All code must pass TypeScript + Rust linting
4. **Testing**: Include tests for new functionality

### **Development Standards**
- **No Panics**: Rust code forbids `unwrap`/`expect`/`panic!` in production
- **Type Safety**: Full TypeScript coverage with strict configuration
- **Security First**: Post-quantum cryptography and secure defaults
- **Local-First**: All features must work offline

---

**Ready to revolutionize collaboration? Start building the future of communication today! 🚀**
