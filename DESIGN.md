# Communitas - Implementation Design Guide

## Overview

This document provides detailed implementation guidance for developers working on Communitas. It complements the master specification in `docs/system_spec.md` by focusing on concrete implementation patterns, code examples, and technical decisions.

For the high-level system architecture and requirements, see [System Specification](docs/system_spec.md).

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
│                    Saorsa P2P Stack                      │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │Identity Mgr │  │DHT Storage   │  │QUIC Network  │  │
│  │(4-word)     │  │(Kademlia)    │  │(ant-quic)    │  │
│  └─────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Implementation Patterns

### 1. Core Application Structure

```rust
// src-tauri/src/main.rs - Main Tauri Application
use tauri::{Manager, State};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub storage_engine: Arc<StorageEngine>,
    pub chat_service: Arc<ChatService>,
    pub network_client: Arc<NetworkClient>,
    pub diagnostics: Arc<DiagnosticsEngine>,
}

#[tauri::command]
async fn get_system_health(
    state: State<'_, Arc<AppState>>
) -> Result<SystemHealth, String> {
    let health = SystemHealth {
        storage: state.storage_engine.get_health().await?,
        network: state.network_client.get_status().await?,
        chat: state.chat_service.get_status().await?,
    };
    Ok(health)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = Arc::new(AppState::new()?);
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_health,
            // ... other commands
        ])
        .run(tauri::generate_context())
        .expect("error while running tauri application");
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

### 6. React Frontend Implementation

```typescript
// src/App.tsx - Main Application Component
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Tabs, Tab, Box, Paper, CircularProgress } from '@mui/material';
import { Overview } from './components/Overview';
import { Messages } from './components/Messages';
import { Network } from './components/Network';
import { Storage } from './components/Storage';
import { Advanced } from './components/Advanced';

interface SystemHealth {
  storage: StorageHealth;
  network: NetworkStatus;
  chat: ChatStatus;
}

export const App: React.FC = () => {
  const [currentTab, setCurrentTab] = useState(0);
  const [systemHealth, setSystemHealth] = useState<SystemHealth | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchHealth = async () => {
      try {
        const health = await invoke<SystemHealth>('get_system_health');
        setSystemHealth(health);
        setError(null);
      } catch (err) {
        setError(err as string);
      } finally {
        setLoading(false);
      }
    };

    fetchHealth();
    const interval = setInterval(fetchHealth, 5000); // Update every 5 seconds
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
        <CircularProgress />
      </Box>
    );
  }

  if (error) {
    return (
      <Box sx={{ p: 2 }}>
        <Paper sx={{ p: 2, bgcolor: 'error.main', color: 'error.contrastText' }}>
          Error: {error}
        </Paper>
      </Box>
    );
  }

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
        {currentTab === 0 && <Overview health={systemHealth} />}
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

## Error Handling Patterns

### Rust Backend Error Handling
```rust
// src-tauri/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommunitasError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<CommunitasError> for String {
    fn from(error: CommunitasError) -> String {
        error.to_string()
    }
}

// Usage in Tauri commands
#[tauri::command]
async fn create_group(
    name: String,
    state: State<'_, Arc<AppState>>
) -> Result<Group, CommunitasError> {
    // Validate input
    if name.trim().is_empty() {
        return Err(CommunitasError::Validation("Group name cannot be empty".to_string()));
    }

    // Create group through service
    let group = state.chat_service.create_group(&name).await?;
    Ok(group)
}
```

### React Error Boundaries
```typescript
// src/components/ErrorBoundary.tsx
import React, { Component, ReactNode } from 'react';
import { Box, Paper, Typography, Button } from '@mui/material';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <Box sx={{ p: 2 }}>
          <Paper sx={{ p: 2, bgcolor: 'error.main', color: 'error.contrastText' }}>
            <Typography variant="h6">Something went wrong</Typography>
            <Typography variant="body2" sx={{ mt: 1 }}>
              {this.state.error?.message}
            </Typography>
            <Button
              variant="contained"
              sx={{ mt: 2 }}
              onClick={() => this.setState({ hasError: false, error: undefined })}
            >
              Try Again
            </Button>
          </Paper>
        </Box>
      );
    }

    return this.props.children;
  }
}
```

## Testing Strategy

### Unit Tests
```rust
// src-tauri/src/chat/service.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_create_group_validation() {
        let service = ChatService::new();

        // Test empty name
        let result = service.create_group("").await;
        assert!(result.is_err());

        // Test valid name
        let result = service.create_group("Test Group").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Test Group");
    }
}
```

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

    // Verify delivery with timeout
    for node in &nodes[1..] {
        let messages = node.get_messages(group.id).await?;
        assert_eq!(messages.last().unwrap().content, "Hello!");
    }
}
```

### Property Tests
```rust
proptest! {
    #[test]
    fn message_ordering_preserved(
        messages in prop::collection::vec(message_strategy(), 1..100)
    ) {
        let mut harness = TestHarness::new();
        harness.test_message_ordering(messages)?;
    }

    #[test]
    fn network_partition_recovery(
        partition_size in 0.1f64..0.9f64
    ) {
        let mut harness = TestHarness::new();
        harness.test_partition_recovery(partition_size)?;
    }
}
```

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