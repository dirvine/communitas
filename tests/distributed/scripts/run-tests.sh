#!/bin/bash
# Run distributed MCP tests
# Usage: ./run-tests.sh [SUITE] [OPTIONS]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
TESTS_DIR="$SCRIPT_DIR/.."

# Default configuration
SUITE="${1:-all}"
shift || true

LOCAL_MCP_PORT=3041
LOCAL_ONLY=false
LOCAL_CLUSTER=false
LOCAL_CLUSTER_BASE_PORT=3041
CUSTOM_NODES=""
START_LOCAL_SERVER=true
REPORT_DIR="$TESTS_DIR/reports/$(date +%Y%m%d_%H%M%S)"
LOCAL_CLUSTER_DIR="$PROJECT_ROOT/.local-cluster"
LOCAL_CLUSTER_STARTED=false
BOOTSTRAP_OVERRIDE=""
NETWORK_BASE_PORT=4100

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

ACTORS=(alice bob charlie dave)
LOCAL_CLUSTER_PIDS=()
LOCAL_CLUSTER_PORTS=()
MCP_BIN="$PROJECT_ROOT/target/release/communitas-mcp"

ensure_mcp_binary() {
    if [[ ! -x "$MCP_BIN" ]]; then
        log_info "Building communitas-mcp binary..."
        cd "$PROJECT_ROOT"
        cargo build --release -p communitas-mcp
    fi
}

start_mcp_instance() {
    local name="$1"
    local port="$2"
    local storage_root="$3"
    local four_words="$4"
    local log_file="$5"
    local network_port="$6"
    local bootstrap_nodes="$7"

    local data_dir="$storage_root/data"
    local demo_dir="$storage_root/demo"
    mkdir -p "$data_dir" "$demo_dir"

    COMMUNITAS_DATA_DIR="$data_dir" \
    COMMUNITAS_PORT="$network_port" \
    COMMUNITAS_BOOTSTRAP="$bootstrap_nodes" \
    "$MCP_BIN" --http --demo --listen "127.0.0.1:$port" \
        --storage-dir "$demo_dir" \
        --four-words "$four_words" \
        --display-name "$name Local" \
        > "$log_file" 2>&1 &
    echo $!
}

wait_for_health() {
    local port="$1"
    local timeout="${2:-120}"
    for ((i = 1; i <= timeout; i++)); do
        if curl -sf "http://127.0.0.1:$port/health" >/dev/null 2>&1; then
            return 0
        fi
        sleep 1
    done
    return 1
}

start_local_cluster() {
    ensure_mcp_binary
    local log_dir="$REPORT_DIR/logs"
    mkdir -p "$log_dir"
    rm -rf "$LOCAL_CLUSTER_DIR"
    mkdir -p "$LOCAL_CLUSTER_DIR"

    local computed_bootstrap="$BOOTSTRAP_OVERRIDE"
    if [[ -z "$computed_bootstrap" ]]; then
        local bootstrap_nodes=()
        if [[ "${LOCAL_CLUSTER_INCLUDE_LOCAL_BOOTSTRAP:-}" == "1" ]]; then
            for idx in "${!ACTORS[@]}"; do
                bootstrap_nodes+=("127.0.0.1:$((NETWORK_BASE_PORT + idx * 2))")
            done
        fi
        # Include remote saorsa nodes as additional introducers so local clusters can join the real network
        local remote_bootstrap=(
            "142.93.199.50:3040"   # saorsa-2 (NYC)
            "147.182.234.192:3040" # saorsa-3 (SFO)
            "65.21.157.229:3040"   # saorsa-6 (Helsinki)
            "116.203.101.172:3040" # saorsa-7 (Nuremberg)
            "149.28.156.231:3040"  # saorsa-8 (Singapore)
            "45.77.176.184:3040"   # saorsa-9 (Tokyo)
        )
        bootstrap_nodes+=("${remote_bootstrap[@]}")
        computed_bootstrap=$(IFS=,; echo "${bootstrap_nodes[*]}")
    fi
    log_info "Bootstrap nodes: ${computed_bootstrap}"

    local idx=0
    for actor in "${ACTORS[@]}"; do
        local port=$((LOCAL_CLUSTER_BASE_PORT + idx))
        LOCAL_CLUSTER_PORTS[$idx]=$port
        local storage_dir="$LOCAL_CLUSTER_DIR/$actor"
        local log_file="$log_dir/mcp-$actor.log"
        local four_words="${actor}-local-demo-node"
        local network_port=$((NETWORK_BASE_PORT + idx * 2))

        log_info "Starting local MCP for $actor on port $port"
        local pid
        pid=$(start_mcp_instance "$actor" "$port" "$storage_dir" "$four_words" "$log_file" "$network_port" "$computed_bootstrap")
        LOCAL_CLUSTER_PIDS[$idx]=$pid

        if ! wait_for_health "$port" 120; then
            log_error "Local MCP instance for $actor (port $port) failed health check"
            exit 1
        fi
        idx=$((idx + 1))
    done

    LOCAL_CLUSTER_STARTED=true
}

