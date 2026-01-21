# macOS Desktop Bundle & Release Guide

This document covers building, signing, notarizing, and releasing the Communitas desktop application for macOS.

## Overview

The Communitas desktop app is built using Dioxus with Tauri, producing a native macOS application that:
- Runs on both Intel (x86_64) and Apple Silicon (arm64) Macs
- Is code-signed with a Developer ID certificate
- Is notarized by Apple for Gatekeeper approval
- Distributed as a signed DMG disk image

## Quick Start

### Local Development Build

```bash
cd communitas-dioxus

# Install dx CLI (if not already installed)
../scripts/install_dx.sh

# Run with hot reload
dx serve --platform desktop --hotpatch

# Build release (unsigned)
dx bundle --platform desktop --release
```

The bundle output is in `communitas-dioxus/dist/bundle/`.

### Creating a Release

Push a tag to trigger the automated release:

```bash
# Tag a release
git tag desktop-v1.0.0
git push origin desktop-v1.0.0
```

Or use manual workflow dispatch from the GitHub Actions UI.

## Architecture

### Universal Binary

The release workflow creates a universal binary supporting both architectures:

```
┌─────────────────────────────────────────────────────────────┐
│                    Universal Binary                          │
├─────────────────────────────────────────────────────────────┤
│  x86_64-apple-darwin  │  aarch64-apple-darwin               │
│  (Intel Macs)         │  (Apple Silicon M1/M2/M3)           │
└─────────────────────────────────────────────────────────────┘
```

This is created using `lipo`:

```bash
lipo \
  target/x86_64-apple-darwin/release/communitas-dioxus \
  target/aarch64-apple-darwin/release/communitas-dioxus \
  -create -output target/universal-apple-darwin/release/communitas-dioxus
```

### Code Signing Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                     macOS Release Pipeline                        │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  1. Build                                                         │
│     ├── cargo build --target x86_64-apple-darwin                 │
│     ├── cargo build --target aarch64-apple-darwin                │
│     └── lipo → universal binary                                   │
│                                                                   │
│  2. Bundle                                                        │
│     ├── dx bundle --platform desktop --release                   │
│     └── Replace binary with universal version                     │
│                                                                   │
│  3. Sign App Bundle                                               │
│     ├── Import certificate to temp keychain                       │
│     └── codesign --deep --force --options runtime                │
│                                                                   │
│  4. Create DMG                                                    │
│     ├── hdiutil create (UDZO compressed)                          │
│     └── Add Applications symlink                                  │
│                                                                   │
│  5. Sign DMG                                                      │
│     └── codesign --force --verify                                 │
│                                                                   │
│  6. Notarize                                                      │
│     ├── xcrun notarytool submit --wait                           │
│     └── xcrun stapler staple (attach ticket)                     │
│                                                                   │
│  7. Release                                                       │
│     └── Upload to GitHub Release (draft)                          │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

## Configuration Files

### Dioxus.toml

Bundle configuration in `communitas-dioxus/Dioxus.toml`:

```toml
[bundle]
identifier = "com.maidsafe.communitas"
publisher = "MaidSafe"
icon = ["assets/icon.png", "assets/icon.icns"]
category = "public.app-category.social-networking"

[bundle.macos]
minimum_system_version = "11.0"
hardened_runtime = true
entitlements = "entitlements.plist"
```

### Entitlements

The `entitlements.plist` file grants necessary permissions:

| Entitlement | Purpose |
|-------------|---------|
| `network.client` | Make outbound network connections |
| `network.server` | Accept incoming P2P connections |
| `files.user-selected.read-write` | Access user-selected files |
| `files.downloads.read-write` | Save files to Downloads |
| `device.camera` | Video calls |
| `device.audio-input` | Voice/video calls |
| `cs.allow-jit` | WebView JavaScript |
| `cs.allow-unsigned-executable-memory` | WebView requirements |
| `cs.disable-library-validation` | WebView plugins |

## GitHub Secrets

The following secrets must be configured in GitHub for the release workflow:

| Secret | Description |
|--------|-------------|
| `MACOS_CERTIFICATE` | Base64-encoded .p12 certificate file |
| `MACOS_CERTIFICATE_PASSWORD` | Password for the .p12 file |
| `MACOS_SIGNING_IDENTITY` | Certificate common name (e.g., "Developer ID Application: Company (TEAMID)") |
| `KEYCHAIN_PASSWORD` | Password for temporary keychain |
| `MACOS_NOTARIZATION_APPLE_ID` | Apple ID email for notarization |
| `MACOS_NOTARIZATION_PASSWORD` | App-specific password for notarization |
| `MACOS_NOTARIZATION_TEAM_ID` | Apple Developer Team ID |

