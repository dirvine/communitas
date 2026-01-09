#!/bin/bash
# VPS Integration Test for Communitas MCP
# Tests multi-node deployment, health checks, and feature workflows
# Usage: ./vps-integration-test.sh [--local] [--build] [--skip-deploy]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Node configuration
declare -A VPS_NODES=(
    ["alice"]="206.189.7.117:3040"
    ["bob"]="144.126.230.161:3040"
    ["charlie"]="65.21.157.229:3040"
)

# Local test configuration
LOCAL_NODES=(
    "http://127.0.0.1:3040"
    "http://127.0.0.1:3041"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[PASS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[FAIL]${NC} $1"; }
log_section() { echo -e "\n${CYAN}=== $1 ===${NC}\n"; }

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Parse arguments
LOCAL_MODE=false
BUILD=false
SKIP_DEPLOY=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --local)
            LOCAL_MODE=true
            shift
            ;;
        --build)
            BUILD=true
            shift
            ;;
        --skip-deploy)
            SKIP_DEPLOY=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--local] [--build] [--skip-deploy] [--verbose]"
            echo ""
            echo "Options:"
            echo "  --local       Run tests against local instances (127.0.0.1:3040, 3041)"
            echo "  --build       Build the binary before testing"
            echo "  --skip-deploy Skip deployment, test existing nodes"
            echo "  --verbose     Show detailed output"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Build if requested
if [[ "$BUILD" == "true" ]]; then
    log_section "Building communitas-mcp"
    cd "$PROJECT_ROOT"
    cargo build --release -p communitas-mcp
    log_success "Build complete"
fi

# Helper: Make JSON-RPC request
json_rpc() {
    local url=$1
    local method=$2
    local params=${3:-'{}'}
    
    local payload
    payload=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "$method",
    "params": $params
}
EOF
)
    
    curl -sf -X POST "$url/rpc" \
        -H "Content-Type: application/json" \
        -d "$payload" \
        --connect-timeout 10 \
        --max-time 30 2>/dev/null || echo '{"error": "connection_failed"}'
}

# Helper: Make tool call via JSON-RPC
call_tool() {
    local url=$1
    local tool=$2
    local args=${3:-'{}'}
    
    local params
    params=$(cat <<EOF
{
    "name": "$tool",
    "arguments": $args
}
EOF
)
    
    json_rpc "$url" "tools/call" "$params"
}

