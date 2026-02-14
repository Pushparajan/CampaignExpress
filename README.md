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
│   │   └── audience_proxy    # Segment proxy, incremental sync, match-rate, budget pacing
│   ├── channels/             # Multi-channel output (email, push, SMS, WhatsApp, web push, webhooks)
│   │   └── content_studio    # Content Studio: HTML editor, localization, render-time personalization
│   ├── management/           # Campaign CRUD, creatives, auth, workflows, approvals
│   │   ├── workspace         # Marketer UX: unified create flow, bulk ops, explainability, calendar
│   │   └── governance        # Unified governance gate (revision + preflight + policy + tasks)
│   ├── journey/              # Journey orchestration & state machines
│   ├── dco/                  # Dynamic Creative Optimization + brand guidelines + asset library
│   │   └── creative_export   # Creative export contracts, IAB placement validation, lineage tracking
│   ├── cdp/                  # CDP adapters (Salesforce, Adobe, Segment, Tealium, Hightouch)
│   │   └── feature_store     # Online feature store with TTL staleness, computed features
│   ├── platform/             # Auth, RBAC, multi-tenancy
│   ├── billing/              # Usage metering, Stripe billing, plan management
│   ├── ops/                  # SLA tracking, health monitoring, operational metrics
│   ├── personalization/      # Recommendation engine (CF, content-based, trending)
│   │   └── decisioning       # Real-time decision API: multi-objective optimization, simulation
│   ├── segmentation/         # Audience segmentation & rule engine
│   ├── reporting/            # Report builder, budget tracking, scheduled exports
│   │   └── measurement       # Unified measurement: cross-channel events, experiment lift
│   ├── integrations/         # Asana, Jira, AEM Assets, Bynder, Aprimo, Power BI, Excel
│   │   └── capabilities      # Connector capability registry, health monitoring, certification
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
├── docs/                     # All project documentation
│   ├── ARCHITECTURE.md       # End-to-end architecture
│   ├── API_REFERENCE.md      # REST & gRPC API specification
│   ├── BRD.md                # Business Requirements Document
│   ├── DEPLOYMENT.md         # Production deployment guide
│   ├── INFRASTRUCTURE.md     # Infrastructure configuration reference
│   ├── LOCAL_DEPLOYMENT.md   # Local dev environment setup
│   ├── MANUAL_TEST_CASES.md  # 20-category manual test cases
│   ├── MARKETER_GUIDE.md     # User guide for marketers
│   ├── ML_ENGINEER_GUIDE.md  # Onboarding for ML engineers
│   ├── PREREQUISITES.md      # Platform installation guide
│   ├── REQUEST_FLOW.md       # 22 request flow scenarios
│   ├── RUST_ENGINEER_GUIDE.md # Onboarding for Rust engineers
│   ├── SAAS_OPERATIONS.md    # Team structure & staffing guide
│   ├── SRE_GUIDE.md          # Onboarding for SRE/DevOps
│   └── TEST_STRATEGY.md      # Test strategy & quality metrics
├── CONTRIBUTING.md           # Contributor guidelines
├── CHANGELOG.md              # Version history
├── SECURITY.md               # Security policy
└── LICENSE                   # Proprietary license
```

## Documentation

### Architecture & Design
- **[Architecture Guide](docs/ARCHITECTURE.md)** — End-to-end architecture covering all modules, data flows, infrastructure, and integrations
- **[Request Flow](docs/REQUEST_FLOW.md)** — 22 detailed request flow scenarios with data flow diagrams
- **[Business Requirements (BRD)](docs/BRD.md)** — Functional requirements across 30+ feature areas

### API & Reference
- **[API Reference](docs/API_REFERENCE.md)** — Complete REST (76+ endpoints) and gRPC API specification with request/response schemas
- **[Infrastructure Reference](docs/INFRASTRUCTURE.md)** — Kubernetes, Terraform, Helm, Docker, and monitoring configuration reference

### Deployment & Operations
- **[Deployment Guide](docs/DEPLOYMENT.md)** — Production Kubernetes deployment instructions
- **[Local Deployment](docs/LOCAL_DEPLOYMENT.md)** — Local development environment setup (includes one-command `quickstart.sh`)
- **[Prerequisites](docs/PREREQUISITES.md)** — Platform-specific installation guide (macOS, Linux, Windows)
- **[SaaS Operations Guide](docs/SAAS_OPERATIONS.md)** — Team structure, skills matrix, and staffing by growth stage (18-60 people)

### Operating Guides (Onboarding)
- **[Rust Engineer Guide](docs/RUST_ENGINEER_GUIDE.md)** — Language fundamentals, async programming, development workflow
- **[ML Engineer Guide](docs/ML_ENGINEER_GUIDE.md)** — Inference pipeline, model deployment, performance optimization
- **[SRE Guide](docs/SRE_GUIDE.md)** — Kubernetes operations, monitoring, incident response, infrastructure management
- **[Marketer Guide](docs/MARKETER_GUIDE.md)** — Campaign management, creative workflows, journey orchestration, reporting

### Testing
- **[Test Strategy](docs/TEST_STRATEGY.md)** — Test levels, types, environments, risk assessment, quality metrics
- **[Manual Test Cases](docs/MANUAL_TEST_CASES.md)** — 20-category manual test cases (authentication, campaigns, RTB, security, etc.)

### Project
- **[Contributing](CONTRIBUTING.md)** — Development workflow, code standards, commit guidelines, PR process
- **[Changelog](CHANGELOG.md)** — Version history and release notes
- **[Security Policy](SECURITY.md)** — Vulnerability reporting, security architecture, compliance
- **[License](LICENSE)** — Proprietary license

## Quick Start

### One-Command Setup (Recommended for Beginners)

```bash
./quickstart.sh
```

This single command handles everything: checks prerequisites, builds the workspace, starts all infrastructure (NATS, Redis, ClickHouse, Prometheus, Grafana), seeds test data, starts the backend, and runs smoke tests. When it finishes, the platform is fully running with demo campaigns, creatives, and journeys pre-loaded.

```bash
./quickstart.sh --check        # Just verify prerequisites are installed
./quickstart.sh --no-build     # Skip Rust build (if already built)
./quickstart.sh --reset        # Wipe all data and start fresh
```

### Prerequisites

- Rust 1.77+ (`rustup install stable`)
- Docker & Docker Compose
- Node.js 18+ and npm (for the management UI)

> **First time?** See [docs/PREREQUISITES.md](docs/PREREQUISITES.md) for detailed installation instructions for macOS, Linux, and Windows.
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
| **Channels** | Email (SendGrid), push, SMS (Twilio), in-app, WhatsApp, web push, content cards, webhooks |
| **Content Studio** | HTML editor, localization engine, variable browser, render-time personalization per block |
| **Journey Orchestration** | State machines, triggers (event/segment/schedule), branching, delays |
| **DCO** | Modular creative assembly, Thompson Sampling, variant performance tracking |
| **Creative Export** | IAB placement validation, export contracts, creative lineage tracking |
| **Brand Guidelines** | Color palette, typography, tone-of-voice, logo usage validation |
| **Asset Library** | Versioned asset storage, search, folder management, asset ops studio |
| **Campaign Workflows** | 9-stage lifecycle, multi-step approvals, role-based review |
| **Unified Governance** | Single go-live gate combining revision + preflight + policy + task checks |
| **Marketer Workspace** | Unified create flow, bulk operations, explainability engine, operator calendar |
| **Budget Tracking** | Pacing alerts (80%/100%/daily), ROAS/ROI calculation |
| **Report Builder** | 10 report types, scheduled exports (CSV/JSON/Excel), 5 templates |
| **Unified Measurement** | Standardized cross-channel events, breakdown reporting, experiment lift |
| **CDP** | Bidirectional sync with Salesforce, Adobe, Segment, Tealium, Hightouch |
| **Feature Store** | Online feature store with TTL staleness, computed features, health monitoring |
| **Segmentation** | Rule-based audience segmentation with real-time evaluation |
| **Personalization** | Real-time offer personalization with ML-powered scoring |
| **Real-Time Decisioning** | Multi-objective optimization (CTR/revenue/LTV), explainability, simulation mode |
| **Suppression** | Global per-channel suppression lists with expiry |
| **Paid Media Proxy** | Segment-to-DSP audience proxy, incremental sync, match-rate estimation, budget pacing |
| **Integrations** | Asana, Jira, AEM Assets, Bynder, Aprimo, Power BI, Excel |
| **Connector Runtime** | Capability registry, EMA health monitoring, 12-test certification harness |
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
