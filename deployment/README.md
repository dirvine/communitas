# Communitas MCP Server Deployment

This directory contains deployment files for running the Communitas MCP server on Saorsa Labs infrastructure.

## Infrastructure

| Node | IP Address | Role |
|------|------------|------|
| saorsa-1 | 77.42.75.115 | Primary MCP Server |
| saorsa-7 | 116.203.101.172 | Secondary MCP Server |

## Quick Deploy

```bash
# Cross-compile from macOS
cargo zigbuild --release --target x86_64-unknown-linux-gnu -p communitas-mcp

# Deploy to node
./deploy.sh saorsa-1
```

## Service Management

```bash
# Start service
sudo systemctl start communitas-mcp

# Stop service
sudo systemctl stop communitas-mcp

# View status
sudo systemctl status communitas-mcp

# View logs
journalctl -u communitas-mcp -f
```

## Files

| File | Purpose |
|------|---------|
| `communitas-mcp.service` | Systemd service unit |
| `deploy.sh` | Deployment script (TODO) |
| `nginx-mcp.conf` | Nginx reverse proxy config (TODO) |
| `setup-tls.sh` | TLS certificate setup (TODO) |

## Endpoints

| Endpoint | Purpose |
|----------|---------|
| `POST /mcp` | MCP JSON-RPC endpoint |
| `GET /health` | Health check |
| `GET /metrics` | Prometheus metrics |

## TLS Configuration

The MCP server uses Let's Encrypt certificates. Certificates are stored at:
- Certificate: `/etc/letsencrypt/live/<domain>/fullchain.pem`
- Private key: `/etc/letsencrypt/live/<domain>/privkey.pem`

## Monitoring

- Prometheus scrapes `/metrics` endpoint
- Grafana dashboard: `grafana-mcp-dashboard.json`
- Alerts configured for request errors and high latency

## Troubleshooting

### Service won't start

1. Check permissions: `ls -la /opt/communitas-mcp/`
2. Check user exists: `id communitas`
3. Check logs: `journalctl -u communitas-mcp -n 50`

### TLS errors

1. Verify certificates exist
2. Check certificate permissions
3. Verify domain matches certificate

### Connection refused

1. Check service is running: `systemctl status communitas-mcp`
2. Check firewall: `ufw status`
3. Check port binding: `ss -tlnp | grep 8443`
