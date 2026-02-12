# Campaign Express

High-throughput real-time ad offer personalization platform built in Rust, designed to serve **50M offers/hour** across a 20-node Kubernetes cluster with AMD XDNA NPU acceleration.

## Architecture

```
                     ┌──────────────┐
                     │  HAProxy LB  │
                     └──────┬───────┘
                            │
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │  Node 0  │  │  Node 1  │  │  Node N  │
        │ 20 agents│  │ 20 agents│  │ 20 agents│
        └────┬─────┘  └────┬─────┘  └────┬─────┘
             │              │              │
     ┌───────┴──────────────┴──────────────┴───────┐
     │            NATS JetStream Cluster            │
     └──────────────────────┬──────────────────────┘
                            │
         ┌──────────┬───────┴───────┬──────────┐
         ▼          ▼               ▼          ▼
   ┌──────────┐ ┌────────┐  ┌───────────┐ ┌────────┐
   │ Redis 6  │ │ NPU    │  │ClickHouse │ │Grafana │
   │ Cluster  │ │ Engine │  │ Analytics │ │  + P8s │
   └──────────┘ └────────┘  └───────────┘ └────────┘
```

Each node runs 20 Tokio-based bid agents consuming from a NATS JetStream queue. Agents score offers through a CoLaNet spiking neural network, with a two-tier cache (DashMap L1 + Redis L2) and non-blocking analytics via batched ClickHouse inserts.

## Workspace

```
CampaignExpress/
├── src/campaign-express/     # Binary — CLI, startup, orchestration
├── crates/
│   ├── core/                 # Types, OpenRTB, config, errors
│   ├── npu-engine/           # CoLaNet SNN model + multi-head inference
│   ├── agents/               # 20 Tokio agents per node, NATS consumers
│   ├── cache/                # Redis + DashMap two-tier cache
│   ├── analytics/            # Async ClickHouse batch logger
│   ├── api-server/           # Axum REST + Tonic gRPC server
│   ├── loyalty/              # 3-tier loyalty program (Green/Gold/Reserve)
│   ├── dsp/                  # DSP integrations (TTD, DV360, Xandr, Amazon)
│   ├── channels/             # Multi-channel output (email, push, SMS, webhooks)
│   ├── management/           # Campaign CRUD, creatives, auth, demo data
│   ├── journey/              # Journey orchestration & state machines
│   ├── dco/                  # Dynamic Creative Optimization engine
│   ├── cdp/                  # CDP adapters (Salesforce, Adobe, Segment, Tealium, Hightouch)
│   └── wasm-edge/            # Cloudflare Workers edge stub
├── ui/                       # Next.js 14 management dashboard
├── deploy/
│   ├── docker/               # Multi-stage Dockerfile + docker-compose
│   ├── k8s/                  # Kustomize base + prod/staging overlays
│   ├── haproxy/              # Ingress load balancer
│   ├── nats/                 # JetStream StatefulSet
│   ├── redis/                # 6-node Redis Cluster StatefulSet
│   ├── clickhouse/           # Analytics DB StatefulSet
│   └── monitoring/           # Prometheus + Grafana dashboards
└── docs/                     # Deployment guides
```

## Quick Start

### Prerequisites

- Rust 1.77+ (`rustup install stable`)
- Docker & Docker Compose
- Node.js 18+ and npm (for the management UI)

> **First time?** See [docs/PREREQUISITES.md](docs/PREREQUISITES.md) for detailed installation instructions for each platform.

### Docker Compose (full stack)

```bash
docker compose -f deploy/docker/docker-compose.yml up -d
```

This starts Campaign Express alongside NATS, Redis, ClickHouse, Prometheus, and Grafana.

| Service            | URL                          |
|--------------------|------------------------------|
| REST API           | http://localhost:8080         |
| gRPC               | localhost:9090                |
| Prometheus Metrics | http://localhost:9091/metrics |
| Grafana            | http://localhost:3000         |
| NATS               | localhost:4222                |
| ClickHouse HTTP    | http://localhost:8123         |

### Native (Rust only)

```bash
# Build
cargo build --release

# Run in API-only mode (no NATS/Redis required)
cargo run --release -- --api-only --node-id dev-01

# Run with full infrastructure
cargo run --release -- --node-id node-01 --agents-per-node 20
```

### Management UI

```bash
cd ui
npm install
npm run dev
# → http://localhost:3000
```

The UI proxies API calls to the Rust backend at `localhost:8080`.

## API

### Health & Metrics

```bash
curl http://localhost:8080/health
curl http://localhost:9091/metrics
```

