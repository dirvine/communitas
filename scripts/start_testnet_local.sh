#!/bin/bash
set -e

echo "Starting Communitas testnet with localhost addresses..."

# Clean up any existing testnet
pkill -f communitas-headless || true
sleep 2

# Create testnet directory
TESTNET_DIR="/tmp/communitas-testnet-local"
rm -rf $TESTNET_DIR
mkdir -p $TESTNET_DIR

# Use existing binary directly
BINARY_PATH="/Users/davidirvine/Desktop/Devel/projects/communitas/target/debug/communitas-headless"

echo "Starting node 0 (bootstrap)..."
$BINARY_PATH \
    --listen 127.0.0.1:9000 \
    --config /tmp/testnet_config_local.toml \
    --storage $TESTNET_DIR/node-0/storage \
    --metrics --metrics-addr 127.0.0.1:9600 > $TESTNET_DIR/node-0.log 2>&1 &

sleep 2

echo "Starting node 1..."
$BINARY_PATH \
    --listen 127.0.0.1:9001 \
    --config /tmp/testnet_config_local.toml \
    --storage $TESTNET_DIR/node-1/storage \
    --metrics --metrics-addr 127.0.0.1:9601 > $TESTNET_DIR/node-1.log 2>&1 &

sleep 2

echo "Starting node 2..."
$BINARY_PATH \
    --listen 127.0.0.1:9002 \
    --config /tmp/testnet_config_local.toml \
    --storage $TESTNET_DIR/node-2/storage \
    --metrics --metrics-addr 127.0.0.1:9602 > $TESTNET_DIR/node-2.log 2>&1 &

echo "Testnet started!"
echo "Nodes running on:"
echo "  Node 0: 127.0.0.1:9000 (metrics: 9600)"
echo "  Node 1: 127.0.0.1:9001 (metrics: 9601)" 
echo "  Node 2: 127.0.0.1:9002 (metrics: 9602)"
echo ""
echo "Monitor with:"
echo "  tail -f $TESTNET_DIR/node-*.log"
echo "  curl http://127.0.0.1:9600/metrics"
