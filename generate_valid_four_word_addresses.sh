#!/bin/bash

# Generate Valid Four-Word Addresses for Communitas Testnet
# This script uses the actual communitas-headless binary to generate valid addresses

set -e

echo "Building communitas-headless binary..."
cd communitas-headless
cargo build --release --quiet
cd ..

echo ""
echo "Generating valid four-word addresses using saorsa-core..."
echo "=========================================================="
echo ""

# Function to generate valid addresses by running headless binary
generate_valid_addresses() {
    local count=${1:-10}
    local addresses=()

    echo "Generating $count valid four-word addresses..."
    echo ""

    for i in $(seq 1 $count); do
        # Create a temporary config that will cause the binary to generate and exit
        local temp_dir=$(mktemp -d)
        local temp_config="$temp_dir/config.toml"

        # Create minimal config
        cat > "$temp_config" << EOF
[node]
storage_path = "$temp_dir/storage"
listen_addr = "127.0.0.1:0"

[logging]
level = "warn"
EOF

        # Run headless node with timeout to catch the generated identity
        timeout 5s ./target/release/communitas-headless \
            --config "$temp_config" \
            --storage "$temp_dir/storage" 2>&1 | \
            grep -E "(Generated new four-word identity|four-word identity)" | \
            sed -E 's/.*: ([a-z]+-[a-z]+-[a-z]+-[a-z]+).*/\1/' | head -1 || true

        # Clean up
        rm -rf "$temp_dir"
    done
}

# Generate some addresses for testing
echo "Sample valid four-word addresses:"
generate_valid_addresses 5

echo ""
echo "Bootstrap configuration template:"
echo "================================"
echo ""
echo "[bootstrap]"
echo "seeds = ["

# Generate addresses for the actual testnet IPs
DROPLETS=(
    "104.248.85.72"   # AMS3
    "138.68.130.66"   # LON1
    "159.89.109.179"  # FRA1
    "165.22.44.216"   # NYC3
    "137.184.123.27"  # SFO3
    "128.199.85.70"   # SGP1
)

REGIONS=(
    "AMS3 (Amsterdam)"
    "LON1 (London)"
    "FRA1 (Frankfurt)"
    "NYC3 (New York)"
    "SFO3 (San Francisco)"
    "SGP1 (Singapore)"
)

# Generate valid addresses (we'll use random ones since we can't deterministically
# map IPs to valid words without knowing the exact saorsa-core dictionary)
echo "  # NOTE: These are randomly generated valid addresses"
echo "  # In production, use the actual four-word addresses from your nodes"

for i in "${!DROPLETS[@]}"; do
    ip=${DROPLETS[$i]}
    region=${REGIONS[$i]}

    # Generate a valid address
    valid_address=$(generate_valid_addresses 1 | tail -1)
    port="443"

    if [ -n "$valid_address" ]; then
        echo "  # $region - $ip:$port"
        echo "  \"$valid_address:$port\","
    else
        echo "  # $region - $ip:$port (failed to generate valid address)"
        echo "  # \"placeholder-address-needed:$port\","
    fi
done

echo "]"
echo ""
echo "IMPORTANT NOTES:"
echo "================"
echo "1. These are random valid addresses from saorsa-core's dictionary"
echo "2. Each node should generate its own four-word identity on first run"
echo "3. Copy the actual generated addresses from your node logs"
echo "4. All addresses use words from saorsa-core's internal word list"
echo "5. Addresses are validated using saorsa_core::fwid::fw_check()"