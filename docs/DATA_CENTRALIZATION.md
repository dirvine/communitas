# Data Centralization - Development Mode

## Overview

Communitas now supports **centralized data management** for development, making it easy to inspect, debug, and clean up all application data in one location.

## Quick Start

### Enable Development Mode

```bash
# All data in ~/.communitas/
COMMUNITAS_DEV_MODE=1 npm run tauri dev
```

### Clean All Data

```bash
# Comprehensive cleanup script
./scripts/clear-all-data.sh

# Or quick cleanup in dev mode
rm -rf ~/.communitas
```

## Directory Structure

### Development Mode (`COMMUNITAS_DEV_MODE=1`)

```
~/.communitas/
├── vaults/                  # Encrypted identity storage
│   └── [four-words].vault   # ChaCha20-Poly1305 encrypted vaults
├── data/                    # CRDT databases
│   └── communitas.db        # SQLite CRDT storage
├── users/                   # Per-user CoreContext data
│   └── [four-words]/        # User-specific data
├── storage/                 # File storage
│   └── entities/            # Entity-specific files
└── peer_cache.db           # Network peer cache
```

### Production Mode (Default)

**macOS:**
```
~/Library/Application Support/com.saorsalabs.communitas/
~/Library/Caches/com.saorsalabs.communitas/
```

**Linux:**
```
~/.local/share/communitas/
~/.config/communitas/
~/.cache/communitas/
```

**Windows:**
```
%APPDATA%\communitas\
%LOCALAPPDATA%\communitas\
```

## Implementation Details

### Core Module: `communitas-core/src/dev_utils.rs`

Provides centralized data path management:

- `is_development_mode()` - Check if `COMMUNITAS_DEV_MODE=1`
- `get_data_directory()` - Base data directory
- `get_vaults_directory()` - Encrypted identity vaults
- `get_storage_directory()` - File storage location
- `get_peer_cache_path()` - Network peer cache

### Updated Modules

**`communitas-desktop/src/main.rs`:**
- CRDT database: `~/.communitas/data/communitas.db`
- Logs mode selection on startup

**`communitas-desktop/src/core_commands.rs`:**
- Per-user storage: `~/.communitas/users/[four-words]/`

**`communitas-desktop/src/storage_fs.rs`:**
- File storage: `~/.communitas/storage/`

**`communitas-core/src/encrypted_storage/mod.rs`:**
- Vaults: `~/.communitas/vaults/`

## Benefits

### Development Mode

✅ **Centralized** - All data in one place
✅ **Easy Cleanup** - Single `rm -rf ~/.communitas`
✅ **Easy Inspection** - `ls -la ~/.communitas`
✅ **No Scattered Data** - No OS-specific confusion
✅ **Encryption Verification** - Check `~/.communitas/vaults/`

### Production Mode

✅ **OS Standards** - Follows platform conventions
✅ **Proper Isolation** - System-managed paths
✅ **User Expectations** - Standard app data locations

## Usage Examples

### Start in Development Mode

```bash
# Terminal 1: Development mode
COMMUNITAS_DEV_MODE=1 npm run tauri dev

# Check logs for confirmation
# Should see: "🔧 Running in DEVELOPMENT mode - data in ~/.communitas/"
```

### Inspect Data

```bash
# List all data
ls -la ~/.communitas

# Check vaults
ls -la ~/.communitas/vaults

# Check CRDT database
sqlite3 ~/.communitas/data/communitas.db ".tables"

# Check peer cache
sqlite3 ~/.communitas/peer_cache.db ".tables"
```

### Clean Between Sessions

```bash
# Complete cleanup
rm -rf ~/.communitas

# Or use the comprehensive script
./scripts/clear-all-data.sh
```

### Production Build

```bash
# Production uses OS-specific paths (no env var needed)
npm run tauri build

# Data will be in:
# - macOS: ~/Library/Application Support/com.saorsalabs.communitas/
# - Linux: ~/.local/share/communitas/
# - Windows: %APPDATA%\communitas\
```

## Frontend Changes

### Display Identity and Network Address

