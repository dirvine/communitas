# ADR-017: Flutter-Rust FFI Integration

## Status

Accepted (2025-01-15)

## Context

### The Problem

The Flutter application initially used an HTTP bridge (`bridge_client.dart` and `bridge_provider.dart`) to communicate with the Rust core. This approach had several limitations:

1. **Performance overhead**: HTTP serialization/deserialization for every call
2. **Deployment complexity**: Required separate Rust bridge server process
3. **Web-centric design**: Built around a pattern that primarily benefits web builds
4. **Authentication duplication**: Separate auth for bridge vs. app identity
5. **No web support anyway**: We don't target web builds (web is demo-only), making HTTP unnecessary

### Requirements

- Direct, efficient communication between Flutter and Rust
- Single binary deployment (no separate server process)
- Type-safe bindings with compile-time verification
- Native-only targets (macOS, iOS, Android, Windows, Linux)
- Keep MCP available for external AI agent integration

### Existing Infrastructure

The codebase already had:
- `flutter_rust_bridge` configured in `communitas-core`
- `CommunitasApi` FFI class generated and working
- Authentication flow creating `CommunitasApi` instance on login

## Decision

Replace the HTTP bridge layer with direct FFI calls via `flutter_rust_bridge`:

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Flutter Application                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                    Riverpod Providers                               │ │
│  │                                                                      │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │ │
│  │  │ unified_data_   │  │ presence_       │  │ ffi_provider.dart   │ │ │
│  │  │ provider.dart   │  │ provider.dart   │  │ (core FFI access)   │ │ │
│  │  └────────┬────────┘  └────────┬────────┘  └──────────┬──────────┘ │ │
│  │           │                    │                       │            │ │
│  │           └────────────────────┴───────────────────────┘            │ │
│  │                                │                                     │ │
│  └────────────────────────────────┼─────────────────────────────────────┘ │
│                                   │                                       │
│                                   ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                     flutter_rust_bridge                              │ │
│  │                   (generated Dart bindings)                          │ │
│  │                                                                       │ │
│  │  CommunitasApi → entityList(), gossipGetNetworkInfo(), messageSend() │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                   │                                       │
└───────────────────────────────────┼───────────────────────────────────────┘
                                    │ FFI
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         communitas-core (Rust)                          │
│                                                                          │
│  CommunitasApp → CoreContext → EntityService, MessageService, etc.      │
└─────────────────────────────────────────────────────────────────────────┘
```

### Provider Hierarchy

```dart
// ffi_provider.dart - Core FFI access
final communitasApiProvider = Provider<CommunitasApi?>((ref) {
  final auth = ref.watch(authNotifierProvider);
  return auth.api;  // CommunitasApi from successful login
});

// Entity providers use FFI directly
final ffiOrganizationsProvider = FutureProvider<List<FlutterEntity>>((ref) async {
  return ref.watch(ffiEntitiesByTypeProvider(FlutterEntityType.organisation).future);
});

// unified_data_provider.dart wraps FFI with unified types
final unifiedOrganizationsProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  if (kDemoMode) {
    return DemoData.organizations.map((e) => UnifiedEntity.fromDemo(e)).toList();
  }
  final orgs = await ref.watch(ffiOrganizationsProvider.future);
  return orgs.map((e) => UnifiedEntity.fromFfi(e)).toList();
});
```

### Authentication Flow

```
User enters passphrase
         │
         ▼
AuthNotifier.login(passphrase)
         │
         ▼
CommunitasApi.authLogin(fourWords, passphrase)
         │
         ▼
AuthState(api: CommunitasApi, ...)
         │
         ▼
communitasApiProvider returns CommunitasApi
         │
         ▼
All FFI providers can now call api methods
```

### Dual Communication Channels

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Communication Architecture                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────────────────┐     ┌──────────────────────────────┐ │
│  │      Flutter GUI             │     │   External AI Agents          │ │
│  │                              │     │   (Claude, saorsa-canvas)     │ │
│  │  Uses: FFI via CommunitasApi │     │  Use: MCP over HTTP/stdio     │ │
│  └──────────────┬───────────────┘     └──────────────┬───────────────┘ │
│                 │                                     │                  │
│                 │ flutter_rust_bridge                 │ JSON-RPC 2.0    │
│                 │                                     │                  │
│                 ▼                                     ▼                  │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                       communitas-core                                ││
│  │                                                                       ││
│  │   ┌─────────────────┐                 ┌─────────────────┐           ││
│  │   │ Flutter API     │                 │  MCP Server     │           ││
│  │   │ (FFI bindings)  │                 │ (embedded/HTTP) │           ││
│  │   └────────┬────────┘                 └────────┬────────┘           ││
│  │            │                                   │                     ││
│  │            └───────────────┬───────────────────┘                     ││
│  │                            ▼                                         ││
│  │                    CommunitasApp / CoreContext                       ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Files Changed

| File | Change |
|------|--------|
| `lib/src/services/ffi_provider.dart` | **Created** - Central FFI providers |
| `lib/src/services/unified_data_provider.dart` | Updated to use FFI |
| `lib/src/features/network/providers/presence_provider.dart` | Updated to use FFI |
| `lib/src/features/messaging/presentation/chat_screen.dart` | Updated identity reference |
| `lib/src/services/bridge_client.dart` | **Removed** |
| `lib/src/services/bridge_provider.dart` | **Removed** |

## Consequences

### Benefits

1. **Performance**: Direct FFI calls, no HTTP overhead
2. **Simplicity**: Single binary, no separate server process
3. **Type safety**: Compile-time verification of Rust↔Dart interface
4. **Native focus**: Optimized for our actual target platforms
5. **Unified auth**: Single authentication flow through `CommunitasApi`

### Trade-offs

1. **No web builds**: FFI doesn't work in browser (intentional - we don't target web)
2. **Code generation**: Must run `flutter_rust_bridge_codegen` on Rust API changes
3. **Platform-specific builds**: Each platform needs native library compiled

### Migration Path

The HTTP bridge has been removed. All runtime communication is now via FFI.

```dart
// New (FFI-first)
final orgs = await ref.watch(ffiOrganizationsProvider.future);
// Or via unified provider:
final orgs = await ref.watch(unifiedOrganizationsProvider.future);
```

### Demo Mode

Demo mode (`kDemoMode`) still uses `DemoData` for UI development without Rust core:

```dart
final unifiedOrganizationsProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  if (kDemoMode) {
    return DemoData.organizations.map((e) => UnifiedEntity.fromDemo(e)).toList();
  }
  // Real FFI path
  final orgs = await ref.watch(ffiOrganizationsProvider.future);
  return orgs.map((e) => UnifiedEntity.fromFfi(e)).toList();
});
```

## Alternatives Considered

1. **Keep HTTP bridge**: Continue with HTTP for all communication
   - Rejected: Unnecessary complexity for native-only apps

2. **WebSocket bridge**: Replace HTTP with WebSocket
   - Rejected: Still requires separate process, FFI is more direct

3. **Platform channels only**: Use raw Flutter platform channels
   - Rejected: `flutter_rust_bridge` provides better ergonomics and type safety

4. **gRPC**: Use gRPC for communication
   - Rejected: Adds protocol complexity, still needs separate process

## References

- flutter_rust_bridge: https://cjycode.com/flutter_rust_bridge/
- Previous: `lib/src/services/bridge_provider.dart` (removed)
- FFI bindings: `lib/src/bindings/flutter_api.dart`
- Auth provider: `lib/src/features/auth/providers/auth_provider.dart`
- See also: [ADR-018](ADR-018-mcp-external-integration.md) for MCP access by external apps
