# Project: communitas

## Overview
This project uses Claude Code with automated hooks for:
- 11Labs voice notifications when Claude needs input
- Task completion announcements
- Quality checks and validation

## Hooks Configuration
Hooks are automatically configured via claude-init.
Settings location: .claude/settings.json

## Notes
Add project-specific context here...

### Lint & Test Policy (Rust)
- Production code: do not use `unwrap`, `expect`, or `panic!`.
- Tests: may use `unwrap/expect/panic!` for clarity and speed.
- Clippy: enforce with `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`.
- Do not require `clippy::pedantic`.
- Format with `cargo fmt --all` before commits.
