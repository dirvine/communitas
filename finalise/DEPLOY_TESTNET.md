# Communitas — DigitalOcean Testnet Deployment

**Updated for saorsa-core 0.3.18** - removes jitter requirement for auto-updates, improves network performance.

Targets: 6 droplets across AMS3, LON1, FRA1, NYC3, SFO3, SGP1. Ubuntu 24.04. **IPv4-first** networking; IPv6 optional fallback.

## 1) Prereqs
- doctl authenticated or Terraform with DO provider.
- SSH key uploaded to DO.
- Release URLs for `communitas-headless`, `communitas-autoupdater`, and `bootstrap` (all built with saorsa-core 0.3.18+).
- Empty `bootstrap.toml` template ready.
- Bootstrap node binary tested and working locally.

## 2) Regions and size
- Regions: `ams3 lon1 fra1 nyc3 sfo3 sgp1`
- Size: `s-1vcpu-2gb`, disk 50 GB, IPv4 public enabled, IPv6 optional.
- Bootstrap node: dedicated droplet in `lon1` region for initial peer discovery.

## 3) Firewall
Allow UDP 443, TCP 443, TCP 22, ICMP. Restrict SSH to maintainer IPs.

## 4) Cloud-init user-data
Replace `{{RELEASE_URL}}`, `{{AUTO_URL}}`, and `{{BOOTSTRAP_URL}}`:
```bash
#cloud-config
package_update: true
package_upgrade: true
runcmd:
  - sysctl -w net.core.rmem_max=2500000
  - ufw allow 443/udp
  - ufw allow 443/tcp
  - ufw --force enable
  - useradd -m -s /bin/bash communitas || true
  - mkdir -p /opt/communitas/bin /var/lib/communitas
  - INSTANCE_ID={{INSTANCE_ID}}
   - curl -L {{RELEASE_URL}} -o /opt/communitas/bin/communitas-headless
   - curl -L {{AUTO_URL}} -o /opt/communitas/bin/communitas-autoupdater
   - curl -L {{BOOTSTRAP_URL}} -o /opt/communitas/bin/bootstrap
   - chmod +x /opt/communitas/bin/communitas-headless /opt/communitas/bin/communitas-autoupdater /opt/communitas/bin/bootstrap
  - chown -R communitas:communitas /opt/communitas /var/lib/communitas
  - install -d -o communitas -g communitas /etc/communitas/${INSTANCE_ID}
  - install -d -o communitas -g communitas /var/lib/communitas/${INSTANCE_ID}
  - printf '%s\n' "[update]\nchannel=stable\n" > /etc/communitas/update.toml
   - systemctl daemon-reload
   - systemctl enable communitas.service
   - systemctl start communitas.service
   - systemctl enable communitas-updater.service
   - systemctl start communitas-updater.service
   # For bootstrap nodes, also enable bootstrap service
   - systemctl enable communitas-bootstrap.service || true
   - systemctl start communitas-bootstrap.service || true
```

Set `{{INSTANCE_ID}}` to a stable identifier (for example `seed-node-1`). Each systemd unit below reuses the same value via `Environment=` variables to keep configuration and storage segregated per headless node. The binary will auto-create a default `config.toml` under the specified `--config` path on first launch if none exists.

### systemd units
`/etc/systemd/system/communitas.service` (update the `Environment=` lines to match your instance):
```ini
[Unit]
Description=Communitas Node
After=network-online.target
Wants=network-online.target

[Service]
User=communitas
Environment=COMMUNITAS_INSTANCE=seed-node-1
Environment=COMMUNITAS_CONFIG=/etc/communitas/seed-node-1/config.toml
Environment=COMMUNITAS_STORAGE=/var/lib/communitas/seed-node-1
ExecStartPre=/usr/bin/install -d -o communitas -g communitas ${COMMUNITAS_STORAGE}
ExecStart=/opt/communitas/bin/communitas-headless \\
    --instance-id=${COMMUNITAS_INSTANCE} \\
    --config=${COMMUNITAS_CONFIG} \\
    --storage=${COMMUNITAS_STORAGE}
Restart=always
RestartSec=5
LimitNOFILE=1048576

[Install]
WantedBy=multi-user.target
```

`/etc/systemd/system/communitas-updater.service`:
```ini
[Unit]
Description=Communitas Headless Auto-Updater

[Service]
User=communitas
ExecStart=/opt/communitas/bin/communitas-autoupdater --config /etc/communitas/update.toml
Restart=always
RestartSec=30

[Install]
WantedBy=multi-user.target
```

