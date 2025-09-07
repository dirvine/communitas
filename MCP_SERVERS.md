MCP Servers for Communitas

Overview
- `servers/mcp-puppeteer/server.js` provides a minimal MCP server to drive the Communitas UI in browser or Tauri dev mode using Puppeteer.
- Tools namespace:
  - browser_navigate(url)
  - browser_wait_for(selector, timeoutMs?)
  - browser_click(selector)
  - browser_click_text(text)
  - browser_type(selector, text)
  - browser_snapshot(fullPage?) → returns base64 PNG
  - browser_eval(script)
  - app_test_identity(), app_setup_workspace(), app_test_groups(), app_test_group_messaging(), app_offline_simulate(), app_list_groups()

Prereqs
- Node 20+ (Node 24 is OK)
- Communitas dev server running:
  - Browser UX mode: `npm run dev:browser` (http://localhost:1420)
  - Or Tauri dev UI exposed at the same URL during development

Install deps
```bash
npm install
```

Run the MCP server
```bash
# Optional: configure target URL and headless mode
export MCP_BROWSER_URL=http://localhost:1420
export MCP_BROWSER_HEADLESS=0

npm run mcp:puppeteer
```

Use with an MCP client
- Connect via stdio. Example (Claude Desktop/CLI) can load the server using a stdio command.
- The server name: `communitas-mcp-puppeteer`.

Notes
- The `app_*` tools call Communitas’ window test helpers exposed in the dev console (e.g., `window.tauriGroups.test()`), so ensure the app UI is loaded at the configured URL.
- For CI, set `MCP_BROWSER_HEADLESS=1`.

One‑shot helpers (avoid port collisions)

The scripts below kill any listeners on the required ports before starting.

- Start browser UI on 1420 and MCP HTTP on 8765, with logs:
  - `scripts/run-browser-and-mcp-http.sh`
  - Env overrides: `APP_PORT`, `MCP_PORT`, `MCP_HOST`, `MCP_BROWSER_URL`, `MCP_BROWSER_HEADLESS`

- Kill stray listeners explicitly:
  - `scripts/kill-ports.sh 1420 8765`

- Run the HTTP demo client (calls navigate, list groups, snapshot):
  - `scripts/mcp-http-demo.sh`
  - Env: `MCP_HTTP_URL` (default `http://127.0.0.1:8765`), `MCP_BROWSER_URL` (default `http://localhost:1420`)
