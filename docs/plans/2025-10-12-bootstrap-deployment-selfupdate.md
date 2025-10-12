# Bootstrap Node Deployment & Self-Update System

**Date:** 2025-10-12
**Status:** 📋 Planning
**Priority:** 🔴 Critical

## Executive Summary

Complete plan for deploying `communitas-headless` as a production bootstrap node on DigitalOcean and implementing self-update functionality for all Communitas binaries (`communitas-desktop`, `communitas-headless`, `communitas-tui`).

## Objectives

1. ✅ **GitHub Releases** - Automated multi-platform binary releases
2. ✅ **DigitalOcean Deployment** - Bootstrap node on DO droplet via MCP
3. ✅ **Bootstrap Endpoint** - Hardcoded endpoint for app testing
4. ✅ **Self-Update System** - Auto-update from GitHub releases for all binaries

## Current State Analysis

### Existing Infrastructure ✅
- ✅ Release workflow exists: `.github/workflows/release-headless.yml`
- ✅ Builds Linux, macOS, Windows binaries
- ✅ `self_update` crate already in dependencies (communitas-headless/Cargo.toml:58)
- ✅ DigitalOcean MCP available for droplet management

### Missing Components ❌
- ❌ Self-update implementation in headless binary
- ❌ Self-update for communitas-desktop (Tauri app)
- ❌ Self-update for communitas-tui
- ❌ DigitalOcean droplet deployment automation
- ❌ Bootstrap endpoint configuration in apps
- ❌ Systemd service for headless node
- ❌ Monitoring and health checks

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        GitHub Releases                          │
│  - Linux x86_64 binaries (headless, TUI)                       │
│  - macOS Universal binaries (headless, TUI, desktop)           │
│  - Windows binaries (headless, desktop)                        │
└─────────────────┬───────────────────────────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
    ▼             ▼             ▼
┌─────────┐  ┌─────────┐  ┌──────────┐
│ Desktop │  │   TUI   │  │ Headless │
│  (Mac)  │  │  (Mac)  │  │  (DO)    │
└────┬────┘  └────┬────┘  └────┬─────┘
     │            │            │
     │            │            │
     └────────────┼────────────┘
                  │
                  ▼
         ┌────────────────┐
         │  Bootstrap Node│
         │  (DO Droplet)  │
         │  138.x.x.x     │
         └────────────────┘
```

## Implementation Plan

### Phase 1: GitHub Releases Enhancement ✅ (Mostly Complete)

**Status:** Existing workflow needs enhancement for TUI binaries

#### Task 1.1: Add communitas-tui to Release Workflow

**File:** `.github/workflows/release-headless.yml`

**Changes Needed:**
```yaml
# Current: Only builds communitas-headless
# New: Build both headless and TUI

- name: Build binaries (headless and TUI)
  run: |
    COMMUNITAS_SKIP_TAURI_BUILD=1 cargo build --release \
      --no-default-features \
      --bin communitas-headless \
      --bin communitas-tui

- name: Create archive (Linux target triple)
  run: |
    cd target/release
    tar -czf communitas-headless-x86_64-unknown-linux-gnu.tar.gz \
      communitas-headless
    tar -czf communitas-tui-x86_64-unknown-linux-gnu.tar.gz \
      communitas-tui
```

**Acceptance Criteria:**
- ✅ Both headless and TUI binaries built for Linux, macOS, Windows
- ✅ Separate tarballs for each binary
- ✅ Release notes include both binaries

**Estimated Effort:** 1 hour

---

### Phase 2: Self-Update Implementation

#### Task 2.1: Implement Self-Update for communitas-headless

**File:** `communitas-headless/src/self_update.rs` (NEW)

**Dependencies:** Already included (`self_update = "0.41"`)

**Implementation:**

```rust
// communitas-headless/src/self_update.rs

use anyhow::Result;
use self_update::backends::github::Update;
use tracing::{info, error};

const REPO_OWNER: &str = "dirvine";
const REPO_NAME: &str = "communitas";
const BIN_NAME: &str = "communitas-headless";

/// Check for updates and install if available
pub async fn check_and_update() -> Result<bool> {
    info!("Checking for updates from GitHub releases...");

    let status = Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;

    if status.updated() {
        info!("✅ Updated to version: {}", status.version());
        Ok(true)
    } else {
        info!("Already on latest version: {}", status.version());
        Ok(false)
    }
}

