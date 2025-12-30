# NAT Emulation Docker Infrastructure

Docker-based NAT emulation for comprehensive Communitas P2P testing.

## Quick Start

```bash
# Build all NAT emulators
docker-compose build

# Start all NAT emulators
docker-compose up -d

# Start specific NAT type
docker-compose up -d nat-symmetric node-symmetric

# View logs
docker-compose logs -f nat-cgnat

# Stop all
docker-compose down
```

## Prerequisites

- Docker with Compose v2
- Linux kernel with iptables support
- `communitas-headless` binary built: `cargo build -p communitas-headless --release`

## NAT Types Emulated

| Type | Container | Difficulty | Common In |
|------|-----------|------------|-----------|
| Full Cone | `nat-fullcone` | Easy | Gaming routers, UPnP-enabled |
| Address-Restricted | `nat-restricted` | Medium | Older home routers |
| Port-Restricted | `nat-portrestricted` | Medium | Most home routers (default) |
| Symmetric | `nat-symmetric` | Very Hard | Enterprise NAT, some ISPs |
| CGNAT | `nat-cgnat` | Hard | ISPs, mobile carriers |
| Double NAT | `nat-doublenat-*` | Very Hard | Apartments, dorms |
| Hairpin | `nat-hairpin` | Special | Better home routers |

## Network Architecture

```
                         nat-external (172.20.0.0/16)
                                    |
    +------------------+------------+------------+------------------+
    |                  |            |            |                  |
nat-fullcone      nat-symmetric  nat-cgnat  nat-doublenat-outer  node-public
(172.20.1.1)      (172.20.4.1)   (172.20.5.1)  (172.20.6.1)     (172.20.100.1)
    |                  |            |            |
internal-1.0      internal-4.0  internal-5.0  middle-1.0
(10.100.1.0/24)   (10.100.4.0/24)(10.100.5.0/24)(10.200.1.0/24)
    |                  |            |            |
node-fullcone     node-symmetric node-cgnat  nat-doublenat-inner
(10.100.1.10)     (10.100.4.10)  (10.100.5.10)  (10.200.1.10)
                                                     |
                                              internal-6.0
                                              (10.100.6.0/24)
                                                     |
                                              node-doublenat
                                              (10.100.6.10)
```

## Communitas Port

**IMPORTANT**: Communitas uses UDP port **11000** (not 9000 like ant-quic).

```bash
# Run communitas-headless inside NAT container
docker exec -it communitas-node-symmetric /usr/local/bin/communitas-headless \
    --listen 0.0.0.0:11000 \
    --bootstrap 172.20.100.1:11000
```

## NAT Behavior Reference (RFC 4787)

### Mapping Behavior
- **Endpoint Independent (EIM)**: Same external port for all destinations (cone NATs)
- **Address Dependent (ADM)**: Different port per destination IP
- **Address+Port Dependent (APDM)**: Different port per destination IP:port (symmetric)

### Filtering Behavior
- **Endpoint Independent (EIF)**: Accept from any external host (full cone)
- **Address Dependent (ADF)**: Only accept from IPs we've sent to (restricted)
- **Address+Port Dependent (APDF)**: Only from exact IP:port we sent to (port-restricted)

### NAT Type Matrix

| NAT Type | Mapping | Filtering | Hole-Punch |
|----------|---------|-----------|------------|
| Full Cone | EIM | EIF | Yes (easy) |
| Address-Restricted | EIM | ADF | Yes |
| Port-Restricted | EIM | APDF | Yes (with coordination) |
| Symmetric | APDM | APDF | Relay required |

## Testing Scenarios

### 1. Simple Connectivity Test

```bash
# Start public node and one NAT type
docker-compose up -d node-public nat-fullcone node-fullcone

# Run bootstrap on public node
docker exec communitas-node-public /usr/local/bin/communitas-headless \
    --listen 0.0.0.0:11000 &

# Connect from behind NAT
docker exec communitas-node-fullcone /usr/local/bin/communitas-headless \
    --listen 0.0.0.0:11000 \
    --bootstrap 172.20.100.1:11000
```

### 2. NAT-to-NAT Connectivity Matrix

```bash
# Start all NAT types
docker-compose up -d

# Test connectivity between different NAT types
./test-nat-matrix.sh
```

### 3. Worst Case: Symmetric-to-Symmetric

```bash
# This requires relay - both nodes behind symmetric NAT
docker exec communitas-node-symmetric /usr/local/bin/communitas-headless \
    --listen 0.0.0.0:11000 \
    --bootstrap 172.20.100.1:11000

# Observe relay usage in logs
docker-compose logs -f nat-symmetric
```

## Monitoring

### Watch conntrack table
```bash
docker exec communitas-nat-symmetric watch -n 1 conntrack -L -p udp
```

### Check NAT rules
```bash
docker exec communitas-nat-symmetric iptables -t nat -L -n -v
```

### View port mappings
```bash
docker exec communitas-nat-symmetric conntrack -L -p udp | grep 11000
```

## Troubleshooting

### Container can't reach external network
```bash
# Check IP forwarding
docker exec communitas-nat-symmetric cat /proc/sys/net/ipv4/ip_forward

# Check iptables rules
docker exec communitas-nat-symmetric iptables -L -n -v
docker exec communitas-nat-symmetric iptables -t nat -L -n -v
```

### Binary not found
```bash
# Ensure release binary is built
cargo build -p communitas-headless --release

# Verify mount
docker exec communitas-node-fullcone ls -la /usr/local/bin/
```

### Port exhaustion with CGNAT
```bash
# Check available ports
docker exec communitas-nat-cgnat cat /proc/sys/net/ipv4/ip_local_port_range

# Check current connections
docker exec communitas-nat-cgnat conntrack -L | wc -l
```

## Adding New NAT Types

1. Create directory: `mkdir nat-newtype`
2. Copy Dockerfile from similar NAT type
3. Create `entrypoint.sh` with iptables rules
4. Add to `docker-compose.yml`
5. Test connectivity

## Integration with VPS Fleet

For combined local + VPS testing:

```bash
# Start local NAT emulation
docker-compose up -d

# VPS nodes are at:
# saorsa-2.saorsalabs.com:11000 (NYC)
# saorsa-3.saorsalabs.com:11000 (SFO)

# Connect local NAT node to VPS bootstrap
docker exec communitas-node-symmetric /usr/local/bin/communitas-headless \
    --listen 0.0.0.0:11000 \
    --bootstrap 142.93.199.50:11000
```

## References

- [RFC 4787: NAT Behavioral Requirements for UDP](https://datatracker.ietf.org/doc/html/rfc4787)
- [RFC 3489: STUN NAT Classification](https://datatracker.ietf.org/doc/html/rfc3489)
- [RFC 6598: CGNAT Address Space](https://datatracker.ietf.org/doc/html/rfc6598)
