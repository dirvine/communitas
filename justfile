# communitas justfile
#
# Run `just --list` for every recipe.

set shell := ["bash", "-uc"]

default:
    @just --list

# ── Rust ─────────────────────────────────────────────────────────────────

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo nextest run --all-features --workspace

build:
    cargo build --all-features

build-release:
    cargo build --release --all-features

clean:
    cargo clean

check: fmt-check lint build test

# ── Apps ─────────────────────────────────────────────────────────────────

dioxus:
    cd communitas-dioxus && cargo build --release

apple:
    cd communitas-apple && swift build -c release

# ── Parity ───────────────────────────────────────────────────────────────

# Generate the Dioxus ↔ Swift feature parity matrix at docs/parity.md.
parity:
    bash scripts/parity-check.sh