`/etc/systemd/system/communitas-seed.service` (for seed/introducer nodes – set a distinct instance id/storage root):
```ini
[Unit]
Description=Communitas Seed Node
After=network-online.target
Wants=network-online.target

[Service]
User=communitas
Environment=COMMUNITAS_INSTANCE=seed-node-1
Environment=COMMUNITAS_CONFIG=/etc/communitas/seed-node-1/config.toml
Environment=COMMUNITAS_STORAGE=/var/lib/communitas/seed-node-1
ExecStartPre=/usr/bin/install -d -o communitas -g communitas ${COMMUNITAS_STORAGE}
ExecStart=/opt/communitas/bin/communitas-headless \\
    --instance-id=${COMMUNITAS_INSTANCE} \\
    --config=${COMMUNITAS_CONFIG} \\
    --storage=${COMMUNITAS_STORAGE}
Restart=always
RestartSec=5
LimitNOFILE=1048576

[Install]
WantedBy=multi-user.target
```

## 5) Config files
`/etc/communitas/seed-node-1/config.toml` (auto-created on first start; adjust values as needed):
```toml
# Optional: pre-assign the node identity (otherwise generated on first boot)
identity = null

bootstrap_nodes = [
  # four-word-address:port entries go here
]

[storage]
base_dir = "/var/lib/communitas/seed-node-1"
cache_size_mb = 1024
enable_fec = true
fec_k = 8
fec_m = 4

[network]
listen_addrs = ["0.0.0.0:443"]
enable_ipv6 = true
enable_webrtc = false
quic_idle_timeout_ms = 30000
quic_max_streams = 100

[update]
channel = "stable"
check_interval_secs = 21600
auto_update = true
jitter_secs = 0
```

## 6) Derive four-word endpoints (post-provision)
Four-word addresses depend on **actual IPv4 and port**. Do this after droplets are up.

### Option A: four-word-networking CLI (if available)
```bash
four-word addr encode --ipv4 $(curl -s ifconfig.me) --port 443
```

### Option B: tiny Rust helper (add to your repo under tools/fwaddr)
`tools/fwaddr/Cargo.toml`:
```toml
[package]
name = "fwaddr"
version = "0.1.0"
edition = "2021"

[dependencies]
four-word-networking = "0.1"
```

`tools/fwaddr/src/main.rs`:
```rust
use std::env;
fn main() {
    let ip = env::args().nth(1).expect("ipv4");
    let port: u16 = env::args().nth(2).expect("port").parse().unwrap();
    let words = four_word_networking::encode_ipv4_port(&ip, port).expect("encode");
    println!("{}", words.join("-"));
}
```

Build and run:
```bash
cd tools/fwaddr && cargo run --release -- 203.0.113.10 443
# prints: e.g. "sparrow-candle-forest-ember"
```

Collect each droplet’s public IPv4 and compute its words.

### Update `bootstrap.toml`
Create and commit a client-distributed `bootstrap.toml`:
```toml
seeds = [
  "sparrow-candle-forest-ember:443",
  "....:443",
  "....:443",
  "....:443",
  "....:443",
  "....:443",
]
```

## 7) Validation
- Bootstrap: verify bootstrap nodes are accepting connections on port 9001.
- Join: from a laptop, confirm peer table includes at least 3 seeds.
- Latency: peer RPC p95 < 600 ms cross-region.
- Messaging: send 1000 msgs in a test channel; verify order and delivery.
- Files: upload 50 MB; verify seal+FEC and retrieval from ≥ 3 peers.
- Auto-update: verify nodes upgrade automatically without jitter delays.

## 8) Observability
- If `metrics` enabled, SSH tunnel to 127.0.0.1:9600.
- Logs: `journalctl -u communitas -f`.

## 9) Rolling upgrades (saorsa-core 0.3.18+)
- Publish signed release.
- Nodes detect new version and upgrade immediately (no jitter required with saorsa-core 0.3.18+).
- Rolling deployment across regions for gradual rollout.
- Other nodes follow as they see new version. No coordinator required.

## 10) Rollback
- Keep N-1 artifact. Updater supports `--pin VERSION`.
- Pin seeds to N-1, wait for convergence, then unpin after fix.

## 11) Costs (rough)
- 6× `s-1vcpu-2gb` ≈ £50-80 GBP/month including bandwidth.
- Additional bootstrap node: ~£15 GBP/month.
- Total estimated: £65-95 GBP/month for full testnet.
