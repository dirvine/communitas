# PLAN-35: Phase 5.1 - Packaging & Installers (macOS)

**Milestone**: M5 Stabilization
**Phase**: 5.1 - Packaging & Installers
**Status**: Planning
**Created**: 2026-01-21

## Goal

Produce signed, notarized macOS DMG installers for the Dioxus desktop application.

**Scope**: macOS only (no Windows certificates available).

## Current State

- **Dioxus app**: `communitas-dioxus/` with minimal Dioxus.toml
- **dx CLI**: Version 0.7.3 pinned via `scripts/install_dx.sh`
- **Assets**: Only `header.svg` and `main.css` - no app icon
- **Existing workflows**: `release-headless.yml` for CLI binaries, no desktop app workflow

### GitHub Secrets Available (macOS)

| Secret | Purpose |
|--------|---------|
| `MACOS_CERTIFICATE` | Developer ID Application certificate (base64) |
| `MACOS_CERTIFICATE_PASSWORD` | Certificate p12 password |
| `MACOS_SIGNING_IDENTITY` | Signing identity string |
| `APPLE_INSTALLER_CERTIFICATE` | Developer ID Installer certificate |
| `APPLE_INSTALLER_CERTIFICATE_PASSWORD` | Installer cert password |
| `MACOS_NOTARIZATION_APPLE_ID` | Apple ID for notarization |
| `MACOS_NOTARIZATION_PASSWORD` | App-specific password |
| `MACOS_NOTARIZATION_TEAM_ID` | Team ID |
| `KEYCHAIN_PASSWORD` | Temporary keychain password |

## Tasks

### Task 1: Create macOS App Icon

**Files**: `communitas-dioxus/assets/`
**Approach**:
1. Create 1024x1024 PNG master icon (from existing branding or new design)
2. Generate .icns file with all required sizes (16, 32, 64, 128, 256, 512, 1024)
3. Use `iconutil` or `sips` for conversion
4. Alternative: Create simple iconset programmatically

**Done when**:
- `communitas-dioxus/assets/icon.icns` exists
- `communitas-dioxus/assets/icon.png` exists (1024x1024)
- Icons render correctly in Finder

### Task 2: Configure Dioxus.toml Bundle Settings

**Files**: `communitas-dioxus/Dioxus.toml`
**Approach**:
1. Add `[bundle]` section with identifier, publisher, icon
2. Add `[bundle.macos]` section with:
   - `minimum_system_version = "11.0"` (Big Sur+)
   - `hardened_runtime = true`
   - Proper entitlements for networking
3. Set appropriate category (Social Networking or Productivity)

**Example config**:
```toml
[bundle]
identifier = "com.maidsafe.communitas"
publisher = "MaidSafe"
icon = ["assets/icon.png", "assets/icon.icns"]
category = "Social Networking"
short_description = "Decentralized collaboration platform"
copyright = "Copyright 2024-2026 MaidSafe. All rights reserved."

[bundle.macos]
minimum_system_version = "11.0"
hardened_runtime = true
entitlements = "entitlements.plist"
```

**Done when**:
- `dx bundle --platform desktop` produces valid .app
- Bundle identifier is correct in Info.plist

### Task 3: Create Entitlements File

**Files**: `communitas-dioxus/entitlements.plist`
**Approach**:
1. Create entitlements.plist with required permissions:
   - Network (client/server)
   - File access (user-selected)
   - Camera/Microphone (for calls)
2. Keep minimal - only what's needed

**Done when**:
- Entitlements file validates
- App runs with hardened runtime

### Task 4: Create macOS Release Workflow

**Files**: `.github/workflows/release-desktop.yml`
**Approach**:
1. Build universal binary (x86_64 + aarch64)
2. Import certificates to temporary keychain
3. Sign with `codesign --deep --force --options runtime`
4. Create DMG with `hdiutil`
5. Sign DMG
6. Notarize with `xcrun notarytool`
7. Staple notarization ticket
8. Upload to GitHub Release

**Workflow structure**:
```yaml
jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - Install dx CLI
      - Build universal app bundle
      - Import certificates
      - Sign app bundle
      - Create DMG
      - Sign DMG
      - Notarize
      - Staple
      - Upload artifact

  release:
    needs: build-macos
    steps:
      - Create/update GitHub release
      - Attach DMG
```

**Done when**:
- Workflow runs successfully
- DMG is signed and notarized
- DMG downloads and mounts on clean Mac

### Task 5: Add Installer Smoke Test

**Files**: `.github/workflows/release-desktop.yml`, `scripts/tests/`
**Approach**:
1. After creating DMG, mount and verify contents
2. Check code signature: `codesign --verify --deep --strict`
3. Check notarization: `spctl --assess --type open --context context:primary-signature`
4. Verify app launches (basic smoke test)

**Done when**:
- Smoke tests pass in CI
- Invalid signatures fail the build

### Task 6: Document Bundle Process

**Files**: `docs/development/macos-bundle.md`
**Approach**:
1. Document certificate requirements
2. Document local signing for development
3. Document CI workflow
4. Troubleshooting section

**Done when**:
- Developer can understand signing process
- CI secrets are documented (without values)

## Verification

```bash
# Local build (unsigned)
cd communitas-dioxus
dx bundle --platform desktop

# Verify app structure
ls -la dist/bundle/macos/Communitas.app/Contents/

# Check Info.plist
plutil -p dist/bundle/macos/Communitas.app/Contents/Info.plist

# CI verification (after workflow runs)
# Download DMG from release
# Mount and check signature
codesign --verify --deep --strict /Volumes/Communitas/Communitas.app
spctl --assess --type execute /Volumes/Communitas/Communitas.app
```

## Dependencies

- macOS runner (GitHub Actions `macos-latest`)
- dx CLI 0.7.3
- Apple Developer ID certificates (already in GitHub Secrets)
- Notarization credentials (already in GitHub Secrets)

## Notes

- Windows installers deferred (no certificates available)
- Linux AppImage can be added later (no signing required)
- Universal binaries (x86_64 + aarch64) for Intel/Apple Silicon support
- Minimum macOS 11.0 (Big Sur) for best WebView support
