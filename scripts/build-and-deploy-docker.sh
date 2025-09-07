#!/bin/bash

# Build and Deploy Docker Image for Communitas Testnet
# This script builds Rust binaries, creates a Docker image, and updates DO app spec

set -e

# Configuration
DOCKER_IMAGE="communitas/testnet-node"
DOCKER_TAG="${1:-latest}"
APP_ID="18ccbf3a-7111-4897-a0b6-f215910bcf1a"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🐳 Building Communitas Testnet Docker Image${NC}"
echo "============================================="

# Function to check prerequisites
check_prerequisites() {
    echo -e "${YELLOW}🔍 Checking prerequisites...${NC}"

    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}❌ Docker is not installed. Please install Docker Desktop for Mac.${NC}"
        exit 1
    fi

    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust is not installed. Please install Rust.${NC}"
        exit 1
    fi

    # Check if cross-compilation tools are available
    if ! command -v cross &> /dev/null; then
        echo -e "${YELLOW}⚠️  'cross' tool not found. Installing...${NC}"
        cargo install cross
    fi

    echo -e "${GREEN}✅ Prerequisites OK${NC}"
}

# Function to build Rust binaries for Linux
build_binaries() {
    echo -e "${YELLOW}🔨 Building Rust binaries for Linux x86_64...${NC}"

    # Use cross for cross-compilation to Linux
    cross build --release --target x86_64-unknown-linux-gnu --bin communitas-node
    cross build --release --target x86_64-unknown-linux-gnu --bin bootstrap
    cross build --release --target x86_64-unknown-linux-gnu --bin communitas-autoupdater

    echo -e "${GREEN}✅ Binaries built successfully${NC}"
}

# Function to build Docker image
build_docker_image() {
    echo -e "${YELLOW}🏗️  Building Docker image...${NC}"

    # Copy binaries to build context
    cp src-tauri/target/x86_64-unknown-linux-gnu/release/communitas-node .
    cp src-tauri/target/x86_64-unknown-linux-gnu/release/bootstrap .
    cp src-tauri/target/x86_64-unknown-linux-gnu/release/communitas-autoupdater .

    # Build Docker image
    docker build -f Dockerfile.testnet -t "$DOCKER_IMAGE:$DOCKER_TAG" .

    echo -e "${GREEN}✅ Docker image built: $DOCKER_IMAGE:$DOCKER_TAG${NC}"
}

# Function to push Docker image
push_docker_image() {
    echo -e "${YELLOW}📤 Pushing Docker image to registry...${NC}"

    # Tag and push
    docker tag "$DOCKER_IMAGE:$DOCKER_TAG" "$DOCKER_IMAGE:$DOCKER_TAG"
    docker push "$DOCKER_IMAGE:$DOCKER_TAG"

    echo -e "${GREEN}✅ Docker image pushed: $DOCKER_IMAGE:$DOCKER_TAG${NC}"
}

