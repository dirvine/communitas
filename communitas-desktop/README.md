# Communitas Desktop

Native desktop application for Communitas - the local-first, post-quantum collaboration platform.

## Overview

Communitas Desktop is a Tauri v2-based native application that brings secure, decentralized collaboration to Windows, macOS, and Linux. Built on React with a Rust backend, it provides a beautiful UI for messaging, file sharing, voice/video calling, and web publishing using human-verifiable Four-Word addressing.

## Features

- **Native Performance**: Built with Tauri v2 for minimal resource usage and maximum speed
- **Cross-Platform**: Windows, macOS (with Touch ID), and Linux support
- **Post-Quantum Security**: ML-DSA signatures and ML-KEM key exchange throughout
- **Four-Word Identities**: Human-readable addresses like "ocean-forest-moon-star"
- **P2P Networking**: Direct peer-to-peer communication via gossip overlay
- **Offline-First**: All functionality works without network, syncs when online
- **Auto-Updates**: Secure automatic updates with signature verification
- **CRDT Collaboration**: Real-time collaborative document editing
- **Virtual Disks**: Private, public, and shared storage per entity
- **Platform Integration**: macOS Touch ID, Windows Hello, Linux Secret Service

## Quick Start

### Prerequisites

- **Rust**: 1.85 or later
- **Node.js**: 20 or later
- **Platform Dependencies**:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools
  - **Linux**: webkit2gtk, libssl-dev, libgtk-3-dev

### Installation

#### From Binary (Recommended)

Download the latest release for your platform:
- **macOS**: `Communitas_0.1.1_aarch64.dmg` or `Communitas_0.1.1_x64.dmg`
- **Windows**: `Communitas_0.1.1_x64.msi`
- **Linux**: `communitas_0.1.1_amd64.AppImage` or `.deb`

#### From Source

```bash
# Clone repository
git clone https://github.com/dirvine/communitas.git
cd communitas

# Install dependencies
npm install

# Build frontend
npm run build

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Development Setup

### Project Structure

```
communitas-desktop/
├── src/
│   ├── commands/           # Organized Tauri command modules
│   │   └── auth.rs        # Authentication commands
│   ├── services/          # Business logic services
│   ├── security/          # Security and crypto
│   ├── core_commands.rs   # Core functionality commands
│   ├── gossip_commands.rs # P2P network commands
│   ├── crdt_manager.rs    # CRDT document management
│   ├── member_manager.rs  # Member and group management
│   ├── update_manager.rs  # Auto-update management
│   ├── main.rs           # Application entry point
│   └── lib.rs            # Library interface
├── icons/                 # Application icons
├── keystore/             # Secure key storage
├── tests/                # Integration tests
├── Cargo.toml            # Rust dependencies
├── tauri.conf.json       # Tauri configuration
└── Communitas.entitlements # macOS entitlements
```

### Development Commands

```bash
# Start development server (hot reload)
npm run tauri dev

# Run with logging
RUST_LOG=debug npm run tauri dev

# Build for production
npm run tauri build

# Build for specific platform
npm run tauri build -- --target x86_64-apple-darwin
npm run tauri build -- --target aarch64-apple-darwin
npm run tauri build -- --target x86_64-pc-windows-msvc
npm run tauri build -- --target x86_64-unknown-linux-gnu

# Run tests
cargo test -p communitas-desktop
npm test

# Format code
cargo fmt --all
npm run format

# Lint code
cargo clippy --all-features -- -D warnings
npm run lint
```

## Tauri Commands API

### Authentication

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Register new user
const result = await invoke('register_user', {
  fourWords: 'ocean-forest-moon-star',
  displayName: 'Alice',
  deviceName: 'MacBook Pro',
  password: 'secure-password'
});

// Login
const session = await invoke('login_user', {
  fourWords: 'ocean-forest-moon-star',
  password: 'secure-password'
});

// Logout
await invoke('logout_user');

// Touch ID authentication (macOS only)
const authResult = await invoke('authenticate_with_touchid');
```

### Four-Word Identity

```typescript
// Generate new identity
const fourWords = await invoke('generate_four_words');
// → "ocean-forest-moon-star"

// Validate identity
const isValid = await invoke('validate_four_words', {
  fourWords: 'ocean-forest-moon-star'
});

// Get identity info
const identity = await invoke('get_identity', {
  fourWords: 'ocean-forest-moon-star'
});
```

