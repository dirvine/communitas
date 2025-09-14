#!/bin/bash
# Test workflows locally with act

set -e

echo "========================================="
echo "   Testing Workflows Locally with Act"
echo "========================================="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Change to project root
cd "$(dirname "$0")/.."

# Function to run a workflow job
run_job() {
    local job_name=$1
    local workflow_file=$2
    local timeout_mins=${3:-10}
    
    echo ""
    echo -e "${YELLOW}Running: $job_name from $workflow_file${NC}"
    echo "Timeout: ${timeout_mins} minutes"
    echo "----------------------------------------"
    
    # Run with timeout using a background process
    (
        act -j "$job_name" \
            --container-architecture linux/amd64 \
            -W ".github/workflows/$workflow_file" \
            --artifact-server-path /tmp/artifacts \
            2>&1
    ) &
    
    local pid=$!
    local count=0
    local max_count=$((timeout_mins * 60))
    
    while kill -0 $pid 2>/dev/null; do
        if [ $count -ge $max_count ]; then
            echo -e "${RED}Timeout reached, killing job${NC}"
            kill -9 $pid 2>/dev/null || true
            return 1
        fi
        sleep 1
        count=$((count + 1))
        
        # Show progress every 30 seconds
        if [ $((count % 30)) -eq 0 ]; then
            echo "  ... still running (${count}s elapsed)"
        fi
    done
    
    wait $pid
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✓ $job_name completed successfully${NC}"
        return 0
    else
        echo -e "${RED}✗ $job_name failed with exit code $exit_code${NC}"
        return 1
    fi
}

# Test selection based on argument
case "${1:-all}" in
    quick)
        echo "Running quick tests..."
        run_job "test-fast" "ci.yml" 5
        ;;
    
    security)
        echo "Running security tests..."
        run_job "security-audit" "security-audit.yml" 15
        ;;
    
    build)
        echo "Running build tests..."
        run_job "build-linux" "release-binaries.yml" 20
        ;;
    
    perf)
        echo "Running performance tests..."
        run_job "benchmark-critical-paths" "performance-monitoring.yml" 10
        ;;
    
    rust)
        echo "Running Rust checks..."
        run_job "rust-checks" "ci.yml" 15
        ;;
    
    all)
        echo "Running all workflow tests..."
        
        # Quick tests first
        echo -e "${YELLOW}=== Phase 1: Quick Tests ===${NC}"
        run_job "test-fast" "ci.yml" 5 || true
        
        # Rust checks
        echo -e "${YELLOW}=== Phase 2: Rust Checks ===${NC}"
        run_job "rust-checks" "ci.yml" 15 || true
        
        # Security
        echo -e "${YELLOW}=== Phase 3: Security Audit ===${NC}"
        run_job "security-audit" "security-audit.yml" 15 || true
        
        # Performance
        echo -e "${YELLOW}=== Phase 4: Performance ===${NC}"
        run_job "benchmark-critical-paths" "performance-monitoring.yml" 10 || true
        
        # Build
        echo -e "${YELLOW}=== Phase 5: Build ===${NC}"
        run_job "build-linux" "release-binaries.yml" 20 || true
        ;;
    
    *)
        echo "Usage: $0 [quick|security|build|perf|rust|all]"
        echo ""
        echo "Options:"
        echo "  quick    - Run JS fast tests"
        echo "  security - Run security audit"
        echo "  build    - Run Linux binary build"
        echo "  perf     - Run performance benchmarks"
        echo "  rust     - Run Rust checks"
        echo "  all      - Run all tests (default)"
        exit 1
        ;;
esac

echo ""
echo "========================================="
echo "   Workflow Testing Complete"
echo "========================================="