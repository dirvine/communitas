# Communitas Headless

Headless daemon for running Communitas as a system service.

## Overview

Communitas Headless is a daemon application that runs Communitas core without a user interface. It's designed for:

- Server deployments
- Bot applications
- Automated systems
- Background services
- Integration with other applications

## Features

- **System Service Integration**: Supports systemd (Linux) and launchd (macOS)
- **JSON-RPC API**: Programmatic access to all Communitas features
- **Webhook Support**: Push notifications for events
- **Automated Backups**: Scheduled backups of state and data
- **Resource Efficient**: Minimal CPU and memory footprint
- **Multi-Tenant**: Support multiple identities/instances

## Installation

### From Binary

```bash
# Download latest release
wget https://github.com/saorsalabs/communitas/releases/latest/communitas-headless

# Make executable
chmod +x communitas-headless

# Move to system path
sudo mv communitas-headless /usr/local/bin/
```

### From Source

```bash
cargo install --path communitas-headless
```

### Via Package Manager

```bash
# Homebrew (macOS)
brew install saorsalabs/tap/communitas-headless

# APT (Debian/Ubuntu) - coming soon
apt install communitas-headless
```

## Configuration

Create `/etc/communitas/headless.toml`:

```toml
[daemon]
api_port = 9090
api_host = "127.0.0.1"
data_dir = "/var/lib/communitas"
log_level = "info"

[identity]
four_words = "ocean-forest-moon-star"
display_name = "Headless Bot"
device_name = "Server-01"
auto_connect = true

[network]
bootstrap_nodes = [
  "bootstrap.communitas.network:8080",
  "bootstrap-eu.communitas.network:8080"
]
enable_mdns = false

[storage]
backup_enabled = true
backup_interval = 3600  # seconds
backup_path = "/var/backups/communitas"

[webhooks]
enabled = true
url = "https://your-app.com/webhooks/communitas"
secret = "your-webhook-secret"
events = ["message", "presence", "channel"]

[logging]
format = "json"
output = "file"
file_path = "/var/log/communitas/headless.log"
rotate_size = "100MB"
rotate_count = 5
```

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

## JSON-RPC API

### Connection

```bash
# Via curl
curl -X POST http://localhost:9090/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "status", "id": 1}'
```

```python
# Via Python
import requests

def call_rpc(method, params=None):
    response = requests.post('http://localhost:9090/rpc', json={
        'jsonrpc': '2.0',
        'method': method,
        'params': params or {},
        'id': 1
    })
    return response.json()['result']

# Get status
status = call_rpc('status')
print(status)
```

### Available Methods

#### Core Methods

- `status` - Get daemon status
- `info` - Get identity and network information
- `shutdown` - Gracefully shutdown daemon

#### Message Methods

- `message.send` - Send a message
  ```json
  {
    "channel_id": "ch_123",
    "content": "Hello!",
    "recipients": ["ocean-forest-moon-star"]
  }
  ```

- `message.list` - List messages
  ```json
  {
    "channel_id": "ch_123",
    "limit": 50,
    "offset": 0
  }
  ```

#### Channel Methods

- `channel.create` - Create a channel
- `channel.list` - List channels
- `channel.join` - Join a channel
- `channel.leave` - Leave a channel

#### Identity Methods

- `identity.get` - Get current identity
- `identity.list_contacts` - List contacts
- `identity.add_contact` - Add a contact

## Webhooks

When enabled, the daemon will send HTTP POST requests to the configured webhook URL for events:

### Webhook Payload

```json
{
  "event": "message:new",
  "timestamp": "2025-10-15T12:34:56Z",
  "signature": "sha256=...",
  "data": {
    "id": "msg_123",
    "channel_id": "ch_456",
    "content": "Hello",
    "sender": "ocean-forest-moon-star",
    "timestamp": "2025-10-15T12:34:56Z"
  }
}
```

### Signature Verification

Webhooks are signed with HMAC-SHA256:

```python
import hmac
import hashlib

def verify_webhook(payload, signature, secret):
    expected = hmac.new(
        secret.encode(),
        payload.encode(),
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(f"sha256={expected}", signature)
```

## Bot Example

```python
#!/usr/bin/env python3
"""
Simple echo bot using Communitas Headless
"""
import requests
import time

RPC_URL = 'http://localhost:9090/rpc'

def rpc_call(method, params=None):
    response = requests.post(RPC_URL, json={
        'jsonrpc': '2.0',
        'method': method,
        'params': params or {},
        'id': 1
    })
    return response.json().get('result')

def echo_bot():
    last_message_id = None

    while True:
        # Get recent messages
        messages = rpc_call('message.list', {
            'limit': 10,
            'after': last_message_id
        })

        for msg in messages:
            if msg['id'] == last_message_id:
                continue

            # Echo the message
            if not msg['content'].startswith('Bot:'):
                rpc_call('message.send', {
                    'channel_id': msg['channel_id'],
                    'content': f"Bot: {msg['content']}"
                })

            last_message_id = msg['id']

        time.sleep(1)

if __name__ == '__main__':
    echo_bot()
```

## Monitoring

### Metrics Endpoint

Prometheus metrics available at `http://localhost:9090/metrics`

Key metrics:
- `communitas_messages_sent_total`
- `communitas_messages_received_total`
- `communitas_peers_connected`
- `communitas_rpc_requests_total`
- `communitas_uptime_seconds`

### Health Check

```bash
curl http://localhost:9090/health
```

Returns:
```json
{
  "status": "healthy",
  "uptime": 3600,
  "identity": "ocean-forest-moon-star",
  "peers_connected": 42,
  "version": "0.1.17"
}
```

## Development

### Building

```bash
cargo build -p communitas-headless --release
```

### Testing

```bash
cargo test -p communitas-headless
```

### Running Locally

```bash
cargo run -p communitas-headless -- --config test-config.toml
```

## Troubleshooting

### Service Won't Start

```bash
# Check logs
journalctl -u communitas -f  # systemd
tail -f /var/log/communitas/headless.log

# Verify configuration
communitas-headless --config /etc/communitas/headless.toml --validate
```

### API Connection Refused

- Verify daemon is running: `systemctl status communitas`
- Check port is not blocked: `netstat -tlnp | grep 9090`
- Review firewall rules

### High Memory Usage

- Reduce backup frequency
- Limit message history retention
- Adjust peer connections in configuration

## Security Best Practices

1. **Run as Dedicated User**: Create a `communitas` user
2. **Limit Network Access**: Bind API to localhost only
3. **Secure Configuration**: Protect config files (chmod 600)
4. **Regular Updates**: Keep daemon updated
5. **Monitor Logs**: Watch for suspicious activity
6. **Backup Encryption**: Encrypt backup data

## Performance

- Memory: ~50MB typical, ~200MB peak
- CPU: <5% on modern hardware
- Disk I/O: Minimal, mostly on message receipt
- Network: ~1KB/s idle, scales with activity

## Contributing

See [../../docs/development/contributing.md](../../docs/development/contributing.md)

## License

Dual-licensed under AGPL-3.0-or-later and commercial license.

## See Also

- [Communitas Core](../communitas-core/README.md)
- [Operations Guide](../../docs/operations/README.md)
- [API Reference](../../docs/api/README.md)
