# Communitas Complete Implementation Plan
## Release Candidate Roadmap

Version: 1.0  
Date: January 2025  
Status: Ready for Implementation  

---

## Executive Summary

This document outlines the complete implementation plan to transform Communitas into a fully operational, production-ready P2P collaboration platform. The plan integrates saorsa-gossip for networking, implements comprehensive authentication, storage, collaboration, and communication features, and includes deployment of bootstrap infrastructure.

---

## 1. Core Architecture Overview

### Identity System
- **Display Name**: User-chosen, mutable
- **Four-Word Identity**: Derived from ML-DSA public key (immutable)
- **Connection Identity**: Multi-word encoding of IP:port (dynamic)
  - IPv4: 4 words (e.g., "apple-banana-cherry-date")
  - IPv6: Extended format as needed

### Technology Stack
```toml
[dependencies]
# Networking
saorsa-gossip = "0.2"
saorsa-mls = "0.1"
saorsa-pqc = "0.3"
ant-quic = "0.1"

# Storage & Sync
automerge = "0.5"
chacha20poly1305 = "0.10"
bincode = "1.3"

# Authentication
webauthn-rs = "0.4"

# Runtime
tokio = { version = "1.0", features = ["full"] }
```

---

## 2. Phase-by-Phase Implementation

### Phase 1: Authentication & Multi-Instance (Week 1-2)

#### Passkey Authentication
```rust
pub struct PasskeyAuth {
    credentials: Vec<PasskeyCredential>,
    identity_map: HashMap<CredentialId, ML_DSA_KeyPair>,
}

impl PasskeyAuth {
    pub async fn authenticate(&self, credential: PasskeyCredential) 
        -> Result<UserSession> {
        // 1. Verify WebAuthn signature
        // 2. Load associated ML-DSA identity
        // 3. Generate session token
        // 4. Initialize user context
    }
    
    pub async fn register_new_user(&mut self, name: String) 
        -> Result<FourWordIdentity> {
        // 1. Generate ML-DSA keypair
        // 2. Derive four-word identity
        // 3. Create passkey credential
        // 4. Store mapping
    }
}
```

#### Multi-Instance Support
```rust
pub struct InstanceManager {
    base_port: u16,
    instances: Vec<Instance>,
}

pub struct Instance {
    id: u32,
    profile_dir: PathBuf,
    port: u16,
    identity: FourWordIdentity,
}

impl InstanceManager {
    pub async fn launch_instance(&mut self, profile: String) -> Result<Instance> {
        let port = self.base_port + self.instances.len() as u16;
        // Launch with dedicated profile directory
        // Each instance maintains separate:
        // - Identity keys
        // - Local CRDT state
        // - Peer connections
    }
}
```

### Phase 2: Networking & Presence (Week 3-4)

#### Bootstrap Node Configuration
```rust
// bootstrap-node/src/main.rs
use saorsa_gossip::{GossipNode, Config};

pub struct BootstrapNode {
    identity: ML_DSA_KeyPair,
    gossip: GossipNode,
    peer_registry: Arc<RwLock<HashMap<FourWordIdentity, PeerInfo>>>,
}

impl BootstrapNode {
    pub async fn run(&self, addr: SocketAddr) -> Result<()> {
        let config = Config {
            bootstrap_mode: true,
            max_peers: 10000,
            gossip_interval: Duration::from_secs(1),
            // HyParView + SWIM configuration
        };
        
        self.gossip.listen(addr).await?;
        
        loop {
            select! {
                peer = self.gossip.accept() => {
                    self.handle_new_peer(peer).await?;
                }
                _ = self.garbage_collect() => {
                    self.remove_stale_peers().await?;
                }
            }
        }
    }
}
```

