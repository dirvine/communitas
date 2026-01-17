# Communitas Headless

Headless daemon for running Communitas as a system service.

## Overview

Communitas Headless runs the Communitas core without a UI. It is designed for:

- Server deployments
- Bot and automation nodes
- Bootstrap/seed nodes
- Background services
- Testnet environments

## Features

- **System Service Integration**: systemd (Linux) and launchd (macOS)
- **Metrics & Health**: `/metrics` and `/health` HTTP endpoints
- **Multi-Instance Support**: isolated configs via `--instance-id`
- **Auto-Update (Optional)**: GitHub Releases self-update
- **Configurable Storage & Network**: TOML-driven settings
- **FEC Storage Options**: Reed-Solomon parameters for resilience

## Installation

### From Source

```bash
cargo install --path communitas-headless
```

### From Binary

```bash
# Download latest release
wget https://github.com/saorsalabs/communitas/releases/latest/communitas-headless

# Make executable
chmod +x communitas-headless

# Move to system path
sudo mv communitas-headless /usr/local/bin/
```

## Configuration

By default, the daemon creates a per-instance config at:

- macOS/Linux: `~/.config/communitas/<instance-id>/config.toml`

You can override with `--config` or `COMMUNITAS_CONFIG_PATH`.

### Example config

```toml
# Communitas Headless Config

# Node identity (four-word address). If omitted, one is generated.
identity = "ocean-forest-moon-star"

# Bootstrap nodes (IP:port)
bootstrap_nodes = [
  "142.93.199.50:11000",
  "147.182.234.192:11000",
  "206.189.7.117:11000",
  "144.126.230.161:11000"
]

[storage]
base_dir = "/var/lib/communitas"
cache_size_mb = 1024
enable_fec = true
fec_k = 8
fec_m = 4

[network]
listen_addrs = ["0.0.0.0:0", "[::]:0"]
enable_ipv6 = true
enable_webrtc = false
quic_idle_timeout_ms = 30000
quic_max_streams = 100

[update]
channel = "stable"
check_interval_secs = 21600
auto_update = true
jitter_secs = 0
public_keys_base64 = []
require_checksum = true
```

## CLI Usage

```bash
communitas-headless \
  --config /etc/communitas/headless.toml \
  --storage /var/lib/communitas \
  --instance-id node-1 \
  --listen 0.0.0.0:0 \
  --bootstrap 142.93.199.50:11000 \
  --metrics --metrics-addr 127.0.0.1:9600
```

### Self-update

```bash
communitas-headless --self-update
```

## Metrics & Health

If `--metrics` is enabled, the daemon serves:

- `GET /health` (JSON health status)
- `GET /metrics` (Prometheus format)

Default listen address: `127.0.0.1:9600`.

## System Service Setup

### systemd (Linux)

Create `/etc/systemd/system/communitas.service`:

```ini
[Unit]
Description=Communitas Headless Daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=communitas
Group=communitas
ExecStart=/usr/local/bin/communitas-headless --config /etc/communitas/headless.toml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/communitas /var/log/communitas

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable communitas
sudo systemctl start communitas
sudo systemctl status communitas
```

### launchd (macOS)

Create `~/Library/LaunchAgents/com.saorsalabs.communitas.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.saorsalabs.communitas</string>

  <key>ProgramArguments</key>
  <array>
    <string>/usr/local/bin/communitas-headless</string>
    <string>--config</string>
    <string>/Users/you/.config/communitas/headless.toml</string>
  </array>

  <key>RunAtLoad</key>
  <true/>

  <key>KeepAlive</key>
  <true/>

  <key>StandardOutPath</key>
  <string>/Users/you/Library/Logs/communitas.log</string>

  <key>StandardErrorPath</key>
  <string>/Users/you/Library/Logs/communitas-error.log</string>
</dict>
</plist>
```

Enable and start:

```bash
launchctl load ~/Library/LaunchAgents/com.saorsalabs.communitas.plist
launchctl start com.saorsalabs.communitas
```
