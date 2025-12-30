# Communitas Testing Infrastructure

Comprehensive testing infrastructure for Communitas P2P collaboration platform.

## Quick Start

```bash
# Deploy to VPS fleet
./scripts/deploy-update.sh direct all

# Check network health
./scripts/verify-network.sh full

# Start local NAT emulation
cd docker/nat-emulation && docker-compose up -d
```

## Testing Components

### 1. VPS Fleet Testing
- 9-node distributed fleet across 3 cloud providers
- Bootstrap nodes (saorsa-2, saorsa-3) for network initialization
- Test nodes (saorsa-4 through saorsa-9) for various scenarios

### 2. NAT Emulation (Docker)
- 7 NAT types emulated locally
- Full connectivity matrix testing
- Worst-case scenario validation (symmetric-to-symmetric)

### 3. Remote GUI Testing (VNC)
- VNC servers on test nodes
- SSH tunnel access for secure connections
- WebRTC call verification across NAT boundaries

### 4. Automated Development Loop
- Build -> Deploy -> Test -> Iterate
- Auto-update capability for rapid iteration
- Health monitoring and alerts

## Directory Structure

```
docs/testing/
├── README.md                     # This file
├── nat-emulation/
│   ├── docker-setup.md           # Local Docker NAT setup
│   └── nat-scenarios.md          # NAT type matrix and scenarios
├── multi-node-testing/
│   ├── vps-fleet-testing.md      # VPS fleet procedures
│   └── crdt-sync-verification.md # CRDT sync testing
├── vnc-remote-gui/
│   └── vnc-server-setup.md       # VNC configuration guide
└── automated-loop/
    └── README.md                 # Development loop workflow
```

## Key Scripts

| Script | Purpose |
|--------|---------|
| `scripts/test-fleet.sh` | VPS fleet orchestration |
| `scripts/deploy-update.sh` | Build and deploy |
| `scripts/verify-network.sh` | Health checks |
| `scripts/vnc-connect.sh` | VNC connection helper |
| `docker/nat-emulation/test-nat-matrix.sh` | NAT connectivity tests |

## VPS Fleet

| Node | Location | IP | Role | Port |
|------|----------|-----|------|------|
| saorsa-1 | Helsinki | 77.42.75.115 | Dashboard | - |
| saorsa-2 | NYC | 142.93.199.50 | Bootstrap | 11000 |
| saorsa-3 | SFO | 147.182.234.192 | Bootstrap | 11000 |
| saorsa-4 | AMS | 206.189.7.117 | Test | 11000 |
| saorsa-5 | LON | 144.126.230.161 | Test | 11000 |
| saorsa-6 | Helsinki | 65.21.157.229 | Test | 11000 |
| saorsa-7 | Nuremberg | 116.203.101.172 | Test | 11000 |
| saorsa-8 | Singapore | 149.28.156.231 | Test | 11000 |
| saorsa-9 | Tokyo | 45.77.176.184 | Test | 11000 |

## NAT Types Tested

| Type | Docker | Difficulty | Description |
|------|--------|------------|-------------|
| Full Cone | `nat-fullcone` | Easy | Most permissive |
| Address-Restricted | `nat-restricted` | Medium | IP-based filtering |
| Port-Restricted | `nat-portrestricted` | Medium | IP:port filtering |
| Symmetric | `nat-symmetric` | Very Hard | Different port per destination |
| CGNAT | `nat-cgnat` | Hard | Limited port pool |
| Double NAT | `nat-doublenat-*` | Very Hard | Two NAT layers |
| Hairpin | `nat-hairpin` | Special | Self-connectivity |

## Common Workflows

### Deploy and Verify

```bash
# 1. Build release binary
./scripts/deploy-update.sh build --release

# 2. Deploy to all VPS nodes
./scripts/deploy-update.sh direct all

# 3. Verify network health
./scripts/verify-network.sh full

# 4. Generate health report
./scripts/verify-network.sh report > health-$(date +%Y%m%d).json
```

### NAT Testing

```bash
# 1. Start NAT emulation
cd docker/nat-emulation
docker-compose up -d

# 2. Run connectivity matrix
./test-nat-matrix.sh full

# 3. Test specific pair
./test-nat-matrix.sh pair Symmetric CGNAT
```

### VNC Remote Testing

```bash
# 1. Check VNC status
./scripts/vnc-connect.sh status

# 2. Install VNC on node (first time only)
./scripts/vnc-connect.sh install saorsa-4

# 3. Connect to node
./scripts/vnc-connect.sh saorsa-4
```

## Success Metrics

| Metric | Target | Description |
|--------|--------|-------------|
| CRDT Sync | 100% | All nodes synchronized |
| NAT Traversal | >95% | Successful hole-punching |
| Deploy Time | <5 min | Fleet update propagation |
| Message Latency | <500ms | Cross-region delivery |

## Troubleshooting

### VPS Connection Issues
```bash
# Check SSH connectivity
./scripts/verify-network.sh connectivity

# Check service status
./scripts/verify-network.sh services

# View logs
./scripts/test-fleet.sh logs saorsa-2
```

### NAT Emulation Issues
```bash
# Rebuild containers
cd docker/nat-emulation
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

### Build Issues
```bash
# Full rebuild
cargo clean
cargo build -p communitas-headless --release
```

## References

- [NAT Traversal RFC 4787](https://datatracker.ietf.org/doc/html/rfc4787)
- [STUN RFC 3489](https://datatracker.ietf.org/doc/html/rfc3489)
- [Infrastructure Documentation](../infrastructure/INFRASTRUCTURE.md)
