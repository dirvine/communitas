#!/bin/bash
#
# Generate Tauri update manifest (update.json)
#
# This script generates the update manifest file used by Tauri's updater
# to detect and download new versions of the application.
#
# Usage:
#   ./generate-update-manifest.sh --version VERSION --dmg-path DMG_PATH [OPTIONS]
#
# Required:
#   --version VERSION      Version string (e.g., "1.0.0-beta.1")
#   --dmg-path DMG_PATH    Path to the DMG file
#
# Optional:
#   --signature SIGNATURE  Update signature (from tauri-cli sign)
#   --notes "NOTES"        Release notes text
#   --repo REPO            GitHub repository (default: maidsafe/communitas)
#   --output PATH          Output file path (default: ./update.json)
#   --help                 Show this help message
#
# Examples:
#   ./generate-update-manifest.sh --version 1.0.0-beta.1 --dmg-path ./dist/Communitas.dmg
#   ./generate-update-manifest.sh --version 1.0.0 --dmg-path ./dist/Communitas.dmg --signature "sig..."
#

set -euo pipefail

# Defaults
VERSION=""
DMG_PATH=""
SIGNATURE=""
NOTES=""
REPO="maidsafe/communitas"
OUTPUT="./update.json"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --dmg-path)
            DMG_PATH="$2"
            shift 2
            ;;
        --signature)
            SIGNATURE="$2"
            shift 2
            ;;
        --notes)
            NOTES="$2"
            shift 2
            ;;
        --repo)
            REPO="$2"
            shift 2
            ;;
        --output)
            OUTPUT="$2"
            shift 2
            ;;
        --help)
            head -30 "$0" | tail -27
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$VERSION" ]]; then
    echo "Error: --version is required"
    exit 1
fi

if [[ -z "$DMG_PATH" ]]; then
    echo "Error: --dmg-path is required"
    exit 1
fi

# Check if DMG exists
if [[ ! -f "$DMG_PATH" ]]; then
    echo "Error: DMG file not found: $DMG_PATH"
    exit 1
fi

# Get DMG filename
DMG_NAME=$(basename "$DMG_PATH")

# Generate publication date
PUB_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Generate default notes if not provided
if [[ -z "$NOTES" ]]; then
    NOTES="Communitas version ${VERSION}. See https://github.com/${REPO}/releases/tag/v${VERSION} for details."
fi

# Construct download URL
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${DMG_NAME}"

# Generate update.json
cat > "$OUTPUT" << EOF
{
  "version": "${VERSION}",
  "notes": "${NOTES}",
  "pub_date": "${PUB_DATE}",
  "platforms": {
    "darwin-x86_64": {
      "signature": "${SIGNATURE}",
      "url": "${DOWNLOAD_URL}"
    },
    "darwin-aarch64": {
      "signature": "${SIGNATURE}",
      "url": "${DOWNLOAD_URL}"
    }
  }
}
EOF

echo "Generated update manifest: $OUTPUT"
echo "  Version: $VERSION"
echo "  DMG: $DMG_NAME"
echo "  URL: $DOWNLOAD_URL"
echo "  Publication date: $PUB_DATE"

# Validate JSON
if command -v jq &> /dev/null; then
    if jq empty "$OUTPUT" 2>/dev/null; then
        echo "  JSON validation: PASS"
    else
        echo "  JSON validation: FAIL"
        exit 1
    fi
fi
