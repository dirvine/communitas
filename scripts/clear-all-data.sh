#!/bin/bash
# Clear All Data Script for Communitas
# This script removes all user data, vaults, network caches, and preferences

set -e  # Exit on error

echo "🧹 Communitas Data Cleanup Script"
echo "=================================="
echo ""
echo "⚠️  WARNING: This will delete ALL Communitas data!"
echo ""
echo "This includes:"
echo "  - All user vaults and identities"
echo "  - All documents and messages"
echo "  - All network peer caches"
echo "  - All application preferences"
echo "  - All authentication sessions"
echo ""
read -p "Are you sure you want to continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "❌ Aborted."
    exit 0
fi

echo ""
echo "🛑 Checking for running processes..."

# Check if app is running
if pgrep -f "communitas-desktop" > /dev/null; then
    echo "⚠️  Communitas is currently running!"
    read -p "Kill the running app? (yes/no): " kill_confirm
    if [ "$kill_confirm" = "yes" ]; then
        pkill -f "communitas-desktop" || true
        pkill -f "tauri dev" || true
        echo "✅ Processes killed"
        sleep 2
    else
        echo "❌ Please close the app manually and run this script again."
        exit 1
    fi
fi

echo ""
echo "🗑️  Clearing data..."

# macOS specific paths
if [ "$(uname)" = "Darwin" ]; then
    echo "  → Detected macOS"

    # Application Support (all bundle identifiers)
    rm -rf ~/Library/Application\ Support/com.saorsalabs.communitas 2>/dev/null && echo "  ✓ Cleared com.saorsalabs.communitas" || true
    rm -rf ~/Library/Application\ Support/com.saorsa.communitas 2>/dev/null && echo "  ✓ Cleared com.saorsa.communitas" || true
    rm -rf ~/Library/Application\ Support/com.p2pfoundation.communitas 2>/dev/null && echo "  ✓ Cleared com.p2pfoundation.communitas" || true
    rm -rf ~/Library/Application\ Support/communitas 2>/dev/null && echo "  ✓ Cleared communitas" || true
    rm -rf ~/Library/Application\ Support/communitas-tui 2>/dev/null && echo "  ✓ Cleared communitas-tui" || true
    rm -rf ~/Library/Application\ Support/.communitas-vaults 2>/dev/null && echo "  ✓ Cleared .communitas-vaults" || true

    # Caches (all variants)
    rm -rf ~/Library/Caches/com.saorsalabs.communitas 2>/dev/null && echo "  ✓ Cleared Caches (com.saorsalabs)" || true
    rm -rf ~/Library/Caches/communitas 2>/dev/null && echo "  ✓ Cleared Caches (communitas)" || true
    rm -rf ~/Library/Caches/communitas-desktop 2>/dev/null && echo "  ✓ Cleared Caches (communitas-desktop)" || true
    rm -rf ~/Library/Caches/communitas-tauri 2>/dev/null && echo "  ✓ Cleared Caches (communitas-tauri)" || true

    # WebKit storage
    rm -rf ~/Library/WebKit/communitas 2>/dev/null && echo "  ✓ Cleared WebKit (communitas)" || true
    rm -rf ~/Library/WebKit/communitas-desktop 2>/dev/null && echo "  ✓ Cleared WebKit (communitas-desktop)" || true
    rm -rf ~/Library/WebKit/communitas-tauri 2>/dev/null && echo "  ✓ Cleared WebKit (communitas-tauri)" || true

    # Logs
    rm -rf ~/Library/Logs/com.p2pfoundation.communitas 2>/dev/null && echo "  ✓ Cleared Logs" || true

    # Preferences
    rm -f ~/Library/Preferences/com.saorsalabs.communitas.plist 2>/dev/null && echo "  ✓ Cleared Preferences" || true

    echo "  ✅ macOS-specific paths cleared"
fi

# Cross-platform paths
echo "  → Clearing cross-platform data"

# Home directory
rm -rf ~/.communitas 2>/dev/null && echo "  ✓ Cleared ~/.communitas" || echo "  - No ~/.communitas"
rm -rf ~/.communitas-vaults 2>/dev/null && echo "  ✓ Cleared ~/.communitas-vaults" || echo "  - No ~/.communitas-vaults"

# XDG paths (Linux)
rm -rf ~/.local/share/communitas 2>/dev/null && echo "  ✓ Cleared ~/.local/share/communitas" || echo "  - No ~/.local/share/communitas"
rm -rf ~/.config/communitas 2>/dev/null && echo "  ✓ Cleared ~/.config/communitas" || echo "  - No ~/.config/communitas"
rm -rf ~/.cache/communitas 2>/dev/null && echo "  ✓ Cleared ~/.cache/communitas" || echo "  - No ~/.cache/communitas"

# Project-local data
echo "  → Clearing project-local data"
rm -rf .communitas-data 2>/dev/null && echo "  ✓ Cleared .communitas-data" || echo "  - No .communitas-data"
rm -f peer_cache.db 2>/dev/null && echo "  ✓ Cleared peer_cache.db" || echo "  - No peer_cache.db"

# Database files in current directory
rm -f communitas.db 2>/dev/null && echo "  ✓ Cleared communitas.db" || echo "  - No communitas.db"

echo ""
echo "✅ Data cleanup complete!"
echo ""
echo "📝 Next steps:"
echo "  1. Restart the app"
echo "  2. You'll see the first-launch welcome screen"
echo "  3. Create a new identity"
echo ""
echo "💡 Tip: Your four-word identity will be freshly generated and"
echo "   your network address will be assigned on first connection."
echo ""
echo "🔧 Development Mode:"
echo "   To use centralized data in ~/.communitas/ for easy cleanup:"
echo "   COMMUNITAS_DEV_MODE=1 npm run tauri dev"
echo ""
