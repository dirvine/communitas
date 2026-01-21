#!/usr/bin/env bash
# Smoke test for macOS DMG installer
# Verifies code signing, notarization, and basic app structure
#
# Usage:
#   ./smoke-test-dmg.sh path/to/Communitas-v1.0.0-universal.dmg
#   ./smoke-test-dmg.sh path/to/Communitas.app  # Test app directly

set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────

log_pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASS_COUNT++))
}

log_fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAIL_COUNT++))
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((WARN_COUNT++))
}

log_info() {
    echo -e "  $1"
}

cleanup() {
    if [[ -n "${MOUNT_POINT:-}" && -d "$MOUNT_POINT" ]]; then
        echo ""
        echo "Cleaning up..."
        hdiutil detach "$MOUNT_POINT" -quiet 2>/dev/null || true
    fi
}

trap cleanup EXIT

# ─────────────────────────────────────────────────────────────────────────────
# Validation Functions
# ─────────────────────────────────────────────────────────────────────────────

check_dmg_signature() {
    local dmg_path="$1"
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "Checking DMG signature..."
    echo "═══════════════════════════════════════════════════════════════════"

    if codesign --verify --verbose=2 "$dmg_path" 2>&1; then
        log_pass "DMG is properly signed"
    else
        log_fail "DMG signature verification failed"
        return 1
    fi
}

check_dmg_notarization() {
    local dmg_path="$1"
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "Checking DMG notarization..."
    echo "═══════════════════════════════════════════════════════════════════"

    # spctl checks Gatekeeper assessment
    if spctl --assess --type open --context context:primary-signature --verbose=2 "$dmg_path" 2>&1; then
        log_pass "DMG passes Gatekeeper assessment"
    else
        log_warn "DMG Gatekeeper assessment failed (may not be notarized)"
    fi

    # Check for stapled ticket
    if xcrun stapler validate "$dmg_path" 2>&1 | grep -q "valid"; then
        log_pass "DMG has valid notarization ticket stapled"
    else
        log_warn "DMG may not have notarization ticket stapled"
    fi
}

mount_dmg() {
    local dmg_path="$1"
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "Mounting DMG..."
    echo "═══════════════════════════════════════════════════════════════════"

    MOUNT_INFO=$(hdiutil attach "$dmg_path" -nobrowse -readonly 2>&1) || {
        log_fail "Failed to mount DMG"
        echo "$MOUNT_INFO"
        return 1
    }

    MOUNT_POINT=$(echo "$MOUNT_INFO" | grep -o '/Volumes/[^[:space:]]*' | head -1)

    if [[ -z "$MOUNT_POINT" ]]; then
        log_fail "Could not determine mount point"
        return 1
    fi

    log_pass "DMG mounted at: $MOUNT_POINT"
}

find_app_in_dmg() {
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "Looking for app bundle..."
    echo "═══════════════════════════════════════════════════════════════════"

    APP_PATH=$(find "$MOUNT_POINT" -maxdepth 1 -name "*.app" -type d | head -1)

    if [[ -z "$APP_PATH" || ! -d "$APP_PATH" ]]; then
        log_fail "No .app bundle found in DMG"
        return 1
    fi

    log_pass "Found app: $(basename "$APP_PATH")"
    log_info "Path: $APP_PATH"
}

