#!/bin/bash
# Simple E2E test runner - requires Tauri dev server to be running

echo "🧪 Running E2E Tests"
echo "Prerequisites: npm run tauri dev (in another terminal)"
echo ""

# Check if server is running on 5173 (Tauri dev port)
if ! lsof -Pi :5173 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "❌ Tauri dev server not running on :5173"
    echo ""
    echo "Start it first:"
    echo "  Terminal 1: npm run tauri dev"
    echo "  Terminal 2: bash scripts/test-e2e.sh"
    echo ""
    echo "Wait for: Local: http://localhost:5173/"
    exit 1
fi

echo "✅ Server detected on :5173"
echo "🧪 Running tests..."
echo ""

npm run test:e2e:tauri
