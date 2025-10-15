# Monitoring and Observability

Comprehensive monitoring setup for Communitas infrastructure.

## Table of Contents

- [Overview](#overview)
- [Metrics](#metrics)
- [Logging](#logging)
- [Tracing](#tracing)
- [Alerting](#alerting)
- [Dashboards](#dashboards)
- [Performance Monitoring](#performance-monitoring)
- [Incident Response](#incident-response)

---

## Overview

### Monitoring Stack

**Components**:
- **Prometheus**: Metrics collection and storage
- **Grafana**: Visualization and dashboards
- **Loki**: Log aggregation
- **Tempo**: Distributed tracing
- **AlertManager**: Alert routing and notification

**Architecture**:
```
┌─────────────────┐
│  Communitas     │ ──metrics──> ┌────────────┐
│  Nodes          │               │ Prometheus │
└─────────────────┘               └──────┬─────┘
        │                                 │
      logs                           metrics
        │                                 │
        ▼                                 ▼
┌─────────────────┐               ┌────────────┐
│      Loki       │<──queries──── │  Grafana   │
└─────────────────┘               └────────────┘
                                         │
                                      alerts
                                         ▼
                                  ┌────────────┐
                                  │AlertManager│
                                  └────────────┘
```

---

## Metrics

### Prometheus Setup

**Installation** (Docker Compose):
```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'
      - '--web.enable-lifecycle'

volumes:
  prometheus-data:
```

**Configuration** (prometheus.yml):
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'communitas-nodes'
    static_configs:
      - targets:
          - 'node1:8081'  # Metrics port
          - 'node2:8081'
          - 'node3:8081'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        regex: '([^:]+):.*'
        replacement: '${1}'

  - job_name: 'communitas-bootstrap'
    static_configs:
      - targets: ['bootstrap:8081']

rule_files:
  - 'alerts.yml'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']
```

### Exposed Metrics

**Node Metrics**:
```rust
// src/metrics.rs
use prometheus::{IntGauge, IntCounter, Histogram, register_int_gauge, register_int_counter, register_histogram};

lazy_static! {
    // Network metrics
    pub static ref PEER_COUNT: IntGauge = register_int_gauge!(
        "communitas_peer_count",
        "Number of connected peers"
    ).unwrap();

    pub static ref MESSAGES_SENT: IntCounter = register_int_counter!(
        "communitas_messages_sent_total",
        "Total messages sent"
    ).unwrap();

    pub static ref MESSAGES_RECEIVED: IntCounter = register_int_counter!(
        "communitas_messages_received_total",
        "Total messages received"
    ).unwrap();

    pub static ref MESSAGE_LATENCY: Histogram = register_histogram!(
        "communitas_message_latency_seconds",
        "Message round-trip latency"
    ).unwrap();

    // Storage metrics
    pub static ref STORAGE_SIZE: IntGauge = register_int_gauge!(
        "communitas_storage_bytes",
        "Total storage used in bytes"
    ).unwrap();

    pub static ref VAULT_COUNT: IntGauge = register_int_gauge!(
        "communitas_vault_count",
        "Number of vaults"
    ).unwrap();

    // System metrics
    pub static ref CPU_USAGE: IntGauge = register_int_gauge!(
        "communitas_cpu_usage_percent",
        "CPU usage percentage"
    ).unwrap();

    pub static ref MEMORY_USAGE: IntGauge = register_int_gauge!(
        "communitas_memory_bytes",
        "Memory usage in bytes"
    ).unwrap();
}

// Update metrics periodically
pub async fn update_metrics(ctx: &CoreContext) {
    PEER_COUNT.set(ctx.peer_count() as i64);
    STORAGE_SIZE.set(ctx.storage_size() as i64);
    VAULT_COUNT.set(ctx.vault_count() as i64);

    // System metrics
    let sys = sysinfo::System::new_all();
    CPU_USAGE.set(sys.global_cpu_info().cpu_usage() as i64);
    MEMORY_USAGE.set(sys.used_memory() as i64);
}
```

**Metrics Endpoint**:
```rust
// Expose metrics on /metrics endpoint
use axum::{Router, routing::get};
use prometheus::{Encoder, TextEncoder};

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

pub fn create_metrics_router() -> Router {
    Router::new().route("/metrics", get(metrics_handler))
}
```

### Key Metrics

**Network Health**:
- `communitas_peer_count` - Active peer connections
- `communitas_messages_sent_total` - Outbound message count
- `communitas_messages_received_total` - Inbound message count
- `communitas_message_latency_seconds` - Message round-trip time

**Storage**:
- `communitas_storage_bytes` - Total storage used
- `communitas_vault_count` - Number of vaults
- `communitas_storage_operations_total` - Storage operation count

**System Resources**:
- `communitas_cpu_usage_percent` - CPU utilization
- `communitas_memory_bytes` - Memory consumption
- `communitas_disk_io_bytes` - Disk I/O

**Application**:
- `communitas_auth_attempts_total` - Authentication attempts
- `communitas_auth_failures_total` - Failed authentications
- `communitas_active_sessions` - Current active sessions

---

## Logging

### Loki Setup

**Installation** (Docker Compose):
```yaml
loki:
  image: grafana/loki:latest
  container_name: loki
  restart: unless-stopped
  ports:
    - "3100:3100"
  volumes:
    - ./loki-config.yml:/etc/loki/loki-config.yml:ro
    - loki-data:/loki
  command: -config.file=/etc/loki/loki-config.yml
```

**Configuration** (loki-config.yml):
```yaml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
  chunk_idle_period: 5m
  chunk_retain_period: 30s

schema_config:
  configs:
    - from: 2024-01-01
      store: boltdb
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb:
    directory: /loki/index
  filesystem:
    directory: /loki/chunks

limits_config:
  enforce_metric_name: false
  reject_old_samples: true
  reject_old_samples_max_age: 168h

chunk_store_config:
  max_look_back_period: 0s

table_manager:
  retention_deletes_enabled: true
  retention_period: 720h  # 30 days
```

### Structured Logging

**Rust Logging**:
```rust
use tracing::{info, warn, error, debug, instrument};
use tracing_subscriber::{fmt, EnvFilter};

// Initialize logging
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json()  // Structured JSON format
        .init();
}

// Instrumented function
#[instrument(skip(vault))]
pub async fn login(four_words: &str, password: &str, vault: &Vault) -> Result<Session> {
    info!(four_words = %four_words, "Login attempt started");

    match vault.verify_password(password) {
        Ok(session) => {
            info!(
                session_id = %session.id,
                four_words = %four_words,
                "Login successful"
            );
            Ok(session)
        }
        Err(e) => {
            warn!(
                four_words = %four_words,
                error = %e,
                "Login failed"
            );
            Err(e)
        }
    }
}
```

**Log Aggregation** (Promtail):
```yaml
# promtail-config.yml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: communitas
    static_configs:
      - targets:
          - localhost
        labels:
          job: communitas
          __path__: /var/log/communitas/*.log
    pipeline_stages:
      - json:
          expressions:
            timestamp: timestamp
            level: level
            message: message
            target: target
      - labels:
          level:
          target:
      - timestamp:
          source: timestamp
          format: RFC3339
```

### Log Levels

**Production**:
```bash
# Minimal logging for performance
export RUST_LOG=info
```

**Debugging**:
```bash
# Detailed logging
export RUST_LOG=debug,communitas_core=trace
```

**Performance Profiling**:
```bash
# Minimal logging to reduce overhead
export RUST_LOG=warn
```

---

## Tracing

### Distributed Tracing

**Tempo Setup**:
```yaml
tempo:
  image: grafana/tempo:latest
  container_name: tempo
  restart: unless-stopped
  ports:
    - "3200:3200"   # Tempo HTTP
    - "4317:4317"   # OTLP gRPC
  volumes:
    - ./tempo-config.yml:/etc/tempo/tempo-config.yml:ro
    - tempo-data:/var/tempo
  command: -config.file=/etc/tempo/tempo-config.yml
```

**OpenTelemetry Integration**:
```rust
use opentelemetry::{global, trace::Tracer};
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, Registry};

pub fn init_tracing() -> Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://tempo:4317")
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "communitas-node"),
                ]))
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry = OpenTelemetryLayer::new(tracer);
    let subscriber = Registry::default().with(telemetry);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

// Traced function
#[instrument]
async fn send_message(channel_id: &str, content: &str) -> Result<Message> {
    let span = tracing::info_span!("send_message", channel_id);
    let _enter = span.enter();

    // ... implementation ...
}
```

---

## Alerting

### Alert Rules

**Prometheus Alert Rules** (alerts.yml):
```yaml
groups:
  - name: communitas_alerts
    interval: 30s
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: rate(communitas_errors_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/sec on {{ $labels.instance }}"

      # Node down
      - alert: NodeDown
        expr: up{job="communitas-nodes"} == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Node is down"
          description: "{{ $labels.instance }} has been down for more than 2 minutes"

      # Low peer count
      - alert: LowPeerCount
        expr: communitas_peer_count < 2
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Low peer count"
          description: "Node {{ $labels.instance }} has only {{ $value }} peers"

      # High CPU usage
      - alert: HighCPUUsage
        expr: communitas_cpu_usage_percent > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage"
          description: "CPU usage is {{ $value }}% on {{ $labels.instance }}"

      # High memory usage
      - alert: HighMemoryUsage
        expr: communitas_memory_bytes > 1000000000  # 1 GB
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value | humanize }}B on {{ $labels.instance }}"

      # Disk space low
      - alert: DiskSpaceLow
        expr: (node_filesystem_avail_bytes{mountpoint="/var/lib/communitas"} / node_filesystem_size_bytes{mountpoint="/var/lib/communitas"}) < 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Disk space low"
          description: "Disk space below 10% on {{ $labels.instance }}"
```

### AlertManager Configuration

**alertmanager.yml**:
```yaml
global:
  resolve_timeout: 5m

route:
  group_by: ['alertname', 'cluster']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'default'
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty'
      continue: true
    - match:
        severity: warning
      receiver: 'slack'

receivers:
  - name: 'default'
    email_configs:
      - to: 'ops@communitas.life'
        from: 'alerts@communitas.life'
        smarthost: 'smtp.gmail.com:587'
        auth_username: 'alerts@communitas.life'
        auth_password: 'app-password'

  - name: 'slack'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
        channel: '#alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: 'YOUR-PAGERDUTY-KEY'

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'instance']
```

---

## Dashboards

### Grafana Setup

**Installation**:
```yaml
grafana:
  image: grafana/grafana:latest
  container_name: grafana
  restart: unless-stopped
  ports:
    - "3000:3000"
  volumes:
    - grafana-data:/var/lib/grafana
    - ./grafana-provisioning:/etc/grafana/provisioning
  environment:
    - GF_SECURITY_ADMIN_USER=admin
    - GF_SECURITY_ADMIN_PASSWORD=admin_password
    - GF_INSTALL_PLUGINS=grafana-clock-panel
```

### Dashboard Panels

**Network Overview Dashboard**:
```json
{
  "dashboard": {
    "title": "Communitas Network Overview",
    "panels": [
      {
        "title": "Peer Count",
        "targets": [
          {
            "expr": "communitas_peer_count",
            "legendFormat": "{{instance}}"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Message Throughput",
        "targets": [
          {
            "expr": "rate(communitas_messages_sent_total[5m])",
            "legendFormat": "Sent - {{instance}}"
          },
          {
            "expr": "rate(communitas_messages_received_total[5m])",
            "legendFormat": "Received - {{instance}}"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Message Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, communitas_message_latency_seconds)",
            "legendFormat": "p95 - {{instance}}"
          },
          {
            "expr": "histogram_quantile(0.99, communitas_message_latency_seconds)",
            "legendFormat": "p99 - {{instance}}"
          }
        ],
        "type": "graph"
      }
    ]
  }
}
```

**System Resources Dashboard**:
```json
{
  "dashboard": {
    "title": "Communitas System Resources",
    "panels": [
      {
        "title": "CPU Usage",
        "targets": [
          {
            "expr": "communitas_cpu_usage_percent",
            "legendFormat": "{{instance}}"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Memory Usage",
        "targets": [
          {
            "expr": "communitas_memory_bytes / 1024 / 1024",
            "legendFormat": "{{instance}}"
          }
        ],
        "type": "graph",
        "yAxes": [
          {
            "format": "megabytes"
          }
        ]
      },
      {
        "title": "Storage Usage",
        "targets": [
          {
            "expr": "communitas_storage_bytes / 1024 / 1024 / 1024",
            "legendFormat": "{{instance}}"
          }
        ],
        "type": "graph",
        "yAxes": [
          {
            "format": "gigabytes"
          }
        ]
      }
    ]
  }
}
```

---

## Performance Monitoring

### Application Performance

**Key Performance Indicators**:
- **Message Latency**: p50, p95, p99 latencies
- **Throughput**: Messages per second
- **Storage Operations**: Read/write latency
- **Authentication**: Login time, session creation time

**Query Examples**:
```promql
# Average message latency (p50)
histogram_quantile(0.50, communitas_message_latency_seconds)

# 99th percentile latency
histogram_quantile(0.99, communitas_message_latency_seconds)

# Message throughput (messages/sec)
rate(communitas_messages_sent_total[5m])

# Error rate
rate(communitas_errors_total[5m])

# Success rate
rate(communitas_operations_total{status="success"}[5m]) /
rate(communitas_operations_total[5m])
```

### Infrastructure Monitoring

**Node Exporter** (system metrics):
```yaml
node-exporter:
  image: prom/node-exporter:latest
  container_name: node-exporter
  restart: unless-stopped
  ports:
    - "9100:9100"
  command:
    - '--path.procfs=/host/proc'
    - '--path.sysfs=/host/sys'
    - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
  volumes:
    - /proc:/host/proc:ro
    - /sys:/host/sys:ro
    - /:/rootfs:ro
```

---

## Incident Response

### Incident Response Playbook

**1. Detection**:
- Alert received via PagerDuty/Slack
- Monitoring dashboard shows anomaly
- User reports issue

**2. Triage**:
```bash
# Check service status
systemctl status communitas-bootstrap

# Check recent logs
journalctl -u communitas-bootstrap -n 100

# Check metrics
curl http://localhost:8081/metrics | grep communitas_peer_count

# Check network connectivity
nc -zu localhost 8080
```

**3. Diagnosis**:
```bash
# Enable debug logging temporarily
systemctl set-environment RUST_LOG=debug
systemctl restart communitas-bootstrap

# Monitor logs in real-time
journalctl -u communitas-bootstrap -f

# Check for errors
journalctl -u communitas-bootstrap | grep ERROR

# Check resource usage
top -p $(pgrep communitas)
```

**4. Resolution**:
```bash
# Restart service if needed
systemctl restart communitas-bootstrap

# Rollback if recent deployment
./rollback.sh

# Restore from backup if data corruption
./restore.sh /backups/communitas/latest
```

**5. Post-Mortem**:
- Document incident timeline
- Root cause analysis
- Preventive measures
- Update runbooks

---

## See Also

- [Operations Guide](README.md) - Complete operations guide
- [Architecture](../architecture/README.md) - System architecture
- [Security](../architecture/security.md) - Security architecture

---

**Monitoring Guide**: Observe, measure, and improve. 📊🔍
