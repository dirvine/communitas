#!/bin/bash
#
# Bootstrap Node Provisioning Script
#
# This script provisions a DigitalOcean droplet to run communitas-headless
# as a bootstrap node with automatic updates and systemd service management.
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/dirvine/communitas/main/scripts/provision-bootstrap.sh | bash
#
# Or with environment variables:
#   GITHUB_REPO=dirvine/communitas \
#   BOOTSTRAP_PEERS="" \
#   ./scripts/provision-bootstrap.sh
#
# Environment Variables:
#   GITHUB_REPO  - GitHub repository (default: dirvine/communitas)
#   BOOTSTRAP_PEERS - Comma-separated bootstrap four-word addresses
#   AUTO_UPDATE  - Enable automatic updates (default: true)
#   ENABLE_METRICS - Enable metrics endpoint (default: true)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
GITHUB_REPO="${GITHUB_REPO:-dirvine/communitas}"
BOOTSTRAP_PEERS="${BOOTSTRAP_PEERS:-}"
AUTO_UPDATE="${AUTO_UPDATE:-true}"
ENABLE_METRICS="${ENABLE_METRICS:-true}"
INSTALL_DIR="/opt/communitas"
DATA_DIR="/var/lib/communitas"
USER="communitas"
GROUP="communitas"

echo -e "${GREEN}═══════════════════════════════════════════${NC}"
echo -e "${GREEN}  Communitas Bootstrap Node Provisioning${NC}"
echo -e "${GREEN}═══════════════════════════════════════════${NC}"
echo ""
echo "Configuration:"
echo "  GitHub Repository: ${GITHUB_REPO}"
echo "  Install Directory: ${INSTALL_DIR}"
echo "  Data Directory: ${DATA_DIR}"
echo "  Service User: ${USER}"
echo "  Auto-Update: ${AUTO_UPDATE}"
echo "  Metrics: ${ENABLE_METRICS}"
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root${NC}"
   exit 1
fi

echo -e "${YELLOW}[1/7] Installing system dependencies...${NC}"
apt-get update -qq
apt-get install -y -qq curl tar jq systemd > /dev/null 2>&1
echo -e "${GREEN}✓ Dependencies installed${NC}"

echo -e "${YELLOW}[2/7] Creating service user and directories...${NC}"
# Create group if it doesn't exist
if ! getent group "${GROUP}" &>/dev/null; then
    groupadd --system "${GROUP}"
    echo -e "${GREEN}✓ Created group: ${GROUP}${NC}"
else
    echo -e "${GREEN}✓ Group already exists: ${GROUP}${NC}"
fi

# Create user if it doesn't exist
if ! id "${USER}" &>/dev/null; then
    useradd --system --no-create-home --shell /bin/false --gid "${GROUP}" "${USER}"
    echo -e "${GREEN}✓ Created user: ${USER}${NC}"
else
    CURRENT_GROUP=$(id -gn "${USER}")
    if [[ "${CURRENT_GROUP}" != "${GROUP}" ]]; then
        usermod -g "${GROUP}" "${USER}"
        echo -e "${GREEN}✓ Updated user primary group to: ${GROUP}${NC}"
    else
        echo -e "${GREEN}✓ User already exists: ${USER}${NC}"
    fi
fi

# Create directories with proper permissions
mkdir -p "${INSTALL_DIR}"
mkdir -p "${DATA_DIR}"
chown -R "${USER}:${GROUP}" "${INSTALL_DIR}"
chown -R "${USER}:${GROUP}" "${DATA_DIR}"
chmod 755 "${INSTALL_DIR}"
chmod 700 "${DATA_DIR}"
echo -e "${GREEN}✓ Directories created and secured${NC}"

echo -e "${YELLOW}[3/7] Downloading latest communitas-headless binary...${NC}"
DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/communitas-headless-x86_64-unknown-linux-gnu.tar.gz"
echo "  URL: ${DOWNLOAD_URL}"

# Download with retry
MAX_RETRIES=3
RETRY=0
while [ $RETRY -lt $MAX_RETRIES ]; do
    if curl -fsSL "${DOWNLOAD_URL}" | tar -xz -C "${INSTALL_DIR}"; then
        echo -e "${GREEN}✓ Binary downloaded and extracted${NC}"
        break
    else
        RETRY=$((RETRY + 1))
        if [ $RETRY -eq $MAX_RETRIES ]; then
            echo -e "${RED}Error: Failed to download binary after ${MAX_RETRIES} attempts${NC}"
            exit 1
        fi
        echo -e "${YELLOW}  Retry ${RETRY}/${MAX_RETRIES}...${NC}"
        sleep 5
    fi
done

# Make binary executable
chmod +x "${INSTALL_DIR}/communitas-headless"

# Verify binary
if ! "${INSTALL_DIR}/communitas-headless" --version > /dev/null 2>&1; then
    echo -e "${RED}Error: Binary verification failed${NC}"
    exit 1
fi

VERSION=$("${INSTALL_DIR}/communitas-headless" --version | head -n1)
echo -e "${GREEN}✓ Binary verified: ${VERSION}${NC}"

echo -e "${YELLOW}[4/7] Creating systemd service...${NC}"
cat > /etc/systemd/system/communitas-bootstrap.service <<EOF
[Unit]
Description=Communitas Bootstrap Node
Documentation=https://communitas.life
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${USER}
Group=${GROUP}
WorkingDirectory=${DATA_DIR}

# Environment
Environment="RUST_LOG=info"
Environment="RUST_BACKTRACE=1"

# Command with auto-update and metrics
ExecStart=${INSTALL_DIR}/communitas-headless \\
    --storage ${DATA_DIR} \\
    --listen 0.0.0.0:8080 \\
    --metrics \\
    --metrics-addr 0.0.0.0:9600

