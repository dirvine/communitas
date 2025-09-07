#!/bin/bash

# Monitor Docker Build Status
# Checks if Docker images are available and provides build status

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

echo "🔍 Checking Docker Build Status"
echo "================================="

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    print_error "Docker is not running"
    exit 1
fi

print_success "Docker is running"

# Check for images
echo ""
print_status "Checking for Docker images..."

# Check bootstrap image
if docker images communitas-bootstrap:latest | grep -q communitas-bootstrap; then
    print_success "communitas-bootstrap:latest - READY ✓"
else
    print_warning "communitas-bootstrap:latest - NOT FOUND"
fi

# Check testnet image
if docker images communitas-testnet:latest | grep -q communitas-testnet; then
    print_success "communitas-testnet:latest - READY ✓"
else
    print_warning "communitas-testnet:latest - NOT FOUND"
fi

# Check build processes
echo ""
print_status "Checking active build processes..."
if docker ps -a | grep -q "build"; then
    print_status "Active build containers found:"
    docker ps -a | grep build
else
    print_status "No active build containers"
fi

# Check Docker system resources
echo ""
print_status "Docker system info:"
docker system df

# Provide next steps
echo ""
echo "📋 Next Steps:"
if ! docker images communitas-bootstrap:latest | grep -q communitas-bootstrap; then
    echo "  1. Wait for Docker build to complete"
    echo "  2. Or run: docker build -f Dockerfile.simple -t communitas-bootstrap:latest ."
fi

if docker images communitas-bootstrap:latest | grep -q communitas-bootstrap; then
    echo "  1. Run local network test: ./test-local-network.sh"
    echo "  2. Monitor with: docker-compose -f docker-compose.testnet.yml logs -f"
    echo "  3. Check connectivity: curl http://localhost:8080/health"
fi

echo ""
print_status "Build monitoring complete"