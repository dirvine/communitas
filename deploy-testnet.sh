#!/bin/bash

# Communitas Testnet Deployment Script
# Deploys 6 headless nodes across DigitalOcean regions

set -e

# Configuration
REGIONS=("ams3" "lon1" "fra1" "nyc3" "sfo3" "sgp1")
SIZE="s-1vcpu-2gb"
IMAGE="ubuntu-24-04-x64"
SSH_KEY_ID="48810465"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Starting Communitas Testnet Deployment${NC}"
echo "Regions: ${REGIONS[*]}"
echo "Size: $SIZE"
echo ""

# Check prerequisites
if ! command -v doctl &> /dev/null; then
    echo -e "${RED}❌ doctl not found. Please install DigitalOcean CLI${NC}"
    exit 1
fi

# Check if we can access DigitalOcean API
echo -e "${YELLOW}🔍 Checking DigitalOcean API access...${NC}"
if ! doctl account get &>/dev/null; then
    echo -e "${RED}❌ Cannot access DigitalOcean API. Please check your access token.${NC}"
    echo -e "${YELLOW}💡 Run: doctl auth init${NC}"
    echo -e "${YELLOW}💡 Or set: export DOCTL_ACCESS_TOKEN=your_token${NC}"
    exit 1
fi
echo -e "${GREEN}✅ DigitalOcean API access confirmed${NC}"

# Note: Binaries will be built on the droplets themselves via cloud-init
echo -e "${GREEN}📦 Binaries will be built on droplets via cloud-init${NC}"

# Create droplets
DROPLET_IDS=()
FOUR_WORD_ADDRS=()

for i in "${!REGIONS[@]}"; do
    REGION=${REGIONS[$i]}
    NAME="communitas-node-$((i+1))"

    echo -e "${GREEN}🌊 Creating droplet in $REGION...${NC}"

