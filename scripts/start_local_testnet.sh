#!/bin/bash
set -e

echo "Starting Communitas local testnet with ports > 14000..."

# Clean up any existing testnet
pkill -f communitas-headless || true
sleep 2

# Create testnet directory
TESTNET_DIR="/tmp/communitas-testnet-local"
rm -rf $TESTNET_DIR
mkdir -p $TESTNET_DIR

# Use existing binary directly
BINARY_PATH="/Users/davidirvine/Desktop/Devel/projects/communitas/target/release/communitas-headless"

echo "Starting node 0 (bootstrap) on port 14000..."
$BINARY_PATH \
    --listen 127.0.0.1:14000 \
    --config /tmp/testnet_config_local.toml \
    --storage $TESTNET_DIR/node-0/storage \
    --metrics --metrics-addr 127.0.0.1:14010 > $TESTNET_DIR/node-0.log 2>&1 &

sleep 2

echo "Starting node 1 on port 14001..."
$BINARY_PATH \
    --listen 127.0.0.1:14001 \
    --config /tmp/testnet_config_local.toml \
    --storage $TESTNET_DIR/node-1/storage \
    --metrics --metrics-addr 127.0.0.1:14011 > $TESTNET_DIR/node-1.log 2>&1 &

sleep 2

echo "Starting node 2 on port 14002..."
$BINARY_PATH \
    --listen 127.0.0.1:14002 \
    --config /tmp/testnet_config_local.toml \
    --storage $TESTNET_DIR/node-2/storage \
    --metrics --metrics-addr 127.0.0.1:14012 > $TESTNET_DIR/node-2.log 2>&1 &

echo "Local testnet started!"
echo "Nodes running on:"
echo "  Node 0: 127.0.0.1:14000 (metrics: 14010)"
echo "  Node 1: 127.0.0.1:14001 (metrics: 14011)"
echo "  Node 2: 127.0.0.1:14002 (metrics: 14012)"
echo ""
echo "Monitor with:"
echo "  tail -f $TESTNET_DIR/node-*.log"
echo "  curl http://127.0.0.1:14010/metrics"
echo ""
echo "Test connectivity:"
echo "  curl http://127.0.0.1:14000/health"
echo "  curl http://127.0.0.1:14001/health"
echo "  curl http://127.0.0.1:14002/health"