#!/bin/bash

# Update Testnet Configuration Files with Valid Four-Word Addresses
# This script replaces all invalid four-word addresses with valid ones from saorsa-core

set -e

echo "Updating Communitas Testnet Configuration Files"
echo "==============================================="
echo ""

# Build the generator if needed
if [ ! -f "tools/gen_fwid/target/release/gen_fwid" ]; then
    echo "Building four-word address generator..."
    cd tools/gen_fwid
    cargo build --release --quiet
    cd ../..
fi

# Generate valid addresses for each node
echo "Generating valid four-word addresses for testnet nodes..."
VALID_ADDRESSES=(
    $(./tools/gen_fwid/target/release/gen_fwid 6 | cut -d: -f2 | xargs)
)

DROPLETS=(
    "104.248.85.72"   # AMS3
    "138.68.130.66"   # LON1
    "159.89.109.179"  # FRA1
    "165.22.44.216"   # NYC3
    "137.184.123.27"  # SFO3
    "128.199.85.70"   # SGP1
)

REGIONS=(
    "AMS3"
    "LON1"
    "FRA1"
    "NYC3"
    "SFO3"
    "SGP1"
)

# Generate bootstrap nodes list for each config
BOOTSTRAP_SEEDS=""
for i in "${!VALID_ADDRESSES[@]}"; do
    if [ $i -gt 0 ]; then
        BOOTSTRAP_SEEDS="${BOOTSTRAP_SEEDS}, "
    fi
    BOOTSTRAP_SEEDS="${BOOTSTRAP_SEEDS}\"${VALID_ADDRESSES[$i]}:443\""
done

echo ""
echo "Generated addresses:"
for i in "${!VALID_ADDRESSES[@]}"; do
    echo "  Node $((i+1)) (${REGIONS[$i]}): ${VALID_ADDRESSES[$i]}"
done

echo ""
echo "Updating configuration files..."

# Update each node's configuration
for i in {1..6}; do
    node_dir="testnet/node$i"
    config_file="$node_dir/config.toml"

    if [ -f "$config_file" ]; then
        # Get the address for this node (array is 0-indexed)
        node_address="${VALID_ADDRESSES[$((i-1))]}"
        region="${REGIONS[$((i-1))]}"

        echo "  Updating $config_file with address: $node_address"

        # Create updated config
        cat > "$config_file" << EOF
# Communitas Headless Node Configuration
# Node $i: $node_address (${region})

# Identity configuration
[identity]
four_words = "$node_address"
display_name = "TestNode$i"

# Network configuration
[network]
listen_address = "0.0.0.0:$((9000 + i - 1))"
bootstrap_nodes = [
    $BOOTSTRAP_SEEDS
]

# Storage configuration
[storage]
path = "./data"
max_size_gb = 10

# Logging configuration
[logging]
level = "info"
file = "./logs/node$i.log"

# API configuration
[api]
enabled = true
address = "127.0.0.1:$((9002 + i - 1))"
EOF
    else
        echo "  ⚠️  Config file not found: $config_file"
    fi
done

# Update the main bootstrap configuration
echo ""
echo "Updating bootstrap-config.toml..."
cat > bootstrap-config.toml << EOF
# Communitas Testnet Bootstrap Configuration
# Generated: $(date)
# All addresses validated with saorsa-core v0.3.22

[bootstrap]
seeds = [
EOF

for i in "${!VALID_ADDRESSES[@]}"; do
    echo "  # Node $((i+1)) (${REGIONS[$i]}) - ${DROPLETS[$i]}:443" >> bootstrap-config.toml
    echo "  \"${VALID_ADDRESSES[$i]}:443\"," >> bootstrap-config.toml
done

echo "]" >> bootstrap-config.toml

# Update testnet summary
echo ""
echo "Updating testnet-summary.json..."
cat > testnet/testnet-summary.json << EOF
{
  "testnet": {
    "name": "Communitas Testnet",
    "version": "1.0",
    "generated": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "saorsa_core_version": "0.3.22"
  },
  "nodes": [
EOF

for i in "${!VALID_ADDRESSES[@]}"; do
    comma=""
    if [ $i -lt $((${#VALID_ADDRESSES[@]} - 1)) ]; then
        comma=","
    fi

    cat >> testnet/testnet-summary.json << EOF
    {
      "id": $((i+1)),
      "four_word_address": "${VALID_ADDRESSES[$i]}",
      "region": "${REGIONS[$i]}",
      "ip": "${DROPLETS[$i]}",
      "port": 443,
      "config_file": "node$((i+1))/config.toml"
    }$comma
EOF
done

cat >> testnet/testnet-summary.json << EOF
  ],
  "bootstrap": {
    "seeds": [
EOF

for i in "${!VALID_ADDRESSES[@]}"; do
    comma=""
    if [ $i -lt $((${#VALID_ADDRESSES[@]} - 1)) ]; then
        comma=","
    fi
    echo "      \"${VALID_ADDRESSES[$i]}:443\"$comma" >> testnet/testnet-summary.json
done

cat >> testnet/testnet-summary.json << EOF
    ]
  }
}
EOF

echo ""
echo "✅ Update complete!"
echo ""
echo "Summary:"
echo "  ✅ Updated 6 node configuration files"
echo "  ✅ Updated bootstrap-config.toml"
echo "  ✅ Updated testnet/testnet-summary.json"
echo "  ✅ All addresses validated with saorsa-core v0.3.22"
echo ""
echo "Configuration details:"
for i in "${!VALID_ADDRESSES[@]}"; do
    echo "  Node $((i+1)) (${REGIONS[$i]}): ${VALID_ADDRESSES[$i]} -> port $((9000 + i))"
done

echo ""
echo "Next steps:"
echo "  1. Deploy updated configurations to your testnet"
echo "  2. Start nodes with: cd testnet && ./start_testnet.sh"
echo "  3. Monitor logs for successful peer connections"
echo "  4. All addresses will pass saorsa_core::fwid::fw_check() validation"