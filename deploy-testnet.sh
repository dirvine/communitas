#!/bin/bash

# Communitas Testnet Deployment Script (headless only)
# Deploys 6 headless nodes across DigitalOcean regions using communitas-headless

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

# Cloud-init user-data (reads from repo file)
USER_DATA=$(cat cloud-init.yml)

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

    # Copy systemd service (headless)
    doctl compute scp deployment/systemd/communitas.service "$DROPLET_ID":/etc/systemd/system/

    echo -e "${GREEN}✅ Services deployed to $NAME${NC}"
done

echo -e "${GREEN}📝 Collecting node endpoints (random ports)...${NC}"
BOOTSTRAP_FILE="bootstrap.toml"
echo "# Generated: $(date)" > "$BOOTSTRAP_FILE"
echo "seeds = [" >> "$BOOTSTRAP_FILE"
for i in "${!DROPLET_IDS[@]}"; do
    DROPLET_ID=${DROPLET_IDS[$i]}
    REGION=${REGIONS[$i]}
    IP=$(doctl compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)
    PORT=$(doctl compute ssh "$DROPLET_ID" --ssh-command "grep -oE '[0-9]+' /etc/communitas-port.env | head -n1" 2>/dev/null)
    echo "Node $((i+1)) ($REGION): $IP:$PORT"
    echo "  \"$IP:$PORT\"," >> "$BOOTSTRAP_FILE"
done
echo "]" >> "$BOOTSTRAP_FILE"

# Deploy bootstrap config and start services
echo -e "${GREEN}🚀 Starting services on droplets...${NC}"
for i in "${!DROPLET_IDS[@]}"; do
    DROPLET_ID=${DROPLET_IDS[$i]}
    NAME="communitas-node-$((i+1))"

    # Start services
    doctl compute ssh "$DROPLET_ID" --ssh-command "sudo systemctl daemon-reload"
    doctl compute ssh "$DROPLET_ID" --ssh-command "sudo systemctl enable communitas"
    doctl compute ssh "$DROPLET_ID" --ssh-command "sudo systemctl start communitas"

    echo -e "${GREEN}✅ Services started on $NAME${NC}"
done

echo ""
echo -e "${GREEN}🎉 Testnet deployment complete!${NC}"
echo ""
echo -e "${BLUE}📋 Bootstrap seeds (ip:port):${NC}"
cat "$BOOTSTRAP_FILE"

echo ""
echo -e "${YELLOW}💡 Next steps:${NC}"
echo "1. ✅ Testnet deployed and running"
echo "2. Monitor logs: doctl compute ssh <droplet-id> --ssh-command 'journalctl -u communitas -f'"
echo "3. Update local bootstrap.toml with these addresses"
echo "4. Run local Communitas (GUI) and pin raw SPKI keys"