check_app_structure() {
    local app_path="$1"
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "Checking app bundle structure..."
    echo "═══════════════════════════════════════════════════════════════════"

    # Check Info.plist exists
    if [[ -f "$app_path/Contents/Info.plist" ]]; then
        log_pass "Info.plist exists"

        # Extract bundle identifier
        BUNDLE_ID=$(/usr/libexec/PlistBuddy -c "Print :CFBundleIdentifier" "$app_path/Contents/Info.plist" 2>/dev/null || echo "unknown")
        log_info "Bundle ID: $BUNDLE_ID"

        # Extract version
        BUNDLE_VERSION=$(/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" "$app_path/Contents/Info.plist" 2>/dev/null || echo "unknown")
        log_info "Version: $BUNDLE_VERSION"
    else
        log_fail "Info.plist missing"
    fi

    # Check main executable exists
    local app_name=$(basename "$app_path" .app)
    local exec_path="$app_path/Contents/MacOS/$app_name"

    if [[ -x "$exec_path" ]]; then
        log_pass "Main executable exists and is executable"

        # Check if universal binary
        ARCH_INFO=$(lipo -info "$exec_path" 2>&1 || echo "unknown")
        if echo "$ARCH_INFO" | grep -q "x86_64" && echo "$ARCH_INFO" | grep -q "arm64"; then
            log_pass "Universal binary (x86_64 + arm64)"
        elif echo "$ARCH_INFO" | grep -q "arm64"; then
            log_warn "arm64 only (not universal)"
        elif echo "$ARCH_INFO" | grep -q "x86_64"; then
            log_warn "x86_64 only (not universal)"
        else
            log_info "Architecture: $ARCH_INFO"
        fi
    else
        log_fail "Main executable missing or not executable: $exec_path"
    fi

    # Check Resources directory
    if [[ -d "$app_path/Contents/Resources" ]]; then
        log_pass "Resources directory exists"
    else
        log_warn "Resources directory missing"
    fi
}

check_app_signature() {
    local app_path="$1"
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "Checking app code signature..."
    echo "═══════════════════════════════════════════════════════════════════"

    # Basic signature verification
    if codesign --verify --deep --strict --verbose=2 "$app_path" 2>&1; then
        log_pass "App signature is valid"
    else
        log_fail "App signature verification failed"
    fi

    # Check signature details
    SIGN_INFO=$(codesign -dv "$app_path" 2>&1 || true)

    # Extract signing identity
    SIGNING_ID=$(echo "$SIGN_INFO" | grep "Authority=" | head -1 | sed 's/Authority=//')
    if [[ -n "$SIGNING_ID" ]]; then
        log_info "Signed by: $SIGNING_ID"
    fi

    # Check for hardened runtime
    if echo "$SIGN_INFO" | grep -q "runtime"; then
        log_pass "Hardened runtime enabled"
    else
        log_warn "Hardened runtime may not be enabled"
    fi

    # Check entitlements
    ENTITLEMENTS=$(codesign -d --entitlements - "$app_path" 2>&1 || true)
    if echo "$ENTITLEMENTS" | grep -q "com.apple.security"; then
        log_pass "Entitlements present"

        # Check specific entitlements
        if echo "$ENTITLEMENTS" | grep -q "com.apple.security.network.client"; then
            log_info "  - Network client: ✓"
        fi
        if echo "$ENTITLEMENTS" | grep -q "com.apple.security.network.server"; then
            log_info "  - Network server: ✓"
        fi
        if echo "$ENTITLEMENTS" | grep -q "com.apple.security.device.camera"; then
            log_info "  - Camera access: ✓"
        fi
        if echo "$ENTITLEMENTS" | grep -q "com.apple.security.device.audio-input"; then
            log_info "  - Microphone access: ✓"
        fi
    else
        log_warn "No entitlements found"
    fi
}

check_app_notarization() {
    local app_path="$1"
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "Checking app notarization..."
    echo "═══════════════════════════════════════════════════════════════════"

    # spctl assessment
    if spctl --assess --verbose=2 "$app_path" 2>&1; then
        log_pass "App passes Gatekeeper assessment"
    else
        log_warn "App Gatekeeper assessment failed (may not be notarized)"
    fi

    # Check for stapled ticket on app
    if xcrun stapler validate "$app_path" 2>&1 | grep -q "valid"; then
        log_pass "App has valid notarization ticket"
    else
        log_warn "App may not have notarization ticket (ticket may be on DMG)"
    fi
}

test_app_launch() {
    local app_path="$1"
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "Testing app launch (quick start/quit)..."
    echo "═══════════════════════════════════════════════════════════════════"

    # Only run if --launch flag provided
    if [[ "${DO_LAUNCH_TEST:-false}" != "true" ]]; then
        log_info "Skipping launch test (use --launch to enable)"
        return 0
    fi

    local app_name=$(basename "$app_path" .app)

    # Try to launch app and kill after 5 seconds
    open -a "$app_path" &
    OPEN_PID=$!

    sleep 5

    # Check if app is running
    if pgrep -f "$app_name" > /dev/null 2>&1; then
        log_pass "App launched successfully"

        # Kill the app
        pkill -f "$app_name" 2>/dev/null || true
        log_info "App terminated"
    else
        log_warn "App may have crashed or failed to start"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

main() {
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║           Communitas macOS Installer Smoke Test                   ║"
    echo "╚═══════════════════════════════════════════════════════════════════╝"

    if [[ $# -lt 1 ]]; then
        echo ""
        echo "Usage: $0 [--launch] <path/to/file.dmg or path/to/App.app>"
        echo ""
        echo "Options:"
        echo "  --launch    Also test that the app launches successfully"
        echo ""
        echo "Examples:"
        echo "  $0 Communitas-v1.0.0-universal.dmg"
        echo "  $0 --launch dist/bundle/Communitas.app"
        exit 1
    fi

    # Parse arguments
    DO_LAUNCH_TEST=false
    TARGET_PATH=""

    for arg in "$@"; do
        case "$arg" in
            --launch)
                DO_LAUNCH_TEST=true
                ;;
            *)
                TARGET_PATH="$arg"
                ;;
        esac
    done

    if [[ -z "$TARGET_PATH" ]]; then
        echo "Error: No target file specified"
        exit 1
    fi

    if [[ ! -e "$TARGET_PATH" ]]; then
        echo "Error: File not found: $TARGET_PATH"
        exit 1
    fi

    # Determine if testing DMG or app directly
    if [[ "$TARGET_PATH" == *.dmg ]]; then
        echo ""
        echo "Testing DMG: $TARGET_PATH"

        check_dmg_signature "$TARGET_PATH"
        check_dmg_notarization "$TARGET_PATH"
        mount_dmg "$TARGET_PATH"
        find_app_in_dmg
        check_app_structure "$APP_PATH"
        check_app_signature "$APP_PATH"
        check_app_notarization "$APP_PATH"
        test_app_launch "$APP_PATH"

    elif [[ -d "$TARGET_PATH" && "$TARGET_PATH" == *.app ]]; then
        echo ""
        echo "Testing App: $TARGET_PATH"
        APP_PATH="$TARGET_PATH"

        check_app_structure "$APP_PATH"
        check_app_signature "$APP_PATH"
        check_app_notarization "$APP_PATH"
        test_app_launch "$APP_PATH"
    else
        echo "Error: Target must be a .dmg file or .app bundle"
        exit 1
    fi

    # Summary
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "                           SUMMARY"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""
    echo -e "  ${GREEN}Passed:${NC}   $PASS_COUNT"
    echo -e "  ${RED}Failed:${NC}   $FAIL_COUNT"
    echo -e "  ${YELLOW}Warnings:${NC} $WARN_COUNT"
    echo ""

    if [[ $FAIL_COUNT -gt 0 ]]; then
        echo -e "${RED}SMOKE TEST FAILED${NC}"
        exit 1
    elif [[ $WARN_COUNT -gt 0 ]]; then
        echo -e "${YELLOW}SMOKE TEST PASSED WITH WARNINGS${NC}"
        exit 0
    else
        echo -e "${GREEN}SMOKE TEST PASSED${NC}"
        exit 0
    fi
}

main "$@"
