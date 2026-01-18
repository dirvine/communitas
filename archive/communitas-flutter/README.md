# Communitas Flutter App

Flutter UI for Communitas, backed by the Rust core via `flutter_rust_bridge`.

Supported targets: iOS, Android, Linux, Windows, Web (limited). The macOS runner is intentionally removed for now.

## Prerequisites

- Flutter 3.27+
- Rust 1.85+

## Quick Start (Native Platforms)

Native platforms use the Rust core through `flutter_rust_bridge`. Build the native library first:

```bash
# Build Rust library (from workspace root)
cargo build --release -p communitas-bindings

# Run Flutter app
cd communitas-flutter
flutter pub get
flutter run -d android  # or: -d ios, -d linux, -d windows
```

For platform-specific library placement, see `docs/adr/ADR-017-flutter-rust-ffi-integration.md`.

## Web

Web builds run with limited functionality (FFI is not available in the browser).
API calls return stub data and the app is view-only:

```bash
cd communitas-flutter
flutter pub get
flutter run -d chrome
```
