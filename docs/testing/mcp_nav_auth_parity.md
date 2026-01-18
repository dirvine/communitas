# MCP ↔️ UiServices Parity Harness

Milestone 1 requires that the navigation/auth flows stay aligned between MCP automation and the Rust UI services. This harness spins up `communitas-mcp` in demo mode, captures its `list_entities` output, and compares it to a direct snapshot produced via the shared core APIs.

## Script overview

- `scripts/tests/mcp_nav_auth.sh` orchestrates the check:
  1. Builds and launches `communitas-mcp --demo --http` with a known four-word identity and storage directory.
  2. Calls `initialize` and `tools/call:list_entities` over HTTP to capture the MCP view.
  3. Runs `cargo run -p communitas-core --bin export_directory` against the same storage path to dump the canonical directory snapshot.
  4. Uses `jq` to canonicalize both JSON payloads and fails if the entity arrays differ.
- The helper binary (`communitas-core/src/bin/export_directory.rs`) reuses `CommunitasApp` to fetch profile, entities, and contacts, mirroring what `UiServices::directory()` would expose to Dioxus. This keeps the comparison squarely focused on shared Rust logic.

## Requirements

- `curl` + `jq`
- `cargo` (builds `communitas-mcp` + the export helper)

## Running locally

```bash
scripts/tests/mcp_nav_auth.sh
```

Optional env vars:

| Variable | Default | Purpose |
| --- | --- | --- |
| `MCP_HTTP_PORT` | `7331` | HTTP listen port for `communitas-mcp` |
| `MCP_DEMO_FOUR_WORDS` | `demo-parity-harness-node` | Demo identity |
| `MCP_DEMO_DISPLAY` | `Parity Harness` | Display name used by both MCP + snapshot |

On success, the script prints `entity list parity verified`. Any divergence shows a diff of the normalized arrays so we can track regressions quickly.

## Next steps

- Extend the helper to include contacts/presence snapshots once we add corresponding MCP tools.
- Wire this script into CI (Linux runner) and archive both JSON payloads as artifacts for traceability.