stop_local_cluster() {
    local idx=0
    for actor in "${ACTORS[@]}"; do
        local pid="${LOCAL_CLUSTER_PIDS[$idx]:-}"
        if [[ -n "$pid" ]]; then
            kill "$pid" 2>/dev/null || true
            wait "$pid" 2>/dev/null || true
        fi
        idx=$((idx + 1))
    done

    if [[ "$LOCAL_CLUSTER_STARTED" == true ]]; then
        rm -rf "$LOCAL_CLUSTER_DIR"
    fi
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    if [[ -n "${LOCAL_MCP_PID:-}" ]]; then
        kill "$LOCAL_MCP_PID" 2>/dev/null || true
    fi
    if [[ -n "${TMP_CONFIG:-}" && -f "${TMP_CONFIG:-}" ]]; then
        rm -f "$TMP_CONFIG"
    fi
    stop_local_cluster
}
trap cleanup EXIT

# Parse additional options
ANTHROPIC_KEY="${ANTHROPIC_API_KEY:-${KIMI_API_KEY:-}}"
MODEL="claude-3-haiku-20240307"
API_BASE="${ANTHROPIC_BASE_URL:-${KIMI_API_BASE_URL:-https://api.anthropic.com/v1}}"
UNLOCK_SCOPES="${UNLOCK_SCOPES:-full_access}"

while [[ $# -gt 0 ]]; do
    case $1 in
        --key)
            ANTHROPIC_KEY="$2"
            shift 2
            ;;
        --model)
            MODEL="$2"
            shift 2
            ;;
        --api-base)
            API_BASE="$2"
            shift 2
            ;;
        --unlock-scopes)
            UNLOCK_SCOPES="$2"
            shift 2
            ;;
        --output)
            REPORT_DIR="$2"
            shift 2
            ;;
        --local-only)
            LOCAL_ONLY=true
            shift
            ;;
        --local-cluster)
            LOCAL_CLUSTER=true
            START_LOCAL_SERVER=false
            LOCAL_ONLY=false
            shift
            ;;
        --local-cluster-base-port)
            LOCAL_CLUSTER_BASE_PORT="$2"
            shift 2
            ;;
        --network-base-port)
            NETWORK_BASE_PORT="$2"
            shift 2
            ;;
        --bootstrap)
            BOOTSTRAP_OVERRIDE="$2"
            shift 2
            ;;
        --nodes)
            CUSTOM_NODES="$2"
            START_LOCAL_SERVER=false
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [SUITE] [OPTIONS]"
            echo ""
            echo "Suites:"
            echo "  all         Run all test phases (default)"
            echo "  identity    Phase 1: Identity & Authentication"
            echo "  entities    Phase 2: Entity Lifecycle"
            echo "  members     Phase 3: Member Management"
            echo "  messaging   Phase 4: Messaging"
            echo "  files       Phase 5: Virtual Disks"
            echo "  kanban      Phase 6: Kanban"
            echo "  contacts    Phase 7: Contacts"
            echo "  networking  Phase 8: P2P Networking"
            echo "  websites    Phase 9: Websites"
            echo "  invites     Phase 10: Invites"
            echo "  sync        Phase 11: Sync & Reconnection"
            echo ""
            echo "Options:"
            echo "  --key KEY       Anthropic-compatible API key (default: \$ANTHROPIC_API_KEY or \$KIMI_API_KEY)"
            echo "  --model MODEL   Model to use (default: claude-3-haiku-20240307)"
            echo "  --api-base URL  Override API base URL (default: \$ANTHROPIC_BASE_URL / \$KIMI_API_BASE_URL or https://api.anthropic.com/v1)"
            echo "  --unlock-scopes scopes  Comma-separated unlock scopes (default: full_access)"
            echo "  --output DIR    Report output directory"
            echo "  --local-only    Run all actors against a single local MCP instance (port 3041)"
            echo "  --local-cluster Run each actor against its own local MCP instance (ports start at 3041)"
            echo "  --local-cluster-base-port PORT  Base port for --local-cluster (default: 3041)"
            echo "  --network-base-port PORT   Base port for networking sockets (default: 4100)"
            echo "  --bootstrap ADDRS  Override COMMUNITAS_BOOTSTRAP (comma-separated host:port)"
            echo "  --nodes STRING  Override node map (name:host:port,...); skips local MCP launch"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate Anthropic key