/// Force update to specific version
pub async fn update_to_version(version: &str) -> Result<()> {
    info!("Forcing update to version: {}", version);

    Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .show_download_progress(true)
        .target_version_tag(version)
        .build()?
        .update()?;

    info!("✅ Updated to version: {}", version);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_update() {
        // Test checks for updates without installing
        // Mock GitHub API responses
    }
}
```

**CLI Integration:**

```rust
// communitas-headless/src/main.rs

mod self_update;

#[derive(Parser)]
#[command(name = "communitas-headless")]
#[command(about = "Communitas headless node with auto-update")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the headless node
    Run {
        #[arg(long)]
        auto_update: bool, // Enable auto-update on startup
    },

    /// Check for updates and install
    Update {
        #[arg(long)]
        version: Option<String>, // Force specific version
    },

    /// Show current version
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Update { version }) => {
            if let Some(v) = version {
                self_update::update_to_version(&v).await?;
            } else {
                self_update::check_and_update().await?;
            }
        }
        Some(Commands::Run { auto_update }) => {
            if auto_update {
                if self_update::check_and_update().await? {
                    info!("⚠️  Updated! Restarting...");
                    // Systemd will restart us automatically
                    return Ok(());
                }
            }

            // Normal startup...
            run_headless_node().await?;
        }
        Some(Commands::Version) => {
            println!("communitas-headless v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            // Default: run node
            run_headless_node().await?;
        }
    }

    Ok(())
}
```

**Acceptance Criteria:**
- ✅ `communitas-headless update` checks and installs updates
- ✅ `communitas-headless run --auto-update` updates on startup
- ✅ `communitas-headless version` shows current version
- ✅ Progress bar during download
- ✅ Graceful fallback if update fails

**Estimated Effort:** 3 hours

---

#### Task 2.2: Implement Self-Update for communitas-tui

**File:** `communitas-tui/src/self_update.rs` (NEW)

**Implementation:** Same pattern as headless, with TUI-specific binary name

```rust
const BIN_NAME: &str = "communitas-tui";
```

**CLI Integration:** Add `update` subcommand to TUI

**Acceptance Criteria:**
- ✅ Same update functionality as headless
- ✅ Works from TUI interface
- ✅ Respects TUI --auto-update flag

**Estimated Effort:** 2 hours

---

#### Task 2.3: Implement Self-Update for communitas-desktop (Tauri)

**Challenge:** Tauri apps require platform-specific updaters

**Solutions:**

**Option A: Use Tauri's Built-in Updater (Recommended)**

```rust
// src-tauri/src/main.rs

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();

            // Check for updates on startup
            tauri::async_runtime::spawn(async move {
                match handle.updater().check().await {
                    Ok(update) => {
                        if update.is_update_available() {
                            println!("Update available: {}", update.latest_version());

                            // Download and install
                            update.download_and_install().await?;
                        }
                    }
                    Err(e) => eprintln!("Failed to check for updates: {}", e),
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Tauri Config:**

```json
// src-tauri/tauri.conf.json

{
  "tauri": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://api.github.com/repos/dirvine/communitas/releases/latest"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

**Option B: Use self_update Crate (Fallback)**

Same pattern as headless, but trigger from Tauri command:

```rust
#[tauri::command]
async fn check_for_updates() -> Result<UpdateStatus, String> {
    // Use self_update crate
}
```

**Acceptance Criteria:**
- ✅ Auto-check on startup
- ✅ User dialog to install updates
- ✅ Progress indicator
- ✅ Restart after update

**Estimated Effort:** 4 hours (includes Tauri config and signing)

---

### Phase 3: DigitalOcean Bootstrap Deployment

#### Task 3.1: Create Droplet via MCP

**Tool:** `mcp__digitalocean-mcp__droplet-create`

**Parameters:**
```json
{
  "Name": "communitas-bootstrap-1",
  "Size": "s-2vcpu-4gb",
  "Region": "nyc3",
  "ImageID": <ubuntu-22-04-id>,
  "Monitoring": true,
  "Backup": false
}
```

**Acceptance Criteria:**
- ✅ Droplet created with 2 vCPU, 4GB RAM
- ✅ Ubuntu 22.04 LTS
- ✅ Monitoring enabled
- ✅ NYC3 region for low latency

**Estimated Effort:** 30 minutes

---

#### Task 3.2: Droplet Provisioning Script

**File:** `scripts/provision-bootstrap.sh` (NEW)

**Script:**

```bash
#!/bin/bash
# Provision DigitalOcean droplet as Communitas bootstrap node

set -euo pipefail

GITHUB_REPO="dirvine/communitas"
RELEASE_VERSION="${1:-latest}"

echo "🚀 Provisioning Communitas Bootstrap Node"
echo "   Version: ${RELEASE_VERSION}"

# Update system
apt-get update
apt-get upgrade -y

# Install dependencies
apt-get install -y \
    curl \
    ca-certificates \
    systemd

# Create communitas user
useradd -r -s /bin/false communitas || true
mkdir -p /opt/communitas
mkdir -p /var/lib/communitas

# Download latest release
DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/communitas-headless-x86_64-unknown-linux-gnu.tar.gz"

if [ "$RELEASE_VERSION" != "latest" ]; then
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${RELEASE_VERSION}/communitas-headless-x86_64-unknown-linux-gnu.tar.gz"
fi

echo "📦 Downloading binary from: ${DOWNLOAD_URL}"
curl -L "${DOWNLOAD_URL}" | tar -xz -C /opt/communitas

# Make executable
chmod +x /opt/communitas/communitas-headless
chown -R communitas:communitas /opt/communitas
chown -R communitas:communitas /var/lib/communitas

# Create systemd service
cat > /etc/systemd/system/communitas-bootstrap.service <<'EOF'
[Unit]
Description=Communitas Bootstrap Node
After=network.target

[Service]
Type=simple
User=communitas
WorkingDirectory=/var/lib/communitas
ExecStart=/opt/communitas/communitas-headless run --auto-update
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/communitas

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
systemctl daemon-reload
systemctl enable communitas-bootstrap
systemctl start communitas-bootstrap

echo "✅ Bootstrap node installed and started"
echo ""
echo "📊 Status:"
systemctl status communitas-bootstrap --no-pager

echo ""
echo "📝 Logs:"
echo "  journalctl -u communitas-bootstrap -f"

# Get endpoint info
sleep 5
ENDPOINT=$(curl -s http://localhost:8080/api/v1/info | jq -r '.endpoint' || echo "pending")

echo ""
echo "🌐 Bootstrap Endpoint: ${ENDPOINT}"
```

**Acceptance Criteria:**
- ✅ Downloads latest Linux binary from GitHub releases
- ✅ Creates systemd service with auto-restart
- ✅ Runs as non-root user
- ✅ Security hardening applied
- ✅ Logs to systemd journal

**Estimated Effort:** 2 hours

---

#### Task 3.3: Deploy to DigitalOcean via MCP

**Automation Script:** `scripts/deploy-bootstrap-do.sh`

```bash
#!/bin/bash
# Deploy bootstrap node to DigitalOcean using MCP

set -euo pipefail

DROPLET_NAME="communitas-bootstrap-1"
REGION="nyc3"
SIZE="s-2vcpu-4gb"

echo "🔧 Creating droplet..."
# Use DigitalOcean MCP to create droplet
# (This would be done via Claude Code with MCP tool calls)

echo "⏳ Waiting for droplet to be ready..."
sleep 60

echo "📦 Copying provisioning script..."
DROPLET_IP=$(doctl compute droplet get ${DROPLET_NAME} --format PublicIPv4 --no-header)

scp scripts/provision-bootstrap.sh root@${DROPLET_IP}:/tmp/

echo "🚀 Running provisioning..."
ssh root@${DROPLET_IP} 'bash /tmp/provision-bootstrap.sh'

echo "✅ Deployment complete!"
echo ""
echo "🌐 Bootstrap endpoint: http://${DROPLET_IP}:8080"
echo "🔑 Four-word address: $(ssh root@${DROPLET_IP} '/opt/communitas/communitas-headless info' | grep 'Four Words')"
```

**Acceptance Criteria:**
- ✅ Droplet created via MCP
- ✅ Provisioning script executed remotely
- ✅ Service running and healthy
- ✅ Endpoint accessible

**Estimated Effort:** 2 hours

---

#### Task 3.4: Extract Bootstrap Endpoint

**Goal:** Get four-word address or IP:port for hardcoding

**Implementation:**

```bash
# On the droplet:
/opt/communitas/communitas-headless info

# Output:
# Communitas Headless Node
# Version: 0.1.17
# Four Words: ocean-forest-moon-star
# Endpoint: 138.68.123.45:8080
# Status: Running
```

**Save to Config:**

```rust
// communitas-core/src/bootstrap.rs

pub const DEFAULT_BOOTSTRAP_NODES: &[&str] = &[
    "138.68.123.45:8080",  // DigitalOcean NYC3
    // Add more as needed
];

pub const DEFAULT_BOOTSTRAP_FOUR_WORDS: &[&str] = &[
    "ocean-forest-moon-star",  // DigitalOcean NYC3
];
```

**Acceptance Criteria:**
- ✅ Endpoint extracted and verified
- ✅ Hardcoded in communitas-core
- ✅ Used by desktop/TUI apps

**Estimated Effort:** 1 hour

---

### Phase 4: Integration & Testing

#### Task 4.1: Update Desktop App with Bootstrap Endpoint

**File:** `src-tauri/src/core_commands.rs`

```rust
use communitas_core::bootstrap::DEFAULT_BOOTSTRAP_NODES;

#[tauri::command]
pub async fn core_initialize(
    four_words: String,
    display_name: String,
    device_name: String,
    state: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<(), String> {
    // ... existing code ...

    // Connect to bootstrap nodes
    for endpoint in DEFAULT_BOOTSTRAP_NODES {
        info!("Connecting to bootstrap node: {}", endpoint);
        ctx.connect_to_bootstrap(endpoint).await
            .map_err(|e| format!("Bootstrap connection failed: {}", e))?;
    }

    // ... existing code ...
}
```

**Acceptance Criteria:**
- ✅ Desktop app connects to hardcoded bootstrap on init
- ✅ Fallback to local mode if bootstrap unreachable
- ✅ Retry logic with exponential backoff

**Estimated Effort:** 2 hours

---

#### Task 4.2: Testing Plan

**Test Scenarios:**

1. **Self-Update Test**
   - [ ] Create test release v0.1.18
   - [ ] Run `communitas-headless update` on v0.1.17
   - [ ] Verify binary updated to v0.1.18
   - [ ] Test rollback if update fails

2. **Bootstrap Connection Test**
   - [ ] Start fresh desktop app
   - [ ] Verify connection to DO bootstrap node
   - [ ] Check peer discovery
   - [ ] Verify message propagation

3. **Multi-Node Test**
   - [ ] Start 3 desktop instances
   - [ ] All connect through bootstrap
   - [ ] Send messages between all peers
   - [ ] Verify CRDT sync

4. **Failover Test**
   - [ ] Stop bootstrap node
   - [ ] Verify apps fallback to local mode
   - [ ] Restart bootstrap
   - [ ] Verify automatic reconnection

**Acceptance Criteria:**
- ✅ All test scenarios pass
- ✅ No crashes or hangs
- ✅ Logs show expected behavior

**Estimated Effort:** 4 hours

---

#### Task 4.3: Monitoring & Health Checks

**File:** `communitas-headless/src/health.rs` (NEW)

```rust
use warp::Filter;

pub async fn start_health_server(port: u16) {
    let health = warp::path("health")
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "status": "ok",
                "version": env!("CARGO_PKG_VERSION"),
                "uptime": get_uptime_seconds(),
                "peers": get_peer_count(),
            }))
        });

    warp::serve(health).run(([0, 0, 0, 0], port)).await;
}
```

**DigitalOcean Monitoring:**
- Set up HTTP health check: `http://138.68.123.45:8080/health`
- Alert if health check fails for 5 minutes
- Email notifications

