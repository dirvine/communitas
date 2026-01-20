# Development Prerequisites

This document lists all dependencies required to build and run Communitas.

## Platform Requirements

### All Platforms

| Requirement | Version | Purpose |
|------------|---------|---------|
| Rust | 1.85+ | Compiler and toolchain |
| dx CLI | 0.7.3 | Dioxus build tool |
| Git | 2.0+ | Version control |

### macOS

macOS has minimal additional requirements since WebKit is bundled with the OS.

| Requirement | Version | Notes |
|------------|---------|-------|
| Xcode CLT | Latest | Command line tools (`xcode-select --install`) |
| WebKit | Bundled | Always available on macOS |

### Linux

Linux requires WebKitGTK for the desktop application's WebView.

| Requirement | Version | Notes |
|------------|---------|-------|
| WebKitGTK | 4.1 or 4.0 | Required for Dioxus/Wry |
| pkg-config | Latest | For library detection |
| Build essentials | Latest | gcc, make, etc. |

**Install WebKitGTK:**

```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel

# Arch Linux
sudo pacman -S webkit2gtk-4.1

# openSUSE
sudo zypper install webkit2gtk3-devel

# Or use the installer script:
sudo scripts/install-webview-linux.sh
```

### Windows

Windows requires WebView2 Runtime and build tools for cryptography compilation.

| Requirement | Version | Notes |
|------------|---------|-------|
| WebView2 | Latest | Microsoft Edge WebView2 Runtime |
| VS Build Tools | 2022 | C++ workload for aws-lc-rs |
| CMake | 3.20+ | Must be in PATH |

**Install WebView2:**

WebView2 may already be installed if you have Microsoft Edge or Windows 11. To check or install:

```powershell
# Check if WebView2 is installed
scripts\install-webview-windows.ps1

# Install for current user only (no admin required)
scripts\install-webview-windows.ps1 -UserInstall

# Force reinstall
scripts\install-webview-windows.ps1 -Force
```

Or download manually from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

**Install Build Tools:**

See [windows-build.md](windows-build.md) for detailed Visual Studio and CMake setup.

## Verifying Installation

### Check WebView Availability

Communitas automatically checks for WebView at startup. If missing, it displays a native error dialog with installation instructions.

### Verify Rust Toolchain

```bash
# Check Rust version
rustc --version  # Should be 1.85.0 or higher

# Check dx CLI
dx --version     # Should be 0.7.3

# Check build works
cargo build -p communitas-dioxus
```

### Run the Application

```bash
cd communitas-dioxus
dx serve --platform desktop --hotpatch
```

## Troubleshooting

### WebView Not Found (Linux)

**Symptom:** Error dialog says "WebKitGTK is not installed"

**Solution:**
1. Install WebKitGTK using your package manager (see above)
2. Verify installation: `pkg-config --exists webkit2gtk-4.1 && echo "Installed"`
3. If using an unusual distro, check `/usr/lib*/libwebkit2gtk-4*.so` exists

### WebView Not Found (Windows)

**Symptom:** Error dialog says "WebView2 Runtime is not installed"

**Solution:**
1. Run the installer script: `scripts\install-webview-windows.ps1`
2. Or install Microsoft Edge browser
3. On Windows 11, WebView2 should be built-in - try restarting

### Build Fails on Windows

**Symptom:** `aws-lc-rs` or CMake errors

**Solution:**
See [windows-build.md](windows-build.md) for complete setup instructions.

### dx CLI Not Found

**Symptom:** `dx: command not found`

**Solution:**
```bash
# Install pinned version
scripts/install_dx.sh

# Or install manually
cargo install dioxus-cli@0.7.3
```
