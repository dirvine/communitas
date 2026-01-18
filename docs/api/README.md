# Communitas API Documentation

This project exposes three primary API surfaces:

1. **UI Core API** (preferred for native apps)
   - Defined in `communitas-core/src/ui_core.rs`
   - Consumed directly by the Rust-based UI service (`communitas-ui-service`) and Dioxus front-end
   - Provides async helpers for auth, directory snapshots, messaging, files, etc.

2. **Core Rust API**
   - Command/query model in `communitas-core/src/command.rs`
   - Public types in `communitas-core/src/lib.rs`

3. **MCP API** (AI agent interface)
   - Server in `communitas-mcp/`
   - Tools map to core commands/queries

**Terminology**: Identity is the public key (pubkey_hex). Four-word networking is used only for
connection words (IP:port). Some APIs still use legacy field names like `fourWords` to carry the
identity value during migration.

## UI Core Example

```rust
use communitas_core::ui_core::CommunitasApi;

let api = CommunitasApi::create(
    four_words.to_string(),
    display_name.to_string(),
    format!("Communitas-{device_name}"),
    storage_path.to_string(),
)
.await?;

api.auth_create_vault(
    four_words.to_string(),
    display_name.to_string(),
    password.to_string(),
)
.await?;

let session = api
    .auth_login(four_words.to_string(), password.to_string())
    .await?;

let entities = api.entity_list().await?;
```

## Core Rust API
See [core-api.md](core-api.md) for core library interfaces and data types.

## MCP API
See `communitas-mcp/README.md` for tool definitions and usage examples.
