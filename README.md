# Communitas — The Local‑First Collaboration App (saorsa-core)

Communitas is a local‑first, PQC‑ready collaboration app that merges the best of WhatsApp, Dropbox, Zoom, and Slack into one experience — without centralized servers or DNS. Identities are human‑verifiable four words, storage is per‑entity virtual disks (org, group, channel, project, individual), and websites are published without DNS via identity‑bound website roots.

Two apps, one core:
- Desktop app (Tauri v2): rich UI with Tauri commands for automation
- Headless node: bootstrap/seeding and personal nodes (future rewards)

Backed by Saorsa Core (crates.io `saorsa-core`, v0.3.17): DHT, QUIC, identities, groups, messaging, virtual disks, and security.

## 🚀 Quick Start (Desktop Dev)

Prerequisites
- Node 20+
- Rust 1.85+
- Platform deps for Tauri

Setup & Run
```bash
git clone https://github.com/dirvine/communitas.git
cd communitas
npm install
npm run tauri dev
```

## Headless Nodes & Testnet

Deploy seeds and personal nodes across regions with systemd and cloud‑init. See `finalise/DEPLOY_TESTNET.md` for DigitalOcean templates, ports, and four‑word endpoints.

### Multiple Node Setup

#### Node 1 Configuration
```bash
# Terminal 1
export COMMUNITAS_LOCAL_BOOTSTRAP="127.0.0.1:9001"
export COMMUNITAS_P2P_PORT="9002"
export COMMUNITAS_DATA_DIR="./node1-data"

npm run tauri dev
```

#### Node 2 Configuration
```bash
# Terminal 2
export COMMUNITAS_LOCAL_BOOTSTRAP="127.0.0.1:9001"
export COMMUNITAS_P2P_PORT="9003"
export COMMUNITAS_DATA_DIR="./node2-data"

npm run tauri dev
```

#### Node 3 Configuration
```bash
# Terminal 3
export COMMUNITAS_LOCAL_BOOTSTRAP="127.0.0.1:9001"
export COMMUNITAS_P2P_PORT="9004"
export COMMUNITAS_DATA_DIR="./node3-data"

npm run tauri dev
```

## Architecture (at a glance)

- Frontend: React + TypeScript + MUI (Tauri WebView)
- Backend: Rust (Tauri v2)
- Core: saorsa-core (DHT, QUIC, identities, groups, messaging, virtual disks)
- Crypto: PQC (ML‑DSA/ML‑KEM); XChaCha20‑Poly1305 for sealing
- Storage: content addressed, FEC‑sealed objects, per‑entity virtual disks

## Docs & Automation

- Communitas agents API: `AGENTS_API.md`
- Saorsa core agents API: `../saorsa-core/AGENTS_API.md`
- Testnet & bootstrap: `finalise/DEPLOY_TESTNET.md`

## Security & Lint Policy

- No panics/unwrap/expect in production code
- Clippy policy:
  - `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`
- Formatting: `cargo fmt --all` before commits/CI

## Contributing

- Conventional commits (`feat:`, `fix:`, `docs:`, `chore:`, etc.)
- Ensure:
  - `npm run typecheck`
  - `cargo fmt --all`
  - `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`

## License

AGPL‑3.0 for open collaboration. Commercial licensing available via Saorsa Labs.

---

### Repository settings (for maintainers)

- Suggested GitHub description:
  - “Local‑first PQC collaboration: messaging, channels, virtual disks per entity, and DNS‑free websites with Four‑Word identities. Powered by saorsa-core.”
- Suggested topics:
  - `p2p`, `quic`, `post-quantum`, `tauri`, `webrtc`, `dht`, `decentralized`, `collaboration`, `virtual-disk`, `dnsless-web`
curl -X POST http://localhost:1420/api/organizations \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Organization", "description": "Local testnet org"}'
```

### 4. Verify P2P Connectivity
```bash
# Check if nodes can discover each other
curl http://localhost:1420/api/network/peers
```

## 🐳 Docker Compose Setup

Create `docker-compose.yml`:
```yaml
version: '3.8'

