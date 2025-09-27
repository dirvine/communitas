# Communitas — Local-First Collaboration Platform

> **Post-quantum collaboration: messaging, virtual disks, DNS-free websites with Four-Word identities.**

Communitas is a local-first, PQC-ready collaboration platform that unifies messaging, file sharing, voice/video calling, and web publishing using human-verifiable Four-Word addressing. Built on saorsa-core v0.3.25 with Tauri v2.

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

### **Core Documentation**
- **[DESIGN.md](DESIGN.md)**: System architecture and technical design
- **[UX_FLOWS.md](UX_FLOWS.md)**: User experience patterns and interface design
- **[ARCHITECTURE.md](ARCHITECTURE.md)**: Technical implementation details

### **Development Resources**
- **[docs/development/AGENTS.md](docs/development/AGENTS.md)**: Agent automation guide
- **[docs/AGENTS_API.md](docs/AGENTS_API.md)**: API reference for automation
- **[CLAUDE.md](CLAUDE.md)**: LLM helper documentation

### **Deployment**
- **[finalise/DEPLOY_TESTNET.md](finalise/DEPLOY_TESTNET.md)**: Network deployment guide

---

## **🏗️ Project Structure**

```
communitas/
├── src/                     # Legacy React SPA (maintained for regression)
├── apps/communitas/         # Next-gen React console (future migration)
├── communitas-core/         # Shared Rust business logic
├── communitas-desktop/      # Tauri v2 desktop application
├── communitas-headless/     # Bootstrap/seed node binary
├── crates/                  # Additional Rust crates
├── dist/                    # Built frontend assets (Tauri serves from here)
├── docs/                    # Core documentation
└── tools/                   # Development utilities
```

### **Key Commands**
```bash
# Development
npm run build && npm run tauri dev

# Production Build  
npm run build && npm run tauri build

# Quality Checks
npm run typecheck && cargo clippy --all-features
```

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
- **No Central Authority**: Fully decentralized with DHT consensus
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