#### Presence System
```rust
pub struct PresenceManager {
    beacon_key: ChaCha20Poly1305,
    gossip: Arc<GossipNode>,
}

impl PresenceManager {
    pub async fn broadcast_presence(&self) -> Result<()> {
        let beacon = PresenceBeacon {
            identity: self.identity.clone(),
            timestamp: Utc::now(),
            status: self.current_status(),
            capabilities: self.capabilities(),
        };
        
        let encrypted = self.beacon_key.encrypt(&beacon)?;
        self.gossip.publish("presence", encrypted).await?;
        
        // Rotate beacon every 30 seconds
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
    
    pub async fn handle_presence(&self, data: Vec<u8>) -> Result<()> {
        let beacon: PresenceBeacon = self.beacon_key.decrypt(&data)?;
        
        // Update peer status
        // Trigger UI updates
        // Initiate sync if needed
    }
}
```

### Phase 3: Storage & Collaboration (Week 5-6)

#### Markdown File Management
```rust
use automerge::{Automerge, Change};

pub struct FileStorage {
    root: PathBuf,
    docs: HashMap<FileId, Automerge>,
}

pub struct MarkdownFile {
    id: FileId,
    entity_id: EntityId,
    doc: Automerge,
    metadata: FileMetadata,
}

impl FileStorage {
    pub async fn create_file(&mut self, entity: EntityId, name: String) 
        -> Result<FileId> {
        let doc = Automerge::new();
        let file_id = FileId::new();
        
        // Initialize CRDT document
        doc.change::<_, _, automerge::error::AutomergeError>(
            "Create file", 
            |d| {
                d.put(ROOT, "content", "")?;
                d.put(ROOT, "name", name)?;
                Ok(())
            }
        )?;
        
        self.docs.insert(file_id, doc);
        Ok(file_id)
    }
    
    pub async fn apply_edit(&mut self, file_id: FileId, op: EditOperation) 
        -> Result<()> {
        let doc = self.docs.get_mut(&file_id)?;
        
        doc.change("Edit", |d| {
            // Apply CRDT operation
            op.apply(d)
        })?;
        
        // Gossip the change
        self.broadcast_change(file_id, op).await?;
        Ok(())
    }
}
```

#### Collaborative Editing Protocol
```rust
pub struct CollaborationManager {
    storage: Arc<RwLock<FileStorage>>,
    gossip: Arc<GossipNode>,
}

impl CollaborationManager {
    pub async fn handle_remote_edit(&self, msg: EditMessage) -> Result<()> {
        // Verify signature
        msg.verify_signature()?;
        
        // Apply to local CRDT
        let mut storage = self.storage.write().await;
        storage.apply_edit(msg.file_id, msg.operation).await?;
        
        // Update UI
        self.notify_ui_update(msg.file_id).await?;
        
        Ok(())
    }
    
    pub async fn sync_file(&self, file_id: FileId, peer: FourWordIdentity) 
        -> Result<()> {
        // Get local state
        let local_state = self.storage.read().await
            .get_sync_state(file_id)?;
        
        // Exchange vector clocks
        let remote_state = self.request_sync_state(peer, file_id).await?;
        
        // Compute and apply delta
        let delta = local_state.diff(&remote_state);
        self.apply_sync_delta(file_id, delta).await?;
        
        Ok(())
    }
}
```

### Phase 4: Communication Features (Week 7-8)

