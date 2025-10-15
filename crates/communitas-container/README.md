# Communitas Container

Docker/OCI container utilities for Communitas deployment.

## Overview

Communitas Container provides containerized deployments and orchestration support for running Communitas in production environments. It includes:

- Multi-architecture Docker images
- Kubernetes operators and manifests
- Docker Compose templates
- Health checks and monitoring integration
- Automated scaling and recovery

## Features

- **Multi-Architecture Images**: amd64 (x86_64) and arm64 (aarch64) support
- **Kubernetes Native**: Operators, CRDs, and Helm charts
- **Docker Compose**: Quick local deployment
- **Health Checks**: Integrated liveness and readiness probes
- **Monitoring**: Prometheus metrics and Grafana dashboards
- **Auto-Scaling**: Horizontal Pod Autoscaler (HPA) support
- **Security**: Runs as non-root, minimal attack surface

## Quick Start

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  communitas:
    image: communitas/node:latest
    container_name: communitas-node
    ports:
      - "8080:8080"      # P2P
      - "9090:9090"      # API
      - "3000:3000"      # Metrics
    volumes:
      - ./data:/data
      - ./config:/config
    environment:
      - COMMUNITAS_IDENTITY=ocean-forest-moon-star
      - COMMUNITAS_DISPLAY_NAME=Docker Node
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9090/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana-dashboards:/etc/grafana/provisioning/dashboards
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin

volumes:
  prometheus-data:
  grafana-data:
```

Start the stack:
```bash
docker-compose up -d
```

### Single Container

```bash
# Pull the image
docker pull communitas/node:latest

# Run with basic configuration
docker run -d \
  --name communitas \
  -p 8080:8080 \
  -p 9090:9090 \
  -v communitas-data:/data \
  -e COMMUNITAS_IDENTITY=ocean-forest-moon-star \
  communitas/node:latest
```

## Kubernetes Deployment

### Using Helm

```bash
# Add the Communitas Helm repository
helm repo add communitas https://charts.communitas.network
helm repo update

# Install with default values
helm install my-communitas communitas/communitas

# Or with custom values
helm install my-communitas communitas/communitas \
  --set image.tag=v0.1.17 \
  --set replicas=3 \
  --set storage.size=10Gi
```

### Using Kubectl

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: communitas-node
  labels:
    app: communitas
spec:
  replicas: 3
  selector:
    matchLabels:
      app: communitas
  template:
    metadata:
      labels:
        app: communitas
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
      - name: communitas
        image: communitas/node:v0.1.17
        ports:
        - name: p2p
          containerPort: 8080
          protocol: UDP
        - name: api
          containerPort: 9090
          protocol: TCP
        - name: metrics
          containerPort: 3000
          protocol: TCP
        env:
        - name: COMMUNITAS_IDENTITY
          valueFrom:
            secretKeyRef:
              name: communitas-identity
              key: four-words
        - name: RUST_LOG
          value: "info,communitas=debug"
        volumeMounts:
        - name: data
          mountPath: /data
        - name: config
          mountPath: /config
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 9090
          initialDelaySeconds: 10
          periodSeconds: 5
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: communitas-data
      - name: config
        configMap:
          name: communitas-config

---
apiVersion: v1
kind: Service
metadata:
  name: communitas-service
  labels:
    app: communitas
spec:
  type: LoadBalancer
  selector:
    app: communitas
  ports:
  - name: p2p
    port: 8080
    targetPort: 8080
    protocol: UDP
  - name: api
    port: 9090
    targetPort: 9090
    protocol: TCP
  - name: metrics
    port: 3000
    targetPort: 3000
    protocol: TCP

---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: communitas-data
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
  storageClassName: fast-ssd
```

Apply:
```bash
kubectl apply -f deployment.yaml
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `COMMUNITAS_IDENTITY` | Four-word identity | Required |
| `COMMUNITAS_DISPLAY_NAME` | Display name | Container hostname |
| `COMMUNITAS_DEVICE_NAME` | Device name | Container ID |
| `COMMUNITAS_DATA_DIR` | Data directory | `/data` |
| `COMMUNITAS_CONFIG_FILE` | Config file path | `/config/communitas.toml` |
| `COMMUNITAS_LOG_LEVEL` | Rust log level | `info` |
| `COMMUNITAS_API_PORT` | API port | `9090` |
| `COMMUNITAS_P2P_PORT` | P2P port | `8080` |
| `COMMUNITAS_METRICS_PORT` | Metrics port | `3000` |
| `COMMUNITAS_BOOTSTRAP_NODES` | Bootstrap nodes (comma-separated) | Default nodes |

### Configuration File

Mount a `communitas.toml` configuration file:

```toml
[identity]
four_words = "ocean-forest-moon-star"
display_name = "Container Node"
device_name = "k8s-pod-1"

