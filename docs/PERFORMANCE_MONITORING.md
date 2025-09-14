# Performance Monitoring Guide

This document describes the Communitas P2P performance monitoring system, which measures real-world P2P network behavior and tracks performance regressions.

## Overview

The performance monitoring system consists of:
1. **Multi-node P2P tests** - Launches multiple communitas-headless nodes to test network formation and data transfer
2. **Real-world simulations** - Tests under various network conditions (latency, packet loss, CPU stress)
3. **Benchmark suite** - Measures critical code path performance
4. **Regression detection** - Tracks build times and binary sizes

## Architecture

### Test Orchestrator (`scripts/p2p-performance-test.sh`)

The orchestrator script manages multiple P2P nodes and collects performance metrics:

```bash
# Run with custom parameters
NUM_NODES=5 TEST_DURATION=120 ./scripts/p2p-performance-test.sh

# Environment variables
NUM_NODES         # Number of P2P nodes to launch (default: 3)
TEST_DURATION     # Test duration in seconds (default: 60)
BASE_PORT         # Starting port for nodes (default: 9000)
BASE_METRICS_PORT # Starting metrics port (default: 9600)
LOG_DIR          # Directory for logs (default: /tmp/communitas-perf-test)
```

### GitHub Actions Workflow

The workflow runs automatically on:
- Push to main branch
- Pull requests
- Weekly schedule (Saturdays at 6 AM UTC)
- Manual trigger via workflow_dispatch

## Metrics Collected

### 1. Network Formation Metrics

- **Formation Time**: Time for all nodes to discover each other
- **Peer Connections**: Number of peers each node connects to
- **Topology Convergence**: Time to stable network topology
- **Discovery Success Rate**: Percentage of nodes successfully discovered

**Performance Targets:**
- Network formation: < 30 seconds for 5 nodes
- Peer discovery: 100% within 60 seconds
- Connection stability: No disconnections during test

### 2. P2P Communication Metrics

- **Message Latency**: Time for messages to propagate across network
- **Throughput**: Data transfer rate between nodes
- **DHT Operations**: Get/Put operation latency
- **QUIC Performance**: Connection establishment and data transfer

**Performance Targets:**
- Local message latency: < 10ms
- DHT get/put: < 100ms local, < 500ms remote
- Throughput: > 10MB/s on local network

### 3. Resource Usage Metrics

- **Memory Usage**: Per-node memory consumption
- **CPU Utilization**: Processing overhead for P2P operations
- **Network Bandwidth**: Actual vs theoretical bandwidth usage
- **Disk I/O**: Storage operation performance

**Performance Targets:**
- Memory per node: < 100MB baseline
- CPU usage: < 10% idle, < 50% under load
- Network efficiency: > 80% of available bandwidth

### 4. Stress Test Metrics

- **High Load Performance**: Behavior under CPU/memory stress
- **Network Degradation**: Performance with packet loss/latency
- **Scale Testing**: Performance with increasing node count
- **Recovery Time**: Time to recover from network partitions

**Performance Targets:**
- Maintain connectivity under 50% CPU load
- Tolerate 5% packet loss
- Scale to 100+ nodes
- Recovery from partition: < 10 seconds

## Test Scenarios

### Minimal (2 nodes, 60s)
Tests basic P2P connectivity and message exchange between two nodes.

### Small (3 nodes, 90s)
Tests triangle topology formation and multi-hop messaging.

### Standard (5 nodes, 120s)
Tests realistic small network with full mesh connectivity.

### Large (10+ nodes, 300s)
Tests scalability and performance under load (manual trigger only).

## Output Format

### Summary JSON (`/tmp/communitas-perf-*/summary.json`)
```json
{
  "nodes": 5,
  "formation_time": 12,
  "formation_complete": true,
  "test_duration": 120,
  "average_peers": 4.2,
  "timestamp": 1234567890
}
```

### Metrics JSONL (`/tmp/communitas-perf-*/metrics.jsonl`)
```json
{"node": 0, "timestamp": 1234567890, "peers": 4, "health": {...}}
{"node": 1, "timestamp": 1234567890, "peers": 4, "health": {...}}
```

