# Communitas MCP Server Deployment Guide

This guide covers deploying the Communitas MCP server to production infrastructure.

## Prerequisites

- Linux server (Ubuntu 22.04+ recommended)
- Domain name pointed to server IP
- Root/sudo access
- Nginx installed
- Certbot installed (for TLS certificates)

## Infrastructure Overview

The MCP server runs as a systemd service with nginx as a reverse proxy:

```
┌─────────────────────────────────────────────────────────────┐
│                    Internet                                  │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                  nginx (port 443)                           │
│  - TLS termination (Let's Encrypt)                          │
│  - Rate limiting (30 req/s)                                 │
│  - CORS headers for MCP Apps                                │
│  - Routes: /mcp, /health, /metrics, /ui/                    │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│              communitas-mcp (port 8443)                     │
│  - MCP JSON-RPC endpoint                                    │
│  - Health endpoint (JSON)                                   │
│  - UI resource serving                                      │
│  - Prometheus metrics                                       │
└─────────────────────────────────────────────────────────────┘
```

## Quick Deployment

### From macOS Development Machine

```bash
# 1. Cross-compile the binary
./deployment/deploy.sh build

# 2. Deploy to a server
./deployment/deploy.sh deploy saorsa-1   # Primary server
./deployment/deploy.sh deploy saorsa-7   # Secondary server
```

### Manual Deployment

```bash
# 1. Build the binary (on macOS)
cargo zigbuild --release --target x86_64-unknown-linux-gnu -p communitas-mcp

# 2. Copy to server
scp target/x86_64-unknown-linux-gnu/release/communitas-mcp root@server:/opt/communitas-mcp/

# 3. Set up systemd service
scp deployment/communitas-mcp.service root@server:/etc/systemd/system/
ssh root@server 'systemctl daemon-reload && systemctl enable communitas-mcp'

# 4. Start the service
ssh root@server 'systemctl start communitas-mcp'
```

## Configuration Files

### Systemd Service (`/etc/systemd/system/communitas-mcp.service`)

The service runs as a dedicated `communitas` user with security hardening:

```ini
[Unit]
Description=Communitas MCP Server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=communitas
Group=communitas
WorkingDirectory=/opt/communitas-mcp
ExecStart=/opt/communitas-mcp/communitas-mcp --http --tls --listen 0.0.0.0:8443 --no-client-auth --demo
Environment=RUST_LOG=info
Restart=always
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ReadWritePaths=/opt/communitas-mcp

[Install]
WantedBy=multi-user.target
```

### Nginx Configuration (`/etc/nginx/sites-available/communitas-mcp.conf`)

See `deployment/nginx-mcp.conf` for the full configuration. Key features:

- **Rate limiting**: 30 requests/second with burst of 50
- **TLS 1.3 only**: Modern security
- **CORS headers**: Required for MCP Apps widgets
- **Health endpoint**: No rate limiting for monitoring

### Directory Structure on Server

```
/opt/communitas-mcp/
├── communitas-mcp          # Binary
├── certs/
│   ├── fullchain.pem      # TLS certificate (symlink)
│   └── privkey.pem        # TLS private key (symlink)
└── data/                  # Runtime data (if needed)

/etc/
├── systemd/system/
│   └── communitas-mcp.service
└── nginx/
    └── sites-available/
        └── communitas-mcp.conf
```

## TLS Setup with Let's Encrypt

```bash
# Run the TLS setup script
./deployment/setup-tls.sh mcp.example.com admin@example.com

# Or manually:
certbot certonly --nginx -d mcp.example.com --email admin@example.com --agree-tos --non-interactive

# Create symlinks
ln -sf /etc/letsencrypt/live/mcp.example.com/fullchain.pem /opt/communitas-mcp/certs/fullchain.pem
ln -sf /etc/letsencrypt/live/mcp.example.com/privkey.pem /opt/communitas-mcp/certs/privkey.pem
```

Certificates auto-renew via the certbot systemd timer.

## Service Management

