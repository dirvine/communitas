#!/bin/bash
# Generate Ed25519 keypair for Tauri updater
# This script creates the signing keys needed for secure updates

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEYS_DIR="$SCRIPT_DIR/../.keys"

# Create keys directory if it doesn't exist
mkdir -p "$KEYS_DIR"

echo "🔐 Generating Ed25519 keypair for Tauri updater..."

# Generate keypair using Tauri CLI
cd "$SCRIPT_DIR/../communitas-desktop"
cargo tauri signer generate --write-keys "$KEYS_DIR/updater-keys.json"

echo ""
echo "✅ Keypair generated successfully!"
echo ""
echo "📝 Next steps:"
echo "1. Copy the PUBLIC KEY and add it to communitas-desktop/tauri.conf.json"
echo "2. Add the PRIVATE KEY to GitHub Secrets as TAURI_SIGNING_PRIVATE_KEY"
echo "3. NEVER commit the private key to git!"
echo ""
echo "🔑 Keys saved to: $KEYS_DIR/updater-keys.json"
echo ""
cat "$KEYS_DIR/updater-keys.json"
