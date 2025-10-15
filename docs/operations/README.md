# Operations Guide

Comprehensive guide for deploying and operating Communitas infrastructure.

## Table of Contents

- [Overview](#overview)
- [Deployment Options](#deployment-options)
- [Infrastructure Setup](#infrastructure-setup)
- [Configuration](#configuration)
- [Monitoring](#monitoring)
- [Backup and Recovery](#backup-and-recovery)
- [Security](#security)
- [Scaling](#scaling)
- [Maintenance](#maintenance)

---

## Overview

Communitas can be deployed in several configurations depending on your use case:

**Deployment Modes**:
- **Desktop Application**: End-user desktop apps (macOS, Windows, Linux)
- **Bootstrap Nodes**: Network bootstrap and discovery
- **Headless Nodes**: Background P2P nodes
- **Bridge Server**: HTTP/REST interface for testing
- **Container Deployment**: Docker/Kubernetes orchestration

---

## Deployment Options

### Desktop Application

**Target Audience**: End users

**Distribution**:
- **macOS**: DMG installer with code signing
- **Windows**: MSI installer with digital signature
- **Linux**: AppImage, DEB, RPM packages

**Build Commands**:
```bash
# Build for current platform
npm run tauri build

# Build with specific features
npm run tauri build -- --features "passkey,touchid"

# Output location
ls src-tauri/target/release/bundle/
```

**System Requirements**:
- **RAM**: 2GB minimum, 4GB recommended
- **Storage**: 500MB for app + data
- **Network**: Broadband internet
- **OS**: macOS 11+, Windows 10+, Ubuntu 20.04+

---

### Bootstrap Node

**Purpose**: Network bootstrap and peer discovery

**Deployment**:
```bash
# Build bootstrap node
cargo build --release -p bootstrap-node

# Run with configuration
./target/release/bootstrap-node --config bootstrap-config.toml
```

**Configuration** (bootstrap-config.toml):
```toml
[network]
listen_addresses = [
  "/ip4/0.0.0.0/udp/8080/quic-v1",
  "/ip6/::/udp/8080/quic-v1"
]

[bootstrap]
# Four-word identity for this bootstrap node
four_words = "ocean-forest-moon-star"

# External address for other nodes to connect
external_address = "198.51.100.42:8080"

[storage]
data_dir = "/var/lib/communitas/bootstrap"

[logging]
level = "info"
file = "/var/log/communitas/bootstrap.log"
```

**Systemd Service** (communitas-bootstrap.service):
```ini
[Unit]
Description=Communitas Bootstrap Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=communitas
Group=communitas
ExecStart=/usr/local/bin/bootstrap-node --config /etc/communitas/bootstrap-config.toml
Restart=on-failure
RestartSec=10s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/communitas /var/log/communitas

[Install]
WantedBy=multi-user.target
```

**Installation**:
```bash
# Copy binary
sudo cp target/release/bootstrap-node /usr/local/bin/

# Create user
sudo useradd -r -s /bin/false communitas

# Create directories
sudo mkdir -p /var/lib/communitas/bootstrap
sudo mkdir -p /var/log/communitas
sudo chown -R communitas:communitas /var/lib/communitas /var/log/communitas

# Install service
sudo cp communitas-bootstrap.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable communitas-bootstrap
sudo systemctl start communitas-bootstrap

# Check status
sudo systemctl status communitas-bootstrap
```

---

### Headless Node

**Purpose**: P2P node without UI

**Deployment**:
```bash
# Build headless node
cargo build --release -p communitas-headless

# Run with configuration
./target/release/communitas-headless --config headless-config.toml
```

**Configuration** (headless-config.toml):
```toml
[network]
listen_addresses = [
  "/ip4/0.0.0.0/udp/8080/quic-v1"
]

bootstrap_nodes = [
  "ocean-forest-moon-star",  # 198.51.100.42:8080
  "valley-river-cloud-wind"  # 203.0.113.7:8080
]

[storage]
data_dir = "/var/lib/communitas/node"

[identity]
four_words = "mountain-lake-tree-bird"

[logging]
level = "info"
file = "/var/log/communitas/node.log"
```

**Docker Deployment**:
```dockerfile
FROM rust:1.85-slim as builder

WORKDIR /build
COPY . .

RUN cargo build --release -p communitas-headless

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/communitas-headless /usr/local/bin/

RUN useradd -r -s /bin/false communitas && \
    mkdir -p /var/lib/communitas /var/log/communitas && \
    chown -R communitas:communitas /var/lib/communitas /var/log/communitas

USER communitas
VOLUME ["/var/lib/communitas", "/var/log/communitas"]

EXPOSE 8080/udp

ENTRYPOINT ["/usr/local/bin/communitas-headless"]
CMD ["--config", "/etc/communitas/headless-config.toml"]
```

**Docker Compose**:
```yaml
version: '3.8'

services:
  communitas-node:
    build: .
    container_name: communitas-node
    restart: unless-stopped
    ports:
      - "8080:8080/udp"
    volumes:
      - ./headless-config.toml:/etc/communitas/headless-config.toml:ro
      - node-data:/var/lib/communitas
      - node-logs:/var/log/communitas
    environment:
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "nc", "-zu", "localhost", "8080"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  node-data:
  node-logs:
```

---

### Kubernetes Deployment

**Purpose**: Scalable P2P network with orchestration

**Deployment**:
```bash
# Build and push image
docker build -t communitas/node:latest .
docker push communitas/node:latest

# Deploy to Kubernetes
kubectl apply -f k8s/
```

**StatefulSet** (k8s/statefulset.yaml):
```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: communitas-nodes
  namespace: communitas
spec:
  serviceName: communitas-nodes
  replicas: 3
  selector:
    matchLabels:
      app: communitas-node
  template:
    metadata:
      labels:
        app: communitas-node
    spec:
      containers:
      - name: node
        image: communitas/node:latest
        ports:
        - containerPort: 8080
          protocol: UDP
          name: quic
        volumeMounts:
        - name: data
          mountPath: /var/lib/communitas
        - name: config
          mountPath: /etc/communitas
          readOnly: true
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          exec:
            command:
            - /bin/sh
            - -c
            - nc -zu localhost 8080
          initialDelaySeconds: 30
          periodSeconds: 30
      volumes:
      - name: config
        configMap:
          name: communitas-config
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 10Gi
```

**Service** (k8s/service.yaml):
```yaml
apiVersion: v1
kind: Service
metadata:
  name: communitas-nodes
  namespace: communitas
spec:
  type: LoadBalancer
  ports:
  - port: 8080
    protocol: UDP
    targetPort: 8080
    name: quic
  selector:
    app: communitas-node
```

**ConfigMap** (k8s/configmap.yaml):
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: communitas-config
  namespace: communitas
data:
  headless-config.toml: |
    [network]
    listen_addresses = ["/ip4/0.0.0.0/udp/8080/quic-v1"]
    bootstrap_nodes = ["ocean-forest-moon-star"]

    [storage]
    data_dir = "/var/lib/communitas/node"

    [logging]
    level = "info"
```

---

## Infrastructure Setup

### Cloud Provider Setup

#### AWS

**EC2 Instance**:
```bash
# Launch EC2 instance (Ubuntu 22.04)
aws ec2 run-instances \
  --image-id ami-xxxxxxxxx \
  --instance-type t3.medium \
  --key-name your-key \
  --security-group-ids sg-xxxxxxxxx \
  --subnet-id subnet-xxxxxxxxx \
  --user-data file://bootstrap.sh
```

**Security Group**:
```bash
# Allow QUIC traffic
aws ec2 authorize-security-group-ingress \
  --group-id sg-xxxxxxxxx \
  --protocol udp \
  --port 8080 \
  --cidr 0.0.0.0/0

# Allow SSH (for management)
aws ec2 authorize-security-group-ingress \
  --group-id sg-xxxxxxxxx \
  --protocol tcp \
  --port 22 \
  --cidr your-ip/32
```

**User Data Script** (bootstrap.sh):
```bash
#!/bin/bash
set -e

# Update system
apt-get update
apt-get upgrade -y

# Install dependencies
apt-get install -y curl build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/saorsalabs/communitas.git
cd communitas
cargo build --release -p bootstrap-node

# Install as service
cp target/release/bootstrap-node /usr/local/bin/
# ... (systemd service setup as above)
```

---

#### Google Cloud Platform

**Compute Engine**:
```bash
# Create instance
gcloud compute instances create communitas-bootstrap \
  --machine-type=n1-standard-1 \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --boot-disk-size=20GB \
  --metadata-from-file startup-script=bootstrap.sh
```

**Firewall Rule**:
```bash
gcloud compute firewall-rules create allow-quic \
  --allow=udp:8080 \
  --source-ranges=0.0.0.0/0 \
  --target-tags=communitas-node
```

---

#### DigitalOcean

**Droplet**:
```bash
# Create droplet via CLI
doctl compute droplet create communitas-node \
  --size s-2vcpu-2gb \
  --image ubuntu-22-04-x64 \
  --region nyc1 \
  --ssh-keys your-key-id \
  --user-data-file bootstrap.sh
```

---

### Network Configuration

**Firewall Rules**:
```bash
# UFW (Ubuntu)
sudo ufw allow 8080/udp comment 'Communitas QUIC'
sudo ufw enable

# firewalld (CentOS/RHEL)
sudo firewall-cmd --permanent --add-port=8080/udp
sudo firewall-cmd --reload

# iptables (manual)
sudo iptables -A INPUT -p udp --dport 8080 -j ACCEPT
sudo iptables-save > /etc/iptables/rules.v4
```

**NAT Traversal**:
- Configure router UPnP if behind NAT
- Or use port forwarding: External 8080 → Internal 8080
- For cloud deployments, use public IP

---

## Configuration

### Environment Variables

```bash
# Logging
export RUST_LOG=info                    # Log level
export RUST_LOG_FILE=/var/log/node.log # Log file

# Network
export COMMUNITAS_LISTEN=0.0.0.0:8080   # Listen address
export COMMUNITAS_BOOTSTRAP=ocean-forest-moon-star  # Bootstrap nodes

# Storage
export COMMUNITAS_DATA_DIR=/var/lib/communitas  # Data directory

# Performance
export RAYON_NUM_THREADS=4              # Parallel threads
```

### Configuration Files

**Priority Order** (highest to lowest):
1. Command-line arguments
2. Environment variables
3. Config file (`--config`)
4. Default values

**Example Override**:
```bash
# Use custom config with env override
RUST_LOG=debug ./bootstrap-node \
  --config /etc/communitas/bootstrap-config.toml \
  --listen-address 0.0.0.0:9090
```

---

## Monitoring

See [Monitoring Guide](monitoring.md) for detailed monitoring setup.

**Quick Overview**:

**Metrics Exposed**:
- Network peer count
- Message throughput
- Storage usage
- CPU and memory usage
- Network latency

**Monitoring Stack**:
- **Prometheus**: Metrics collection
- **Grafana**: Visualization
- **Loki**: Log aggregation
- **Alert Manager**: Alerting

**Health Checks**:
```bash
# Check if node is running
systemctl status communitas-bootstrap

# Check network connectivity
nc -zu localhost 8080

# Check logs
journalctl -u communitas-bootstrap -f
```

---

## Backup and Recovery

### Data Backup

**What to Back Up**:
- Vault data: `/var/lib/communitas/vaults/`
- Identity keys: `/var/lib/communitas/identity/`
- Configuration: `/etc/communitas/`

**Backup Script**:
```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backups/communitas/$(date +%Y%m%d-%H%M%S)"
DATA_DIR="/var/lib/communitas"
CONFIG_DIR="/etc/communitas"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup data
tar -czf "$BACKUP_DIR/data.tar.gz" "$DATA_DIR"

# Backup config
tar -czf "$BACKUP_DIR/config.tar.gz" "$CONFIG_DIR"

# Verify backup
tar -tzf "$BACKUP_DIR/data.tar.gz" >/dev/null
tar -tzf "$BACKUP_DIR/config.tar.gz" >/dev/null

echo "Backup completed: $BACKUP_DIR"

# Cleanup old backups (keep last 7 days)
find /backups/communitas/ -type d -mtime +7 -exec rm -rf {} \;
```

**Automated Backups** (cron):
```bash
# Edit crontab
crontab -e

# Add daily backup at 2 AM
0 2 * * * /usr/local/bin/backup.sh >> /var/log/communitas/backup.log 2>&1
```

### Recovery

**Restore from Backup**:
```bash
#!/bin/bash
# restore.sh

BACKUP_FILE="$1"

if [ -z "$BACKUP_FILE" ]; then
  echo "Usage: $0 <backup-file>"
  exit 1
fi

# Stop service
systemctl stop communitas-bootstrap

# Restore data
tar -xzf "$BACKUP_FILE" -C /

# Start service
systemctl start communitas-bootstrap

echo "Recovery completed"
```

---

## Security

### Access Control

**SSH Hardening**:
```bash
# /etc/ssh/sshd_config
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
Port 2222  # Non-standard port
```

**User Permissions**:
```bash
# Create dedicated user
sudo useradd -r -s /bin/false communitas

# Set permissions
sudo chown -R communitas:communitas /var/lib/communitas
sudo chmod 700 /var/lib/communitas/vaults
sudo chmod 600 /var/lib/communitas/identity/*
```

### TLS/SSL

**For Bridge Server**:
```toml
# bridge-config.toml
[tls]
enabled = true
cert_file = "/etc/letsencrypt/live/communitas.example.com/fullchain.pem"
key_file = "/etc/letsencrypt/live/communitas.example.com/privkey.pem"
```

**Let's Encrypt Setup**:
```bash
# Install certbot
sudo apt install certbot

# Get certificate
sudo certbot certonly --standalone \
  -d communitas.example.com \
  --email admin@example.com \
  --agree-tos

# Auto-renewal
sudo crontab -e
# Add: 0 3 * * * certbot renew --quiet --post-hook "systemctl reload communitas-bridge"
```

---

## Scaling

### Horizontal Scaling

**Load Distribution**:
```
                                    [Load Balancer]
                                          |
              ┌───────────────────────────┼───────────────────────────┐
              │                           │                           │
         [Bootstrap 1]              [Bootstrap 2]              [Bootstrap 3]
              │                           │                           │
      ┌───────┴───────┐           ┌───────┴───────┐           ┌───────┴───────┐
   [Node]  [Node]  [Node]      [Node]  [Node]  [Node]      [Node]  [Node]  [Node]
```

**Kubernetes Horizontal Pod Autoscaler**:
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: communitas-nodes
  namespace: communitas
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: StatefulSet
    name: communitas-nodes
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

## Maintenance

### Updates

**Update Procedure**:
```bash
# 1. Backup current state
./backup.sh

# 2. Pull latest changes
git pull origin main

# 3. Build new version
cargo build --release -p bootstrap-node

# 4. Stop service
sudo systemctl stop communitas-bootstrap

# 5. Replace binary
sudo cp target/release/bootstrap-node /usr/local/bin/

# 6. Start service
sudo systemctl start communitas-bootstrap

# 7. Verify
sudo systemctl status communitas-bootstrap
journalctl -u communitas-bootstrap -f
```

### Health Checks

**Automated Health Check Script**:
```bash
#!/bin/bash
# health-check.sh

SERVICE="communitas-bootstrap"
LOG_FILE="/var/log/communitas/health-check.log"

check_service() {
  if systemctl is-active --quiet "$SERVICE"; then
    echo "$(date): Service is running" >> "$LOG_FILE"
    return 0
  else
    echo "$(date): Service is NOT running" >> "$LOG_FILE"
    systemctl start "$SERVICE"
    return 1
  fi
}

check_network() {
  if nc -zu localhost 8080; then
    echo "$(date): Network port is open" >> "$LOG_FILE"
    return 0
  else
    echo "$(date): Network port is closed" >> "$LOG_FILE"
    return 1
  fi
}

# Run checks
check_service && check_network

# Exit with failure if either check failed
exit $?
```

**Cron Job**:
```bash
# Run health check every 5 minutes
*/5 * * * * /usr/local/bin/health-check.sh
```

---

## See Also

- [Monitoring Guide](monitoring.md) - Detailed monitoring and observability
- [Architecture](../architecture/README.md) - System architecture
- [Development Guide](../development/README.md) - Development setup and workflows

---

**Operations Guide**: Deploy and operate Communitas infrastructure. 🚀⚙️
