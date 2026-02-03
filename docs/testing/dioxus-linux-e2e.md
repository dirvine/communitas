## Dioxus GUI E2E on Linux

The WebDriverIO suite under `tests/webdriverio/` currently fails on macOS
because `tauri-driver` does not expose a usable backend there. To unblock the
desktop automation flows we can run the exact same suite on a Linux host (bare
metal, VM, or container) where Tauri’s driver is supported.

### 1. System requirements

- Ubuntu 22.04+ (or Debian/Fedora with equivalent packages)
- `webkit2gtk`/`webkit2gtk-driver` for the embedded WebKit runtime
- X11 display (use `xvfb-run` for headless CI)
- Rust toolchain + `cargo-zigbuild` (for cross-building the Dioxus binary)
- Node.js 20+

Install the runtime prerequisites:

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev build-essential pkg-config unzip xvfb

cargo install tauri-driver
```

### 2. Build the desktop app

```bash
cd communitas-dioxus
cargo zigbuild --target x86_64-unknown-linux-gnu
```

`scripts/run-e2e-tests.sh` will automatically pick up the built binary from
`target/x86_64-unknown-linux-gnu/debug/communitas-dioxus` if the standard
`target/debug` build is missing.

### 3. Run the WebDriverIO specs

```bash
cd tests/webdriverio
npm install

# Local display
TAURI_APP_BINARY=../target/x86_64-unknown-linux-gnu/debug/communitas-dioxus \
  TAURI_DRIVER_BIN=~/.cargo/bin/tauri-driver \
  npx wdio run wdio.conf.js --spec specs/full-e2e.spec.js

# Headless CI
xvfb-run -a \
  TAURI_APP_BINARY=../target/x86_64-unknown-linux-gnu/debug/communitas-dioxus \
  TAURI_DRIVER_BIN=~/.cargo/bin/tauri-driver \
  npx wdio run wdio.conf.js --spec specs/full-e2e.spec.js
```

### 4. Troubleshooting

- “Unable to connect to http://127.0.0.1:4444/” → ensure `tauri-driver` is in
  `$PATH` and not blocked by firewalls. The WDIO config spawns it automatically.
- GTK/WebKit errors on startup → verify the `libwebkit2gtk-4.1-dev` runtime is
  installed.
- GPU/Wayland issues inside headless CI → keep using `xvfb-run` or switch to a
  nested virtual display (e.g., `weston --xwayland`).

Once this workflow is green we can wire the same steps into CI (GitHub Actions
or the internal runner) and unblock full E2E coverage for Dioxus.
