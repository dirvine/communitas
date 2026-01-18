# Milestone 1 — Tauri WebDriver Smoke Test

The nav/auth milestone now includes a **tauri-driver** smoke test so CI can programmatically validate that the login shell renders before we move on to heavier feature work.

## Why tauri-driver?

- [tauri-driver](https://github.com/tauri-apps/tauri-driver) is the upstream WebDriver intermediary for Wry/Tauri apps; it spins up the platform-native driver (WebKitWebDriver on Linux, EdgeDriver on Windows) and proxies the session to WebDriver clients.citeturn1search1
- Tauri’s example suite demonstrates how to pair tauri-driver with WebdriverIO; we follow the same pattern here with a single spec that waits for the Communitas login view.citeturn1search3

⚠️ **Platform note**: tauri-driver currently supports Linux and Windows. macOS support is still marked “TODO” upstream, so our script bails out on Darwin runners and the CI job must target Linux until support lands.citeturn1search1

## Repository layout

```
scripts/tests/m1_nav_auth.tauri.sh    # entrypoint for CI
tests/webdriverio/package.json        # minimal WebdriverIO workspace
tests/webdriverio/wdio.conf.js        # spawns tauri-driver + configures WebDriver session
tests/webdriverio/specs/nav-auth.smoke.js
```

## Running locally

1. **Install prerequisites**
   - `cargo install tauri-driver --locked`
   - WebKitWebDriver (Linux) or EdgeDriver (Windows) available on `PATH`
   - Node 18+ (for WebdriverIO)
2. **Build the Communitas Dioxus client**
   ```bash
   cargo build -p communitas-dioxus
   ```
3. **Execute the smoke test**
   ```bash
   scripts/tests/m1_nav_auth.tauri.sh
   ```
   - The script auto-installs WebdriverIO deps on first run.
   - Override ports/binaries via `TAURI_DRIVER_PORT`, `TAURI_DRIVER_NATIVE_PORT`, `APP_BINARY`, or `TAURI_DRIVER_BIN` env vars.

## CI wiring

- Add a Linux GitHub Actions job that:
  1. Installs WebKitWebDriver (Ubuntu: `sudo apt install webkit2gtk-driver`).
  2. Installs Node + `tauri-driver`.
  3. Runs `scripts/tests/m1_nav_auth.tauri.sh`.
- Archive the WebdriverIO reports (`tests/webdriverio/.wdio-temp`) as build artifacts for debugging.

## What the smoke test covers

- Launches `communitas-dioxus` via tauri-driver and waits for the login route.
- Asserts the “Welcome back” heading and “Sign in” button are present—exercising the nav shell, router bootstrap, and auth layout hooks in one pass.
- Serves as a gating signal before we let agents or humans proceed to deeper flows.

Extend this harness with additional specs (e.g., sidebar toggling, demo-mode splash) as soon as we wire real test data into UiServices. The scaffolding here keeps that iteration loop fast while we continue tracking Blitz renderer feasibility separately.
