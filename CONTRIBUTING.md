# Contributing to Communitas

## Setup
1. Clone: `git clone https://github.com/dirvine/communitas`
2. Rust: `rustup default stable`
3. Flutter: Install Flutter 3.27+
4. Build core: `cargo build -p communitas-core`
5. Flutter deps: `cd communitas-flutter && flutter pub get`
6. Run native: `flutter run -d android` (or ios/linux/windows)
7. Web demo (demo-only): `flutter run -d chrome --dart-define=DEMO_MODE=true`

## FFI Codegen
If you change `communitas-core/src/flutter_api.rs`, re-run codegen:
```bash
flutter_rust_bridge_codegen generate \
  --rust-root communitas-core \
  --rust-input crate::flutter_api \
  --dart-output communitas-flutter/lib/src/bindings \
  --rust-output communitas-core/src/frb_generated.rs \
  --dart-format-line-length 120

# Sanitize generated Rust (remove unwrap/panic paths to satisfy lint policy)
python3 scripts/sanitize-frb-generated.py
```

## Conventions
- Rust: `cargo fmt --all`, clippy denies (panic/unwrap/expect).
- Flutter: `flutter analyze`, `flutter test`.
- Commits: Conventional (feat/fix/chore).
- Tests: prefer end-to-end flows for key behaviors.

## PRs
- Branch: feature/xxx
- Tests pass CI
- Update docs (AGENTS.md for agents).

Thanks for contributing!
