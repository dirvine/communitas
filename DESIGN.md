# Communitas - Technical Design Document

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│              Communitas React Frontend                   │
├─────────────────────────────────────────────────────────┤
│  ┌─────────┬─────────┬─────────┬─────────┬──────────┐  │
│  │Overview │Messages │Network  │Storage  │Advanced   │  │
│  └─────────┴─────────┴─────────┴─────────┴──────────┘  │
├─────────────────────────────────────────────────────────┤
│                    Tauri v2 IPC Layer                    │
├─────────────────────────────────────────────────────────┤
│                  Rust Backend (Tauri)                    │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │Chat Service │  │Diagnostics   │  │Test Harness  │  │
│  │            │  │Engine        │  │Controller   │  │
│  └─────────────┘  └──────────────┘  └──────────────┘  │
├─────────────────────────────────────────────────────────┤
│                    P2P Foundation                        │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │Identity Mgr │  │DHT Storage   │  │QUIC Network  │  │
│  │(4-word)     │  │(Kademlia)    │  │(ant-quic)    │  │
│  └─────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Component Design

### 1. Core Application Structure

```rust
// crates/communitas/src/lib.rs
pub struct CommuniasApp {
    // Core components
    chat_service: ChatService,
    diagnostics: DiagnosticsEngine,
    test_harness: TestHarness,
    
    // P2P integration
    node: Arc<P2pNode>,
    identity: Identity,
    
    // UI state
    ui_state: Arc<RwLock<UiState>>,
    current_tab: Tab,
}

#[derive(Clone)]
pub enum Tab {
    Overview,
    Messages,
    Network,
    Storage,
    Advanced,
}
```

### 2. Chat Service

```rust
// src/chat/service.rs
pub struct ChatService {
    // Message handling
    message_store: MessageStore,
    active_groups: HashMap<GroupId, ChatGroup>,
    
    // Network integration
    dht: Arc<DhtStorage>,
    transport: Arc<QuicTransport>,
    
    // Voice/video
    media_engine: Option<MediaEngine>,
}

pub struct ChatGroup {
    id: GroupId,
    name: String,
    participants: Vec<Identity>,
    threshold_key: ThresholdPublicKey,
    messages: VecDeque<Message>,
}

pub struct Message {
    id: MessageId,
    sender: Identity,
    content: MessageContent,
    timestamp: SystemTime,
    signatures: Vec<Signature>,
}

pub enum MessageContent {
    Text(String),
    File(FileMetadata),
    VoiceCall(CallInfo),
    VideoCall(CallInfo),
}
```

### 3. Diagnostics Engine

```rust
// src/diagnostics/engine.rs
pub struct DiagnosticsEngine {
    // Metrics collection
    network_metrics: NetworkMetrics,
    storage_metrics: StorageMetrics,
    peer_metrics: PeerMetrics,
    
    // Visualization data
    nat_traversal_viz: NatTraversalVisualizer,
    dht_operation_log: RingBuffer<DhtOperation>,
    packet_inspector: PacketInspector,
}

pub struct NetworkMetrics {
    connected_peers: usize,
    message_latency: Histogram,
    bandwidth_usage: BandwidthCounter,
    nat_type: NatType,
}

impl DiagnosticsEngine {
    pub fn start_collection(&mut self) {
        // Hook into P2P events
        self.node.on_peer_connected(|peer| {
            self.peer_metrics.record_connection(peer);
        });
        
        self.node.on_message_sent(|msg| {
            self.network_metrics.record_latency(msg);
        });
    }
}
```

### 4. Test Harness

```rust
// src/testing/harness.rs
pub struct TestHarness {
    // Property testing
    prop_test_runner: PropTestRunner,
    
    // Multi-node testing
    test_nodes: Vec<TestNode>,
    network_simulator: NetworkSimulator,
    
    // Test scenarios
    scenarios: Vec<Box<dyn TestScenario>>,
}

pub trait TestScenario {
    fn name(&self) -> &str;
    fn setup(&mut self, nodes: &mut [TestNode]) -> Result<()>;
    fn execute(&mut self) -> Result<TestResult>;
    fn teardown(&mut self) -> Result<()>;
}

// Property tests
proptest! {
    #[test]
    fn message_ordering_preserved(
        messages in prop::collection::vec(message_strategy(), 1..100)
    ) {
        // Test that messages arrive in causal order
        let mut harness = TestHarness::new();
        harness.run_message_ordering_test(messages)?;
    }
    
    #[test]
    fn network_partition_recovery(
        partition_size in 0.1f64..0.9f64
    ) {
        // Test network healing after partition
        let mut harness = TestHarness::new();
        harness.run_partition_test(partition_size)?;
    }
}
```