**Acceptance Criteria:**
- ✅ /health endpoint returns 200 OK
- ✅ DO monitoring configured
- ✅ Alerts sent on failure

**Estimated Effort:** 2 hours

---

## File Structure

```
communitas/
├── .github/
│   └── workflows/
│       ├── release-headless.yml        # ✅ Exists, needs TUI addition
│       └── release-summary.yml         # ✅ Exists
├── communitas-headless/
│   └── src/
│       ├── main.rs                     # ✅ Exists, needs update commands
│       ├── self_update.rs              # ❌ NEW
│       └── health.rs                   # ❌ NEW
├── communitas-tui/
│   └── src/
│       ├── main.rs                     # ✅ Exists, needs update commands
│       └── self_update.rs              # ❌ NEW
├── communitas-desktop/
│   └── src-tauri/
│       ├── src/
│       │   ├── main.rs                 # ✅ Exists, needs updater setup
│       │   └── core_commands.rs        # ✅ Exists, needs bootstrap
│       └── tauri.conf.json             # ✅ Exists, needs updater config
├── communitas-core/
│   └── src/
│       └── bootstrap.rs                # ❌ NEW (constants)
├── scripts/
│   ├── provision-bootstrap.sh          # ❌ NEW
│   └── deploy-bootstrap-do.sh          # ❌ NEW
└── docs/
    └── plans/
        └── 2025-10-12-bootstrap-deployment-selfupdate.md  # This file
```

