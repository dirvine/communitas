# Clear All Data - Fresh Start Guide

This guide shows you how to completely clear all Communitas data and start fresh with a new identity.

## Why Clear Data?

You might want to clear all data to:
- Start fresh with a new identity after testing
- Remove old test identities and messages
- Clean up the login screen showing many old identities
- Reset to a clean state for development

## What Gets Cleared

When you clear all data, you'll remove:
- All user vaults and encrypted identities
- All documents and CRDT data
- All network peer caches
- All application preferences
- All authentication sessions

## Method 1: Automatic Script (Recommended)

We provide a convenient script that clears everything:

```bash
# From the project root
./scripts/clear-all-data.sh
```

## Method 2: Manual Cleanup

If you prefer manual cleanup or the script doesn't work, follow these steps:

### Step 1: Close the App

Make sure Communitas is completely closed (no running processes).

### Step 2: Clear Tauri/Desktop Data

```bash
# macOS
rm -rf ~/Library/Application\ Support/com.saorsalabs.communitas
rm -rf ~/.communitas
rm -rf ~/.local/share/communitas
```

```bash
# Linux
rm -rf ~/.local/share/communitas
rm -rf ~/.config/communitas
rm -rf ~/.cache/communitas
```

```bash
# Windows (PowerShell)
Remove-Item -Recurse -Force "$env:APPDATA\communitas"
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\communitas"
```

### Step 3: Clear Vault Storage

```bash
# macOS/Linux - Clear encrypted vaults
rm -rf ~/.communitas-vaults
rm -rf ~/Library/Application\ Support/.communitas-vaults  # macOS fallback

# Also clear data directory if using custom location
rm -rf .communitas-data
```

### Step 4: Clear Browser Storage (if using web mode)

If you've been testing in the browser:

1. Open Browser DevTools (F12)
2. Go to Application tab
3. Clear:
   - Local Storage
   - Session Storage
   - IndexedDB
   - Cookies

Or use this in browser console:
```javascript
localStorage.clear();
sessionStorage.clear();
indexedDB.deleteDatabase('communitas');
```

### Step 5: Clear Peer Cache

```bash
# Remove gossip network peer cache
rm -f ./peer_cache.db
```

### Step 6: Verify Clean State

Start the app - you should see the "Welcome to Communitas" first-launch screen asking you to create a new identity.

## After Clearing

When you restart the app:

1. ✅ You'll see the first-launch welcome screen
2. ✅ You'll be prompted to generate a new four-word identity
3. ✅ No old identities will appear in the login screen
4. ✅ Network address will be freshly generated on first connection

## Preserving Specific Data

If you want to keep some data while clearing others:

### Keep Vaults, Clear Network Cache

```bash
# Just clear network/peer cache
rm -f ./peer_cache.db
rm -rf ~/.communitas/network
```

### Keep Network, Clear Vaults

```bash
# Just clear vaults
rm -rf ~/.communitas-vaults
```

## Troubleshooting

### "Old identities still showing"

This means vault data wasn't fully cleared. Try:

```bash
# Find all vault locations
find ~ -name "*communitas*" -type d 2>/dev/null

# Remove any that appear in the results
rm -rf <found-directory>
```

### "Network address not changing"

Clear the peer cache specifically:

```bash
rm -f ./peer_cache.db
rm -rf ~/.communitas/gossip
```

### "App crashes after clearing"

This is normal if you cleared data while the app was running. Just restart the app.

## Development Mode

**NEW**: Communitas now supports a centralized development mode for easier data management!

### Using Development Mode

Set the `COMMUNITAS_DEV_MODE=1` environment variable to store ALL data in `~/.communitas/`:

```bash
# Start in development mode
COMMUNITAS_DEV_MODE=1 npm run tauri dev
```

**Development Mode Benefits:**
- All data in one place: `~/.communitas/`
- Easy cleanup: `rm -rf ~/.communitas`
- Easy inspection: `ls -la ~/.communitas`
- No scattered data across OS-specific paths

**Directory Structure (Dev Mode):**
```
~/.communitas/
├── vaults/           # Encrypted identities
├── data/            # CRDT databases
│   └── communitas.db
├── users/           # Per-user CoreContext data
│   └── [four-words]/
├── storage/         # File storage
│   └── entities/
└── peer_cache.db   # Network peer cache
```

### Production Mode

Without `COMMUNITAS_DEV_MODE=1`, data is stored in OS-specific locations:

**macOS:**
- `~/Library/Application Support/com.saorsalabs.communitas/`
- `~/Library/Caches/com.saorsalabs.communitas/`

**Linux:**
- `~/.local/share/communitas/`
- `~/.config/communitas/`
- `~/.cache/communitas/`

**Windows:**
- `%APPDATA%\communitas\`
- `%LOCALAPPDATA%\communitas\`

### Quick Cleanup (Dev Mode)

```bash
# Single command cleanup
rm -rf ~/.communitas

# Or use the script
./scripts/clear-all-data.sh
```

## Development Notes

During development, consider:

1. **Use Development Mode** - Set `COMMUNITAS_DEV_MODE=1` for centralized data
2. **Regular Cleanup** - Clear `~/.communitas` between test sessions
3. **Identity Switcher** - Test multiple identities without clearing data
4. **Check Encryption** - Verify vault encryption in `~/.communitas/vaults/`

## See Also

- `docs/RUNNING_MULTIPLE_INSTANCES.md` - Test with multiple identities without clearing data
- `docs/CRDT_INTEGRATION_STATUS.md` - CRDT document storage locations
