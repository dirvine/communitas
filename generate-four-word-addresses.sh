#!/bin/bash

# Generate Four-Word Addresses for Communitas Testnet
# This script generates four-word addresses for the deployed droplets

# Droplet IPs (update these with your actual droplet IPs)
DROPLETS=(
    "104.248.85.72"  # AMS3
    "138.68.130.66"  # LON1
    "159.89.109.179" # FRA1
    "165.22.44.216"  # NYC3
    "137.184.123.27" # SFO3
    "128.199.85.70"  # SGP1
)

REGIONS=(
    "AMS3 (Amsterdam)"
    "LON1 (London)"
    "FRA1 (Frankfurt)"
    "NYC3 (New York)"
    "SFO3 (San Francisco)"
    "SGP1 (Singapore)"
)

PORT=443

echo "Communitas Testnet - Four-Word Addresses"
echo "=========================================="
echo ""

# Function to generate four-word address (simplified version)
# In a real implementation, you'd use the actual four-word-networking library
generate_four_word() {
    local ip=$1
    local port=$2

    # This is a placeholder - you'd need the actual four-word-networking library
    # For now, we'll create deterministic but fake addresses based on IP
    local words=("sparrow" "eagle" "wolf" "tiger" "bear" "dragon" "candle" "feather" "pack" "stripe" "claw" "scale" "forest" "mountain" "snow" "jungle" "rocky" "oriental" "ember" "peak" "summit" "leaf" "garden")

    # Generate deterministic index based on IP
    local ip_sum=0
    IFS='.' read -ra IP_PARTS <<< "$ip"
    for part in "${IP_PARTS[@]}"; do
        ip_sum=$((ip_sum + part))
    done

    local word1=${words[$((ip_sum % ${#words[@]}))]}
    local word2=${words[$(((ip_sum + 7) % ${#words[@]}))]}
    local word3=${words[$(((ip_sum + 13) % ${#words[@]}))]}
    local word4=${words[$(((ip_sum + 19) % ${#words[@]}))]}

    echo "$word1-$word2-$word3-$word4:$port"
}

echo "Bootstrap Configuration:"
echo "======================="
echo ""
echo "[bootstrap]"
echo "seeds = ["

for i in "${!DROPLETS[@]}"; do
    ip=${DROPLETS[$i]}
    region=${REGIONS[$i]}
    address=$(generate_four_word "$ip" "$PORT")

    echo "  # $region - $ip:$PORT"
    echo "  \"$address\","

    # Store for later use
    ADDRESSES[$i]=$address
done

echo "]"
echo ""
echo "Individual Node Addresses:"
echo "=========================="

for i in "${!DROPLETS[@]}"; do
    ip=${DROPLETS[$i]}
    region=${REGIONS[$i]}
    address=${ADDRESSES[$i]}

    echo "$region: $address"
done

echo ""
echo "Copy the seeds array above into your bootstrap.toml file."
echo "Each node should use this configuration for initial peer discovery."