if [[ -z "$ANTHROPIC_KEY" ]]; then
    log_error "No API key configured. Use --key or export ANTHROPIC_API_KEY / KIMI_API_KEY"
    exit 1
fi

# Map suite to config file
get_config_file() {
    case $1 in
        all)       echo "$TESTS_DIR/scenarios/all_tools.yaml" ;;
        identity)  echo "$TESTS_DIR/scenarios/phase_01_identity.yaml" ;;
        entities)  echo "$TESTS_DIR/scenarios/phase_02_entities.yaml" ;;
        members)   echo "$TESTS_DIR/scenarios/phase_03_members.yaml" ;;
        messaging) echo "$TESTS_DIR/scenarios/phase_04_messaging.yaml" ;;
        files)     echo "$TESTS_DIR/scenarios/phase_05_files.yaml" ;;
        kanban)    echo "$TESTS_DIR/scenarios/phase_06_kanban.yaml" ;;
        contacts)  echo "$TESTS_DIR/scenarios/phase_07_contacts.yaml" ;;
        networking) echo "$TESTS_DIR/scenarios/phase_08_networking.yaml" ;;
        websites)  echo "$TESTS_DIR/scenarios/phase_09_websites.yaml" ;;
        invites)   echo "$TESTS_DIR/scenarios/phase_10_invites.yaml" ;;
        sync)      echo "$TESTS_DIR/scenarios/phase_11_sync.yaml" ;;
        *)
            log_error "Unknown suite: $1"
            exit 1
            ;;
    esac
}

CONFIG_FILE=$(get_config_file "$SUITE")
if [[ ! -f "$CONFIG_FILE" ]]; then
    log_error "Config file not found: $CONFIG_FILE"
    exit 1
fi

# Some suites need bootstrap phases to define shared context
TMP_CONFIG=""

combine_scenarios() {
    local output="$1"
    shift
    python3 - "$output" "$@" <<'PY'
import sys, yaml
out = sys.argv[1]
files = sys.argv[2:]
combined = {
    "name": "Combined Scenario",
    "description": "",
    "tools_covered": [],
    "test_cases": []
}
tool_set = []
for path in files:
    with open(path, "r", encoding="utf-8") as f:
        data = yaml.safe_load(f)
    if not data:
        continue
    if not combined["description"]:
        combined["description"] = data.get("description", "")
    for tool in data.get("tools_covered", []) or []:
        if tool not in tool_set:
            tool_set.append(tool)
    combined["test_cases"].extend(data.get("test_cases", []))
combined["tools_covered"] = tool_set
with open(out, "w", encoding="utf-8") as f:
    yaml.safe_dump(combined, f, sort_keys=False)
PY
}

if [[ "$SUITE" == "all" ]]; then
    TMP_CONFIG=$(mktemp)
    PHASE_FILES=(
        "$TESTS_DIR/scenarios/phase_01_identity.yaml"
        "$TESTS_DIR/scenarios/phase_02_entities.yaml"
        "$TESTS_DIR/scenarios/phase_03_members.yaml"
        "$TESTS_DIR/scenarios/phase_04_messaging.yaml"
        "$TESTS_DIR/scenarios/phase_05_files.yaml"
        "$TESTS_DIR/scenarios/phase_06_kanban.yaml"
        "$TESTS_DIR/scenarios/phase_07_contacts.yaml"
        "$TESTS_DIR/scenarios/phase_08_networking.yaml"
        "$TESTS_DIR/scenarios/phase_09_websites.yaml"
        "$TESTS_DIR/scenarios/phase_10_invites.yaml"
        "$TESTS_DIR/scenarios/phase_11_sync.yaml"
    )
    combine_scenarios "$TMP_CONFIG" "${PHASE_FILES[@]}"
    CONFIG_FILE="$TMP_CONFIG"