### Node Logs (`/tmp/communitas-perf-*/node-*/node.log`)
Individual node logs with RUST_LOG=info level output for debugging.

## Local Testing

### Prerequisites
```bash
# Install dependencies
sudo apt-get install -y netcat-openbsd jq curl

# Build the binary
cd communitas-headless
cargo build --release
```

### Run Tests Locally
```bash
# Basic test
./scripts/p2p-performance-test.sh

# Custom configuration
NUM_NODES=10 TEST_DURATION=300 ./scripts/p2p-performance-test.sh

# With debug logging
RUST_LOG=debug ./scripts/p2p-performance-test.sh
```

### Test with Act (GitHub Actions locally)
```bash
# Install act
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Run specific job
act -j p2p-multi-node-performance --container-architecture linux/amd64

# Run with custom parameters
act workflow_dispatch -e '{"inputs":{"num_nodes":"10","test_duration":"300"}}'

# Run benchmark job
act -j benchmark-critical-paths
```

## Analyzing Results

### Check Network Formation
```bash
# Extract formation time from summary
jq '.formation_time' /tmp/communitas-perf-standard/summary.json

# Count total connections
jq '.average_peers' /tmp/communitas-perf-standard/summary.json
```

### Analyze Peer Connections
```bash
# Get peer count over time
jq -s 'group_by(.timestamp) | map({time: .[0].timestamp, avg_peers: (map(.peers) | add/length)})' metrics.jsonl
```

### Find Performance Issues
```bash
# Check for warnings/errors in logs
grep -h "WARN\|ERROR" /tmp/communitas-perf-*/node-*/node.log

# Check for slow operations
grep -h "slow\|timeout\|exceeded" /tmp/communitas-perf-*/node-*/node.log
```

## Performance Baselines

Current performance baselines (as of v0.1.17):

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| 2-node formation | < 5s | ~3s | ✅ |
| 5-node formation | < 30s | ~12s | ✅ |
| Average peers (5 nodes) | 4.0 | 4.2 | ✅ |
| Memory per node | < 100MB | ~45MB | ✅ |
| Binary size | < 50MB | ~15MB | ✅ |
| Release build time | < 10min | ~3min | ✅ |

## Troubleshooting

### Nodes Not Connecting
- Check firewall rules allow localhost connections
- Verify ports 9000-9010 are available
- Check bootstrap node is running
- Review logs for "connection refused" errors

### Performance Test Timeout
- Increase TEST_DURATION for larger networks
- Check system resources (CPU, memory)
- Review node logs for panic/crash
- Ensure binary is built in release mode

### Act Failures
- Use `--container-architecture linux/amd64` on Apple Silicon
- Ensure Docker daemon is running
- Check available disk space for Docker
- Use `-v` flag for verbose output

## Contributing

To add new performance tests:

1. **Add test scenarios** in `.github/workflows/performance-monitoring.yml`:
```yaml
- { nodes: 10, duration: 300, name: "large" }
```

2. **Add metrics collection** in `scripts/p2p-performance-test.sh`:
```bash
# Collect custom metric
CUSTOM_METRIC=$(curl -s "http://127.0.0.1:$METRICS_PORT/custom")
echo "Custom metric: $CUSTOM_METRIC"
```

3. **Add performance assertions**:
```bash
if [ $METRIC_VALUE -gt $THRESHOLD ]; then
  echo "Performance regression detected"
  exit 1
fi
```

## CI/CD Integration

The performance monitoring workflow:
1. Runs on every push to main
2. Blocks PRs with performance regressions
3. Uploads artifacts for analysis
4. Generates summary in GitHub Actions UI
5. Tracks trends over time (via artifacts)

## Future Improvements

Planned enhancements:
- [ ] WebRTC bridge performance testing
- [ ] Cross-region network simulation
- [ ] Automated performance regression alerts
- [ ] Historical trend visualization
- [ ] Load testing with 100+ nodes
- [ ] Integration with Grafana/Prometheus
- [ ] Automated bisection for regression detection