# Campaign Express — Local Deployment Guide

Complete guide to running the entire Campaign Express platform on a local development machine.

---

## Fastest Path (One Command)

If you just want everything running with test data pre-loaded:

```bash
./quickstart.sh
```

This checks prerequisites, builds the workspace, starts infrastructure (NATS, Redis, ClickHouse, Prometheus, Grafana), seeds demo data (campaigns, users, bid events), starts the backend, and runs smoke tests. When done, the API is live at `http://localhost:8080` with 7 demo campaigns, 12 creatives, and 5 journeys ready to go.

**Other options:**

| Command | What it does |
|---------|-------------|
| `./quickstart.sh --check` | Just verify prerequisites are installed |
| `./quickstart.sh --no-build` | Skip Rust build (use existing binary) |
| `./quickstart.sh --no-frontend` | Skip Node.js/UI setup |
| `./quickstart.sh --seed-only` | Only seed data (infra must already be running) |
| `./quickstart.sh --reset` | Wipe all data volumes and start fresh |
| `./quickstart.sh --full-agents` | Run with NATS agents instead of API-only mode |

If you need more control, follow the step-by-step sections below.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start (Docker Compose)](#quick-start-docker-compose)
3. [Manual Setup (Native Rust + Services)](#manual-setup)
4. [Management UI Setup](#management-ui-setup)
5. [Verifying the Stack](#verifying-the-stack)
6. [API Reference](#api-reference)
7. [Environment Variables](#environment-variables)
8. [Architecture Overview](#architecture-overview)
9. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | 1.77+ | Compile all crates |
| **Docker** + **Docker Compose** | 24.0+ / 2.20+ | Run infrastructure services |
| **Node.js** | 18+ | Management UI frontend |
| **npm** or **pnpm** | 9+ / 8+ | JS dependency management |

### Optional

| Tool | Purpose |
|------|---------|
| `cargo-watch` | Auto-rebuild on file changes |
| `jq` | Pretty-print API responses |
| `httpie` or `curl` | Manual API testing |

### Install Rust toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

---

## Quick Start (Docker Compose)

The fastest way to get the full stack running.

### 1. Start infrastructure + application

```bash
cd deploy/docker
docker compose up -d
```

This starts:

| Service | Port | Description |
|---------|------|-------------|
| **campaign-express** | `8080` (HTTP), `9090` (gRPC), `9091` (metrics) | Main application |
| **NATS** | `4222` (clients), `8222` (monitoring) | Message broker with JetStream |
| **Redis** | `6379` | User profile cache (L2) |
| **ClickHouse** | `8123` (HTTP), `9000` (native) | Analytics event storage |
| **Prometheus** | `9092` | Metrics collection |
| **Grafana** | `3000` | Dashboards (admin/campaign-express) |

### 2. Start the Management UI

```bash
cd ui
npm install
npm run dev
```

UI available at **http://localhost:3001**

### 3. Verify

```bash
# Health check
curl http://localhost:8080/health

# Login to management API
curl -X POST http://localhost:8080/api/v1/management/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}'
```

### 4. Tear down

```bash
cd deploy/docker
docker compose down        # Stop containers
docker compose down -v     # Stop + remove volumes
```

---

## Manual Setup

Run the Rust binary natively (faster iteration cycle, no Docker for the app).

### Step 1: Start infrastructure services only

Create a minimal `docker-compose.infra.yml` or start services individually:

```bash
# NATS with JetStream
docker run -d --name ce-nats \
  -p 4222:4222 -p 8222:8222 \
  nats:2.10-alpine --jetstream --store_dir /data -m 8222

# Redis
docker run -d --name ce-redis \
  -p 6379:6379 \
  redis:7-alpine redis-server --maxmemory 2gb --maxmemory-policy allkeys-lru

# ClickHouse
docker run -d --name ce-clickhouse \
  -p 8123:8123 -p 9000:9000 \
  -e CLICKHOUSE_DB=campaign_express \
  -e CLICKHOUSE_DEFAULT_ACCESS_MANAGEMENT=1 \
  clickhouse/clickhouse-server:24-alpine
```

### Step 2: Build the workspace

```bash
# From project root
cargo build --workspace
```

First build downloads and compiles all dependencies (~2-4 minutes). Subsequent builds are incremental (~5-15 seconds).

### Step 3: Run the application

```bash
# Full mode (with NATS agents)
cargo run --bin campaign-express

# API-only mode (no NATS dependency, good for UI development)
cargo run --bin campaign-express -- --api-only
```

The `--api-only` flag skips NATS agent startup. The HTTP API server, management endpoints, and all in-memory demo data will still be available. This is the recommended mode for frontend development.

### Step 4: Verify the build

```bash
# Run all tests
cargo test --workspace

# Check for compilation warnings
cargo check --workspace
```

### Development with auto-reload

```bash
cargo install cargo-watch
cargo watch -x 'run --bin campaign-express -- --api-only'
```

---

## Management UI Setup

The management UI is a Next.js application that connects to the Rust backend.

### Install and run

```bash
cd ui
npm install
npm run dev
```

The UI starts on **http://localhost:3001** (or the next available port).

### How it connects

The `next.config.js` has a rewrite rule that proxies all `/api/*` requests to `http://localhost:8080`:

```js
// ui/next.config.js
rewrites() {
  return [{ source: "/api/:path*", destination: "http://localhost:8080/api/:path*" }];
}
```

No CORS configuration is needed because the proxy makes all API calls same-origin.

### Login credentials (development)

| Username | Password | Notes |
|----------|----------|-------|
| `admin` | `admin` | Full access |
| Any username | `campaign2024` | Demo access |

### UI Pages

| Path | Page | Description |
|------|------|-------------|
| `/` | Dashboard | Real-time KPIs, throughput/latency charts, recent campaigns |
| `/campaigns` | Campaigns | CRUD for campaigns with budget, targeting, pacing |
| `/creatives` | Creatives | Ad creative management by format |
| `/journeys` | Journeys | Journey orchestration flows (welcome, cart abandon, re-engage) |
| `/dco` | DCO Templates | Dynamic creative optimization template builder |
| `/cdp` | CDP Integrations | CDP platform connections + sync history |
| `/experiments` | Experiments | A/B/n test management with lift tracking |
| `/monitoring` | Monitoring | System health, pod status, error rates |
| `/workflows` | Workflows | Campaign approval workflows, pending reviews |
| `/brand` | Brand | Brand guidelines, asset library, validation |
| `/reports` | Reports | Report builder, budget tracking, scheduled exports |
| `/integrations` | Integrations | Asana, Jira, DAM, BI tool connections |
| `/settings` | Settings | Model reload, inference provider, system configuration |

---

## Verifying the Stack

### Health endpoints

```bash
# Liveness probe
curl http://localhost:8080/live

# Readiness probe
curl http://localhost:8080/ready

# Health check
curl http://localhost:8080/health
```

### Management API walkthrough

```bash
# 1. Login
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/management/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}' | jq -r '.token')

echo "Token: $TOKEN"

# 2. List campaigns (7 demo campaigns pre-seeded)
curl -s http://localhost:8080/api/v1/management/campaigns \
  -H "Authorization: Bearer $TOKEN" | jq '.[].name'

# 3. Create a new campaign
curl -s -X POST http://localhost:8080/api/v1/management/campaigns \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Test Campaign",
    "budget": 10000,
    "daily_budget": 500,
    "targeting": {
      "geo_regions": ["US"],
      "segments": [100, 200],
      "devices": ["mobile"]
    }
  }' | jq '.id, .name, .status'

# 4. Get monitoring overview
curl -s http://localhost:8080/api/v1/management/monitoring/overview \
  -H "Authorization: Bearer $TOKEN" | jq .

# 5. List journeys (5 demo journeys pre-seeded)
curl -s http://localhost:8080/api/v1/management/journeys \
  -H "Authorization: Bearer $TOKEN" | jq '.[].name'

# 6. List DCO templates (3 demo templates pre-seeded)
curl -s http://localhost:8080/api/v1/management/dco/templates \
  -H "Authorization: Bearer $TOKEN" | jq '.[].name'

# 7. List CDP platforms (5 platforms pre-configured)
curl -s http://localhost:8080/api/v1/management/cdp/platforms \
  -H "Authorization: Bearer $TOKEN" | jq '.[].platform'

# 8. List experiments (4 demo experiments pre-seeded)
curl -s http://localhost:8080/api/v1/management/experiments \
  -H "Authorization: Bearer $TOKEN" | jq '.[] | {name, status, metric}'

# 9. Submit a bid request
curl -s -X POST http://localhost:8080/v1/bid \
  -H "Content-Type: application/json" \
  -d '{
    "id": "req-001",
    "imp": [{"id": "imp-1", "bidfloor": 0.50}],
    "site": {"domain": "example.com"},
    "device": {"ua": "Mozilla/5.0", "ip": "1.2.3.4"},
    "user": {"id": "user-123"}
  }' | jq .

# 10. View audit log
curl -s http://localhost:8080/api/v1/management/audit-log \
  -H "Authorization: Bearer $TOKEN" | jq '.[0:3]'
```

### Metrics

```bash
# Prometheus metrics (if metrics server is running)
curl http://localhost:9091/metrics 2>/dev/null | head -20
```

### Grafana (Docker Compose only)

Open **http://localhost:3000** and login with `admin` / `campaign-express`.

---

## API Reference

### Authentication

All `/api/v1/management/*` endpoints (except login) require a bearer token:

```
Authorization: Bearer <token>
```

### Core Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/bid` | Submit OpenRTB bid request |
| `GET` | `/health` | Health check |
| `GET` | `/ready` | Readiness probe |
| `GET` | `/live` | Liveness probe |

### Loyalty

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/loyalty/earn` | Earn stars |
| `POST` | `/v1/loyalty/redeem` | Redeem reward |
| `GET` | `/v1/loyalty/balance/{user_id}` | Get balance |
| `POST` | `/v1/loyalty/reward-signal` | RLHF reward signal |

### DSP

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/dsp/bid` | Route bid to DSP |
| `POST` | `/v1/dsp/win` | Report win notification |
| `GET` | `/v1/dsp/status` | DSP integration status |

### Channels

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/channels/ingest` | Ingest omnichannel event |
| `POST` | `/v1/channels/activate` | Activate offer on channel |
| `POST` | `/v1/webhooks/sendgrid` | SendGrid webhook receiver |
| `GET` | `/v1/channels/email/analytics/{id}` | Email analytics by activation |
| `GET` | `/v1/channels/email/analytics` | All email analytics |

### Management — Campaigns

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/management/auth/login` | Login (returns token) |
| `GET` | `/api/v1/management/campaigns` | List all campaigns |
| `POST` | `/api/v1/management/campaigns` | Create campaign |
| `GET` | `/api/v1/management/campaigns/{id}` | Get campaign |
| `PUT` | `/api/v1/management/campaigns/{id}` | Update campaign |
| `DELETE` | `/api/v1/management/campaigns/{id}` | Delete campaign |
| `POST` | `/api/v1/management/campaigns/{id}/pause` | Pause campaign |
| `POST` | `/api/v1/management/campaigns/{id}/resume` | Resume campaign |

### Management — Creatives

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/management/creatives` | List all creatives |
| `POST` | `/api/v1/management/creatives` | Create creative |
| `GET` | `/api/v1/management/creatives/{id}` | Get creative |
| `PUT` | `/api/v1/management/creatives/{id}` | Update creative |
| `DELETE` | `/api/v1/management/creatives/{id}` | Delete creative |

### Management — Journeys

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/management/journeys` | List all journeys |
| `POST` | `/api/v1/management/journeys` | Create journey |
| `GET` | `/api/v1/management/journeys/{id}` | Get journey |
| `DELETE` | `/api/v1/management/journeys/{id}` | Delete journey |
| `GET` | `/api/v1/management/journeys/{id}/stats` | Journey statistics |

### Management — DCO

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/management/dco/templates` | List DCO templates |
| `POST` | `/api/v1/management/dco/templates` | Create DCO template |
| `GET` | `/api/v1/management/dco/templates/{id}` | Get DCO template |
| `DELETE` | `/api/v1/management/dco/templates/{id}` | Delete DCO template |

### Management — CDP

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/management/cdp/platforms` | List CDP platforms |
| `GET` | `/api/v1/management/cdp/sync-history` | Sync history |

### Management — Experiments

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/management/experiments` | List experiments |
| `POST` | `/api/v1/management/experiments` | Create experiment |
| `GET` | `/api/v1/management/experiments/{id}` | Get experiment |

### Management — Monitoring & Operations

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/management/monitoring/overview` | Platform metrics overview |
| `GET` | `/api/v1/management/monitoring/campaigns/{id}/stats` | Campaign stats with hourly data |
| `POST` | `/api/v1/management/models/reload` | Trigger NPU model hot-reload |
| `GET` | `/api/v1/management/audit-log` | Audit log entries |

### Workflows & Approvals

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/workflows/campaigns/{id}/submit` | Submit campaign for approval |
| `POST` | `/api/v1/workflows/approvals/{id}/decide` | Record approval decision |
| `GET` | `/api/v1/workflows/approvals/pending/{user_id}` | List pending approvals |
| `GET` | `/api/v1/workflows/calendar` | Campaign calendar events |

### Brand & Asset Library

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/brand/assets` | List brand assets (filter by type/folder) |
| `POST` | `/api/v1/brand/assets` | Upload asset with versioning |
| `POST` | `/api/v1/brand/validate` | Validate against brand guidelines |

### Budget & Reporting

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/reports/budget/{campaign_id}` | Budget status, pacing, ROAS |
| `POST` | `/api/v1/reports/generate` | Generate report with filters |
| `GET` | `/api/v1/reports/templates` | List report templates |
| `GET` | `/api/v1/reports/scheduled` | List scheduled reports |

### Recommendations & Personalization

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/recommendations/{user_id}` | Personalized recommendations (CF, content-based, trending) |
| `POST` | `/api/v1/recommendations/interactions` | Record user-item interaction |

### Segmentation

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/segments` | List audience segments |
| `POST` | `/api/v1/segments` | Create dynamic segment |

### Suppression

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/suppression/add` | Add to suppression list |
| `GET` | `/api/v1/suppression/check/{identifier}` | Check suppression status |
| `DELETE` | `/api/v1/suppression/{identifier}` | Remove from suppression list |

### OfferFit / RL Engine

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/offerfit/recommend` | Get RL-optimized recommendation |
| `POST` | `/api/v1/offerfit/reward` | Send reward signal |

### Integration Adaptors

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/integrations/tasks/create` | Create Asana/Jira task |
| `GET` | `/api/v1/integrations/dam/search` | Search DAM assets (AEM/Bynder/Aprimo) |
| `POST` | `/api/v1/integrations/bi/push` | Push data to Power BI / export Excel |

### Inference Providers

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/inference/providers` | List available providers |
| `POST` | `/api/v1/inference/predict` | Run inference with batching |

---

## Environment Variables

All configuration is via environment variables prefixed with `CAMPAIGN_EXPRESS__`. Nested fields use double underscores.

### Application

| Variable | Default | Description |
|----------|---------|-------------|
| `CAMPAIGN_EXPRESS__NODE_ID` | `node-01` | Unique node identifier |
| `CAMPAIGN_EXPRESS__AGENTS_PER_NODE` | `20` | Concurrent bid-processing agents |
| `CAMPAIGN_EXPRESS__API__HOST` | `0.0.0.0` | Bind address |
| `CAMPAIGN_EXPRESS__API__HTTP_PORT` | `8080` | REST API port |
| `CAMPAIGN_EXPRESS__API__GRPC_PORT` | `9090` | gRPC port |
| `CAMPAIGN_EXPRESS__METRICS__PORT` | `9091` | Prometheus metrics port |

### Infrastructure

| Variable | Default | Description |
|----------|---------|-------------|
| `CAMPAIGN_EXPRESS__NATS__URLS` | `nats://localhost:4222` | NATS server URLs (comma-separated) |
| `CAMPAIGN_EXPRESS__REDIS__URLS` | `redis://localhost:6379` | Redis cluster URLs |
| `CAMPAIGN_EXPRESS__REDIS__TTL_SECS` | `3600` | Profile cache TTL |
| `CAMPAIGN_EXPRESS__CLICKHOUSE__URL` | `http://localhost:8123` | ClickHouse HTTP endpoint |
| `CAMPAIGN_EXPRESS__CLICKHOUSE__DATABASE` | `campaign_express` | ClickHouse database name |
| `CAMPAIGN_EXPRESS__CLICKHOUSE__BATCH_SIZE` | `10000` | Analytics batch flush size |

### NPU / Inference

| Variable | Default | Description |
|----------|---------|-------------|
| `CAMPAIGN_EXPRESS__NPU__MODEL_PATH` | `/models/colanet.onnx` | Model file path (falls back to synthetic weights) |
| `CAMPAIGN_EXPRESS__NPU__DEVICE` | `cpu` | Device: `cpu` or `xdna` (AMD Ryzen AI) |
| `CAMPAIGN_EXPRESS__NPU__PROVIDER` | `cpu` | Inference provider: `cpu`, `groq`, `inferentia2`, `inferentia3`, `ampere`, `tenstorrent` |
| `CAMPAIGN_EXPRESS__NPU__NUM_THREADS` | `4` | Inference threads |
| `CAMPAIGN_EXPRESS__NPU__BATCH_SIZE` | `64` | Max inference batch size |
| `CAMPAIGN_EXPRESS__NPU__BATCHER_FLUSH_US` | `500` | Nagle-style batch flush interval (microseconds) |
| `CAMPAIGN_EXPRESS__NPU__BATCHER_MAX_ITEMS` | `16` | Max items before batch flush |

### Feature Flags

| Variable | Default | Description |
|----------|---------|-------------|
| `CAMPAIGN_EXPRESS__LOYALTY__ENABLED` | `true` | Enable loyalty program |
| `CAMPAIGN_EXPRESS__DSP__ENABLED` | `false` | Enable DSP integrations |
| `CAMPAIGN_EXPRESS__JOURNEY__ENABLED` | `true` | Enable journey orchestration |
| `CAMPAIGN_EXPRESS__DCO__ENABLED` | `true` | Enable dynamic creative optimization |
| `CAMPAIGN_EXPRESS__CDP__ENABLED` | `false` | Enable CDP syncing |
| `CAMPAIGN_EXPRESS__WORKFLOWS__ENABLED` | `true` | Enable campaign approval workflows |
| `CAMPAIGN_EXPRESS__BRAND__ENABLED` | `true` | Enable brand guidelines enforcement |
| `CAMPAIGN_EXPRESS__RECOMMENDATIONS__ENABLED` | `true` | Enable recommendation engine |
| `CAMPAIGN_EXPRESS__SUPPRESSION__ENABLED` | `true` | Enable global suppression lists |

### Integration Adaptors

| Variable | Default | Description |
|----------|---------|-------------|
| `CAMPAIGN_EXPRESS__ASANA__API_TOKEN` | _(none)_ | Asana personal access token |
| `CAMPAIGN_EXPRESS__JIRA__BASE_URL` | _(none)_ | Jira Cloud instance URL |
| `CAMPAIGN_EXPRESS__JIRA__API_TOKEN` | _(none)_ | Jira API token |
| `CAMPAIGN_EXPRESS__AEM__BASE_URL` | _(none)_ | AEM Assets API endpoint |
| `CAMPAIGN_EXPRESS__BYNDER__BASE_URL` | _(none)_ | Bynder API endpoint |
| `CAMPAIGN_EXPRESS__APRIMO__BASE_URL` | _(none)_ | Aprimo DAM API endpoint |
| `CAMPAIGN_EXPRESS__POWERBI__CLIENT_ID` | _(none)_ | Power BI service principal client ID |
| `CAMPAIGN_EXPRESS__OFFERFIT__API_URL` | _(none)_ | OfferFit API endpoint |
| `CAMPAIGN_EXPRESS__OFFERFIT__API_KEY` | _(none)_ | OfferFit API key |

### Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `campaign_express=info` | Log level filter |

---

## Architecture Overview

### Workspace Crates (26 total)

```
CampaignExpress/
  crates/
    core/                 Campaign types, config, errors, OpenRTB, experimentation,
                          templates, inference provider trait (CoLaNetProvider)
    npu-engine/           CoLaNet SNN model + multi-head inference
      └── backends/       Hardware backends: CPU, Groq, Inferentia, Ampere, Tenstorrent
    agents/               20 Tokio agents per node, NATS queue consumers
      └── batcher.rs      Nagle-style inference batcher (500µs / 16-item flush)
    cache/                Two-tier cache: DashMap L1 -> Redis Cluster L2
    analytics/            Async ClickHouse batch logger (non-blocking mpsc)
    api-server/           Axum REST + Tonic gRPC endpoints
    loyalty/              Starbucks-style loyalty engine (tiers, stars, rewards)
    dsp/                  DSP integration router (Google DV360, TTD, Amazon, Meta)
    channels/             Omnichannel ingest + activation (push, SMS/Twilio, email, in-app)
    management/           Management API, auth, demo data, approval workflows, calendar
    journey/              Journey orchestration engine (state machine, triggers, branching)
    dco/                  Dynamic Creative Optimization + brand guidelines + asset library
    cdp/                  CDP adapters (Salesforce, Adobe, Segment, Tealium, Hightouch)
    platform/             Multi-tenant auth, RBAC, API key management
    billing/              Usage metering, Stripe integration, plan management
    ops/                  SLA tracking, health monitoring, operational metrics
    personalization/      Recommendation engine (CF, content-based, trending, new arrivals)
    segmentation/         Audience segmentation & rule engine
    reporting/            Report builder, budget tracking, scheduled exports (CSV/JSON/Excel)
    integrations/         Task mgmt (Asana, Jira), DAM (AEM, Bynder, Aprimo), BI (Power BI, Excel)
    intelligent-delivery/ Smart delivery optimization + global suppression lists
    rl-engine/            Reinforcement learning + OfferFit connector
    mobile-sdk/           Mobile SDK server-side support
    plugin-marketplace/   Plugin marketplace & extensibility
    sdk-docs/             API reference, guides, examples, search engine
    wasm-edge/            Cloudflare Workers edge stub
  src/
    campaign-express/     Binary entry point (main.rs)
  ui/                     Next.js 14 management frontend
  deploy/
    docker/               Multi-stage Dockerfile + docker-compose
    k8s/                  Kustomize base + overlays + network policies
    helm/                 Helm chart (campaign-express)
    terraform/azure/      AKS, Redis, ACR, Key Vault, ClickHouse IaC
    haproxy/              Ingress load balancer
    monitoring/           Prometheus, AlertManager, Grafana, Tempo, Loki
  docs/                   Architecture & deployment documentation
```

### Local service topology

```
┌──────────────────────────────────────────────────────────────────┐
│  Browser :3001                                                    │
│  ┌──────────────────────────┐                                    │
│  │  Next.js Management UI   │──proxy──┐                          │
│  └──────────────────────────┘         │                          │
│                                       ▼                          │
│  ┌──────────────────────────────────────────────────────┐        │
│  │  Campaign Express  :8080 (HTTP)  :9090 (gRPC)        │        │
│  │  ┌─────────┐ ┌───────────┐ ┌──────────┐ ┌────────┐  │        │
│  │  │ Bid API │ │ Mgmt API  │ │ Channels │ │ Loyalty│  │        │
│  │  └────┬────┘ └─────┬─────┘ └────┬─────┘ └───┬────┘  │        │
│  │       │            │            │            │       │        │
│  │  ┌────▼────────────▼────────────▼────────────▼────┐  │        │
│  │  │            NPU Engine (multi-head SNN)          │  │        │
│  │  │   offer scoring + DCO variant scoring           │  │        │
│  │  └────────────────────────────────────────────────┘  │        │
│  │       │            │            │                    │        │
│  │  ┌────▼───┐  ┌─────▼────┐  ┌───▼──────┐            │        │
│  │  │DashMap │  │ Journey  │  │ DCO      │            │        │
│  │  │L1 Cache│  │ Engine   │  │ Engine   │            │        │
│  │  └────┬───┘  └──────────┘  └──────────┘            │        │
│  └───────┼──────────────────────────────────────────────┘        │
│          │                                                       │
│  ┌───────▼──────┐  ┌───────────┐  ┌────────────────┐            │
│  │ Redis :6379  │  │ NATS :4222│  │ ClickHouse     │            │
│  │ (L2 cache)   │  │ JetStream │  │ :8123 (analytics)│           │
│  └──────────────┘  └───────────┘  └────────────────┘            │
│                                                                  │
│  ┌──────────────┐  ┌───────────┐                                │
│  │Prometheus    │  │ Grafana   │                                │
│  │ :9092        │  │ :3000     │                                │
│  └──────────────┘  └───────────┘                                │
└──────────────────────────────────────────────────────────────────┘
```

### Demo data pre-seeded at startup

| Entity | Count | Examples |
|--------|-------|---------|
| Campaigns | 7 | Holiday Season Push, Back to School, VIP Loyalty Rewards |
| Creatives | 12 | 3 per active campaign (Banner 300x250, Banner 728x90, Native 600x400) |
| Journeys | 5 | Welcome Series, Cart Abandonment, Loyalty Re-engagement |
| DCO Templates | 3 | Holiday Banner DCO, Product Recommendation, Retargeting |
| CDP Platforms | 5 | Salesforce, Adobe, Segment, Tealium, Hightouch |
| Experiments | 4 | Headline A/B, Bid Strategy, DCO vs Static, Channel Priority |
| Approval Rules | 3 | Standard Campaign (min 1), High Budget (min 2), Regulated Channel (min 2) |
| Brand Colors | 6 | Primary (#1B4FDB), Secondary (#FF6B35), Accent (#00D4AA), etc. |
| Font Rules | 3 | Inter body (14-20px), Inter heading (18-72px), Roboto Mono code (12-16px) |
| Report Templates | 5 | Campaign Performance, Channel Breakdown, Audience Insights, Revenue, Executive |

---

## Troubleshooting

### Build fails with missing dependencies

```bash
# Update Rust toolchain
rustup update stable

# Clean and rebuild
cargo clean
cargo build --workspace
```

### Redis connection refused

```bash
# Check if Redis is running
docker ps | grep redis

# Start Redis
docker run -d --name ce-redis -p 6379:6379 redis:7-alpine

# Or use API-only mode (bypasses Redis/NATS/ClickHouse)
cargo run --bin campaign-express -- --api-only
```

### NATS connection error at startup

Use `--api-only` mode during development — it skips NATS agent startup but keeps the full HTTP API working:

```bash
cargo run --bin campaign-express -- --api-only
```

### UI shows "Failed to load dashboard data"

1. Confirm the Rust backend is running on port 8080
2. Check the proxy: `curl http://localhost:3001/api/v1/management/monitoring/overview`
3. Try logging in first: the management API requires auth

### Port already in use

```bash
# Find what's using port 8080
lsof -i :8080

# Use a different port
CAMPAIGN_EXPRESS__API__HTTP_PORT=8081 cargo run --bin campaign-express -- --api-only
```

### ClickHouse analytics not recording

In `--api-only` mode, ClickHouse is not required. The management API uses an in-memory store. For full analytics, start ClickHouse:

```bash
docker run -d --name ce-clickhouse -p 8123:8123 -p 9000:9000 \
  -e CLICKHOUSE_DB=campaign_express \
  clickhouse/clickhouse-server:24-alpine
```

### redis v0.25 future-incompat warning

This is an upstream issue in the `redis` crate, not in our code. It does not affect functionality and will be resolved when `redis` publishes a fix.
