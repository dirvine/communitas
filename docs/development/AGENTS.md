# Communitas Agents Guide

_Last updated: 2025-09-15_

This playbook is for anyone (human or AI) automating Communitas. It captures the layout of the mono-repo, the critical flows that new agents must support, and the tooling expectations that keep our workflows green.

## 1. Workspace at a Glance
- `apps/communitas/` – React/Material UI console that now fronts the identity and storage surfaces. Uses the Tauri bindings defined in `communitas-desktop`.
- `communitas-desktop/` – Tauri v2 desktop crate. The only place we expose IPC commands (see `src/core_commands.rs`, `core_groups.rs`, `core_cmds.rs`, `container.rs`, `sync.rs`, `security/raw_spki.rs`).
 - `communitas-core/` – Shared Rust library. `CoreContext` wires saorsa-core (v0.3.23) managers together (including the exported `get_user_four_words` helpers), persists PQC identities to the platform keyring, and caches group signing keys.
- `communitas-headless/` – Headless QUIC node with self-update, bootstrap, and metrics support. Ideal for CI smoke checks and autonomous seeders.
- `crates/communitas-container/` – Pointer-only container/CRDT engine that produces signed tips and optional FEC metadata. Desktop and headless both depend on it.
- `src/` – Legacy React SPA still compiled for regression coverage. Tests under `src/**/__tests__` remain part of CI; do not delete until the migration completes.

## 2. Core Agent Flow
1. **Claim identity** – Call `core_claim(words: [String; 4])`. Keys are persisted in the keyring (`communitas-core::keystore`).
2. **Advertise presence** – `core_advertise(addr, storage_gb)` signs a presence heartbeat and returns optional Four-Word IPv4 endpoints for UI display.
3. **Initialize runtime** – `core_initialize` instantiates `CoreContext`, creating enhanced identities, chat/messaging services, and per-device storage handles.
4. **Messaging & channels** – Use `core_create_channel`, `core_send_message_to_channel`, `core_send_message_to_recipients`, `core_subscribe_messages`, and UI receives `message-received` events with decrypted payloads when possible. New in saorsa-core v0.3.23: `core_channel_list_members` and `core_resolve_channel_members` hydrate Four-Word handles directly from the address book so automations can map user IDs without guessing.
5. **Groups** – `core_group_create`, `core_group_add_member`, `core_group_remove_member` manage ML-DSA signed membership packets. Group signing keys are cached in-memory on the Tauri side.
6. **Container & virtual disk pointers** – `container_init`, `container_put_object`, `container_get_object`, `container_apply_ops`, and `container_current_tip` provide pointer-only storage. Use `core_private_put/get` for encrypted KV storage in the local store.
7. **Sync & repair** – `sync_start_tip_watcher` emits `container-tip` events; `sync_fetch_deltas` pulls CRDT ops over raw-key-pinned QUIC; `sync_repair_fec` exposes Reed–Solomon recovery helpers. Pin raw SPKI values via `sync_set_quic_pinned_spki`.
8. **Bootstrap maintenance** – `core_get_bootstrap_nodes` / `core_update_bootstrap_nodes` read/write `bootstrap.toml` so automations can configure seeds.

## 3. Storage & Container Notes
- Container engine lives in `crates/communitas-container`. It encrypts payloads with AEAD (default on) and can emit FEC shards (k=4, m=2) for higher-layer distribution.
- Desktop persists opaque blobs to `COMMUNITAS_DATA_DIR` (defaults to `src-tauri/.communitas-data`) so offline reads never block.
- Pointer-only DHT policy: the app never writes large blobs directly to the DHT. Publish signed tips or manifests and store payloads locally or via delegated providers.

