# Communitas Testnet Deployment Guide

## Overview
This guide covers deploying Communitas binaries to your Digital Ocean testnet infrastructure using two approaches:

1. **SSH Deployment** (Recommended for development/testing)
2. **Docker Deployment** (Recommended for production)

## Current Status
✅ **Infrastructure**: 7-node global testnet deployed
- 1 Bootstrap Coordinator (NYC)
- 6 Regional Worker Nodes (NYC, SFO, LON, FRA, SGP, SYD)
- PostgreSQL metrics database

✅ **Containers**: Ubuntu 22.04 with build tools installed
✅ **SSH Access**: Configured for all containers
❌ **Binaries**: Need to be built and deployed

---

## 🚀 Quick Start (Docker Approach - Recommended)

### Prerequisites
- Docker Desktop for Mac installed
- Docker Hub account
- Rust toolchain installed

### Build and Deploy
```bash
# Install cross-compilation tool
cargo install cross

# Run the build and deploy script
chmod +x scripts/build-and-deploy-docker.sh
./scripts/build-and-deploy-docker.sh v0.1.0

# Login to Docker Hub when prompted
docker login

# The script will:
# 1. Build Linux binaries using cross-compilation
# 2. Build Docker image with binaries
# 3. Push to Docker Hub
# 4. Generate updated Digital Ocean app spec
```

### Update Digital Ocean App
1. Go to Digital Ocean App Platform dashboard
2. Update your app specification with `do-app-spec-updated.json`
3. Deploy the updated specification
4. The containers will automatically run the Communitas binaries

---

## 🔧 Alternative: SSH Deployment Approach

### 1. Build Binaries on GitHub
```bash
# Create a new release tag to trigger binary build
git tag v0.1.0
git push origin v0.1.0
```

The `release-binaries.yml` workflow will:
- Build `communitas-node`, `bootstrap`, and `communitas-autoupdater` binaries
- Create `communitas-binaries-linux-x86_64.tar.gz` release artifact
- Upload to GitHub Releases

### 2. Collect Node IP Addresses
```bash
# Run on a Linux machine with SSH access
chmod +x scripts/collect-node-ips.sh
./scripts/collect-node-ips.sh
```

### 3. Deploy Binaries to Nodes
```bash
# Run on a Linux machine with SSH access
chmod +x scripts/deploy-testnet-binaries.sh
./scripts/deploy-testnet-binaries.sh v0.1.0
```

---

## 📋 Node Configuration

### Ports
- Bootstrap Coordinator: 8080
- NYC Node: 8081
- SFO Node: 8082
- LON Node: 8083
- FRA Node: 8084
- SGP Node: 8085
- SYD Node: 8086

### Regions
- **North America**: NYC, SFO
- **Europe**: LON, FRA
- **Asia-Pacific**: SGP, SYD

---

## 🧪 Testing the Network

### 1. Check Node Health
```bash
# Test bootstrap node
curl https://communitas-testnet.ondigitalocean.app/health

# Test individual nodes (if exposed)
curl https://communitas-testnet.ondigitalocean.app/node-nyc/health
```

### 2. Test P2P Connectivity
```bash
# Run local Communitas app
npm run tauri dev

# The app will connect to testnet via bootstrap node
```

### 3. Monitor Network
```bash
# Check metrics database
# Connect via Digital Ocean dashboard or psql
```

---

## 🐳 Docker Approach Details

### Dockerfile Features
- Ubuntu 22.04 base image
- Pre-installed system dependencies
- Health checks configured
- Binaries included in image
- Configurable via environment variables

### Build Process
1. **Cross-compile** Rust binaries for Linux x86_64
2. **Build Docker image** with binaries included
3. **Push to registry** (Docker Hub)
4. **Update DO app spec** to use new image
5. **Deploy updated spec** to infrastructure

### Advantages
- ✅ **Atomic deployments** - All nodes get same binary version
- ✅ **Easy rollbacks** - Just change Docker tag
- ✅ **No SSH required** - Fully automated
- ✅ **Reproducible** - Same image across all environments

---

## 🔧 SSH Approach Details

### Deployment Process
1. **Build on GitHub** - Automated binary compilation
2. **Download releases** - Fetch binaries from GitHub
3. **SCP to containers** - Copy binaries via SSH
4. **Configure nodes** - Update bootstrap configuration
5. **Start services** - Launch nodes with proper config

### Advantages
- ✅ **Direct access** - Full control over containers
- ✅ **Debugging** - Easy to inspect running processes
- ✅ **Development friendly** - Quick iteration cycles
- ✅ **No registry needed** - Direct file transfer

---

## 📁 File Structure
```
scripts/
├── build-and-deploy-docker.sh    # Docker deployment (recommended)
├── deploy-testnet-binaries.sh    # SSH deployment
└── collect-node-ips.sh          # IP collection utility

Dockerfile.testnet                 # Testnet Docker image
bootstrap.toml.template           # Configuration template
.github/workflows/
└── release-binaries.yml         # GitHub Actions for SSH approach
```

---

## 🚨 Troubleshooting

### Docker Issues
```bash
# Check Docker is running
docker ps

# Test cross-compilation
cross build --target x86_64-unknown-linux-gnu --bin communitas-node

# Check image exists
docker images | grep communitas
```

### SSH Issues
```bash
# Test SSH connectivity
ssh -i ~/.ssh/id_rsa -v root@app-18ccbf3a-7111-4897-a0b6-f215910bcf1a-bootstrap-coordinator.ondigitalocean.app

# Check SSH key permissions
chmod 600 ~/.ssh/id_rsa
```

### Binary Issues
```bash
# Verify binary architecture
file communitas-node

# Check dependencies
ldd communitas-node

# Test binary execution
./communitas-node --help
```

---

## 🎯 Next Steps
1. ✅ Choose deployment approach (Docker recommended)
2. ✅ Build and deploy binaries
3. ✅ Update Digital Ocean app specification
4. ✅ Test network connectivity
5. ✅ Connect local Communitas app
6. ✅ Monitor and optimize performance

---

## 📞 Support
- Check Digital Ocean app logs in dashboard
- Monitor GitHub Actions for build issues
- Review container health in DO App Platform
- Test connectivity with local Communitas app