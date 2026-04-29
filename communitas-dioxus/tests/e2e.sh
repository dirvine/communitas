#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Build x0xd + Communitas Dioxus and run the live parity E2E harness.
# Proof output lands in proofs/dioxus-parity-YYYYMMDD/.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKSPACE_DIR="$(cd "$CRATE_DIR/.." && pwd)"
X0X_DIR="${X0X_DIR:-$(cd "$WORKSPACE_DIR/../x0x" && pwd)}"
PROOF_DIR="${PROOF_DIR:-$CRATE_DIR/proofs/dioxus-parity-$(date -u +%Y%m%d)}"
mkdir -p "$PROOF_DIR"

STDOUT_LOG="$PROOF_DIR/stdout.log"
STDERR_LOG="$PROOF_DIR/stderr.log"
: >"$STDOUT_LOG"
: >"$STDERR_LOG"
exec > >(tee -a "$STDOUT_LOG") 2> >(tee -a "$STDERR_LOG" >&2)

echo "[e2e] proof_dir=$PROOF_DIR"
echo "[e2e] workspace=$WORKSPACE_DIR"
echo "[e2e] x0x=$X0X_DIR"

RUN_PREFIX=()
if command -v caffeinate >/dev/null 2>&1; then
    RUN_PREFIX=(caffeinate -i)
fi

echo "[e2e] building x0xd release binary"
"${RUN_PREFIX[@]}" cargo build --manifest-path "$X0X_DIR/Cargo.toml" --release --bin x0xd

echo "[e2e] building Communitas Dioxus test-mode binary"
"${RUN_PREFIX[@]}" cargo build --manifest-path "$WORKSPACE_DIR/Cargo.toml" -p communitas-dioxus --features e2e-test-mode

export X0XD_BIN="${X0XD_BIN:-$X0X_DIR/target/release/x0xd}"
export CI_DIOXUS_BIN="${CI_DIOXUS_BIN:-$WORKSPACE_DIR/target/debug/communitas-dioxus}"

TESTS=(
    e2e_identity
    e2e_trust_contacts
    e2e_connectivity
    e2e_groups
    e2e_kv_store
    e2e_presence
    e2e_upgrade
)

# The E2E tests live in a self-contained nested Cargo package
# (`tests/e2e/Cargo.toml`) that is intentionally NOT a member of the
# communitas workspace. This keeps the `x0x-test-harness` path dep
# (which only exists in dev environments where the x0x repo is checked
# out as a sibling) out of the workspace dependency graph so CI
# checkouts that only include communitas can still run
# `cargo metadata` / `cargo build` cleanly.
E2E_MANIFEST="$SCRIPT_DIR/e2e/Cargo.toml"

for test_name in "${TESTS[@]}"; do
    echo "[e2e] running $test_name"
    "${RUN_PREFIX[@]}" cargo test \
        --manifest-path "$E2E_MANIFEST" \
        --test "$test_name" \
        -- --ignored --test-threads=1
    echo "[e2e] $test_name passed"
done

echo "[e2e] all Dioxus parity cells completed"
echo "[e2e] proof bundle: $PROOF_DIR"