## 4. Messaging, Channels, and Events
- `message-received` (App side) delivers decrypted payloads when `MessagingService::decrypt_message` succeeds; otherwise payloads are tagged `encrypted: true`.
- `channel-member-resolved` events fire when `core_resolve_channel_members` iterates channel membership and resolves human metadata. Payload now includes both `four_words` (array of words) and `four_words_text` (hyphenated string) sourced from `saorsa_core::get_user_four_words`.
- Channel helpers: `core_channel_list_members`, `core_channel_invite_by_words` (currently returns an error until saorsa-core exposes membership writes), and `core_channel_recipients` (placeholder for UI fallbacks). `core_send_message_to_channel` now looks up Four-Word addresses via the saorsa-core address book before falling back to heuristics.

## 5. Sync, Security & Networking
- `sync_progress` events provide `{ phase, peer, ops?, new_count?, root? }` updates during QUIC delta fetches.
- Raw SPKI pinning flows: prefer `sync_set_quic_pinned_spki`/`sync_clear_quic_pinned_spki`, or set `COMMUNITAS_QUIC_PINNED_SPKI`/`COMMUNITAS_RPK_ALLOW_ANY` in dev.
- QUIC/IPv4 first: `sync_fetch_deltas` resolves addresses via `lookup_host`, ordering IPv4 before IPv6.
- Headless binary exposes the same container + sync stack via CLI. Config file controls FEC, bootstrap, update cadence, and metrics (`127.0.0.1:9600`).

## 6. Tooling & Workflows
- **Rust**: `cargo fmt --all`, `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`, `cargo test -p communitas-desktop`, `cargo test -p communitas-core`, `cargo test -p communitas-headless`.
- **Node/React**: `npm ci`, `npm run typecheck`, `npm run test:run` (fast Vitest slice), `npm run build`.
- **Desktop builds**: `cargo build --release -p communitas-desktop`, `npm run tauri build` for signed bundles (requires TAURI_PRIVATE_KEY in CI).
- **Headless smoke**: `cargo build --release -p communitas-headless` then `./target/release/communitas-headless --help`.
- GitHub workflows in `.github/workflows/` assume Node 20 and Rust stable; keep scripts aligned when changing tooling.

## 7. Observability & Logs
- Tracing uses `tracing_subscriber` with `RUST_LOG=info,communitas=debug,saorsa_core=debug` by default. Override per workflow.
- Container watchers emit `container-tip`; sync flows emit `sync-progress`; UI network diagnostics remain under `window.testNetwork.*` in legacy `src/` tests.
- Metrics: headless node exposes Prometheus-like endpoint when `--metrics` flag is used.

## 8. MCP (Model Context Protocol) Automation

### MCP Server Overview
When running `npm run tauri dev`, a Unix socket server starts at `/tmp/tauri-mcp-communitas-<pid>.sock` enabling programmatic UI control.

### Chrome DevTools MCP Browser Automation
- **Why**: Provides a standards-based MCP surface on top of headless Chrome so agents can exercise the browser build (`npm run dev:browser`) without relying on the Tauri webview bridge.
- **Server launch**: `npx chrome-devtools-mcp --headless --isolated` (the CLI spawns its own Chrome profile and speaks JSON-RPC 2.0). Pass `--browserUrl http://127.0.0.1:1420` to attach to an existing Vite session instead of launching a fresh browser.
- **Handshake**: Send an `initialize` request with `protocolVersion: "0.1.0"` and `clientInfo` populated. A healthy server replies with `protocolVersion: "2025-06-18"`, advertises 26 tools (navigation, DOM snapshots, console/network inspectors), and remains ready for `tools/call` RPCs.
- **Signin/logout recipe**:
  1. Ensure Communitas browser mode is running (`npm run dev:browser`).
  2. Launch the MCP server and record its stdio stream (or wire it into your MCP client).
  3. `tools/call` → `navigate_page` with `{"url": "http://localhost:1420"}` to load the app shell.
  4. `tools/call` → `take_snapshot` to capture UIDs, then `fill` / `fill_form` against the signin form fields, followed by `click` on the submit UID.
  5. Verify success via `evaluate_script` (e.g., `() => !!window.__COMMUNITAS_USER__`).
  6. Call `evaluate_script` again with `() => window.__TAURI__?.invoke?.('core_logout')` or drive the UI logout button via another `click`.
  7. Capture a post-logout snapshot and console logs (`list_console_messages`) to confirm state reset.