## Execution Timeline

### Week 1: Self-Update Implementation
- **Days 1-2:** Task 2.1 (Headless self-update)
- **Day 3:** Task 2.2 (TUI self-update)
- **Days 4-5:** Task 2.3 (Desktop self-update)

### Week 2: Deployment & Integration
- **Day 1:** Task 1.1 (Release workflow enhancement)
- **Day 2:** Task 3.1-3.2 (Droplet creation and provisioning)
- **Day 3:** Task 3.3-3.4 (Deployment and endpoint extraction)
- **Day 4:** Task 4.1 (Desktop app integration)
- **Day 5:** Task 4.2-4.3 (Testing and monitoring)

**Total Effort:** ~25 hours (2 weeks)

## Risk Mitigation

### Risk 1: Self-Update Breaks Binary
**Mitigation:**
- Always keep backup of previous version
- Implement rollback mechanism
- Test updates in staging environment first

### Risk 2: Bootstrap Node Goes Down
**Mitigation:**
- Deploy multiple bootstrap nodes in different regions
- Implement automatic failover
- Apps work offline without bootstrap

### Risk 3: GitHub Rate Limits
**Mitigation:**
- Cache update checks (once per hour max)
- Use authenticated GitHub API calls
- Implement exponential backoff

### Risk 4: DigitalOcean Costs
**Mitigation:**
- Start with smallest droplet ($12/month)
- Monitor bandwidth usage
- Set up billing alerts

