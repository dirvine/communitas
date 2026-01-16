# Communitas MCP Server

Model Context Protocol (MCP) server for AI agents and automation. Communicates via JSON-RPC 2.0 over stdio or HTTP/HTTPS.

## Features

- MCP tool surface for Communitas core actions
- JSON-RPC 2.0 over stdio (default)
- Optional HTTP transport (`/mcp`)
- Optional TLS with ML-DSA-65 raw public keys
- Demo mode for testing without real identities

## Usage

### Stdio (default)

```bash
cargo run -p communitas-mcp -- --demo
```

### HTTP

```bash
cargo run -p communitas-mcp -- --http --demo
```

### HTTPS (TLS)

```bash
cargo run -p communitas-mcp -- --http --tls --demo --no-client-auth
```

## CLI Flags

- `--demo` : Auto-initialize a temporary identity (skips auth). Use for dev only.
- `--storage-dir <path>` : Storage dir for demo mode.
- `--four-words <id>` : Use a specific four-word identity in demo mode.
- `--display-name <name>` : Display name for demo session.
- `--http` : Serve MCP over HTTP (`POST /mcp`).
- `--tls` : Enable HTTPS with ML-DSA-65 raw public keys (requires `--http`).
- `--listen <addr>` : Override listen address (default 127.0.0.1:8080 / 8443).
- `--no-client-auth` : Disable client cert verification (TLS only, dev only).

## Protocol

- Input: JSON-RPC 2.0 requests
- Output: JSON-RPC 2.0 responses
- Logging: stderr

## Readiness

See `docs/MCP_PRODUCTION_READINESS_REPORT.md` for current production readiness and gaps.