- **Teardown**: The server lacks a `shutdown` RPC; terminate the process (`Ctrl+C` or PID kill) once snapshots and logs are saved.

### Automated Testing with MCP

#### User Registration Flow
```javascript
// MCP-based user registration test
async function testUserRegistration(mcpClient) {
  // 1. Navigate to registration
  await mcpClient.call('execute_js', {
    script: `window.location.href = '#/register'`
  });

  // 2. Generate Four-Word identity
  await mcpClient.call('execute_js', {
    script: `document.querySelector('#generate-words').click()`
  });

  // 3. Wait for words to generate
  await sleep(500);

  // 4. Capture generated words
  const words = await mcpClient.call('execute_js', {
    script: `document.querySelector('#four-words-display').textContent`
  });

  // 5. Enter display name
  await mcpClient.call('send_text_to_element', {
    selector: '#display-name',
    text: 'Test User'
  });

  // 6. Enter device name
  await mcpClient.call('send_text_to_element', {
    selector: '#device-name',
    text: 'Test Device'
  });

  // 7. Click register
  await mcpClient.call('execute_js', {
    script: `document.querySelector('#register-button').click()`
  });

  // 8. Verify success
  await sleep(1000);
  const profile = await mcpClient.call('get_dom', {
    selector: '.user-profile'
  });

  return { words, profile };
}
```

#### User Login Flow
```javascript
// MCP-based login test
async function testUserLogin(mcpClient, fourWords) {
  // 1. Navigate to login
  await mcpClient.call('execute_js', {
    script: `window.location.href = '#/login'`
  });

  // 2. Enter Four-Words
  await mcpClient.call('send_text_to_element', {
    selector: '#four-words-input',
    text: fourWords,
    clear_first: true
  });

  // 3. Click login
  await mcpClient.call('execute_js', {
    script: `document.querySelector('#login-button').click()`
  });

  // 4. Wait for auth
  await sleep(1000);

  // 5. Verify logged in
  const isLoggedIn = await mcpClient.call('execute_js', {
    script: `!!window.__COMMUNITAS_USER__`
  });

  // 6. Check network connection
  const networkStatus = await mcpClient.call('execute_js', {
    script: `window.testNetwork?.status() || 'unknown'`
  });

  return { isLoggedIn, networkStatus };
}
```

### Complete App Test Plan

#### Phase 1: Identity & Auth
```javascript
// Test identity lifecycle
async function testIdentityLifecycle(mcp) {
  // Registration
  const { words } = await testUserRegistration(mcp);

  // Logout
  await mcp.call('execute_js', {
    script: `await window.__TAURI__.invoke('core_logout')`
  });

  // Login with saved words
  await testUserLogin(mcp, words);

  // Verify keyring persistence
  const keystored = await mcp.call('execute_js', {
    script: `await window.__TAURI__.invoke('core_keyring_status')`
  });

  assert(keystored.saved === true);
}
```

#### Phase 2: Messaging & Channels
```javascript
// Test messaging capabilities
async function testMessaging(mcp) {
  // Create channel
  await mcp.call('execute_js', {
    script: `
      await window.__TAURI__.invoke('core_create_channel', {
        name: 'test-channel',
        description: 'MCP test channel'
      })
    `
  });

  // Send message
  await mcp.call('send_text_to_element', {
    selector: '#message-input',
    text: 'Hello from MCP test!'
  });

  await mcp.call('execute_js', {
    script: `document.querySelector('#send-button').click()`
  });

  // Verify message appears
  await sleep(500);
  const messages = await mcp.call('get_dom', {
    selector: '.message-list'
  });

  assert(messages.includes('Hello from MCP test!'));
}
```