## Success Metrics

- ✅ **GitHub Releases:** Automated releases for all 3 binaries
- ✅ **Self-Update:** All binaries can update themselves from GitHub
- ✅ **Bootstrap Deployed:** Production node running on DigitalOcean
- ✅ **Apps Connected:** Desktop/TUI connect to bootstrap on startup
- ✅ **Uptime:** Bootstrap node > 99% uptime
- ✅ **Update Success Rate:** > 95% successful auto-updates

## Cost Estimate

- **DigitalOcean Droplet:** $12/month (s-2vcpu-4gb)
- **Bandwidth:** ~$0.01/GB (minimal for bootstrap)
- **Monitoring:** Included in droplet price
- **Total:** ~$15/month

## Future Enhancements

1. **Multi-Region Bootstrap Nodes**
   - Deploy in NYC, SFO, LON, SGP
   - Geo-routing for best latency

2. **Health Dashboard**
   - Web UI showing bootstrap node status
   - Peer count, message throughput, uptime

3. **Automated Scaling**
   - Spin up more nodes when peer count > 1000
   - Scale down during low usage

4. **Update Channels**
   - Stable, beta, nightly channels
   - Users choose update frequency

5. **Signed Binaries**
   - Code signing for macOS/Windows
   - Verify binary authenticity before update

## References

- [self_update crate docs](https://docs.rs/self_update/)
- [Tauri Updater docs](https://tauri.app/v1/guides/distribution/updater)
- [DigitalOcean MCP tools](mcp__digitalocean-mcp__)
- [GitHub Releases API](https://docs.github.com/en/rest/releases)

---

**Next Steps:**
1. Review and approve this plan
2. Create GitHub issues for each task
3. Start with Phase 1 (release enhancements)
4. Deploy bootstrap node to staging first
5. Roll out to production after testing

**Questions/Blockers:**
- [ ] Confirm DigitalOcean region preference (NYC3 vs others)
- [ ] Approve monthly cost estimate ($15/month)
- [ ] Decide on code signing requirements
- [ ] Choose update channel strategy
