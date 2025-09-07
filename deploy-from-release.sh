#!/bin/bash

# Communitas - Deploy from GitHub Release
# Downloads binaries from GitHub releases and deploys to DigitalOcean droplets

set -e

# Configuration
REPO="dirvine/communitas"
VERSION=${1:-"latest"}
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Droplet information (update these with your actual droplet IPs)
declare -A DROPLETS=(
    ["ams3"]="104.248.85.72"
    ["lon1"]="138.68.130.66"
    ["fra1"]="159.89.109.179"
    ["nyc3"]="165.22.44.216"
    ["sfo3"]="137.184.123.27"
    ["sgp1"]="128.199.85.70"
)

echo -e "${GREEN}🚀 Starting Communitas Deployment from GitHub Release${NC}"
echo "Repository: $REPO"
echo "Version: $VERSION"
echo ""

# Get the latest release information
if [ "$VERSION" = "latest" ]; then
    RELEASE_INFO=$(curl -s "https://api.github.com/repos/$REPO/releases/latest")
else
    RELEASE_INFO=$(curl -s "https://api.github.com/repos/$REPO/releases/tags/$VERSION")
fi

if [ -z "$RELEASE_INFO" ] || echo "$RELEASE_INFO" | grep -q "Not Found"; then
    echo -e "${RED}❌ Error: Could not find release $VERSION${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Found release information${NC}"

# Extract download URLs for different platforms
LINUX_URL=$(echo "$RELEASE_INFO" | grep -o 'https://github.com/[^"]*linux-x86_64.tar.gz' | head -1)
MACOS_URL=$(echo "$RELEASE_INFO" | grep -o 'https://github.com/[^"]*macos-universal.tar.gz' | head -1)
WINDOWS_URL=$(echo "$RELEASE_INFO" | grep -o 'https://github.com/[^"]*windows-x86_64.zip' | head -1)

echo "Download URLs:"
echo "  Linux: $LINUX_URL"
echo "  macOS: $MACOS_URL"
echo "  Windows: $WINDOWS_URL"
echo ""

# Function to deploy to a single droplet
deploy_to_droplet() {
    local region=$1
    local ip=$2

    echo -e "${YELLOW}📦 Deploying to $region ($ip)...${NC}"

    # Download and extract binaries on the droplet
    ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -i ~/.ssh/id_rsa communitas@$ip << EOF
        set -e
        echo "Downloading Linux binaries..."
        cd /tmp
        curl -L "$LINUX_URL" -o communitas-headless-linux-x86_64.tar.gz
        echo "Extracting binaries..."
        tar -xzf communitas-headless-linux-x86_64.tar.gz
        echo "Installing binaries..."
        sudo cp communitas-node /opt/communitas/bin/
        sudo cp bootstrap /opt/communitas/bin/
        sudo cp communitas-autoupdater /opt/communitas/bin/
        sudo chown communitas:communitas /opt/communitas/bin/*
        sudo chmod +x /opt/communitas/bin/*
        echo "Cleaning up..."
        rm -f communitas-headless-linux-x86_64.tar.gz communitas-node bootstrap communitas-autoupdater
        echo "✅ Deployment to $region complete"
EOF

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Successfully deployed to $region${NC}"
    else
        echo -e "${RED}❌ Failed to deploy to $region${NC}"
        return 1
    fi
}

# Deploy to all droplets
echo -e "${GREEN}🌊 Starting deployment to all droplets...${NC}"

failed_deployments=()
for region in "${!DROPLETS[@]}"; do
    ip=${DROPLETS[$region]}
    if ! deploy_to_droplet "$region" "$ip"; then
        failed_deployments+=("$region")
    fi
done

# Summary
echo ""
echo -e "${GREEN}📊 Deployment Summary:${NC}"
echo "Total droplets: ${#DROPLETS[@]}"
echo "Successful deployments: $((${#DROPLETS[@]} - ${#failed_deployments[@]}))"
echo "Failed deployments: ${#failed_deployments[@]}"

if [ ${#failed_deployments[@]} -gt 0 ]; then
    echo -e "${RED}Failed regions: ${failed_deployments[*]}${NC}"
    exit 1
else
    echo -e "${GREEN}🎉 All deployments successful!${NC}"
fi

echo ""
echo -e "${YELLOW}📝 Next steps:${NC}"
echo "1. Create bootstrap.toml with all node addresses"
echo "2. Configure systemd services on each droplet"
echo "3. Start the communitas services"
echo "4. Test peer connectivity and messaging"

echo ""
echo -e "${GREEN}✅ Deployment from GitHub release complete!${NC}"