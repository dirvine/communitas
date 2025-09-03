# Repository Guidelines

## Project Structure & Modules
- `src`: React/Vite TypeScript UI (components, services, utils, tests).
- `src-tauri`: Rust Tauri backend crate (`communitas-tauri`) with commands, storage, security, and tests.
- `public`: Static assets for the UI. `docs/`: design/architecture notes. `tests/`: JS integration helpers. Other helpers: `deployment/`, scripts, Docker files.

## Build, Test, and Development
- Frontend dev: `npm run dev` (Node 20 required). Desktop dev: `npm run tauri dev`.
- Frontend build/preview: `npm run build`, `npm run preview`.
- Typecheck: `npm run typecheck`.
- JS unit tests: `npm test` (Vitest). Fast subset: `npm run test:run`.
- Rust checks: `cargo check`; tests: `cargo test -p communitas-tauri`.
- Rust lint policy: `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used` (add `-W clippy::pedantic` optionally).
- Formatting: `cargo fmt --all` before commits/CI.

## Coding Style & Naming
- TypeScript: 2-space indent, camelCase for vars/functions, PascalCase for components/types. Tests in `__tests__` with `*.test.ts[x]`.
- Rust: snake_case for functions/modules, CamelCase for types. Non-test code must not use `unwrap/expect/panic!`; prefer `thiserror` for errors and `tracing` for logs.
- File naming: kebab-case for files in UI; module-focused `mod.rs`/`lib.rs` in Rust where applicable.

## Testing Guidelines
- Frontend: Vitest + jsdom. Keep fast, deterministic unit tests near code (`src/**/__tests__`).
- Rust: unit tests under `#[cfg(test)]` and integration tests in `src-tauri/tests`.
- Name tests clearly (what it ensures) and avoid cross-test state.

## Commit & PR Guidelines
- Conventional Commits: `feat:`, `fix:`, `docs:`, `test:`, `chore:`, optional scope (e.g., `feat(pqc): ...`).
- PRs include: concise description, linked issues, screenshots for UI changes, and reproduction/testing steps.
- CI hygiene: ensure `npm run typecheck`, `npm test`, `cargo fmt --all`, and Clippy lints all pass.

## Security & Configuration
- Do not commit secrets. Use platform keyring (Rust uses `keyring`) and environment configs.
- Optional: run dependency checks if installed (e.g., `cargo deny check` using `src-tauri/deny.toml`).
- See `ARCHITECTURE.md` and `DESIGN.md` for deeper system context.

