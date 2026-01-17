# Communitas Flutter App

Flutter UI for Communitas, backed by the Rust core via `flutter_rust_bridge`.

Supported targets: iOS, Android, Linux, Windows, Web (demo). The macOS runner is intentionally removed for now.

## Prerequisites

- Flutter 3.27+
- Rust 1.85+

## Quick Start (Demo Mode)

Demo mode runs without the native Rust library (useful for UI work):

```bash
cd communitas-flutter
flutter pub get
flutter run -d android --dart-define=DEMO_MODE=true  # or: -d ios, -d linux, -d windows
```

## Native FFI Mode

Native mode uses the Rust core through `flutter_rust_bridge`. Ensure the native library is built
and discoverable by the app (see `docs/adr/ADR-017-flutter-rust-ffi-integration.md`).

```bash
cd communitas-flutter
flutter pub get
flutter run -d android  # or: -d ios, -d linux, -d windows
```

## Web

Web builds run in demo mode only (FFI is not available in the browser):

```bash
flutter run -d chrome --dart-define=DEMO_MODE=true
```
