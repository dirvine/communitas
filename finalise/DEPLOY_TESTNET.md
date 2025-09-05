# Communitas — DigitalOcean Testnet Deployment

Targets: 6 droplets across AMS3, LON1, FRA1, NYC3, SFO3, SGP1. Ubuntu 24.04. **IPv4-first** networking; IPv6 optional fallback.

## 1) Prereqs
- doctl authenticated or Terraform with DO provider.
- SSH key uploaded to DO.
- Release URLs for `communitas-node` and `communitas-autoupdater`.
- Empty `bootstrap.toml` template ready.

## 2) Regions and size
- Regions: `ams3 lon1 fra1 nyc3 sfo3 sgp1`
- Size: `s-1vcpu-2gb`, disk 50 GB, IPv4 public enabled, IPv6 optional.

## 3) Firewall
Allow UDP 443, TCP 443, TCP 22, ICMP. Restrict SSH to maintainer IPs.

## 4) Cloud-init user-data
Replace `{{RELEASE_URL}}` and `{{AUTO_URL}}`:
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
  - curl -L {{RELEASE_URL}} -o /opt/communitas/bin/communitas-node
  - curl -L {{AUTO_URL}} -o /opt/communitas/bin/communitas-autoupdater
  - chmod +x /opt/communitas/bin/communitas-node /opt/communitas/bin/communitas-autoupdater
  - chown -R communitas:communitas /opt/communitas /var/lib/communitas
  - install -d -o communitas -g communitas /etc/communitas
  - printf '%s\n' "[update]\nchannel=stable\n" > /etc/communitas/update.toml
  - systemctl daemon-reload
  - systemctl enable communitas.service
  - systemctl start communitas.service
  - systemctl enable communitas-updater.service
  - systemctl start communitas-updater.service
```

### systemd units
`/etc/systemd/system/communitas.service`:
```ini
[Unit]
Description=Communitas Node
After=network-online.target
Wants=network-online.target

[Service]
User=communitas
ExecStart=/opt/communitas/bin/communitas-node --config /etc/communitas/config.toml
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

## 5) Config files
`/etc/communitas/config.toml`:
```toml
[node]
storage_path = "/var/lib/communitas"
listen_quic = ["0.0.0.0:443"]          # IPv4-first
listen_webrtc = ["0.0.0.0:443"]
# Optionally add IPv6 listeners:
# listen_quic = ["0.0.0.0:443", "[::]:443"]

[bootstrap]
# Filled after four-word derivation (see next section)
seeds = []
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
- Join: from a laptop, confirm peer table includes at least 3 seeds.
- Latency: peer RPC p95 < 600 ms cross-region.
- Messaging: send 1000 msgs in a test channel; verify order and delivery.
- Files: upload 50 MB; verify seal+FEC and retrieval from ≥ 3 peers.

## 8) Observability
- If `metrics` enabled, SSH tunnel to 127.0.0.1:9600.
- Logs: `journalctl -u communitas -f`.

## 9) Rolling upgrades with jitter
- Publish signed release.
- Seeds detect V, draw `delay ∈ [0,6h]`, upgrade at delay.
- Other nodes follow as they see V. No coordinator required.

## 10) Rollback
- Keep N-1 artifact. Updater supports `--pin VERSION`.
- Pin seeds to N-1, wait for convergence, then unpin after fix.

## 11) Costs (rough)
- 6× `s-1vcpu-2gb` ≈ low three-figures GBP/month including bandwidth.
