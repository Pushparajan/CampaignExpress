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

Each node runs 20 Tokio-based bid agents consuming from a NATS JetStream queue. Agents score offers through a CoLaNet spiking neural network via the hardware-agnostic `CoLaNetProvider` trait (supporting CPU, Groq LPU, AWS Inferentia, Oracle Ampere Altra, and Tenstorrent RISC-V), with a Nagle-style batching buffer, two-tier cache (DashMap L1 + Redis L2), and non-blocking analytics via batched ClickHouse inserts.

## Workspace

```
CampaignExpress/
├── src/campaign-express/     # Binary — CLI, startup, orchestration
├── crates/
│   ├── core/                 # Types, OpenRTB, config, errors, inference provider trait
│   ├── npu-engine/           # CoLaNet SNN model + multi-head inference
│   │   └── backends/         # Hardware backends (CPU, Groq, Inferentia, Ampere, Tenstorrent)
│   ├── agents/               # 20 Tokio agents per node, NATS consumers + inference batcher
│   ├── cache/                # Redis + DashMap two-tier cache
│   ├── analytics/            # Async ClickHouse batch logger
│   ├── api-server/           # Axum REST + Tonic gRPC server
│   ├── loyalty/              # 3-tier loyalty program (Green/Gold/Reserve)
│   ├── dsp/                  # DSP integrations (TTD, DV360, Xandr, Amazon)
│   ├── channels/             # Multi-channel output (email, push, SMS, webhooks)
│   ├── management/           # Campaign CRUD, creatives, auth, workflows, approvals
│   ├── journey/              # Journey orchestration & state machines
│   ├── dco/                  # Dynamic Creative Optimization + brand guidelines + asset library
│   ├── cdp/                  # CDP adapters (Salesforce, Adobe, Segment, Tealium, Hightouch)
│   ├── platform/             # Auth, RBAC, multi-tenancy
│   ├── billing/              # Usage metering, Stripe billing, plan management
│   ├── ops/                  # SLA tracking, health monitoring, operational metrics
│   ├── personalization/      # Recommendation engine (CF, content-based, trending)
│   ├── segmentation/         # Audience segmentation & rule engine
│   ├── reporting/            # Report builder, budget tracking, scheduled exports
│   ├── integrations/         # Asana, Jira, AEM Assets, Bynder, Aprimo, Power BI, Excel
│   ├── intelligent-delivery/ # Smart delivery optimization + global suppression lists
│   ├── rl-engine/            # Reinforcement learning + OfferFit connector
│   ├── mobile-sdk/           # Mobile SDK server-side support
│   ├── plugin-marketplace/   # Plugin marketplace & extensibility
│   ├── sdk-docs/             # API reference, guides, examples, search engine
│   └── wasm-edge/            # Cloudflare Workers edge stub
├── ui/                       # Next.js 14 management dashboard
├── deploy/
│   ├── docker/               # Multi-stage Dockerfile + docker-compose
│   ├── k8s/                  # Kustomize base + prod/staging overlays + network policies
│   ├── helm/                 # Helm chart (campaign-express)
│   ├── haproxy/              # Ingress load balancer
│   ├── terraform/azure/      # AKS, Redis, ACR, Key Vault, ClickHouse IaC
│   ├── nats/                 # JetStream StatefulSet
│   ├── redis/                # 6-node Redis Cluster StatefulSet
│   ├── clickhouse/           # Analytics DB StatefulSet
│   └── monitoring/           # Prometheus, Grafana, AlertManager, Tempo, Loki
└── docs/                     # Deployment & architecture guides
    ├── MARKETER_GUIDE.md     # User guide for campaign managers and marketers
    └── ...
```

## Documentation

### Architecture & Deployment
- **[Architecture Guide](docs/ARCHITECTURE.md)** - Comprehensive end-to-end architecture documentation covering all modules, data flows, infrastructure, and integrations
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Production deployment instructions
- **[Local Deployment](docs/LOCAL_DEPLOYMENT.md)** - Local development environment setup
- **[Request Flow](docs/REQUEST_FLOW.md)** - Detailed request processing flow
- **[Prerequisites](docs/PREREQUISITES.md)** - Installation prerequisites for each platform
- **[SaaS Operations Guide](docs/SAAS_OPERATIONS.md)** - People, skills, and team structure needed to operate CampaignExpress as a SaaS product

### Operating Guides for Engineers (College Freshers)
- **[Rust Engineer Guide](docs/RUST_ENGINEER_GUIDE.md)** - Getting started guide for Rust engineers covering language fundamentals, async programming, and development workflow
- **[ML Engineer Guide](docs/ML_ENGINEER_GUIDE.md)** - Real-time ML inference operations, model deployment, and performance optimization for ML engineers
- **[SRE Guide](docs/SRE_GUIDE.md)** - Kubernetes operations, monitoring, incident response, and infrastructure management for SRE specialists

### Testing Documentation
- **[Test Strategy](docs/TEST_STRATEGY.md)** - Comprehensive test strategy covering test levels, types, environment setup, risk assessment, and quality metrics
- **[Manual Test Cases](docs/MANUAL_TEST_CASES.md)** - Detailed manual test cases for all features including campaign management, real-time bidding, integrations, performance, and security testing