### Messaging

```typescript
// Send message
await invoke('send_message', {
  channelId: 'channel-123',
  content: 'Hello, World!',
  recipients: ['ocean-forest-moon-star']
});

// Get messages
const messages = await invoke('get_messages', {
  channelId: 'channel-123',
  limit: 50,
  offset: 0
});

// Subscribe to new messages
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('message:new', (event) => {
  console.log('New message:', event.payload);
});
```

### Channels

```typescript
// Create channel
const channel = await invoke('create_channel', {
  name: 'general',
  description: 'General discussion',
  visibility: 'public'
});

// List channels
const channels = await invoke('list_channels');

// Join channel
await invoke('join_channel', {
  channelId: 'channel-123'
});

// Leave channel
await invoke('leave_channel', {
  channelId: 'channel-123'
});
```

### Groups

```typescript
// Create group
const group = await invoke('create_group', {
  name: 'Project Team',
  members: [
    'ocean-forest-moon-star',
    'river-mountain-cloud-wind'
  ]
});

// Add member
await invoke('add_group_member', {
  groupId: 'group-123',
  memberFourWords: 'valley-lake-tree-bird'
});

// Remove member
await invoke('remove_group_member', {
  groupId: 'group-123',
  memberFourWords: 'valley-lake-tree-bird'
});
```

### Storage (Virtual Disks)

```typescript
// Write file to private disk
await invoke('disk_write', {
  entityId: 'entity-123',
  diskType: 'Private',
  path: '/documents/readme.md',
  contentBase64: btoa('# Documentation')
});

// Read file
const content = await invoke('disk_read', {
  entityId: 'entity-123',
  diskType: 'Private',
  path: '/documents/readme.md'
});

// List files
const files = await invoke('disk_list', {
  entityId: 'entity-123',
  diskType: 'Private',
  path: '/documents'
});

// Delete file
await invoke('disk_delete', {
  entityId: 'entity-123',
  diskType: 'Private',
  path: '/documents/readme.md'
});
```

### CRDT Documents

```typescript
// Create document
const doc = await invoke('create_document', {
  documentId: 'doc-123',
  title: 'Shared Document'
});

// Apply update
await invoke('apply_document_update', {
  documentId: 'doc-123',
  update: updateBytes
});

// Get document state
const state = await invoke('get_document_state', {
  documentId: 'doc-123'
});

// Subscribe to updates
const unlisten = await listen('document:update', (event) => {
  console.log('Document updated:', event.payload);
});
```

### Network

```typescript
// Connect to network
await invoke('connect_network');

// Disconnect from network
await invoke('disconnect_network');

// Get network status
const status = await invoke('get_network_status');

// Get peers
const peers = await invoke('get_peers');

// Check if peer is online
const isOnline = await invoke('is_peer_online', {
  fourWords: 'ocean-forest-moon-star'
});
```

### Auto-Updates

```typescript
// Check for updates
const update = await invoke('check_for_updates');

if (update.available) {
  console.log('Update available:', update.version);

  // Install update
  await invoke('install_update');
}

// Subscribe to update events
const unlisten = await listen('update:downloaded', (event) => {
  console.log('Update downloaded, ready to install');
});
```

## Configuration

### tauri.conf.json

```json
{
  "productName": "Communitas",
  "version": "0.1.1",
  "identifier": "com.saorsalabs.communitas",
  "build": {
    "beforeDevCommand": "npm run build && npm run dev:frontend",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/dirvine/communitas/releases/latest/download/latest.json"
      ],
      "dialog": true
    }
  }
}
```

### Cargo.toml Features

```toml
[features]
default = ["gossip_overlay"]
gossip_overlay = []  # Enable P2P gossip networking
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `COMMUNITAS_DATA_DIR` | Data storage directory | `~/.communitas` |
| `RUST_LOG` | Logging level | `info` |
| `TAURI_ENV` | Tauri environment | `production` |

## Platform-Specific Features

### macOS

#### Touch ID Integration

```rust
// Authenticate with Touch ID
use security_framework::item::*;

let auth_result = LocalAuthentication::authenticate(
    "Authenticate to access Communitas",
    LAPolicy::DeviceOwnerAuthenticationWithBiometrics
)?;
```

#### Keychain Integration

```rust
use keyring::Entry;