[network]
listen_addr = "0.0.0.0:8080"
api_addr = "0.0.0.0:9090"
bootstrap_nodes = [
    "bootstrap.communitas.network:8080"
]

[storage]
path = "/data"
cache_size_mb = 500

[logging]
level = "info"
format = "json"

[metrics]
enabled = true
port = 3000
```

## Building Custom Images

### Dockerfile

```dockerfile
FROM rust:1.85-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY . .

# Build release binary
RUN cargo build --release -p communitas-headless

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 communitas

# Copy binary
COPY --from=builder /app/target/release/communitas-headless /usr/local/bin/

# Set up directories
RUN mkdir -p /data /config && \
    chown -R communitas:communitas /data /config

USER communitas
WORKDIR /home/communitas

EXPOSE 8080/udp 9090/tcp 3000/tcp

HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
  CMD curl -f http://localhost:9090/health || exit 1

ENTRYPOINT ["communitas-headless"]
CMD ["--config", "/config/communitas.toml"]
```

Build:
```bash
docker build -t communitas/node:custom .
```

### Multi-Architecture Build

```bash
# Set up buildx
docker buildx create --use

# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t communitas/node:v0.1.17 \
  --push \
  .
```

## Monitoring

### Prometheus Configuration

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'communitas'
    static_configs:
      - targets: ['communitas:3000']
    scrape_interval: 15s
```

### Grafana Dashboard

Import the pre-built dashboard:
```bash
kubectl apply -f https://raw.githubusercontent.com/saorsalabs/communitas/main/deployments/kubernetes/grafana-dashboard.json
```

Key metrics:
- Node uptime
- Peer connections
- Message throughput
- Storage usage
- CPU/Memory usage

## Auto-Scaling

### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: communitas-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: communitas-node
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 30
```

## Security Best Practices

1. **Run as Non-Root**: Images run as user `1000` by default
2. **Minimal Base Image**: Uses Debian Slim for reduced attack surface
3. **Read-Only Root Filesystem**: Enable with `readOnlyRootFilesystem: true`
4. **Network Policies**: Restrict ingress/egress traffic
5. **Secret Management**: Use Kubernetes Secrets or external secret managers
6. **Regular Updates**: Keep base images updated
7. **Image Scanning**: Use tools like Trivy or Snyk

### Example Network Policy

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: communitas-network-policy
spec:
  podSelector:
    matchLabels:
      app: communitas
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          role: monitoring
    ports:
    - protocol: TCP
      port: 3000
  egress:
  - to:
    - namespaceSelector: {}
    ports:
    - protocol: UDP
      port: 8080
```

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs communitas

# Or in Kubernetes
kubectl logs -f deployment/communitas-node
```

### Permission Errors

Ensure volumes are writable:
```bash
docker run --rm -v communitas-data:/data alpine chown -R 1000:1000 /data
```

### Network Issues

Test connectivity:
```bash
docker exec communitas curl -f http://localhost:9090/health
```

### Resource Constraints

Check resource usage:
```bash
# Docker
docker stats communitas

# Kubernetes
kubectl top pod -l app=communitas
```

## Performance Tuning

### Resource Recommendations

| Deployment Size | CPU | Memory | Storage |
|----------------|-----|---------|---------|
| Small (1-10 users) | 500m | 512Mi | 5Gi |
| Medium (10-100 users) | 1000m | 2Gi | 20Gi |
| Large (100-1000 users) | 2000m | 4Gi | 50Gi |
| Enterprise (1000+ users) | 4000m | 8Gi | 100Gi+ |

### Optimization Tips

- Use SSD storage for better performance
- Enable CPU pinning for consistent performance
- Configure proper resource limits to prevent OOM
- Use local volumes for data-heavy workloads

## Development

### Local Testing

```bash
# Build locally
docker build -t communitas/node:dev .

# Run with volume mounts for development
docker run -it --rm \
  -v $(pwd)/data:/data \
  -v $(pwd)/config:/config \
  -p 8080:8080 \
  -p 9090:9090 \
  communitas/node:dev
```

### Debugging

```bash
# Interactive shell
docker exec -it communitas sh

# Check processes
docker exec communitas ps aux

# Network debugging
docker exec communitas netstat -tuln
```

## Contributing

See [../../docs/development/contributing.md](../../docs/development/contributing.md)

## License

Dual-licensed under AGPL-3.0-or-later and commercial license.

## See Also

- [Communitas Headless](../../communitas-headless/README.md) - The daemon running in containers
- [Operations Guide](../../docs/operations/README.md) - Production deployment guide
- [Monitoring Guide](../../docs/operations/monitoring.md) - Detailed monitoring setup
- [Kubernetes Examples](../../deployments/kubernetes/) - Complete K8s manifests
