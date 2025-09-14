# Local Workflow Testing with Act

This document describes how to test GitHub Actions workflows locally using `act` and direct commands.

## Prerequisites

- Docker Desktop installed and running
- `act` installed: `brew install act` (macOS) or [other methods](https://github.com/nektos/act)
- Rust toolchain: `rustup update stable`
- Node.js 18+: `brew install node@18`

## Running Workflows with Act

### Basic Usage

```bash
# List all available workflows
act -l

# Run a specific job with Apple Silicon support
act -j <job-name> --container-architecture linux/amd64

# Run with specific workflow file
act -j security-audit -W .github/workflows/security-audit.yml --container-architecture linux/amd64
```

### Common Workflows to Test

1. **Security Audit**
```bash
act -j security-audit --container-architecture linux/amd64
```

2. **Rust Checks (CI)**
```bash
act -j rust-checks --container-architecture linux/amd64
```

3. **Performance Monitoring**
```bash
act -j benchmark-critical-paths --container-architecture linux/amd64
```

4. **Build Binaries**
```bash
act -j build-linux --container-architecture linux/amd64
```

## Direct Local Testing (Faster)

Instead of using `act`, you can run the same checks directly:

### 1. Clippy (Security Linting)
```bash
# Run clippy as Security Audit does
cargo clippy --workspace --lib --bins --all-features -- -D warnings

# Separate production vs test code checks
cargo clippy --workspace --lib --bins --all-features -- \
  -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic

# All targets (less strict for tests)
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### 2. Tests
```bash
# Run all workspace tests
cargo test --workspace --lib --bins

# Run with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test --workspace integration_
```

### 3. P2P Performance Test
```bash
# Run multi-node P2P test
NUM_NODES=3 TEST_DURATION=60 ./scripts/p2p-performance-test.sh

# Quick test with 2 nodes
NUM_NODES=2 TEST_DURATION=10 ./scripts/p2p-performance-test.sh
```

### 4. Build Binaries
```bash
# Build headless binary
cd communitas-headless
cargo build --release --bin communitas-headless

# Build desktop app
cd communitas-desktop  
cargo build --release
```

### 5. Format Check
```bash
# Check formatting
cargo fmt --all -- --check

# Auto-fix formatting
cargo fmt --all
```

## Test Helper Script

Use the provided test script for automated testing:

```bash
# Run all tests
./scripts/test-workflows-locally.sh all

# Run specific test suites
./scripts/test-workflows-locally.sh quick    # Fast JS tests
./scripts/test-workflows-locally.sh rust     # Rust checks
./scripts/test-workflows-locally.sh security # Security audit
./scripts/test-workflows-locally.sh perf     # Performance tests
./scripts/test-workflows-locally.sh build    # Build binaries
```

## Troubleshooting Act

### Docker Issues
```bash
# Check Docker is running
docker ps

# Clean up stuck act containers
docker stop $(docker ps -q --filter "name=act-")
docker rm $(docker ps -aq --filter "name=act-")
```

### Slow Performance
- Act downloads full Ubuntu images and installs dependencies each run
- Consider using direct commands for faster iteration
- Use `--reuse` flag to keep containers between runs

### Apple Silicon Issues
- Always use `--container-architecture linux/amd64` on M1/M2 Macs
- Some workflows may be slower due to emulation

## Verification Checklist

Before pushing changes, verify:

- [ ] ✅ Clippy passes: `cargo clippy --workspace --lib --bins --all-features -- -D warnings`
- [ ] ✅ Tests pass: `cargo test --workspace`
- [ ] ✅ Format correct: `cargo fmt --all -- --check`
- [ ] ✅ Builds succeed: `cargo build --release --workspace`
- [ ] ✅ P2P test runs: `./scripts/p2p-performance-test.sh`

## Current Status (as of latest test)

| Check | Status | Command |
|-------|--------|---------|
| Clippy | ✅ Pass | `cargo clippy --workspace --lib --bins --all-features -- -D warnings` |
| Tests | ✅ Pass | `cargo test --workspace --lib --bins` |
| Format | ✅ Pass | `cargo fmt --all -- --check` |
| P2P Test | ✅ Runs | `NUM_NODES=2 TEST_DURATION=10 ./scripts/p2p-performance-test.sh` |
| Build | ✅ Pass | `cargo build --release --bin communitas-headless` |

## Performance Metrics

Latest P2P test results:
- 2 nodes start successfully
- Ports 9000-9001 listening
- Metrics endpoints available on 9600-9601
- Network formation begins within 3 seconds

## CI/CD Alignment

These local tests mirror the GitHub Actions workflows:
- `security-audit.yml` → `cargo clippy` with strict flags
- `test-suite.yml` → `cargo test` across workspace
- `performance-monitoring.yml` → P2P performance test script
- `release-binaries.yml` → `cargo build --release`

Running these locally ensures your changes will pass CI/CD.