#### Phase 3: Storage & Virtual Disks
```javascript
// Test storage operations
async function testStorage(mcp) {
  // Write to private disk
  await mcp.call('execute_js', {
    script: `
      const content = btoa('Test content');
      await window.__TAURI__.invoke('core_disk_write', {
        entityHex: window.__COMMUNITAS_USER__.id,
        diskType: 'Private',
        path: '/test.txt',
        contentBase64: content
      })
    `
  });

  // Read from disk
  const data = await mcp.call('execute_js', {
    script: `
      await window.__TAURI__.invoke('core_disk_read', {
        entityHex: window.__COMMUNITAS_USER__.id,
        diskType: 'Private',
        path: '/test.txt'
      })
    `
  });

  assert(atob(data) === 'Test content');
}
```

#### Phase 4: Network & P2P
```javascript
// Test network connectivity
async function testNetwork(mcp) {
  // Check initial status
  const status = await mcp.call('execute_js', {
    script: `window.testNetwork.status()`
  });

  // Simulate offline
  await mcp.call('execute_js', {
    script: `window.testNetwork.simulateOffline()`
  });

  // Verify offline mode
  const offline = await mcp.call('get_dom', {
    selector: '.network-indicator.offline'
  });
  assert(offline !== null);

  // Reconnect
  await mcp.call('execute_js', {
    script: `window.testNetwork.connect()`
  });

  // Verify connected
  await sleep(2000);
  const connected = await mcp.call('get_dom', {
    selector: '.network-indicator.connected'
  });
  assert(connected !== null);
}
```

### MCP Client Implementation
```javascript
// Simple MCP client for testing
class MCPClient {
  constructor(socketPath) {
    this.socket = net.createConnection(socketPath);
    this.id = 0;
    this.pending = new Map();

    this.socket.on('data', (data) => {
      const response = JSON.parse(data.toString());
      const promise = this.pending.get(response.id);
      if (promise) {
        promise.resolve(response.result);
        this.pending.delete(response.id);
      }
    });
  }

  async call(method, params = {}) {
    const id = ++this.id;
    const request = {
      jsonrpc: '2.0',
      method,
      params,
      id
    };

    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.socket.write(JSON.stringify(request));

      // Timeout after 5 seconds
      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error('MCP timeout'));
        }
      }, 5000);
    });
  }

  close() {
    this.socket.end();
  }
}
```

### Running MCP Tests
```bash
# 1. Start Tauri in dev mode
npm run tauri dev

# 2. Find MCP socket
export MCP_SOCKET=$(ls /tmp/tauri-mcp-communitas-*.sock | head -1)

# 3. Run test suite
node test-mcp-suite.js $MCP_SOCKET

# 4. Generate test report
npm run test:mcp:report
```

### Visual Regression Testing
```javascript
// Screenshot comparison for UI changes
async function visualRegression(mcp) {
  const screens = [];

  // Capture each major view
  for (const route of ['#/login', '#/chat', '#/files', '#/settings']) {
    await mcp.call('execute_js', {
      script: `window.location.href = '${route}'`
    });
    await sleep(500);

    const screenshot = await mcp.call('take_screenshot', {
      format: 'png'
    });

    screens.push({
      route,
      image: screenshot,
      timestamp: Date.now()
    });
  }

  // Compare with baseline
  return compareWithBaseline(screens);
}
```

## 9. Reference Library
- Low-level API details: `AGENTS_API.md` (same directory).
- saorsa-core references: see `communitas-core/src/` and `COMMUNITAS_ARCHITECTURE.md`.
- Deployment & bootstrap specifics: `finalise/` docs, `bootstrap.toml.template`, `deployment/` scripts.
- MCP automation examples: `MCP_SERVERS.md`, `servers/mcp-puppeteer`.
- MCP protocol spec: See `CLAUDE.md` for complete tool documentation.

Keep this file updated when:
- saorsa-core is bumped,
- new Tauri commands are surfaced,
- container/FEC defaults change,
- workspace layout shifts (e.g., once `apps/communitas` fully replaces `src/`),
- MCP tools are added or modified.
