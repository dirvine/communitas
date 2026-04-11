#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
MODE="${1:-all}"
RUN_ID="$(date +%Y%m%d-%H%M%S)"
LOG_ROOT="$ROOT_DIR/.testnet-logs/x0x-client-harness-$RUN_ID"
mkdir -p "$LOG_ROOT"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_success() { echo -e "${GREEN}[OK]${NC} $*"; }
log_error() { echo -e "${RED}[ERR]${NC} $*"; }

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    log_error "Missing required command: $1"
    exit 1
  }
}

require_cmd cargo
require_cmd python3
require_cmd curl

SSH_OPTS=(-o ConnectTimeout=10 -o BatchMode=yes -o ControlMaster=no -o ControlPath=none)
LOCAL_PIDS=()
LOCAL_DATA_DIRS=()
REMOTE_CLEANUPS=()

cleanup() {
  for pid in "${LOCAL_PIDS[@]:-}"; do
    kill "$pid" 2>/dev/null || true
  done
  for dir in "${LOCAL_DATA_DIRS[@]:-}"; do
    rm -rf "$dir" 2>/dev/null || true
  done
  for cmd in "${REMOTE_CLEANUPS[@]:-}"; do
    eval "$cmd" >/dev/null 2>&1 || true
  done
}
trap cleanup EXIT

platform_data_base() {
  case "${OSTYPE:-}" in
    darwin*) printf '%s/Library/Application Support' "$HOME" ;;
    linux*) printf '%s/.local/share' "$HOME" ;;
    *)
      python3 - <<'PY'
from pathlib import Path
print(Path.home() / '.local' / 'share')
PY
      ;;
  esac
}

write_matrix() {
  local outfile="$1"
  shift
  python3 - "$outfile" "$@" <<'PY'
import json
import sys

outfile = sys.argv[1]
targets = []
for raw in sys.argv[2:]:
    name, address, token, role, region, kind = raw.split('|', 5)
    targets.append({
        'name': name,
        'address': address,
        'token': token,
        'role': role or None,
        'region': region or None,
        'kind': kind or None,
    })
with open(outfile, 'w', encoding='utf-8') as fh:
    json.dump({'targets': targets}, fh, indent=2)
PY
}

run_live_matrix_suite() {
  local matrix_file="$1"
  local enable_multi_target="${2:-0}"
  log_info "Running live matrix contract suite with $matrix_file"
  X0X_TEST_MATRIX_FILE="$matrix_file" \
  X0X_TEST_ENABLE_MULTI_TARGET="$enable_multi_target" \
    cargo test -p communitas-x0x-client --test live_matrix_contract -- --ignored
}

run_live_mutation_suite() {
  local matrix_file="$1"
  local enable_direct_file="${2:-0}"
  log_info "Running live mutation contract suite with $matrix_file"
  X0X_TEST_MATRIX_FILE="$matrix_file" \
  X0X_TEST_ALLOW_MUTATION=1 \
  X0X_TEST_ENABLE_DIRECT_FILE="$enable_direct_file" \
    cargo test -p communitas-x0x-client --test live_mutation_contract -- --ignored
}

free_local_port() {
  python3 - <<'PY'
import socket
sock = socket.socket()
sock.bind(('127.0.0.1', 0))
print(sock.getsockname()[1])
sock.close()
PY
}

free_remote_port() {
  local ip="$1"
  ssh "${SSH_OPTS[@]}" "root@$ip" "python3 - <<'PY'
import socket
sock = socket.socket()
sock.bind(('127.0.0.1', 0))
print(sock.getsockname()[1])
sock.close()
PY"
}

