#!/bin/bash

# Communitas Local Testnet Setup Verification
# This script helps verify that your local testnet is working correctly

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Docker is running
check_docker() {
    log_info "Checking Docker..."
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker first."
        exit 1
    fi
    log_info "Docker is running"
}

# Check if bootstrap node is running
check_bootstrap() {
    log_info "Checking bootstrap node..."
    if docker ps | grep -q communitas-bootstrap; then
        log_info "Bootstrap node is running"
        return 0
    else
        log_warn "Bootstrap node is not running"
        return 1
    fi
}

# Test bootstrap connectivity
test_bootstrap_connectivity() {
    log_info "Testing bootstrap connectivity..."
    if nc -z localhost 9001 2>/dev/null; then
        log_info "Bootstrap node is accessible on port 9001"
        return 0
    else
        log_error "Cannot connect to bootstrap node on port 9001"
        return 1
    fi
}

# Check environment variables
check_environment() {
    log_info "Checking environment variables..."
    if [[ -n "${COMMUNITAS_LOCAL_BOOTSTRAP}" ]]; then
        log_info "COMMUNITAS_LOCAL_BOOTSTRAP is set to: $COMMUNITAS_LOCAL_BOOTSTRAP"
    else
        log_warn "COMMUNITAS_LOCAL_BOOTSTRAP is not set. Using production bootstrap."
    fi
}

# Test application health
test_app_health() {
    log_info "Testing application health endpoint..."
    if curl -f http://localhost:1420/health > /dev/null 2>&1; then
        log_info "Application health endpoint is responding"
        return 0
    else
        log_warn "Application health endpoint is not responding"
        return 1
    fi
}

# Show network status
show_network_status() {
    log_info "Network Status:"
    echo "  Bootstrap Node: $(docker ps | grep communitas-bootstrap | wc -l) running"
    echo "  Environment Variables:"
    echo "    COMMUNITAS_LOCAL_BOOTSTRAP: ${COMMUNITAS_LOCAL_BOOTSTRAP:-'Not set'}"
    echo "    COMMUNITAS_DATA_DIR: ${COMMUNITAS_DATA_DIR:-'./communitas-data'}"
    echo "    RUST_LOG: ${RUST_LOG:-'info'}"
}

# Main verification process
main() {
    log_info "Starting Communitas local testnet verification..."

    check_docker
    check_environment
    show_network_status

    if check_bootstrap; then
        test_bootstrap_connectivity
    else
        log_warn "Bootstrap node is not running. Here's how to start it:"
        echo ""
        echo "  # Using Docker:"
        echo "  docker run -d --name communitas-bootstrap -p 9001:9001 -p 9100:9100 -p 9110:9110 -p 9120:9120 communitas-bootstrap"
        echo ""
        echo "  # Using local build:"
        echo "  cargo run --bin bootstrap -- --port 9001"
        echo ""
    fi

    if test_app_health; then
        log_info "✅ Local testnet setup looks good!"
        log_info "You can now:"
        echo "  1. Open the application in your browser"
        echo "  2. Check the health endpoint: curl http://localhost:1420/health"
        echo "  3. Monitor logs: docker-compose logs -f"
    else
        log_warn "Application is not responding. Make sure it's running:"
        echo "  npm run tauri dev"
    fi

    log_info "Verification complete!"
}

# Handle command line arguments
case "${1:-}" in
    "bootstrap")
        log_info "Starting bootstrap node..."
        docker run -d \
            --name communitas-bootstrap \
            -p 9001:9001 \
            -p 9100:9100 \
            -p 9110:9110 \
            -p 9120:9120 \
            -e RUST_LOG=info \
            communitas-bootstrap:latest
        ;;
    "stop")
        log_info "Stopping bootstrap node..."
        docker stop communitas-bootstrap
        docker rm communitas-bootstrap
        ;;
    "logs")
        log_info "Showing bootstrap logs..."
        docker logs -f communitas-bootstrap
        ;;
    *)
        main
        ;;
esac