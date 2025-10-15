# Troubleshooting Guide

Common issues and solutions for Communitas development.

## Table of Contents

- [Build Issues](#build-issues)
- [Runtime Issues](#runtime-issues)
- [Testing Issues](#testing-issues)
- [Network Issues](#network-issues)
- [Platform-Specific Issues](#platform-specific-issues)
- [Performance Issues](#performance-issues)
- [Development Tools](#development-tools)

---

## Build Issues

### Rust Compilation Errors

#### Issue: "error: could not compile `communitas-core`"

**Symptoms**:
```
error: could not compile `communitas-core` due to 5 previous errors
```

**Solutions**:

**1. Clean build artifacts**:
```bash
cargo clean
cargo build
```

**2. Update Rust toolchain**:
```bash
rustup update stable
rustup default stable
cargo --version  # Should be 1.85+
```

**3. Clear cargo cache**:
```bash
rm -rf ~/.cargo/registry/index/*
rm -rf ~/.cargo/git/db/*
cargo build
```

---

#### Issue: "linker error" or "undefined reference"

**Symptoms**:
```
= note: /usr/bin/ld: cannot find -lwebkit2gtk-4.1
```

**Solutions**:

**macOS**:
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Update Homebrew packages
brew update
brew upgrade
```

**Linux (Ubuntu/Debian)**:
```bash
sudo apt update
sudo apt install build-essential libwebkit2gtk-4.1-dev \
  libssl-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev
```

**Windows**:
- Reinstall Visual Studio 2022 Build Tools
- Ensure Windows SDK is installed

---

#### Issue: "error: requires `cargo` version X, but only version Y is installed"

**Symptoms**:
```
error: package requires rustc 1.85 or newer
```

**Solution**:
```bash
# Update Rust
rustup update stable

# Verify version
rustc --version
cargo --version

# If still old, reinstall
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

### Frontend Build Errors

#### Issue: "MODULE_NOT_FOUND" or "Cannot find module"

**Symptoms**:
```
Error: Cannot find module '@tauri-apps/api/tauri'
```

**Solution**:
```bash
# Clean install
rm -rf node_modules package-lock.json
npm install

# Verify installation
npm ls @tauri-apps/api
```

---

#### Issue: "TypeScript errors" during build

**Symptoms**:
```
error TS2304: Cannot find name 'SessionInfo'
```

**Solution**:
```bash
# Check TypeScript configuration
npm run typecheck

# Ensure all dependencies installed
npm install

# Check import paths
# Ensure you're using the correct import:
import type { SessionInfo } from '@/types/auth';
```

---

#### Issue: Vite build fails with "out of memory"

**Symptoms**:
```
FATAL ERROR: Reached heap limit Allocation failed
```

**Solution**:
```bash
# Increase Node.js memory limit
export NODE_OPTIONS="--max-old-space-size=4096"
npm run build

# Or in package.json scripts:
"build": "NODE_OPTIONS='--max-old-space-size=4096' vite build"
```

---

## Runtime Issues

### Application Won't Start

#### Issue: Tauri app window doesn't appear

**Symptoms**:
- App seems to build successfully
- No window appears
- Process runs but nothing visible

**Solutions**:

**1. Check console for errors**:
```bash
# Run with debug logging
RUST_LOG=debug npm run tauri dev
```

**2. Clear app data**:
```bash
# macOS
rm -rf ~/Library/Application\ Support/com.communitas.app

# Linux
rm -rf ~/.config/communitas

# Windows
# Delete: C:\Users\<username>\AppData\Roaming\communitas
```

**3. Check port conflicts**:
```bash
# Ensure port 1420 is available
lsof -i :1420
# Kill conflicting process if found
kill -9 <PID>
```

---

#### Issue: "Failed to initialize core context"

**Symptoms**:
```
Error: Failed to initialize core context: StorageError
```

**Solutions**:

**1. Check storage permissions**:
```bash
# Ensure data directory exists and is writable
mkdir -p .communitas-data
chmod 755 .communitas-data
```

**2. Check disk space**:
```bash
df -h .communitas-data
```

**3. Reset storage**:
```bash
# WARNING: Deletes all local data
rm -rf .communitas-data
```

---

### Authentication Issues

#### Issue: Login fails with "invalid credentials"

**Symptoms**:
```
Error: Authentication failed: invalid credentials
```

**Solutions**:

**1. Verify vault exists**:
```bash
# Check if vault was created
ls .communitas-data/vaults/
```

**2. Reset vault** (development only):
```bash
# Delete specific vault
rm -rf .communitas-data/vaults/<four-words>

# Create new vault through UI
```

**3. Check keyring access**:
```bash
# macOS - Verify Keychain Access permissions
# Linux - Check secret-service is running
systemctl --user status gnome-keyring-daemon

# Windows - Ensure Credential Manager is accessible
```

---

#### Issue: Passkey/Touch ID not working

**Symptoms**:
- Passkey registration fails
- Touch ID prompt doesn't appear

**Solutions**:

**macOS**:
```bash
# Verify Touch ID is enabled
bioutil -r  # Should show fingerprints

# Check security permissions in System Preferences
```

**All Platforms**:
```bash
# Ensure secure context (HTTPS or localhost)
# Check browser console for WebAuthn errors
```

---

## Testing Issues

### Tests Failing

#### Issue: "test result: FAILED" with no specific error

**Symptoms**:
```
test result: FAILED. 10 passed; 5 failed; 0 ignored
```

**Solutions**:

**1. Run with detailed output**:
```bash
cargo test -- --nocapture
```

**2. Run specific test**:
```bash
cargo test test_name -- --nocapture
```

**3. Enable debug logging**:
```bash
RUST_LOG=debug cargo test -- --nocapture
```

**4. Clean test artifacts**:
```bash
# Remove test data
rm -rf .communitas-data-test/

# Clean build
cargo clean
cargo test
```

---

#### Issue: Tests timeout

**Symptoms**:
```
test ... has been running for over 60 seconds
```

**Solutions**:

**1. Increase timeout**:
```bash
# In test file
#[tokio::test]
#[timeout(120000)]  // 120 seconds
async fn test_long_operation() {
    // Test code
}
```

**2. Check for deadlocks**:
```bash
# Run with logging to identify hanging operations
RUST_LOG=trace cargo test test_name -- --nocapture
```

---

#### Issue: Frontend tests fail with "TypeError: Cannot read property 'X' of undefined"

**Symptoms**:
```
TypeError: Cannot read property 'user' of undefined
```

**Solutions**:

**1. Mock dependencies properly**:
```typescript
import { vi } from 'vitest';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn()
}));
```

**2. Provide test providers**:
```typescript
import { render } from '@testing-library/react';
import { AuthProvider } from '@/contexts/AuthContext';

render(
  <AuthProvider>
    <MyComponent />
  </AuthProvider>
);
```

---

## Network Issues

### P2P Connection Failures

#### Issue: "Failed to connect to peer"

**Symptoms**:
```
Error: Failed to connect to peer: ConnectionTimeout
```

**Solutions**:

**1. Check network connectivity**:
```bash
# Test basic connectivity
ping 8.8.8.8

# Check if ports are open
nc -zv <peer-ip> 8080
```

**2. Verify firewall settings**:
```bash
# macOS
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

# Linux (ufw)
sudo ufw status

# Linux (firewalld)
sudo firewall-cmd --list-all
```

**3. Check NAT traversal**:
```bash
# Ensure QUIC can traverse NAT
# May need UPnP enabled on router
```

**4. Use bridge mode for testing**:
```bash
# Terminal 1: Start bridge
cargo run -p communitas-bridge

# Terminal 2: Test via HTTP
curl http://localhost:3030/health
```

---

#### Issue: Network indicator shows "offline" when online

**Symptoms**:
- Network indicator red or yellow
- App functional but shows wrong status

**Solutions**:

**1. Check network service**:
```typescript
// In browser console
window.testNetwork.status()
```

**2. Force reconnect**:
```typescript
// In browser console
window.testNetwork.connect()
```

**3. Check bootstrap nodes**:
```bash
# Verify bootstrap nodes are reachable
ping <bootstrap-node-ip>
```

---

## Platform-Specific Issues

### macOS

#### Issue: "App is damaged and can't be opened"

**Symptoms**:
- macOS refuses to open the app
- "Move to Trash" dialog appears

**Solutions**:

**1. Clear quarantine attribute**:
```bash
xattr -cr /Applications/Communitas.app
```

**2. Allow app in Security & Privacy**:
```
System Preferences > Security & Privacy > General
Click "Open Anyway" for Communitas
```

**3. Sign the app** (for distribution):
```bash
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name" \
  Communitas.app
```

---

#### Issue: Keychain access prompts repeatedly

**Symptoms**:
- Constant keychain permission prompts
- "Communitas wants to access key 'X' in your keychain"

**Solution**:
```
1. Open Keychain Access
2. Find "communitas" entries
3. Right-click > Get Info
4. Access Control tab
5. Select "Allow all applications to access this item"
```

---

### Linux

#### Issue: "error while loading shared libraries"

**Symptoms**:
```
error while loading shared libraries: libwebkit2gtk-4.1.so.0
```

**Solution**:
```bash
# Install missing dependencies
sudo apt install libwebkit2gtk-4.1-0

# Or reinstall all dependencies
sudo apt install --reinstall build-essential \
  libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev
```

---

#### Issue: Secret Service not available

**Symptoms**:
```
Error: Failed to access keyring: SecretServiceNotAvailable
```

**Solution**:
```bash
# Check if gnome-keyring is running
systemctl --user status gnome-keyring-daemon

# Start if not running
systemctl --user start gnome-keyring-daemon

# Enable auto-start
systemctl --user enable gnome-keyring-daemon
```

---

### Windows

#### Issue: "VCRUNTIME140.dll not found"

**Symptoms**:
- App fails to start
- Missing DLL error

**Solution**:
```
1. Download Microsoft Visual C++ Redistributable
   https://aka.ms/vs/17/release/vc_redist.x64.exe
2. Install the redistributable
3. Restart app
```

---

#### Issue: Antivirus blocking app

**Symptoms**:
- App starts then immediately closes
- No error messages

**Solution**:
```
1. Add Communitas to antivirus exceptions
2. For Windows Defender:
   Settings > Update & Security > Windows Security
   > Virus & threat protection > Manage settings
   > Exclusions > Add exclusion > Folder
   > Select Communitas installation folder
```

---

## Performance Issues

### High CPU Usage

#### Issue: App uses 100% CPU

**Symptoms**:
- Fan runs constantly
- System becomes sluggish
- Activity Monitor shows high CPU

**Solutions**:

**1. Check for infinite loops**:
```bash
# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --bin communitas-desktop
```

**2. Reduce sync frequency**:
```typescript
// Adjust sync interval in config
const SYNC_INTERVAL = 5000; // Increase from 1000ms
```

**3. Disable debug logging**:
```bash
# Don't run with RUST_LOG=trace in production
# Use RUST_LOG=info or no logging
```

---

### High Memory Usage

#### Issue: Memory usage grows over time

**Symptoms**:
- App starts at 200MB, grows to 2GB+
- Eventually crashes or freezes

**Solutions**:

**1. Check for memory leaks**:
```rust
// Use weak references for callbacks
use std::sync::Weak;

struct EventHandler {
    callback: Weak<dyn Fn()>,
}
```

**2. Clear caches periodically**:
```typescript
// In OfflineStorageService
await offlineStorage.clearCache();
```

**3. Limit cache size**:
```typescript
const MAX_CACHE_SIZE = 100 * 1024 * 1024; // 100 MB
if (cacheSize > MAX_CACHE_SIZE) {
  await offlineStorage.clearCache();
}
```

---

## Development Tools

### rust-analyzer Issues

#### Issue: "rust-analyzer failed to discover workspace"

**Symptoms**:
- No code completion
- No type hints
- Errors not showing

**Solutions**:

**1. Reload VS Code window**:
```
Cmd/Ctrl + Shift + P > Developer: Reload Window
```

**2. Clear rust-analyzer cache**:
```bash
rm -rf ~/.cache/rust-analyzer/
```

**3. Check Cargo.toml**:
```toml
# Ensure workspace is defined
[workspace]
members = [
  "communitas-core",
  "communitas-desktop",
  # ... other crates
]
```

---

### Git Issues

#### Issue: "fatal: not a git repository"

**Solution**:
```bash
# Reinitialize git
git init
git remote add origin https://github.com/saorsalabs/communitas.git
git fetch origin
git branch --set-upstream-to=origin/main main
```

---

#### Issue: Large files rejected by Git

**Symptoms**:
```
remote: error: File too large (> 100 MB)
```

**Solution**:
```bash
# Remove large file from history
git filter-branch --force --index-filter \
  'git rm --cached --ignore-unmatch large-file.bin' \
  --prune-empty --tag-name-filter cat -- --all

# Or use BFG Repo-Cleaner (faster)
bfg --delete-files large-file.bin
git reflog expire --expire=now --all
git gc --prune=now --aggressive
```

---

## Diagnostic Tools

### Logging

**Enable Different Log Levels**:
```bash
# Trace (most verbose)
RUST_LOG=trace npm run tauri dev

# Debug
RUST_LOG=debug npm run tauri dev

# Info (production)
RUST_LOG=info npm run tauri dev

# Specific module
RUST_LOG=communitas_core=trace npm run tauri dev

# Multiple modules
RUST_LOG=communitas_core=trace,communitas_desktop=debug npm run tauri dev
```

### Network Debugging

**Monitor Network Traffic**:
```bash
# macOS/Linux
sudo tcpdump -i any port 8080 -v

# Or use Wireshark for GUI
```

**Test Network Status**:
```typescript
// In browser console (with app running)
window.testNetwork = {
  status: () => networkService.getState(),
  connect: () => networkService.connect(),
  disconnect: () => networkService.disconnect(),
  simulateOffline: () => {
    networkService.disconnect();
    setTimeout(() => networkService.connect(), 5000);
  }
};

// Usage
window.testNetwork.status()
window.testNetwork.simulateOffline()
```

### Performance Profiling

**Rust Profiling**:
```bash
# Install tools
cargo install flamegraph
cargo install cargo-criterion

# Generate flamegraph
cargo flamegraph --bin communitas-desktop

# Run benchmarks
cargo bench
```

**Frontend Profiling**:
```typescript
// React DevTools Profiler
import { Profiler } from 'react';

<Profiler id="App" onRender={(id, phase, duration) => {
  console.log(`${id} ${phase}: ${duration}ms`);
}}>
  <App />
</Profiler>

// Performance API
const mark = performance.mark('operation-start');
// ... operation ...
performance.measure('operation', 'operation-start');
const measures = performance.getEntriesByType('measure');
console.log(measures[0].duration);
```

---

## Getting Help

### Before Asking for Help

1. **Search existing issues**: Your problem might already be solved
2. **Check documentation**: Review relevant guides
3. **Try diagnostic steps**: Use logging and debugging tools
4. **Create minimal reproduction**: Isolate the problem

### Creating a Good Bug Report

**Template**:
```markdown
## Environment
- OS: macOS 14.5 / Windows 11 / Ubuntu 22.04
- Rust version: `rustc --version`
- Node version: `node --version`
- App version: 0.3.17

## Description
Clear description of the problem

## Steps to Reproduce
1. Start app
2. Navigate to X
3. Click Y
4. Error occurs

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Logs
```
Paste relevant logs here
Use RUST_LOG=debug for detailed logs
```

## Screenshots
Attach screenshots if relevant

## Additional Context
Any other information that might be helpful
```

---

## See Also

- [Development Guide](README.md) - Complete development setup
- [Coding Standards](coding-standards.md) - Code quality guidelines
- [Contributing](contributing.md) - How to contribute
- [API Reference](../api/README.md) - API documentation

---

**Troubleshooting Guide**: Solve problems efficiently. 🔧🎯
