# Communitas API Documentation

This project exposes three primary API surfaces:

1. **Flutter FFI API** (preferred for native apps)
   - Defined in `communitas-core/src/flutter_api.rs`
   - Generated Dart bindings live in `communitas-flutter/lib/src/bindings`
   - Uses `flutter_rust_bridge`

2. **Core Rust API**
   - Command/query model in `communitas-core/src/command.rs`
   - Public types in `communitas-core/src/lib.rs`

3. **MCP API** (AI agent interface)
   - Server in `communitas-mcp/`
   - Tools map to core commands/queries

**Terminology**: Identity is the public key (pubkey_hex). Four-word networking is used only for
connection words (IP:port). Some APIs still use legacy field names like `fourWords` to carry the
identity value during migration.

## Flutter FFI Example

```dart
final api = await CommunitasApi.create(
  fourWords: 'pubkey_hex_goes_here',
  displayName: 'Alice',
  deviceName: 'Flutter-android',
  storagePath: '/path/to/storage',
);

await api.authCreateVault(
  fourWords: 'pubkey_hex_goes_here',
  displayName: 'Alice',
  password: 'strong-password',
);

final session = await api.authLogin(
  fourWords: 'pubkey_hex_goes_here',
  password: 'strong-password',
);

final entities = await api.entityList();
```

## Core Rust API
See [core-api.md](core-api.md) for core library interfaces and data types.

## MCP API
See `communitas-mcp/README.md` for tool definitions and usage examples.