start_remote_scratch_instance() {
  local name="$1"
  local ip="$2"
  local role="$3"
  local region="$4"
  local api_port="$5"
  local bootstrap_peer="${6:-}"
  local remote_data_dir="/root/.local/share/x0x-$name"
  local remote_log="/tmp/$name.log"
  local remote_config="/tmp/$name-config.toml"

  local remote_bin
  remote_bin=$(ssh "${SSH_OPTS[@]}" "root@$ip" "command -v x0xd || for bin in /root/.local/bin/x0xd /opt/x0x/x0xd /usr/local/bin/x0xd /usr/bin/x0xd; do if [ -x \"\$bin\" ]; then echo \"\$bin\"; break; fi; done")
  if [[ -z "$remote_bin" ]]; then
    log_warn "Skipping scratch instance on $ip: could not find x0xd binary"
    return 1
  fi

  local remote_version
  remote_version=$(ssh "${SSH_OPTS[@]}" "root@$ip" "'$remote_bin' --version | awk '{print \$2}'" 2>/dev/null || true)
  if ! python3 - "$remote_version" <<'PY' >/dev/null 2>&1
import re
import sys
raw = sys.argv[1].strip()
match = re.search(r'(\d+)\.(\d+)\.(\d+)', raw)
if not match:
    raise SystemExit(1)
version = tuple(int(part) for part in match.groups())
raise SystemExit(0 if version >= (0, 15, 0) else 1)
PY
  then
    log_warn "Skipping scratch instance on $ip: x0xd $remote_version is too old for reliable named-instance scratch orchestration"
    return 1
  fi

  if [[ -n "$bootstrap_peer" ]]; then
    ssh "${SSH_OPTS[@]}" "root@$ip" "printf 'bootstrap_peers = [\"%s\"]\n' '$bootstrap_peer' > '$remote_config'"
  else
    ssh "${SSH_OPTS[@]}" "root@$ip" "printf 'bootstrap_peers = []\n' > '$remote_config'"
  fi

  local pid
  pid=$(ssh "${SSH_OPTS[@]}" "root@$ip" "bash -lc 'rm -rf \"$remote_data_dir\"; nohup \"$remote_bin\" --config \"$remote_config\" --name \"$name\" --api-port \"$api_port\" --skip-update-check </dev/null >\"$remote_log\" 2>&1 & echo \$!'" )
  REMOTE_CLEANUPS+=("ssh ${SSH_OPTS[*]} root@$ip 'kill $pid 2>/dev/null || true; pkill -f \"x0xd --name $name\" 2>/dev/null || true; rm -rf \"$remote_data_dir\" \"$remote_log\" \"$remote_config\"'")

  for _ in $(seq 1 60); do
    if out=$(ssh "${SSH_OPTS[@]}" "root@$ip" "if [ -f '$remote_data_dir/api.port' ] && [ -f '$remote_data_dir/api-token' ]; then addr=\$(tr -d '[:space:]' < '$remote_data_dir/api.port'); token=\$(tr -d '[:space:]' < '$remote_data_dir/api-token'); printf '%s|%s\\n' \"\$addr\" \"\$token\"; fi" 2>/dev/null) && [[ -n "$out" ]]; then
      local addr token
      IFS='|' read -r addr token <<< "$out"
      if ssh "${SSH_OPTS[@]}" "root@$ip" "curl -fsS --max-time 2 http://$addr/health >/dev/null" >/dev/null 2>&1; then
        printf '%s|%s|%s|%s|%s\n' "$name" "$addr" "$token" "$role" "$region"
        return 0
      fi
    fi
    sleep 1
  done

  log_error "Timed out waiting for remote scratch instance $name on $ip"
  exit 1
}

