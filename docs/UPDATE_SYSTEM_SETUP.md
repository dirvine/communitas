# Communitas Update System Setup Guide

This guide explains how to set up and use the automatic update system for Communitas.

## Overview

Communitas uses Tauri's built-in updater with Ed25519 signature verification to provide secure, automatic updates from GitHub releases.

**Architecture:**
```
GitHub Release (Signed Binaries)
    ↓
Tauri Updater (Signature Verification)
    ↓
Update Manager Service (Rust)
    ↓
Update Notification UI (React)
    ↓
User Approval & Installation
```

## Initial Setup

### 1. Generate Signing Keys

Generate the Ed25519 keypair for signing releases:

```bash
chmod +x scripts/generate-update-keys.sh
./scripts/generate-update-keys.sh
```

This creates `.keys/updater-keys.json` with your public and private keys.

**⚠️ CRITICAL SECURITY:**
- **NEVER commit the private key to git**
- The `.keys/` directory is already in `.gitignore`
- Keep the private key secure and backed up

### 2. Configure GitHub Secrets

Add the private key to GitHub repository secrets:

1. Go to your repository on GitHub
2. Navigate to Settings → Secrets and variables → Actions
3. Click "New repository secret"
4. Add two secrets:

   **Secret 1:**
   - Name: `TAURI_SIGNING_PRIVATE_KEY`
   - Value: Copy the entire `privateKey` value from `.keys/updater-keys.json`

   **Secret 2 (optional):**
   - Name: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
   - Value: Leave empty unless you encrypted your key

### 3. Add Public Key to Configuration

Copy the **public key** from `.keys/updater-keys.json` and add it to `communitas-desktop/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/dirvine/communitas/releases/latest/download/latest.json"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

Replace `YOUR_PUBLIC_KEY_HERE` with your actual public key.

## Creating a Release

### Automated Release (Recommended)

1. **Tag a new version:**
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

2. **GitHub Actions will automatically:**
   - Build for all platforms (macOS x64/ARM, Linux, Windows)
   - Sign the binaries with your private key
   - Create updater artifacts (.tar.gz.sig, .app.tar.gz.sig, etc.)
   - Generate `latest.json` manifest
   - Create a GitHub release with all artifacts

3. **Publish the release:**
   - The release is created as a draft
   - Review the build artifacts
   - Add release notes describing changes
   - Click "Publish release"

### Manual Release (Advanced)

If you need to build manually:

```bash
# Build the app with updater artifacts
cd communitas-desktop
cargo tauri build

# Sign the artifacts
cargo tauri signer sign \
  --private-key ~/.keys/updater-keys.json \
  --file target/release/bundle/macos/Communitas.app.tar.gz

# Upload to GitHub release manually
```

## Version Numbering

Follow Semantic Versioning (SemVer):

- **MAJOR.MINOR.PATCH** (e.g., 1.2.3)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

Update version in:
1. `communitas-desktop/Cargo.toml`
2. `communitas-desktop/tauri.conf.json`

## Update Checking

The app automatically checks for updates:

- **On startup**: Checks once when app starts
- **Every 6 hours**: Background checks while running
- **Manual check**: User can trigger from settings

### Frontend Integration

Add the UpdateNotification component to your app:

```tsx
import UpdateNotification from './components/UpdateNotification';

function App() {
  return (
    <>
      <UpdateNotification />
      {/* Rest of your app */}
    </>
  );
}
```

## Update Flow

1. **Check**: App calls `check_for_updates()` command
2. **Notify**: If update available, dialog appears
3. **Download**: User clicks "Install Update"
4. **Verify**: Tauri verifies signature automatically
5. **Install**: Update is installed
6. **Restart**: App restarts to apply update

## Security Features

- **Signature Verification**: Every update is verified using Ed25519
- **HTTPS Only**: Updates only downloaded over HTTPS
- **Rollback**: Failed updates automatically rollback
- **User Approval**: Updates require explicit user consent

## Testing Updates

### Local Testing

1. Build current version:
   ```bash
   npm run tauri build
   ```

2. Bump version and rebuild:
   ```bash
   # Edit version in Cargo.toml and tauri.conf.json
   npm run tauri build
   ```

3. Create a mock GitHub release structure:
   ```bash
   mkdir -p mock-release
   cp target/release/bundle/... mock-release/
   ```

4. Update `endpoints` in tauri.conf.json to point to local server:
   ```json
   "endpoints": ["http://localhost:8080/latest.json"]
   ```

### Production Testing

1. Create a pre-release on GitHub
2. Tag as `v0.2.0-beta.1`
3. Install the pre-release version
4. Tag final release as `v0.2.0`
5. Verify update from beta to final works

## Troubleshooting

### "Invalid signature" error

- Ensure public key in tauri.conf.json matches private key in GitHub Secrets
- Verify the release was built with GitHub Actions (not local build)

### Update not detected

- Check `endpoints` URL in tauri.conf.json
- Verify `latest.json` exists at the endpoint
- Check network connectivity
- View logs: `RUST_LOG=debug cargo tauri dev`

### Failed to install

- Check disk space
- Verify write permissions to app directory
- Check logs for specific error

## API Reference

### Rust Commands

```rust
// Check for available updates
check_for_updates(app: AppHandle, state: State<UpdateState>)
  -> Result<UpdateStatus, String>

// Install available update
install_update(app: AppHandle, state: State<UpdateState>)
  -> Result<(), String>

// Get current update status
get_update_status(state: State<UpdateState>)
  -> Result<UpdateStatus, String>
```

### Frontend API

```typescript
import { invoke } from '@tauri-apps/api/core';

// Check for updates
const status = await invoke('check_for_updates');

// Install update
await invoke('install_update');

// Get status
const status = await invoke('get_update_status');
```

## Release Checklist

Before creating a release:

- [ ] Version bumped in Cargo.toml
- [ ] Version bumped in tauri.conf.json
- [ ] CHANGELOG.md updated
- [ ] All tests passing
- [ ] Frontend built (`npm run build`)
- [ ] Public key configured in tauri.conf.json
- [ ] Private key added to GitHub Secrets
- [ ] Release workflow file exists (.github/workflows/release.yml)

## Resources

- [Tauri Updater Documentation](https://v2.tauri.app/plugin/updater/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Semantic Versioning](https://semver.org/)
- [Ed25519 Signature Scheme](https://ed25519.cr.yp.to/)
