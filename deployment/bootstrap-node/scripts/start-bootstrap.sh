#\!/bin/bash
set -euo pipefail

# Bootstrap Node Startup Script for Communitas

LOG_DIR="/var/log/saorsa"
DATA_DIR="/var/lib/saorsa"
CONFIG_FILE="/etc/saorsa/bootstrap.toml"

# Ensure directories exist
mkdir -p "$LOG_DIR" "$DATA_DIR/dht" "$DATA_DIR/data"

# Generate identity if it doesn't exist
if [[ \! -f "$DATA_DIR/identity.key" ]]; then
    echo "$(date): Generating new bootstrap node identity..."
    /usr/local/bin/saorsa generate-identity --output "$DATA_DIR/identity.key"
fi

# Log startup
echo "$(date): Starting Communitas Bootstrap Node..." | tee -a "$LOG_DIR/startup.log"
echo "$(date): Configuration: $CONFIG_FILE" | tee -a "$LOG_DIR/startup.log"
echo "$(date): Data directory: $DATA_DIR" | tee -a "$LOG_DIR/startup.log"

# Start the bootstrap node
exec /usr/local/bin/saorsa bootstrap \
    --config "$CONFIG_FILE" \
    --log-level info \
    --port 8888 \
    --data-dir "$DATA_DIR"
EOF < /dev/null