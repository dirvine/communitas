#!/bin/bash

# Generate Tauri Updater Keys
# This script generates the private/public key pair needed for Tauri auto-updates

set -e

echo "🔐 Generating Tauri Updater Keys"
echo "================================="

# Check if tauri-cli is installed
if ! command -v tauri &> /dev/null; then
    echo "❌ tauri-cli is not installed. Please install it first:"
    echo "   cargo install tauri-cli"
    exit 1
fi

# Generate the key pair
echo "📝 Generating key pair..."
tauri signer generate -w ~/.tauri/keys

# Get the private key (for CI/CD)
echo ""
echo "🔑 Private Key (store as TAURI_UPDATER_PRIVATE_KEY in GitHub secrets):"
if [ -f ~/.tauri/keys/private.key ]; then
    cat ~/.tauri/keys/private.key
    echo ""
    echo "⚠️  IMPORTANT: Store this private key securely in your CI/CD secrets!"
    echo "   Never commit it to version control."
else
    echo "❌ Private key file not found"
    exit 1
fi

echo ""
echo "🛡️  Public Key (store as TAURI_UPDATER_PUBKEY in GitHub secrets):"
if [ -f ~/.tauri/keys/public.key ]; then
    cat ~/.tauri/keys/public.key
    echo ""
    echo "📋 Copy this public key to your tauri.conf.json 'updater.pubkey' field"
else
    echo "❌ Public key file not found"
    exit 1
fi

echo ""
echo "✅ Key generation complete!"
echo ""
echo "Next steps:"
echo "1. Add TAURI_UPDATER_PRIVATE_KEY to your GitHub repository secrets"
echo "2. Add TAURI_UPDATER_PUBKEY to your GitHub repository secrets"
echo "3. Update tauri.conf.json with the public key"
echo "4. Set up the latest.json generation workflow"

echo ""
echo "🔒 Security Notes:"
echo "- Keep the private key secure and never expose it"
echo "- The private key is used to sign update packages"
echo "- The public key is used by clients to verify update signatures"
echo "- Rotate keys periodically for security"
