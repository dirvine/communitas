# Update Signing Keys Setup Guide

## Overview

This guide explains how to generate Ed25519 signing keys for Communitas updates and configure them for GitHub Actions.

## Prerequisites

- Tauri CLI installed: `npm install -g @tauri-apps/cli`
- Access to GitHub repository settings
- Command line access

## Step 1: Generate Ed25519 Keypair

### Option A: Using the Helper Script (Recommended)

```bash
./scripts/generate-update-keys.sh
```

This script will:
1. Create `.keys/` directory (already in .gitignore)
2. Generate Ed25519 keypair
3. Prompt for password to protect private key
4. Display next steps

### Option B: Manual Generation

```bash
# Create keys directory
mkdir -p .keys

# Generate keypair
npx @tauri-apps/cli signer generate --write-keys .keys/updater-keys.json

# Enter password when prompted (or press Enter for no password)
```

### Understanding the Keys

The generated file `.keys/updater-keys.json` contains:

```json
{
  "privateKey": "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5...",
  "publicKey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEU4RjdDM..."
}
```

- **privateKey**: Secret key for signing releases (NEVER commit to git)
- **publicKey**: Public key for verifying updates (safe to embed in app)

## Step 2: Configure Public Key in Tauri

Open `communitas-desktop/tauri.conf.json` and update the `pubkey` field:

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

Replace `YOUR_PUBLIC_KEY_HERE` with the **publicKey** value from `.keys/updater-keys.json`.

**Example:**
```json
"pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEU4RjdDM0MyNEQzNjZFRDcKUldRYlZSNFcxMGcyOXJuVXhLWXlGMUJodEJqYjlRRzdVeVllRitGNUYvWHFjbVl6aGVKbGJRWT0K"
```

## Step 3: Add Private Key to GitHub Secrets

### Navigate to Repository Settings

1. Go to your repository on GitHub
2. Click **Settings** (repository settings, not account settings)
3. In left sidebar, click **Secrets and variables** → **Actions**
4. Click **New repository secret**

### Add Required Secrets

#### Secret 1: TAURI_SIGNING_PRIVATE_KEY

- **Name**: `TAURI_SIGNING_PRIVATE_KEY`
- **Value**: Copy the entire `privateKey` value from `.keys/updater-keys.json`

**Important**: Copy the ENTIRE string, including the base64-encoded data. It should start with something like:
```
dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5...
```

#### Secret 2: TAURI_SIGNING_PRIVATE_KEY_PASSWORD (Optional)

- **Name**: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- **Value**: The password you entered when generating the key

**Note**: If you didn't set a password (pressed Enter), you can skip this secret.

### Verify Secrets

After adding, you should see:
- ✅ TAURI_SIGNING_PRIVATE_KEY
- ✅ TAURI_SIGNING_PRIVATE_KEY_PASSWORD (if you used a password)

## Step 4: Verify Configuration

### Check tauri.conf.json

```bash
grep -A 3 '"updater"' communitas-desktop/tauri.conf.json
```

Should show:
```json
"updater": {
  "active": true,
  "endpoints": [...],
  "dialog": true,
  "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ..."  ← Should NOT be empty
}
```

### Check .gitignore

Verify private keys won't be committed:

```bash
git check-ignore .keys/updater-keys.json
```

Should output: `.keys/updater-keys.json` (meaning it's ignored)

### Check GitHub Secrets

Navigate to:
```
https://github.com/YOUR_USERNAME/communitas/settings/secrets/actions
```

Verify both secrets are present.

## Step 5: Test Signing

### Manual Test (Optional)

Build the app and verify signing:

```bash
# Build release
cd communitas-desktop
npm run tauri build

# Verify signature file exists
ls -la target/release/bundle/macos/*.app.tar.gz.sig

# Check signature content
file target/release/bundle/macos/*.app.tar.gz.sig
```

Should show: "data" or "ASCII text"

## Security Best Practices

### ✅ DO

- **Keep private key secret**: Never share or commit
- **Use strong password**: Protect private key with strong password
- **Backup keys**: Store securely offline (encrypted USB, password manager)
- **Rotate keys**: Plan to rotate annually
- **Review access**: Limit who can access GitHub Secrets

### ❌ DON'T

- **Don't commit private key**: Even temporarily
- **Don't share keys**: Via email, Slack, etc.
- **Don't store in plaintext**: Use password manager
- **Don't skip password**: Always protect private key
- **Don't ignore warnings**: If git tries to commit .keys/

## Troubleshooting

### Error: "Invalid signature"

**Cause**: Public key in tauri.conf.json doesn't match private key

**Solution**:
1. Verify you copied the correct public key
2. Ensure no extra spaces or characters
3. Regenerate keys if necessary

### Error: "Failed to sign update"

**Cause**: Private key not accessible in GitHub Actions

**Solution**:
1. Verify TAURI_SIGNING_PRIVATE_KEY exists in GitHub Secrets
2. Check secret name is exactly correct (case-sensitive)
3. Verify private key is complete

### Error: "Password required"

**Cause**: Private key is password-protected but password not provided

**Solution**:
1. Add TAURI_SIGNING_PRIVATE_KEY_PASSWORD to GitHub Secrets
2. Or regenerate keys without password

### Keys not in expected location

**Cause**: Tauri CLI may use default location

**Solution**:
Keys might be in `~/.tauri/` directory:

```bash
ls -la ~/.tauri/
cat ~/.tauri/*.keys
```

## Key Rotation

To rotate keys (recommended annually):

1. Generate new keypair (different filename)
2. Update tauri.conf.json with new public key
3. Update GitHub Secrets with new private key
4. Create new release with new signature
5. Old versions will continue working with old key
6. New versions will use new key

## Backup & Recovery

### Backup Private Key

```bash
# Encrypt and backup
openssl enc -aes-256-cbc -salt -in .keys/updater-keys.json \
  -out updater-keys.json.enc

# Store updater-keys.json.enc in secure location
# (password manager, encrypted USB, offline storage)
```

### Restore from Backup

```bash
# Decrypt backup
openssl enc -aes-256-cbc -d -in updater-keys.json.enc \
  -out .keys/updater-keys.json

# Re-add to GitHub Secrets
```

## Related Documentation

- **UPDATE_SYSTEM_SETUP.md** - Complete update system setup
- **UPDATE_SIGNING_DECISION.md** - Why we chose Ed25519
- **PQC_UPDATE_SIGNING_ANALYSIS.md** - Future PQC migration plan
- **PHASE1_TASK1_COMPLETE.md** - Implementation summary

## Support

If you encounter issues:

1. Check [Tauri Updater Documentation](https://v2.tauri.app/plugin/updater/)
2. Review GitHub Actions logs for signing errors
3. Verify all steps in this guide
4. Create issue in repository with error details

## Summary Checklist

Before proceeding to creating releases:

- [ ] Generated Ed25519 keypair
- [ ] Public key added to `communitas-desktop/tauri.conf.json`
- [ ] Private key added to GitHub Secrets as `TAURI_SIGNING_PRIVATE_KEY`
- [ ] Password added to GitHub Secrets as `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (if used)
- [ ] Verified `.keys/` directory is in `.gitignore`
- [ ] Backed up private key securely
- [ ] Tested configuration (optional)
- [ ] Documented key generation date

**Date Generated**: _______________

**Generated By**: _______________

**Next Rotation Due**: _______________ (1 year from generation)