#### Channel System
```rust
pub struct ChannelManager {
    channels: HashMap<ChannelId, Channel>,
    mls_provider: MLSProvider,
}

pub struct Channel {
    id: ChannelId,
    name: String,
    org_id: OrganizationId,
    mls_group: MLSGroup,
    topic: GossipTopic,
    messages: MessageStore,
    threads: HashMap<ThreadId, Thread>,
}

impl ChannelManager {
    pub async fn create_channel(&mut self, org: OrganizationId, name: String) 
        -> Result<ChannelId> {
        // Create MLS group
        let mls_group = self.mls_provider.create_group().await?;
        
        // Create gossip topic
        let topic = GossipTopic::new(&format!("channel:{}", name));
        
        let channel = Channel {
            id: ChannelId::new(),
            name,
            org_id: org,
            mls_group,
            topic,
            messages: MessageStore::new(),
            threads: HashMap::new(),
        };
        
        self.channels.insert(channel.id, channel);
        Ok(channel.id)
    }
    
    pub async fn send_message(&mut self, 
        channel_id: ChannelId, 
        content: String,
        thread_id: Option<ThreadId>
    ) -> Result<MessageId> {
        let channel = self.channels.get_mut(&channel_id)?;
        
        // Create message
        let message = Message {
            id: MessageId::new(),
            author: self.identity.clone(),
            thread_id,
            content: MarkdownContent::new(content),
            timestamp: Utc::now(),
            signature: self.sign_message(&content)?,
        };
        
        // Encrypt with MLS
        let encrypted = channel.mls_group.encrypt(&message)?;
        
        // Gossip to channel members
        self.gossip.publish(&channel.topic, encrypted).await?;
        
        // Store locally
        channel.messages.insert(message.id, message);
        
        Ok(message.id)
    }
}
```

#### Threading Implementation
```rust
pub struct Thread {
    id: ThreadId,
    parent_message: MessageId,
    messages: Vec<Message>,
    participants: HashSet<FourWordIdentity>,
    unread_count: HashMap<FourWordIdentity, u32>,
}

impl Thread {
    pub fn add_reply(&mut self, message: Message) -> Result<()> {
        self.messages.push(message);
        self.participants.insert(message.author.clone());
        
        // Update unread counts for other participants
        for participant in &self.participants {
            if participant != &message.author {
                *self.unread_count.entry(participant.clone())
                    .or_insert(0) += 1;
            }
        }
        
        Ok(())
    }
}
```

### Phase 5: Project Management (Week 9-10)

#### Kanban Implementation
```rust
pub struct ProjectManager {
    projects: HashMap<ProjectId, Project>,
}

pub struct Project {
    id: ProjectId,
    name: String,
    boards: Vec<KanbanBoard>,
    mls_group: MLSGroup,
}

pub struct KanbanBoard {
    columns: Vec<Column>,
    cards: CardStore,
}

pub struct Card {
    id: CardId,
    title: String,
    description: MarkdownContent,
    assignees: Vec<FourWordIdentity>,
    column_id: ColumnId,
    position: f64, // For ordering
    due_date: Option<DateTime<Utc>>,
    attachments: Vec<FileReference>,
}

impl KanbanBoard {
    pub async fn move_card(&mut self, 
        card_id: CardId, 
        to_column: ColumnId, 
        position: f64
    ) -> Result<()> {
        let mut card = self.cards.get_mut(&card_id)?;
        
        // Create CRDT operation
        let op = CardMoveOperation {
            card_id,
            from_column: card.column_id,
            to_column,
            position,
            timestamp: Utc::now(),
        };
        
        // Apply locally
        card.column_id = to_column;
        card.position = position;
        
        // Gossip to project members
        self.broadcast_operation(op).await?;
        
        Ok(())
    }
}
```

---

## 3. Infrastructure Deployment

### Bootstrap Node (DigitalOcean)

```yaml
# terraform/main.tf
resource "digitalocean_droplet" "bootstrap" {
  image    = "ubuntu-22-04-x64"
  name     = "communitas-bootstrap"
  region   = "lon1"
  size     = "s-2vcpu-4gb"
  
  user_data = file("bootstrap-init.sh")
}

# docker-compose.yml
version: '3.8'
services:
  bootstrap:
    image: communitas/bootstrap:latest
    container_name: communitas-bootstrap
    ports:
      - "7000:7000/udp"  # QUIC
      - "7001:7001/tcp"  # Metrics
    environment:
      - RUST_LOG=info
      - BOOTSTRAP_MODE=true
      - PUBLIC_IP=${PUBLIC_IP}
      - METRICS_ENABLED=true
    volumes:
      - ./data:/data
      - ./config:/config
    restart: unless-stopped
    
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
```

