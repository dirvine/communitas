# NAT Scenarios and Testing Matrix

## NAT Type Behavior (RFC 4787)

### Mapping Types
| Type | Behavior | Description |
|------|----------|-------------|
| EIM | Endpoint Independent | Same external port for all destinations |
| ADM | Address Dependent | Different port per destination IP |
| APDM | Address+Port Dependent | Different port per destination IP:port |

### Filtering Types
| Type | Behavior | Description |
|------|----------|-------------|
| EIF | Endpoint Independent | Accept from any external host |
| ADF | Address Dependent | Only from IPs we've sent to |
| APDF | Address+Port Dependent | Only from exact IP:port we sent to |

## NAT Types in Docker

| NAT Type | Mapping | Filtering | Container |
|----------|---------|-----------|-----------|
| Full Cone | EIM | EIF | `nat-fullcone` |
| Address-Restricted | EIM | ADF | `nat-restricted` |
| Port-Restricted | EIM | APDF | `nat-portrestricted` |
| Symmetric | APDM | APDF | `nat-symmetric` |
| CGNAT | APDM | APDF | `nat-cgnat` |

## Connectivity Matrix

### Expected Results

| From \\ To | Public | FullCone | Restricted | PortRestricted | Symmetric | CGNAT |
|------------|--------|----------|------------|----------------|-----------|-------|
| **Public** | - | ✓ | ✓ | ✓ | ✓ | ✓ |
| **FullCone** | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| **Restricted** | ✓ | ✓ | ✓ | ✓ | ? | ? |
| **PortRestricted** | ✓ | ✓ | ✓ | ✓ | ? | ? |
| **Symmetric** | ✓ | ✓ | ? | ? | ✗ | ✗ |
| **CGNAT** | ✓ | ✓ | ? | ? | ✗ | ✗ |

Legend:
- ✓ Direct connectivity possible
- ? Requires coordination (hole-punching)
- ✗ Requires relay

## Test Scenarios

### Scenario 1: Easy Case (Full Cone)

```bash
# Both nodes behind full cone - should connect easily
docker-compose up -d nat-fullcone node-fullcone node-public

# Start bootstrap
docker exec -d communitas-node-public \
    /usr/local/bin/communitas-headless --listen 0.0.0.0:11000

# Connect from full cone
docker exec communitas-node-fullcone \
    /usr/local/bin/communitas-headless \
    --listen 0.0.0.0:11000 \
    --bootstrap 172.20.100.1:11000
```

### Scenario 2: Moderate (Port-Restricted)

```bash
# Typical home router scenario
docker-compose up -d nat-portrestricted node-portrestricted node-public

# Both need to send to each other for hole-punch
# Bootstrap coordinates this
```

### Scenario 3: Hard (Symmetric NAT)

```bash
# Enterprise/ISP NAT
docker-compose up -d nat-symmetric node-symmetric node-public

# Connection to public works
# Connection to another symmetric needs relay
```

### Scenario 4: Worst Case (Symmetric-to-Symmetric)

```bash
# Start two symmetric NAT environments
docker-compose up -d \
    nat-symmetric node-symmetric \
    nat-cgnat node-cgnat \
    node-public

# These cannot connect directly
# Must use node-public as relay
```

### Scenario 5: Double NAT

```bash
# Common in apartments/dorms
docker-compose up -d \
    nat-doublenat-outer nat-doublenat-inner node-doublenat \
    node-public

# Extremely restrictive - relay almost always needed
```

## Measuring Results

### Success Metrics
```bash
# Run test matrix
./test-nat-matrix.sh full

# Generate report
./test-nat-matrix.sh full > nat-results-$(date +%Y%m%d).txt
```

### Connection Time
```bash
# Time connection establishment
time docker exec communitas-node-symmetric \
    timeout 30 /usr/local/bin/communitas-headless \
    --listen 0.0.0.0:11000 \
    --bootstrap 172.20.100.1:11000 \
    --connect-only
```

### Packet Loss
```bash
# Test UDP packet loss through NAT
docker exec communitas-node-symmetric \
    ping -c 100 172.20.100.1 | tail -2
```

## Real-World Correlation

| Docker NAT | Real-World Equivalent |
|------------|----------------------|
| Full Cone | Gaming routers with UPnP |
| Address-Restricted | Older Linksys/Netgear |
| Port-Restricted | Most modern home routers |
| Symmetric | Enterprise firewalls, some ISPs |
| CGNAT | Mobile carriers, ISP IPv4 shortage |
| Double NAT | Apartment complexes, dorms |

## Troubleshooting Failed Connections

### Check NAT rules applied
```bash
docker exec communitas-nat-symmetric iptables -t nat -L -n -v
```

### Verify conntrack entries
```bash
docker exec communitas-nat-symmetric conntrack -L | grep 11000
```

### Trace packet flow
```bash
docker exec communitas-nat-symmetric tcpdump -i any udp port 11000
```
