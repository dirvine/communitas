#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
X0X_DIR="$(cd "$ROOT_DIR/../x0x" && pwd)"

cd "$ROOT_DIR"

echo "== communitas-x0x-client contract harness (local + VPS) =="
bash "$ROOT_DIR/scripts/tests/x0x_client_contract_harness.sh" all

echo "== communitas-x0x-client package tests =="
cargo test -p communitas-x0x-client -- --nocapture

echo "== x0x reference semantic tests =="
cargo test --manifest-path "$X0X_DIR/Cargo.toml" --test crdt_convergence_concurrent -- --nocapture
cargo test --manifest-path "$X0X_DIR/Cargo.toml" --test crdt_partition_tolerance -- --nocapture
cargo test --manifest-path "$X0X_DIR/Cargo.toml" --test proptest_crdt -- --nocapture
cargo test --manifest-path "$X0X_DIR/Cargo.toml" --test trust_evaluation_test -- --nocapture
cargo test --manifest-path "$X0X_DIR/Cargo.toml" --test proptest_groups -- --nocapture
cargo test --manifest-path "$X0X_DIR/Cargo.toml" --test proptest_presence -- --nocapture

echo "== release validation complete =="
