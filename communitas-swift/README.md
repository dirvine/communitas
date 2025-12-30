# Communitas Swift Client

Native iOS and macOS client for the [Communitas](https://github.com/saorsa-labs/communitas) collaboration platform.

## Status

**Phase 1-3 COMPLETE** | **Phase 4 IN PROGRESS**

Current capabilities:
- UniFFI bindings (Rust ↔ Swift) working
- Four-word identity creation and management
- Encrypted vault storage with password protection
- Passkey/Touch ID biometric authentication
- P2P networking via QUIC (saorsa-gossip)
- macOS app fully functional

## Overview
This project wraps the `communitas-core` Rust library using [UniFFI](https://github.com/mozilla/uniffi-rs) to provide a native SwiftUI experience on Apple platforms.

## Prerequisites
- Xcode 15+
- Rust 1.75+
- `cargo-swift` (optional, helpful for scaffolding) or manual UniFFI setup.

## Architecture
- **Core**: Shared logic in `../communitas-core`
- **Bindings**: Rust crate `communitas-bindings` (TBD) exposing the UniFFI interface.
- **App**: SwiftUI application consuming the bindings.

See [IMPLEMENTATION_SPEC.md](IMPLEMENTATION_SPEC.md) for the detailed roadmap.
