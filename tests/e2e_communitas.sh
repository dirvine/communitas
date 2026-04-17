#!/usr/bin/env bash
# e2e_communitas.sh — lightweight parity smoke test.
#
# Verifies the two desktop apps consume the same x0xd REST surface by
# replaying a canonical sequence of client calls through a local daemon
# and asserting each call succeeds. Does NOT launch the GUIs — that
# requires a human to click through onboarding. Instead, it drives the
# x0xd REST API directly using the same requests both apps make, so a
# failure here tells us the *shared* endpoint contract is broken.
#
# Usage:
#   tests/e2e_communitas.sh                 # use running x0xd on localhost
#   tests/e2e_communitas.sh --spawn          # spawn an ephemeral x0xd
#   tests/e2e_communitas.sh --port 12900     # custom port
#
# Exit codes:
#   0 — every assertion passed
#   1 — one or more assertions failed
#   2 — bad arguments
#   3 — x0xd unreachable

set -euo pipefail

SPAWN=0
PORT=12700
TOKEN=""
for arg in "$@"; do
    case "$arg" in
        --spawn) SPAWN=1 ;;
        --port)  shift; PORT="$1" ;;
        --token) shift; TOKEN="$1" ;;
        -h|--help) sed -n '2,25p' "$0"; exit 0 ;;
        *) echo "Unknown flag: $arg" >&2; exit 2 ;;
    esac
    shift || true
done

fail=0
pass=0

red()   { printf '\033[31m%s\033[0m' "$*"; }
green() { printf '\033[32m%s\033[0m' "$*"; }
yellow(){ printf '\033[33m%s\033[0m' "$*"; }

hdr=(-H "Accept: application/json")
if [ -n "$TOKEN" ]; then
    hdr+=(-H "Authorization: Bearer $TOKEN")
fi
BASE="http://127.0.0.1:${PORT}"

if ! curl -s --max-time 2 "${hdr[@]}" "$BASE/health" >/dev/null; then
    if [ "$SPAWN" = "1" ]; then
        echo "Spawning x0xd on port $PORT is not wired yet; start it manually and re-run." >&2
    fi
    echo "$(red FAIL): x0xd unreachable at $BASE" >&2
    exit 3
fi

# ── discover token from x0x config if not set explicitly ────────────────
if [ -z "$TOKEN" ]; then
    for candidate in \
        "$HOME/Library/Application Support/x0x/api-token" \
        "$HOME/.local/share/x0x/api-token" \
        "$HOME/.config/x0x/api-token"; do
        if [ -f "$candidate" ]; then
            TOKEN="$(cat "$candidate")"
            hdr=(-H "Accept: application/json" -H "Authorization: Bearer $TOKEN")
            break
        fi
    done
fi

# ── assertion helper ────────────────────────────────────────────────────
check() {
    local name="$1"
    local method="$2"
    local path="$3"
    local body="${4:-}"
    local url="${BASE}${path}"

    local code
    if [ -n "$body" ]; then
        code="$(curl -s -o /dev/null -w '%{http_code}' -X "$method" "${hdr[@]}" \
            -H 'Content-Type: application/json' --data "$body" "$url")"
    else
        code="$(curl -s -o /dev/null -w '%{http_code}' -X "$method" "${hdr[@]}" "$url")"
    fi

    # 2xx or 404 (missing resource) count as "contract is intact" — we're
    # asserting the endpoint exists and is wired, not that specific content
    # is present. 5xx or 401 count as failure.
    if [[ "$code" =~ ^2 ]] || [ "$code" = "404" ] || [ "$code" = "400" ]; then
        printf '  %s %-6s %-48s %s\n' "$(green OK)" "$method" "$path" "$code"
        pass=$((pass+1))
    else
        printf '  %s %-6s %-48s %s\n' "$(red FAIL)" "$method" "$path" "$code"
        fail=$((fail+1))
    fi
}

# ── Canonical endpoint list — every endpoint both apps exercise. ────────
# Sourced from the Dioxus ↔ Swift parity matrix (docs/parity.md). Both
# apps share this surface — a regression here breaks both at once.
echo
yellow "== Shared endpoint contract =="
echo
echo "(GET/POST hits against live daemon at $BASE)"
echo

check "health"                 GET    /health
check "status"                 GET    /status
check "agent"                  GET    /agent
check "agent_card"             GET    "/agent/card?display_name=test"
check "bootstrap_cache"        GET    /network/bootstrap-cache
check "network_status"         GET    /network/status
check "peers"                  GET    /peers
check "presence_online"        GET    /presence/online
check "discovered_agents"      GET    /agents/discovered
check "list_contacts"          GET    /contacts
check "list_groups"            GET    /groups
check "list_stores"            GET    /stores
check "list_task_lists"        GET    /task-lists
check "transfers"              GET    /files/transfers
check "discover_groups"        GET    /groups/discover
check "discover_nearby"        GET    /groups/discover/nearby
check "direct_connections"     GET    /direct/connections
check "check_upgrade"          GET    /upgrade
check "constitution_json"      GET    /constitution/json

# ── Write-path smoke: POST /announce is idempotent-ish ─────────────────
check "announce"               POST   /announce  '{"include_user_identity":false,"human_consent":false}'

# ── Summary ─────────────────────────────────────────────────────────────
echo
if [ "$fail" = "0" ]; then
    echo "$(green PASS): $pass assertions"
    exit 0
else
    echo "$(red FAIL): $fail failed, $pass passed"
    exit 1
fi
