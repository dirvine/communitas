# Beta Release Process

This document describes the process for creating and distributing beta releases of Communitas for macOS.

## Prerequisites

### Apple Developer Account
- Active Apple Developer Program membership ($99/year)
- Developer ID Application certificate
- App Store Connect API key (for notarization)

### GitHub Repository Secrets
Configure the following secrets in your GitHub repository settings:

| Secret | Description |
|--------|-------------|
| `MACOS_CERTIFICATE` | Base64-encoded Developer ID Application certificate (.p12) |
| `MACOS_CERTIFICATE_PASSWORD` | Password for the certificate |
| `KEYCHAIN_PASSWORD` | Temporary keychain password (any secure value) |
| `MACOS_SIGNING_IDENTITY` | Certificate common name (e.g., "Developer ID Application: MaidSafe (ABC123)") |
| `MACOS_NOTARIZATION_APPLE_ID` | Apple ID email for notarization |
| `MACOS_NOTARIZATION_PASSWORD` | App-specific password for notarization |
| `MACOS_NOTARIZATION_TEAM_ID` | Apple Developer Team ID |

### Generating Certificates

1. **Export Developer ID Certificate**:
   ```bash
   # In Keychain Access, export the certificate as .p12
   # Then base64 encode it:
   base64 -i DeveloperID.p12 | pbcopy
   # Paste this into MACOS_CERTIFICATE secret
   ```

2. **Create App-Specific Password**:
   - Go to https://appleid.apple.com
   - Sign in and go to Security > App-Specific Passwords
   - Generate a new password for "Communitas Notarization"
   - Use this for `MACOS_NOTARIZATION_PASSWORD`

### Tauri Signing Key (Optional)

For the auto-updater to work, you need to generate a signing key:

```bash
# Generate Tauri signing key
cargo install tauri-cli
cargo tauri signer generate -w ~/.tauri/communitas.key

# The public key goes in Dioxus.toml [bundle.updater]
# The private key is used to sign updates
```

## Release Process

### 1. Version Bump

Update the version in `communitas-dioxus/Cargo.toml`:

```toml
[package]
version = "1.0.0-beta.1"  # Bump this
```

Also update any other crates that need version sync.

### 2. Create Changelog Entry

Add an entry to `CHANGELOG.md`:

```markdown
## [1.0.0-beta.1] - 2026-01-23

### Added
- New onboarding tour for first-time users
- Version info in Settings page

### Fixed
- Tour overlay keyboard navigation
```

### 3. Commit and Tag

```bash
# Commit version bump
git add -A
git commit -m "release: prepare v1.0.0-beta.1"

# Create and push tag
git tag v1.0.0-beta.1
git push origin main --tags
```

### 4. Automatic Workflow

The `release-beta.yml` workflow will automatically:
1. Build universal macOS binary (x86_64 + arm64)
2. Sign with Developer ID
3. Create DMG
4. Notarize with Apple
5. Create GitHub pre-release
6. Upload DMG and update manifest

### 5. Manual Trigger (Alternative)

If you need to build without creating a tag:

1. Go to Actions > "Release Beta (macOS)"
2. Click "Run workflow"
3. Enter the version (e.g., "1.0.0-beta.2")
4. Click "Run workflow"

## Distribution

### Internal Team Distribution

1. Go to the GitHub Releases page
2. Download the DMG from the pre-release
3. Share the DMG directly or via file sharing

### Download Instructions for Testers

Send testers these instructions:

```
# Communitas Beta Installation

1. Download Communitas-1.0.0-beta.1-universal.dmg from:
   https://github.com/maidsafe/communitas/releases

2. Open the downloaded DMG file

3. Drag "Communitas" to your Applications folder

4. Launch Communitas from Applications

5. If you see "Communitas can't be opened":
   - Go to System Preferences > Security & Privacy
   - Click "Open Anyway" for Communitas

6. The app should now launch with the onboarding tour

To report issues: https://github.com/maidsafe/communitas/issues
```

### Uninstallation

```bash
# Remove the application
rm -rf /Applications/Communitas.app

# Remove application data (optional)
rm -rf ~/Library/Application\ Support/com.maidsafe.communitas
rm -rf ~/Library/Caches/com.maidsafe.communitas
rm -rf ~/Library/Preferences/com.maidsafe.communitas.plist
```

## Auto-Update System

### How It Works

1. App checks `update.json` at configured endpoint
2. If newer version found, prompts user
3. Downloads and installs update in background
4. Restarts app when ready

### Update Manifest Format

```json
{
  "version": "1.0.0-beta.2",
  "notes": "Bug fixes and improvements",
  "pub_date": "2026-01-24T12:00:00Z",
  "platforms": {
    "darwin-x86_64": {
      "signature": "base64-encoded-signature",
      "url": "https://github.com/.../download/v1.0.0-beta.2/Communitas.dmg"
    },
    "darwin-aarch64": {
      "signature": "base64-encoded-signature",
      "url": "https://github.com/.../download/v1.0.0-beta.2/Communitas.dmg"
    }
  }
}
```

### Manual Manifest Generation

```bash
# Generate update manifest locally
./scripts/generate-update-manifest.sh \
  --version 1.0.0-beta.2 \
  --dmg-path ./dist/Communitas-1.0.0-beta.2-universal.dmg \
  --notes "Bug fixes and performance improvements"
```

## Troubleshooting

### Notarization Failures

**"The software is not signed"**
- Check that `MACOS_SIGNING_IDENTITY` matches your certificate exactly
- Verify the certificate is valid and not expired

**"The app is from an unidentified developer"**
- Notarization may not have completed
- Check the workflow logs for notarization status

**Notarization timeout**
- Apple's servers may be slow
- The workflow waits up to 1 hour
- Retry the workflow if it times out

### Build Failures

**"dx: command not found"**
- Ensure dx CLI is installed: `cargo install dioxus-cli --version 0.7.3`

**"No such file or directory: entitlements.plist"**
- Create `communitas-dioxus/entitlements.plist` with required entitlements

### Certificate Issues

**"No identity found"**
- Check that the certificate is properly base64 encoded
- Verify the password is correct
- Ensure the certificate is for "Developer ID Application" (not Mac App Store)

## Security Considerations

- Never commit secrets to the repository
- Rotate certificates before expiration
- Use app-specific passwords, not your main Apple ID password
- Review and limit repository access for team members
- Consider code signing verification in CI

## Support

- GitHub Issues: https://github.com/maidsafe/communitas/issues
- MaidSafe Discord: https://discord.gg/maidsafe
- Documentation: https://github.com/maidsafe/communitas/docs
