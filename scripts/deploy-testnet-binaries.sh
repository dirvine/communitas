#!/bin/bash

# Deploy Communitas Testnet Binaries Script
# This script downloads binaries from GitHub releases and deploys them to Digital Ocean containers

set -e

# Configuration
GITHUB_REPO="dirvine/communitas-foundation"
RELEASE_TAG="${1:-latest}"
APP_ID="18ccbf3a-7111-4897-a0b6-f215910bcf1a"

# Node configuration
NODES=(
    "bootstrap-coordinator:8080"
    "node-nyc:8081"
    "node-sfo:8082"
    "node-lon:8083"
    "node-fra:8084"
    "node-sgp:8085"
    "node-syd:8086"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Starting Communitas Testnet Binary Deployment${NC}"
echo "=============================================="

# Function to get latest release if not specified
get_latest_release() {
    if [ "$RELEASE_TAG" = "latest" ]; then
        echo -e "${YELLOW}📦 Getting latest release...${NC}"
        RELEASE_TAG=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
        echo -e "${GREEN}📦 Using release: $RELEASE_TAG${NC}"
    fi
}

# Function to download binaries
download_binaries() {
    echo -e "${YELLOW}📥 Downloading binaries from GitHub...${NC}"

    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Download the release archive
    DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/download/$RELEASE_TAG/communitas-binaries-linux-x86_64.tar.gz"

    if curl -L -o "communitas-binaries.tar.gz" "$DOWNLOAD_URL"; then
        echo -e "${GREEN}✅ Downloaded binaries successfully${NC}"
    else
        echo -e "${RED}❌ Failed to download binaries from $DOWNLOAD_URL${NC}"
        exit 1
    fi

    # Extract binaries
    tar -xzf communitas-binaries.tar.gz
    echo -e "${GREEN}✅ Extracted binaries${NC}"

    # Verify binaries exist
    for binary in communitas-node bootstrap communitas-autoupdater; do
        if [ ! -f "$binary" ]; then
            echo -e "${RED}❌ Binary $binary not found in release${NC}"
            exit 1
        fi
    done

    echo "$TEMP_DIR"
}

# Function to get container SSH command
get_ssh_command() {
    local component=$1
    local port=$2

    # For Digital Ocean App Platform, we need to construct the SSH command
    # This assumes you have SSH access set up
    echo "ssh -i ~/.ssh/id_rsa root@app-$APP_ID-$component.ondigitalocean.app"
}

# Function to deploy to a single node
deploy_to_node() {
    local component=$1
    local port=$2
    local temp_dir=$3

    echo -e "${YELLOW}🚀 Deploying to $component (port: $port)...${NC}"

    # Copy binaries to container
    scp -i ~/.ssh/id_rsa "$temp_dir/communitas-node" "root@app-$APP_ID-$component.ondigitalocean.app:/root/"
    scp -i ~/.ssh/id_rsa "$temp_dir/bootstrap" "root@app-$APP_ID-$component.ondigitalocean.app:/root/"

    # SSH into container and set up
    ssh -i ~/.ssh/id_rsa "root@app-$APP_ID-$component.ondigitalocean.app" << EOF
        chmod +x /root/communitas-node /root/bootstrap
        echo "Binary deployment complete for $component"
EOF

    echo -e "${GREEN}✅ Deployed to $component${NC}"
}

# Function to collect node information
collect_node_info() {
    local component=$1
    local port=$2
    local temp_dir=$3

    echo -e "${YELLOW}📊 Collecting info from $component...${NC}"

    # Get internal IP from container
    local internal_ip=$(ssh -i ~/.ssh/id_rsa "root@app-$APP_ID-$component.ondigitalocean.app" "hostname -i | awk '{print \$1}'")

    echo "$component:$internal_ip:$port"
}

# Main deployment process
main() {
    get_latest_release

    # Download binaries
    TEMP_DIR=$(download_binaries)

    echo -e "${YELLOW}🔧 Deploying binaries to all nodes...${NC}"

    # Deploy to all nodes
    for node_info in "${NODES[@]}"; do
        IFS=':' read -r component port <<< "$node_info"
        deploy_to_node "$component" "$port" "$TEMP_DIR"
    done

    echo -e "${YELLOW}📊 Collecting node information...${NC}"

    # Collect node information
    NODE_LIST=()
    for node_info in "${NODES[@]}"; do
        IFS=':' read -r component port <<< "$node_info"
        node_data=$(collect_node_info "$component" "$port" "$TEMP_DIR")
        NODE_LIST+=("$node_data")
    done

    # Generate bootstrap configuration
    echo -e "${YELLOW}📝 Generating bootstrap configuration...${NC}"

    cat > bootstrap.toml << EOF
# Communitas Testnet Bootstrap Configuration
# Generated on $(date)

[network]
name = "communitas-testnet"
version = "$RELEASE_TAG"

[[nodes]]
name = "bootstrap-coordinator"
internal_ip = "$(echo "${NODE_LIST[0]}" | cut -d: -f2)"
port = $(echo "${NODE_LIST[0]}" | cut -d: -f3)
region = "nyc"
role = "bootstrap"

EOF

    # Add worker nodes
    for i in {1..6}; do
        node_data="${NODE_LIST[$i]}"
        IFS=':' read -r component ip port <<< "$node_data"
        region=$(echo "$component" | sed 's/node-//')

        cat >> bootstrap.toml << EOF
[[nodes]]
name = "$component"
internal_ip = "$ip"
port = $port
region = "$region"
role = "worker"

EOF
    done

    echo -e "${GREEN}✅ Bootstrap configuration generated: bootstrap.toml${NC}"

    # Clean up
    rm -rf "$TEMP_DIR"

    echo -e "${GREEN}🎉 Deployment complete!${NC}"
    echo -e "${YELLOW}📋 Next steps:${NC}"
    echo "1. Review bootstrap.toml configuration"
    echo "2. Start the bootstrap node: ssh into bootstrap-coordinator and run ./bootstrap"
    echo "3. Start worker nodes: ssh into each node and run ./communitas-node --config bootstrap.toml"
    echo "4. Test P2P connectivity between nodes"
}

# Run main function
main "$@"