### Setting Up Certificates

1. **Export Certificate from Keychain Access:**
   ```bash
   # In Keychain Access:
   # 1. Find "Developer ID Application" certificate
   # 2. Right-click → Export
   # 3. Save as .p12 with password
   ```

2. **Base64 Encode for GitHub:**
   ```bash
   base64 -i certificate.p12 | pbcopy
   # Paste as MACOS_CERTIFICATE secret
   ```

3. **Create App-Specific Password:**
   - Go to https://appleid.apple.com/
   - Sign in → Security → App-Specific Passwords
   - Generate password for "GitHub Actions"

## Local Signing (Optional)

For local testing with signing:

```bash
# Set up environment
export MACOS_SIGNING_IDENTITY="Developer ID Application: Your Company (TEAMID)"

# Build and bundle
cd communitas-dioxus
dx bundle --platform desktop --release

# Sign the app
APP_PATH="dist/bundle/macos/Communitas.app"
codesign --deep --force --verify --verbose \
  --options runtime \
  --entitlements entitlements.plist \
  --sign "$MACOS_SIGNING_IDENTITY" \
  "$APP_PATH"

# Verify
codesign --verify --deep --strict --verbose=2 "$APP_PATH"
```

## Testing

### Smoke Test Script

Use the provided smoke test to verify a DMG or app bundle:

```bash
# Test a DMG
./scripts/tests/smoke-test-dmg.sh path/to/Communitas-v1.0.0-universal.dmg

# Test an app bundle directly
./scripts/tests/smoke-test-dmg.sh dist/bundle/macos/Communitas.app

# Include launch test
./scripts/tests/smoke-test-dmg.sh --launch path/to/Communitas.app
```

The smoke test verifies:
- Code signature validity
- Hardened runtime enabled
- Entitlements present
- Notarization ticket (if applicable)
- Universal binary architecture
- App bundle structure

### Manual Verification

```bash
# Verify code signature
codesign --verify --deep --strict --verbose=2 /path/to/Communitas.app

# Check Gatekeeper assessment
spctl --assess --verbose=2 /path/to/Communitas.app

# Verify notarization ticket
xcrun stapler validate /path/to/Communitas.dmg

# Check architectures
lipo -info /path/to/Communitas.app/Contents/MacOS/Communitas
```

## Troubleshooting

### "App is damaged and can't be opened"

This usually means the app isn't properly signed or notarized:

```bash
# Check signature
codesign --verify --deep --strict --verbose=2 /path/to/App.app

# Check notarization
spctl --assess --verbose=2 /path/to/App.app
```

### Notarization Failures

Common issues:
- **Missing entitlements**: Ensure `entitlements.plist` covers all app capabilities
- **Unsigned nested code**: Use `--deep` flag when signing
- **Hardened runtime not enabled**: Check `--options runtime` flag

View notarization log:
```bash
xcrun notarytool log <submission-id> \
  --apple-id "$APPLE_ID" \
  --password "$APP_SPECIFIC_PASSWORD" \
  --team-id "$TEAM_ID"
```

### Signature Issues

```bash
# Check what's wrong with signature
codesign -vvv --deep --strict /path/to/App.app

# Re-sign with verbose output
codesign --force --deep --verbose=4 \
  --options runtime \
  --sign "$IDENTITY" \
  /path/to/App.app
```

## CI/CD Workflow

The release workflow (`.github/workflows/release-desktop.yml`) is triggered by:

1. **Tag Push**: `desktop-v*` tags (e.g., `desktop-v1.0.0`)
2. **Manual Dispatch**: From GitHub Actions UI

### Workflow Jobs

1. **build-macos**: Builds, signs, notarizes, uploads artifact
2. **release**: Creates GitHub Release with DMG (only on tag push)

### Release Output

- DMG filename: `Communitas-{version}-universal.dmg`
- Supports macOS 11.0 (Big Sur) and later
- Universal binary (Intel + Apple Silicon)

## Icon Generation

To regenerate the app icon from SVG:

```bash
./scripts/generate-icon.sh
```

This creates:
- `communitas-dioxus/assets/icon.png` (1024x1024)
- `communitas-dioxus/assets/icon.icns` (all macOS sizes)

Requirements: `rsvg-convert` (from librsvg) or ImageMagick

## References

- [Apple Code Signing Guide](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Dioxus CLI Documentation](https://dioxuslabs.com/learn/0.6/CLI)
- [notarytool Reference](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution/customizing_the_notarization_workflow)
