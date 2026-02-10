# Campaign Express — Deployment Guide

## Table of Contents
1. [Architecture Overview](#architecture-overview)
2. [Prerequisites](#prerequisites)
3. [Local Development (Docker Compose)](#local-development)
4. [Kubernetes Deployment](#kubernetes-deployment)
5. [Infrastructure Components](#infrastructure-components)
6. [Production Configuration](#production-configuration)
7. [Monitoring & Observability](#monitoring--observability)
8. [Scaling](#scaling)
9. [Operations](#operations)

---

## Architecture Overview

Campaign Express deploys as a 20-node Kubernetes cluster with the following topology:

```
                        ┌──────────────────────┐
                        │   HAProxy Ingress     │
                        │   (3 replicas, LB)    │
                        │   :80 / :443 / :9090  │
                        └──────────┬───────────┘
                                   │
              ┌────────────────────┼────────────────────┐
              │                    │                    │
     ┌────────▼────────┐ ┌────────▼────────┐  ┌────────▼────────┐
     │ Campaign Express │ │ Campaign Express │  │ Campaign Express │
     │   Pod 1..20      │ │   Pod 2          │  │   Pod N          │
     │   :8080 REST     │ │   :8080 REST     │  │   :8080 REST     │
     │   :9090 gRPC     │ │   :9090 gRPC     │  │   :9090 gRPC     │
     │   :9091 metrics  │ │   :9091 metrics  │  │   :9091 metrics  │
     │   20 agents each │ │   20 agents each │  │   20 agents each │
     └──────┬───────────┘ └──────┬───────────┘  └──────┬───────────┘
            │                    │                      │
   ┌────────┴────────────────────┴──────────────────────┴─────────┐
   │                    Shared Infrastructure                      │
   │  ┌─────────┐  ┌──────────────┐  ┌───────────┐  ┌──────────┐ │
   │  │ NATS    │  │ Redis Cluster│  │ClickHouse │  │Prometheus│ │
   │  │ 3 nodes │  │ 6 nodes      │  │ 2 nodes   │  │+ Grafana │ │
   │  │JetStream│  │ (3M + 3R)    │  │           │  │          │ │
   │  └─────────┘  └──────────────┘  └───────────┘  └──────────┘ │
   └──────────────────────────────────────────────────────────────┘
```

## Prerequisites

| Component           | Version  | Purpose                           |
|---------------------|----------|-----------------------------------|
| Kubernetes          | 1.28+    | Container orchestration           |
| kubectl             | 1.28+    | Cluster CLI                       |
| kustomize           | 5.0+     | K8s manifest management           |
| Docker              | 24+      | Container builds                  |
| Rust                | 1.77+    | Build toolchain (for local dev)   |
| Helm (optional)     | 3.14+    | Alternative package management    |

**Hardware requirements (production):**
- 20 nodes with AMD Ryzen AI XDNA NPUs
- 8 CPU / 16 Gi RAM per application pod
- 200 Gi SSD per ClickHouse node
- 50 Gi SSD per Redis node

---

## Local Development

### Docker Compose (full stack)

```bash
# Start all services
docker compose -f deploy/docker/docker-compose.yml up -d

# Verify
curl http://localhost:8080/health
curl http://localhost:8080/ready

# View logs
docker compose -f deploy/docker/docker-compose.yml logs -f campaign-express

# Stop
docker compose -f deploy/docker/docker-compose.yml down
```

**Services started:**

| Service           | Port(s)          | Purpose                        |
|-------------------|------------------|--------------------------------|
| campaign-express  | 8080, 9090, 9091 | Application (REST, gRPC, metrics) |
| nats              | 4222, 8222       | Message broker + monitoring    |
| redis             | 6379             | Distributed cache              |
| clickhouse        | 8123, 9000       | Analytics DB                   |
| prometheus        | 9092             | Metrics collection             |
| grafana           | 3000             | Dashboards (admin/campaign-express) |

### Dev Mode with Hot Reload

```bash
docker build -f deploy/docker/Dockerfile.dev -t campaign-express-dev .
docker run -p 8080:8080 -p 9090:9090 -v $(pwd):/app campaign-express-dev
```

Uses `cargo-watch` to rebuild on file changes.

---

## Kubernetes Deployment

### 1. Create Namespace

```bash
kubectl apply -f deploy/k8s/base/namespace.yaml
```

### 2. Deploy Infrastructure First

```bash
# NATS JetStream cluster (3 nodes)
kubectl apply -f deploy/nats/nats-deployment.yaml

# Redis cluster (6 nodes: 3 master + 3 replica)
kubectl apply -f deploy/redis/redis-deployment.yaml

# Wait for Redis pods, then initialize cluster
kubectl wait --for=condition=Ready pod -l app.kubernetes.io/name=redis-cluster \
  -n campaign-express --timeout=120s
# The redis-cluster-init Job runs automatically

# ClickHouse analytics (2 nodes)
kubectl apply -f deploy/clickhouse/clickhouse-deployment.yaml
```

### 3. Deploy Application

**Staging:**
```bash
kustomize build deploy/k8s/overlays/staging | kubectl apply -f -
```

**Production:**
```bash
kustomize build deploy/k8s/overlays/production | kubectl apply -f -
```

### 4. Deploy Ingress

```bash
kubectl apply -f deploy/haproxy/haproxy-deployment.yaml
```

### 5. Deploy Monitoring

```bash
kubectl apply -f deploy/monitoring/prometheus/prometheus-deployment.yaml
kubectl apply -f deploy/monitoring/grafana/grafana-deployment.yaml
```

### 6. Verify Deployment

```bash
# Check all pods
kubectl get pods -n campaign-express

# Check endpoints
kubectl get svc -n campaign-express

# Test health
kubectl port-forward svc/campaign-express 8080:8080 -n campaign-express
curl http://localhost:8080/health
```

---

## Infrastructure Components

### NATS JetStream (Messaging)

| Property       | Value                                          |
|----------------|------------------------------------------------|
| Replicas       | 3 (StatefulSet)                                |
| Ports          | 4222 (client), 6222 (cluster), 8222 (monitor)  |
| Storage        | 20 Gi per node (JetStream file store)           |
| Memory store   | 4 Gi per node                                   |
| Cluster name   | campaign-nats                                   |
| Discovery      | Headless service DNS (nats-0.nats-headless...)  |

Streams used:
- `campaign-bids` — bid request/response routing
- `ingest.*` — omnichannel event ingestion (mobile, POS, kiosk, web)

### Redis Cluster (Cache)

| Property       | Value                                          |
|----------------|------------------------------------------------|
| Replicas       | 6 (3 master + 3 replica)                       |
| Max memory     | 8 Gb per node (allkeys-lru eviction)            |
| Storage        | 50 Gi per node (AOF persistence)                |
| Timeout        | 5000 ms cluster node timeout                    |
| Snapshots      | Every 60s if 10K+ keys changed                  |

Key patterns:
- `profile:{user_id}` — UserProfile JSON (TTL: 3600s)

### ClickHouse (Analytics)

| Property       | Value                                          |
|----------------|------------------------------------------------|
| Replicas       | 2                                              |
| Storage        | 200 Gi per node                                 |
| Max memory     | 10 Gb per query                                 |
| Threads        | 8 per query                                     |
| Retention      | 90 days (TTL on timestamp)                      |
| Partitioning   | Monthly (toYYYYMM)                              |

Tables:
- `analytics_events` — all bid, loyalty, DSP, and channel events

### HAProxy (Ingress)

| Property       | Value                                          |
|----------------|------------------------------------------------|
| Replicas       | 3                                              |
| Max connections| 100,000                                         |
| Threads        | 4                                               |
| Rate limit     | 10,000 req/10s per IP → 429                     |
| Balance (HTTP) | leastconn                                       |
| Balance (gRPC) | roundrobin                                      |
| Health check   | GET /ready, expect 200                          |

Frontend routing:
- `/v1/bid*` → campaign-express backend (HTTP)
- `:9090` → gRPC backend (H2)
- `/health`, `/ready`, `/live` → health backend

---

## Production Configuration

### Environment Overlay Differences

| Setting            | Staging            | Production          |
|--------------------|--------------------|---------------------|
| Replicas           | 3                  | 20                  |
| Agents per node    | 4                  | 20                  |
| NPU device         | cpu                | xdna                |
| CPU request/limit  | 1/2                | 4/8                 |
| Memory request/limit| 2Gi/4Gi           | 8Gi/16Gi            |
| Log level          | debug              | info                |
| Node selector      | —                  | `npu: xdna`         |

### Key Environment Variables

```bash
CAMPAIGN_EXPRESS__NODE_ID          # Unique node identifier
CAMPAIGN_EXPRESS__AGENTS_PER_NODE  # Tokio agent count (default: 20)
CAMPAIGN_EXPRESS__API__HTTP_PORT   # REST port (default: 8080)
CAMPAIGN_EXPRESS__API__GRPC_PORT   # gRPC port (default: 9090)
CAMPAIGN_EXPRESS__NPU__DEVICE      # "cpu" or "xdna"
CAMPAIGN_EXPRESS__NPU__MODEL_PATH  # Path to ONNX model file
CAMPAIGN_EXPRESS__LOYALTY__ENABLED # Enable loyalty engine (default: true)
CAMPAIGN_EXPRESS__DSP__ENABLED     # Enable DSP routing (default: false)
RUST_LOG                           # Tracing filter (e.g., campaign_express=info)
```

---

## Monitoring & Observability

### Prometheus Scrape Targets

| Target           | Discovery     | Endpoint                        |
|------------------|---------------|---------------------------------|
| campaign-express | Pod annotation| `:9091/metrics`                 |
| nats             | Static        | `:8222`                         |
| redis            | Static        | `:9121` (exporter)              |
| clickhouse       | Static        | `:9363/metrics`                 |
| haproxy          | Static        | `:8404`                         |

### Grafana Dashboard Panels

The pre-built dashboard (`campaign-express-main`) includes:

1. **Bid Requests/Second** — per-pod throughput
2. **Bid Response Latency** — p50/p95/p99
3. **NPU Inference Latency** — p50/p99 in microseconds
4. **Cache Hit Rate** — L1 and L1+L2 combined
5. **Active Pods** — count of healthy instances
6. **Total Throughput** — offers/hour (target: 50M)
7. **No-Bid Rate** — frequency-capped and no-winner ratio
8. **Error Rate** — API errors / total requests
9. **Analytics Pipeline** — queued vs flushed vs dropped events

Access Grafana: `http://<grafana-svc>:3000` (admin / campaign-express)

### Key Alerts to Configure

```
# Throughput below target
rate(bids_requests_total[5m]) * 3600 < 50000000 / <node_count>

# P99 latency above SLA
histogram_quantile(0.99, rate(bids_total_latency_us_bucket[5m])) > 10000

# Analytics pipeline dropping events
rate(analytics_dropped_total[1m]) > 0

# Cache hit rate degradation
(sum(l1_hit) + sum(l2_hit)) / (sum(l1_hit) + sum(l1_miss)) < 0.9
```

---

## Scaling

### Horizontal Pod Autoscaler

The HPA scales between **10–40 pods** based on three signals:

| Metric                      | Target | Type     |
|-----------------------------|--------|----------|
| CPU utilization             | 70%    | Resource |
| Memory utilization          | 80%    | Resource |
| bids_requests_per_second    | 700    | Custom   |

**Scale-up:** +4 pods per 60s (stabilization: 60s)
**Scale-down:** -2 pods per 120s (stabilization: 300s / 5min cool-down)

### Pod Disruption Budget

Minimum 80% of pods must remain available during voluntary disruptions (node drain, rolling updates).

### Capacity Planning

```
Target: 50M offers/hour
Per pod: ~700 bids/sec × 3600 = 2.52M offers/hour
Min pods: 50M / 2.52M ≈ 20 pods
```

---

## Operations

### Rolling Update

```bash
# Build and push new image
docker build -f deploy/docker/Dockerfile -t campaign-express:v1.2.0 .
docker push <registry>/campaign-express:v1.2.0

# Update image in deployment
kubectl set image deployment/campaign-express \
  campaign-express=<registry>/campaign-express:v1.2.0 \
  -n campaign-express

# Monitor rollout
kubectl rollout status deployment/campaign-express -n campaign-express
```

The deployment uses `maxUnavailable: 2, maxSurge: 3` for zero-downtime updates.

### Model Hot-Reload

The NPU engine supports hot-reloading without pod restart. Upload a new model to the shared PVC:

```bash
kubectl cp colanet-v2.onnx campaign-express/<pod>:/models/colanet-v2.onnx
# Then trigger reload via admin API or NATS ModelUpdate message
```

### Disaster Recovery

| Component    | Persistence              | Backup Strategy                  |
|-------------|--------------------------|----------------------------------|
| Redis       | AOF + RDB snapshots      | Periodic RDB export to S3        |
| ClickHouse  | 200Gi PVC per node       | Replicated tables + S3 backups   |
| NATS        | JetStream file store     | Ephemeral (replay from sources)  |
| Models      | ReadOnlyMany PVC (10Gi)  | Stored in model registry/S3      |

### Useful Commands

```bash
# Check pod resource usage
kubectl top pods -n campaign-express

# View HPA status
kubectl get hpa campaign-express -n campaign-express

# View recent events
kubectl get events -n campaign-express --sort-by='.lastTimestamp'

# HAProxy stats
kubectl port-forward svc/haproxy-ingress 8404:8404 -n campaign-express
# Then open http://localhost:8404/stats

# ClickHouse query
kubectl exec -it clickhouse-0 -n campaign-express -- \
  clickhouse-client -q "SELECT event_type, count() FROM analytics_events GROUP BY event_type"
```
