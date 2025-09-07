#!/bin/bash

# Communitas Local Network Test Script
# Tests P2P connectivity between Docker containers

set -e

echo "🚀 Starting Communitas Local Network Test"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker first."
    exit 1
fi

# Check if images exist
if ! docker images communitas-bootstrap:latest | grep -q communitas-bootstrap; then
    print_error "Docker image 'communitas-bootstrap:latest' not found."
    print_status "Please wait for Docker build to complete or run: docker build -f Dockerfile.simple -t communitas-bootstrap:latest ."
    exit 1
fi

print_status "Found Docker images ✓"

# Create data directories
print_status "Creating data directories..."
mkdir -p data/bootstrap data/node1 data/node2 data/node3

# Clean up any existing containers
print_status "Cleaning up existing containers..."
docker-compose -f docker-compose.testnet.yml down --remove-orphans 2>/dev/null || true

# Start the bootstrap node
print_status "Starting bootstrap node..."
docker-compose -f docker-compose.testnet.yml up -d bootstrap

# Wait for bootstrap to be healthy
print_status "Waiting for bootstrap node to be healthy..."
max_attempts=30
attempt=1
while [ $attempt -le $max_attempts ]; do
    if docker-compose -f docker-compose.testnet.yml ps bootstrap | grep -q "healthy\|running"; then
        print_success "Bootstrap node is ready!"
        break
    fi

    print_status "Waiting for bootstrap... (attempt $attempt/$max_attempts)"
    sleep 10
    attempt=$((attempt + 1))
done

if [ $attempt -gt $max_attempts ]; then
    print_error "Bootstrap node failed to become healthy"
    docker-compose -f docker-compose.testnet.yml logs bootstrap
    exit 1
fi

# Start the other nodes
print_status "Starting node1..."
docker-compose -f docker-compose.testnet.yml up -d node1

print_status "Starting node2..."
docker-compose -f docker-compose.testnet.yml up -d node2

print_status "Starting node3..."
docker-compose -f docker-compose.testnet.yml up -d node3

# Wait for nodes to start
print_status "Waiting for all nodes to start..."
sleep 15

# Check container status
print_status "Checking container status..."
docker-compose -f docker-compose.testnet.yml ps

# Test connectivity
print_status "Testing network connectivity..."

# Test bootstrap health endpoint
if curl -f http://localhost:8080/health >/dev/null 2>&1; then
    print_success "Bootstrap health check passed ✓"
else
    print_warning "Bootstrap health check failed"
fi

# Check logs for connectivity
print_status "Checking logs for P2P connectivity..."

# Bootstrap logs
print_status "Bootstrap node logs:"
docker-compose -f docker-compose.testnet.yml logs bootstrap | tail -20

# Node1 logs
print_status "Node1 logs:"
docker-compose -f docker-compose.testnet.yml logs node1 | tail -20

# Node2 logs
print_status "Node2 logs:"
docker-compose -f docker-compose.testnet.yml logs node2 | tail -20

# Node3 logs
print_status "Node3 logs:"
docker-compose -f docker-compose.testnet.yml logs node3 | tail -20

# Test inter-container connectivity
print_status "Testing inter-container connectivity..."
if docker exec communitas-bootstrap curl -f http://bootstrap:8080/health >/dev/null 2>&1; then
    print_success "Inter-container connectivity test passed ✓"
else
    print_warning "Inter-container connectivity test failed"
fi

print_success "Local network test completed!"
print_status "Use 'docker-compose -f docker-compose.testnet.yml logs -f' to monitor logs"
print_status "Use 'docker-compose -f docker-compose.testnet.yml down' to stop the network"