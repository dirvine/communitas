#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <port> [port2 port3 ...]" >&2
  exit 1
fi

for port in "$@"; do
  echo "Killing any process listening on TCP port ${port}..."
  # shellcheck disable=SC2046
  PIDS=$(lsof -tiTCP:"${port}" -sTCP:LISTEN || true)
  if [ -n "${PIDS}" ]; then
    echo " PIDs: ${PIDS}"
    kill -9 ${PIDS} || true
  else
    echo " No listener on ${port}"
  fi
done

echo "Done."