# Restart policy
Restart=always
RestartSec=10
StartLimitIntervalSec=300
StartLimitBurst=5

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${DATA_DIR}
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF

chmod 644 /etc/systemd/system/communitas-bootstrap.service
echo -e "${GREEN}✓ Systemd service created${NC}"

echo -e "${YELLOW}[5/7] Creating auto-update timer...${NC}"
if [[ "${AUTO_UPDATE}" == "true" ]]; then
    # Create update script
    cat > "${INSTALL_DIR}/update.sh" <<'UPDATEEOF'
#!/bin/bash
set -euo pipefail

INSTALL_DIR="/opt/communitas"
GITHUB_REPO="${GITHUB_REPO:-dirvine/communitas}"
DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/communitas-headless-x86_64-unknown-linux-gnu.tar.gz"

# Check current version
CURRENT_VERSION=$("${INSTALL_DIR}/communitas-headless" --version 2>/dev/null | head -n1 || echo "unknown")

# Download new version to temp
TMP_DIR=$(mktemp -d)
trap "rm -rf ${TMP_DIR}" EXIT

if curl -fsSL "${DOWNLOAD_URL}" | tar -xz -C "${TMP_DIR}"; then
    NEW_VERSION=$("${TMP_DIR}/communitas-headless" --version 2>/dev/null | head -n1 || echo "unknown")

    if [[ "${CURRENT_VERSION}" != "${NEW_VERSION}" ]]; then
        echo "Updating from ${CURRENT_VERSION} to ${NEW_VERSION}"

        # Backup current binary
        cp "${INSTALL_DIR}/communitas-headless" "${INSTALL_DIR}/communitas-headless.backup"

        # Install new binary
        cp "${TMP_DIR}/communitas-headless" "${INSTALL_DIR}/communitas-headless"
        chmod +x "${INSTALL_DIR}/communitas-headless"

        # Restart service
        systemctl restart communitas-bootstrap.service

        echo "Update successful"
    else
        echo "Already on latest version: ${CURRENT_VERSION}"
    fi
else
    echo "Failed to download update"
    exit 1
fi
UPDATEEOF

    chmod +x "${INSTALL_DIR}/update.sh"

    # Create systemd timer
    cat > /etc/systemd/system/communitas-update.timer <<EOF
[Unit]
Description=Communitas Auto-Update Timer
Requires=communitas-update.service

[Timer]
OnBootSec=10min
OnUnitActiveSec=6h

[Install]
WantedBy=timers.target
EOF

    cat > /etc/systemd/system/communitas-update.service <<EOF
[Unit]
Description=Communitas Auto-Update Service
After=network-online.target

[Service]
Type=oneshot
ExecStart=${INSTALL_DIR}/update.sh
User=root
StandardOutput=journal
StandardError=journal
EOF

    systemctl daemon-reload
    systemctl enable communitas-update.timer
    echo -e "${GREEN}✓ Auto-update timer enabled (every 6 hours)${NC}"
else
    echo -e "${YELLOW}✓ Auto-update disabled${NC}"
fi

echo -e "${YELLOW}[6/7] Starting service...${NC}"
systemctl daemon-reload
systemctl enable communitas-bootstrap.service
systemctl start communitas-bootstrap.service

# Wait for service to start
sleep 5

if systemctl is-active --quiet communitas-bootstrap.service; then
    echo -e "${GREEN}✓ Service started successfully${NC}"
else
    echo -e "${RED}Error: Service failed to start${NC}"
    systemctl status communitas-bootstrap.service --no-pager
    exit 1
fi

echo -e "${YELLOW}[7/7] Gathering bootstrap information...${NC}"

# Get public IP
PUBLIC_IP=$(curl -s https://api.ipify.org || echo "unknown")

# Get four-word address (from logs after node starts)
sleep 3
FOUR_WORDS=$(journalctl -u communitas-bootstrap.service --no-pager -n 100 | grep -oP 'Four-word address: \K[^\s]+' | tail -1 || echo "check-logs")

echo ""
echo -e "${GREEN}═══════════════════════════════════════════${NC}"
echo -e "${GREEN}  Bootstrap Node Provisioned Successfully${NC}"
echo -e "${GREEN}═══════════════════════════════════════════${NC}"
echo ""
echo "Bootstrap Endpoint Information:"
echo "  Public IP: ${PUBLIC_IP}:8080"
echo "  Four-Word Address: ${FOUR_WORDS}"
echo "  Metrics: http://${PUBLIC_IP}:9600/metrics"
echo ""
echo "Service Management:"
echo "  Status: systemctl status communitas-bootstrap"
echo "  Logs: journalctl -u communitas-bootstrap -f"
echo "  Restart: systemctl restart communitas-bootstrap"
echo ""
echo "Firewall Configuration:"
echo "  ufw allow 8080/tcp  # P2P traffic"
echo "  ufw allow 9600/tcp  # Metrics (optional)"
echo ""
echo -e "${YELLOW}⚠️  Remember to configure your firewall!${NC}"
echo ""

# Save bootstrap info to file
cat > "${INSTALL_DIR}/bootstrap-info.txt" <<EOF
Bootstrap Node Information
Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")

Public IP: ${PUBLIC_IP}
Port: 8080
Four-Word Address: ${FOUR_WORDS}
Full Endpoint: ${PUBLIC_IP}:8080

Metrics Endpoint: http://${PUBLIC_IP}:9600/metrics

Service: communitas-bootstrap.service
Data Directory: ${DATA_DIR}
Binary: ${INSTALL_DIR}/communitas-headless
Version: ${VERSION}
EOF

echo -e "${GREEN}Bootstrap info saved to: ${INSTALL_DIR}/bootstrap-info.txt${NC}"
echo ""
