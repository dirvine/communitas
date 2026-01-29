# Testnet Deployment Documentation

## Overview

This document describes the MCP server deployment on Saorsa Labs VPS testnet infrastructure for Phase 10.8.

**Deployment Date**: 2026-01-29
**Phase**: 10.8 - Testnet Deployment
**Binary Version**: 0.8.2
**Binary Source**: GitHub Actions CI (run 21482833333)

## Node Inventory

### Production Nodes (Deployed)

| Node | Region | Provider | IP | Port | Status | External Access |
|------|--------|----------|----|----|--------|----------------|
| saorsa-2 | NYC1, US | DigitalOcean | 142.93.199.50 | 3040 | ✓ Running | ✓ Yes |
| saorsa-3 | SFO3, US | DigitalOcean | 147.182.234.192 | 3040 | ✓ Running | ✓ Yes |
| saorsa-7 | Nuremberg, DE | Hetzner | 116.203.101.172 | 3040 | ✓ Running | ✗ Firewall blocked |

### Node Roles

- **saorsa-2 (NYC)**: Primary test node, US East Coast
- **saorsa-3 (SFO)**: Secondary test node, US West Coast
- **saorsa-7 (Nuremberg)**: EU test node (firewall configuration needed)

### Node Identities

Each node runs with a unique demo identity:

- **saorsa-2**: yukon-pluto-muslim-helmet
- **saorsa-3**: sheriff-band-caesar-arson
- **saorsa-7**: toss-cheap-asylum-insect

## Deployment Architecture

### Service Configuration

- **Service Name**: `communitas-mcp-test.service`
- **Binary Location**: `/opt/communitas-test/communitas-mcp`
- **Working Directory**: `/opt/communitas-test`
- **Data Directory**: `/tmp/communitas-mcp-http-demo`
- **Listen Address**: `0.0.0.0:3040`
- **Transport**: HTTP (no TLS - testnet only)
- **Auth Mode**: Demo mode (no authentication)
- **User**: root (testnet only)

### Resource Limits

- **Memory Limit**: 512MB
- **File Descriptors**: 65535
- **Restart Policy**: On failure, 5s delay

### Environment Variables

```bash
RUST_LOG=info
RUST_BACKTRACE=1
```

## Deployment Process

### Prerequisites

1. **Build Tools** (local machine):
   - cargo zigbuild
   - zig compiler
   - SSH access to testnet nodes

2. **CI/CD**:
   - GitHub Actions for Linux builds
   - Release artifacts uploaded

3. **Network**:
   - SSH access to root@<node-ip>
   - Firewall rules for port 3040 (DigitalOcean nodes)

### Deployment Steps

1. **Build Binary** (via CI or local):
   ```bash
   # Option 1: Use GitHub Actions artifact
   gh run download <run-id> --name communitas-linux-x86_64

   # Option 2: Build locally
   cargo zigbuild --release --target x86_64-unknown-linux-gnu -p communitas-mcp
   ```

2. **Deploy to Node**:
   ```bash
   # Stop existing service
   ssh root@<node-ip> 'systemctl stop communitas-mcp-test 2>/dev/null || true'

   # Copy binary
   scp communitas-mcp root@<node-ip>:/opt/communitas-test/
   ssh root@<node-ip> 'chmod +x /opt/communitas-test/communitas-mcp'

   # Create systemd service (see systemd-service.md)
   # ...

   # Start service
   ssh root@<node-ip> 'systemctl daemon-reload'
   ssh root@<node-ip> 'systemctl enable communitas-mcp-test'
   ssh root@<node-ip> 'systemctl start communitas-mcp-test'
   ```

3. **Verify Deployment**:
   ```bash
   # Check service status
   ssh root@<node-ip> 'systemctl status communitas-mcp-test'

   # Test health endpoint
   curl http://<node-ip>:3040/health

   # List MCP tools
   curl -X POST http://<node-ip>:3040/mcp \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' | jq '.result.tools | length'
   ```

### Using Deployment Script

```bash
# Deploy to all nodes
./scripts/deploy-mcp-testnet.sh

# Deploy to specific nodes
./scripts/deploy-mcp-testnet.sh saorsa-2 saorsa-3

# Build locally and deploy
./scripts/deploy-mcp-testnet.sh -b

# Clean and redeploy
./scripts/deploy-mcp-testnet.sh -c

# Show status
./scripts/deploy-mcp-testnet.sh -s

# Teardown testnet
./scripts/deploy-mcp-testnet.sh -t
```

## Health Check Procedures

### Quick Health Check

```bash
# Check all nodes
for node in 142.93.199.50 147.182.234.192 116.203.101.172; do
  echo -n "$node: "
  curl -s --max-time 5 http://$node:3040/health | jq -r .status || echo "TIMEOUT"
done
```

### Detailed Service Check

```bash
ssh root@<node-ip> << 'EOF'
  echo "=== Service Status ==="
  systemctl status communitas-mcp-test

  echo ""
  echo "=== Memory Usage ==="
  systemctl show communitas-mcp-test --property=MemoryCurrent

  echo ""
  echo "=== Recent Logs ==="
  journalctl -u communitas-mcp-test -n 20 --no-pager

  echo ""
  echo "=== Network Listener ==="
  ss -tlnp | grep 3040
EOF
```

### MCP Endpoint Health

```bash
# Test tools/list endpoint
curl -X POST http://<node-ip>:3040/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' \
  | jq '.result.tools | length'

# Test a simple tool
curl -X POST http://<node-ip>:3040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"tools/call",
    "params":{"name":"identity_current","arguments":{}},
    "id":1
  }' | jq .
```

## Troubleshooting

### Service Won't Start

