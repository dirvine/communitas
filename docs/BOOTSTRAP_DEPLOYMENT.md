# Bootstrap Node Deployment Guide

This guide covers the complete process for deploying a Communitas bootstrap node on DigitalOcean.

## Prerequisites

- DigitalOcean account with API access
- `doctl` CLI tool installed (optional, for automation)
- SSH key added to DigitalOcean account

## Phase 1: Manual Droplet Creation

### Option 1: Using DigitalOcean Web UI

1. **Create Droplet:**
   - Go to https://cloud.digitalocean.com/droplets/new
   - Select Ubuntu 24.04 LTS (recommended)
   - Choose a size:
     - **Basic/Regular**: $6/month (1GB RAM, 1 vCPU, 25GB SSD) - Minimum
     - **Basic/Regular**: $12/month (2GB RAM, 1 vCPU, 50GB SSD) - Recommended
     - **Basic/Regular**: $18/month (2GB RAM, 2 vCPU, 60GB SSD) - Production
   - Select region (choose closest to target users):
     - `nyc3` - New York
     - `sfo3` - San Francisco
     - `lon1` - London
     - `fra1` - Frankfurt
     - `sgp1` - Singapore
   - Add your SSH key
   - Name: `communitas-bootstrap-1` (or similar)
   - Click "Create Droplet"

2. **Note the IP Address:**
   - Copy the droplet's public IPv4 address
   - This will be used in the provisioning step

### Option 2: Using doctl CLI

```bash
# Install doctl if not already installed
# macOS: brew install doctl
# Linux: https://docs.digitalocean.com/reference/doctl/how-to/install/

# Authenticate
doctl auth init

# List available sizes
doctl compute size list

# List available regions
doctl compute region list

# Create droplet
doctl compute droplet create communitas-bootstrap-1 \
  --size s-1vcpu-2gb \
  --image ubuntu-24-04-x64 \
  --region nyc3 \
  --ssh-keys $(doctl compute ssh-key list --format ID --no-header) \
  --wait

# Get droplet IP
doctl compute droplet list --format Name,PublicIPv4
```

## Phase 2: Provision Bootstrap Node

Once the droplet is created, provision it with the bootstrap node software:

```bash
# SSH into the droplet
ssh root@YOUR_DROPLET_IP

# Run the provisioning script
curl -sSL https://raw.githubusercontent.com/dirvine/communitas/main/scripts/provision-bootstrap.sh | bash

# Alternative: With custom settings
GITHUB_REPO=dirvine/communitas \
AUTO_UPDATE=true \
ENABLE_METRICS=true \
curl -sSL https://raw.githubusercontent.com/dirvine/communitas/main/scripts/provision-bootstrap.sh | bash
```

The provisioning script will:
1. Install system dependencies
2. Create service user and directories
3. Download latest communitas-headless binary
4. Configure systemd service with security hardening
5. Set up auto-update timer (runs every 6 hours)
6. Start the bootstrap node service
7. Extract and display bootstrap endpoint information

## Phase 3: Configure Firewall

After provisioning, configure the firewall to allow necessary traffic:

```bash
# SSH into the droplet
ssh root@YOUR_DROPLET_IP

# Allow P2P traffic (required)
ufw allow 8080/tcp

# Allow metrics endpoint (optional, for monitoring)
ufw allow 9600/tcp

# Allow SSH (if not already allowed)
ufw allow 22/tcp

# Enable firewall
ufw --force enable

# Check status
ufw status verbose
```

## Phase 4: Verify Bootstrap Node

### Check Service Status

```bash
# Check service is running
systemctl status communitas-bootstrap

# View logs
journalctl -u communitas-bootstrap -f

# Check metrics endpoint
curl http://localhost:9600/metrics
```

### Extract Bootstrap Information

The provisioning script saves bootstrap information to `/opt/communitas/bootstrap-info.txt`:

```bash
cat /opt/communitas/bootstrap-info.txt
```

Example output:
```
Bootstrap Node Information
Generated: 2025-10-12 15:30:00 UTC

Public IP: 138.197.10.50
Port: 8080
Four-Word Address: ocean-forest-moon-star
Full Endpoint: 138.197.10.50:8080

Metrics Endpoint: http://138.197.10.50:9600/metrics

Service: communitas-bootstrap.service
Data Directory: /var/lib/communitas
Binary: /opt/communitas/communitas-headless
Version: communitas-headless 0.1.1
```

### Test Network Connectivity

From a local machine:

```bash
# Test bootstrap endpoint (replace with your droplet IP)
curl http://138.197.10.50:8080/health

# Test metrics endpoint
curl http://138.197.10.50:9600/metrics
```

## Phase 5: Update Desktop App Configuration

Once you have the bootstrap endpoint information, update the desktop app to use it:

1. **Extract Bootstrap Endpoint:**
   ```bash
   # SSH into droplet
   ssh root@YOUR_DROPLET_IP

   # Get four-word address from logs
   journalctl -u communitas-bootstrap --no-pager -n 100 | grep -oP 'Four-word address: \K[^\s]+'
   ```

2. **Update Desktop App:**

   Edit `communitas-core/src/constants.rs` (or appropriate config file):
   ```rust
   // Add your bootstrap node
   pub const BOOTSTRAP_PEERS: &[&str] = &[
       "ocean-forest-moon-star",  // Your bootstrap node
       // Add more bootstrap nodes here
   ];
   ```

3. **Rebuild Desktop App:**
   ```bash
   npm run build
   npm run tauri build
   ```

## Maintenance

### View Logs

```bash
# View service logs
journalctl -u communitas-bootstrap -f

# View auto-update logs
journalctl -u communitas-update -f
```

### Manual Updates

The bootstrap node auto-updates every 6 hours. To manually update:

