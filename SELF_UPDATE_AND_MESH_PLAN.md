# Self-Updating & Mesh Networking Implementation Plan

**Status**: Planning Phase
**Priority**: Critical - Foundation for Communitas Launch
**Date**: 2025-10-15

---

## 🎯 Executive Summary

This plan covers two critical systems for Communitas:
1. **Self-Updating System** (Phase 1) - Enable automatic updates from GitHub releases
2. **Mesh Networking** (Phase 2) - Enable peer-to-peer communication, file sharing, and collaboration

Both systems must work together seamlessly to provide a secure, decentralized application that stays up-to-date automatically.

---

# PHASE 1: SELF-UPDATING SYSTEM

## 📊 Overview

**Goal**: Desktop apps automatically update from GitHub releases without user intervention.

**Key Requirements**:
- ✅ Cross-platform (macOS, Windows, Linux)
- ✅ Secure (cryptographic signature verification)
- ✅ Safe (rollback on failure)
- ✅ Transparent (user knows what's happening)
- ✅ Offline-tolerant (check when network available)

## 🏗️ Architecture

### Components

```
┌─────────────────────────────────────────────────────────┐
│                   Communitas App                        │
│  ┌───────────────────────────────────────────────────┐  │
│  │          Update Manager (Rust/Tauri)              │  │
│  │                                                    │  │
│  │  • Version Checker                                │  │
│  │  • Signature Verifier                             │  │
│  │  • Download Manager                               │  │
│  │  • Installer                                      │  │
│  │  • Rollback Handler                               │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                         ↕
┌─────────────────────────────────────────────────────────┐
│              GitHub Releases (CDN)                      │
│                                                         │
│  • Binary artifacts (.dmg, .msi, .AppImage, .deb)      │
│  • Signature files (.sig)                              │
│  • Update manifest (latest.json)                       │
└─────────────────────────────────────────────────────────┘
```

### Update Flow

```
1. App Startup
   ↓
2. Check Current Version (Cargo.toml)
   ↓
3. Fetch Latest Version (GitHub API)
   ↓
4. Compare Versions
   ↓
   ├─ Same/Older → Continue Normal Operation
   ↓
   └─ Newer Available → Notify User
      ↓
5. User Approves Update (or Auto-Update Setting)
   ↓
6. Download New Binary
   ↓
7. Verify Signature
   ↓
   ├─ Invalid → Abort, Log Error
   ↓
   └─ Valid → Continue
      ↓
8. Backup Current Binary
   ↓
9. Install New Binary
   ↓
10. Restart App
    ↓
11. Verify New Version Works
    ↓
    ├─ Success → Delete Backup
    ↓
    └─ Failure → Rollback to Backup
```

## 🔧 Implementation Tasks

### Task 1: Tauri Updater Configuration

**File**: `communitas-desktop/tauri.conf.json`

```json
{
  "tauri": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.communitas.life/{{target}}/{{current_version}}"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

**Subtasks**:
- [ ] Enable Tauri updater in config
- [ ] Generate signing keypair (Ed25519)
- [ ] Store private key securely (GitHub Secrets)
- [ ] Add public key to tauri.conf.json
- [ ] Configure update endpoints

**Effort**: 2 hours
**Dependencies**: None

---

### Task 2: GitHub Release Automation

**File**: `.github/workflows/release.yml`

**Requirements**:
- Build binaries for all platforms
- Sign binaries with private key
- Create GitHub release
- Upload artifacts
- Generate update manifest

**Subtasks**:
- [ ] Create release workflow
- [ ] Add platform-specific build jobs (macOS, Windows, Linux)
- [ ] Add signing step (using GitHub secrets)
- [ ] Add artifact upload step
- [ ] Generate and upload latest.json manifest
- [ ] Test release process

**Effort**: 1 day
**Dependencies**: Task 1

---

### Task 3: Version Management

**Files**:
- `communitas-desktop/Cargo.toml`
- `communitas-desktop/package.json`

**Requirements**:
- Single source of truth for version
- Semantic versioning (MAJOR.MINOR.PATCH)
- Version display in UI

**Subtasks**:
- [ ] Sync version between Cargo.toml and package.json
- [ ] Add version display in app UI
- [ ] Create version bump script
- [ ] Document versioning strategy

**Effort**: 4 hours
**Dependencies**: None

---

### Task 4: Update UI Components

**Files**:
- `src/components/UpdateNotification.tsx` (new)
- `src/components/UpdateProgress.tsx` (new)

**Requirements**:
- Non-intrusive notification of updates
- Progress indicator during download
- User can defer updates
- Clear messaging about what's changing

**Subtasks**:
- [ ] Create UpdateNotification component
- [ ] Create UpdateProgress component
- [ ] Add update settings (auto-update toggle)
- [ ] Integrate with Tauri updater events
- [ ] Add update changelog display

**Effort**: 1 day
**Dependencies**: Task 1, Task 2

---

### Task 5: Update Manager Service

**File**: `communitas-desktop/src/update_manager.rs` (new)

**Requirements**:
- Check for updates on schedule (daily)
- Handle download failures gracefully
- Implement exponential backoff
- Respect metered connections
- Log all update activities

**Subtasks**:
- [ ] Create UpdateManager struct
- [ ] Implement version checking logic
- [ ] Implement download with progress
- [ ] Implement signature verification
- [ ] Add rollback mechanism
- [ ] Add comprehensive error handling
- [ ] Add logging/telemetry

**Effort**: 2 days
**Dependencies**: Task 1, Task 3

---

### Task 6: Rollback & Recovery

**File**: `communitas-desktop/src/update_manager.rs`

**Requirements**:
- Backup old binary before update
- Detect failed updates (crashes on startup)
- Automatic rollback on 3 consecutive failures
- Manual rollback option in UI

**Subtasks**:
- [ ] Implement binary backup logic
- [ ] Add startup health check
- [ ] Add crash detection
- [ ] Implement automatic rollback
- [ ] Add manual rollback UI
- [ ] Test rollback scenarios

**Effort**: 1 day
**Dependencies**: Task 5

---

### Task 7: Testing & Validation

**Requirements**:
- Test update flow end-to-end
- Test signature verification
- Test rollback scenarios
- Test offline behavior
- Test on all platforms

**Subtasks**:
- [ ] Create test GitHub releases
- [ ] Test update detection
- [ ] Test download and install
- [ ] Test signature verification failure
- [ ] Test rollback
- [ ] Test across macOS, Windows, Linux
- [ ] Document testing procedures

**Effort**: 2 days
**Dependencies**: All above tasks

---

## 🔐 Security Considerations

### Signature Verification

**Algorithm**: Ed25519 (Tauri default)

**Key Management**:
- **Private Key**: Stored in GitHub Secrets, never committed to repo
- **Public Key**: Embedded in tauri.conf.json, shipped with app
- **Rotation**: Document key rotation procedure

**Verification Process**:
1. Download binary and signature file
2. Verify signature using public key
3. Only install if signature valid
4. Log verification failures for monitoring

### Attack Vectors & Mitigations

| Attack Vector | Mitigation |
|---------------|------------|
| Man-in-the-middle | HTTPS + signature verification |
| Compromised GitHub | Signature verification (attacker needs private key) |
| Downgrade attack | Version comparison (never install older version) |
| Binary tampering | Signature verification fails |
| Private key leak | Key rotation procedure + new release |

---

## 📦 Release Strategy

### Version Numbering

**Format**: `MAJOR.MINOR.PATCH` (Semantic Versioning)

**Rules**:
- **MAJOR**: Breaking changes (API changes, migration required)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

**Examples**:
- `0.1.0` - Initial alpha release
- `0.1.1` - Bug fix
- `0.2.0` - New feature (gossip mesh enabled)
- `1.0.0` - Production ready

### Release Channels

**Stable**: Production-ready releases
- Auto-update enabled by default
- Thorough testing required
- Released monthly (or as needed for critical fixes)

**Beta** (Future): Early access features
- Opt-in auto-updates
- Weekly releases
- User feedback encouraged

### Release Checklist

- [ ] Update version in Cargo.toml
- [ ] Update version in package.json
- [ ] Update CHANGELOG.md
- [ ] Run full test suite
- [ ] Build and test locally
- [ ] Create Git tag (vX.Y.Z)
- [ ] Push tag to trigger release workflow
- [ ] Verify GitHub release created
- [ ] Verify binaries signed
- [ ] Test update from previous version
- [ ] Announce release

---

## 📈 Metrics & Monitoring

### Key Metrics

- **Update Success Rate**: % of successful updates
- **Update Failure Reasons**: Why updates fail
- **Rollback Rate**: % of updates that rollback
- **Update Latency**: Time from release to user update
- **Version Distribution**: What versions users are on

### Monitoring

**Logging**:
- All update attempts (success/failure)
- Signature verification results
- Rollback events
- Version transitions

**Alerts**:
- High update failure rate (>5%)
- Signature verification failures
- Unusual rollback rate

---

## ⏱️ Timeline

| Task | Duration | Start After |
|------|----------|-------------|
| Task 1: Tauri Config | 2 hours | Immediate |
| Task 2: GitHub Workflow | 1 day | Task 1 |
| Task 3: Version Management | 4 hours | Task 1 |
| Task 4: Update UI | 1 day | Task 1, 2 |
| Task 5: Update Manager | 2 days | Task 1, 3 |
| Task 6: Rollback | 1 day | Task 5 |
| Task 7: Testing | 2 days | All tasks |

**Total Estimated Time**: 7-8 days

---

# PHASE 2: MESH NETWORKING

## 📊 Overview

**Goal**: Enable peer-to-peer communication, file sharing, and collaboration without centralized servers.

**Key Requirements**:
- ✅ Decentralized (no single point of failure)
- ✅ NAT traversal (works behind routers)
- ✅ Offline-first (local operations always work)
- ✅ Encrypted (end-to-end encryption)
- ✅ Resilient (handles network partitions)
- ✅ Scalable (thousands of peers)

## 🏗️ Architecture

### Network Topology

```
        Bootstrap Nodes (Initial Discovery)
               ↓
    ┌──────────┴──────────┐
    │                     │
  Peer A ←──────────────→ Peer B
    │                     │
    └──────────┬──────────┘
               │
          ┌────┴────┐
          │         │
        Peer C   Peer D

• Peers connect directly to each other
• Gossip protocol propagates messages
• No central server required after bootstrap
```

### Protocol Stack

```
┌─────────────────────────────────────┐
│     Application Layer               │
│  • File Sharing                     │
│  • Messaging                        │
│  • Collaboration                    │
├─────────────────────────────────────┤
│     Gossip Overlay                  │
│  • Message Propagation (Plumtree)   │
│  • CRDT State Management            │
│  • Four-Word Addressing             │
├─────────────────────────────────────┤
│     Transport Layer                 │
│  • QUIC Protocol (ant-quic)         │
│  • NAT Traversal                    │
│  • Connection Management            │
├─────────────────────────────────────┤
│     Cryptography Layer              │
│  • ML-DSA Signatures                │
│  • ML-KEM Key Exchange              │
│  • ChaCha20-Poly1305 Encryption     │
└─────────────────────────────────────┘
```

## 🔧 Implementation Tasks

### Task 1: Bootstrap System

**Goal**: Enable new peers to discover the network

**Options**:

**Option A: Hardcoded Bootstrap Nodes**
- Pros: Simple, reliable
- Cons: Centralization point, single point of failure

**Option B: DNS Bootstrap**
- Pros: Can update bootstrap list without app updates
- Cons: Requires DNS infrastructure

**Option C: Hybrid (RECOMMENDED)**
- Hardcoded fallback nodes
- DNS for primary bootstrap
- Peer cache for offline-first

**Implementation**:

**File**: `communitas-desktop/src/bootstrap.rs` (new)

```rust
pub struct BootstrapManager {
    // Hardcoded fallback bootstrap nodes
    fallback_nodes: Vec<SocketAddr>,

    // DNS-based bootstrap
    dns_bootstrap: Option<String>,

    // Cached peers from previous sessions
    peer_cache: PeerCache,
}

impl BootstrapManager {
    pub async fn discover_peers(&self) -> Result<Vec<PeerInfo>> {
        // Try DNS bootstrap first
        if let Some(peers) = self.dns_bootstrap().await? {
            return Ok(peers);
        }

        // Fallback to hardcoded nodes
        if let Some(peers) = self.hardcoded_bootstrap().await? {
            return Ok(peers);
        }

        // Last resort: peer cache
        self.peer_cache.get_active_peers()
    }
}
```

**Subtasks**:
- [ ] Design bootstrap node discovery
- [ ] Implement DNS bootstrap (optional)
- [ ] Implement hardcoded fallback nodes
- [ ] Implement peer cache persistence
- [ ] Add bootstrap retry logic
- [ ] Test bootstrap across network conditions

**Effort**: 2 days
**Dependencies**: None

---

### Task 2: NAT Traversal

**Goal**: Enable peers behind routers/firewalls to connect

**Current State**:
- ✅ QUIC protocol (ant-quic) has some NAT traversal built-in
- ✅ IPv4 + IPv6 support with Happy Eyeballs

**Additional Needs**:
- STUN-like functionality for discovering public IP/port
- Relay fallback for symmetric NATs (TURN-like)

**Approaches**:

**Option A: Rely on QUIC's Built-in NAT Traversal**
- Pros: Simple, no additional infrastructure
- Cons: May not work for all NAT types

**Option B: Add STUN Support (RECOMMENDED)**
- Pros: Works for most NAT types
- Cons: Requires STUN servers

**Option C: Full TURN Relay**
- Pros: Works for all NAT types
- Cons: High bandwidth cost, centralization

**Implementation Strategy**: Start with Option A, add B if needed

**File**: `communitas-desktop/src/nat_traversal.rs` (new)

**Subtasks**:
- [ ] Test QUIC NAT traversal in real networks
- [ ] Measure success rate of direct connections
- [ ] Implement STUN client (if needed)
- [ ] Add relay fallback (if needed)
- [ ] Document NAT traversal behavior

**Effort**: 3 days
**Dependencies**: Task 1

---

### Task 3: Peer Discovery & Routing

**Goal**: Peers find each other and route messages efficiently

**Current State**:
- ✅ Gossip protocol (Plumtree) for message propagation
- ✅ Four-word addressing
- ✅ Contact management

**Needed**:
- Efficient peer discovery beyond bootstrap
- Routing table management
- Connection health monitoring

**File**: `communitas-desktop/src/peer_discovery.rs` (new)

```rust
pub struct PeerDiscovery {
    // Known peers and their metadata
    routing_table: HashMap<FourWordAddr, PeerInfo>,

    // Active connections
    connections: HashMap<FourWordAddr, Connection>,

    // Peer health monitoring
    health_monitor: HealthMonitor,
}

impl PeerDiscovery {
    // Discover peers through gossip
    pub async fn discover_peers(&mut self) -> Result<Vec<PeerInfo>> {
        // Query gossip overlay for peer announcements
        let announcements = self.gossip.get_peer_announcements().await?;

        // Update routing table
        self.update_routing_table(announcements).await?;

        // Return active peers
        Ok(self.get_active_peers())
    }

    // Find route to specific peer
    pub fn find_route(&self, target: &FourWordAddr) -> Option<Route> {
        // Direct connection if available
        if self.connections.contains_key(target) {
            return Some(Route::Direct(target.clone()));
        }

        // Multi-hop route through intermediate peers
        self.find_multihop_route(target)
    }
}
```

**Subtasks**:
- [ ] Implement peer announcement mechanism
- [ ] Build routing table
- [ ] Add peer health monitoring
- [ ] Implement multihop routing (optional)
- [ ] Add peer reputation system (future)
- [ ] Test discovery with 100+ peers

**Effort**: 3 days
**Dependencies**: Task 1, Task 2

---

### Task 4: File Sharing System

**Goal**: Share files between peers efficiently and reliably

**Requirements**:
- Large file support (GB+)
- Chunked transfers
- Resume capability
- Integrity verification
- Progress tracking

**Architecture**:

```
File Sharing Flow:

1. Sender announces file availability
   ↓
2. Receiver requests file
   ↓
3. File chunked into blocks (1MB each)
   ↓
4. Chunks transferred over QUIC
   ↓
5. Each chunk verified (BLAKE3 hash)
   ↓
6. Receiver reassembles file
   ↓
7. Final integrity check
```

**File**: `communitas-desktop/src/file_sharing.rs` (new)

```rust
const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks

pub struct FileShare {
    // File metadata
    file_id: FileId,
    file_name: String,
    file_size: u64,
    chunk_hashes: Vec<Blake3Hash>,

    // Transfer state
    chunks_received: BitSet,
    progress: Arc<AtomicU64>,
}

impl FileShare {
    pub async fn send_file(
        &self,
        path: PathBuf,
        recipient: FourWordAddr,
    ) -> Result<()> {
        // 1. Compute chunk hashes
        let chunks = self.chunk_file(&path).await?;

        // 2. Send file metadata
        self.send_metadata(recipient, chunks.metadata()).await?;

        // 3. Send chunks
        for (idx, chunk) in chunks.iter().enumerate() {
            self.send_chunk(recipient, idx, chunk).await?;
            self.update_progress(idx);
        }

        Ok(())
    }

    pub async fn receive_file(
        &mut self,
        output_path: PathBuf,
    ) -> Result<()> {
        // 1. Receive metadata
        let metadata = self.receive_metadata().await?;

        // 2. Receive chunks
        let mut chunks = vec![Vec::new(); metadata.num_chunks];
        for idx in 0..metadata.num_chunks {
            chunks[idx] = self.receive_chunk(idx).await?;

            // Verify chunk hash
            if !self.verify_chunk(idx, &chunks[idx]) {
                return Err(Error::ChunkVerificationFailed(idx));
            }

            self.chunks_received.set(idx);
            self.update_progress(idx);
        }

        // 3. Reassemble file
        self.reassemble_file(&output_path, chunks).await?;

        // 4. Final verification
        self.verify_complete_file(&output_path)?;

        Ok(())
    }
}
```

**Subtasks**:
- [ ] Implement file chunking
- [ ] Implement chunk transfer over QUIC
- [ ] Add integrity verification (BLAKE3)
- [ ] Implement resume capability
- [ ] Add progress tracking
- [ ] Create file transfer UI
- [ ] Test with large files (1GB+)
- [ ] Test resume after interruption

**Effort**: 4 days
**Dependencies**: Task 2, Task 3

---

### Task 5: Messaging System

**Goal**: Real-time chat between peers

**Requirements**:
- 1-to-1 messaging
- Group messaging
- Message history
- Offline message queue
- Read receipts
- Typing indicators

**Current State**:
- ✅ core_messages_send command
- ✅ CRDT storage for messages
- ✅ Message encryption

**Needed**:
- Real-time delivery notifications
- Message sync across devices
- Proper message ordering

**File**: `communitas-desktop/src/messaging.rs` (enhance existing)

**Subtasks**:
- [ ] Implement real-time message delivery
- [ ] Add message history sync
- [ ] Implement offline message queue
- [ ] Add read receipts
- [ ] Add typing indicators
- [ ] Create messaging UI components
- [ ] Test message delivery reliability

**Effort**: 3 days
**Dependencies**: Task 3

---

### Task 6: Collaboration Features

**Goal**: Enable project collaboration between peers

**Requirements**:
- Shared workspaces
- Document collaboration
- Version control integration
- Activity feeds
- Permission management

**Architecture**:

```
Project Workspace:
├── Shared Files (synced via file sharing)
├── Shared State (CRDT-based collaboration)
├── Activity Log (gossip-based events)
└── Members (access control)
```

**File**: `communitas-desktop/src/collaboration.rs` (new)

**Subtasks**:
- [ ] Define workspace data model
- [ ] Implement workspace creation
- [ ] Add member management
- [ ] Implement file sync for workspaces
- [ ] Add activity tracking
- [ ] Create collaboration UI
- [ ] Test multi-peer collaboration

**Effort**: 5 days
**Dependencies**: Task 4, Task 5

---

### Task 7: Network Resilience

**Goal**: Handle network failures gracefully

**Requirements**:
- Detect network partitions
- Automatic reconnection
- Message queue for offline peers
- Conflict resolution (CRDT helps)
- Network status UI

**File**: `communitas-desktop/src/network_resilience.rs` (new)

**Subtasks**:
- [ ] Implement connection health monitoring
- [ ] Add automatic reconnection logic
- [ ] Implement offline message queue
- [ ] Add network partition detection
- [ ] Create network status UI
- [ ] Test resilience scenarios

**Effort**: 2 days
**Dependencies**: Task 3

---

### Task 8: Performance Optimization

**Goal**: Ensure mesh network scales to thousands of peers

**Requirements**:
- Efficient gossip propagation
- Connection pooling
- Message batching
- Bandwidth management
- CPU optimization

**Subtasks**:
- [ ] Profile gossip message overhead
- [ ] Optimize CRDT merge operations
- [ ] Implement connection pooling
- [ ] Add message batching
- [ ] Test with 1000+ simulated peers
- [ ] Measure and optimize bandwidth usage

**Effort**: 3 days
**Dependencies**: All above tasks

---

### Task 9: Testing & Validation

**Requirements**:
- Multi-peer testing
- Network condition simulation
- Stress testing
- Security testing

**Subtasks**:
- [ ] Create multi-peer test environment
- [ ] Simulate various network conditions
- [ ] Stress test with many peers
- [ ] Security audit of mesh protocol
- [ ] Test NAT traversal scenarios
- [ ] Document testing procedures

**Effort**: 3 days
**Dependencies**: All above tasks

---

## 🔐 Security Considerations

### Encryption

**In Transit**:
- QUIC provides TLS 1.3 encryption
- ML-KEM for key exchange
- ChaCha20-Poly1305 for messages

**At Rest**:
- Local storage encrypted
- Peer cache encrypted
- Message history encrypted

### Authentication

**Peer Authentication**:
- Four-word identities backed by ML-DSA signatures
- Challenge-response for connection establishment
- Signature verification on all messages

### Privacy

**Metadata Protection**:
- Onion routing for metadata privacy (future)
- No centralized logging
- Local-only contact lists

**Anti-Spam**:
- Rate limiting per peer
- Reputation system (future)
- Block/mute functionality

---

## 📈 Metrics & Monitoring

### Key Metrics

- **Connection Success Rate**: % of successful peer connections
- **NAT Traversal Success**: % of direct connections vs relayed
- **Message Latency**: Time from send to receive
- **File Transfer Speed**: MB/s for file sharing
- **Network Partition Events**: Frequency and duration
- **Peer Count**: Number of active peers

### Monitoring

**Logging**:
- Connection attempts and results
- NAT traversal outcomes
- Message delivery status
- File transfer progress
- Network partition events

**Alerts**:
- High connection failure rate
- NAT traversal failures
- Message delivery delays
- Network partitions

---

## ⏱️ Timeline

| Task | Duration | Start After |
|------|----------|-------------|
| Task 1: Bootstrap | 2 days | Phase 1 complete |
| Task 2: NAT Traversal | 3 days | Task 1 |
| Task 3: Peer Discovery | 3 days | Task 1, 2 |
| Task 4: File Sharing | 4 days | Task 2, 3 |
| Task 5: Messaging | 3 days | Task 3 |
| Task 6: Collaboration | 5 days | Task 4, 5 |
| Task 7: Resilience | 2 days | Task 3 |
| Task 8: Performance | 3 days | All above |
| Task 9: Testing | 3 days | All above |

**Total Estimated Time**: 20-25 days

---

## 🎯 Success Criteria

### Phase 1: Self-Updating

- [ ] Apps auto-update from GitHub releases
- [ ] 95%+ update success rate
- [ ] Rollback works on all platforms
- [ ] Zero security vulnerabilities in update process
- [ ] Update latency < 24 hours for 90% of users

### Phase 2: Mesh Networking

- [ ] Peers can discover each other via bootstrap
- [ ] 80%+ NAT traversal success rate
- [ ] File sharing works for files up to 10GB
- [ ] Real-time messaging < 500ms latency
- [ ] Network handles 1000+ peers
- [ ] Offline mode works seamlessly

---

## 🚀 Launch Strategy

### Alpha Release (v0.1.0)
- Self-updating working
- Basic peer connection
- Simple messaging

### Beta Release (v0.2.0)
- File sharing working
- NAT traversal optimized
- Basic collaboration

### Production Release (v1.0.0)
- All features stable
- Performance optimized
- Security audited
- Comprehensive testing

---

## 📚 References

### Tauri Updater
- https://tauri.app/v1/guides/distribution/updater

### QUIC Protocol
- https://www.rfc-editor.org/rfc/rfc9000.html

### Gossip Protocols
- Plumtree: https://asc.di.fct.unl.pt/~jleitao/pdf/srds07-leitao.pdf

### NAT Traversal
- STUN: https://www.rfc-editor.org/rfc/rfc5389.html
- ICE: https://www.rfc-editor.org/rfc/rfc8445.html

---

## ✅ Next Steps

1. **Review this plan** - Identify any gaps or concerns
2. **Prioritize tasks** - Decide on exact task order
3. **Set milestones** - Define checkpoints for progress tracking
4. **Begin Phase 1** - Start with self-updating system
5. **Parallel work** - Some mesh networking research can happen during Phase 1

---

**Document Version**: 1.0
**Last Updated**: 2025-10-15
**Owner**: David Irvine
**Status**: Ready for Review
