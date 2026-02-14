# Campaign Express — Infrastructure Reference

Comprehensive reference for all infrastructure components, deployment manifests, and configuration files.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Kubernetes (deploy/k8s/)](#2-kubernetes)
3. [Terraform (deploy/terraform/)](#3-terraform)
4. [Helm (deploy/helm/)](#4-helm)
5. [Docker (deploy/docker/)](#5-docker)
6. [Monitoring (deploy/monitoring/)](#6-monitoring)
7. [HAProxy (deploy/haproxy/)](#7-haproxy)
8. [NATS (deploy/nats/)](#8-nats)
9. [Redis (deploy/redis/)](#9-redis)
10. [ClickHouse (deploy/clickhouse/)](#10-clickhouse)
11. [AWS Alternative (deploy/aws/)](#11-aws-alternative)

---

## 1. Overview

Campaign Express deploys as a 20-node Kubernetes cluster with the following infrastructure:

```
┌─────────────────────────────────────────────────────────┐
│                    HAProxy Ingress                       │
│              (3 replicas, rate limiting)                 │
│           :80 / :443 (REST) / :9090 (gRPC)             │
└────────────────────────┬────────────────────────────────┘
                         │
    ┌────────────────────┼────────────────────┐
    │                    │                    │
    ▼                    ▼                    ▼
┌────────────┐   ┌────────────┐      ┌────────────┐
│ Pod 1..20  │   │ Pod 2      │ ...  │ Pod N      │
│ 20 agents  │   │ 20 agents  │      │ 20 agents  │
│ REST:8080  │   │ REST:8080  │      │ REST:8080  │
│ gRPC:9090  │   │ gRPC:9090  │      │ gRPC:9090  │
│ Metrics    │   │ Metrics    │      │ Metrics    │
│ :9091      │   │ :9091      │      │ :9091      │
└─────┬──────┘   └─────┬──────┘      └─────┬──────┘
      │                │                    │
┌─────┴────────────────┴────────────────────┴────────┐
│                Shared Infrastructure                │
│  ┌──────┐  ┌───────────┐  ┌──────────┐  ┌───────┐ │
│  │ NATS │  │   Redis   │  │ClickHouse│  │Monitor│ │
│  │3-node│  │ 6-node    │  │  2-node  │  │ Stack │ │
│  │JS    │  │ cluster   │  │          │  │       │ │
│  └──────┘  └───────────┘  └──────────┘  └───────┘ │
└────────────────────────────────────────────────────┘
```

### File Organization

```
deploy/
├── k8s/                    # Kubernetes manifests (Kustomize)
│   ├── base/               #   Base resources
│   └── overlays/           #   Environment-specific patches
│       ├── production/
│       └── staging/
├── terraform/
│   └── azure/              # Azure IaC (AKS, Redis, ACR, Key Vault)
├── helm/
│   └── campaign-express/   # Helm chart
├── docker/                 # Dockerfiles + docker-compose
├── monitoring/             # Observability stack
│   ├── prometheus/
│   ├── alertmanager/
│   ├── grafana/
│   └── logging/            # Loki + Promtail
├── haproxy/                # Load balancer
├── nats/                   # Message broker
├── redis/                  # Cache cluster
├── clickhouse/             # Analytics database
└── aws/                    # AWS alternative deployment
```

---

## 2. Kubernetes

### Base Resources (`deploy/k8s/base/`)

| File | Resource | Description |
|------|----------|-------------|
| `namespace.yaml` | Namespace | `campaign-express` namespace with organizational labels |
| `configmap.yaml` | ConfigMap | 20+ environment variables (NATS, Redis, ClickHouse URLs, NPU settings, logging) |
| `deployment.yaml` | Deployment | 20 replicas, 2-4 CPU / 4-8Gi memory, AMD XDNA NPU resource, pod anti-affinity, readiness/liveness/startup probes |
| `service.yaml` | Service, Headless Service | ClusterIP service (8080, 9090, 9091) + headless service for gRPC load balancing |
| `hpa.yaml` | HorizontalPodAutoscaler | 10-40 replicas, CPU 70% / Memory 80% targets, custom bid metrics |
| `pdb.yaml` | PodDisruptionBudget | Minimum 80% availability during disruptions |
| `pvc.yaml` | PersistentVolumeClaim | 10Gi ReadOnlyMany for ML model storage |
| `cert-manager.yaml` | ClusterIssuer | Let's Encrypt production + staging issuers for TLS |
| `external-secrets.yaml` | SecretStore, ExternalSecret | Azure Key Vault integration for Redis, ClickHouse, NATS, Twilio, SendGrid credentials |
| `network-policies.yaml` | NetworkPolicy (x8) | Default deny-all + allow rules for internal, NATS, Redis, ClickHouse, Prometheus, HAProxy, DNS |
| `npu-device-plugin.yaml` | DaemonSet | AMD XDNA NPU device plugin on labeled nodes |

### Overlays

#### Production (`deploy/k8s/overlays/production/`)

| File | Description |
|------|-------------|
| `kustomization.yaml` | References base with production patches |
| `deployment-patch.yaml` | 20 replicas, 4-8 CPU / 8-16Gi memory, XDNA device, tolerations for NPU nodes |

#### Staging (`deploy/k8s/overlays/staging/`)

| File | Description |
|------|-------------|
| `kustomization.yaml` | References base with staging patches |
| `deployment-patch.yaml` | Reduced resource limits for staging environment |

### Deployment Commands

```bash
# Staging
kubectl apply -k deploy/k8s/overlays/staging

# Production
kubectl apply -k deploy/k8s/overlays/prod

# Verify
kubectl -n campaign-express get pods,svc,hpa
```

---

## 3. Terraform

### Azure (`deploy/terraform/azure/`)

| File | Description |
|------|-------------|
| `main.tf` | All Azure resource definitions |
| `variables.tf` | Configuration variables with defaults and validation |
| `outputs.tf` | Exported values (kubeconfig, connection strings, etc.) |

### Resources Provisioned

| Resource | Type | Configuration |
|----------|------|---------------|
| Resource Group | `azurerm_resource_group` | Regional deployment |
| Virtual Network | `azurerm_virtual_network` | `10.0.0.0/8` CIDR |
| AKS Subnet | `azurerm_subnet` | `10.240.0.0/16` |
| Redis Subnet | `azurerm_subnet` | `10.241.0.0/24` |
| ClickHouse Subnet | `azurerm_subnet` | `10.241.1.0/24` |
| AKS Cluster | `azurerm_kubernetes_cluster` | K8s 1.29, system pool (3x D4s_v5) |
| Bidding Node Pool | `azurerm_kubernetes_cluster_node_pool` | 20x D16s_v5, auto-scale 10-40, NPU labels |
| ClickHouse Node Pool | `azurerm_kubernetes_cluster_node_pool` | 2 nodes, 500GB OS disk, NVMe labels |
| Container Registry | `azurerm_container_registry` | Premium, geo-replication |
| Redis Cache | `azurerm_redis_cache` | Premium, capacity 2, 3 shards, 8Gi max |
| Key Vault | `azurerm_key_vault` | Kubelet identity access |
| Log Analytics | `azurerm_log_analytics_workspace` | 30-day retention |
| Managed Disks | `azurerm_managed_disk` | PremiumV2_LRS, 500GB per ClickHouse node |

### Usage

```bash
cd deploy/terraform/azure
terraform init
terraform plan -var="environment=prod"
terraform apply -var="environment=prod"

# Get kubeconfig
terraform output -raw kubeconfig > ~/.kube/config
```

### Key Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `environment` | — | `dev`, `staging`, or `prod` |
| `location` | `eastus2` | Primary Azure region |
| `dr_location` | `westus2` | Disaster recovery region |
| `kubernetes_version` | `1.29` | AKS K8s version |
| `bidding_vm_size` | `Standard_D16s_v5` | Bidding node VM size |
| `bidding_node_count` | `20` | Initial bidding nodes |
| `bidding_min_count` | `10` | Min auto-scale nodes |
| `bidding_max_count` | `40` | Max auto-scale nodes |
| `redis_capacity` | `2` | Redis Premium capacity |
| `redis_shard_count` | `3` | Redis cluster shards |
| `clickhouse_disk_size_gb` | `500` | ClickHouse disk per node |

---

## 4. Helm

### Chart (`deploy/helm/campaign-express/`)

| File | Description |
|------|-------------|
| `Chart.yaml` | Chart metadata (v0.1.0, application type) |
| `values.yaml` | Default configuration values |
| `templates/deployment.yaml` | Templated Deployment |
| `templates/service.yaml` | Templated Services |
| `templates/hpa.yaml` | Templated HPA |
| `templates/pdb.yaml` | Templated PDB |
| `templates/pvc.yaml` | Templated PVC |
| `templates/serviceaccount.yaml` | Templated ServiceAccount |
| `templates/_helpers.tpl` | Template helper functions |

### Default Values

```yaml
replicaCount: 20
image: campaign-express:latest
config:
  agentsPerNode: 20
  httpPort: 8080
  grpcPort: 9090
  metricsPort: 9091
npu:
  enabled: true
  resource: amd.com/xdna
  count: 1
nats: nats://nats:4222
redis: redis://redis-cluster:6379
clickhouse: http://clickhouse:8123
resources:
  requests: { cpu: 2, memory: 4Gi }
  limits: { cpu: 4, memory: 8Gi }
autoscaling:
  minReplicas: 10
  maxReplicas: 40
  targetCPU: 70
  targetMemory: 80
pdb:
  minAvailable: 80%
```

### Usage

```bash
# Install
helm install campaign-express deploy/helm/campaign-express/ \
  --namespace campaign-express --create-namespace

# Upgrade
helm upgrade campaign-express deploy/helm/campaign-express/ \
  --set replicaCount=30

# Template (dry-run)
helm template campaign-express deploy/helm/campaign-express/
```

---

## 5. Docker

### Files (`deploy/docker/`)

| File | Description |
|------|-------------|
| `Dockerfile` | Multi-stage production build (builder -> runtime) |
| `Dockerfile.dev` | Development image with `cargo-watch` hot-reload |
| `docker-compose.yml` | Full local development stack (7 services) |

### Production Dockerfile

```
Stage 1 (builder): rust:1.77-bookworm
  - Dependency caching via dummy source build
  - Full workspace release build

Stage 2 (runtime): debian:bookworm-slim
  - Non-root user (campaign)
  - Health check: curl /health
  - Ports: 8080 (HTTP), 9090 (gRPC), 9091 (metrics)
```

### Docker Compose Services

| Service | Image | Ports | Purpose |
|---------|-------|-------|---------|
| `campaign-express` | Build from Dockerfile | 8080, 9090, 9091 | Application |
| `nats` | nats:2.10-alpine | 4222, 8222 | Message broker (JetStream, 4Gi mem, 20Gi file) |
| `redis` | redis:7-alpine | 6379 | Cache (2Gi max, LRU eviction) |
| `clickhouse` | clickhouse-server:24-alpine | 8123, 9000 | Analytics DB |
| `prometheus` | prom/prometheus:v2.50.1 | 9092 | Metrics collection |
| `grafana` | grafana/grafana:10.3.3 | 3000 | Dashboards |

### Usage

```bash
# Start full stack
docker compose -f deploy/docker/docker-compose.yml up -d

# Start with development hot-reload
docker compose -f deploy/docker/docker-compose.yml \
  -f deploy/docker/docker-compose.dev.yml up -d

# View logs
docker compose -f deploy/docker/docker-compose.yml logs -f campaign-express

# Stop
docker compose -f deploy/docker/docker-compose.yml down
```

---

## 6. Monitoring

### Prometheus (`deploy/monitoring/prometheus/`)

| File | Description |
|------|-------------|
| `prometheus.yml` | Scrape config (10s interval, 5 jobs) |
| `prometheus-deployment.yaml` | Deployment (1 replica, 30-day retention, 50Gi PVC) |

**Scrape Targets:**

| Job | Target | Port |
|-----|--------|------|
| `campaign-express` | K8s SD (annotated pods) | 9091 |
| `nats` | nats.campaign-express.svc | 8222 |
| `redis` | redis-exporter.campaign-express.svc | 9121 |
| `haproxy` | haproxy-ingress.campaign-express.svc | 8404 |
| `clickhouse` | clickhouse.campaign-express.svc | 9363 |

### AlertManager (`deploy/monitoring/alertmanager/`)

| File | Description |
|------|-------------|
| `alert-rules.yaml` | 11 alert rules in 3 groups |
| `alertmanager-config.yaml` | Routing, receivers (PagerDuty, Slack, email) |
| `alertmanager-deployment.yaml` | Deployment (1 replica) |

**Alert Rules:**

| Group | Alert | Condition | Severity |
|-------|-------|-----------|----------|
| campaign-express | HighBidLatency | p99 > 50ms for 5m | critical |
| campaign-express | LowThroughput | < 500 req/s for 5m | warning |
| campaign-express | HighErrorRate | > 1% for 5m | critical |
| campaign-express | PodCrashLooping | 3+ restarts / 15m | critical |
| campaign-express | HighMemoryUsage | > 85% for 10m | warning |
| campaign-express | HighCPUUsage | > 80% for 10m | warning |
| redis | RedisHighMemory | > 85% for 5m | warning |
| redis | RedisConnectionsExhausted | > 90% for 5m | critical |
| redis | RedisHighLatency | > 10ms for 5m | warning |
| npu | NPUInferenceLatency | p99 > 10ms for 5m | warning |
| npu | NPUModelLoadFailure | Any failures | critical |

**Routing:**
- Critical alerts -> PagerDuty
- Warning alerts -> Ops team email + Slack

### Grafana (`deploy/monitoring/grafana/`)

| File | Description |
|------|-------------|
| `grafana-deployment.yaml` | Deployment (1 replica, port 3000) |
| `provisioning/datasources/datasource.yml` | Prometheus datasource (auto-configured) |
| `provisioning/dashboards/dashboards.yml` | Dashboard provisioning config |
| `dashboards/campaign-express.json` | Main dashboard (bid rate, latency p99/p95/p50, inference) |

### Logging (`deploy/monitoring/logging/`)

| File | Description |
|------|-------------|
| `loki-stack.yaml` | Loki (log aggregation) + Promtail (log collection) |

**Loki:** grafana/loki:2.9.0, boltdb-shipper backend, 7-day retention, 50Gi PVC
**Promtail:** grafana/promtail:2.9.0, DaemonSet on all nodes, K8s SD for pod logs

### Tracing (`deploy/monitoring/tracing/`)

| File | Description |
|------|-------------|
| `tempo-deployment.yaml` | Tempo distributed tracing |

**Tempo:** grafana/tempo:2.4.0, OTLP (4317/4318) + Jaeger (14268) receivers, 72h retention, 20Gi PVC

---

## 7. HAProxy

### Files (`deploy/haproxy/`)

| File | Description |
|------|-------------|
| `haproxy.cfg` | Load balancer configuration |
| `haproxy-deployment.yaml` | K8s Deployment |

### Configuration

- **Max Connections:** 100,000
- **Threads:** 4
- **Rate Limiting:** 10,000 req/10s per source IP (HTTP 429 on exceed)
- **DNS Resolver:** kube-dns (`10.96.0.10:53`) for dynamic K8s service discovery

### Frontends

| Frontend | Bind | Protocol | Routes |
|----------|------|----------|--------|
| `ft_http` | `:80`, `:443` (SSL) | HTTP/1.1 | `/v1/bid` -> bidding, `/health` -> health, `/metrics` -> metrics |
| `ft_grpc` | `:9090` | HTTP/2 (h2) | All -> gRPC backend |

### Backends

| Backend | Algorithm | Targets |
|---------|-----------|---------|
| `bk_campaign_express` | leastconn | 20 server-template (DNS) |
| `bk_grpc` | roundrobin | 20 servers (h2) |
| `bk_health` | — | Health check endpoint |

---

## 8. NATS

### Files (`deploy/nats/`)

| File | Description |
|------|-------------|
| `nats-deployment.yaml` | StatefulSet + Services |

### Configuration

- **Image:** nats:2.10-alpine
- **Replicas:** 3 (StatefulSet)
- **JetStream:** 4Gi memory store, 20Gi file store
- **Cluster:** `campaign-nats`, routes on port 6222
- **Resources:** 500m-1 CPU, 2-4Gi memory
- **Storage:** 20Gi PVC per replica
- **Discovery:** Headless service for pod DNS

### Ports

| Port | Protocol | Purpose |
|------|----------|---------|
| 4222 | TCP | Client connections |
| 6222 | TCP | Cluster routing |
| 8222 | HTTP | Monitoring |

---

## 9. Redis

### Files (`deploy/redis/`)

| File | Description |
|------|-------------|
| `redis-deployment.yaml` | StatefulSet + Services + Cluster Init Job |

### Configuration

- **Image:** redis:7-alpine
- **Replicas:** 6 (3 masters + 3 replicas via `--cluster-replicas 1`)
- **Mode:** Cluster mode enabled
- **Max Memory:** 8Gi with allkeys-LRU eviction
- **Persistence:** AOF (appendonly) + snapshot (60s / 10000 changes)
- **Resources:** 1-2 CPU, 8-10Gi memory
- **Storage:** 50Gi PVC per replica
- **Cluster Init:** Job runs `redis-cli --cluster create` after StatefulSet starts

---

## 10. ClickHouse

### Files (`deploy/clickhouse/`)

| File | Description |
|------|-------------|
| `clickhouse-deployment.yaml` | StatefulSet + Service + ConfigMap |

### Configuration

- **Image:** clickhouse/clickhouse-server:24-alpine
- **Replicas:** 2
- **Database:** `campaign_express`
- **Max Memory:** 10GB per query
- **Max Threads:** 8
- **Prometheus Metrics:** Enabled on port 9363
- **Resources:** 2-4 CPU, 8-16Gi memory
- **Storage:** 200Gi PVC per replica

### Ports

| Port | Protocol | Purpose |
|------|----------|---------|
| 8123 | HTTP | HTTP interface |
| 9000 | TCP | Native interface |
| 9363 | HTTP | Prometheus metrics |

---

## 11. AWS Alternative

### Files (`deploy/aws/`)

| File | Description |
|------|-------------|
| `terraform/main.tf` | EKS cluster, VPC, ECR, ElastiCache, Secrets Manager |
| `external-secrets-aws.yaml` | AWS Secrets Manager SecretStore |
| `neuron-device-plugin.yaml` | AWS Trainium/Inferentia device plugin (alternative to AMD XDNA) |
| `values-aws.yaml` | Helm values overrides for AWS |
| `ui-deployment.yaml` | Optional frontend deployment |
| `ui.Dockerfile` | Frontend container build |

### AWS Resources

| Resource | Service | Notes |
|----------|---------|-------|
| VPC | terraform-aws-modules/vpc | Multi-AZ networking |
| EKS Cluster | aws_eks_cluster | K8s 1.29+ |
| Container Registry | aws_ecr_repository | Lifecycle policies |
| Redis Cache | aws_elasticache | Managed Redis |
| Secrets | aws_secretsmanager | Credential storage |
| Inference | aws_neuron_device_plugin | Inferentia/Trainium NPU |
