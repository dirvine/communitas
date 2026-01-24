#!/usr/bin/env bash
#
# run-full-regression.sh - Comprehensive regression test suite for release validation
#
# Usage: ./scripts/run-full-regression.sh [--quick] [--no-release]
#
# Options:
#   --quick       Skip slow tests (ignored tests, stress tests)
#   --no-release  Run in debug mode instead of release
#   --help        Show this help message
#
# Exit codes:
#   0 - All tests passed
#   1 - One or more tests failed
#   2 - Build failed
#   3 - Environment issue

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
QUICK_MODE=false
RELEASE_MODE=true
START_TIME=$(date +%s)

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --no-release)
            RELEASE_MODE=false
            shift
            ;;
        --help)
            head -20 "$0" | tail -n +2 | sed 's/^# //' | sed 's/^#//'
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 3
            ;;
    esac
done

# Build mode flag
if $RELEASE_MODE; then
    BUILD_FLAG="--release"
    BUILD_NAME="release"
else
    BUILD_FLAG=""
    BUILD_NAME="debug"
fi

log() {
    local level=$1
    shift
    local timestamp
    timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    case $level in
        INFO)
            echo -e "${BLUE}[$timestamp] [INFO]${NC} $*"
            ;;
        OK)
            echo -e "${GREEN}[$timestamp] [OK]${NC} $*"
            ;;
        WARN)
            echo -e "${YELLOW}[$timestamp] [WARN]${NC} $*"
            ;;
        ERROR)
            echo -e "${RED}[$timestamp] [ERROR]${NC} $*"
            ;;
    esac
}

section() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Track results
TOTAL_STEPS=0
PASSED_STEPS=0
FAILED_STEPS=0

run_step() {
    local name=$1
    shift
    TOTAL_STEPS=$((TOTAL_STEPS + 1))
    
    log INFO "Running: $name"
    
    if "$@"; then
        PASSED_STEPS=$((PASSED_STEPS + 1))
        log OK "$name - PASSED"
        return 0
    else
        FAILED_STEPS=$((FAILED_STEPS + 1))
        log ERROR "$name - FAILED"
        return 1
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# Pre-flight Checks
# ═══════════════════════════════════════════════════════════════════════════════

section "Pre-flight Checks"

log INFO "Mode: $BUILD_NAME (quick=$QUICK_MODE)"

# Check Rust toolchain
log INFO "Checking Rust toolchain..."
RUST_VERSION=$(rustc --version)
log INFO "Rust version: $RUST_VERSION"

# Check cargo
CARGO_VERSION=$(cargo --version)
log INFO "Cargo version: $CARGO_VERSION"

# Check we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "communitas-core" ]]; then
    log ERROR "Must be run from the communitas repository root"
    exit 3
fi

# Get project version
PROJECT_VERSION=$(grep -m1 'version = ' Cargo.toml | cut -d'"' -f2)
log INFO "Project version: $PROJECT_VERSION"

# ═══════════════════════════════════════════════════════════════════════════════
# Build Validation
# ═══════════════════════════════════════════════════════════════════════════════

section "Build Validation"

# Format check
run_step "Format check (cargo fmt)" \
    cargo fmt --all -- --check

# Clippy
run_step "Clippy lint check" \
    cargo clippy --workspace --all-features $BUILD_FLAG -- \
        -D warnings \
        -D clippy::unwrap_used \
        -D clippy::expect_used

# Build all targets (except fuzzing which is linux-only)
run_step "Build workspace ($BUILD_NAME)" \
    cargo build --workspace $BUILD_FLAG

# ═══════════════════════════════════════════════════════════════════════════════
# Test Execution
# ═══════════════════════════════════════════════════════════════════════════════

section "Test Execution"

# Unit and integration tests
run_step "Unit & integration tests" \
    cargo test --workspace $BUILD_FLAG

# Doc tests
run_step "Documentation tests" \
    cargo test --workspace --doc $BUILD_FLAG

# Slow/ignored tests (unless quick mode)
if ! $QUICK_MODE; then
    section "Extended Tests (Slow)"
    
    run_step "Ignored tests" \
        cargo test --workspace $BUILD_FLAG -- --ignored || true
    
    # Property tests
    run_step "Property tests" \
        cargo test --workspace $BUILD_FLAG --test property_tests -- --test-threads=1 || true
    
    # Stress tests (subset for regression)
    run_step "Stress tests (subset)" \
        cargo test --workspace $BUILD_FLAG --test stress -- --test-threads=1 || true
fi

# ═══════════════════════════════════════════════════════════════════════════════
# Integration Tests
# ═══════════════════════════════════════════════════════════════════════════════

section "Integration Tests"

# E2E tests
for test_file in communitas-ui-service/tests/*_e2e.rs; do
    test_name=$(basename "$test_file" .rs)
    run_step "E2E: $test_name" \
        cargo test -p communitas-ui-service $BUILD_FLAG --test "$test_name" || true
done

# ═══════════════════════════════════════════════════════════════════════════════
# Report
# ═══════════════════════════════════════════════════════════════════════════════

section "Summary"

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))
MINUTES=$((ELAPSED / 60))
SECONDS=$((ELAPSED % 60))

echo ""
log INFO "Version: $PROJECT_VERSION"
log INFO "Mode: $BUILD_NAME"
log INFO "Duration: ${MINUTES}m ${SECONDS}s"
echo ""
echo -e "  Total steps:  $TOTAL_STEPS"
echo -e "  ${GREEN}Passed:       $PASSED_STEPS${NC}"
if [[ $FAILED_STEPS -gt 0 ]]; then
    echo -e "  ${RED}Failed:       $FAILED_STEPS${NC}"
fi
echo ""

if [[ $FAILED_STEPS -gt 0 ]]; then
    log ERROR "REGRESSION FAILED - $FAILED_STEPS test(s) failed"
    exit 1
else
    log OK "REGRESSION PASSED - All $PASSED_STEPS steps completed successfully"
    exit 0
fi
