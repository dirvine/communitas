#!/bin/bash
# Check health of all MCP nodes
# Usage: ./health-check.sh

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== MCP Node Health Check ===${NC}"
echo ""

HEALTHY=0
UNHEALTHY=0

# Check alice (saorsa-4)
printf "%-30s " "alice (saorsa-4)"
if curl -sf --connect-timeout 5 "http://206.189.7.117:3040/health" >/dev/null 2>&1; then
    echo -e "${GREEN}[HEALTHY]${NC}"
    HEALTHY=$((HEALTHY + 1))
else
    echo -e "${RED}[UNHEALTHY]${NC}"
    UNHEALTHY=$((UNHEALTHY + 1))
fi

# Check bob (saorsa-5)
printf "%-30s " "bob (saorsa-5)"
if curl -sf --connect-timeout 5 "http://144.126.230.161:3040/health" >/dev/null 2>&1; then
    echo -e "${GREEN}[HEALTHY]${NC}"
    HEALTHY=$((HEALTHY + 1))
else
    echo -e "${RED}[UNHEALTHY]${NC}"
    UNHEALTHY=$((UNHEALTHY + 1))
fi

# Check charlie (saorsa-6)
printf "%-30s " "charlie (saorsa-6)"
if curl -sf --connect-timeout 5 "http://65.21.157.229:3040/health" >/dev/null 2>&1; then
    echo -e "${GREEN}[HEALTHY]${NC}"
    HEALTHY=$((HEALTHY + 1))
else
    echo -e "${RED}[UNHEALTHY]${NC}"
    UNHEALTHY=$((UNHEALTHY + 1))
fi

# Check dave (localhost)
printf "%-30s " "dave (localhost)"
if curl -sf --connect-timeout 5 "http://127.0.0.1:3041/health" >/dev/null 2>&1; then
    echo -e "${GREEN}[HEALTHY]${NC}"
    HEALTHY=$((HEALTHY + 1))
else
    echo -e "${RED}[UNHEALTHY]${NC}"
    UNHEALTHY=$((UNHEALTHY + 1))
fi

echo ""
echo -e "${BLUE}Summary:${NC} $HEALTHY healthy, $UNHEALTHY unhealthy"

if [[ $UNHEALTHY -eq 4 ]]; then
    echo -e "${RED}All nodes unhealthy! Cannot run tests.${NC}"
    exit 1
fi