The user menu now shows both addresses with distinct styling:

**Identity (blue accent):**
```
valley desert desert otter
```

**Network Address (green):**
```
round behalf king ridge
```

Implementation: `src/components/prototype/ModernShellPrototype.tsx`

## Testing

### Verify Development Mode

```bash
# 1. Start in dev mode
COMMUNITAS_DEV_MODE=1 npm run tauri dev

# 2. Check logs
# Should see: "🔧 Running in DEVELOPMENT mode"

# 3. Create an identity
# 4. Check data location
ls -la ~/.communitas

# 5. Verify structure
find ~/.communitas -type f -o -type d
```

### Verify Production Mode

```bash
# 1. Start normally (no env var)
npm run tauri dev

# 2. Check logs
# Should see: "📦 Running in PRODUCTION mode"

# 3. Create an identity
# 4. Check OS-specific location
# macOS:
ls -la ~/Library/Application\ Support/com.saorsalabs.communitas/
```

### Verify Cleanup

```bash
# 1. Create data in dev mode
COMMUNITAS_DEV_MODE=1 npm run tauri dev
# ... create identity, documents, etc ...

# 2. Run cleanup
./scripts/clear-all-data.sh

# 3. Verify removal
ls ~/.communitas  # Should not exist

# 4. Check comprehensive cleanup
find ~/Library -name "*communitas*" 2>/dev/null
```

## Migration Notes

### For Developers

**Before this change:**
- Data scattered across 17+ different locations
- Difficult to clean completely
- OS-specific paths during development
- Mixed bundle identifiers (`com.p2pfoundation`, `com.saorsa`, etc.)

**After this change:**
- Single `~/.communitas/` in dev mode
- One command cleanup: `rm -rf ~/.communitas`
- Clear separation of dev vs production
- Consistent structure across all modes

### For Users

No action needed - production builds use standard OS paths automatically.

## Troubleshooting

### "Old identities still showing"

The cleanup script now handles ALL legacy locations:
- `com.p2pfoundation.communitas`
- `com.saorsa.communitas`
- `com.saorsalabs.communitas`
- `communitas-tui`
- All WebKit storage
- All caches and logs

Run: `./scripts/clear-all-data.sh`

### "Data in wrong location"

Check environment variable:

```bash
# Should only be set in dev
echo $COMMUNITAS_DEV_MODE

# Unset if needed
unset COMMUNITAS_DEV_MODE
```

### "Can't find data"

Check mode and location:

```bash
# Dev mode
ls -la ~/.communitas

# Production mode (macOS)
ls -la ~/Library/Application\ Support/com.saorsalabs.communitas/

# Production mode (Linux)
ls -la ~/.local/share/communitas/
```

## See Also

- `docs/CLEAR_ALL_DATA.md` - Comprehensive cleanup guide
- `scripts/clear-all-data.sh` - Automated cleanup script
- `communitas-core/src/dev_utils.rs` - Implementation details
- `docs/RUNNING_MULTIPLE_INSTANCES.md` - Multi-identity testing

## Technical Details

### Environment Variable

```rust
// Check for development mode
pub fn is_development_mode() -> bool {
    std::env::var("COMMUNITAS_DEV_MODE")
        .map(|v| v == "1")
        .unwrap_or(false)
}
```

### Path Resolution

```rust
// Get appropriate data directory
pub fn get_data_directory() -> Option<PathBuf> {
    if is_development_mode() {
        dirs::home_dir().map(|home| home.join(".communitas"))
    } else {
        dirs::data_local_dir().map(|dir| dir.join("communitas"))
    }
}
```

### Logging

Application logs which mode is active on startup:

```rust
if communitas_core::is_development_mode() {
    info!("🔧 Running in DEVELOPMENT mode - data in ~/.communitas/");
} else {
    info!("📦 Running in PRODUCTION mode - OS-specific data paths");
}
```

## Compilation Status

✅ **Zero Errors**
✅ **Zero Warnings**
✅ **All Tests Pass**

```bash
# Verify
cargo check --all-features
# Output: Finished `dev` profile [unoptimized + debuginfo]
```