## Quick Start

### Prerequisites

- Rust 1.77+ (`rustup install stable`)
- Docker & Docker Compose
- Node.js 18+ and npm (for the management UI)

> **First time?** See [docs/PREREQUISITES.md](docs/PREREQUISITES.md) for detailed installation instructions for each platform.
> 
> **For Marketers:** See [docs/MARKETER_GUIDE.md](docs/MARKETER_GUIDE.md) for a comprehensive guide on creating campaigns, managing creatives, and using all marketer-facing features.
>
> **For Engineers:** New to the team? Check out our role-specific guides:
> - [Rust Engineer Guide](docs/RUST_ENGINEER_GUIDE.md) - For backend engineers working with Rust
> - [ML Engineer Guide](docs/ML_ENGINEER_GUIDE.md) - For ML engineers working on inference and models
> - [SRE Guide](docs/SRE_GUIDE.md) - For SRE/DevOps engineers managing infrastructure

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

# Workflows & Approvals
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/workflows/calendar

# Brand & Asset Library
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/brand/assets

# Budget & Reports
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/reports/templates

# Recommendations
curl http://localhost:8080/api/v1/recommendations/user-123?strategy=cf

# Integrations (Asana, Jira, DAM, BI)
curl -H "Authorization: Bearer campaign-express-demo-token" \
  http://localhost:8080/api/v1/integrations/dam/search?query=banner

# Inference Providers
curl http://localhost:8080/api/v1/inference/providers
```

## Key Features

| Category | Capabilities |
|---|---|
| **Real-Time Bidding** | OpenRTB 2.6, sub-10ms inference, multi-impression support |
| **ML Inference** | CoLaNet SNN with hardware-agnostic provider (CPU, Groq, Inferentia, Ampere, Tenstorrent) |
| **Inference Batching** | Nagle-style batcher (500µs / 16-item flush) for accelerator throughput |
| **Recommendations** | Collaborative filtering, content-based, frequently-bought-together, trending, new arrivals |
| **RL Engine** | OfferFit connector with Thompson Sampling fallback, reward signals |
| **Loyalty** | 3-tier program (Green/Gold/Reserve), star earning/redemption, tier upgrades |
| **DSP Integration** | The Trade Desk, Google DV360, Xandr, Amazon DSP |
| **Channels** | Email (SendGrid), push notifications, SMS (Twilio), in-app, webhooks |
| **Journey Orchestration** | State machines, triggers (event/segment/schedule), branching, delays |
| **DCO** | Modular creative assembly, Thompson Sampling, variant performance tracking |
| **Brand Guidelines** | Color palette, typography, tone-of-voice, logo usage validation |
| **Asset Library** | Versioned asset storage, search, folder management |
| **Campaign Workflows** | 9-stage lifecycle, multi-step approvals, role-based review |
| **Budget Tracking** | Pacing alerts (80%/100%/daily), ROAS/ROI calculation |
| **Report Builder** | 10 report types, scheduled exports (CSV/JSON/Excel), 5 templates |
| **CDP** | Bidirectional sync with Salesforce, Adobe, Segment, Tealium, Hightouch |
| **Segmentation** | Rule-based audience segmentation with real-time evaluation |
| **Personalization** | Real-time offer personalization with ML-powered scoring |
| **Suppression** | Global per-channel suppression lists with expiry |
| **Integrations** | Asana, Jira, AEM Assets, Bynder, Aprimo, Power BI, Excel |
| **Platform** | Multi-tenant auth, RBAC, API key management, audit logging |
| **Billing** | Usage metering, Stripe integration, plan management |
| **Experimentation** | A/B/n testing, deterministic assignment, significance checking |
| **Caching** | DashMap L1 (lock-free) → Redis L2 cluster, automatic maintenance |
| **Analytics** | Non-blocking mpsc → batched ClickHouse insert pipeline |
| **Observability** | Prometheus + AlertManager (11 rules), Grafana, Tempo tracing, Loki logging |
| **Security** | Network policies, cert-manager (Let's Encrypt), External Secrets (Azure KV) |
| **IaC** | Terraform (Azure AKS, Redis Premium, ACR, Key Vault, ClickHouse) |

## Development

### Build & Test

```bash
cargo check --workspace          # Type-check
cargo test --workspace           # Run all tests
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
| ML Inference | ndarray 0.15 (pure-Rust, hardware-agnostic CoLaNetProvider) |
| Inference Backends | CPU, Groq LPU, AWS Inferentia 2/3, Oracle Ampere Altra, Tenstorrent RISC-V |
| Frontend | Next.js 14, React 18, TanStack Query 5, Tailwind CSS |
| Observability | Prometheus + AlertManager, Grafana 10, Tempo (tracing), Loki (logging) |
| Security | cert-manager (Let's Encrypt), External Secrets Operator, K8s NetworkPolicies |
| IaC | Terraform (Azure AKS, Redis Premium, ACR, Key Vault) |
| Container | Multi-stage Docker, Kubernetes + Kustomize + Helm |
| Edge | Cloudflare Workers (WASM stub) |
| Load Balancer | HAProxy |
| Integrations | Asana, Jira, AEM Assets, Bynder, Aprimo, Power BI, Excel |

## License

Proprietary. All rights reserved.
