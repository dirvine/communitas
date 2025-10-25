# Getting Started with Communitas

Welcome to Communitas - the local-first, post-quantum collaboration platform! This guide will help you get up and running quickly.

## What is Communitas?

Communitas is a decentralized collaboration platform that combines messaging, file sharing, voice/video calling, and web publishing into a single application. Built with post-quantum cryptography and human-verifiable Four-Word addressing, Communitas provides secure, private communication without relying on centralized servers.

### Key Features

- **Four-Word Identities**: Human-readable addresses like "ocean-forest-moon-star"
- **Post-Quantum Security**: ML-DSA signatures and ML-KEM key exchange
- **Local-First**: All data stored locally, syncs when online
- **Offline Capable**: Core functionality works without internet
- **Decentralized**: No central servers or single points of failure
- **End-to-End Encrypted**: All communications encrypted by default

## Choose Your Installation

Communitas offers multiple deployment options depending on your use case:

### Desktop Application (Recommended for Most Users)

Native application for Windows, macOS, and Linux with full UI.

#### Download Binary

Visit [GitHub Releases](https://github.com/dirvine/communitas/releases) and download:
- **macOS**: `Communitas_0.1.1_aarch64.dmg` (Apple Silicon) or `Communitas_0.1.1_x64.dmg` (Intel)
- **Windows**: `Communitas_0.1.1_x64.msi`
- **Linux**: `communitas_0.1.1_amd64.AppImage` or `.deb`

#### Build from Source

```bash
# Prerequisites: Rust 1.85+, Node.js 20+
git clone https://github.com/dirvine/communitas.git
cd communitas

# Install dependencies
npm install

# Build frontend
npm run build

# Run development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Headless Daemon (Servers & Bots)

For running as a system service without UI.

```bash
# Install from binary
wget https://github.com/dirvine/communitas/releases/latest/communitas-headless
chmod +x communitas-headless
sudo mv communitas-headless /usr/local/bin/

# Or build from source
cargo build --release -p communitas-headless
```

See [communitas-headless/README.md](../../communitas-headless/README.md) for detailed setup.

### Docker Container (Cloud Deployment)

For containerized deployments.

```bash
# Docker Compose
docker-compose up -d

# Or standalone
docker run -p 8080:8080 -p 9090:9090 communitas/node:latest
```

See [Operations Guide](../operations/README.md) for Kubernetes and advanced deployment.

## First Launch: Desktop Application

### 1. Create Your Identity

On first launch, you'll be prompted to create your identity:

```
┌────────────────────────────────────┐
│  Welcome to Communitas!            │
│                                    │
│  Create Your Identity:             │
│                                    │
│  Display Name: [Alice____________] │
│  Device Name:  [MacBook Pro______] │
│                                    │
│  Your Four-Word Address:           │
│  ocean-forest-moon-star            │
│                                    │
│  [Create Identity]                 │
└────────────────────────────────────┘
```

**Important**: Your four-word address is generated automatically and uniquely identifies you on the network. Write it down - you'll need it to log in on other devices!

### 2. Set a Password

Choose a strong password to encrypt your local data:

```
┌────────────────────────────────────┐
│  Secure Your Identity              │
│                                    │
│  Password: [******************]    │
│  Confirm:  [******************]    │
│                                    │
│  Password Strength: ██████████ Strong │
│                                    │
│  [Secure Identity]                 │
└────────────────────────────────────┘
```

### 3. Optional: Enable Passkey (Recommended)

For faster, more secure login using biometrics:

```
┌────────────────────────────────────┐
│  Enable Passkey Authentication?    │
│                                    │
│  ✓ Login with Touch ID / Face ID  │
│  ✓ More secure than passwords      │
│  ✓ Faster authentication           │
│                                    │
│  [Enable Passkey]  [Skip]          │
└────────────────────────────────────┘
```

**macOS**: Uses Touch ID or Apple Watch
**Windows**: Uses Windows Hello
**Linux**: Uses fingerprint reader if available

### 4. Network Connection

Communitas will automatically connect to the P2P network:

```
┌────────────────────────────────────┐
│  Connecting to Network...          │
│                                    │
│  ● Finding peers...                │
│  ● Establishing connections...     │
│  ● Syncing data...                 │
│                                    │
│  Status: Connected to 42 peers     │
└────────────────────────────────────┘
```

If network connection fails, Communitas operates in **local mode** - all functionality works offline and will sync when network becomes available.

## Your First Actions

### Create a Channel

Channels are topic-focused discussion spaces:

1. Click **"+ New Channel"** in the sidebar
2. Enter channel details:
   - **Name**: "general"
   - **Description**: "General discussion"
   - **Visibility**: Public or Private
3. Click **"Create"**

### Send a Message

1. Select a channel from the sidebar
2. Type your message in the composer at the bottom
3. Press **Enter** to send

Your message is:
- ✓ End-to-end encrypted
- ✓ Stored locally first (instant send)
- ✓ Synced to peers when online
- ✓ Delivered even if recipient is offline

### Add a Contact

To connect with someone:

1. Ask for their **four-word address** (e.g., "ocean-forest-moon-star")
2. Click **"+ Add Contact"**
3. Enter their four-word address
4. Add an optional note
5. Click **"Add"**

Once added, you can:
- Send direct messages
- See their online status
- Add them to groups
- Share channels with them

### Create a Group

Groups are persistent collections of people:

1. Click **"+ New Group"**
2. Enter group name
3. Add members by four-word address
4. Click **"Create"**

Groups can have:
- Shared channels
- Shared virtual disks (file storage)
- Voice/video calls
- Group administration

## Understanding Four-Word Addresses

Every entity in Communitas has a four-word address:

- **Users**: "ocean-forest-moon-star"
- **Organizations**: "valley-river-cloud-wind"
- **Groups**: "mountain-lake-tree-bird"
- **Channels**: Derived from creator's address

### Why Four Words?

1. **Human-Readable**: Easier to remember than cryptographic hashes
2. **Verifiable**: Words from a known dictionary prevent typos
3. **Anti-Phishing**: Dictionary validation stops spoofing attacks
4. **Universal**: Works without DNS or central registries
5. **Private**: No personal information revealed

### Sharing Your Address

Your four-word address is **public information** - it's safe to share widely:

```
Hi! Add me on Communitas:
ocean-forest-moon-star

Or scan this QR code:
[QR CODE]
```

See [four-word-addresses.md](four-word-addresses.md) for deep dive.

## Using Virtual Disks

Every entity has three virtual disks for file storage:

### Private Disk

Encrypted, local-only storage:

```typescript
// Access via UI: Entity → Files → Private
// Or via API:
await invoke('disk_write', {
  entityId: 'entity-123',
  diskType: 'Private',
  path: '/personal/notes.txt',
  content: 'My private notes'
});
```

Use for:
- Personal files
- Private notes
- Credentials (auto-encrypted)
- Drafts

### Public Disk

Content-addressed, distributed storage:

```typescript
// Access via UI: Entity → Files → Public
// Or via API:
await invoke('disk_write', {
  entityId: 'entity-123',
  diskType: 'Public',
  path: '/blog/index.html',
  content: '<html>...'
});
```

Use for:
- Public documents
- Shared files
- Website content
- Open data

### Shared Disk

Group-accessible with shared encryption:

```typescript
// Access via UI: Group → Files → Shared
// Or via API:
await invoke('disk_write', {
  entityId: 'group-123',
  diskType: 'Shared',
  path: '/projects/design.fig',
  content: designData
});
```

Use for:
- Team documents
- Collaborative files
- Project resources
- Shared media

## Collaborative Documents (CRDT)

Communitas supports real-time collaborative editing:

### Create a Document

1. Navigate to channel or group
2. Click **"+ New Document"**
3. Enter document name
4. Start typing!

### Features

- **Real-Time Sync**: See others' changes instantly
- **Conflict-Free**: Automatic merge of concurrent edits
- **Offline Editing**: Changes sync when back online
- **Version History**: Track all changes over time
- **Cursor Presence**: See where others are typing

### Supported Formats

- Plain text (`.txt`)
- Markdown (`.md`)
- Rich text (`.rtf`)
- Code (`.js`, `.rs`, `.py`, etc.)

## Network Status & Connectivity

### Understanding Connection States

**🟢 Connected** - Fully connected to P2P network
- All features available
- Syncing with peers in real-time
- Message delivery immediate

**🟡 Local Mode** - Operating offline
- All features still work
- Data stored locally
- Will sync when network returns

**🔴 Connecting** - Establishing connections
- Searching for peers
- May take 10-30 seconds
- Patience recommended

**⚪ Offline** - Intentionally offline
- Network disabled by user
- Local-only operations
- No sync happening

### Checking Network Status

```
┌─────────────────────────────────┐
│ Network Status                  │
├─────────────────────────────────┤
│ Status: Connected               │
│ Peers: 42 connected             │
│ Latency: 45ms average           │
│ Bandwidth: 156 KB/s             │
│ Last Sync: 2 seconds ago        │
└─────────────────────────────────┘
```

### Troubleshooting Connection Issues

If you can't connect:

1. **Check firewall** - Allow UDP port 8080
2. **Check router** - UPnP enabled or port forwarded
3. **Check bootstrap nodes** - Verify they're reachable
4. **Try offline mode** - Still fully functional!

```bash
# Test connectivity
curl -v http://bootstrap.communitas.network:8080/health
```

## Keyboard Shortcuts

### Global
- `Ctrl/Cmd + N` - New channel
- `Ctrl/Cmd + K` - Quick command palette
- `Ctrl/Cmd + F` - Search
- `Ctrl/Cmd + ,` - Settings
- `Ctrl/Cmd + Q` - Quit

### Navigation
- `Ctrl/Cmd + 1-9` - Switch between channels
- `Ctrl/Cmd + ↑/↓` - Previous/next channel
- `Alt + ←/→` - Navigate history
- `Esc` - Close modal/dialog

### Messaging
- `Enter` - Send message
- `Shift + Enter` - New line
- `Ctrl/Cmd + E` - Edit last message
- `@` - Mention user
- `:` - Emoji picker

### Documents
- `Ctrl/Cmd + S` - Save document
- `Ctrl/Cmd + B` - Bold text
- `Ctrl/Cmd + I` - Italic text
- `Ctrl/Cmd + L` - Insert link

## Settings & Configuration

### Accessing Settings

Click your avatar → **Settings** or press `Ctrl/Cmd + ,`

### Key Settings

**General**
- Language
- Theme (Light/Dark/Auto)
- Startup behavior
- Notifications

**Identity**
- View your four-word address
- Change display name
- Change device name
- Backup identity

**Network**
- Connection mode (Auto/Manual/Offline)
- Bootstrap nodes
- Port configuration
- Bandwidth limits

**Privacy**
- Message retention
- Read receipts
- Typing indicators
- Online status visibility

**Security**
- Change password
- Enable/disable passkey
- Two-factor authentication
- Session management

**Storage**
- Data directory location
- Cache size limits
- Auto-cleanup settings
- Export data

**Updates**
- Auto-update (recommended)
- Update channel (Stable/Beta)
- Check for updates manually

## Data Storage

### Where is Data Stored?

Communitas stores all data locally:

**macOS**: `~/Library/Application Support/communitas/`
**Windows**: `%APPDATA%\communitas\`
**Linux**: `~/.config/communitas/`

### What's Stored?

```
communitas/
├── identity/          # Your identity and keys
├── storage/          # Message and channel data
├── cache/            # Temporary cached data
├── logs/             # Application logs
└── config.toml       # User configuration
```

### Backup Your Data

**Automatic Backup** (recommended):
- Settings → Backup → Enable auto-backup
- Backups stored to external drive or cloud

**Manual Backup**:
```bash
# Copy entire data directory
cp -r ~/Library/Application\ Support/communitas/ ~/communitas-backup/
```

### Restore from Backup

1. Close Communitas
2. Replace data directory with backup
3. Restart Communitas
4. Verify identity and data

## Security Best Practices

### Identity Security

✅ **DO**:
- Use a strong, unique password
- Enable passkey authentication
- Back up your identity regularly
- Store your four-word address securely

❌ **DON'T**:
- Share your password with anyone
- Use the same password elsewhere
- Write your password in plaintext
- Store identity on shared computers

### Communication Security

✅ **DO**:
- Verify four-word addresses before trusting
- Use end-to-end encrypted channels
- Enable disappearing messages for sensitive info
- Review group membership regularly

❌ **DON'T**:
- Accept contacts from unknown addresses
- Share sensitive info in public channels
- Disable encryption
- Ignore security warnings

### Network Security

✅ **DO**:
- Keep Communitas updated
- Use trusted bootstrap nodes
- Enable firewall on your device
- Monitor network activity

❌ **DON'T**:
- Disable security features
- Use untrusted network nodes
- Ignore security updates
- Connect to suspicious peers

## Getting Help

### In-App Help

- Press `F1` or `?` for keyboard shortcuts
- Click "Help" in menu for documentation
- Hover over UI elements for tooltips

### Documentation

- [Authentication Guide](authentication.md)
- [Four-Word Addresses](four-word-addresses.md)
- [Testing Guide](testing.md)
- [API Documentation](../api/)

### Community Support

- GitHub Issues: https://github.com/dirvine/communitas/issues
- Discussions: https://github.com/dirvine/communitas/discussions
- Official Website: https://communitas.life

### Reporting Bugs

Found a bug? Please report it:

1. Check if it's already reported
2. Gather information:
   - Communitas version
   - Operating system
   - Steps to reproduce
   - Error messages/logs
3. Create issue on GitHub
4. Include logs from `~/Library/Application Support/communitas/logs/`

## Next Steps

Now that you're set up, explore:

- **[Authentication Guide](authentication.md)** - Advanced authentication features
- **[Four-Word Addresses](four-word-addresses.md)** - Deep dive into identity system
- **[Testing Guide](testing.md)** - Test your setup
- **[Architecture](../architecture/)** - How Communitas works
- **[API Documentation](../api/)** - Build integrations

## Quick Reference Card

```
┌─────────────────────────────────────────────┐
│ Communitas Quick Reference                  │
├─────────────────────────────────────────────┤
│ Your Address: ocean-forest-moon-star        │
│ Network: 🟢 Connected (42 peers)            │
│                                             │
│ ACTIONS                                     │
│ • Ctrl+N - New channel                      │
│ • Ctrl+K - Command palette                  │
│ • Ctrl+F - Search                           │
│ • @ - Mention user                          │
│ • : - Emoji picker                          │
│                                             │
│ VIEWS                                       │
│ • Ctrl+1-9 - Switch channels                │
│ • Ctrl+↑/↓ - Previous/next                  │
│ • Alt+←/→ - Navigate history                │
│                                             │
│ HELP                                        │
│ • F1 or ? - Show shortcuts                  │
│ • Ctrl+, - Settings                         │
│ • Ctrl+Q - Quit                             │
└─────────────────────────────────────────────┘
```

---

**Welcome to Communitas! Enjoy secure, private collaboration! 🚀**