start_local_instance() {
  local name="$1"
  local api_port="$2"
  local bootstrap_peer="${3:-}"
  local data_base
  data_base="$(platform_data_base)"
  local data_dir="$data_base/x0x-$name"
  local log_file="$LOG_ROOT/$name.log"
  local config_file="$data_dir/config.toml"

  rm -rf "$data_dir"
  mkdir -p "$data_dir"
  LOCAL_DATA_DIRS+=("$data_dir")
  if [[ -n "$bootstrap_peer" ]]; then
    printf 'bootstrap_peers = ["%s"]\n' "$bootstrap_peer" > "$config_file"
  else
    printf 'bootstrap_peers = []\n' > "$config_file"
  fi

  log_info "Starting local x0xd instance $name on API port $api_port" >&2
  x0xd --config "$config_file" --name "$name" --api-port "$api_port" --skip-update-check >"$log_file" 2>&1 &
  LOCAL_PIDS+=("$!")

  local api_file="$data_dir/api.port"
  local token_file="$data_dir/api-token"
  for _ in $(seq 1 60); do
    if [[ -f "$api_file" && -f "$token_file" ]]; then
      local addr token
      addr="$(tr -d '[:space:]' < "$api_file")"
      token="$(tr -d '[:space:]' < "$token_file")"
      if [[ -n "$addr" && -n "$token" ]]; then
        if curl -fsS --max-time 2 "http://$addr/health" >/dev/null 2>&1; then
          printf '%s|%s|%s\n' "$data_dir" "$addr" "$token"
          return 0
        fi
      fi
    fi
    sleep 1
  done

  log_error "Timed out waiting for local instance $name to become healthy. See $log_file" >&2
  exit 1
}

discover_bootstrap_addr() {
  local api_addr="$1"
  local token="$2"
  local mode="${3:-auto}"
  local public_ip="${4:-}"

  local body
  body=$(curl -fsS --max-time 5 -H "Authorization: Bearer $token" "http://$api_addr/network/status")
  python3 - "$mode" "$body" "$public_ip" <<'PY'
import json
import sys

mode = sys.argv[1]
raw = sys.argv[2]
public_ip = sys.argv[3]
try:
    data = json.loads(raw)
except Exception:
    raise SystemExit(1)

external = data.get("external_addrs") or []
local_addr = data.get("local_addr")

candidates = []
if mode == "local":
    if local_addr and ":" in local_addr:
        port = local_addr.rsplit(":", 1)[1]
        candidates.append(f"127.0.0.1:{port}")
    if local_addr:
        candidates.append(local_addr)
    candidates.extend(external)
elif mode == "external":
    if public_ip and local_addr and ":" in local_addr:
        port = local_addr.rsplit(":", 1)[1]
        candidates.append(f"{public_ip}:{port}")
    candidates.extend(external)
    if local_addr:
        candidates.append(local_addr)
else:
    candidates.extend(external)
    if local_addr:
        candidates.append(local_addr)

for candidate in candidates:
    if candidate:
        print(candidate)
        raise SystemExit(0)
raise SystemExit(1)
PY
}

run_local_suite() {
  require_cmd x0xd

  local names=("cxh-a-$RUN_ID" "cxh-b-$RUN_ID" "cxh-c-$RUN_ID")
  local specs=()
  local bootstrap_peer=""

  for idx in 0 1 2; do
    local name="${names[$idx]}"
    local api_port
    api_port="$(free_local_port)"
    IFS='|' read -r data_dir addr token < <(start_local_instance "$name" "$api_port" "$bootstrap_peer")
    specs+=("$name|$addr|$token|scratch|local|local")
    log_success "$name healthy at $addr"
    if [[ -z "$bootstrap_peer" ]]; then
      bootstrap_peer="$(discover_bootstrap_addr "$addr" "$token" local)"
      log_info "Using local scratch bootstrap peer $bootstrap_peer"
    fi
  done

  log_info "Waiting 15s for local instances to gossip-discover each other"
  sleep 15

  local matrix="$LOG_ROOT/local-matrix.json"
  write_matrix "$matrix" "${specs[@]}"
  log_success "Local matrix written to $matrix"

  run_live_matrix_suite "$matrix" 1
  run_live_mutation_suite "$matrix" 1
}