```bash
# SSH into droplet
ssh root@YOUR_DROPLET_IP

# Run update script
/opt/communitas/update.sh

# Or restart service to trigger update check
systemctl restart communitas-bootstrap
```

### Service Management

```bash
# Check status
systemctl status communitas-bootstrap

# Restart service
systemctl restart communitas-bootstrap

# Stop service
systemctl stop communitas-bootstrap

# View configuration
cat /etc/systemd/system/communitas-bootstrap.service

# View auto-update timer
systemctl list-timers communitas-update.timer
```

### Monitoring

Check metrics endpoint for monitoring:

```bash
# Get metrics
curl http://YOUR_DROPLET_IP:9600/metrics

# Example metrics to monitor:
# - communitas_peer_count - Number of connected peers
# - communitas_message_count - Total messages processed
# - communitas_storage_size - Storage usage
# - communitas_uptime_seconds - Service uptime
```

### Backup and Recovery

The bootstrap node stores data in `/var/lib/communitas`. To backup:

```bash
# Create backup
tar -czf communitas-backup-$(date +%Y%m%d).tar.gz /var/lib/communitas

# Restore from backup
systemctl stop communitas-bootstrap
tar -xzf communitas-backup-YYYYMMDD.tar.gz -C /
chown -R communitas:communitas /var/lib/communitas
systemctl start communitas-bootstrap
```

## Troubleshooting

### Service Won't Start

```bash
# Check logs for errors
journalctl -u communitas-bootstrap -n 50

# Verify binary
/opt/communitas/communitas-headless --version

# Check permissions
ls -la /opt/communitas
ls -la /var/lib/communitas

# Verify user exists
id communitas
```

### Network Connectivity Issues

```bash
# Check firewall rules
ufw status verbose

# Test local connectivity
curl http://localhost:8080/health

# Check if port is listening
netstat -tlnp | grep 8080

# Test from external machine
curl http://YOUR_DROPLET_IP:8080/health
```

### Auto-Update Issues

```bash
# Check update timer status
systemctl status communitas-update.timer

# Check update service logs
journalctl -u communitas-update -n 50

# Manually trigger update
systemctl start communitas-update.service

# Check update script
cat /opt/communitas/update.sh
```

### High Resource Usage

```bash
# Check resource usage
top -p $(pgrep communitas-headless)

# Check storage usage
du -sh /var/lib/communitas

# Check file descriptor usage
lsof -p $(pgrep communitas-headless) | wc -l

# View resource limits
systemctl show communitas-bootstrap | grep Limit
```

## Security Considerations

### Hardening Checklist

- ✅ Non-root service user (communitas)
- ✅ Systemd security sandboxing (NoNewPrivileges, PrivateTmp, ProtectSystem)
- ✅ Firewall enabled and configured
- ✅ Resource limits enforced (65536 files, 4096 processes)
- ✅ Auto-updates enabled
- ⚠️ SSH key authentication (disable password auth)
- ⚠️ Fail2ban for SSH protection
- ⚠️ Regular security updates (`apt update && apt upgrade`)

### Additional Hardening (Optional)

```bash
# Disable password authentication
echo "PasswordAuthentication no" >> /etc/ssh/sshd_config
systemctl restart sshd

# Install fail2ban
apt install -y fail2ban
systemctl enable fail2ban
systemctl start fail2ban

# Enable automatic security updates
apt install -y unattended-upgrades
dpkg-reconfigure -plow unattended-upgrades
```

## Cost Estimation

### Droplet Costs (Monthly)

- **Minimum** ($6/month): 1GB RAM, 1 vCPU, 25GB SSD
  - Suitable for testing and small networks
  - Limited to ~100 concurrent connections

- **Recommended** ($12/month): 2GB RAM, 1 vCPU, 50GB SSD
  - Good for production bootstrap nodes
  - Supports ~500 concurrent connections

- **Production** ($18/month): 2GB RAM, 2 vCPU, 60GB SSD
  - Best for high-traffic bootstrap nodes
  - Supports ~1000+ concurrent connections

### Additional Costs

- **Bandwidth**: First 1TB included, then $0.01/GB
- **Backups**: +20% of droplet cost (optional)
- **Snapshots**: $0.05/GB/month (optional)

### Cost Optimization

1. Start with minimum size and scale up based on metrics
2. Use monitoring to track actual resource usage
3. Consider reserved instances for long-term commitments
4. Use load balancer only if running multiple bootstrap nodes

## Multi-Node Deployment

For production environments, deploy multiple bootstrap nodes:

1. **Different Regions:**
   - Deploy nodes in multiple geographic regions
   - Update `BOOTSTRAP_PEERS` with all node addresses

2. **Load Balancing:**
   - Use DigitalOcean Load Balancer for high availability
   - Configure health checks on port 8080

3. **Monitoring:**
   - Set up centralized logging (e.g., Papertrail, Loggly)
   - Use metrics aggregation (e.g., Prometheus, Grafana)

Example multi-node configuration:

```rust
pub const BOOTSTRAP_PEERS: &[&str] = &[
    "ocean-forest-moon-star",  // NYC node
    "valley-river-cloud-tree", // SFO node
    "island-wave-sand-coral",  // LON node
];
```

## References

- Provisioning Script: `scripts/provision-bootstrap.sh`
- Systemd Service: `/etc/systemd/system/communitas-bootstrap.service`
- Auto-Update Timer: `/etc/systemd/system/communitas-update.timer`
- Data Directory: `/var/lib/communitas`
- Binary Location: `/opt/communitas/communitas-headless`
- Bootstrap Info: `/opt/communitas/bootstrap-info.txt`

## Support

For issues or questions:
- GitHub Issues: https://github.com/dirvine/communitas/issues
- Documentation: https://communitas.life
- Community: [Add community links]
