# Contributing to Communitas

## Setup
1. Clone: `git clone https://github.com/dirvine/communitas`
2. Rust: `rustup default stable`
3. Install the pinned `dx` CLI: `scripts/install_dx.sh`
4. Build core: `cargo build -p communitas-core`
5. UI dev loop:
```bash
cd communitas-dioxus
dx serve --platform desktop --hotpatch
```

## Conventions
- Rust: `cargo fmt --all`, clippy denies (panic/unwrap/expect).
- Dioxus: `dx check --platform desktop`, component/unit tests live in `communitas-dioxus`.
- Commits: Conventional (feat/fix/chore).
- Tests: prefer end-to-end flows for key behaviors.

## PRs
- Branch: feature/xxx
- Tests pass CI
- Update docs (AGENTS.md for agents).

Thanks for contributing!