# Function to update Digital Ocean app spec
update_do_app_spec() {
    echo -e "${YELLOW}🔄 Updating Digital Ocean app specification...${NC}"

    # Create updated app spec with new Docker image
    cat > do-app-spec-updated.json << EOF
{
  "name": "communitas-testnet",
  "services": [
    {
      "name": "bootstrap-coordinator",
      "image": {
        "registry_type": "DOCKER_HUB",
        "registry": "",
        "repository": "$DOCKER_IMAGE",
        "tag": "$DOCKER_TAG"
      },
      "run_command": "./bootstrap --config /app/bootstrap.toml",
      "envs": [
        {
          "key": "NODE_TYPE",
          "value": "bootstrap",
          "scope": "RUN_TIME"
        },
        {
          "key": "PORT",
          "value": "8080",
          "scope": "RUN_TIME"
        }
      ],
      "instance_size_slug": "professional-xs",
      "instance_count": 1,
      "http_port": 8080
    }
  ],
  "workers": [
    {
      "name": "node-nyc",
      "image": {
        "registry_type": "DOCKER_HUB",
        "registry": "",
        "repository": "$DOCKER_IMAGE",
        "tag": "$DOCKER_TAG"
      },
      "run_command": "./communitas-node --config /app/bootstrap.toml --node-name node-nyc",
      "envs": [
        {
          "key": "REGION",
          "value": "NYC",
          "scope": "RUN_TIME"
        },
        {
          "key": "NODE_CAPACITY",
          "value": "1000",
          "scope": "RUN_TIME"
        },
        {
          "key": "PORT",
          "value": "8081",
          "scope": "RUN_TIME"
        }
      ],
      "instance_size_slug": "professional-l",
      "instance_count": 1
    },
    {
      "name": "node-sfo",
      "image": {
        "registry_type": "DOCKER_HUB",
        "registry": "",
        "repository": "$DOCKER_IMAGE",
        "tag": "$DOCKER_TAG"
      },
      "run_command": "./communitas-node --config /app/bootstrap.toml --node-name node-sfo",
      "envs": [
        {
          "key": "REGION",
          "value": "SFO",
          "scope": "RUN_TIME"
        },
        {
          "key": "NODE_CAPACITY",
          "value": "1000",
          "scope": "RUN_TIME"
        },
        {
          "key": "PORT",
          "value": "8082",
          "scope": "RUN_TIME"
        }
      ],
      "instance_size_slug": "professional-l",
      "instance_count": 1
    },
    {
      "name": "node-lon",
      "image": {
        "registry_type": "DOCKER_HUB",
        "registry": "",
        "repository": "$DOCKER_IMAGE",
        "tag": "$DOCKER_TAG"
      },
      "run_command": "./communitas-node --config /app/bootstrap.toml --node-name node-lon",
      "envs": [
        {
          "key": "REGION",
          "value": "LON",
          "scope": "RUN_TIME"
        },
        {
          "key": "NODE_CAPACITY",
          "value": "1000",
          "scope": "RUN_TIME"
        },
        {
          "key": "PORT",
          "value": "8083",
          "scope": "RUN_TIME"
        }
      ],
      "instance_size_slug": "professional-l",
      "instance_count": 1
    },
    {
      "name": "node-fra",
      "image": {
        "registry_type": "DOCKER_HUB",
        "registry": "",
        "repository": "$DOCKER_IMAGE",
        "tag": "$DOCKER_TAG"
      },
      "run_command": "./communitas-node --config /app/bootstrap.toml --node-name node-fra",
      "envs": [
        {
          "key": "REGION",
          "value": "FRA",
          "scope": "RUN_TIME"
        },
        {
          "key": "NODE_CAPACITY",
          "value": "1000",
          "scope": "RUN_TIME"
        },
        {
          "key": "PORT",
          "value": "8084",
          "scope": "RUN_TIME"
        }
      ],
      "instance_size_slug": "professional-l",
      "instance_count": 1
    },
    {
      "name": "node-sgp",
      "image": {
        "registry_type": "DOCKER_HUB",
        "registry": "",
        "repository": "$DOCKER_IMAGE",
        "tag": "$DOCKER_TAG"
      },
      "run_command": "./communitas-node --config /app/bootstrap.toml --node-name node-sgp",
      "envs": [
        {
          "key": "REGION",
          "value": "SGP",
          "scope": "RUN_TIME"
        },
        {
          "key": "NODE_CAPACITY",
          "value": "1000",
          "scope": "RUN_TIME"
        },
        {
          "key": "PORT",
          "value": "8085",
          "scope": "RUN_TIME"
        }
      ],
       "instance_size_slug": "professional-l",
       "instance_count": 1
     },
     {
       "name": "node-syd",
       "image": {
         "registry_type": "DOCKER_HUB",
         "registry": "",
         "repository": "$DOCKER_IMAGE",
         "tag": "$DOCKER_TAG"
       },
       "run_command": "./communitas-node --config /app/bootstrap.toml --node-name node-syd",
       "envs": [
         {
           "key": "REGION",
           "value": "SYD",
           "scope": "RUN_TIME"
         },
         {
           "key": "NODE_CAPACITY",
           "value": "1000",
           "scope": "RUN_TIME"
         },
         {
           "key": "PORT",
           "value": "8086",
           "scope": "RUN_TIME"
         }
       ],
       "instance_size_slug": "professional-l",
       "instance_count": 1
     }
   ],
   "databases": [
     {
       "name": "testnet-metrics",
       "engine": "PG",
       "version": "15",
       "size": "db-s-2vcpu-4gb",
       "num_nodes": 1
     }
   ],
   "region": "nyc",
   "ingress": {
     "rules": [
       {
         "match": {
           "path": {
             "prefix": "/"
           }
         },
         "component": {
           "name": "bootstrap-coordinator"
         }
       }
     ]
   },
   "features": [
     "buildpack-stack=ubuntu-22"
   ]
 }
EOF

    echo -e "${GREEN}✅ Updated app spec created: do-app-spec-updated.json${NC}"
    echo -e "${YELLOW}💡 Next: Update your Digital Ocean app with this spec${NC}"
}

# Function to create bootstrap config
create_bootstrap_config() {
    echo -e "${YELLOW}📝 Creating bootstrap configuration template...${NC}"

    cp bootstrap.toml.template bootstrap.toml

    echo -e "${GREEN}✅ Bootstrap config template created${NC}"
    echo -e "${YELLOW}💡 Remember to update IP addresses in bootstrap.toml${NC}"
}

# Main build process
main() {
    check_prerequisites
    build_binaries
    build_docker_image

    echo -e "${YELLOW}🔐 Please log in to Docker Hub:${NC}"
    echo "docker login"
    read -p "Press Enter after logging in to Docker Hub..."

    push_docker_image
    update_do_app_spec
    create_bootstrap_config

    echo ""
    echo -e "${GREEN}🎉 Docker build and deployment preparation complete!${NC}"
    echo ""
    echo -e "${BLUE}📋 Next steps:${NC}"
    echo "1. ✅ Docker image built and pushed: $DOCKER_IMAGE:$DOCKER_TAG"
    echo "2. 🔄 Update Digital Ocean app with: do-app-spec-updated.json"
    echo "3. 📝 Update bootstrap.toml with actual container IPs"
    echo "4. 🚀 Deploy the updated app specification"
    echo "5. 🧪 Test the network connectivity"

    # Clean up copied binaries
    rm -f communitas-node bootstrap communitas-autoupdater
}

# Run main function
main "$@"