#!/bin/bash
set -e

echo "Starting multiple Communitas desktop applications..."

# Clean up any existing instances
pkill -f "communitas-desktop" || true
sleep 2

# Create data directories for each instance
APP_DIR="/tmp/communitas-desktop-apps"
rm -rf $APP_DIR
mkdir -p $APP_DIR/app1 $APP_DIR/app2

echo "Starting desktop app 1 on port 14020..."
RUST_LOG=info,communitas=debug,saorsa_core=debug \
COMMUNITAS_DATA_DIR=$APP_DIR/app1 \
COMMUNITAS_QUIC_PORT=14020 \
COMMUNITAS_WEBRTC_PORT=14020 \
/Users/davidirvine/Desktop/Devel/projects/communitas/target/release/communitas-desktop > $APP_DIR/app1.log 2>&1 &

sleep 3

echo "Starting desktop app 2 on port 14021..."
RUST_LOG=info,communitas=debug,saorsa_core=debug \
COMMUNITAS_DATA_DIR=$APP_DIR/app2 \
COMMUNITAS_QUIC_PORT=14021 \
COMMUNITAS_WEBRTC_PORT=14021 \
/Users/davidirvine/Desktop/Devel/projects/communitas/target/release/communitas-desktop > $APP_DIR/app2.log 2>&1 &

echo "Desktop applications started!"
echo "App instances:"
echo "  App 1: Data dir $APP_DIR/app1, Ports 14020 (QUIC/WebRTC)"
echo "  App 2: Data dir $APP_DIR/app2, Ports 14021 (QUIC/WebRTC)"
echo ""
echo "Monitor with:"
echo "  tail -f $APP_DIR/app*.log"
echo ""
echo "The apps should automatically connect to the local testnet nodes"
echo "and be available for testing signup/login and group creation."