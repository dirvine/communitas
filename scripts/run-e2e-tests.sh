#!/bin/bash
# Run E2E tests for Communitas Tauri app

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo ""
echo "🧪 Communitas E2E Test Runner"
echo "=============================="
echo ""

# Build frontend
echo -e "${GREEN}📦 Building frontend...${NC}"
npm run build
echo ""

# Check if Tauri dev is running on port 5173
if lsof -Pi :5173 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Tauri already running on :5173${NC}"
    SKIP_START=1
else
    SKIP_START=0
fi

if [ $SKIP_START -eq 0 ]; then
    echo -e "${GREEN}🚀 Starting Tauri dev server...${NC}"
    npm run tauri dev > /tmp/tauri-dev.log 2>&1 &
    TAURI_PID=$!
    
    echo "⏳ Waiting for server on :5173..."
    for i in {1..60}; do
        if lsof -Pi :5173 -sTCP:LISTEN -t >/dev/null 2>&1; then
            break
        fi
        sleep 1
        printf "."
    done
    echo ""
    echo -e "${GREEN}✅ Server ready at http://localhost:5173${NC}"
    sleep 3
fi

# Run tests
echo -e "${GREEN}🧪 Running tests...${NC}"
echo ""

if npm run test:e2e:tauri; then
    RESULT=0
    echo ""
    echo -e "${GREEN}✅ PASSED${NC}"
else
    RESULT=1
    echo ""
    echo -e "${RED}❌ FAILED${NC}"
fi

# Cleanup
if [ $SKIP_START -eq 0 ]; then
    echo ""
    echo -e "${YELLOW}🛑 Stopping server...${NC}"
    kill $TAURI_PID 2>/dev/null || true
fi

exit $RESULT
