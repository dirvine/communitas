#!/usr/bin/env bash
#
# Install WebKitGTK for Communitas on Linux
#
# This script detects the package manager and installs the appropriate
# WebKitGTK development package required by Dioxus/Wry.
#
# Usage:
#   ./install-webview-linux.sh
#
# Supported package managers:
#   - apt (Debian, Ubuntu, Linux Mint, Pop!_OS)
#   - dnf (Fedora, RHEL 8+, CentOS Stream)
#   - pacman (Arch Linux, Manjaro, EndeavourOS)
#   - zypper (openSUSE)
#   - apk (Alpine Linux)

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run with sudo or as root"
        echo "Please run: sudo $0"
        exit 1
    fi
}

detect_package_manager() {
    if command -v apt-get &>/dev/null; then
        echo "apt"
    elif command -v dnf &>/dev/null; then
        echo "dnf"
    elif command -v pacman &>/dev/null; then
        echo "pacman"
    elif command -v zypper &>/dev/null; then
        echo "zypper"
    elif command -v apk &>/dev/null; then
        echo "apk"
    else
        echo "unknown"
    fi
}

install_webkit_apt() {
    log_info "Installing WebKitGTK via apt..."
    apt-get update
    # Try webkit2gtk-4.1 first (newer), fall back to 4.0
    if apt-cache show libwebkit2gtk-4.1-dev &>/dev/null; then
        apt-get install -y libwebkit2gtk-4.1-dev
    else
        log_warn "webkit2gtk-4.1 not available, installing 4.0..."
        apt-get install -y libwebkit2gtk-4.0-dev
    fi
}

install_webkit_dnf() {
    log_info "Installing WebKitGTK via dnf..."
    # Fedora/RHEL use webkit2gtk4.1-devel
    if dnf info webkit2gtk4.1-devel &>/dev/null 2>&1; then
        dnf install -y webkit2gtk4.1-devel
    else
        log_warn "webkit2gtk4.1 not available, installing webkit2gtk3-devel..."
        dnf install -y webkit2gtk3-devel
    fi
}

install_webkit_pacman() {
    log_info "Installing WebKitGTK via pacman..."
    pacman -Sy --noconfirm webkit2gtk-4.1 || pacman -Sy --noconfirm webkit2gtk
}

install_webkit_zypper() {
    log_info "Installing WebKitGTK via zypper..."
    zypper refresh
    zypper install -y webkit2gtk3-devel
}

install_webkit_apk() {
    log_info "Installing WebKitGTK via apk..."
    apk update
    apk add webkit2gtk-dev
}

verify_installation() {
    log_info "Verifying WebKitGTK installation..."

    # Check if pkg-config can find webkit2gtk
    if pkg-config --exists webkit2gtk-4.1 2>/dev/null; then
        local version
        version=$(pkg-config --modversion webkit2gtk-4.1)
        log_info "WebKitGTK 4.1 installed successfully (version: $version)"
        return 0
    elif pkg-config --exists webkit2gtk-4.0 2>/dev/null; then
        local version
        version=$(pkg-config --modversion webkit2gtk-4.0)
        log_info "WebKitGTK 4.0 installed successfully (version: $version)"
        return 0
    else
        log_error "WebKitGTK installation could not be verified"
        return 1
    fi
}

main() {
    echo "========================================"
    echo "  Communitas WebKitGTK Installer"
    echo "========================================"
    echo

    check_root

    local pm
    pm=$(detect_package_manager)
    log_info "Detected package manager: $pm"

    case "$pm" in
        apt)
            install_webkit_apt
            ;;
        dnf)
            install_webkit_dnf
            ;;
        pacman)
            install_webkit_pacman
            ;;
        zypper)
            install_webkit_zypper
            ;;
        apk)
            install_webkit_apk
            ;;
        *)
            log_error "Unsupported package manager"
            echo
            echo "Please install WebKitGTK manually:"
            echo "  - Debian/Ubuntu: sudo apt install libwebkit2gtk-4.1-dev"
            echo "  - Fedora: sudo dnf install webkit2gtk4.1-devel"
            echo "  - Arch: sudo pacman -S webkit2gtk-4.1"
            echo "  - openSUSE: sudo zypper install webkit2gtk3-devel"
            exit 1
            ;;
    esac

    if verify_installation; then
        echo
        log_info "WebKitGTK installation complete!"
        echo "You can now run Communitas."
    else
        log_error "Installation may have failed. Please check the output above."
        exit 1
    fi
}

main "$@"
