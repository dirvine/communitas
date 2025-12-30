# NAT Emulation Docker Setup

Local Docker-based NAT emulation for testing Communitas P2P connectivity.

## Prerequisites

- Docker Engine 20.10+
- Docker Compose v2
- Linux kernel with iptables support (Docker Desktop works on macOS/Windows)
- `communitas-headless` binary built

## Quick Start

```bash
# Build release binary
cargo build -p communitas-headless --release

# Navigate to NAT emulation directory
cd docker/nat-emulation

# Build all NAT containers
docker-compose build

# Start all NAT types
docker-compose up -d

# Verify containers running
docker-compose ps
```

## Container Architecture

Each NAT type has two containers:
1. **NAT Router** (`nat-<type>`) - Implements iptables NAT rules
2. **Test Node** (`node-<type>`) - Client behind the NAT

```
node-fullcone (10.100.1.10)
        |
        v
nat-fullcone (10.100.1.1 / 172.20.1.1)
        |
        v
nat-external network (172.20.0.0/16)
        |
        v
node-public (172.20.100.1)
```

## Starting Specific NAT Types

```bash
# Start only symmetric NAT for testing
docker-compose up -d nat-symmetric node-symmetric

# Start symmetric + public for relay testing
docker-compose up -d nat-symmetric node-symmetric node-public

# Start multiple NAT types
docker-compose up -d \
    nat-symmetric node-symmetric \
    nat-cgnat node-cgnat \
    node-public
```

## Running Communitas in Containers

```bash
# Start bootstrap on public node
docker exec -d communitas-node-public \
    /usr/local/bin/communitas-headless \
    --listen 0.0.0.0:11000

# Connect from behind NAT
docker exec -it communitas-node-symmetric \
    /usr/local/bin/communitas-headless \
    --listen 0.0.0.0:11000 \
    --bootstrap 172.20.100.1:11000
```

## Monitoring NAT State

### View conntrack table
```bash
docker exec communitas-nat-symmetric conntrack -L -p udp
```

### Watch NAT mappings
```bash
docker exec communitas-nat-symmetric watch -n 1 'conntrack -L -p udp | grep 11000'
```

### Check iptables rules
```bash
docker exec communitas-nat-symmetric iptables -t nat -L -n -v
```

## Custom Binary

To test a different binary:

```bash
# Copy binary into container
docker cp ./my-custom-binary communitas-node-symmetric:/usr/local/bin/

# Or mount a different path in docker-compose.yml
volumes:
  - ./path/to/binary:/usr/local/bin/communitas-headless:ro
```

## Cleanup

```bash
# Stop all containers
docker-compose down

# Stop and remove volumes
docker-compose down -v

# Remove built images
docker-compose down --rmi all
```

## Troubleshooting

### "Binary not found"
```bash
# Ensure binary is built
cargo build -p communitas-headless --release
ls -la ../../target/release/communitas-headless

# Verify mount in container
docker exec communitas-node-fullcone ls -la /usr/local/bin/
```

### "Permission denied"
```bash
# Make binary executable
chmod +x ../../target/release/communitas-headless
```

### "Network unreachable"
```bash
# Check container routing
docker exec communitas-node-fullcone ip route

# Should show default via 10.100.1.1
```

### "Container won't start"
```bash
# Check logs
docker-compose logs nat-symmetric

# Rebuild specific container
docker-compose build --no-cache nat-symmetric
docker-compose up -d nat-symmetric
```