# Cloud-init user-data
USER_DATA=$(cat <<EOF
#cloud-config
package_update: true
package_upgrade: true
packages:
  - curl
  - build-essential
  - pkg-config
  - libssl-dev
users:
  - name: communitas
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    ssh_authorized_keys:
      - $(cat ~/.ssh/id_rsa.pub 2>/dev/null || echo "")
runcmd:
  - sysctl -w net.core.rmem_max=2500000
  - ufw allow 443/udp
  - ufw allow 443/tcp
  - ufw allow 22/tcp
  - ufw --force enable
  - mkdir -p /opt/communitas/bin /var/lib/communitas
  - chown communitas:communitas /opt/communitas /var/lib/communitas
  - printf '%s\n' "[update]\nchannel=stable\n" > /etc/communitas/update.toml
  # Install Rust
  - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  - source ~/.cargo/env
  - export PATH="\$HOME/.cargo/bin:\$PATH"
  # Clone and build communitas
  - cd /tmp
  - git clone https://github.com/dirvine/communitas-foundation.git communitas
  - cd communitas
  - cargo build --release --bin communitas-node
  - cargo build --release --bin communitas-autoupdater
  - cargo build --release --bin bootstrap
  - cp target/release/communitas-node /opt/communitas/bin/
  - cp target/release/communitas-autoupdater /opt/communitas/bin/
  - cp target/release/bootstrap /opt/communitas/bin/
  - chown communitas:communitas /opt/communitas/bin/*
  - chmod +x /opt/communitas/bin/*
EOF
)

    # Create droplet
    DROPLET_ID=$(doctl compute droplet create "$NAME" \
        --region "$REGION" \
        --size "$SIZE" \
        --image "$IMAGE" \
        --ssh-keys "$SSH_KEY_ID" \
        --user-data "$USER_DATA" \
        --wait \
        --format ID \
        --no-header)

    DROPLET_IDS+=("$DROPLET_ID")
    echo "Created droplet $NAME with ID: $DROPLET_ID"
done

echo -e "${GREEN}⏳ Waiting for droplets to be ready...${NC}"
sleep 60

# Deploy systemd services to droplets
echo -e "${GREEN}📦 Deploying systemd services to droplets...${NC}"
for i in "${!DROPLET_IDS[@]}"; do
    DROPLET_ID=${DROPLET_IDS[$i]}
    REGION=${REGIONS[$i]}
    NAME="communitas-node-$((i+1))"

    IP=$(doctl compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)
    echo -e "${YELLOW}Deploying to $NAME ($IP)...${NC}"

    # Copy systemd services
    doctl compute scp deployment/systemd/communitas.service "$DROPLET_ID":/etc/systemd/system/
    doctl compute scp deployment/systemd/communitas-updater.service "$DROPLET_ID":/etc/systemd/system/

    echo -e "${GREEN}✅ Services deployed to $NAME${NC}"
done

# Get IPs and generate four-word addresses
echo -e "${GREEN}📝 Generating four-word addresses...${NC}"
for i in "${!DROPLET_IDS[@]}"; do
    DROPLET_ID=${DROPLET_IDS[$i]}
    REGION=${REGIONS[$i]}

    IP=$(doctl compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)

    # Generate four-word address
    WORDS=$(./tools/fwaddr/target/release/fwaddr "$IP" 443)
    FOUR_WORD_ADDRS+=("$WORDS")

    echo "Node $((i+1)) ($REGION): $IP -> $WORDS"
done

# Create bootstrap.toml
echo -e "${GREEN}📄 Creating bootstrap.toml...${NC}"
BOOTSTRAP_FILE="bootstrap.toml"
cat > "$BOOTSTRAP_FILE" << EOF
# Communitas Testnet Bootstrap Configuration
# Generated: $(date)

seeds = [
EOF

for addr in "${FOUR_WORD_ADDRS[@]}"; do
    echo "  \"$addr:443\"," >> "$BOOTSTRAP_FILE"
done

cat >> "$BOOTSTRAP_FILE" << EOF
]
EOF

# Deploy bootstrap config and start services
echo -e "${GREEN}🚀 Starting services on droplets...${NC}"
for i in "${!DROPLET_IDS[@]}"; do
    DROPLET_ID=${DROPLET_IDS[$i]}
    NAME="communitas-node-$((i+1))"

    # Upload bootstrap config
    doctl compute scp "$BOOTSTRAP_FILE" "$DROPLET_ID":/opt/communitas/bootstrap.toml

    # Start services
    doctl compute ssh "$DROPLET_ID" --ssh-command "sudo systemctl daemon-reload"
    doctl compute ssh "$DROPLET_ID" --ssh-command "sudo systemctl enable communitas"
    doctl compute ssh "$DROPLET_ID" --ssh-command "sudo systemctl start communitas"
    doctl compute ssh "$DROPLET_ID" --ssh-command "sudo systemctl enable communitas-updater"
    doctl compute ssh "$DROPLET_ID" --ssh-command "sudo systemctl start communitas-updater"

    echo -e "${GREEN}✅ Services started on $NAME${NC}"
done

echo ""
echo -e "${GREEN}🎉 Testnet deployment complete!${NC}"
echo ""
echo -e "${BLUE}📋 Bootstrap addresses:${NC}"
for i in "${!FOUR_WORD_ADDRS[@]}"; do
    REGION=${REGIONS[$i]}
    WORDS=${FOUR_WORD_ADDRS[$i]}
    echo "  $REGION: $WORDS:443"
done

echo ""
echo -e "${YELLOW}💡 Next steps:${NC}"
echo "1. ✅ Testnet deployed and running"
echo "2. Monitor logs: doctl compute ssh <droplet-id> --ssh-command 'journalctl -u communitas -f'"
echo "3. Test connectivity between nodes"
echo "4. Update local bootstrap.toml with these addresses"
echo "5. Run local communitas instances for testing"