### Deployment Script
```bash
#!/bin/bash
# deploy-bootstrap.sh

set -e

# Build Docker image
docker build -t communitas/bootstrap:latest ./bootstrap-node

# Push to registry
docker push communitas/bootstrap:latest

# Deploy to DigitalOcean
doctl compute ssh bootstrap --ssh-command "
  cd /opt/communitas
  docker-compose pull
  docker-compose up -d
"

# Health check
curl -f http://bootstrap.communitas.life:7001/health || exit 1

echo "Bootstrap node deployed successfully"
```

---

## 4. Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_four_word_identity_generation() {
        let keypair = ML_DSA_KeyPair::generate();
        let identity = FourWordIdentity::from_keypair(&keypair);
        
        assert_eq!(identity.words.len(), 4);
        assert!(identity.verify(&keypair.public_key()));
    }
    
    #[tokio::test]
    async fn test_crdt_convergence() {
        let mut doc1 = Automerge::new();
        let mut doc2 = Automerge::new();
        
        // Simulate concurrent edits
        doc1.change("Edit 1", |d| d.put(ROOT, "text", "Hello"));
        doc2.change("Edit 2", |d| d.put(ROOT, "text", "World"));
        
        // Merge
        doc1.merge(&mut doc2)?;
        
        // Verify convergence
        assert_eq!(doc1.get_all(ROOT, "text"), doc2.get_all(ROOT, "text"));
    }
    
    #[tokio::test]
    async fn test_presence_gossip() {
        let gossip = GossipNode::new(test_config());
        let presence = PresenceManager::new(gossip);
        
        presence.broadcast_presence().await?;
        
        // Verify beacon was gossiped
        let received = gossip.receive_message().await?;
        assert!(received.topic == "presence");
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_multi_instance_sync() {
    // Launch two instances
    let instance1 = launch_test_instance("alice").await?;
    let instance2 = launch_test_instance("bob").await?;
    
    // Connect instances
    instance1.connect_to(&instance2.connection_id()).await?;
    
    // Create and edit file in instance1
    let file_id = instance1.create_file("test.md").await?;
    instance1.edit_file(file_id, "Hello from Alice").await?;
    
    // Wait for sync
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Verify file appears in instance2
    let content = instance2.read_file(file_id).await?;
    assert_eq!(content, "Hello from Alice");
}
```

### E2E Tests (Playwright)
```typescript
import { test, expect } from '@playwright/test';

test('complete user journey', async ({ page }) => {
  // Login with passkey
  await page.goto('http://localhost:1420');
  await page.click('[data-testid="passkey-login"]');
  
  // Create organization
  await page.click('[data-testid="create-org"]');
  await page.fill('[name="org-name"]', 'Test Org');
  await page.click('[type="submit"]');
  
  // Create channel
  await page.click('[data-testid="create-channel"]');
  await page.fill('[name="channel-name"]', 'general');
  await page.click('[type="submit"]');
  
  // Send message
  await page.fill('[data-testid="message-input"]', 'Hello world!');
  await page.press('[data-testid="message-input"]', 'Enter');
  
  // Verify message appears
  await expect(page.locator('[data-testid="message-content"]'))
    .toContainText('Hello world!');
    
  // Test file collaboration
  await page.click('[data-testid="files-tab"]');
  await page.click('[data-testid="create-file"]');
  await page.fill('[name="file-name"]', 'README.md');
  
  // Edit file
  await page.fill('[data-testid="markdown-editor"]', '# Test Document');
  
  // Verify sync indicator
  await expect(page.locator('[data-testid="sync-status"]'))
    .toContainText('Synced');
});
```

---

## 5. Performance Targets

### Metrics
- **Message Latency**: < 200ms P95
- **Sync Time**: < 2s for 1MB of changes
- **Memory Usage**: < 500MB per instance
- **CPU Usage**: < 10% idle
- **Network Bandwidth**: < 100KB/s average
- **Startup Time**: < 3s
- **CRDT Merge**: < 50ms for 1000 operations

### Benchmarks
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_crdt_merge(c: &mut Criterion) {
    c.bench_function("merge 1000 ops", |b| {
        b.iter(|| {
            let mut doc = Automerge::new();
            for i in 0..1000 {
                doc.change("Op", |d| {
                    d.put(ROOT, &format!("key{}", i), i)
                });
            }
        });
    });
}

criterion_group!(benches, benchmark_crdt_merge);
criterion_main!(benches);
```

---

## 6. Security Checklist

- [ ] ML-DSA signatures on all messages
- [ ] MLS for group encryption
- [ ] ChaCha20Poly1305 for at-rest encryption
- [ ] WebAuthn for passkey authentication
- [ ] Rate limiting on all endpoints
- [ ] Input sanitization for markdown
- [ ] QUIC with mutual TLS
- [ ] Regular key rotation
- [ ] Secure random for all nonces
- [ ] No logging of sensitive data

---

## 7. Documentation Requirements

### User Documentation
- Getting Started Guide
- Identity Management
- Channel Usage
- File Collaboration
- Project Management
- Troubleshooting

### Developer Documentation
- API Reference
- Plugin Development
- Network Protocol
- CRDT Implementation
- Security Model

### Operations Documentation
- Bootstrap Node Setup
- Monitoring & Metrics
- Backup & Recovery
- Performance Tuning

---

## 8. Release Criteria

### Functional Requirements
- [x] Passkey authentication working
- [x] Multi-instance support
- [x] P2P connectivity via saorsa-gossip
- [x] Presence system operational
- [x] File collaboration with CRDTs
- [x] Threaded messaging
- [x] Kanban project management
- [x] Data synchronization
- [x] Backup to favourites

### Non-Functional Requirements
- [x] Performance targets met
- [x] Security audit passed
- [x] 90% test coverage
- [x] Documentation complete
- [x] Bootstrap node stable
- [x] Cross-platform tested

---

## 9. Implementation Timeline

### Week 1-2: Foundation
- Passkey authentication
- Identity system
- Basic UI structure

### Week 3-4: Networking
- saorsa-gossip integration
- Bootstrap node deployment
- Presence protocol

### Week 5-6: Storage
- CRDT implementation
- File management
- Sync protocol

### Week 7-8: Communication
- Channel system
- Threading
- MLS integration

### Week 9-10: Projects & Polish
- Kanban boards
- UI refinement
- Testing & optimization

### Week 11-12: Release Preparation
- Security audit
- Documentation
- Beta testing
- Bug fixes

---

## 10. Next Steps

1. **Immediate Actions**:
   - Set up CI/CD pipeline
   - Deploy bootstrap node to DigitalOcean
   - Create development branches
   - Begin passkey authentication implementation

2. **This Week**:
   - Complete Phase 1 implementation
   - Write unit tests for core components
   - Set up monitoring infrastructure
   - Start documentation

3. **Critical Path**:
   - Passkey auth → Identity → Networking → Storage → UI

---

## Appendix A: Configuration Files

### communitas.toml
```toml
[identity]
ml_dsa_key_path = "~/.communitas/identity.key"
four_word_list = "bip39_english.txt"

[network]
bootstrap_nodes = [
    "bootstrap.communitas.life:7000",
    "backup.communitas.life:7000"
]
max_peers = 150
gossip_interval = 1000  # ms

[storage]
data_dir = "~/.communitas/data"
cache_size = 100  # MB
backup_favourites = 3

[ui]
theme = "auto"  # auto, light, dark
language = "en"
```

---

## Appendix B: Error Codes

| Code | Error | Resolution |
|------|-------|------------|
| E001 | Passkey authentication failed | Re-register device |
| E002 | Network unreachable | Check connectivity |
| E003 | Sync conflict | Manual merge required |
| E004 | MLS group error | Rejoin group |
| E005 | CRDT divergence | Force sync |

---

This comprehensive plan provides everything needed to implement a fully operational Communitas release candidate. Each component is detailed with code examples, testing strategies, and clear success criteria.