discover_remote_target() {
  local name="$1"
  local ip="$2"
  local role="$3"
  local region="$4"
  local script='for base in /root/.local/share/x0x /var/lib/x0x/data /var/lib/x0x; do
    if [ -f "$base/api.port" ] && [ -f "$base/api-token" ]; then
      addr=$(tr -d "[:space:]" < "$base/api.port")
      token=$(tr -d "[:space:]" < "$base/api-token")
      printf "%s|%s\n" "$addr" "$token"
      exit 0
    fi
  done
  exit 1'

  if out=$(ssh "${SSH_OPTS[@]}" "root@$ip" "$script" 2>/dev/null); then
    local addr token
    IFS='|' read -r addr token <<< "$out"
    printf '%s|%s|%s|%s|%s\n' "$name" "$addr" "$token" "$role" "$region"
  else
    log_warn "Skipping $name ($ip): could not discover api.port/api-token over SSH"
    return 1
  fi
}

run_vps_suite() {
  require_cmd ssh

  local specs=()
  local inventory=(
    "saorsa-1|77.42.75.115|registry|Helsinki, FI"
    "saorsa-2|142.93.199.50|bootstrap|NYC1, US"
    "saorsa-3|147.182.234.192|bootstrap|SFO3, US"
    "saorsa-4|206.189.7.117|nat-full-cone|AMS3, NL"
    "saorsa-5|144.126.230.161|nat-addr-restricted|LON1, UK"
    "saorsa-6|65.21.157.229|nat-port-restricted|Helsinki, FI"
    "saorsa-7|116.203.101.172|bootstrap|Nuremberg, DE"
    "saorsa-8|149.28.156.231|latency|Singapore, SG"
    "saorsa-9|45.77.176.184|latency|Tokyo, JP"
    "saorsa-10|77.42.39.239|nat-symmetric|Falkenstein, DE"
  )

  for entry in "${inventory[@]}"; do
    IFS='|' read -r name ip role region <<< "$entry"
    if remote=$(discover_remote_target "$name" "$ip" "$role" "$region"); then
      IFS='|' read -r remote_name remote_addr token remote_role remote_region <<< "$remote"
      local forward_port
      forward_port="$(free_local_port)"
      ssh -o ExitOnForwardFailure=yes -o ServerAliveInterval=30 -o ServerAliveCountMax=3 \
        -o ControlMaster=no -o ControlPath=none -o BatchMode=yes -N \
        -L "${forward_port}:${remote_addr}" "root@${ip}" >/dev/null 2>&1 &
      LOCAL_PIDS+=("$!")

      local forwarded_addr="127.0.0.1:${forward_port}"
      local ready=false
      for _ in $(seq 1 20); do
        if curl -fsS --max-time 2 "http://${forwarded_addr}/health" >/dev/null 2>&1; then
          ready=true
          break
        fi
        sleep 1
      done
      if [[ "$ready" != true ]]; then
        log_warn "Skipping $name ($ip): SSH tunnel to ${remote_addr} never became healthy"
        continue
      fi

      specs+=("${remote_name}|${forwarded_addr}|${token}|${remote_role}|${remote_region}|remote")
      log_success "Discovered remote target $name via tunnel ${forwarded_addr} -> ${remote_addr}"
    fi
  done

  if [[ ${#specs[@]} -eq 0 ]]; then
    log_error "No remote x0xd targets discovered"
    exit 1
  fi

  local matrix="$LOG_ROOT/vps-matrix.json"
  write_matrix "$matrix" "${specs[@]}"
  log_success "VPS matrix written to $matrix"

  run_live_matrix_suite "$matrix" 0
}

run_vps_mutation_suite() {
  require_cmd ssh
  require_cmd x0xd

  local specs=()
  local bootstrap_peer=""
  local inventory=(
    "cxh-vps-a-$RUN_ID|147.182.234.192|scratch-sfo|SFO3, US"
    "cxh-vps-b-$RUN_ID|116.203.101.172|scratch-nuremberg|Nuremberg, DE"
    "cxh-vps-c-$RUN_ID|149.28.156.231|scratch-singapore|Singapore, SG"
    "cxh-vps-d-$RUN_ID|142.93.199.50|scratch-nyc|NYC1, US"
    "cxh-vps-e-$RUN_ID|45.77.176.184|scratch-tokyo|Tokyo, JP"
    "cxh-vps-f-$RUN_ID|65.21.157.229|scratch-helsinki|Helsinki, FI"
  )

  for entry in "${inventory[@]}"; do
    IFS='|' read -r name ip role region <<< "$entry"
    local remote_port
    if ! remote_port="$(free_remote_port "$ip" 2>/dev/null)"; then
      log_warn "Skipping scratch instance $name on $ip: could not allocate remote port"
      continue
    fi
    local remote
    if ! remote=$(start_remote_scratch_instance "$name" "$ip" "$role" "$region" "$remote_port" "$bootstrap_peer"); then
      continue
    fi
    IFS='|' read -r remote_name remote_addr token remote_role remote_region <<< "$remote"

    local forward_port
    forward_port="$(free_local_port)"
    ssh -o ExitOnForwardFailure=yes -o ServerAliveInterval=30 -o ServerAliveCountMax=3 \
      -o ControlMaster=no -o ControlPath=none -o BatchMode=yes -N \
      -L "${forward_port}:${remote_addr}" "root@${ip}" >/dev/null 2>&1 &
    LOCAL_PIDS+=("$!")

    local forwarded_addr="127.0.0.1:${forward_port}"
    local ready=false
    for _ in $(seq 1 20); do
      if curl -fsS --max-time 2 "http://${forwarded_addr}/health" >/dev/null 2>&1; then
        ready=true
        break
      fi
      sleep 1
    done
    if [[ "$ready" != true ]]; then
      log_warn "Skipping scratch instance $name on $ip: tunnel to ${remote_addr} never became healthy"
      continue
    fi

    specs+=("${remote_name}|${forwarded_addr}|${token}|${remote_role}|${remote_region}|remote")
    log_success "Started scratch VPS target $name via tunnel ${forwarded_addr} -> ${remote_addr}"
    if [[ -z "$bootstrap_peer" ]]; then
      bootstrap_peer="$(discover_bootstrap_addr "$forwarded_addr" "$token" external "$ip")"
      log_info "Using scratch VPS bootstrap peer $bootstrap_peer"
    fi

    if [[ ${#specs[@]} -ge 3 ]]; then
      break
    fi
  done

  if [[ ${#specs[@]} -lt 3 ]]; then
    log_warn "Skipping scratch VPS mutation suite: need 3 compatible remote targets, found ${#specs[@]}"
    return 0
  fi

  local matrix="$LOG_ROOT/vps-mutation-matrix.json"
  write_matrix "$matrix" "${specs[@]}"
  log_success "Scratch VPS mutation matrix written to $matrix"

  log_info "Waiting 30s for scratch VPS daemons to announce and discover each other"
  sleep 30

  run_live_mutation_suite "$matrix" 0
}

print_usage() {
  cat <<EOF
x0x client contract harness

Usage:
  $0 local         Start 3 local named x0xd instances and run full contract suite
  $0 vps           Discover all configured VPS x0xd nodes and run read-only matrix suite
  $0 vps-mutation  Start 3 scratch VPS x0xd instances and run the mutation suite
  $0 all           Run local suite, VPS read-only suite, then scratch VPS mutation suite
EOF
}

case "$MODE" in
  local)
    run_local_suite
    ;;
  vps)
    run_vps_suite
    ;;
  vps-mutation)
    run_vps_mutation_suite
    ;;
  all)
    run_local_suite
    run_vps_suite
    run_vps_mutation_suite
    ;;
  help|--help|-h)
    print_usage
    ;;
  *)
    log_error "Unknown mode: $MODE"
    print_usage
    exit 1
    ;;
esac

log_success "Harness run complete. Logs: $LOG_ROOT"