### 5. Tauri Backend & IPC

```rust
// src-tauri/src/main.rs
use tauri::Manager;

#[tauri::command]
async fn send_message(
    group_id: String,
    content: String,
    state: tauri::State<'_, AppState>
) -> Result<MessageId, String> {
    let app = state.app.lock().await;
    app.chat_service
        .send_message(&group_id, &content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_network_health(
    state: tauri::State<'_, AppState>
) -> Result<NetworkHealth, String> {
    let app = state.app.lock().await;
    Ok(app.diagnostics.get_network_health())
}

#[tauri::command]
async fn create_group(
    name: String,
    state: tauri::State<'_, AppState>
) -> Result<Group, String> {
    let app = state.app.lock().await;
    app.chat_service
        .create_group(&name)
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = AppState::new()?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            send_message,
            get_network_health,
            create_group,
            // ... more commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 6. React Frontend

```typescript
// src/App.tsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Tabs, Tab, Box, Paper } from '@mui/material';
import { Overview } from './components/Overview';
import { Messages } from './components/Messages';
import { Network } from './components/Network';
import { Storage } from './components/Storage';
import { Advanced } from './components/Advanced';

export const App: React.FC = () => {
  const [currentTab, setCurrentTab] = useState(0);
  const [networkHealth, setNetworkHealth] = useState<NetworkHealth | null>(null);

  useEffect(() => {
    const fetchHealth = async () => {
      const health = await invoke<NetworkHealth>('get_network_health');
      setNetworkHealth(health);
    };

    fetchHealth();
    const interval = setInterval(fetchHealth, 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <Box sx={{ width: '100%', height: '100vh', display: 'flex', flexDirection: 'column' }}>
      <Paper sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Tabs value={currentTab} onChange={(_, v) => setCurrentTab(v)}>
          <Tab label="Overview" />
          <Tab label="Messages" />
          <Tab label="Network" />
          <Tab label="Storage" />
          <Tab label="Advanced" />
        </Tabs>
      </Paper>
      
      <Box sx={{ flex: 1, p: 2 }}>
        {currentTab === 0 && <Overview health={networkHealth} />}
        {currentTab === 1 && <Messages />}
        {currentTab === 2 && <Network />}
        {currentTab === 3 && <Storage />}
        {currentTab === 4 && <Advanced />}
      </Box>
    </Box>
  );
};
```

```typescript
// src/components/Messages.tsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { 
  Box, TextField, IconButton, List, ListItem, 
  ListItemText, Paper, Typography 
} from '@mui/material';
import SendIcon from '@mui/icons-material/Send';

export const Messages: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [currentGroup, setCurrentGroup] = useState<string>('default');

  const handleSend = async () => {
    if (!inputValue.trim()) return;
    
    await invoke('send_message', {
      groupId: currentGroup,
      content: inputValue
    });
    
    setInputValue('');
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Paper sx={{ flex: 1, mb: 2, p: 2, overflow: 'auto' }}>
        <List>
          {messages.map((msg) => (
            <ListItem key={msg.id}>
              <ListItemText
                primary={msg.sender.fourWordAddress}
                secondary={msg.content}
              />
            </ListItem>
          ))}
        </List>
      </Paper>
      
      <Box sx={{ display: 'flex', gap: 1 }}>
        <TextField
          fullWidth
          variant="outlined"
          placeholder="Type a message..."
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && handleSend()}
        />
        <IconButton color="primary" onClick={handleSend}>
          <SendIcon />
        </IconButton>
      </Box>
    </Box>
  );
};
```

### 7. Network Integration

```rust
// src/network/integration.rs
pub struct NetworkIntegration {
    // Bootstrap configuration
    bootstrap_node: SocketAddr,
    
    // QUIC transport
    quic_config: QuicConfig,
    transport: Arc<QuicTransport>,
    
    // Identity management
    identity_manager: IdentityManager,
    four_word_address: FourWordAddress,
}