```bash
# Start/stop/restart
sudo systemctl start communitas-mcp
sudo systemctl stop communitas-mcp
sudo systemctl restart communitas-mcp

# Check status
sudo systemctl status communitas-mcp

# View logs
sudo journalctl -u communitas-mcp -f
sudo journalctl -u communitas-mcp --since "1 hour ago"

# Check if service is active
systemctl is-active communitas-mcp
```

## Health Monitoring

### Health Endpoint

```bash
curl https://mcp.example.com/health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

### Prometheus Metrics

Configure Prometheus to scrape the MCP server. See `deployment/prometheus-mcp.yml`:

```yaml
scrape_configs:
  - job_name: 'communitas-mcp'
    scheme: https
    static_configs:
      - targets:
        - 'mcp.saorsalabs.com:8443'
        labels:
          node: 'saorsa-1'
```

### Grafana Dashboard

Import `deployment/grafana-mcp-dashboard.json` for:
- Node status (up/down)
- Response time graphs
- Uptime tracking

## Multi-Node Deployment

For high availability, deploy to multiple nodes:

| Node | Domain | Role |
|------|--------|------|
| saorsa-1 | mcp.saorsalabs.com | Primary |
| saorsa-7 | mcp-secondary.saorsalabs.com | Secondary |

Both nodes run identical configurations. Use DNS-based load balancing or a load balancer for failover.

## Security Considerations

### Firewall Rules

```bash
# Allow HTTPS
ufw allow 443/tcp

# Allow direct MCP port only from localhost/monitoring
ufw allow from 127.0.0.1 to any port 8443
```

### Metrics Endpoint Access

The `/metrics` endpoint is restricted in nginx to localhost and the Prometheus server IP:

```nginx
location /metrics {
    allow 127.0.0.1;
    allow ::1;
    # Add Prometheus server IP:
    # allow 10.0.0.0/8;
    deny all;
    proxy_pass http://communitas_mcp;
}
```

### Security Headers

Nginx adds security headers:
- `X-Frame-Options: SAMEORIGIN`
- `X-Content-Type-Options: nosniff`
- `X-XSS-Protection: 1; mode=block`
- `Referrer-Policy: strict-origin-when-cross-origin`

## Troubleshooting

### Service Won't Start

```bash
# Check logs
journalctl -u communitas-mcp -n 50

# Verify binary exists and is executable
ls -la /opt/communitas-mcp/communitas-mcp

# Test binary directly
/opt/communitas-mcp/communitas-mcp --help

# Check user permissions
ls -la /opt/communitas-mcp/
```

### TLS Certificate Issues

```bash
# Check certificate status
certbot certificates

# Test certificate
openssl s_client -connect mcp.example.com:443 -servername mcp.example.com

# Renew manually
certbot renew --dry-run
```

### Connection Refused

```bash
# Check if service is running
systemctl status communitas-mcp

# Check if port is listening
ss -tlnp | grep 8443

# Check nginx configuration
nginx -t
systemctl status nginx
```

### High Memory/CPU

```bash
# Check resource usage
top -p $(pgrep communitas-mcp)

# Check for excessive connections
ss -s
netstat -ant | grep 8443 | wc -l
```

## Updating the Server

```bash
# 1. Build new binary
cargo zigbuild --release --target x86_64-unknown-linux-gnu -p communitas-mcp

# 2. Deploy update
scp target/x86_64-unknown-linux-gnu/release/communitas-mcp root@server:/opt/communitas-mcp/communitas-mcp.new
ssh root@server 'mv /opt/communitas-mcp/communitas-mcp.new /opt/communitas-mcp/communitas-mcp && systemctl restart communitas-mcp'

# 3. Verify
curl https://mcp.example.com/health
```

## Related Documentation

- [MCP API Reference](../api/mcp-api.md)
- [Claude Desktop Setup](../guides/claude-desktop-setup.md)
- [nginx Configuration](../../deployment/nginx-mcp.conf)
- [Prometheus Configuration](../../deployment/prometheus-mcp.yml)
