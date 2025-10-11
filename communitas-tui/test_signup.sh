#!/bin/bash

echo "Testing Communitas TUI Signup Flow"
echo "==================================="

# Test with reduced PBKDF2 iterations for quick testing
echo "1. Testing with 1000 iterations (fast mode)..."
echo -e "1\nTest User\nq" | cargo run --release -- --pbkdf2-iterations 1000 --no-keyring 2>&1 | grep -E "🔄|🔑|🔒|✅|❌|Welcome"

echo ""
echo "2. Testing with 10000 iterations (medium mode)..."
echo -e "1\nAnother User\nq" | cargo run --release -- --pbkdf2-iterations 10000 --no-keyring 2>&1 | grep -E "🔄|🔑|🔒|✅|❌|Welcome"

echo ""
echo "3. Testing with default 100000 iterations (production mode)..."
echo -e "1\nProduction User\nq" | cargo run --release -- --no-keyring 2>&1 | grep -E "🔄|🔑|🔒|✅|❌|Welcome"

echo ""
echo "Test complete! If you see progress indicators above, the TUI is working correctly."