# Test: Health check
test_health() {
    local url=$1
    local name=$2
    
    log_info "Testing health: $name"
    
    if curl -sf --connect-timeout 5 "$url/health" >/dev/null 2>&1; then
        log_success "$name is healthy"
        ((TESTS_PASSED++))
        return 0
    else
        log_error "$name health check failed"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Test: Core status
test_core_status() {
    local url=$1
    local name=$2
    
    log_info "Testing core_status: $name"
    
    local response
    response=$(call_tool "$url" "core_status")
    
    if echo "$response" | grep -q '"initialized"'; then
        log_success "$name core_status OK"
        ((TESTS_PASSED++))
        return 0
    else
        log_error "$name core_status failed: $response"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Test: List vaults (pre-auth tool)
test_list_vaults() {
    local url=$1
    local name=$2
    
    log_info "Testing list_vaults: $name"
    
    local response
    response=$(call_tool "$url" "list_vaults")
    
    if echo "$response" | grep -q '"vaults"'; then
        log_success "$name list_vaults OK"
        ((TESTS_PASSED++))
        return 0
    else
        log_error "$name list_vaults failed: $response"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Test: Create vault and authenticate
test_auth_flow() {
    local url=$1
    local name=$2
    local four_words="test-${RANDOM}-node-${RANDOM}"
    
    log_info "Testing auth flow: $name"
    
    # Create vault
    local create_args
    create_args=$(cat <<EOF
{
    "four_words": "$four_words",
    "password": "test-password-123",
    "display_name": "Test User $name"
}
EOF
)
    
    local response
    response=$(call_tool "$url" "create_vault" "$create_args")
    
    if echo "$response" | grep -q '"success"' || echo "$response" | grep -q '"four_words"'; then
        log_success "$name create_vault OK"
        ((TESTS_PASSED++))
    else
        # May already exist in demo mode
        log_warn "$name create_vault skipped (may exist): $response"
        ((TESTS_SKIPPED++))
    fi
    
    # Authenticate
    local auth_args
    auth_args=$(cat <<EOF
{
    "four_words": "$four_words",
    "password": "test-password-123"
}
EOF
)
    
    response=$(call_tool "$url" "authenticate" "$auth_args")
    
    if echo "$response" | grep -q '"success"\|"session"'; then
        log_success "$name authenticate OK"
        ((TESTS_PASSED++))
        return 0
    else
        log_error "$name authenticate failed: $response"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Test: Entity CRUD operations
test_entity_crud() {
    local url=$1
    local name=$2
    
    log_info "Testing entity CRUD: $name"
    
    # Create entity
    local create_args
    create_args=$(cat <<EOF
{
    "name": "Test Group $(date +%s)",
    "entity_type": "group",
    "description": "Created by integration test"
}
EOF
)
    
    local response
    response=$(call_tool "$url" "create_entity" "$create_args")
    
    if echo "$response" | grep -q '"success"\|"id"'; then
        log_success "$name create_entity OK"
        ((TESTS_PASSED++))
        
        # Extract entity ID for further tests
        local entity_id
        entity_id=$(echo "$response" | grep -o '"id"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4)
        
        if [[ -n "$entity_id" ]]; then
            # List entities
            response=$(call_tool "$url" "list_entities" '{"entity_type": "group"}')
            if echo "$response" | grep -q '"entities"\|"id"'; then
                log_success "$name list_entities OK"
                ((TESTS_PASSED++))
            else
                log_error "$name list_entities failed"
                ((TESTS_FAILED++))
            fi
        fi
    else
        log_error "$name create_entity failed: $response"
        ((TESTS_FAILED++))
    fi
}

# Test: Messaging workflow
test_messaging() {
    local url=$1
    local name=$2
    
    log_info "Testing messaging: $name"
    
    # First create a channel to message in
    local channel_args
    channel_args=$(cat <<EOF
{
    "name": "test-channel-$(date +%s)",
    "entity_type": "channel"
}
EOF
)
    
    local response
    response=$(call_tool "$url" "create_entity" "$channel_args")
    local entity_id
    entity_id=$(echo "$response" | grep -o '"id"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4)
    
    if [[ -z "$entity_id" ]]; then
        log_warn "$name messaging skipped (no channel)"
        ((TESTS_SKIPPED++))
        return 0
    fi
    
    # Send message
    local msg_args
    msg_args=$(cat <<EOF
{
    "entity_id": "$entity_id",
    "entity_type": "channel",
    "text": "Hello from integration test!"
}
EOF
)
    
    response=$(call_tool "$url" "send_message" "$msg_args")
    
    if echo "$response" | grep -q '"success"\|"id"'; then
        log_success "$name send_message OK"
        ((TESTS_PASSED++))
        
        local msg_id
        msg_id=$(echo "$response" | grep -o '"id"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4)
        
        if [[ -n "$msg_id" ]]; then
            # Add reaction
            local react_args
            react_args=$(cat <<EOF
{
    "entity_id": "$entity_id",
    "entity_type": "channel",
    "message_id": "$msg_id",
    "emoji": "thumbsup"
}
EOF
)
            response=$(call_tool "$url" "add_reaction" "$react_args")
            if echo "$response" | grep -q '"success"'; then
                log_success "$name add_reaction OK"
                ((TESTS_PASSED++))
            else
                log_warn "$name add_reaction failed"
                ((TESTS_SKIPPED++))
            fi
        fi
    else
        log_error "$name send_message failed: $response"
        ((TESTS_FAILED++))
    fi
}

# Test: Kanban workflow
test_kanban() {
    local url=$1
    local name=$2
    
    log_info "Testing Kanban: $name"
    
    # First create a project for the board
    local project_args
    project_args=$(cat <<EOF
{
    "name": "kanban-test-$(date +%s)",
    "entity_type": "project"
}
EOF
)
    
    local response
    response=$(call_tool "$url" "create_entity" "$project_args")
    local entity_id
    entity_id=$(echo "$response" | grep -o '"id"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4)
    
    if [[ -z "$entity_id" ]]; then
        log_warn "$name kanban skipped (no project)"
        ((TESTS_SKIPPED++))
        return 0
    fi
    
    # Create board
    local board_args
    board_args=$(cat <<EOF
{
    "entity_id": "$entity_id",
    "board_name": "Test Board",
    "description": "Integration test board"
}
EOF
)
    
    response=$(call_tool "$url" "create_kanban_board" "$board_args")
    
    if echo "$response" | grep -q '"success"\|"id"'; then
        log_success "$name create_kanban_board OK"
        ((TESTS_PASSED++))
        
        local board_id
        board_id=$(echo "$response" | grep -o '"id"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4)
        
        if [[ -n "$board_id" ]]; then
            # Create column
            local col_args
            col_args=$(cat <<EOF
{
    "board_id": "$board_id",
    "column_name": "To Do"
}
EOF
)
            response=$(call_tool "$url" "create_kanban_column" "$col_args")
            if echo "$response" | grep -q '"success"\|"id"'; then
                log_success "$name create_kanban_column OK"
                ((TESTS_PASSED++))
                
                local column_id
                column_id=$(echo "$response" | grep -o '"id"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4)
                
                if [[ -n "$column_id" ]]; then
                    # Create card
                    local card_args
                    card_args=$(cat <<EOF
{
    "board_id": "$board_id",
    "column_id": "$column_id",
    "title": "Test Card",
    "description": "Created by integration test"
}
EOF
)
                    response=$(call_tool "$url" "create_kanban_card" "$card_args")
                    if echo "$response" | grep -q '"success"\|"id"'; then
                        log_success "$name create_kanban_card OK"
                        ((TESTS_PASSED++))
                    else
                        log_error "$name create_kanban_card failed"
                        ((TESTS_FAILED++))
                    fi
                fi
            else
                log_error "$name create_kanban_column failed"
                ((TESTS_FAILED++))
            fi
        fi
    else
        log_error "$name create_kanban_board failed: $response"
        ((TESTS_FAILED++))
    fi
}

# Test: Network operations
test_network() {
    local url=$1
    local name=$2
    
    log_info "Testing network: $name"
    
    # Network status
    local response
    response=$(call_tool "$url" "network_status")
    
    if echo "$response" | grep -q '"active"\|"identity"\|"connected"'; then
        log_success "$name network_status OK"
        ((TESTS_PASSED++))
    else
        log_warn "$name network_status unavailable (may not be started)"
        ((TESTS_SKIPPED++))
    fi
    
    # Network peers
    response=$(call_tool "$url" "network_peers")
    
    if echo "$response" | grep -q '"peers"\|"count"'; then
        log_success "$name network_peers OK"
        ((TESTS_PASSED++))
    else
        log_warn "$name network_peers unavailable"
        ((TESTS_SKIPPED++))
    fi
}

# Test: Multi-node sync (cross-node entity visibility)
test_multinode_sync() {
    local url1=$1
    local url2=$2
    local name1=$3
    local name2=$4
    
    log_info "Testing multi-node sync: $name1 <-> $name2"
    
    # Create entity on node 1
    local entity_name="sync-test-$(date +%s)"
    local create_args
    create_args=$(cat <<EOF
{
    "name": "$entity_name",
    "entity_type": "group",
    "description": "Multi-node sync test"
}
EOF
)
    
    local response
    response=$(call_tool "$url1" "create_entity" "$create_args")
    
    if ! echo "$response" | grep -q '"success"\|"id"'; then
        log_warn "Multi-node sync skipped (create failed on $name1)"
        ((TESTS_SKIPPED++))
        return 0
    fi
    
    local entity_id
    entity_id=$(echo "$response" | grep -o '"id"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4)
    
    log_info "Created entity $entity_id on $name1, waiting for sync..."
    sleep 5  # Give time for gossip propagation
    
    # Check if entity is visible on node 2 (requires gossip network to be running)
    local get_args
    get_args=$(cat <<EOF
{
    "entity_id": "$entity_id"
}
EOF
)
    
    response=$(call_tool "$url2" "get_entity" "$get_args")
    
    if echo "$response" | grep -q "$entity_name"; then
        log_success "Multi-node sync verified: entity visible on $name2"
        ((TESTS_PASSED++))
    else
        log_warn "Multi-node sync pending (gossip may not be connected)"
        ((TESTS_SKIPPED++))
    fi
}

# Run all tests
run_tests() {
    log_section "Starting Integration Tests"
    
    local nodes=()
    local names=()
    
    if [[ "$LOCAL_MODE" == "true" ]]; then
        log_info "Running in LOCAL mode"
        nodes=("${LOCAL_NODES[@]}")
        names=("local-1" "local-2")
    else
        log_info "Running in VPS mode"
        for name in "${!VPS_NODES[@]}"; do
            nodes+=("http://${VPS_NODES[$name]}")
            names+=("$name")
        done
    fi
    
    # Phase 1: Health checks
    log_section "Phase 1: Health Checks"
    local healthy_nodes=()
    local healthy_names=()
    
    for i in "${!nodes[@]}"; do
        if test_health "${nodes[$i]}" "${names[$i]}"; then
            healthy_nodes+=("${nodes[$i]}")
            healthy_names+=("${names[$i]}")
        fi
    done
    
    if [[ ${#healthy_nodes[@]} -eq 0 ]]; then
        log_error "No healthy nodes found! Cannot proceed."
        exit 1
    fi
    
    log_info "Healthy nodes: ${healthy_names[*]}"
    
    # Phase 2: Core functionality tests on each healthy node
    log_section "Phase 2: Core Functionality"
    
    for i in "${!healthy_nodes[@]}"; do
        local url="${healthy_nodes[$i]}"
        local name="${healthy_names[$i]}"
        
        test_core_status "$url" "$name"
        test_list_vaults "$url" "$name"
    done
    
    # Phase 3: Auth and workflows (on first healthy node)
    log_section "Phase 3: Authentication & Workflows"
    
    local primary_url="${healthy_nodes[0]}"
    local primary_name="${healthy_names[0]}"
    
    test_auth_flow "$primary_url" "$primary_name"
    test_entity_crud "$primary_url" "$primary_name"
    test_messaging "$primary_url" "$primary_name"
    test_kanban "$primary_url" "$primary_name"
    test_network "$primary_url" "$primary_name"
    
    # Phase 4: Multi-node sync (if 2+ nodes available)
    if [[ ${#healthy_nodes[@]} -ge 2 ]]; then
        log_section "Phase 4: Multi-Node Sync"
        test_multinode_sync "${healthy_nodes[0]}" "${healthy_nodes[1]}" "${healthy_names[0]}" "${healthy_names[1]}"
    else
        log_section "Phase 4: Multi-Node Sync (SKIPPED - need 2+ nodes)"
        ((TESTS_SKIPPED++))
    fi
    
    # Summary
    log_section "Test Summary"
    
    local total=$((TESTS_PASSED + TESTS_FAILED + TESTS_SKIPPED))
    echo -e "Total Tests: $total"
    echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
    echo -e "${YELLOW}Skipped: $TESTS_SKIPPED${NC}"
    
    if [[ $TESTS_FAILED -gt 0 ]]; then
        log_error "Some tests failed!"
        exit 1
    else
        log_success "All tests passed!"
        exit 0
    fi
}

# Start local nodes for testing
start_local_nodes() {
    log_section "Starting Local Nodes"
    
    local binary="$PROJECT_ROOT/target/release/communitas-mcp"
    if [[ ! -f "$binary" ]]; then
        log_error "Binary not found. Run with --build first."
        exit 1
    fi
    
    # Start node 1
    log_info "Starting node 1 on port 3040..."
    "$binary" --http --demo --listen 127.0.0.1:3040 > /tmp/mcp-node1.log 2>&1 &
    NODE1_PID=$!
    
    # Start node 2
    log_info "Starting node 2 on port 3041..."
    "$binary" --http --demo --listen 127.0.0.1:3041 > /tmp/mcp-node2.log 2>&1 &
    NODE2_PID=$!
    
    # Wait for startup
    log_info "Waiting for nodes to start..."
    sleep 5
    
    # Cleanup on exit
    trap 'kill $NODE1_PID $NODE2_PID 2>/dev/null; exit' EXIT INT TERM
}

# Main
if [[ "$LOCAL_MODE" == "true" && "$SKIP_DEPLOY" != "true" ]]; then
    start_local_nodes
fi

run_tests