services:
  bootstrap:
    build:
      context: .
      dockerfile: Dockerfile.bootstrap
    ports:
      - "9001:9001"
      - "9100:9100"
      - "9110:9110"
      - "9120:9120"
    environment:
      - RUST_LOG=info

  node1:
    build: .
    depends_on:
      - bootstrap
    environment:
      - COMMUNITAS_LOCAL_BOOTSTRAP=bootstrap:9001
      - COMMUNITAS_P2P_PORT=9002
      - COMMUNITAS_DATA_DIR=/app/node1-data
    volumes:
      - ./node1-data:/app/node1-data

  node2:
    build: .
    depends_on:
      - bootstrap
    environment:
      - COMMUNITAS_LOCAL_BOOTSTRAP=bootstrap:9001
      - COMMUNITAS_P2P_PORT=9003
      - COMMUNITAS_DATA_DIR=/app/node2-data
    volumes:
      - ./node2-data:/app/node2-data
```

Run with:
```bash
docker-compose up -d
```

## 🔍 Monitoring & Debugging

### Logs
```bash
# View application logs
docker-compose logs -f

# View specific service logs
docker-compose logs -f bootstrap
```

### Network Inspection
```bash
# Check container networking
docker network ls
docker inspect communitas_default

# Test connectivity between containers
docker exec communitas_node1 curl http://bootstrap:9001/health
```

### Performance Monitoring
```bash
# Monitor resource usage
docker stats

# Check health endpoints
curl http://localhost:1420/health
curl http://localhost:1421/health  # Node 2
curl http://localhost:1422/health  # Node 3
```

## 🛠️ Troubleshooting

### Common Issues

#### Bootstrap Node Not Starting
```bash
# Check bootstrap logs
docker-compose logs bootstrap

# Verify port binding
netstat -tlnp | grep 9001
```

#### Nodes Can't Connect
```bash
# Check network connectivity
docker exec communitas_node1 ping bootstrap

# Verify environment variables
docker exec communitas_node1 env | grep COMMUNITAS
```

#### Port Conflicts
```bash
# Find conflicting processes
lsof -i :9001

# Kill conflicting process
kill $(lsof -t -i :9001)
```

### Debug Mode
```bash
# Enable debug logging
export RUST_LOG=debug,communitas=trace,saorsa_core=debug

# Run with debug symbols
npm run tauri dev -- --debug
```

## 📊 Testnet Metrics

Monitor these key metrics:
- **Peer Count**: Number of connected nodes
- **Message Latency**: Average message delivery time
- **Storage Operations**: DHT put/get success rates
- **Network Health**: Connection stability and uptime

## 🗺️ Roadmap & Future Features

### Current Status
- ✅ Basic Tauri application structure
- ✅ React frontend with TypeScript
- ✅ Rust backend integration
- ✅ Development environment setup
- 🚧 Core UI component library
- 📅 User authentication system
- 📅 Real-time messaging
- 📅 File sharing capabilities

### Upcoming Features
- **Real-time Collaboration**: Yjs CRDTs for collaborative editing
- **Voice/Video Calling**: WebRTC integration with PQC
- **Local AI Integration**: On-device language model for assistance
- **Mobile Applications**: Native iOS/Android clients
- **Storage Market**: Paid storage for large public datasets
- **Federated Search**: Search across four-word identities

### Long-term Vision
- **Plugin System**: Extensible architecture for third-party integrations
- **Advanced Collaboration**: Real-time collaborative document editing
- **Cross-platform Sync**: Seamless experience across all devices
- **Decentralized Governance**: Community-driven feature development

## 🎯 Next Steps

1. **Scale Testing**: Add more nodes to test scalability
2. **Load Testing**: Simulate high message volumes
3. **Security Testing**: Test encryption and authentication
4. **Performance Optimization**: Monitor and optimize resource usage

## 📞 Support

For issues with local testnet setup:
1. Check the troubleshooting section above
2. Review container logs: `docker-compose logs`
3. Verify network connectivity between containers
4. Ensure all required ports are available

---

**Happy testing! 🎉**
