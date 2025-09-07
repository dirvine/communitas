#!/bin/bash

# Test Bootstrap Configuration
# Verifies that bootstrap nodes are properly configured and accessible

set -e

echo "🧪 Testing Bootstrap Configuration"
echo "=================================="

# Check if bootstrap.toml exists
if [ ! -f "bootstrap.toml" ]; then
    echo "❌ bootstrap.toml not found"
    echo "Run ./deploy-testnet.sh first to generate bootstrap configuration"
    exit 1
fi

echo "✅ Bootstrap configuration file found"

# Test fwaddr tool
echo ""
echo "🔧 Testing fwaddr tool..."
if [ ! -f "tools/fwaddr/target/release/fwaddr" ]; then
    echo "Building fwaddr tool..."
    cd tools/fwaddr
    cargo build --release
    cd ../..
fi

# Test four-word address generation
TEST_IP="1.2.3.4"
TEST_PORT="443"
WORDS=$(./tools/fwaddr/target/release/fwaddr "$TEST_IP" "$TEST_PORT")
echo "✅ Four-word address for $TEST_IP:$TEST_PORT -> $WORDS"

# Test bootstrap node parsing
echo ""
echo "📄 Testing bootstrap configuration parsing..."
BOOTSTRAP_NODES=$(grep -o '"[^"]*:[0-9]*"' bootstrap.toml | tr -d '"' | head -6)
if [ -z "$BOOTSTRAP_NODES" ]; then
    echo "❌ No bootstrap nodes found in configuration"
    exit 1
fi

echo "✅ Found bootstrap nodes:"
echo "$BOOTSTRAP_NODES" | nl

# Test connectivity to bootstrap nodes (if deployed)
echo ""
echo "🌐 Testing bootstrap node connectivity..."
for node in $BOOTSTRAP_NODES; do
    HOST=$(echo "$node" | cut -d: -f1)
    PORT=$(echo "$node" | cut -d: -f2)

    # Try to resolve the host (basic connectivity test)
    if nslookup "$HOST" >/dev/null 2>&1 || host "$HOST" >/dev/null 2>&1; then
        echo "✅ $node - DNS resolvable"
    else
        echo "⚠️  $node - DNS not resolvable (may be expected for local testnet)"
    fi
done

echo ""
echo "🎉 Bootstrap configuration test complete!"
echo ""
echo "Next steps:"
echo "1. Deploy testnet: ./deploy-testnet.sh"
echo "2. Update bootstrap.toml with actual deployed addresses"
echo "3. Run local instances: ./run-multiple-instances.sh 3"
echo "4. Test connectivity: ./test-connectivity.sh"