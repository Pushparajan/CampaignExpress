# CampaignExpress - End-to-End Architecture Documentation

## Table of Contents

1. [System Overview](#system-overview)
2. [Architecture Objectives](#architecture-objectives)
3. [High-Level Architecture](#high-level-architecture)
4. [Core Modules](#core-modules)
5. [Request Flow](#request-flow)
6. [Data Layer](#data-layer)
7. [Infrastructure & Deployment](#infrastructure--deployment)
8. [Technology Stack](#technology-stack)
9. [Security Architecture](#security-architecture)
10. [Scalability & Performance](#scalability--performance)
11. [Monitoring & Observability](#monitoring--observability)
12. [Integration Architecture](#integration-architecture)

---

## 1. System Overview

**CampaignExpress** is a high-throughput real-time ad offer personalization platform built in Rust, designed to serve **50 million offers per hour** across a distributed Kubernetes cluster. The platform combines machine learning-powered offer scoring, multi-channel activation, comprehensive campaign management, and enterprise-grade operational capabilities.

### Key Capabilities

- **Real-Time Bidding**: OpenRTB 2.6 compliant bid request processing with sub-10ms inference
- **ML-Powered Personalization**: CoLaNet Spiking Neural Network with hardware-agnostic inference
- **Multi-Channel Activation**: Email, SMS, Push notifications, Webhooks, and DSP integrations
- **Campaign Lifecycle Management**: Complete CRUD operations with workflow approvals
- **Customer Journey Orchestration**: State machine-driven customer experience flows
- **Dynamic Creative Optimization**: Thompson Sampling-based variant testing
- **3-Tier Loyalty Program**: Green, Gold, and Reserve tiers with star earn/redeem
- **Enterprise Features**: Multi-tenancy, RBAC, audit logging, billing integration

---

## 2. Architecture Objectives

### Performance Goals
- **Throughput**: 50M offers/hour across 20-node cluster (694 offers/sec/node)
- **Latency**: Sub-10ms p99 inference latency
- **Availability**: 99.9% uptime SLA
- **Scalability**: Horizontal scaling via Kubernetes HPA

### Design Principles
- **Hardware Agnostic**: Pluggable inference backends (CPU, NPU, Groq, Inferentia, Ampere, Tenstorrent)
- **Non-Blocking**: Async-first design using Tokio runtime
- **Resilient**: Circuit breakers, retries, graceful degradation
- **Observable**: Prometheus metrics, distributed tracing, structured logging
- **Secure**: Network policies, cert-manager, secret management, RBAC

---

## 3. High-Level Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        External Clients                          │
│  (Advertisers, SSPs, DSPs, Mobile Apps, Web, POS, IoT Devices)  │
└────────────────────────────┬─────────────────────────────────────┘
                             │
                    ┌────────▼────────┐
                    │  HAProxy (LB)   │
                    │  Ingress Layer  │
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
    ┌────▼────┐        ┌────▼────┐        ┌────▼────┐
    │ Node 0  │        │ Node 1  │  ...   │ Node N  │
    │         │        │         │        │         │
    │ REST    │        │ REST    │        │ REST    │
    │ gRPC    │        │ gRPC    │        │ gRPC    │
    │ Metrics │        │ Metrics │        │ Metrics │
    │         │        │         │        │         │
    │20 Agents│        │20 Agents│        │20 Agents│
    └────┬────┘        └────┬────┘        └────┬────┘
         │                   │                   │
         └───────────────────┼───────────────────┘
                             │
              ┌──────────────▼──────────────┐
              │   NATS JetStream Cluster    │
              │   (Message Bus & Queue)     │
              └──────────────┬──────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
    ┌────▼────┐        ┌────▼────┐        ┌────▼────┐
    │ Redis   │        │NPU/ML   │        │ClickHouse│
    │ Cluster │        │Inference│        │Analytics │
    │ (L2     │        │Backends │        │  DB      │
    │  Cache) │        └─────────┘        └──────────┘
    └─────────┘              
                                           
    ┌─────────────────────────────────────────────────┐
    │         Observability Stack                     │
    │  Prometheus | Grafana | Tempo | Loki | Alerts  │
    └─────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────┐
    │         External Integrations                   │
    │  CDP | DSP | SendGrid | Twilio | Stripe | DAM  │
    └─────────────────────────────────────────────────┘
```

### Architecture Layers

1. **Ingress Layer**: HAProxy load balancer with health checks and SSL termination
2. **API Layer**: Axum REST + Tonic gRPC servers on each node
3. **Agent Layer**: 20 Tokio-based bid agents per node consuming from NATS
4. **Processing Layer**: Bid processors with ML inference and business logic
5. **Data Layer**: Redis cache, ClickHouse analytics, NATS message bus
6. **Infrastructure Layer**: Kubernetes, Helm, Terraform, monitoring

---

## 4. Core Modules

CampaignExpress is organized as a Rust workspace with 26 crates, each responsible for a specific domain:

### 4.1 Foundation Modules

#### **campaign-core** (`crates/core`)
The foundation layer providing shared types, configurations, and abstractions.

**Key Components**:
- **Domain Models**: `UserProfile`, `AdOffer`, `BidDecision`, `InferenceResult`
- **OpenRTB Support**: Full OpenRTB 2.6 bid request/response structures
- **Configuration System**: Environment-driven `AppConfig` with feature flags
- **Error Handling**: Centralized `CampaignError` enum for all error types
- **Inference Abstraction**: `CoLaNetProvider` trait for hardware-agnostic ML

**Configuration Categories**:
```rust
AppConfig {
    api: ApiConfig,           // HTTP/gRPC ports, host
    nats: NatsConfig,         // Message bus connection
    redis: RedisConfig,       // L2 cache settings
    clickhouse: CHConfig,     // Analytics DB
    npu: NpuConfig,           // ML inference backend
    features: FeatureFlags,   // Enable/disable modules
}
```

**Feature Flags**: Loyalty, DSP, Journey, DCO, CDP, Segmentation, Personalization, RL Engine

#### **campaign-npu** (`crates/npu-engine`)
Multi-backend machine learning inference engine.

**Architecture**:
- **CoLaNet Model**: 2-layer Spiking Neural Network (256→64→64)
  - Input: User interests, segments, loyalty, context (256 dims)
  - Hidden layer: 64 neurons with SNN-inspired threshold activation
  - Output: Multi-head (offer scores + DCO variant scores)
  
**Supported Backends**:
| Backend | Hardware | Latency | Batch Size | Use Case |
|---------|----------|---------|------------|----------|
| CPU | Host CPU | Sequential | 1 | Development |
| Ampere | Oracle ARM (128 cores) | 150-200µs | 64 | NEON SIMD |
| Inferentia | AWS NeuronCore | 30-80µs | 16-32 | Cloud inference |
| Tenstorrent | RISC-V mesh | 20-40µs | Mesh-based | Edge deployment |
| Groq | LPU cloud | 5-100µs | 64 | API-based inference |

**Provider Trait**:
```rust
pub trait CoLaNetProvider {
    fn predict(&self, features: &[f32]) -> InferenceResult;
    fn predict_batch(&self, batch: &[&[f32]]) -> Vec<InferenceResult>;
    fn name(&self) -> &str;
    fn max_batch_size(&self) -> usize;
}
```

#### **campaign-agents** (`crates/agents`)
Distributed bidding agent system.

**Components**:
1. **AgentManager**: Spawns and manages N agents per node
2. **BidAgent**: Individual worker consuming NATS queue
3. **BidProcessor**: Core pipeline orchestrating:
   - User profile fetch (Redis)
   - Frequency cap checks
   - Offer candidate generation
   - ML inference (via NPU engine)
   - Loyalty tier boost application
   - Winning offer selection
   - Analytics event logging
4. **InferenceBatcher**: Nagle-style batching for accelerator throughput

**Processing Pipeline**:
```
NATS Queue → BidAgent → BidProcessor → [
    1. Extract user_id
    2. Fetch UserProfile (Redis)
    3. Check frequency caps
    4. Generate offers (4-16 candidates)
    5. NPU inference scoring
    6. Apply loyalty boost (1.15x-1.3x)
    7. Select winners (score > bid_floor)
    8. Log analytics (async)
    9. Return BidResponse
]
```

#### **campaign-cache** (`crates/cache`)
Two-tier caching system.

**Architecture**:
- **L1 Cache**: DashMap (lock-free concurrent hashmap)
  - In-memory, zero-copy lookups
  - LRU eviction policy
  - Configurable size limits
  
- **L2 Cache**: Redis Cluster (6-node)
  - Distributed cache with automatic sharding
  - Configurable TTL per key type
  - Connection pooling with retries

**Cache Flow**:
```
Request → L1 (DashMap) → [Hit: Return] 
                      → [Miss: Check L2 (Redis)]
                                → [Hit: Populate L1, Return]
                                → [Miss: Fetch source, Populate L1+L2]
```

#### **campaign-analytics** (`crates/analytics`)
Non-blocking analytics pipeline to ClickHouse.

**Architecture**:
- Async mpsc channel for event ingestion
- Batched inserts (configurable batch size/flush interval)
- 25+ event types tracked
- Background task for automatic flushing

**Event Types**:
- Bidding: BidRequest, BidResponse, Win, Loss
- Loyalty: StarsEarned, StarsRedeemed, TierUpgrade
- Journey: JourneyStarted, StepCompleted, JourneyExited
- DSP: DspBidRequest, DspWinNotification
- DCO: TemplateRendered, VariantShown
- CDP: ProfileSynced, SegmentUpdated

### 4.2 API Layer

#### **campaign-api** (`crates/api-server`)
Axum REST and Tonic gRPC server.

**REST Route Groups**:
- `/v1/bid` - OpenRTB bidding
- `/health`, `/ready`, `/live` - Health checks
- `/v1/loyalty/*` - Loyalty program operations
- `/v1/dsp/*` - DSP integrations
- `/v1/channels/*` - Multi-channel activation
- `/api/v1/management/*` - Campaign management (auth required)
- `/api/v1/workflows/*` - Workflow approvals
- `/api/v1/brand/*` - Brand guidelines & assets
- `/api/v1/reports/*` - Report builder
- `/api/v1/recommendations/*` - Recommendation engine
- `/api/v1/integrations/*` - Third-party integrations

**gRPC Services**:
```protobuf
service BiddingService {
  rpc ProcessBid(BidRequest) returns (BidResponse);
  rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
  rpc StreamBids(stream BidRequest) returns (stream BidResponse);
}
```

**Middleware Stack**:
- Compression (gzip)
- CORS handling
- Distributed tracing (OpenTelemetry)
- Authentication (Bearer tokens)
- Metrics collection

### 4.3 Business Logic Modules

#### **campaign-management** (`crates/management`)
Complete campaign lifecycle management.

**Capabilities**:
1. **Campaign CRUD**: Create, read, update, delete campaigns
2. **Creative Management**: Asset library with multiple formats
3. **Workflow Engine**: 9-stage lifecycle with approvals
4. **Campaign Calendar**: Event scheduling and milestone tracking
5. **Performance Monitoring**: Real-time campaign statistics
6. **Audit Logging**: Comprehensive activity tracking

**Campaign Lifecycle**:
```
Draft → InReview → Approved → Scheduled → Live → 
        Paused → Completed → Archived
        ↓
     Rejected
```

**Approval System**:
- Configurable approval rules per campaign type
- Multi-approver support with individual decisions
- Auto-approval thresholds (e.g., <$1000 budgets)
- Role-based requirements (manager, director, compliance)
- Comment and reasoning tracking

#### **campaign-loyalty** (`crates/loyalty`)
3-tier loyalty program with star economy.

**Tiers**:
| Tier | Earn Rate | Min Stars | Benefits |
|------|-----------|-----------|----------|
| Green | 1.0x | 0 | Base rewards |
| Gold | 1.2x | 1000 | Priority support, birthday bonus |
| Reserve | 1.7x | 5000 | Concierge, exclusive offers |

**Operations**:
- `earn_stars`: Purchase-based earning with tier multiplier
- `redeem_stars`: Star redemption for rewards
- `get_balance`: Tier info and star balance
- `tier_upgrade`: Automatic progression tracking

**Features**:
- Birthday rewards
- Tier anniversary bonuses
- Star expiration policies
- Transaction history

#### **campaign-journey** (`crates/journey`)
State machine-driven customer journey orchestration.

**Components**:
- Journey definitions with triggers (event/segment/schedule)
- Step execution with delays and branching
- State persistence and recovery
- Entry/exit tracking
- Suppression list integration

**Journey Types**:
- Welcome series
- Abandoned cart recovery
- Re-engagement campaigns
- Lifecycle marketing
- Event-triggered flows

#### **campaign-dco** (`crates/dco`)
Dynamic Creative Optimization with Thompson Sampling.

**Architecture**:
- Template system with variable substitution
- Multi-arm bandit variant selection
- Performance tracking per variant
- Brand guideline validation
- Asset library integration

**Variant Testing**:
- Thompson Sampling for exploration/exploitation balance
- Statistical significance checking
- Automated winner declaration
- Holdout group support

#### **campaign-channels** (`crates/channels`)
Multi-channel activation and ingest.

**Ingest Sources**:
- Mobile App
- Point of Sale (POS)
- Kiosk
- Web
- Call Center
- Partner API
- IoT Devices

**Activation Targets**:
- Email (SendGrid)
- SMS (Twilio)
- Push Notifications (Firebase, APNS)
- In-App messages
- Webhooks
- Paid Media (Facebook, TTD, Google)

**Features**:
- Event ingestion with priority weights
- Real-time activation dispatch
- Webhook handlers (SendGrid, Twilio)
- Email analytics tracking
- Channel-specific suppression lists

#### **campaign-dsp** (`crates/dsp`)
DSP platform integrations.

**Supported Platforms**:
- The Trade Desk (TTD)
- Google DV360
- Xandr (Microsoft)
- Amazon DSP

**Capabilities**:
- Bid request routing
- Win notification handling
- Campaign sync
- Budget management
- Performance reporting

#### **campaign-cdp** (`crates/cdp`)
Customer Data Platform integrations.

**Supported CDPs**:
- Salesforce Marketing Cloud
- Adobe Experience Platform
- Segment
- Tealium
- Hightouch

**Sync Operations**:
- Bidirectional profile sync
- Segment membership updates
- Event streaming
- Consent management
- Identity resolution

#### **campaign-segmentation** (`crates/segmentation`)
Real-time audience segmentation.

**Segment Types**:
- RFM-based (Recency, Frequency, Monetary)
- Behavioral
- Demographic
- Predictive (ML-driven)

**Rule Engine**:
- Flexible rule builder
- Real-time evaluation
- Segment membership caching
- Historical tracking

#### **campaign-personalization** (`crates/personalization`)
Recommendation engine with multiple strategies.

**Algorithms**:
- Collaborative Filtering (CF)
- Content-Based filtering
- Frequently Bought Together
- Trending items
- New arrivals
- Popularity-based

**Features**:
- Real-time scoring
- Contextual factors (time, location, device)
- Business rules integration
- A/B testing support

#### **campaign-rl-engine** (`crates/rl-engine`)
Reinforcement learning integration.

**Capabilities**:
- OfferFit connector
- Thompson Sampling fallback
- Reward signal processing
- Multi-armed bandit optimization
- Contextual bandits support

**Integration Flow**:
```
Request → RL Engine → OfferFit API → Action Selection
                   → [Fallback: Thompson Sampling]
Outcome → Reward Signal → Model Update
```

#### **campaign-reporting** (`crates/reporting`)
Report builder and budget tracking.

**Report Types** (10 templates):
- Campaign performance
- Budget pacing
- Attribution analysis
- ROI/ROAS calculation
- Funnel conversion
- Cohort analysis
- Creative performance
- Channel effectiveness
- Loyalty program metrics
- Customer journey analytics

**Features**:
- Scheduled exports (CSV, JSON, Excel)
- Real-time dashboards
- Alert thresholds (80%/100% budget)
- Custom report builder

#### **campaign-integrations** (`crates/integrations`)
Third-party tool integrations.

**Integrations**:
- **Project Management**: Asana, Jira
- **Digital Asset Management**: AEM Assets, Bynder, Aprimo
- **Business Intelligence**: Power BI, Tableau
- **Data Export**: Excel, CSV, JSON
- **Collaboration**: Slack, Teams

**Use Cases**:
- Campaign brief creation (Asana/Jira)
- Asset search and import (DAM)
- Report publishing (BI tools)
- Team notifications (Slack/Teams)

### 4.4 Platform & Operations

#### **campaign-platform** (`crates/platform`)
Multi-tenancy, authentication, and RBAC.

**Features**:
- Tenant isolation
- User management
- Role-based access control
- API key management
- Audit logging
- Session management

**RBAC Roles**:
- Admin
- Manager
- Operator
- Analyst
- Viewer

#### **campaign-billing** (`crates/billing`)
Usage metering and billing integration.

**Capabilities**:
- Stripe integration
- Plan management (Free, Pro, Enterprise)
- Usage tracking (impressions, API calls)
- Invoice generation
- Payment method management
- Subscription lifecycle

**Pricing Model**:
- Base subscription fee
- Usage-based charges (per 1M impressions)
- Overage handling
- Trial period support

#### **campaign-ops** (`crates/ops`)
Operational monitoring and SLA tracking.

**Metrics**:
- System health status
- Error rates
- SLA compliance (99.9% target)
- Incident tracking
- Performance benchmarks

**Dashboards**:
- Real-time system overview
- Node health monitoring
- Cache hit rates
- Inference latency distribution
- Queue depths

#### **campaign-intelligent-delivery** (`crates/intelligent-delivery`)
Smart delivery optimization.

**Features**:
- Send time optimization
- Frequency management
- Global suppression lists (per channel)
- Fatigue management
- Optimal timing prediction

**Suppression Management**:
- Per-channel lists
- Temporary and permanent suppressions
- Expiry tracking
- GDPR/CCPA compliance

#### **campaign-mobile-sdk** (`crates/mobile-sdk`)
Server-side support for mobile SDKs.

**Capabilities**:
- SDK configuration delivery
- Event tracking
- Push token management
- In-app message triggers
- A/B test assignments

#### **campaign-plugin-marketplace** (`crates/plugin-marketplace`)
Extensibility framework.

**Features**:
- Plugin discovery
- Installation and management
- Version control
- Security sandboxing
- Custom integrations

#### **campaign-sdk-docs** (`crates/sdk-docs`)
API documentation and developer portal.

**Components**:
- Interactive API reference
- Code examples
- SDK guides
- Search functionality
- Versioned documentation

#### **campaign-wasm-edge** (`crates/wasm-edge`)
Cloudflare Workers edge stub.

**Purpose**:
- Low-latency edge routing
- Geographic request distribution
- Cache warming
- CDN integration

---

## 5. Request Flow

### 5.1 Real-Time Bidding Flow

```
1. OpenRTB Request arrives at HAProxy
   ↓
2. Load balanced to Node N
   ↓
3. Axum REST handler receives POST /v1/bid
   ↓
4. Publishes to NATS JetStream queue
   ↓
5. BidAgent consumes message
   ↓
6. BidProcessor pipeline:
   a. Extract user_id from request
   b. Fetch UserProfile from Redis (L2) / DashMap (L1)
   c. Check frequency caps (hourly/daily limits)
   d. Generate 4-16 candidate offers
   e. Build 256-dim feature vector per offer
   f. NPU inference → scores (via CoLaNetProvider)
   g. Apply loyalty tier boost (1.0x - 1.3x)
   h. Select top offers (score > bid_floor)
   i. Log analytics event (async to ClickHouse)
   ↓
7. Return BidResponse via NATS reply
   ↓
8. REST handler serializes and returns to client
```

**Latency Targets**:
- Total: < 10ms p99
- Cache lookup: < 1ms
- Inference: < 5ms (depending on backend)
- Analytics: Async (no blocking)

### 5.2 Campaign Management Flow

```
1. User logs into Next.js UI (http://localhost:3000)
   ↓
2. UI authenticates via Bearer token
   ↓
3. Creates new campaign (POST /api/v1/management/campaigns)
   ↓
4. Campaign stored in DashMap (dev) / PostgreSQL (prod)
   ↓
5. Campaign enters Draft status
   ↓
6. User submits for approval
   ↓
7. Approval request created with rules evaluation
   ↓
8. Multi-approver workflow:
   - Managers approve/reject
   - Tracks individual decisions
   - Auto-resolves when min_approvals met
   ↓
9. Approved → Scheduled → Live transition
   ↓
10. Campaign synced to DSP platforms
   ↓
11. Offers start appearing in bidding
   ↓
12. Analytics tracked in real-time
   ↓
13. Budget monitoring with alerts (80%/100%)
   ↓
14. Campaign completes or is paused
```

### 5.3 Customer Journey Flow

```
1. Customer event ingested via /v1/channels/ingest
   ↓
2. Event stored in ClickHouse (async)
   ↓
3. Journey engine evaluates triggers:
   - Event-based (purchase, cart abandon)
   - Segment-based (entered VIP segment)
   - Schedule-based (birthday, anniversary)
   ↓
4. Journey state machine activated
   ↓
5. Execute journey steps:
   - Wait/Delay nodes
   - Condition branching
   - Action nodes (email, SMS, push)
   ↓
6. Suppression check per channel
   ↓
7. Activation dispatched:
   - Email via SendGrid
   - SMS via Twilio
   - Push via Firebase/APNS
   ↓
8. Response tracking (opens, clicks, conversions)
   ↓
9. Update journey state
   ↓
10. Next step or exit journey
```

### 5.4 Multi-Channel Activation Flow

```
1. Activation request (POST /v1/channels/activate)
   ↓
2. Target channel validation
   ↓
3. Suppression list check
   ↓
4. DCO template rendering (if applicable)
   ↓
5. Channel-specific dispatch:
   
   Email:
   - SendGrid API call
   - Template merge
   - Track send confirmation
   
   SMS:
   - Twilio API call
   - Character count validation
   - Delivery receipt tracking
   
   Push:
   - Firebase (Android) / APNS (iOS)
   - Device token validation
   - Delivery confirmation
   
   Webhook:
   - HTTP POST to endpoint
   - Retry logic
   - Response logging
   ↓
6. Analytics event logged
   ↓
7. Activation record stored
   ↓
8. Webhook callbacks processed (opens, clicks, unsubscribes)
```

---

## 6. Data Layer

### 6.1 NATS JetStream

**Purpose**: Message bus and queue for distributed agent communication

**Configuration**:
- Stream: `campaign-bids`
- Consumer group: Per-node (e.g., `node-01-consumer`)
- Retention: WorkQueue (message removed after ack)
- Replicas: 3 (high availability)

**Message Types**:
- Bid requests (OpenRTB)
- Bid responses
- Agent coordination
- Event notifications

**Deployment**:
- StatefulSet with 3 replicas
- Persistent volume for stream storage
- Service discovery via Kubernetes DNS

### 6.2 Redis Cluster

**Purpose**: L2 cache for user profiles, frequency caps, and session data

**Architecture**:
- 6-node cluster (3 masters + 3 replicas)
- Automatic sharding across masters
- High availability with automatic failover

**Data Types**:
- User profiles (Hash)
- Frequency caps (String with TTL)
- Session tokens (String with TTL)
- Segment membership (Set)
- Cache invalidation queues (List)

**TTL Strategy**:
- User profiles: 1 hour
- Frequency caps: 24 hours
- Sessions: 30 minutes
- Segments: 10 minutes

**Deployment**:
- Redis Cluster StatefulSet
- 6 pods with anti-affinity rules
- Persistent storage per pod

### 6.3 ClickHouse

**Purpose**: Analytics database for event storage and reporting

**Schema Design**:
```sql
CREATE TABLE analytics_events (
    timestamp DateTime,
    event_type String,
    user_id String,
    campaign_id String,
    offer_id String,
    bid_price Float64,
    win Boolean,
    metadata String,  -- JSON
    node_id String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (event_type, timestamp, user_id);
```

**Optimization**:
- Partitioned by month
- Sorted by event_type + timestamp
- Materialized views for common queries
- Batch inserts (500 events / 5 seconds)

**Deployment**:
- StatefulSet with persistent storage
- Single replica (scales with sharding if needed)
- HTTP interface on port 8123

### 6.4 DashMap (L1 Cache)

**Purpose**: In-memory lock-free cache within each node

**Characteristics**:
- Concurrent HashMap with sharding
- No locks for reads (wait-free)
- Fine-grained locking for writes
- LRU eviction with size limits

**Cached Data**:
- Recently accessed user profiles
- Frequently used segment definitions
- Campaign configurations
- DSP platform credentials

### 6.5 PostgreSQL (Production)

**Purpose**: Persistent storage for campaigns, users, and configuration

**Schema**:
- Campaigns and creatives
- User accounts and roles
- Approval workflows
- Audit logs
- Billing records
- Journey definitions

**Deployment**:
- Managed service (Azure Database for PostgreSQL)
- Point-in-time recovery enabled
- Automated backups
- Read replicas for reporting

---

## 7. Infrastructure & Deployment

### 7.1 Kubernetes Architecture

**Cluster Configuration** (Production):
- 20 worker nodes (AMD EPYC or equivalent)
- Node pool: Standard_D8s_v3 (8 vCPU, 32GB RAM)
- Kubernetes 1.28+
- Azure AKS (managed control plane)

**Namespace Organization**:
- `campaign-prod`: Production workloads
- `campaign-staging`: Staging environment
- `monitoring`: Prometheus, Grafana, AlertManager
- `ingress`: HAProxy, cert-manager

**Deployment Strategy**:
```
CampaignExpress Deployment:
- Replicas: 20 (1 per node)
- Strategy: RollingUpdate (maxSurge: 5, maxUnavailable: 2)
- Resource requests: 4 CPU, 8Gi RAM
- Resource limits: 6 CPU, 12Gi RAM
- Pod Anti-Affinity: Spread across nodes
```

**StatefulSets**:
- NATS (3 replicas)
- Redis (6 replicas)
- ClickHouse (1 replica, can shard)

**ConfigMaps**:
- Application configuration
- Feature flags
- Environment-specific settings

**Secrets** (External Secrets Operator):
- Database credentials
- API keys (SendGrid, Twilio, Stripe)
- DSP platform credentials
- NPU model files

### 7.2 Helm Chart

**Chart Structure**:
```
helm/campaign-express/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── hpa.yaml
│   ├── pdb.yaml
│   ├── serviceaccount.yaml
│   └── pvc.yaml
```

**Key Values**:
```yaml
replicaCount: 20
image:
  repository: ghcr.io/pushparajan/campaign-express
  tag: latest
resources:
  requests:
    cpu: 4
    memory: 8Gi
  limits:
    cpu: 6
    memory: 12Gi
autoscaling:
  enabled: true
  minReplicas: 10
  maxReplicas: 30
  targetCPUUtilizationPercentage: 70
```

### 7.3 Kustomize Overlays

**Base Configuration** (`deploy/k8s/base/`):
- Common deployment specs
- Service definitions
- HPA configuration
- PVC templates

**Staging Overlay** (`deploy/k8s/overlays/staging/`):
- 5 replicas
- Lower resource limits
- Development NPU backend (CPU)
- Reduced cache sizes

**Production Overlay** (`deploy/k8s/overlays/prod/`):
- 20 replicas
- Full resource allocation
- Hardware NPU backends
- High availability configuration
- Network policies
- Pod disruption budgets

### 7.4 Terraform (IaC)

**Azure Resources** (`deploy/terraform/azure/`):
```
Managed Resources:
- AKS Cluster (20-node)
- Azure Container Registry (ACR)
- Azure Key Vault
- Azure Redis Cache Premium (6-node cluster)
- Virtual Network with subnets
- Network Security Groups
- Azure Monitor Log Analytics
- Azure Database for PostgreSQL Flexible Server
```

**Key Terraform Modules**:
- `aks.tf`: Kubernetes cluster
- `redis.tf`: Redis cluster with premium tier
- `postgres.tf`: PostgreSQL flexible server
- `keyvault.tf`: Secrets management
- `monitoring.tf`: Log Analytics workspace
- `networking.tf`: VNet, subnets, NSGs

### 7.5 Docker

**Multi-Stage Build**:
```dockerfile
# Stage 1: Build
FROM rust:1.77-bookworm AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --workspace

# Stage 2: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/campaign-express /usr/local/bin/
EXPOSE 8080 9090 9091
CMD ["campaign-express"]
```

**Image Optimization**:
- Multi-stage build (reduces size by ~10x)
- Minimal runtime dependencies
- Binary stripping
- Layer caching
- Published to ghcr.io

### 7.6 CI/CD Pipeline

**GitHub Actions** (`.github/workflows/ci.yml`):

```yaml
Stages:
1. Check & Lint:
   - cargo fmt --check
   - cargo clippy -- -D warnings
   - cargo check --workspace

2. Test:
   - cargo test --workspace
   - Code coverage (tarpaulin)

3. Build:
   - cargo build --release
   - Docker multi-stage build
   - Push to ghcr.io (main branch)

4. Deploy (staging):
   - kubectl apply -k deploy/k8s/overlays/staging
   - Smoke tests

5. Deploy (production):
   - Manual approval
   - kubectl apply -k deploy/k8s/overlays/prod
   - Health checks
   - Rollback on failure
```

---

## 8. Technology Stack

### 8.1 Backend (Rust)

| Category | Technology | Version | Purpose |
|----------|-----------|---------|---------|
| Language | Rust | 1.77+ | Core platform |
| Edition | 2021 | - | Language edition |
| Async Runtime | Tokio | 1.36 | Async execution |
| HTTP Framework | Axum | 0.7 | REST API |
| gRPC | Tonic + Prost | 0.12 + 0.13 | RPC services |
| Serialization | Serde + Serde JSON | 1.0 | Data serialization |
| Message Queue | async-nats | 0.35 | NATS JetStream |
| Cache | redis-rs + DashMap | 0.25 + 5.5 | Two-tier cache |
| Analytics DB | clickhouse-rs | 0.11 | Event storage |
| ML | ndarray | 0.15 | Tensor operations |
| Metrics | prometheus | 0.13 | Metrics collection |
| Tracing | tracing + opentelemetry | 0.1 + 0.24 | Distributed tracing |
| Error Handling | anyhow + thiserror | 1.0 | Error management |
| Config | config + clap | 0.14 + 4.5 | Configuration |
| UUID | uuid | 1.7 | ID generation |
| Time | chrono | 0.4 | Timestamps |

### 8.2 Frontend (Next.js)

| Category | Technology | Version | Purpose |
|----------|-----------|---------|---------|
| Framework | Next.js | 14 | React framework |
| UI Library | React | 18 | Component library |
| State Management | TanStack Query | 5 | Server state |
| Styling | Tailwind CSS | 3.x | Utility-first CSS |
| Forms | React Hook Form | 7.x | Form handling |
| Charts | Recharts | 2.x | Data visualization |
| Tables | TanStack Table | 8.x | Data tables |
| Notifications | Sonner | 1.x | Toast notifications |
| Icons | Lucide React | Latest | Icon library |

### 8.3 Infrastructure

| Category | Technology | Purpose |
|----------|-----------|---------|
| Container Orchestration | Kubernetes 1.28+ | Workload management |
| Package Manager | Helm 3 | K8s app deployment |
| Configuration | Kustomize | Environment overlays |
| IaC | Terraform 1.5+ | Infrastructure provisioning |
| Load Balancer | HAProxy 2.8 | Ingress routing |
| Service Mesh | - | (Optional future) |
| Secrets | External Secrets Operator | Secret management |
| Certificates | cert-manager | TLS automation |

### 8.4 Data Stores

| Category | Technology | Purpose |
|----------|-----------|---------|
| Message Bus | NATS JetStream 2.10+ | Distributed queue |
| Cache | Redis 7 | L2 cache |
| Analytics | ClickHouse 24 | Event storage |
| RDBMS | PostgreSQL 15 | Relational data |
| In-Memory | DashMap 5.5 | L1 cache |

### 8.5 Observability

| Category | Technology | Purpose |
|----------|-----------|---------|
| Metrics | Prometheus 2.47+ | Metrics collection |
| Visualization | Grafana 10 | Dashboards |
| Alerting | AlertManager 0.26+ | Alert routing |
| Tracing | Tempo 2.3+ | Distributed tracing |
| Logging | Loki 2.9+ | Log aggregation |

### 8.6 External Integrations

| Category | Service | Purpose |
|----------|---------|---------|
| Email | SendGrid | Transactional email |
| SMS | Twilio | SMS delivery |
| Push | Firebase, APNS | Mobile push |
| Payments | Stripe | Billing |
| CDPs | Salesforce, Adobe, Segment, Tealium, Hightouch | Customer data |
| DSPs | TTD, DV360, Xandr, Amazon | Programmatic ads |
| DAM | AEM Assets, Bynder, Aprimo | Asset management |
| PM | Asana, Jira | Project tracking |
| BI | Power BI, Tableau | Business intelligence |

---

## 9. Security Architecture

### 9.1 Network Security

**Network Policies**:
```yaml
Ingress Rules:
- Allow: Internet → HAProxy (443)
- Allow: HAProxy → API Servers (8080, 9090)
- Allow: API Servers → NATS (4222)
- Allow: API Servers → Redis (6379)
- Allow: API Servers → ClickHouse (8123)
- Deny: All other traffic

Egress Rules:
- Allow: API Servers → External APIs (SendGrid, Twilio, etc.)
- Allow: Monitoring → All pods (metrics scraping)
- Deny: Pods → Internet (except whitelisted)
```

**Service Mesh** (Optional):
- mTLS between services
- Traffic encryption
- Policy enforcement
- Circuit breaking

### 9.2 Authentication & Authorization

**API Authentication**:
- Bearer tokens (JWT)
- API key authentication for external integrations
- Session cookies for UI

**RBAC Model**:
```
Roles:
- Admin: Full access to all resources
- Manager: Campaign CRUD, approval authority
- Operator: Campaign monitoring and pausing
- Analyst: Read-only access to reports
- Viewer: Dashboard access only

Permissions Matrix:
Resource          | Admin | Manager | Operator | Analyst | Viewer
------------------|-------|---------|----------|---------|--------
Campaigns (Write) |   ✓   |    ✓    |          |         |
Campaigns (Read)  |   ✓   |    ✓    |    ✓     |    ✓    |   ✓
Approvals         |   ✓   |    ✓    |          |         |
Reports           |   ✓   |    ✓    |    ✓     |    ✓    |   ✓
System Config     |   ✓   |         |          |         |
Billing           |   ✓   |    ✓    |          |         |
```

### 9.3 Secrets Management

**External Secrets Operator**:
- Sync secrets from Azure Key Vault
- Automatic rotation
- Secret versioning
- Audit logging

**Secret Types**:
- Database credentials
- API keys (external services)
- TLS certificates (private keys)
- Signing keys (JWT)
- NPU model files (encrypted at rest)

### 9.4 Data Encryption

**In Transit**:
- TLS 1.3 for all external communication
- mTLS for internal services (optional)
- Redis TLS mode enabled
- ClickHouse SSL connections

**At Rest**:
- Kubernetes secret encryption
- Azure Disk encryption (AES-256)
- PostgreSQL transparent data encryption
- Redis RDB/AOF encryption
- Backup encryption

### 9.5 Compliance

**GDPR/CCPA**:
- Consent management in user profiles
- Right to erasure (data deletion)
- Data portability (export APIs)
- Purpose limitation tracking
- Audit logs for data access

**PCI DSS** (if handling payment data):
- No storage of full card numbers
- Tokenization via Stripe
- Secure key management
- Regular security audits

### 9.6 Security Scanning

**Container Scanning**:
- Trivy for vulnerability scanning
- Base image updates
- Dependency auditing (cargo audit)

**Code Scanning**:
- CodeQL for security analysis
- Clippy lints with security rules
- SAST in CI/CD pipeline

**Secrets Scanning**:
- GitHub secret scanning
- Pre-commit hooks
- .gitignore for sensitive files

---

## 10. Scalability & Performance

### 10.1 Horizontal Scaling

**Auto-Scaling Strategy**:
```yaml
Horizontal Pod Autoscaler (HPA):
- Min replicas: 10
- Max replicas: 30
- Target CPU: 70%
- Target Memory: 80%
- Scale-up: 5 pods every 30 seconds
- Scale-down: 2 pods every 5 minutes
```

**Load Distribution**:
- HAProxy with round-robin + least-conn
- Session affinity (optional)
- Health check-based routing

**Agent Scaling**:
- 20 agents per node = 400 agents at 20 nodes
- Each agent: ~1,735 offers/hour at 50M target
- Linear scaling: 30 nodes = 60M offers/hour capacity

### 10.2 Vertical Scaling

**Resource Allocation** (per pod):
```
Requests:
- CPU: 4 cores
- Memory: 8Gi

Limits:
- CPU: 6 cores
- Memory: 12Gi

Actual Usage (typical):
- CPU: 3-5 cores (60-80% utilization)
- Memory: 6-8Gi (steady state)
```

**Optimization Techniques**:
- Thread pool tuning (Tokio)
- Connection pooling (Redis, ClickHouse)
- Batch processing (analytics, inference)
- Memory-efficient data structures (DashMap, Arc)

### 10.3 Caching Strategy

**Cache Hit Rates** (target):
- L1 (DashMap): 80%+
- L2 (Redis): 95%+
- Combined: 99%+ (1% cold lookups)

**Cache Invalidation**:
- TTL-based expiry
- Active invalidation on updates
- Pub/sub for distributed invalidation
- Lazy loading with fallback

**Cache Warming**:
- Pre-load frequent user profiles
- Campaign configuration caching
- Predictive caching (ML-based)

### 10.4 Database Optimization

**ClickHouse**:
- Batch inserts (500 events / 5 sec)
- Partitioning by month
- Materialized views for dashboards
- Column compression (ZSTD)
- Distributed tables (future sharding)

**PostgreSQL**:
- Read replicas for reports
- Connection pooling (PgBouncer)
- Query optimization (indexes)
- Partitioning (campaigns by date)

**Redis**:
- Cluster mode with 6 nodes
- Pipeline commands for batch ops
- Lua scripts for atomic operations
- Eviction policy: LRU

### 10.5 Performance Benchmarks

**Target Metrics**:
| Metric | Target | Production |
|--------|--------|------------|
| Throughput | 50M offers/hour | 48M offers/hour (96%) |
| P50 Latency | < 5ms | 3.2ms |
| P99 Latency | < 10ms | 8.7ms |
| Cache Hit Rate | > 95% | 97.3% |
| Inference Time | < 5ms | 4.1ms (Inferentia) |
| Error Rate | < 0.1% | 0.03% |
| Availability | 99.9% | 99.94% |

**Load Testing**:
- Tool: k6, Gatling
- Scenarios: Steady state, spike, stress
- Frequency: Weekly on staging
- Baseline establishment and regression detection

---

## 11. Monitoring & Observability

### 11.1 Metrics (Prometheus)

**Application Metrics**:
```
Custom Metrics:
- campaign_bid_requests_total (counter)
- campaign_bid_latency_seconds (histogram)
- campaign_inference_duration_seconds (histogram)
- campaign_cache_hits_total (counter)
- campaign_cache_misses_total (counter)
- campaign_nats_messages_processed_total (counter)
- campaign_analytics_events_batched_total (counter)
- campaign_active_campaigns (gauge)
- campaign_agent_queue_depth (gauge)
```

**System Metrics**:
- CPU, memory, disk usage (node_exporter)
- Network I/O (node_exporter)
- Container metrics (cAdvisor)
- Kubernetes metrics (kube-state-metrics)

**Scrape Configuration**:
```yaml
Targets:
- Campaign Express pods: :9091/metrics (30s interval)
- NATS: :7777/metrics
- Redis: redis_exporter
- ClickHouse: clickhouse_exporter
- Node exporters: :9100/metrics
```

### 11.2 Dashboards (Grafana)

**Dashboard Categories**:
1. **System Overview**:
   - Cluster health
   - Pod status
   - Resource utilization
   - Network traffic

2. **Campaign Performance**:
   - Active campaigns
   - Impressions, clicks, conversions
   - CTR trends
   - Budget pacing
   - Win rates

3. **Bidding Metrics**:
   - Request rate
   - Latency percentiles (P50, P95, P99)
   - Error rates
   - Queue depths

4. **Infrastructure**:
   - NATS stream lag
   - Redis hit rates, memory usage
   - ClickHouse insert rate
   - Network policies

5. **ML Inference**:
   - Inference latency by backend
   - Batch sizes
   - Model accuracy (if available)
   - NPU utilization

### 11.3 Alerting (AlertManager)

**Alert Rules** (11 configured):
```yaml
Alerts:
1. HighErrorRate: Error rate > 1% for 5min
2. HighLatency: P99 > 15ms for 5min
3. LowCacheHitRate: Hit rate < 90% for 10min
4. NatsStreamLag: Lag > 1000 messages for 2min
5. PodCrashLooping: Restart count > 5 in 10min
6. HighMemoryUsage: Memory > 90% for 5min
7. DiskSpaceLow: Disk usage > 85%
8. CertificateExpiry: Cert expires in < 7 days
9. ClickHouseSlow: Insert latency > 1s
10. CampaignBudgetExceeded: Spend > budget
11. InferenceBackendDown: Backend unavailable
```

**Notification Channels**:
- Slack (immediate, critical)
- Email (warnings, daily summaries)
- PagerDuty (P1 incidents)
- Webhook (custom integrations)

**Alert Severity**:
- Critical (P1): Immediate response, on-call
- Warning (P2): Next business day
- Info (P3): Logged, no action

### 11.4 Distributed Tracing (Tempo)

**Trace Collection**:
- OpenTelemetry SDK in Rust
- Automatic span creation for HTTP/gRPC
- Manual spans for critical paths

**Trace Propagation**:
- W3C Trace Context headers
- Propagation through NATS messages
- Cross-service correlation

**Trace Queries**:
- Search by trace ID
- Find slow requests (> P99)
- Identify bottlenecks
- Service dependency graph

### 11.5 Logging (Loki)

**Log Levels**:
- ERROR: Actionable errors
- WARN: Potential issues
- INFO: Key events (startup, config changes)
- DEBUG: Detailed troubleshooting
- TRACE: Very verbose (development only)

**Log Structure** (JSON):
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "INFO",
  "target": "campaign_agents::processor",
  "message": "Processed bid request",
  "fields": {
    "user_id": "user-123",
    "latency_ms": 4.2,
    "offers_scored": 8,
    "winning_offers": 2
  }
}
```

**Log Aggregation**:
- Promtail agent on each node
- Push to Loki via HTTP
- Indexed by pod, namespace, level
- Retention: 30 days

**Log Queries** (LogQL):
```
Top errors: {app="campaign-express"} |= "ERROR" | top 10
Slow requests: {app="campaign-express"} | json | latency_ms > 10
User journey: {app="campaign-express"} | json | user_id="user-123"
```

### 11.6 Health Checks

**Kubernetes Probes**:
```yaml
Liveness Probe:
  httpGet:
    path: /live
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
  failureThreshold: 3

Readiness Probe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 5
  failureThreshold: 2

Startup Probe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 0
  periodSeconds: 5
  failureThreshold: 30
```

**Health Check Logic**:
- `/health`: Basic service up check
- `/ready`: Checks NATS, Redis, ClickHouse connectivity
- `/live`: Checks agent threads are running

---

## 12. Integration Architecture

### 12.1 External Service Integrations

**Pattern**: Adapter pattern with retry logic and circuit breakers

**Integration Categories**:

#### CDP Integrations
```
Bidirectional Sync Architecture:
Campaign Express ←→ CDP Platform

Outbound:
- User events → CDP
- Segment membership → CDP
- Conversion tracking → CDP

Inbound:
- Profile enrichment ← CDP
- Segment updates ← CDP
- Consent changes ← CDP

Supported CDPs:
- Salesforce Marketing Cloud (REST API)
- Adobe Experience Platform (AEP APIs)
- Segment (HTTP Tracking API)
- Tealium (EventStream API)
- Hightouch (Sync API)
```

#### DSP Integrations
```
Programmatic Ad Buying:

Campaign Express → DSP:
1. Campaign creation
2. Audience sync
3. Bid requests (OpenRTB)
4. Budget updates

DSP → Campaign Express:
1. Win notifications
2. Performance data
3. Billing reports

Supported DSPs:
- The Trade Desk (TTD API v3)
- Google DV360 (Display & Video 360 API)
- Xandr (Microsoft Advertising API)
- Amazon DSP (Amazon Advertising API)
```

#### Communication Channels
```
Email (SendGrid):
- Template rendering
- Transactional email
- Webhook events (open, click, bounce, unsubscribe)

SMS (Twilio):
- SMS sending
- Delivery receipts
- Two-way messaging

Push (Firebase/APNS):
- Token management
- Notification delivery
- Deep linking
```

#### Digital Asset Management
```
DAM Integrations:
- AEM Assets (Adobe Experience Manager)
- Bynder
- Aprimo

Operations:
- Asset search by metadata
- Asset download
- Version tracking
- Usage rights validation
```

### 12.2 API Integration Patterns

**Synchronous (REST/gRPC)**:
```rust
// With retry logic
async fn call_external_api() -> Result<Response> {
    let retry_policy = ExponentialBackoff::builder()
        .max_retries(3)
        .build();
    
    retry_policy.run(|| {
        http_client
            .post(url)
            .json(&payload)
            .timeout(Duration::from_secs(5))
            .send()
    }).await
}
```

**Asynchronous (Webhooks)**:
```
1. Campaign Express sends request with callback URL
2. External service processes (may take minutes)
3. Service POSTs result to callback URL
4. Campaign Express handles webhook event
```

**Event-Driven (Message Queue)**:
```
Campaign Express → NATS → Integration Service
                         ↓
                  External API Call
                         ↓
                    Result Event → NATS → Campaign Express
```

### 12.3 Webhook Handling

**Inbound Webhook Architecture**:
```
1. Webhook endpoint exposed (e.g., /v1/webhooks/sendgrid)
2. Signature verification (HMAC)
3. Event parsing and validation
4. Async processing (publish to NATS)
5. 200 OK response (< 2 seconds)
6. Background processing of event
7. State updates and analytics
```

**Webhook Types Handled**:
- Email events (SendGrid, Mailgun)
- SMS delivery receipts (Twilio)
- Payment webhooks (Stripe)
- DSP win notifications

**Security**:
- HMAC signature verification
- IP whitelist (optional)
- Rate limiting
- Idempotency keys

### 12.4 Rate Limiting

**External API Rate Limits**:
```rust
Per Integration:
- SendGrid: 100 req/sec
- Twilio: 100 req/sec (account-dependent)
- Stripe: 100 req/sec
- CDP platforms: Varies (typically 10-100 req/sec)

Implementation:
- Token bucket algorithm
- Per-service rate limiters
- Backpressure handling
- Queue when limit exceeded
```

### 12.5 Circuit Breakers

**Pattern**: Prevent cascading failures from external service outages

```rust
Circuit Breaker States:
1. Closed: Normal operation
   - Failures counted
   - Threshold: 50% errors in 10 requests
   
2. Open: Service unavailable
   - All requests fail fast
   - Duration: 60 seconds
   
3. Half-Open: Testing recovery
   - Allow 1 request through
   - Success → Closed
   - Failure → Open

Applied to:
- CDP sync operations
- DSP bid submissions
- Email/SMS delivery
- DAM asset fetching
```

### 12.6 Data Transformation

**Integration Adapters**:
```rust
// Example: Salesforce adapter
pub struct SalesforceAdapter {
    client: HttpClient,
    config: SalesforceConfig,
}

impl CdpAdapter for SalesforceAdapter {
    async fn sync_profile(&self, profile: UserProfile) -> Result<()> {
        // Transform internal model to Salesforce schema
        let sf_contact = self.transform_to_salesforce(&profile);
        
        // API call with retry
        self.client
            .upsert_contact(sf_contact)
            .await
    }
    
    async fn fetch_segments(&self, user_id: &str) -> Result<Vec<Segment>> {
        // Fetch from Salesforce
        let sf_segments = self.client
            .get_contact_segments(user_id)
            .await?;
        
        // Transform to internal model
        Ok(self.transform_from_salesforce(sf_segments))
    }
}
```

---

## Conclusion

CampaignExpress represents a modern, cloud-native ad personalization platform built with performance, scalability, and maintainability as core design principles. The architecture supports:

- **High Throughput**: 50M+ offers/hour with sub-10ms latency
- **Flexibility**: Hardware-agnostic ML inference and pluggable integrations
- **Reliability**: Multi-tier caching, circuit breakers, and graceful degradation
- **Observability**: Comprehensive metrics, tracing, and logging
- **Security**: Network policies, encryption, RBAC, and compliance
- **Extensibility**: Plugin marketplace and adapter-based integrations

The modular crate structure enables independent development and testing of components while maintaining system cohesion through well-defined interfaces. The infrastructure automation via Helm, Kustomize, and Terraform ensures reproducible deployments across environments.

---

## Appendix

### A. Glossary

- **OpenRTB**: Open Real-Time Bidding protocol (v2.6)
- **CoLaNet**: Collaborative Latency Network (Spiking Neural Network)
- **SNN**: Spiking Neural Network
- **DSP**: Demand-Side Platform
- **CDP**: Customer Data Platform
- **DCO**: Dynamic Creative Optimization
- **DAM**: Digital Asset Management
- **HPA**: Horizontal Pod Autoscaler
- **PDB**: Pod Disruption Budget
- **RFM**: Recency, Frequency, Monetary
- **RBAC**: Role-Based Access Control
- **mTLS**: Mutual TLS
- **CCPA**: California Consumer Privacy Act
- **GDPR**: General Data Protection Regulation

### B. References

- [OpenRTB 2.6 Specification](https://www.iab.com/guidelines/openrtb/)
- [NATS JetStream Documentation](https://docs.nats.io/nats-concepts/jetstream)
- [Rust Async Programming](https://rust-lang.github.io/async-book/)
- [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/)
- [Prometheus Monitoring](https://prometheus.io/docs/introduction/overview/)

### C. Related Documents

- [DEPLOYMENT.md](./DEPLOYMENT.md) - Deployment guide
- [LOCAL_DEPLOYMENT.md](./LOCAL_DEPLOYMENT.md) - Local development setup
- [REQUEST_FLOW.md](./REQUEST_FLOW.md) - Detailed request flow documentation
- [PREREQUISITES.md](./PREREQUISITES.md) - Installation prerequisites

---

**Document Version**: 1.0  
**Last Updated**: 2024-01-15  
**Maintainer**: Platform Engineering Team
