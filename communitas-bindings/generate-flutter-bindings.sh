#!/bin/bash
# Generate Flutter/Dart bindings from Rust using flutter_rust_bridge
#
# Prerequisites:
#   cargo install flutter_rust_bridge_codegen
#
# Usage:
#   ./generate-flutter-bindings.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Generating Flutter Rust Bridge bindings ==="

# Ensure the output directory exists
mkdir -p ../communitas-flutter/lib/src/bindings

# Generate the bindings
flutter_rust_bridge_codegen generate

echo "=== Bindings generated successfully ==="
echo "Output: ../communitas-flutter/lib/src/bindings/"
