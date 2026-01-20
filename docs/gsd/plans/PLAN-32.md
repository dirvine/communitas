# PLAN-32: Phase 5.2 — WebView Provisioning

**Phase**: 5.2 (WebView Provisioning)
**Status**: Ready
**Depends on**: None (can start immediately)

## Overview

Implement WebView dependency detection and graceful error handling before the Dioxus app launches. ADR-020 commits to this but it's not yet implemented.

## Context

The app currently calls `dioxus::launch(App)` without checking if the platform's WebView runtime is available:
- **macOS**: Requires WebKit (bundled with Safari, always present)
- **Linux**: Requires WebKitGTK (`libwebkit2gtk-4.1-0`)
- **Windows**: Requires WebView2 (Edge runtime, may need installation)

Without checks, the app crashes with unhelpful errors on systems missing dependencies.

---

## Tasks

<task type="auto" priority="p1">
  <n>Add platform detection module</n>
  <files>
    communitas-dioxus/src/platform.rs,
    communitas-dioxus/src/main.rs
  </files>
  <action>
    1. Create `platform.rs` module with WebView detection functions
    2. Add `#[cfg(target_os = "...")]` blocks for each platform
    3. macOS: Always return Ok (WebKit bundled)
    4. Linux: Check for libwebkit2gtk via pkg-config or library probe
    5. Windows: Check WebView2 registry key or file existence
    6. Export `check_webview_available() -> Result<(), String>`
  </action>
  <verify>
    cargo build -p communitas-dioxus --target x86_64-apple-darwin
    cargo clippy -p communitas-dioxus -- -D warnings
  </verify>
  <done>
    - platform.rs exists with check_webview_available()
    - Function compiles for all target platforms
    - Returns Ok on macOS unconditionally
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add startup dependency check</n>
  <files>
    communitas-dioxus/src/main.rs
  </files>
  <action>
    1. Call `platform::check_webview_available()` before `dioxus::launch()`
    2. On failure, show native error dialog (not Dioxus - it won't work!)
    3. Use `native-dialog` crate for cross-platform dialogs
    4. Include helpful message with install instructions
    5. Exit gracefully with clear error code
  </action>
  <verify>
    cargo build -p communitas-dioxus
    cargo clippy -p communitas-dioxus -- -D warnings
  </verify>
  <done>
    - main.rs checks WebView before launch
    - Error dialog shown if missing
    - Install instructions included in message
  </done>
</task>

<task type="auto" priority="p2">
  <n>Add WebView installer scripts</n>
  <files>
    scripts/install-webview-windows.ps1,
    scripts/install-webview-linux.sh
  </files>
  <action>
    1. Create PowerShell script for Windows WebView2 bootstrap
    2. Create shell script for Linux WebKitGTK installation
    3. Detect package manager (apt, dnf, pacman) on Linux
    4. Download WebView2 bootstrapper on Windows
    5. Add to repository with executable permissions
  </action>
  <verify>
    shellcheck scripts/install-webview-linux.sh
  </verify>
  <done>
    - Scripts exist and are executable
    - Linux script handles apt/dnf/pacman
    - Windows script downloads WebView2 bootstrapper
  </done>
</task>

<task type="auto" priority="p2">
  <n>Add unit tests for platform detection</n>
  <files>
    communitas-dioxus/src/platform.rs
  </files>
  <action>
    1. Add #[cfg(test)] module to platform.rs
    2. Test that check_webview_available() returns Ok on macOS
    3. Mock file/registry checks for other platforms
    4. Test error message formatting
  </action>
  <verify>
    cargo test -p communitas-dioxus platform
  </verify>
  <done>
    - Tests pass on CI
    - Platform detection logic covered
  </done>
</task>

<task type="auto" priority="p3">
  <n>Update documentation</n>
  <files>
    docs/development/prerequisites.md,
    CLAUDE.md
  </files>
  <action>
    1. Document WebView requirements per platform
    2. Add troubleshooting section for WebView issues
    3. Reference installer scripts
    4. Update CLAUDE.md troubleshooting section
  </action>
  <verify>
    N/A (documentation only)
  </verify>
  <done>
    - Prerequisites documented
    - Installer scripts referenced
    - Troubleshooting guidance added
  </done>
</task>

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `native-dialog` | Cross-platform error dialogs (no WebView needed) |
| `pkg-config` | Linux library detection (optional, build-time) |

---

## Acceptance Criteria

1. App shows helpful error dialog when WebView missing (not crash)
2. Error message includes platform-specific install instructions
3. Installer scripts work on Windows/Linux
4. Documentation updated with prerequisites
