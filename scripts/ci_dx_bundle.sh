#!/usr/bin/env bash
set -euo pipefail

echo "[dx] installing pinned CLI"
scripts/install_dx.sh

echo "[dx] running desktop bundle for CI smoke"
cd communitas-dioxus
dx bundle --platform desktop --release
