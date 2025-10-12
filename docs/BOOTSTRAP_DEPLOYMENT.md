# Bootstrap Node Deployment Guide

**Date:** 2025-10-12
**Bootstrap Node:** NYC Production
**IP Address:** 138.197.29.195
**QUIC Port:** 8080
**Metrics Port:** 9600

## Overview

This document describes the production bootstrap node deployment for Communitas. Bootstrap nodes are "introducer nodes" that help new users discover the P2P network when they have no existing contacts.

## Deployed Infrastructure

### Droplet Configuration
- **Provider:** DigitalOcean NYC3 region
- **Size:** s-1vcpu-2gb (1 CPU, 2GB RAM, 50GB SSD)
- **OS:** Ubuntu 24.04 LTS x64
- **Monitoring:** Enabled
- **Droplet ID:** 523969479

### Network Configuration
- **Public IPv4:** 138.197.29.195
- **QUIC Listen:** 0.0.0.0:8080
- **Metrics Listen:** 0.0.0.0:9600
- **QUIC Server Key (hex):** 5ce22da6cefb6e766e6d9c3775248bd92375de7c2d225135a01b7057cbf9d988

## Quick Reference

### Service Commands
```bash
# Status
sudo systemctl status communitas-bootstrap.service

# Logs
sudo journalctl -u communitas-bootstrap.service -f

# Restart
sudo systemctl restart communitas-bootstrap.service
```

### Testing Connection
```bash
# Check QUIC port
nc -zv 138.197.29.195 8080

# Check metrics
curl http://138.197.29.195:9600/metrics
```

## Code Integration

The bootstrap node is configured as the default introducer in `communitas-core/src/gossip/discovery.rs:256-265`:

```rust
impl Default for IntroducerConfig {
    fn default() -> Self {
        Self {
            addresses: vec![
                "138.197.29.195:8080".to_string(), // NYC bootstrap node
            ],
            timeout_secs: 10,
        }
    }
}
```

## Commits
- `45a418eb` - feat: Add production bootstrap node to default introducer config
- `b210109f` - feat: Add Tauri updater plugins for desktop app updates