elif [[ "$SUITE" != "identity" && "$SUITE" != "entities" ]]; then
    TMP_CONFIG=$(mktemp)
    combine_scenarios "$TMP_CONFIG" \
        "$TESTS_DIR/scenarios/phase_01_identity.yaml" \
        "$TESTS_DIR/scenarios/phase_02_entities.yaml" \
        "$CONFIG_FILE"
    CONFIG_FILE="$TMP_CONFIG"
fi

# Create report directory
mkdir -p "$REPORT_DIR"
log_info "Reports will be saved to: $REPORT_DIR"

# Build orchestrator if needed
ORCHESTRATOR="$PROJECT_ROOT/target/release/distributed-test-orchestrator"
if [[ ! -f "$ORCHESTRATOR" ]]; then
    log_info "Building test orchestrator..."
    cd "$PROJECT_ROOT"
    cargo build --release -p distributed-test-orchestrator
fi

if [[ "$LOCAL_CLUSTER" == true ]]; then
    log_info "Starting local MCP cluster (one node per actor)..."
    start_local_cluster
fi

if [[ "$START_LOCAL_SERVER" == true ]]; then
    # Start local MCP server
    log_info "Starting local MCP server on port $LOCAL_MCP_PORT..."
    cd "$PROJECT_ROOT"
    cargo run --release -p communitas-mcp -- \
        --http --demo --listen "127.0.0.1:$LOCAL_MCP_PORT" &
    LOCAL_MCP_PID=$!

    # Wait for local server to be ready (allow longer build times on first run)
    STARTUP_TIMEOUT="${LOCAL_MCP_STARTUP_TIMEOUT:-300}"
    log_info "Waiting for local MCP server (timeout ${STARTUP_TIMEOUT}s)..."
    for ((i = 1; i <= STARTUP_TIMEOUT; i++)); do
        if curl -sf "http://127.0.0.1:$LOCAL_MCP_PORT/health" > /dev/null 2>&1; then
            log_success "Local MCP server is ready"
            break
        fi
        if [[ $i -eq STARTUP_TIMEOUT ]]; then
            log_error "Local MCP server failed to start within ${STARTUP_TIMEOUT}s"
            exit 1
        fi
        sleep 1
    done
fi

# Node configuration
if [[ -n "$CUSTOM_NODES" ]]; then
    log_info "Using custom node map: $CUSTOM_NODES"
    NODES="$CUSTOM_NODES"
elif [[ "$LOCAL_CLUSTER" == true ]]; then
    log_info "Running in local cluster mode (dedicated MCP per actor)"
    nodes_array=()
    for idx in "${!ACTORS[@]}"; do
        actor="${ACTORS[$idx]}"
        port="${LOCAL_CLUSTER_PORTS[$idx]}"
        nodes_array+=("$actor:127.0.0.1:$port")
    done
    NODES=$(IFS=,; echo "${nodes_array[*]}")
elif [[ "$LOCAL_ONLY" == true ]]; then
    log_info "Running in local-only mode (all actors -> 127.0.0.1:$LOCAL_MCP_PORT)"
    NODES="alice:127.0.0.1:$LOCAL_MCP_PORT,bob:127.0.0.1:$LOCAL_MCP_PORT,charlie:127.0.0.1:$LOCAL_MCP_PORT,dave:127.0.0.1:$LOCAL_MCP_PORT"
else
    NODES="alice:142.93.199.50:3040,bob:147.182.234.192:3040,charlie:65.21.157.229:3040,dave:127.0.0.1:$LOCAL_MCP_PORT"
fi

# Run orchestrator
log_info "=== Running Test Suite: $SUITE ==="
echo ""

"$ORCHESTRATOR" \
    --config "$CONFIG_FILE" \
    --nodes "$NODES" \
    --output "$REPORT_DIR" \
    --anthropic-key "$ANTHROPIC_KEY" \
    --model "$MODEL" \
    --api-base "$API_BASE" \
    --unlock-scopes "$UNLOCK_SCOPES" \
    "$@"

RESULT=$?

if [[ $RESULT -ne 0 ]]; then
    echo ""
fi

log_info "=== Test Complete ==="
log_info "Report: $REPORT_DIR/report.html"
log_info "JSON: $REPORT_DIR/report.json"

exit $RESULT
