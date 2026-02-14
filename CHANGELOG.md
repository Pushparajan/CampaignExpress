# Changelog

All notable changes to Campaign Express are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-02-14

### Added

#### Core Platform
- OpenRTB 2.6 compliant bid request processing with sub-10ms inference latency
- CoLaNet Spiking Neural Network inference engine with hardware-agnostic `CoLaNetProvider` trait
- Hardware backends: CPU, Groq LPU, AWS Inferentia 2/3, Oracle Ampere Altra, Tenstorrent RISC-V
- Nagle-style inference batching buffer (500us / 16-item flush) for accelerator throughput
- 20 Tokio-based bid agents per node consuming from NATS JetStream
- Two-tier caching: DashMap L1 (lock-free) -> Redis L2 cluster
- Non-blocking analytics pipeline: mpsc channel -> batched ClickHouse inserts
- Axum REST API server with 76+ endpoints
- Tonic gRPC server with bidirectional streaming support
- Prometheus metrics, health/readiness/liveness probes

#### Campaign Management
- Full campaign CRUD with lifecycle management (draft -> active -> paused -> completed -> archived)
- Creative management with banner, native, video, HTML5, and rich formats
- 9-stage campaign workflow with multi-step role-based approvals
- Unified governance gate combining revision, preflight, policy, and task checks
- Marketer workspace with unified create flow, bulk operations, and operator calendar
- Explainability engine for campaign performance insights
- Audit logging for all management operations

#### Multi-Channel Delivery
- 8 activation channels: email (SendGrid), SMS (Twilio), push, in-app, WhatsApp, web push, content cards, webhooks
- 7 ingest sources: mobile app, POS, kiosk, web, call center, partner API, IoT
- Content Studio with HTML editor, localization engine, and render-time personalization
- SendGrid webhook integration for email delivery tracking

#### Personalization & ML
- Recommendation engine: collaborative filtering, content-based, trending, new arrivals, frequently-bought-together
- Real-time decisioning API with multi-objective optimization (CTR, revenue, LTV)
- Reinforcement learning engine with OfferFit connector and Thompson Sampling fallback
- Dynamic Creative Optimization with variant performance tracking
- Audience segmentation with rule-based real-time evaluation

#### Customer Data
- CDP adapters for Salesforce, Adobe, Segment, Tealium, Hightouch
- Online feature store with TTL staleness alerts and computed features
- Bidirectional sync with external platforms

#### Loyalty Program
- 3-tier program: Green (1.0x), Gold (1.2x, 500 Stars/12mo), Reserve (1.7x, 2500 Stars/12mo)
- Star earning and redemption APIs
- RL reward signal integration for SNN training

#### DSP Integration
- Multi-platform routing: The Trade Desk, Google DV360, Xandr, Amazon DSP
- Paid media audience proxy with incremental sync and match-rate estimation
- Win notification tracking and budget pacing

#### Journey Orchestration
- State machine-driven customer journeys
- Event, segment, and schedule-based triggers
- Branching, delays, and conditional logic

#### Brand & Creative
- Brand guidelines enforcement: color palette, typography, tone-of-voice, logo usage
- Versioned asset library with folder management and search
- Creative export contracts with IAB placement validation and lineage tracking

#### Reporting & Analytics
- Report builder with 10 report types and 5 templates
- Scheduled exports (CSV, JSON, Excel)
- Budget tracking with pacing alerts (80%/100%/daily) and ROAS/ROI calculation
- Unified measurement: standardized cross-channel events, breakdown reporting, experiment lift

#### Platform & Enterprise
- Multi-tenant architecture with tenant isolation
- RBAC with role management and API key generation
- User management with invitations and access control
- Usage metering with Stripe billing integration
- Plan management (starter, professional, enterprise)
- SLA tracking, incident management, health monitoring

#### Integrations
- Project management: Asana, Jira
- Digital asset management: AEM Assets, Bynder, Aprimo
- Business intelligence: Power BI, Excel
- Connector capability registry with health monitoring and 12-test certification harness

#### Experimentation
- A/B/n testing with deterministic assignment
- Statistical significance checking
- Experiment lifecycle management

#### Infrastructure
- Kubernetes deployment with Kustomize (base + staging/prod overlays)
- Helm chart for package management
- Multi-stage Docker builds with development hot-reload
- HAProxy load balancer with rate limiting (10,000 req/10s)
- NATS JetStream cluster (3-node StatefulSet)
- Redis 7 cluster (6-node, 3 masters + 3 replicas)
- ClickHouse analytics database (2-node StatefulSet)
- Terraform IaC for Azure (AKS, Redis Premium, ACR, Key Vault)
- AWS alternative deployment (EKS, ElastiCache, ECR)

#### Observability
- Prometheus with 5 scrape targets and 11 alert rules
- AlertManager with PagerDuty and Slack integrations
- Grafana dashboards for bid latency, throughput, and infrastructure
- Tempo distributed tracing (OTLP + Jaeger receivers)
- Loki + Promtail log aggregation (7-day retention)

#### Security
- Kubernetes NetworkPolicies (default deny-all + 7 allow rules)
- cert-manager with Let's Encrypt (production + staging issuers)
- External Secrets Operator for Azure Key Vault integration
- AMD XDNA NPU device plugin DaemonSet

#### Developer Experience
- SDK documentation server with API reference, guides, examples, and search
- Mobile SDK server-side support
- Plugin marketplace and extensibility framework
- Cloudflare Workers edge stub (WASM)
- Blueprint starter templates (Next.js web app, React Native mobile app)

#### Documentation
- End-to-end architecture documentation (ARCHITECTURE.md)
- 22 detailed request flow scenarios (REQUEST_FLOW.md)
- Business Requirements Document v2.0 (BRD.md)
- Production and local deployment guides
- Role-specific onboarding guides (Rust, ML, SRE, Marketer)
- Comprehensive test strategy and 20-category manual test cases
- SaaS operations staffing guide
- Infrastructure reference documentation
- API reference with full endpoint specifications
- Contributing guidelines and security policy
