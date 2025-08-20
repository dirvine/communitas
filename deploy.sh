#!/bin/bash

# Communitas Production Deployment Script
# Enhanced security and automated deployment

set -euo pipefail

# Configuration
APP_NAME="communitas"
VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.1.0")
BUILD_DIR="target/release"
DEPLOY_DIR="/opt/communitas"
DATA_DIR="/var/lib/communitas"
LOG_DIR="/var/log/communitas"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   log_error "This script should not be run as root"
   exit 1
fi

# Pre-deployment checks
pre_deployment_checks() {
    log_info "Running pre-deployment checks..."

    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo is not installed"
        exit 1
    fi

    # Check if system dependencies are installed
    local deps=("libwebkit2gtk-4.0-dev" "libgtk-3-dev" "libayatana-appindicator3-dev" "librsvg2-dev")
    for dep in "${deps[@]}"; do
        if ! dpkg -l | grep -q "$dep"; then
            log_warn "Missing dependency: $dep"
        fi
    done

    # Check available disk space
    local free_space=$(df -BG . | awk 'NR==2 {print $4}' | sed 's/G//')
    if [[ $free_space -lt 5 ]]; then
        log_error "Insufficient disk space. Need at least 5GB free"
        exit 1
    fi

    log_info "Pre-deployment checks completed"
}

# Build application
build_application() {
    log_info "Building application..."

    # Clean previous build
    cargo clean

    # Build with security optimizations
    RUSTFLAGS="-C target-cpu=generic -C opt-level=3 -C panic=abort" \
    cargo build --release --locked

    if [[ ! -f "$BUILD_DIR/$APP_NAME" ]]; then
        log_error "Build failed - binary not found"
        exit 1
    fi

    log_info "Application built successfully"
}

# Security checks
security_checks() {
    log_info "Running security checks..."

    # Check for unwrap() calls in production code
    local unwrap_count=$(find src-tauri/src -name "*.rs" -not -path "*/tests/*" -not -path "*test*.rs" -exec grep -c "\.unwrap()" {} \; | awk '{sum+=$1} END {print sum}')
    if [[ $unwrap_count -gt 0 ]]; then
        log_warn "Found $unwrap_count unwrap() calls in production code"
    fi

    # Check for expect() calls in production code
    local expect_count=$(find src-tauri/src -name "*.rs" -not -path "*/tests/*" -not -path "*test*.rs" -exec grep -c "\.expect(" {} \; | awk '{sum+=$1} END {print sum}')
    if [[ $expect_count -gt 0 ]]; then
        log_warn "Found $expect_count expect() calls in production code"
    fi

    # Check binary security
    if command -v hardentools &> /dev/null; then
        log_info "Running binary hardening check..."
        hardentools "$BUILD_DIR/$APP_NAME" || log_warn "Binary hardening check failed"
    fi

    log_info "Security checks completed"
}

# Run tests
run_tests() {
    log_info "Running tests..."

    # Run unit tests
    if ! cargo test --lib --release; then
        log_error "Unit tests failed"
        exit 1
    fi

    # Run integration tests if they exist
    if [[ -d "tests" ]]; then
        if ! cargo test integration_ --release; then
            log_warn "Integration tests failed - continuing with deployment"
        fi
    fi

    log_info "Tests completed"
}

# Deploy application
deploy_application() {
    log_info "Deploying application..."

    # Create directories
    sudo mkdir -p "$DEPLOY_DIR"
    sudo mkdir -p "$DATA_DIR"
    sudo mkdir -p "$LOG_DIR"

    # Set permissions
    sudo chown -R $USER:$USER "$DEPLOY_DIR"
    sudo chown -R $USER:$USER "$DATA_DIR"
    sudo chown -R $USER:$USER "$LOG_DIR"

    # Copy binary
    cp "$BUILD_DIR/$APP_NAME" "$DEPLOY_DIR/"

    # Create systemd service file
    create_systemd_service

    # Set executable permissions
    chmod +x "$DEPLOY_DIR/$APP_NAME"

    log_info "Application deployed to $DEPLOY_DIR"
}

# Create systemd service
create_systemd_service() {
    local service_file="/etc/systemd/system/communitas.service"

    sudo tee "$service_file" > /dev/null << EOF
[Unit]
Description=Communitas P2P Collaboration Platform
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$USER
Group=$USER
WorkingDirectory=$DEPLOY_DIR
ExecStart=$DEPLOY_DIR/$APP_NAME
Restart=always
RestartSec=10

# Environment variables
Environment=RUST_LOG=info,communitas=info,saorsa_core=warn
Environment=COMMUNITAS_DATA_DIR=$DATA_DIR

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$DATA_DIR $LOG_DIR

# Resource limits
MemoryMax=2G
CPUQuota=80%

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    log_info "Systemd service created"
}

# Post-deployment tasks
post_deployment_tasks() {
    log_info "Running post-deployment tasks..."

    # Create backup
    local backup_dir="backup-$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$backup_dir"
    cp "$BUILD_DIR/$APP_NAME" "$backup_dir/"
    log_info "Backup created in $backup_dir"

    # Generate deployment report
    cat > "deployment-report.txt" << EOF
Communitas Deployment Report
===========================
Version: $VERSION
Deployed: $(date)
User: $USER
Location: $DEPLOY_DIR
Data Directory: $DATA_DIR
Log Directory: $LOG_DIR

Build Information:
- Rust Version: $(rustc --version)
- Cargo Version: $(cargo --version)
- Target: $(rustc -vV | grep host | cut -d' ' -f2)

Security Status:
- Binary Hardening: $(command -v hardentools &> /dev/null && echo "Available" || echo "Not Available")
- Firewall: $(sudo ufw status | head -1)
- SELinux/AppArmor: $(command -v getenforce &> /dev/null && getenforce || echo "Not Available")

Next Steps:
1. Start service: sudo systemctl start communitas
2. Enable on boot: sudo systemctl enable communitas
3. Check status: sudo systemctl status communitas
4. View logs: journalctl -u communitas -f
5. Health check: curl http://localhost:1420/health
EOF

    log_info "Deployment report generated: deployment-report.txt"
}

# Main deployment process
main() {
    log_info "Starting Communitas deployment (v$VERSION)"

    pre_deployment_checks
    build_application
    security_checks
    run_tests
    deploy_application
    post_deployment_tasks

    log_info "Deployment completed successfully!"
    log_info "Run 'sudo systemctl start communitas' to start the service"
}

# Handle script arguments
case "${1:-}" in
    "check")
        pre_deployment_checks
        ;;
    "build")
        build_application
        ;;
    "test")
        run_tests
        ;;
    "deploy")
        deploy_application
        ;;
    *)
        main
        ;;
esac