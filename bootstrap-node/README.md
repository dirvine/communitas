# Bootstrap Node

Network bootstrap and discovery service for Communitas.

## Overview

The bootstrap node provides initial peer discovery and network topology management for the Communitas P2P network. It helps new nodes discover peers and maintains a healthy network topology.

## Features

- **DHT Bootstrap Service**: Provides initial peer discovery for new nodes joining the network
- **Peer Discovery**: Supports both mDNS for local discovery and global DHT-based discovery
- **Network Health Monitoring**: Tracks peer status and network topology
- **Geographic Routing**: Optimizes peer connections based on geographic proximity
- **Multiple Transport Support**: QUIC and custom transport protocols

## Architecture

### Components

- **DHT Layer**: Distributed hash table for peer routing
- **Discovery Service**: Handles peer discovery via multiple protocols
- **Health Monitor**: Monitors peer health and network statistics
- **Geographic Router**: Routes connections based on latency and geographic data

### Network Protocol

- **Transport**: QUIC over UDP (default port 8080)
- **Encryption**: TLS 1.3 with post-quantum fallback
- **Peer IDs**: Based on ML-DSA public keys

## Deployment

### Docker

```bash
docker run -p 8080:8080 communitas/bootstrap-node
```

### From Source

```bash
cargo build --release -p bootstrap-node
./target/release/bootstrap-node --config config.toml
```

### Configuration

Create a `config.toml` file:

```toml
[network]
listen_addr = "0.0.0.0:8080"
discovery_interval = 30
max_peers = 5000

[storage]
path = "/var/lib/communitas/bootstrap"

[monitoring]
metrics_port = 9090
enable_prometheus = true

[logging]
level = "info"
format = "json"
```

## Environment Variables

- `COMMUNITAS_LISTEN_ADDR` - Override listen address (default: `0.0.0.0:8080`)
- `COMMUNITAS_STORAGE_PATH` - Data storage path
- `RUST_LOG` - Logging level (e.g., `info`, `debug`)

## Monitoring

### Metrics

Prometheus metrics available at `:9090/metrics`:

- `communitas_peers_connected` - Number of connected peers
- `communitas_discovery_requests` - Peer discovery request count
- `communitas_network_latency` - Average network latency
- `communitas_bandwidth_usage` - Network bandwidth usage

### Health Check

```bash
curl http://localhost:8080/health
```

## Operations

### Deployment Best Practices

1. **High Availability**: Deploy multiple bootstrap nodes in different geographic regions
2. **Load Balancing**: Use DNS-based load balancing for bootstrap endpoints
3. **Monitoring**: Set up Prometheus + Grafana for network monitoring
4. **Backup**: Regular backups of peer database

### Scaling

- Single node can handle ~5,000 concurrent peers
- Deploy regional clusters for larger networks
- Use geographic routing for optimal performance

## Security

- All peer connections use TLS 1.3
- Post-quantum cryptography support (ML-KEM, ML-DSA)
- DDoS protection via rate limiting
- Peer authentication via ML-DSA signatures

## Development

### Building

```bash
cargo build -p bootstrap-node
```

### Testing

```bash
cargo test -p bootstrap-node
```

### Local Testing

```bash
# Terminal 1: Start bootstrap node
cargo run -p bootstrap-node -- --config test-config.toml

# Terminal 2: Test connection
curl http://localhost:8080/peers
```

## Troubleshooting

### Port Already in Use

```bash
# Check what's using the port
lsof -i :8080

# Use a different port
cargo run -p bootstrap-node -- --listen 0.0.0.0:9080
```

### High Memory Usage

- Reduce `max_peers` in configuration
- Enable peer eviction for inactive connections
- Monitor with `htop` or similar tools

### Connection Issues

- Check firewall rules (UDP port 8080 must be open)
- Verify TLS certificates are valid
- Check DNS resolution for discovery

## Contributing

See [../../docs/development/contributing.md](../../docs/development/contributing.md) for contribution guidelines.

## License

Dual-licensed under AGPL-3.0-or-later and commercial license.
See [../../LICENSE-AGPL-3.0](../../LICENSE-AGPL-3.0) and [../../LICENSE-COMMERCIAL.md](../../LICENSE-COMMERCIAL.md).

## See Also

- [Communitas Core](../communitas-core/README.md)
- [Network Architecture](../../docs/architecture/networking.md)
- [Operations Guide](../../docs/operations/README.md)