1. **Check logs**:
   ```bash
   journalctl -u communitas-mcp-test -n 50 --no-pager
   ```

2. **Verify binary**:
   ```bash
   file /opt/communitas-test/communitas-mcp
   ldd /opt/communitas-test/communitas-mcp
   ```

3. **Check permissions**:
   ```bash
   ls -la /opt/communitas-test/communitas-mcp
   # Should be: -rwxr-xr-x 1 root root
   ```

4. **Test manual start**:
   ```bash
   /opt/communitas-test/communitas-mcp --http --demo --listen 0.0.0.0:3040
   ```

### Port Not Accessible Externally

1. **Check firewall** (UFW):
   ```bash
   ufw status
   ufw allow 3040/tcp  # If needed
   ```

2. **Check cloud firewall** (DigitalOcean/Hetzner):
   - DigitalOcean: Networking → Firewalls
   - Hetzner: Cloud Console → Firewalls
   - Add rule: TCP port 3040, source 0.0.0.0/0

3. **Check service is listening**:
   ```bash
   ss -tlnp | grep 3040
   # Should show: 0.0.0.0:3040
   ```

### High Memory Usage

1. **Check current usage**:
   ```bash
   systemctl show communitas-mcp-test --property=MemoryCurrent
   ```

2. **Restart service** (if near limit):
   ```bash
   systemctl restart communitas-mcp-test
   ```

3. **Adjust memory limit** (if needed):
   ```bash
   # Edit service file
   vim /etc/systemd/system/communitas-mcp-test.service
   # Change MemoryMax=512M to higher value
   systemctl daemon-reload
   systemctl restart communitas-mcp-test
   ```

### Connection Timeouts

1. **Test from node itself**:
   ```bash
   ssh root@<node-ip> 'curl -s http://localhost:3040/health'
   ```

2. **If localhost works but external doesn't**:
   - Check cloud firewall rules
   - Check service is bound to 0.0.0.0 (not 127.0.0.1)

3. **If both fail**:
   - Service likely crashed - check logs
   - Restart service

## Performance Metrics

### Expected Performance

- **Startup Time**: <5 seconds
- **Initial Memory**: 3-4 MB
- **Steady State Memory**: 5-10 MB
- **Memory Limit**: 512 MB
- **Health Check Latency**: <50ms (local), <500ms (cross-region)
- **Tool Call Latency**: <100ms (simple), <500ms (complex)

### Latency Matrix

| Route | Latency |
|-------|---------|
| NYC → SFO | ~121ms |
| NYC → EU | Blocked (firewall) |
| SFO → EU | Blocked (firewall) |

## Known Issues

### 1. Hetzner Cloud Firewall Blocks Port 3040

**Severity**: Medium
**Impact**: saorsa-7 not accessible from other nodes
**Workaround**: Tests run on saorsa-2 and saorsa-3 only
**Fix**: Configure Hetzner Cloud Firewall rules in Phase 10.9

**Resolution**:
1. Log into Hetzner Cloud Console
2. Navigate to Firewalls
3. Add rule: TCP port 3040, source 0.0.0.0/0
4. Apply to saorsa-7

### 2. Tool Count Variance

**Severity**: Low
**Impact**: Different nodes report 187-194 tools
**Cause**: jq availability affects count method
**Workaround**: Use JSON-RPC tools/list for accurate count

## Security Considerations

⚠️ **TESTNET ONLY - DO NOT USE IN PRODUCTION**

This deployment uses:
- HTTP without TLS encryption
- Demo mode without authentication
- Root user privileges
- Public internet exposure

For production deployment:
- Enable TLS with ML-DSA certificates
- Remove demo mode
- Use dedicated service user
- Implement authentication
- Use private network or VPN
- Enable rate limiting
- Add monitoring and alerting

## Maintenance

### Log Management

```bash
# View recent logs
journalctl -u communitas-mcp-test -n 100 --no-pager

# Follow logs
journalctl -u communitas-mcp-test -f

# Clean old logs (keep 1 day)
journalctl --vacuum-time=1d
```

### Binary Updates

```bash
# Stop service
systemctl stop communitas-mcp-test

# Backup old binary
cp /opt/communitas-test/communitas-mcp /opt/communitas-test/communitas-mcp.bak

# Copy new binary
scp new-binary root@<node-ip>:/opt/communitas-test/communitas-mcp

# Restart service
systemctl start communitas-mcp-test
```

### Clean Shutdown

```bash
# Stop service
systemctl stop communitas-mcp-test

# Disable autostart
systemctl disable communitas-mcp-test

# Clean data (optional)
rm -rf /tmp/communitas-mcp-http-demo

# Clean logs (optional)
journalctl --vacuum-time=1d
```

## Next Steps (Phase 10.9)

1. **Fix Hetzner Firewall**: Configure port 3040 access for saorsa-7
2. **Add NAT Traversal Tests**: Use saorsa-4, 5, 6, 10 for NAT testing
3. **Multi-node CRDT Sync**: Test distributed CRDT synchronization
4. **Network Partition Tests**: Simulate network splits and recovery
5. **Gossip Protocol Tests**: Verify peer discovery and message routing
6. **Geographic Latency Tests**: Measure Asia-Pacific nodes (saorsa-8, 9)

## References

- Deployment Script: `scripts/deploy-mcp-testnet.sh`
- Service Template: `deployment/communitas-mcp.service`
- Testnet Status: `.planning/testnet-status.json`
- Phase 10.8 Plan: `.planning/PLAN-phase-10.8.md`
- VPS Infrastructure: `~/Desktop/Devel/projects/saorsa-testnet/docs/infrastructure/VPS_INFRASTRUCTURE.md`