### OpenRTB Bidding

```bash
curl -X POST http://localhost:8080/v1/bid \
  -H "Content-Type: application/json" \
  -d '{
    "id": "req-001",
    "imp": [{"id": "imp-1", "bidfloor": 0.50, "banner": {"w": 300, "h": 250}}],
    "site": {"domain": "example.com"},
    "user": {"id": "user-123"}
  }'
```

### Campaign Management

All management endpoints require `Authorization: Bearer campaign-express-demo-token`.

```bash
# Campaigns
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/management/campaigns

# Creatives, Loyalty, DSP, Channels
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/management/creatives

# Journey Orchestration
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/management/journeys

# DCO Templates
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/management/dco/templates

# CDP Platforms
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/management/cdp/platforms

# Experiments
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/management/experiments
```

## Key Features

| Category | Capabilities |
|---|---|
| **Real-Time Bidding** | OpenRTB 2.6, sub-10ms inference, multi-impression support |
| **ML Inference** | CoLaNet SNN on AMD XDNA NPU, multi-head output (offers + DCO variants) |
| **Loyalty** | 3-tier program (Green/Gold/Reserve), star earning/redemption, tier upgrades |
| **DSP Integration** | The Trade Desk, Google DV360, Xandr, Amazon DSP |
| **Channels** | Email (SendGrid), push notifications, SMS, in-app, webhooks |
| **Journey Orchestration** | State machines, triggers (event/segment/schedule), branching, delays |
| **DCO** | Modular creative assembly, Thompson Sampling, variant performance tracking |
| **CDP** | Bidirectional sync with Salesforce, Adobe, Segment, Tealium, Hightouch |
| **Experimentation** | A/B/n testing, deterministic assignment, significance checking |
| **Caching** | DashMap L1 (lock-free) → Redis L2 cluster, automatic maintenance |
| **Analytics** | Non-blocking mpsc → batched ClickHouse insert pipeline |
| **Observability** | Prometheus metrics, OpenTelemetry traces, structured JSON logging |

## Development

### Build & Test

```bash
cargo check --workspace          # Type-check
cargo test --workspace           # Run all tests (18 tests)
cargo clippy --workspace -- -D warnings  # Lint
cargo fmt --all                  # Format
```

### Configuration

The app loads config from environment variables and CLI arguments. Key variables:

| Variable | Default | Description |
|---|---|---|
| `NODE_ID` | `node-01` | Unique identifier for this node |
| `AGENTS_PER_NODE` | `20` | Number of bid agents per node |
| `HTTP_PORT` | `8080` | REST API listen port |
| `GRPC_PORT` | `9090` | gRPC listen port |
| `METRICS_PORT` | `9091` | Prometheus metrics port |
| `NATS_URL` | `nats://localhost:4222` | NATS server URL |
| `REDIS_URL` | `redis://localhost:6379` | Redis connection URL |
| `CLICKHOUSE_URL` | `http://localhost:8123` | ClickHouse HTTP URL |
| `NPU_DEVICE` | `cpu` | NPU device (`cpu`, `npu0`, etc.) |
| `RUST_LOG` | `info` | Log level filter |

### CI/CD

The GitHub Actions pipeline (`.github/workflows/ci.yml`) runs:

1. **Check & Lint** — `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`
2. **Test** — `cargo test --workspace`
3. **Docker Build** — Multi-stage image push to `ghcr.io` (main branch only)

### Kubernetes Deployment

Production and staging overlays are in `deploy/k8s/`:

```bash
# Staging
kubectl apply -k deploy/k8s/overlays/staging

# Production (20-node cluster)
kubectl apply -k deploy/k8s/overlays/prod
```

See `docs/LOCAL_DEPLOYMENT.md` for detailed setup instructions.

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust 1.77, Edition 2021 |
| Async Runtime | Tokio 1.36 |
| HTTP Framework | Axum 0.7 |
| gRPC | Tonic 0.12 + Prost 0.13 |
| Message Queue | NATS JetStream (async-nats 0.35) |
| Cache | Redis 7 (redis-rs 0.25) + DashMap 5 |
| Analytics DB | ClickHouse 24 |
| ML Inference | ndarray 0.15 (pure-Rust, NPU extension point) |
| Frontend | Next.js 14, React 18, TanStack Query 5, Tailwind CSS |
| Observability | Prometheus, Grafana 10, OpenTelemetry, tracing |
| Container | Multi-stage Docker, Kubernetes + Kustomize |
| Edge | Cloudflare Workers (WASM stub) |
| Load Balancer | HAProxy |

## License

Proprietary. All rights reserved.