// Store credentials in Keychain
let entry = Entry::new("communitas", "user@example.com")?;
entry.set_password("secure-password")?;

// Retrieve credentials
let password = entry.get_password()?;
```

### Windows

#### Windows Hello

Integration with Windows Hello for biometric authentication (planned).

#### Credential Manager

```rust
use keyring::Entry;

// Store in Windows Credential Manager
let entry = Entry::new("communitas", "user@example.com")?;
entry.set_password("secure-password")?;
```

### Linux

#### Secret Service API

```rust
use keyring::Entry;

// Store in libsecret
let entry = Entry::new("communitas", "user@example.com")?;
entry.set_password("secure-password")?;
```

## Building for Distribution

### Code Signing

#### macOS

```bash
# Generate signing key
tauri signer generate

# Sign the app
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name" \
  target/release/bundle/macos/Communitas.app

# Create DMG
npm run tauri build -- --target universal-apple-darwin
```

#### Windows

```bash
# Sign with certificate
signtool sign /f certificate.pfx /p password \
  target/release/Communitas.exe
```

### Generating Update Artifacts

```bash
# Enable in tauri.conf.json
"createUpdaterArtifacts": true

# Build with updater artifacts
npm run tauri build

# Artifacts generated:
# - latest.json (update metadata)
# - [platform]-[arch].tar.gz (update bundle)
# - [platform]-[arch].tar.gz.sig (signature)
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test -p communitas-desktop

# Run specific test module
cargo test -p communitas-desktop core_commands::tests

# Run with logging
RUST_LOG=debug cargo test -p communitas-desktop -- --nocapture
```

### Integration Tests

```bash
# Run integration tests
cargo test -p communitas-desktop --test '*'

# Test Touch ID integration (macOS only)
cargo test -p communitas-desktop touch_id_tests

# Test auto-update flow
cargo test -p communitas-desktop update_tests
```

### End-to-End Tests

```bash
# Run Playwright tests
npm run test:e2e

# Run with UI
npm run test:e2e:ui
```

## Troubleshooting

### Application Won't Start

```bash
# Check logs
tail -f ~/.communitas/logs/communitas.log

# macOS: Check Console.app for crash reports
# Windows: Check Event Viewer
# Linux: Check journalctl
```

### Touch ID Not Working (macOS)

- Verify entitlements in `Communitas.entitlements`:
```xml
<key>com.apple.security.device.biometric</key>
<true/>
```

- Check System Preferences → Touch ID

### Network Connection Issues

```bash
# Check network status
curl http://localhost:8080/health

# Verify firewall settings
# macOS: System Preferences → Security → Firewall
# Windows: Windows Defender Firewall
# Linux: ufw status
```

### Auto-Update Failures

- Verify update server is accessible
- Check signature verification keys
- Review update logs: `~/.communitas/logs/updater.log`

### Build Failures

```bash
# Clean build cache
cargo clean
npm run clean

# Update dependencies
cargo update
npm update

# Reinstall node_modules
rm -rf node_modules package-lock.json
npm install
```

## Performance Optimization

### Bundle Size Optimization

```bash
# Enable production optimizations in Cargo.toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Remove debug symbols
```

### Memory Usage

- **Baseline**: ~80MB idle
- **Active Use**: ~150-300MB depending on documents open
- **Peak**: <500MB with multiple large documents

### Startup Time

- **Cold Start**: <2 seconds
- **Warm Start**: <1 second

## Security Considerations

1. **Secure by Default**: All communications encrypted with PQC
2. **Platform Integration**: Uses OS-level credential storage
3. **Auto-Updates**: Verified with ML-DSA signatures
4. **Sandboxing**: Runs with minimal permissions
5. **No Telemetry**: Zero tracking or analytics
6. **Local-First**: Data stays on your device

## Contributing

See [../docs/development/contributing.md](../docs/development/contributing.md)

## License

Dual-licensed under AGPL-3.0-or-later and commercial license.

## See Also

- [Communitas Core](../communitas-core/README.md) - Core library used by this app
- [API Documentation](../docs/AGENTS_API.md) - Complete Tauri commands API
- [Architecture Guide](../DESIGN.md) - System architecture
- [Development Guide](../CLAUDE.md) - Development workflow
