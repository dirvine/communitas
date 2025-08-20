# Communitas - Local Testnet Setup Guide

## 🚀 Quick Start - Local Development

### Prerequisites
- Rust 1.85+ with Tauri CLI
- Node.js 18+ with npm
- Docker (optional, for containerized bootstrap)

### 1. Clone and Setup
```bash
git clone https://github.com/dirvine/communitas.git
cd communitas
npm install
```

### 2. Start Local Bootstrap Node
```bash
# Option A: Using Docker (Recommended)
docker run -d \
  --name communitas-bootstrap \
  -p 9001:9001 \
  -p 9100:9100 \
  -p 9110:9110 \
  -p 9120:9120 \
  saorsa/bootstrap-node:latest

# Option B: Using local Rust bootstrap (if available)
cargo run --bin bootstrap-node -- --port 9001
```

### 3. Configure Environment
```bash
# Set local bootstrap address
export COMMUNITAS_LOCAL_BOOTSTRAP="127.0.0.1:9001"

# Optional: Set local data directory
export COMMUNITAS_DATA_DIR="./communitas-test-data"
```

### 4. Run the Application
```bash
# Development mode
npm run tauri dev

# Or build and run
npm run tauri build
npm run tauri dev
```

## 🏗️ Setting Up a Local Testnet

### Architecture Overview
```
┌─────────────────┐    ┌─────────────────┐
│   Bootstrap     │────│     Node 1      │
│   Node (Port    │    │   (Port 9002)   │
│    9001)        │    └─────────────────┘
└─────────────────┘             │
         │                     │
         │                     │
         │              ┌─────────────────┐
         └──────────────│     Node 2      │
                        │   (Port 9003)   │
                        └─────────────────┘
```

### Bootstrap Node Setup

#### Option 1: Docker Container
```bash
# Build bootstrap image
docker build -f Dockerfile.bootstrap -t communitas-bootstrap .

# Run bootstrap node
docker run -d \
  --name communitas-bootstrap \
  -p 9001:9001 \
  -p 9100:9100 \
  -p 9110:9110 \
  -p 9120:9120 \
  -e RUST_LOG=info \
  communitas-bootstrap
```

#### Option 2: Local Bootstrap Node
```rust
// Create a simple bootstrap node
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9001".parse::<SocketAddr>()?;
    let listener = TcpListener::bind(addr).await?;

    println!("Bootstrap node listening on {}", addr);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        println!("New connection from {}", peer_addr);

        // Handle P2P handshake
        tokio::spawn(async move {
            // Implement basic P2P protocol handling
            handle_peer_connection(socket).await;
        });
    }
}
```

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

## 🔧 Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `COMMUNITAS_LOCAL_BOOTSTRAP` | Bootstrap node address | Production bootstrap |
| `COMMUNITAS_P2P_PORT` | P2P listening port | Auto-assigned |
| `COMMUNITAS_DATA_DIR` | Data storage directory | `./communitas-data` |
| `RUST_LOG` | Logging level | `info` |
| `COMMUNITAS_TESTNET` | Enable testnet mode | `false` |

## 🧪 Testing the Testnet

### 1. Health Check
```bash
curl http://localhost:1420/health
```

### 2. Network Status
```bash
# Check peer connections
curl http://localhost:1420/api/network/status
```

### 3. Create Test Data
```bash
# Create a test organization
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