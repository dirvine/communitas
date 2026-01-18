#!/usr/bin/env bash
set -euo pipefail

PINNED_VERSION="0.7.3"

if command -v dx >/dev/null 2>&1; then
  CURRENT_VERSION="$(dx --version | awk '{print $2}')"
  if [[ "${CURRENT_VERSION}" == "${PINNED_VERSION}" ]]; then
    echo "dx ${PINNED_VERSION} already installed"
    exit 0
  fi
fi

echo "Installing dx ${PINNED_VERSION}..."
cargo install dioxus-cli --locked --version "${PINNED_VERSION}"
