# macOS Code Signing and Notarization

This document describes how to set up macOS code signing and notarization for the Communitas Flutter app releases.

## Overview

Apple requires apps distributed outside the Mac App Store to be:
1. **Signed** with a Developer ID certificate
2. **Notarized** by Apple's notary service

Without both, users will see security warnings or be unable to run the app.

## Required GitHub Secrets

Configure these secrets in your GitHub repository settings (Settings > Secrets and variables > Actions):

| Secret Name | Description |
|-------------|-------------|
| `MACOS_CERTIFICATE` | Base64-encoded Developer ID Application certificate (.p12) |
| `MACOS_CERTIFICATE_PASSWORD` | Password for the .p12 certificate file |
| `KEYCHAIN_PASSWORD` | Password for the temporary keychain used during signing |
| `MACOS_SIGNING_IDENTITY` | Full signing identity string (e.g., "Developer ID Application: Your Name (TEAM_ID)") |
| `MACOS_NOTARIZATION_APPLE_ID` | Apple ID email used for notarization |
| `MACOS_NOTARIZATION_PASSWORD` | App-specific password (NOT your Apple ID password) |
| `MACOS_NOTARIZATION_TEAM_ID` | Your Apple Developer Team ID (10 characters) |

## Getting the Certificates

### 1. Developer ID Application Certificate

1. Log in to [Apple Developer](https://developer.apple.com/account)
2. Go to Certificates, Identifiers & Profiles > Certificates
3. Click "+" to create a new certificate
4. Select "Developer ID Application"
5. Follow the CSR (Certificate Signing Request) process
6. Download the certificate (.cer file)

### 2. Export as .p12

1. Open Keychain Access on your Mac
2. Import the .cer file (double-click)
3. Find the certificate under "My Certificates"
4. Right-click and select "Export..."
5. Choose .p12 format
6. Set a strong password (this becomes `MACOS_CERTIFICATE_PASSWORD`)

### 3. Base64 Encode

```bash
base64 -i DeveloperIDApplication.p12 | pbcopy
```

This copies the base64 string to your clipboard. Paste it as `MACOS_CERTIFICATE` secret.

### 4. Get Signing Identity

Find your full signing identity:
```bash
security find-identity -v -p codesigning
```

Look for a line like:
```
"Developer ID Application: Your Company Name (ABCD123456)"
```

Use this full string as `MACOS_SIGNING_IDENTITY`.

## Getting App-Specific Password

1. Go to [appleid.apple.com](https://appleid.apple.com)
2. Sign in and go to Security > App-Specific Passwords
3. Click "Generate an app-specific password"
4. Name it "GitHub Actions Notarization"
5. Copy the generated password as `MACOS_NOTARIZATION_PASSWORD`

**Important**: This is NOT your Apple ID password. It's a separate app-specific password.

## Getting Team ID

1. Go to [Apple Developer](https://developer.apple.com/account)
2. Find your Team ID in the upper right or Membership section
3. It's a 10-character alphanumeric string (e.g., `ABCD123456`)

## Bundle Identifier

The app uses bundle identifier `io.saorsa.communitas`. Make sure this matches:
- `communitas-flutter/macos/Runner/Configs/AppInfo.xcconfig`
- Any provisioning profiles or App IDs in Apple Developer portal

## Workflow Behavior

### With Signing Secrets
When all secrets are configured, the workflow will:
1. Build the Flutter macOS app
2. Sign the .app bundle with Developer ID Application certificate
3. Create a DMG installer
4. Sign the DMG
5. Submit to Apple for notarization
6. Staple the notarization ticket to the DMG
7. Upload to GitHub Release

### Without Signing Secrets
If secrets are not configured, the workflow will:
1. Build the Flutter macOS app (unsigned)
2. Create a DMG installer (unsigned)
3. Upload as artifact (not to release)

Users will need to bypass Gatekeeper to run unsigned builds.

## Entitlements

The app uses these entitlements:

### Release (communitas-flutter/macos/Runner/Release.entitlements)
```xml
<key>com.apple.security.app-sandbox</key>
<true/>
```

### Debug (communitas-flutter/macos/Runner/DebugProfile.entitlements)
```xml
<key>com.apple.security.app-sandbox</key>
<false/>
<key>com.apple.security.cs.allow-jit</key>
<true/>
<key>com.apple.security.network.server</key>
<true/>
```

## Troubleshooting

### "The app is damaged and can't be opened"
The DMG wasn't notarized or stapled correctly. Check the notarization logs.

### "Developer cannot be verified"
The app is signed but not notarized. Ensure notarization succeeded.

### Notarization timeout
Apple's notarization can take 5-30 minutes. The workflow uses `--wait` flag.

### Certificate expired
Developer ID certificates are valid for 5 years. Renew in Apple Developer portal.

## Manual Local Signing

For local testing:

```bash
# Build release
cd communitas-flutter
flutter build macos --release

# Sign (replace with your certificate)
codesign --force --deep --options runtime \
  --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --entitlements macos/Runner/Release.entitlements \
  build/macos/Build/Products/Release/communitas.app

# Create DMG
create-dmg \
  --volname "Communitas" \
  --app-drop-link 450 185 \
  Communitas.dmg \
  build/macos/Build/Products/Release/communitas.app

# Notarize
xcrun notarytool submit Communitas.dmg \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait

# Staple
xcrun stapler staple Communitas.dmg
```

## References

- [Apple Developer ID](https://developer.apple.com/developer-id/)
- [Notarizing macOS Software](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [App-Specific Passwords](https://support.apple.com/en-us/102654)