impl NetworkIntegration {
    pub async fn connect_to_bootstrap() -> Result<()> {
        let bootstrap = "quic.saorsalabs.com:8888".parse()?;
        
        let config = QuicConfig {
            enable_pqc: true,
            nat_traversal: true,
            keep_alive_interval: Duration::from_secs(30),
        };
        
        let transport = QuicTransport::new(config).await?;
        transport.connect(bootstrap).await?;
        
        Ok(())
    }
}
```

## Data Flow

### Message Flow
1. User types message in Messages tab
2. ChatService encrypts with group threshold key
3. Message stored in local cache
4. DHT PUT operation for persistence
5. QUIC multicast to online group members
6. Acknowledgments collected
7. UI updated with delivery status

### Diagnostic Data Flow
1. P2P events captured by hooks
2. DiagnosticsEngine processes events
3. Metrics updated in real-time
4. UI polls metrics at 10Hz
5. Visualizations rendered in respective tabs

## Storage Design

### Message Storage
- DHT key: `SHA3-256(group_id || message_id)`
- Value: Encrypted message with metadata
- TTL: 1 week (604,800 seconds)
- Replication: K=8 closest nodes

### File Storage
- Chunking: 256KB chunks
- Content addressing: BLAKE3 hash
- Metadata stored separately
- Progress tracking for transfers

## Security Model

### Identity
- Ed25519 keypair per user
- 4-word address derived from public key
- Persistent storage in encrypted keystore

### Group Management
- Threshold signatures for group operations
- M-of-N approval for adding members
- Encrypted group state in DHT

### Message Security
- End-to-end encryption with ML-KEM
- Forward secrecy via key rotation
- Message authentication with ML-DSA

## Performance Targets

### Latency
- Message delivery: <100ms (LAN), <500ms (WAN)
- UI responsiveness: <16ms frame time
- DHT operations: <200ms average

### Throughput
- Messages: 1000 msg/sec per group
- File transfers: 10MB/s (LAN)
- Concurrent users: 20 per group

### Resource Usage
- Memory: <200MB baseline
- CPU: <5% idle, <25% active
- Bandwidth: Adaptive to available

## Testing Strategy

### Unit Tests
- Component isolation
- Mock P2P interfaces
- Fast feedback cycle

### Integration Tests
```rust
#[tokio::test]
async fn test_multi_node_chat() {
    let mut nodes = TestHarness::spawn_nodes(5).await?;
    
    // Form group
    let group = nodes[0].create_group("Test Group").await?;
    for node in &nodes[1..] {
        node.join_group(group.id).await?;
    }
    
    // Exchange messages
    nodes[0].send_message(group.id, "Hello!").await?;
    
    // Verify delivery
    for node in &nodes[1..] {
        assert_eq!(
            node.get_messages(group.id).await?.last().unwrap().content,
            "Hello!"
        );
    }
}
```

### Property Tests
- Message ordering
- Network partitioning
- Storage consistency
- Concurrent operations

## Deployment

### Project Structure
```
communitas/
├── src-tauri/          # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs     # Tauri app entry
│       ├── chat/       # Chat service
│       ├── diagnostics/# Network diagnostics
│       └── commands.rs # Tauri commands
├── src/                # React frontend
│   ├── App.tsx
│   ├── components/
│   ├── hooks/
│   └── utils/
├── package.json
└── tsconfig.json
```

### Tauri Configuration
```json
// tauri.conf.json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://localhost:5173",
    "distDir": "../dist"
  },
  "package": {
    "productName": "Communitas",
    "version": "0.1.0"
  },
  "tauri": {
    "bundle": {
      "active": true,
      "identifier": "com.p2pfoundation.communitas",
      "icon": ["icons/icon.png"],
      "resources": [],
      "targets": ["dmg", "msi", "deb", "AppImage"],
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      }
    },
    "security": {
      "csp": "default-src 'self'; img-src 'self' data: https:"
    }
  }
}
```

### Cargo.toml (Backend)
```toml
[package]
name = "communitas"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0.0-alpha", features = [] }

[dependencies]
tauri = { version = "2.0.0-alpha", features = ["api-all"] }
saorsa-core = "0.1"
ant-quic = "0.5.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
proptest = "1.4"
```

### Configuration
```toml
# Default configuration
[network]
bootstrap_node = "quic.saorsalabs.com:8888"
enable_pqc = true

[chat]
max_group_size = 20
message_ttl_days = 7
max_file_size_mb = 5

[diagnostics]
enable_packet_inspection = false
metrics_interval_ms = 100
```

## Future Enhancements
1. Mobile support via egui
2. Web interface
3. Message search
4. Moderation tools
5. Performance warnings
6. Extended documentation