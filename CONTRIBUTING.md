# Contributing to Communitas

## Setup
1. Clone: `git clone https://github.com/saorsa-labs/communitas`
2. Rust: `rustup default stable`
3. Node: `npx setup-node@latest --install-dirs`
4. Deps: `npm ci && cargo build`
5. Run: `npm run tauri dev`

## Conventions
- Rust: `cargo fmt --all`, clippy denies (panic/todo).
- JS: ESLint/Prettier; TS strict.
- Commits: Conventional (feat/fix/chore).
- Tests: 80%+ coverage; e2e for flows.

## PRs
- Branch: feature/xxx
- Tests pass CI
- Update docs (AGENTS.md for agents).

Thanks for contributing!