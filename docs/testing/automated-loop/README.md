# Automated Development Loop

Rapid iteration workflow: Code -> Build -> Deploy -> Test -> Iterate

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    DEVELOPMENT LOOP                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   1. CODE         2. BUILD         3. DEPLOY      4. VERIFY     │
│   ┌─────┐        ┌─────┐         ┌─────┐        ┌─────┐        │
│   │Edit │───────>│Cargo│────────>│Fleet│───────>│Test │        │
│   │Rust │        │Build│         │Deploy│       │Matrix│        │
│   └─────┘        └─────┘         └─────┘        └─────┘        │
│      ^                                              │           │
│      │                                              │           │
│      └──────────────── ITERATE ─────────────────────┘           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# Full cycle: build + deploy + verify
./scripts/deploy-update.sh full 0.2.0
```

## Step-by-Step Workflow

### 1. Code Changes
```bash
# Edit source files
$EDITOR communitas-core/src/lib.rs

# Run local tests
cargo test -p communitas-core
```

### 2. Build Release
```bash
# Build headless binary
./scripts/deploy-update.sh build --release

# Verify binary
ls -la target/release/communitas-headless
```

### 3. Deploy to Fleet

#### Option A: Direct Deploy (Immediate)
```bash
# Deploy to all VPS nodes
./scripts/deploy-update.sh direct all

# Deploy to specific node
./scripts/deploy-update.sh direct saorsa-4
```

#### Option B: GitHub Release (With Auto-Update)
```bash
# Create GitHub release
./scripts/deploy-update.sh github-release 0.2.0

# Nodes auto-update from release
./scripts/deploy-update.sh trigger-check
```

### 4. Verify Deployment
```bash
# Full network verification
./scripts/verify-network.sh full

# Generate health report
./scripts/verify-network.sh report > health.json
```

### 5. Run Tests
```bash
# E2E tests against VPS fleet
cargo test -p communitas-core --test infrastructure_e2e -- --nocapture

# NAT matrix tests (local Docker)
cd docker/nat-emulation
./test-nat-matrix.sh full
```

## Auto-Update Integration

### Iced App (Rust)
- Checks GitHub releases on startup
- Downloads and replaces binary automatically
- Shows update banner in UI

### Swift App (macOS)
- Uses Sparkle framework
- Checks appcast.xml from GitHub releases
- Silent background updates

### Headless Nodes (VPS)
- Direct deploy via SCP
- Or pull from GitHub releases
- Systemd service restart

## Monitoring During Iteration

### Watch Fleet Status
```bash
watch -n 5 ./scripts/verify-network.sh quick
```

### Stream Logs
```bash
./scripts/test-fleet.sh logs saorsa-2
```

### Monitor Gossip Health
```bash
./scripts/verify-network.sh gossip
```

## CI/CD Integration

### GitHub Actions Trigger
When a release is created:
1. `release-headless.yml` builds binaries
2. Uploads to GitHub release
3. Generates appcast.xml for Sparkle
4. Nodes auto-update on check

### Manual Trigger
```bash
# Force nodes to check for updates
./scripts/deploy-update.sh trigger-check
```

## Common Workflows

### Hot Fix Deployment
```bash
# 1. Fix bug
$EDITOR communitas-core/src/networking.rs

# 2. Test locally
cargo test

# 3. Build and deploy immediately
./scripts/deploy-update.sh build --release
./scripts/deploy-update.sh direct all

# 4. Verify
./scripts/verify-network.sh full
```

### Feature Testing on Single Node
```bash
# Deploy to test node only
./scripts/deploy-update.sh direct saorsa-4

# Test on that node
./scripts/test-fleet.sh ssh saorsa-4
```

### Full Release Cycle
```bash
# 1. Bump version
$EDITOR Cargo.toml  # Update version

# 2. Commit and tag
git add -A
git commit -m "release: v0.2.0"
git tag v0.2.0

# 3. Full deployment
./scripts/deploy-update.sh full 0.2.0

# 4. Push (triggers CI)
git push origin main --tags
```

## Timing Targets

| Step | Target | Description |
|------|--------|-------------|
| Build | <2 min | Release build |
| Deploy | <3 min | All 8 nodes |
| Verify | <1 min | Health check |
| E2E Tests | <5 min | Full suite |

**Total iteration cycle: <10 minutes**

## Troubleshooting

### Deploy Fails
```bash
# Check SSH connectivity
./scripts/verify-network.sh connectivity

# Check specific node
./scripts/test-fleet.sh ssh saorsa-2
```

### Tests Fail After Deploy
```bash
# Check service status
./scripts/verify-network.sh services

# View service logs
./scripts/test-fleet.sh logs saorsa-2
```

### Version Mismatch
```bash
# Check running versions
for node in saorsa-{2..9}; do
    echo "$node: $(ssh root@$node.saorsalabs.com '/opt/communitas/communitas-headless --version')"
done
```
