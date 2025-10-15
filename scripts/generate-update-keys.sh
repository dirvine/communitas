#!/bin/bash
# Generate Ed25519 keypair for Tauri updater
# This script creates the signing keys needed for secure updates

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEYS_DIR="$SCRIPT_DIR/../.keys"

# Create keys directory if it doesn't exist
mkdir -p "$KEYS_DIR"

echo "🔐 Generating Ed25519 keypair for Tauri updater..."
echo ""
echo "⚠️  IMPORTANT: You will be prompted to enter a password to protect the private key."
echo "    This password will be stored in GitHub Secrets as TAURI_SIGNING_PRIVATE_KEY_PASSWORD"
echo ""
echo "💡 TIP: Use a strong password or press Enter for no password (less secure)"
echo ""

# Generate keypair using Tauri CLI
cd "$SCRIPT_DIR/.."
npx @tauri-apps/cli signer generate --write-keys "$KEYS_DIR/updater-keys.json"

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Keypair generated successfully!"
    echo ""
    echo "📝 Next steps:"
    echo "1. Copy the PUBLIC KEY from below and add it to communitas-desktop/tauri.conf.json"
    echo "   Replace the empty 'pubkey' field with your public key"
    echo ""
    echo "2. Add these secrets to GitHub (Settings → Secrets and variables → Actions):"
    echo "   - TAURI_SIGNING_PRIVATE_KEY: Copy the private key from the file"
    echo "   - TAURI_SIGNING_PRIVATE_KEY_PASSWORD: The password you just entered (if any)"
    echo ""
    echo "3. NEVER commit the private key to git!"
    echo "   (The .keys/ directory is already in .gitignore)"
    echo ""
    echo "🔑 Keys saved to: $KEYS_DIR/updater-keys.json"
    echo ""

    if [ -f "$KEYS_DIR/updater-keys.json" ]; then
        echo "📋 Generated keys:"
        cat "$KEYS_DIR/updater-keys.json"
    else
        echo "⚠️  Key file not found. Keys may be in default location."
        echo "    Check: ~/.tauri/ directory"
    fi
else
    echo ""
    echo "❌ Key generation failed!"
    echo "   Try running manually: npx @tauri-apps/cli signer generate -w .keys/updater-keys.json"
fi
