# Campaign Express — Business Requirements Document (BRD)

**Version:** 1.0
**Date:** 2026-02-13
**Status:** Approved
**Owner:** Campaign Express Platform Team

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Business Objectives](#2-business-objectives)
3. [Scope](#3-scope)
4. [Stakeholders](#4-stakeholders)
5. [Functional Requirements](#5-functional-requirements)
   - [FR-RTB: Real-Time Bidding](#fr-rtb-real-time-bidding)
   - [FR-INF: ML Inference](#fr-inf-ml-inference)
   - [FR-REC: Recommendation Engine](#fr-rec-recommendation-engine)
   - [FR-RL: Reinforcement Learning](#fr-rl-reinforcement-learning)
   - [FR-LOY: Loyalty Program](#fr-loy-loyalty-program)
   - [FR-DSP: DSP Integration](#fr-dsp-dsp-integration)
   - [FR-CHN: Multi-Channel Delivery](#fr-chn-multi-channel-delivery)
   - [FR-JRN: Journey Orchestration](#fr-jrn-journey-orchestration)
   - [FR-DCO: Dynamic Creative Optimization](#fr-dco-dynamic-creative-optimization)
   - [FR-BRD: Brand Guidelines & Asset Library](#fr-brd-brand-guidelines--asset-library)
   - [FR-WKF: Campaign Workflows & Approvals](#fr-wkf-campaign-workflows--approvals)
   - [FR-CDP: Customer Data Platform](#fr-cdp-customer-data-platform)
   - [FR-SEG: Audience Segmentation](#fr-seg-audience-segmentation)
   - [FR-PER: Personalization](#fr-per-personalization)
   - [FR-DEL: Intelligent Delivery](#fr-del-intelligent-delivery)
   - [FR-RPT: Reporting & Budget Tracking](#fr-rpt-reporting--budget-tracking)
   - [FR-INT: External Integrations](#fr-int-external-integrations)
   - [FR-EXP: Experimentation](#fr-exp-experimentation)
   - [FR-PLT: Platform & Security](#fr-plt-platform--security)
   - [FR-BIL: Billing & Metering](#fr-bil-billing--metering)
   - [FR-OPS: Operations & SLA](#fr-ops-operations--sla)
   - [FR-SDK: Mobile SDK & Developer Tools](#fr-sdk-mobile-sdk--developer-tools)
   - [FR-PLG: Plugin Marketplace](#fr-plg-plugin-marketplace)
   - [FR-EDG: Edge Computing](#fr-edg-edge-computing)
6. [Non-Functional Requirements](#6-non-functional-requirements)
7. [Infrastructure Requirements](#7-infrastructure-requirements)
8. [Data Requirements](#8-data-requirements)
9. [Integration Matrix](#9-integration-matrix)
10. [Security & Compliance](#10-security--compliance)
11. [Acceptance Criteria](#11-acceptance-criteria)
12. [Glossary](#12-glossary)

---

## 1. Executive Summary

Campaign Express is a high-throughput, real-time ad offer personalization platform designed to serve **50 million offers per hour** across a 20-node Kubernetes cluster. Built entirely in Rust for performance and safety, the platform combines spiking neural network inference (CoLaNet), reinforcement learning (OfferFit), multi-channel delivery, and a full marketer experience (workflows, brand guidelines, budget tracking, reporting) into a single integrated system.

The platform targets feature parity with Braze for marketer experience and OfferFit for ML-driven optimization, while delivering sub-10ms end-to-end bid latency on commodity and accelerator hardware (AMD XDNA, Groq LPU, AWS Inferentia, Oracle Ampere Altra, Tenstorrent RISC-V).

**Key Differentiators:**
- Hardware-agnostic inference via `CoLaNetProvider` trait
- Nagle-style batching for accelerator throughput
- Pure-Rust implementation (no C/C++ runtime dependencies)
- Two-tier cache (lock-free DashMap L1 + Redis Cluster L2)
- Non-blocking analytics pipeline (mpsc → batched ClickHouse)

---

## 2. Business Objectives

| ID | Objective | Success Metric | Target |
|----|-----------|----------------|--------|
| BO-001 | Maximize offer throughput | Offers served per hour | 50M/hour across 20 nodes |
| BO-002 | Minimize inference latency | P99 end-to-end bid latency | < 10 ms |
| BO-003 | Optimize offer relevance | Recommendation click-through rate | > 25% lift vs. random |
| BO-004 | Maximize marketer productivity | Campaign setup time | < 15 minutes for standard campaigns |
| BO-005 | Ensure brand compliance | Off-brand creative launches | Zero tolerance |
| BO-006 | Automate governance | Campaigns launched without approval | Zero (for governed channels) |
| BO-007 | Optimize budget efficiency | Budget utilization accuracy | ±5% of planned pacing |
| BO-008 | Reduce channel fatigue | Message frequency compliance | 100% rule adherence |
| BO-009 | Enable hardware flexibility | Supported inference backends | 6+ hardware targets |
| BO-010 | Achieve platform reliability | Monthly uptime SLA | 99.9% |

---

## 3. Scope

### In Scope

- Real-time bidding engine (OpenRTB 2.6)
- ML inference with hardware-agnostic provider pattern
- 7-strategy recommendation engine
- OfferFit reinforcement learning integration
- 3-tier loyalty program
- Multi-channel delivery (email, push, SMS, in-app, WhatsApp, web push, webhooks)
- Journey orchestration with state machines
- Dynamic creative optimization with Thompson Sampling
- Brand guidelines enforcement and asset library
- 9-stage campaign approval workflows
- CDP integration (Salesforce, Adobe, Segment, Tealium, Hightouch)
- Audience segmentation (behavioral, demographic, predictive, lifecycle, lookalike)
- Intelligent delivery (frequency capping, quiet hours, send-time optimization, suppression)
- Report builder with scheduled exports
- Budget tracking with pacing alerts
- External integrations (Asana, Jira, AEM Assets, Bynder, Aprimo, Power BI, Excel)
- A/B/n experimentation framework
- Multi-tenant platform (auth, RBAC, API keys, audit logging)
- Usage metering and billing (Stripe)
- SLA tracking and incident management
- Mobile SDK support (iOS, Android, React Native, Flutter)
- Plugin marketplace with sandboxing
- Edge computing (Cloudflare Workers)
- Full Kubernetes deployment (AKS, Helm, Kustomize, Terraform)
- Observability stack (Prometheus, AlertManager, Grafana, Tempo, Loki)

### Out of Scope

- Native mobile SDK client libraries (server-side support only)
- Management UI implementation (Next.js shell exists; frontend features deferred)
- Third-party DSP bid execution (simulation layer only)
- ONNX Runtime integration (extension point provided, CDN download not available)

---

## 4. Stakeholders

| Role | Responsibility |
|------|----------------|
| Campaign Managers | Create, configure, and launch campaigns through workflows |
| Brand Managers | Define and enforce brand guidelines across all creatives |
| Data Analysts | Build reports, track budgets, analyze cohort performance |
| ML Engineers | Configure inference providers, tune recommendation strategies |
| Platform Engineers | Deploy, scale, monitor the Kubernetes cluster |
| Compliance Officers | Review approval workflows, manage suppression lists |
| Finance | Track budget utilization, review ROAS/ROI metrics |
| Developers | Integrate via REST/gRPC APIs, build plugins, use mobile SDK |

---

## 5. Functional Requirements

### FR-RTB: Real-Time Bidding

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-RTB-001 | Accept OpenRTB 2.6 bid requests via REST (`POST /v1/bid`) and gRPC | P0 |
| FR-RTB-002 | Support multi-impression bid requests | P0 |
| FR-RTB-003 | Score offers using CoLaNet SNN with 256-dimension feature vectors | P0 |
| FR-RTB-004 | Return bid response with offer_id, bid_price, creative_url within SLA | P0 |
| FR-RTB-005 | Run 20 concurrent Tokio bid agents per node consuming from NATS JetStream | P0 |
| FR-RTB-006 | Apply frequency capping before bid response (per-hour, per-day) | P0 |
| FR-RTB-007 | Apply loyalty tier boost to bid scoring (tier multiplier) | P1 |
| FR-RTB-008 | Log all bid events to analytics pipeline asynchronously | P0 |

**Feature Vector Layout (256 dimensions):**

| Range | Content |
|-------|---------|
| `[0..64)` | User interest embeddings |
| `[64..128)` | Segment one-hot encoding |
| `[128..136)` | Loyalty features (tier, balance, progress, earn_rate) |
| `[136..140)` | Context (recency, frequency cap, device, position) |
| `[140..256)` | Reserved / zero-padded |

---

### FR-INF: ML Inference

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-INF-001 | Implement `CoLaNetProvider` trait for hardware-agnostic inference dispatch | P0 |
| FR-INF-002 | Support CPU backend (wrapping existing NpuEngine, pure-Rust) | P0 |
| FR-INF-003 | Support Groq LPU backend (max batch 64, ~100µs latency) | P1 |
| FR-INF-004 | Support AWS Inferentia 2/3 backend (Neuron SDK, max batch 16/32) | P1 |
| FR-INF-005 | Support Oracle Ampere Altra ARM backend (NEON SIMD, up to 128 cores) | P2 |
| FR-INF-006 | Support Tenstorrent RISC-V Mesh backend (Wormhole/Grayskull, ~40µs) | P2 |
| FR-INF-007 | Implement Nagle-style `InferenceBatcher` that flushes at 16 items or 500µs | P0 |
| FR-INF-008 | Expose `predict()` (single) and `predict_batch()` (batched) methods | P0 |
| FR-INF-009 | Report provider_name, supports_batching, max_batch_size capabilities | P1 |
| FR-INF-010 | Support warm_up() for model pre-loading at startup | P1 |

**CoLaNetProvider Trait:**

```
trait CoLaNetProvider: Send + Sync {
    predict(profile, offer_ids) → Vec<InferenceResult>
    predict_batch(requests) → Vec<Vec<InferenceResult>>
    provider_name() → &str
    supports_batching() → bool
    max_batch_size() → usize
    warm_up() → Result<()>
}
```

**Provider Capabilities:**

| Backend | Max Batch | Typical Latency | Use Case |
|---------|-----------|-----------------|----------|
| CPU | 32 | ~1ms | Development, staging |
| AMD XDNA | 64 | ~200µs | Default production |
| Groq LPU | 64 | ~100µs | Low-latency cloud |
| AWS Inferentia 2 | 16 | ~80µs | AWS deployments |
| AWS Inferentia 3 | 32 | ~50µs | AWS next-gen |
| Oracle Ampere | 128 | ~200µs | ARM-native workloads |
| Tenstorrent | 32 | ~40µs | Edge / custom silicon |

---

### FR-REC: Recommendation Engine

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-REC-001 | Support 7 recommendation strategies with pluggable dispatch | P0 |
| FR-REC-002 | Collaborative filtering via item co-occurrence matrix (DashMap) | P0 |
| FR-REC-003 | Content-based filtering via cosine similarity on item feature vectors | P0 |
| FR-REC-004 | Frequently-bought-together using pair co-occurrence counts | P1 |
| FR-REC-005 | Trending items based on time-windowed interaction velocity | P1 |
| FR-REC-006 | New arrivals ranking by item registration timestamp | P2 |
| FR-REC-007 | Most popular ranking by global interaction count | P0 |
| FR-REC-008 | Recently viewed from user interaction history | P0 |
| FR-REC-009 | Exclude already-interacted items from CF results | P0 |
| FR-REC-010 | Accept RecommendationRequest with strategy, limit, exclude_ids, context | P0 |

**Strategy Dispatch:**

| Strategy | Data Source | Algorithm |
|----------|------------|-----------|
| `MostPopular` | `popularity_scores` DashMap | Global rank by interaction count |
| `RecentlyViewed` | `user_interactions` DashMap | Last N interactions for user |
| `FrequentlyBoughtTogether` | `item_cooccurrence` DashMap | Pair counts → rank |
| `PersonalizedCf` | `item_cooccurrence` DashMap | Sum co-occurrence scores across user history |
| `ContentBased` | `item_features` DashMap | Cosine similarity: dot(user, item) / (‖user‖ × ‖item‖) |
| `Trending` | `interaction_timestamps` DashMap | Recent window count vs. historical baseline |
| `NewArrivals` | `item_created_at` DashMap | Registration recency sort |

---

### FR-RL: Reinforcement Learning

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-RL-001 | Implement multi-armed bandit engine with Thompson Sampling (default), UCB1, and Epsilon-Greedy | P0 |
| FR-RL-002 | Implement contextual bandit engine for feature-aware optimization | P1 |
| FR-RL-003 | Integrate OfferFit API connector (get_recommendation, send_reward, sync_catalog) | P1 |
| FR-RL-004 | Fall back to Thompson Sampling when OfferFit API is unavailable | P0 |
| FR-RL-005 | Track per-variant stats: impressions, conversions, confidence intervals, traffic allocation | P0 |
| FR-RL-006 | Support experiment lifecycle: Draft → Active → Paused → Completed | P1 |
| FR-RL-007 | Implement guardrails engine for variant safety checks | P1 |
| FR-RL-008 | Provide explainability engine for decision transparency | P2 |
| FR-RL-009 | Support holdout groups for causal impact measurement | P1 |

**OfferFit Configuration:**

| Setting | Default | Description |
|---------|---------|-------------|
| API Base URL | `https://api.offerfit.ai/v1` | OfferFit endpoint |
| Timeout | 5,000 ms | API call timeout |
| Objectives | Maximize / Minimize | Optimization direction |

---

### FR-LOY: Loyalty Program

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-LOY-001 | Implement 4-tier loyalty system: Basic → Silver → Gold → Reserve | P0 |
| FR-LOY-002 | Base earn rate: $1 spent = 1 star (ceil of amount_cents / 100) | P0 |
| FR-LOY-003 | Support channel-specific earn multipliers with time-gating | P1 |
| FR-LOY-004 | Referral bonus: 2x multiplier | P1 |
| FR-LOY-005 | Digital purchase bonus: 1.2x multiplier | P2 |
| FR-LOY-006 | Automatic tier evaluation on every earn event | P0 |
| FR-LOY-007 | Star expiry after 180 days of inactivity | P0 |
| FR-LOY-008 | Track lifetime stars, qualifying stars, and redemption history | P0 |
| FR-LOY-009 | Expose earn, redeem, and balance APIs | P0 |
| FR-LOY-010 | Support RLHF reward signal endpoint for model feedback | P1 |

**Tier Thresholds:**

| Tier | Qualifying Stars | Qualifying Period |
|------|-----------------|-------------------|
| Basic | 0 | — |
| Silver | — | Default entry tier |
| Gold | 500 | 12 months |
| Reserve | 2,500 | 12 months |

---

### FR-DSP: DSP Integration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-DSP-001 | Route bid requests to 4 DSP platforms: Google DV360, Amazon DSP, The Trade Desk, Meta Ads | P0 |
| FR-DSP-002 | Per-platform timeout enforcement (default: 200ms) | P0 |
| FR-DSP-003 | Track per-platform spend with real-time counters | P0 |
| FR-DSP-004 | Limit concurrent requests per DSP (default: 1,000) | P1 |
| FR-DSP-005 | Record per-platform metrics: requests, bids, latency, errors | P0 |
| FR-DSP-006 | Support win notification reporting | P1 |

---

### FR-CHN: Multi-Channel Delivery

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-CHN-001 | Email delivery via SendGrid with template support | P0 |
| FR-CHN-002 | Push notifications via WebPush (browser) | P0 |
| FR-CHN-003 | SMS delivery via Twilio with GSM-7/Unicode segment calculation | P0 |
| FR-CHN-004 | In-app messaging with targeting rules | P1 |
| FR-CHN-005 | Content card persistent feed management | P1 |
| FR-CHN-006 | WhatsApp messaging integration | P2 |
| FR-CHN-007 | Webhook outbound dispatch for custom channels | P0 |
| FR-CHN-008 | Omnichannel event ingestion (mobile, POS, kiosk, web) via NATS | P0 |
| FR-CHN-009 | Channel activation dispatcher routing messages to correct provider | P0 |
| FR-CHN-010 | Twilio delivery webhook callback handler for status tracking | P1 |

**SMS Segment Calculation:**
- GSM-7: 160 chars/segment (single), 153 chars/segment (multi)
- Unicode: 70 chars/segment (single), 67 chars/segment (multi)

---

### FR-JRN: Journey Orchestration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-JRN-001 | Define journeys with multi-step state machines | P0 |
| FR-JRN-002 | Support 5 trigger types: event-based, segment-entry, schedule, API, bid-context | P0 |
| FR-JRN-003 | Support step types: action, wait, decision (branch), split, exit | P0 |
| FR-JRN-004 | Support 10 action types: send push/email/SMS/in-app/webhook, suppress bid, update profile, add/remove segment, trigger campaign | P0 |
| FR-JRN-005 | Wait steps with duration or until-event conditions | P1 |
| FR-JRN-006 | Track per-user journey instances with status: Active, Waiting, Paused, Completed, Exited | P0 |
| FR-JRN-007 | Journey lifecycle: Draft → Active → Paused → Completed → Archived | P0 |
| FR-JRN-008 | Version journeys for safe updates | P1 |

**Limits:**

| Limit | Value |
|-------|-------|
| Max active journeys | 100 |
| Max instances per journey | 1,000,000 |
| Evaluation interval | 100 ms |

---

### FR-DCO: Dynamic Creative Optimization

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-DCO-001 | Modular creative assembly from component slots | P0 |
| FR-DCO-002 | Thompson Sampling variant selection with exploration rate | P0 |
| FR-DCO-003 | Variant scoring: CTR (50%) + CVR (30%) + Segment Affinity (20%) | P0 |
| FR-DCO-004 | Track per-variant performance (impressions, clicks, conversions) | P0 |
| FR-DCO-005 | Max 1,000 combinations per template | P1 |
| FR-DCO-006 | Default exploration rate: 10% | P0 |

---

### FR-BRD: Brand Guidelines & Asset Library

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-BRD-001 | Validate creative colors against brand palette | P0 |
| FR-BRD-002 | Validate typography (font family, size range per usage) | P0 |
| FR-BRD-003 | Validate tone of voice against approved keywords and forbidden terms | P1 |
| FR-BRD-004 | Validate logo usage rules (min size, clear space, allowed backgrounds) | P1 |
| FR-BRD-005 | Asset library with versioning (version counter per asset) | P0 |
| FR-BRD-006 | Asset search by name, type, tags, folder | P0 |
| FR-BRD-007 | Virtual folder hierarchy for asset organization | P1 |
| FR-BRD-008 | Support 10 asset types: Image, Video, Logo, Font, ColorPalette, Template, Document, Audio, Icon, Animation | P0 |
| FR-BRD-009 | Asset lifecycle: Active, Archived, PendingReview, Rejected | P0 |
| FR-BRD-010 | Font validation: check if ANY matching rule allows the font/size combination (not all) | P0 |

**Seeded Brand Defaults:**

| Category | Count | Examples |
|----------|-------|---------|
| Colors | 6 | Primary (#1B4FDB), Secondary (#FF6B35), Accent (#00D4AA) |
| Font Rules | 3 | Inter body (14–20px), Inter heading (18–72px), Roboto Mono code (12–16px) |
| Tone Keywords | Approved + Forbidden lists | "innovative", "effortless" vs. "cheap", "guaranteed" |

---

### FR-WKF: Campaign Workflows & Approvals

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-WKF-001 | 9-stage campaign lifecycle: Draft → InReview → Approved → Scheduled → Live → Paused → Completed → Archived (+ Rejected) | P0 |
| FR-WKF-002 | 10 workflow actions: Submit, Approve, Reject, RequestChanges, Schedule, GoLive, Pause, Resume, Complete, Archive | P0 |
| FR-WKF-003 | Configurable approval rules with role, min_approvals, auto_approve_below_budget | P0 |
| FR-WKF-004 | Auto-transition campaign on approval threshold met | P0 |
| FR-WKF-005 | Auto-transition campaign on any rejection | P0 |
| FR-WKF-006 | Record workflow transitions with actor, role, timestamp, comment | P0 |
| FR-WKF-007 | Campaign calendar with date-range queries | P1 |
| FR-WKF-008 | Query pending approvals by approver ID | P0 |
| FR-WKF-009 | Support creative review and legal review flags per rule | P2 |
| FR-WKF-010 | Support channel-specific approval rules | P2 |

**Seeded Approval Rules:**

| Rule | Required Role | Min Approvals | Auto-Approve |
|------|---------------|---------------|--------------|
| Standard Campaign | manager | 1 | Below $1,000 budget |
| High Budget Campaign | director | 2 | Never |
| Regulated Channel | compliance | 2 | Never |

**Stage Transition Matrix:**

| From | Allowed Actions |
|------|-----------------|
| Draft | Submit |
| InReview | Approve, Reject, RequestChanges |
| Approved | Schedule, GoLive |
| Rejected | Submit (resubmit) |
| Scheduled | GoLive, Pause |
| Live | Pause, Complete |
| Paused | Resume, Complete |
| Completed | Archive |

---

### FR-CDP: Customer Data Platform

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-CDP-001 | Bidirectional sync with 5 CDP platforms: Salesforce Data Cloud, Segment, Tealium, mParticle, Zeotap | P0 |
| FR-CDP-002 | Configurable field mappings per platform | P0 |
| FR-CDP-003 | Track sync history: records synced, errors, status, timestamps | P0 |
| FR-CDP-004 | Support sync directions: Inbound, Outbound, Bidirectional | P0 |
| FR-CDP-005 | Transform profiles between platform-specific and canonical formats | P0 |
| FR-CDP-006 | Consent flag tracking per user profile | P1 |

---

### FR-SEG: Audience Segmentation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-SEG-001 | 6 segment types: Behavioral, Demographic, Predictive, Lifecycle, Custom, Lookalike | P0 |
| FR-SEG-002 | Rule-based criteria with AND/OR logical predicate groups | P0 |
| FR-SEG-003 | Dynamic segments with configurable refresh intervals | P0 |
| FR-SEG-004 | Evaluate user against all segments and return matching segment IDs | P0 |
| FR-SEG-005 | Lookalike segments from seed segment with similarity threshold | P2 |
| FR-SEG-006 | Estimated and actual size tracking per segment | P1 |
| FR-SEG-007 | Computed properties engine for derived user attributes | P1 |

---

### FR-PER: Personalization

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-PER-001 | Real-time offer personalization using ML scoring | P0 |
| FR-PER-002 | Template engine with variable substitution | P0 |
| FR-PER-003 | Connected content engine for dynamic content insertion | P1 |
| FR-PER-004 | Product catalog engine for item management | P0 |

---

### FR-DEL: Intelligent Delivery

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-DEL-001 | Frequency capping per channel: PerHour, PerDay, PerWeek, PerMonth, PerCampaign | P0 |
| FR-DEL-002 | 8 capping channels: Push, Email, SMS, InApp, ContentCard, WhatsApp, WebPush, All | P0 |
| FR-DEL-003 | Priority-ordered rule evaluation (first violation blocks send) | P0 |
| FR-DEL-004 | Quiet hours with timezone-aware evaluation and wrap-around support | P1 |
| FR-DEL-005 | Transactional message override for quiet hours | P1 |
| FR-DEL-006 | Send-time optimization: PersonalOptimal (50+ messages), CohortBased, GlobalBest, Fallback (9 AM UTC) | P1 |
| FR-DEL-007 | Global suppression lists with per-channel and global scope | P0 |
| FR-DEL-008 | Suppression entry expiry (time-bound or permanent) | P0 |
| FR-DEL-009 | 6 suppression reasons: UserOptOut, Bounced, Complained, Regulatory, AdminAction, Blocklisted | P0 |
| FR-DEL-010 | Bulk add/remove from suppression lists | P1 |
| FR-DEL-011 | Message throttling for rate limiting | P1 |

**Send-Time Optimization Thresholds:**

| Method | Condition | Confidence |
|--------|-----------|------------|
| PersonalOptimal | User has 50+ messages | 85% |
| CohortBased | User in known cohort | 70% |
| GlobalBest | Default platform avg | 60% |
| Fallback | No data | 9 AM UTC |

---

### FR-RPT: Reporting & Budget Tracking

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-RPT-001 | 10 report types: Campaign Performance, Channel Comparison, Segment Analysis, Revenue Attribution, Funnel Conversion, Cohort Retention, A/B Test Results, Budget Utilization, Engagement Over Time, Custom Query | P0 |
| FR-RPT-002 | 9 aggregation functions: Sum, Average, Count, Min, Max, Median, CountDistinct, P90, P99 | P0 |
| FR-RPT-003 | 10 filter operators: Equals, NotEquals, GreaterThan, LessThan, Between, In, Contains, StartsWith, IsNull, IsNotNull | P0 |
| FR-RPT-004 | Scheduled reports: Daily, Weekly, Biweekly, Monthly, Quarterly, Annually, Once | P1 |
| FR-RPT-005 | Export formats: CSV, JSON, Excel, PDF, HTML | P0 |
| FR-RPT-006 | 5 seeded report templates: Campaign Performance, Channel Breakdown, Audience Insights, Revenue, Executive Summary | P1 |
| FR-RPT-007 | Budget tracking with total and daily budget pacing | P0 |
| FR-RPT-008 | Budget alerts at 80% (NearingLimit) and 100% (BudgetExhausted) thresholds | P0 |
| FR-RPT-009 | Daily budget overspend detection | P0 |
| FR-RPT-010 | ROAS/ROI calculation: revenue/spend, (revenue-spend)/spend × 100 | P0 |
| FR-RPT-011 | CPA, CPC, CPM cost metrics | P1 |
| FR-RPT-012 | Revenue attribution engine with multi-touch models | P1 |
| FR-RPT-013 | Funnel analysis with stage conversion rates | P1 |
| FR-RPT-014 | Cohort retention analysis | P2 |

**Budget Alert Thresholds:**

| Alert | Condition |
|-------|-----------|
| NearingLimit | spent_total ≥ 80% of total_budget |
| OverTotalBudget | spent_total ≥ 100% of total_budget |
| OverDailyBudget | spent_today > daily_budget |
| PacingBehind | Actual spend below expected pacing curve |
| PacingAhead | Actual spend above expected pacing curve |

---

### FR-INT: External Integrations

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-INT-001 | Task management: create/sync tasks in Asana | P1 |
| FR-INT-002 | Task management: create/sync issues in Jira | P1 |
| FR-INT-003 | Campaign review task generation from approval workflows | P1 |
| FR-INT-004 | Task status sync (Todo, InProgress, InReview, Done, Cancelled) | P1 |
| FR-INT-005 | DAM: sync, search, folder listing from AEM Assets | P1 |
| FR-INT-006 | DAM: sync, search from Bynder | P2 |
| FR-INT-007 | DAM: sync, search from Aprimo | P2 |
| FR-INT-008 | DAM auto-sync with configurable interval | P2 |
| FR-INT-009 | BI: push dataset to Power BI with schema management | P1 |
| FR-INT-010 | BI: generate Excel export with multi-sheet support | P1 |
| FR-INT-011 | BI: trigger Power BI dataset refresh on data push | P2 |
| FR-INT-012 | Webhook manager for outbound event dispatch | P0 |
| FR-INT-013 | Integration connector framework for extensibility | P0 |

---

### FR-EXP: Experimentation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-EXP-001 | A/B/n testing with deterministic user assignment | P0 |
| FR-EXP-002 | Statistical significance checking | P0 |
| FR-EXP-003 | Configurable traffic allocation per variant | P0 |
| FR-EXP-004 | Support multiple experiment metrics | P1 |
| FR-EXP-005 | 95% confidence interval calculation per variant | P1 |

---

### FR-PLT: Platform & Security

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-PLT-001 | Multi-tenant architecture with tenant isolation | P0 |
| FR-PLT-002 | Authentication: Local, OAuth2, SAML, API Key providers | P0 |
| FR-PLT-003 | RBAC with 20 granular permissions (Campaign/Creative/Journey/Experiment/DCO/CDP/Analytics/Billing/User/Tenant/System CRUD) | P0 |
| FR-PLT-004 | JWT-based auth tokens with tenant_id, roles, scopes, expiry | P0 |
| FR-PLT-005 | Audit logging for all management actions | P0 |
| FR-PLT-006 | Rate limiting per API key | P1 |
| FR-PLT-007 | Privacy manager for GDPR/CCPA data subject requests | P1 |

---

### FR-BIL: Billing & Metering

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-BIL-001 | Usage metering for 8 meter types: OffersServed, ApiCalls, CampaignsActive, StorageBytes, BandwidthBytes, JourneyExecutions, DcoRenders, CdpSyncs | P0 |
| FR-BIL-002 | Billing providers: Stripe, Chargebee, Manual | P0 |
| FR-BIL-003 | Subscription lifecycle: Active, PastDue, Cancelled, Trialing, Paused | P0 |
| FR-BIL-004 | Pricing plans with included quotas and overage pricing | P0 |
| FR-BIL-005 | Invoice generation with line items per meter | P1 |
| FR-BIL-006 | Payment method management (card last four, expiry, default) | P1 |
| FR-BIL-007 | Tenant onboarding engine for guided setup | P2 |

---

### FR-OPS: Operations & SLA

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-OPS-001 | SLA tracking with configurable targets per service | P0 |
| FR-OPS-002 | Uptime record collection with response time tracking | P0 |
| FR-OPS-003 | Automatic SLA degradation on unhealthy checks | P0 |
| FR-OPS-004 | Incident management with lifecycle tracking | P1 |
| FR-OPS-005 | Backup manager for scheduled data exports | P1 |
| FR-OPS-006 | Status page manager for external communication | P2 |

---

### FR-SDK: Mobile SDK & Developer Tools

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-SDK-001 | SDK configuration management for iOS, Android, React Native, Flutter | P0 |
| FR-SDK-002 | Device registration with push token tracking per platform | P0 |
| FR-SDK-003 | Server-side event ingestion (purchase, view, click) | P0 |
| FR-SDK-004 | App session lifecycle tracking (start, pause, end) | P1 |
| FR-SDK-005 | API reference engine with endpoint documentation | P0 |
| FR-SDK-006 | 20+ code examples in Python, JavaScript, cURL | P1 |
| FR-SDK-007 | 29 step-by-step guides across all difficulty levels | P1 |
| FR-SDK-008 | Full-text documentation search with relevance scoring | P1 |

---

### FR-PLG: Plugin Marketplace

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-PLG-001 | Plugin registry for discovery and installation | P1 |
| FR-PLG-002 | Sandboxed plugin execution environment | P0 |
| FR-PLG-003 | Developer portal for plugin submission and review | P2 |
| FR-PLG-004 | Plugin store with search and filtering | P1 |

---

### FR-EDG: Edge Computing

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-EDG-001 | Cloudflare Workers edge stub for bid preprocessing | P2 |
| FR-EDG-002 | OpenRTB JSON validation at edge before origin routing | P2 |
| FR-EDG-003 | Edge region tagging for latency-aware routing | P2 |

---

## 6. Non-Functional Requirements

### Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-PERF-001 | End-to-end bid latency (P99) | < 10 ms |
| NFR-PERF-002 | Inference latency (P99) | < 5 ms |
| NFR-PERF-003 | Throughput per node | 2.52M offers/hour (700 bids/sec) |
| NFR-PERF-004 | Cluster throughput | 50M offers/hour (20 nodes) |
| NFR-PERF-005 | Cache hit rate (L1 + L2) | > 90% |
| NFR-PERF-006 | Analytics event drop rate | 0% |
| NFR-PERF-007 | DSP response latency | < 200 ms |
| NFR-PERF-008 | API response time (management) | < 500 ms |

### Scalability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-SCALE-001 | Horizontal pod autoscaling | 10–40 pods |
| NFR-SCALE-002 | HPA scale-up | +4 pods per 60s |
| NFR-SCALE-003 | HPA scale-down | −2 pods per 120s (300s stabilization) |
| NFR-SCALE-004 | HPA CPU trigger | 70% utilization |
| NFR-SCALE-005 | HPA Memory trigger | 80% utilization |
| NFR-SCALE-006 | HPA custom metric trigger | 700 bids/sec per pod |
| NFR-SCALE-007 | Pod Disruption Budget | 80% minimum available |

### Reliability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-REL-001 | Monthly uptime | 99.9% |
| NFR-REL-002 | Zero-downtime rolling updates | maxUnavailable: 2, maxSurge: 3 |
| NFR-REL-003 | Redis cluster redundancy | 6 nodes (3 master + 3 replica) |
| NFR-REL-004 | NATS cluster redundancy | 3 nodes (JetStream) |
| NFR-REL-005 | ClickHouse replication | 2 nodes |

### Caching

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-CACHE-001 | L1 cache technology | DashMap (lock-free concurrent) |
| NFR-CACHE-002 | L2 cache technology | Redis Premium Cluster (6 shards) |
| NFR-CACHE-003 | L2 cache TTL | 3,600 seconds |
| NFR-CACHE-004 | L2 connection pool | 32 connections |
| NFR-CACHE-005 | L2 max memory per node | 8 GB |
| NFR-CACHE-006 | L2 eviction policy | allkeys-lru |

### Analytics Pipeline

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-ANLYT-001 | Batch flush size | 10,000 events |
| NFR-ANLYT-002 | Flush interval | 1,000 ms |
| NFR-ANLYT-003 | Data retention | 90 days (TTL on timestamp) |
| NFR-ANLYT-004 | Partitioning | Monthly (toYYYYMM) |
| NFR-ANLYT-005 | Max memory per query | 10 GB |

---

## 7. Infrastructure Requirements

### Kubernetes Cluster (Production)

| Resource | Specification |
|----------|---------------|
| System Node Pool | 3 × Standard_D4s_v5 (4 vCPU, 16 GiB) |
| Bidding Node Pool | Auto-scale (min 10, max 40) × Standard_D16s_v5 with NPU labels |
| ClickHouse Node Pool | 2 × Standard_L8s_v3 (storage-optimized) |
| Total nodes | 25–45 (system + bidding + ClickHouse) |

### Infrastructure Services

| Service | Configuration |
|---------|---------------|
| Redis Premium | 6 shards, 8 GB max memory per node, AOF persistence |
| NATS JetStream | 3-node StatefulSet, 20 GiB per node |
| ClickHouse | 2 nodes, 200 GiB SSD + 1 TB managed disks (PremiumV2_LRS) |
| HAProxy | 3 replicas, 100K max connections, 4 threads |

### Azure Resources (Terraform)

| Resource | Type |
|----------|------|
| Resource Group | azurerm_resource_group |
| VNet + 3 Subnets | azurerm_virtual_network (AKS, Redis, ClickHouse) |
| AKS Cluster | azurerm_kubernetes_cluster |
| Container Registry | azurerm_container_registry (Premium, geo-replicated) |
| Key Vault | azurerm_key_vault |
| Log Analytics | azurerm_log_analytics_workspace |
| Managed Disks | azurerm_managed_disk (PremiumV2_LRS, 1 TB each) |

### Monitoring Stack

| Component | Purpose |
|-----------|---------|
| Prometheus | Metrics collection (pod annotations, static targets) |
| AlertManager | 11 pre-configured alert rules |
| Grafana | Dashboards (9 panels), Tempo queries, Loki queries |
| Tempo | Distributed tracing (OTLP 4317/4318, Jaeger 14268/16686) |
| Loki + Promtail | Centralized logging (DaemonSet on every node) |

### Security & Networking

| Component | Purpose |
|-----------|---------|
| NetworkPolicies | Default-deny ingress + 7 allow rules |
| cert-manager | Let's Encrypt staging + production ClusterIssuers |
| External Secrets Operator | Azure Key Vault → K8s Secret sync |

---

## 8. Data Requirements

### User Profile Schema

| Field | Type | Description |
|-------|------|-------------|
| segments | Vec<u32> | Audience segment membership |
| interests | Vec<f32> | Interest embedding (64 dims) |
| device_type | String | Mobile, desktop, tablet |
| recency_score | f32 | Normalized recency (0.0–1.0) |
| frequency_cap | FrequencyCap | Hourly/daily impression caps |
| loyalty | Option<LoyaltyProfile> | Tier, balance, qualifying stars |

### Analytics Event Schema

| Field | Type | Description |
|-------|------|-------------|
| event_type | String | bid, impression, click, conversion, loyalty, channel |
| timestamp | DateTime | Event timestamp (partitioned monthly) |
| user_id | String | User identifier |
| campaign_id | String | Campaign identifier |
| offer_id | String | Offer identifier |
| properties | JSON | Event-specific metadata |

### Data Retention

| Store | Retention | Backup |
|-------|-----------|--------|
| Redis | Session (evicted on memory pressure) | Periodic RDB to S3 |
| ClickHouse | 90 days (TTL) | Replicated tables + S3 |
| NATS | Ephemeral (replay from sources) | — |
| Models | Persistent (ReadOnlyMany PVC) | Model registry / S3 |

---

## 9. Integration Matrix

| System | Protocol | Direction | Purpose |
|--------|----------|-----------|---------|
| SendGrid | REST API | Outbound | Email delivery |
| Twilio | REST API + Webhooks | Bidirectional | SMS delivery + status |
| Salesforce Data Cloud | REST API | Bidirectional | CDP sync |
| Adobe Experience Platform | REST API | Bidirectional | CDP sync |
| Segment | REST API | Bidirectional | CDP sync |
| Tealium | REST API | Bidirectional | CDP sync |
| Hightouch | REST API | Bidirectional | CDP sync |
| Google DV360 | OpenRTB | Outbound | DSP bid routing |
| The Trade Desk | OpenRTB | Outbound | DSP bid routing |
| Amazon DSP | OpenRTB | Outbound | DSP bid routing |
| Meta Ads | OpenRTB | Outbound | DSP bid routing |
| OfferFit | REST API | Bidirectional | RL recommendations + rewards |
| Asana | REST API | Bidirectional | Task management |
| Jira Cloud | REST API | Bidirectional | Issue tracking |
| AEM Assets | REST API | Bidirectional | DAM sync |
| Bynder | REST API | Bidirectional | DAM sync |
| Aprimo | REST API | Bidirectional | DAM sync |
| Power BI | REST API | Outbound | BI data push |
| Stripe | REST API | Bidirectional | Billing |
| Chargebee | REST API | Bidirectional | Billing (alternative) |
| Cloudflare Workers | WASM | Edge | Bid preprocessing |

---

## 10. Security & Compliance

### Authentication & Authorization

| Requirement | Implementation |
|-------------|----------------|
| Multi-provider auth | Local, OAuth2, SAML, API Key |
| Token format | JWT with tenant_id, roles, scopes, expiry |
| RBAC | 20 granular permissions across 10 resource types |
| API key management | Per-tenant, rate-limited |
| Session management | Token expiry + refresh |

### Data Protection

| Requirement | Implementation |
|-------------|----------------|
| Encryption at rest | Azure Managed Disks (SSE), Redis TLS |
| Encryption in transit | TLS 1.2+ (cert-manager / Let's Encrypt) |
| Secret management | External Secrets Operator → Azure Key Vault |
| Network isolation | Kubernetes NetworkPolicies (default-deny + allow list) |

### Compliance

| Requirement | Implementation |
|-------------|----------------|
| GDPR data subject requests | Privacy manager (export, delete) |
| CCPA compliance | User opt-out handling via suppression lists |
| CAN-SPAM | Global suppression + unsubscribe management |
| Audit trail | All management actions logged with actor, timestamp, details |
| Consent tracking | CDP consent flags per user profile |

---

## 11. Acceptance Criteria

### AC-1: Throughput

- [ ] System sustains 50M offers/hour across 20 nodes for 24 hours
- [ ] No dropped analytics events during sustained load
- [ ] HPA scales correctly between 10–40 pods based on load

### AC-2: Latency

- [ ] P99 end-to-end bid latency < 10 ms at 50M/hour load
- [ ] P99 inference latency < 5 ms on CPU backend
- [ ] DSP responses return within 200 ms timeout

### AC-3: Inference

- [ ] All 6 inference backends (CPU, Groq, Inferentia 2/3, Ampere, Tenstorrent) implement CoLaNetProvider trait
- [ ] InferenceBatcher correctly batches requests at 16 items or 500µs
- [ ] Backend selection configurable via environment variable

### AC-4: Recommendations

- [ ] All 7 strategies return ranked results for seeded test data
- [ ] Collaborative filtering correctly excludes already-interacted items
- [ ] Content-based cosine similarity produces valid scores (0.0–1.0)

### AC-5: Workflows

- [ ] Campaigns follow 9-stage lifecycle with enforced transition rules
- [ ] Approval rules with min_approvals are correctly evaluated
- [ ] Invalid transitions return descriptive error messages

### AC-6: Brand Guidelines

- [ ] Color, font, tone, and logo validation all function independently
- [ ] Font validation passes when ANY matching rule allows the combination
- [ ] Asset library supports versioning and search

### AC-7: Budget & Reporting

- [ ] Budget alerts fire at 80% and 100% thresholds
- [ ] ROAS calculated correctly (revenue / spend)
- [ ] Report builder generates CSV and JSON exports
- [ ] Scheduled reports execute at configured frequencies

### AC-8: Integrations

- [ ] Asana and Jira task creation from campaign workflows
- [ ] DAM sync from AEM Assets, Bynder, Aprimo
- [ ] Power BI data push and Excel export generation

### AC-9: Security

- [ ] NetworkPolicies enforce default-deny ingress
- [ ] cert-manager provisions TLS certificates automatically
- [ ] External Secrets syncs from Azure Key Vault
- [ ] RBAC blocks unauthorized access to protected resources

### AC-10: Observability

- [ ] All 11 AlertManager rules fire on simulated conditions
- [ ] Tempo receives and displays distributed traces
- [ ] Loki + Promtail collects logs from all pods

---

## 12. Glossary

| Term | Definition |
|------|------------|
| **CoLaNet** | Collaborative Lateral Network — spiking neural network architecture for offer scoring |
| **CoLaNetProvider** | Rust trait abstracting hardware-specific inference backends |
| **DCO** | Dynamic Creative Optimization — automated creative assembly and testing |
| **DashMap** | Lock-free concurrent hash map used for L1 cache |
| **DSP** | Demand-Side Platform — programmatic ad buying platform |
| **HPA** | Horizontal Pod Autoscaler — Kubernetes automatic scaling |
| **InferenceBatcher** | Nagle-style buffer that collects inference requests before batch dispatch |
| **JetStream** | NATS persistent messaging layer for at-least-once delivery |
| **Nagle-style** | Batching algorithm inspired by Nagle's TCP algorithm (time + count threshold) |
| **NPU** | Neural Processing Unit — dedicated ML inference accelerator |
| **OfferFit** | Third-party reinforcement learning platform for offer optimization |
| **OpenRTB** | Open Real-Time Bidding protocol (v2.6) for programmatic advertising |
| **PDB** | Pod Disruption Budget — Kubernetes availability guarantee during disruptions |
| **ROAS** | Return on Ad Spend — revenue generated per dollar of ad spend |
| **SNN** | Spiking Neural Network — biologically-inspired neural network model |
| **Thompson Sampling** | Bayesian bandit algorithm using Beta distribution sampling |
| **XDNA** | AMD Ryzen AI NPU architecture |

---

*End of Business Requirements Document*
