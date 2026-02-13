# Campaign Express — Request Flow Architecture

## Table of Contents
1. [System Entry Points](#system-entry-points)
2. [Bid Request Flow (Primary)](#bid-request-flow)
3. [Loyalty Program Flow](#loyalty-program-flow)
4. [DSP Integration Flow](#dsp-integration-flow)
5. [Omnichannel Ingest Flow](#omnichannel-ingest-flow)
6. [Email Activation & Analytics Flow](#email-activation--analytics-flow)
7. [Inference Provider Pattern](#inference-provider-pattern)
8. [Recommendation Engine Flow](#recommendation-engine-flow)
9. [Campaign Workflow & Approval Flow](#campaign-workflow--approval-flow)
10. [Global Suppression Flow](#global-suppression-flow)
11. [OfferFit RL Connector Flow](#offerfit-rl-connector-flow)
12. [Integration Adaptor Flows](#integration-adaptor-flows)
13. [Data Flow Diagram](#data-flow-diagram)
14. [Latency Breakdown](#latency-breakdown)
15. [Key Design Patterns](#key-design-patterns)

---

## System Entry Points

Campaign Express accepts requests through four entry points:

| Entry Point       | Protocol | Port | Path                      | Purpose                     |
|-------------------|----------|------|---------------------------|-----------------------------|
| REST API          | HTTP/1.1 | 8080 | `/v1/bid`                 | OpenRTB bid requests        |
| gRPC API          | H2       | 9090 | `BiddingService/*`        | Streaming bid requests      |
| NATS Agents       | NATS     | 4222 | `campaign-bids.bid-requests` | Queue-based bid processing |
| REST API          | HTTP/1.1 | 8080 | `/v1/loyalty/*`           | Loyalty program operations  |
| REST API          | HTTP/1.1 | 8080 | `/v1/dsp/*`               | DSP routing                 |
| REST API          | HTTP/1.1 | 8080 | `/v1/channels/*`          | Omnichannel ingest/activate |
| Webhook           | HTTP/1.1 | 8080 | `/v1/webhooks/sendgrid`   | Email delivery analytics    |

---

## Bid Request Flow

This is the primary hot path, designed for sub-10ms end-to-end latency.

### Step-by-Step Walkthrough

```
Client (SSP/Exchange)
    │
    │  POST /v1/bid  (OpenRTB 2.6 JSON)
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│  HAProxy Ingress                                              │
│  • Rate limit: 10K req/10s per IP                            │
│  • Balance: leastconn across 20 pods                         │
│  • Health: checks /ready on each pod                         │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  REST Handler (rest.rs)                                       │
│  • Deserialize OpenRTB BidRequest                            │
│  • Assign agent_id: "{node_id}-rest"                         │
│  • Delegate to BidProcessor::process()                       │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  STEP 1: Extract User ID                                      │
│                                                               │
│  user_id = request.user.id                                   │
│         || request.user.buyeruid                             │
│         || "anonymous"                                       │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  STEP 2: Two-Tier Cache Lookup                                │
│                                                               │
│  ┌─────────────────┐     miss     ┌─────────────────┐        │
│  │ L1: DashMap     │ ──────────→  │ L2: Redis       │        │
│  │ (local, <1µs)   │             │ (cluster, ~5ms) │        │
│  │ lock-free       │   ┌─ hit ──→│ TTL: 3600s      │        │
│  └────────┬────────┘   │         └────────┬────────┘        │
│           │ hit         │                  │ miss            │
│           ▼             │                  ▼                 │
│      UserProfile        │         default_profile()          │
│      (with loyalty      │         (Green tier, no history)   │
│       tier data)        │                                    │
│                         │                                    │
│  Metrics: cache.l1.hit, cache.l1.miss, cache.l2.hit         │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  STEP 3: Frequency Cap Check                                  │
│                                                               │
│  if impressions_1h >= max_per_hour:                          │
│      log NoBid event → Analytics (non-blocking)              │
│      return BidResponse::no_bid()                            │
│      metric: bids.frequency_capped                           │
└──────────────────────────┬───────────────────────────────────┘
                           │ (not capped)
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  STEP 4: Generate Candidate Offers                            │
│                                                               │
│  offer_ids = ["offer-0000", "offer-0001", ... "offer-0063"]  │
│  (up to batch_size candidates, typically 64)                 │
│                                                               │
│  Production: replaced by campaign targeting query             │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  STEP 5: NPU Inference (SNN Scoring)                          │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  build_features() → 256-dim feature matrix             │  │
│  │                                                        │  │
│  │  Layout:                                               │  │
│  │  [0..64)    user interests (float)                     │  │
│  │  [64..128)  segment one-hot encoding                   │  │
│  │  [128..136) loyalty features ← NEW                     │  │
│  │             [128] tier (0.0=Green, 0.5=Gold, 1.0=Rsv)  │  │
│  │             [129] stars balance (normalized /5000)      │  │
│  │             [130] tier progress (0.0–1.0)              │  │
│  │             [131] effective earn rate (/2.0)           │  │
│  │             [132] redeem recency (1/(1+days/30))       │  │
│  │             [133] birthday eligible (0 or 1)           │  │
│  │             [134] preferred channel encoding           │  │
│  │             [135] lifetime stars (normalized /50000)   │  │
│  │  [136]      recency score                              │  │
│  │  [137]      frequency cap utilization                  │  │
│  │  [138]      device type encoding                       │  │
│  │  [139]      offer positional encoding                  │  │
│  │  [140..256) reserved (zero-padded for SIMD alignment)  │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
│  CoLaNetModel::infer(features, offer_ids)                    │
│  • Layer 1: input × W1 + bias → SNN threshold activation    │
│  • Layer 2: hidden × W2 + bias → tanh output                │
│  • Returns: Vec<InferenceResult { score, predicted_ctr,      │
│                                   recommended_bid }>          │
│                                                               │
│  metric: inference.latency_us (histogram)                    │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  STEP 6: Loyalty Tier Bid Boost                               │
│                                                               │
│  Reserve tier → recommended_bid × 1.30                       │
│  Gold tier    → recommended_bid × 1.15                       │
│  Green tier   → no change                                    │
│                                                               │
│  Rationale: higher-tier users have higher LTV, so we bid     │
│  more aggressively to win their impressions.                 │
│                                                               │
│  metric: bids.loyalty_boosted                                │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  STEP 7: Winner Selection (per impression)                    │
│                                                               │
│  For each impression in the bid request:                     │
│    winner = results                                          │
│        .filter(|r| r.recommended_bid >= imp.bidfloor)        │
│        .max_by(score)                                        │
│                                                               │
│    if winner found:                                          │
│      create Bid {                                            │
│        id: UUID, impid, price: recommended_bid,              │
│        adid: offer_id, nurl: win notice URL,                 │
│        adm: creative HTML, crid: creative ID, w, h           │
│      }                                                       │
│      log BidResponse event → Analytics (non-blocking)        │
│                                                               │
│    if no winner: metric bids.no_bid                          │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  STEP 8: Return Response                                      │
│                                                               │
│  BidResponse {                                               │
│    id: request_id,                                           │
│    seatbid: [ SeatBid { bid: [Bid], seat: "campaign-express" │
│    bidid: UUID,                                              │
│    cur: "USD"                                                │
│  }                                                           │
│                                                               │
│  metric: bids.total_latency_us (histogram)                   │
│  metric: bids.responded                                      │
└──────────────────────────────────────────────────────────────┘
```

### Alternative Entry: gRPC

The gRPC path supports two modes:

**Unary RPC** (`process_bid`):
```
BidRequestProto { openrtb_json } → deserialize → BidProcessor::process()
                                 → serialize → BidResponseProto { openrtb_json, processing_time_us }
```

**Bidirectional Streaming** (`stream_bids`):
```
Stream<BidRequestProto> → for each message → BidProcessor::process()
                        → send BidResponseProto on response stream
```

### Alternative Entry: NATS Agent

20 agents per pod subscribe to `campaign-bids.bid-requests` as queue group `bid-agents`:

```
NATS message → deserialize OpenRTB → BidProcessor::process()
             → serialize response → publish to NATS reply subject
```

All three paths converge on the same `BidProcessor::process()` pipeline.

---

## Loyalty Program Flow

### Earning Stars

```
POST /v1/loyalty/earn
    │
    ▼
┌───────────────────────────────────────────────┐
│  EarnStarsRequest {                           │
│    user_id, amount_cents, channel,            │
│    transaction_id, is_referral, is_digital    │
│  }                                            │
└────────────────────┬──────────────────────────┘
                     │
                     ▼
┌───────────────────────────────────────────────┐
│  LoyaltyEngine::earn_stars()                  │
│                                               │
│  base_stars = amount_cents / 100 (ceiling)    │
│                                               │
│  rate = tier.earn_multiplier()                │
│       + active promotion bonuses              │
│       + channel bonuses                       │
│       × 2.0 (if referral)                     │
│       × 1.2 (if digital purchase)             │
│                                               │
│  Tier multipliers:                            │
│    Green:   1.0x                              │
│    Gold:    1.2x                              │
│    Reserve: 1.7x                              │
│                                               │
│  stars_earned = base_stars × rate             │
│  Update: balance, qualifying, lifetime        │
│  Check tier upgrade (500→Gold, 2500→Reserve)  │
└────────────────────┬──────────────────────────┘
                     │
                     ▼
  EarnStarsResponse { stars_earned, new_balance, tier, tier_changed }
```

### Redeeming Stars

```
POST /v1/loyalty/redeem
    │
    ▼
┌───────────────────────────────────────────────┐
│  Redemption Tiers:                            │
│                                               │
│   25 Stars  → Customization ($0.80)           │
│   60 Stars  → Discount ($2.00)                │
│  100 Stars  → Basic Item ($3.50)              │
│  200 Stars  → Premium Item ($6.50)            │
│  300 Stars  → Premium Food ($8.00)            │
│  400 Stars  → Merchandise ($15.00)            │
│  500 Stars  → Exclusive Service ($25.00)      │
│              (requires Gold+ tier)            │
│                                               │
│  Checks: sufficient balance, minimum tier     │
│  Deducts stars, increments redemption count   │
└───────────────────────────────────────────────┘
```

### RL Reward Signal (SNN Training Feedback)

```
POST /v1/loyalty/reward-signal
    │
    ▼
  LoyaltyRewardSignal {
    offer_id, user_id, signal_type, tier
  }

  Signal Types:
    Clicked  → reward = +1.0  (interest signal)
    Redeemed → reward = +2.0  (high LTV action)
    Ignored  → reward = -0.1  (refine spike thresholds)

  → Forwarded to SNN training pipeline via NATS (production)
```

### How Loyalty Feeds Into Bid Scoring

```
UserProfile.loyalty (LoyaltyProfile)
         │
         ├─── as_feature_vector() → 8 floats → NPU feature matrix [128..136)
         │    (tier, balance, progress, earn_rate, recency, birthday, channel, lifetime)
         │
         └─── tier → bid boost multiplier in BidProcessor
              (Reserve 1.3x, Gold 1.15x)
```

---

## DSP Integration Flow

### Bid Routing to DSPs

```
POST /v1/dsp/bid
    │
    ▼
┌───────────────────────────────────────────────┐
│  DspRouter::route_bid()                       │
│                                               │
│  Fan out to all enabled DSP clients:          │
│                                               │
│  ┌─────────────┐  ┌─────────────┐            │
│  │ Google DV360 │  │ Amazon DSP  │            │
│  │ Authorized   │  │ API         │            │
│  │ Buyers RTB   │  │             │            │
│  └──────┬──────┘  └──────┬──────┘            │
│         │                │                    │
│  ┌──────┴──────┐  ┌──────┴──────┐            │
│  │ Trade Desk  │  │ Meta Ads    │            │
│  │ OpenPath    │  │ Marketing   │            │
│  │ API         │  │ API         │            │
│  └──────┬──────┘  └──────┴──────┘            │
│         │                │                    │
│         └───────┬────────┘                    │
│                 ▼                             │
│  Aggregate DspBidResponse[] from all DSPs     │
│  Track: latency_ms per platform               │
│  Track: spend via spend_tracker (DashMap)      │
└───────────────────────────────────────────────┘
```

### Win Notification

```
POST /v1/dsp/win { platform, win_price }
    │
    ▼
  DspRouter::record_win()
  → Updates spend tracker (platform → cumulative wins, spend)
  → metric: dsp.wins
```

---

## Omnichannel Ingest Flow

### Event Ingestion

Real-time events flow from multiple sources into the platform:

```
┌─────────┐ ┌─────┐ ┌───────┐ ┌─────┐ ┌────────────┐ ┌───────────┐ ┌─────┐
│Mobile   │ │ POS │ │ Kiosk │ │ Web │ │Call Center │ │Partner API│ │ IoT │
│App      │ │     │ │       │ │     │ │            │ │           │ │     │
└────┬────┘ └──┬──┘ └───┬───┘ └──┬──┘ └─────┬──────┘ └─────┬─────┘ └──┬──┘
     │         │        │        │           │              │          │
     └─────────┴────────┴────────┴───────────┴──────────────┴──────────┘
                                 │
                    NATS Queue: ingest.{source}.>
                                 │
                                 ▼
               ┌─────────────────────────────────┐
               │  POST /v1/channels/ingest        │
               │  IngestProcessor::process_event()│
               ├─────────────────────────────────┤
               │  1. Validate source is enabled   │
               │  2. Extract user_id or device_id │
               │  3. Determine activation trigger │
               │  4. Determine loyalty relevance  │
               └──────────────┬──────────────────┘
                              │
                 ┌────────────┴────────────┐
                 │                         │
          should_activate?          loyalty_relevant?
                 │                         │
                 ▼                         ▼
          Trigger real-time         Route to loyalty
          activation                engine for star
                                    earning
```

**Activation triggers** (high-intent events):
- Cart abandon → immediate retargeting
- Purchase → post-purchase upsell
- Store visit / loyalty swipe / check-in → in-store offer
- Product view / wishlist add → if within 30 min session

**Loyalty-relevant events:**
- Purchase, LoyaltySwipe, CheckIn → star earning

### Activation Dispatch

```
POST /v1/channels/activate
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│  ActivationDispatcher::dispatch(ActivationRequest)            │
│                                                               │
│  Channel selection priority:                                 │
│  1. In-store (kiosk/signage) — if user is physically present │
│  2. User preferred channel — if set                          │
│  3. Push notification — if has push token                    │
│  4. In-app message — always available                        │
│  5. SMS — if has phone                                       │
│  6. Email (SendGrid) — if has email                          │
│  7. Web personalization — fallback                           │
│  8. Paid media (Facebook/TTD/Google/Amazon) — retargeting    │
│                                                               │
│  Dispatches to channel-specific sender:                      │
│  ┌──────────┬──────────┬──────────┬──────────┬────────────┐  │
│  │Push      │SMS       │Email     │In-App    │Paid Media  │  │
│  │(FCM/APNs)│(Twilio)  │(SendGrid)│(Queue)   │(DSP API)   │  │
│  └──────────┴──────────┴──────────┴──────────┴────────────┘  │
│                                                               │
│  Returns: ActivationResult { status, provider_message_id }   │
└──────────────────────────────────────────────────────────────┘
```

---

## Email Activation & Analytics Flow

### Sending Email via SendGrid

```
ActivationRequest { channel: Email }
    │
    ▼
┌──────────────────────────────────────────────┐
│  SendGridProvider::send_email()               │
│                                               │
│  Build SendGrid API payload:                 │
│  {                                           │
│    "personalizations": [{                    │
│      "to": [{"email": "user@example.com"}],  │
│      "custom_args": {                        │
│        "activation_id": "...",               │
│        "user_id": "...",                     │
│        "offer_id": "..."                     │
│      }                                       │
│    }],                                       │
│    "from": { "email": "offers@...",          │
│              "name": "Campaign Express" },   │
│    "subject": "...",                         │
│    "content": [{ "type": "text/html" }],     │
│    "tracking_settings": {                    │
│      "click_tracking": true,                 │
│      "open_tracking": true                   │
│    }                                         │
│  }                                           │
│                                               │
│  POST https://api.sendgrid.com/v3/mail/send  │
│  (production)                                │
│                                               │
│  Initialize analytics tracking               │
│  Return: ActivationResult { status: Queued,  │
│          provider_message_id: "sg-..." }     │
└──────────────────────────────────────────────┘
```

### Email Analytics Webhook Processing

```
SendGrid servers → POST /v1/webhooks/sendgrid
    │
    │  Array of EmailWebhookEvent:
    │  { email, event, sg_message_id, activation_id, url, ... }
    │
    ▼
┌──────────────────────────────────────────────┐
│  SendGridProvider::process_webhook()          │
│                                               │
│  For each event:                             │
│  ┌────────────────┬──────────────────────┐   │
│  │ Event Type     │ Analytics Update     │   │
│  ├────────────────┼──────────────────────┤   │
│  │ Delivered      │ delivered++          │   │
│  │ Open           │ opens++, unique_opens│   │
│  │ Click          │ clicks++, unique_clks│   │
│  │ Bounce         │ bounces++           │   │
│  │ Spam Report    │ spam_reports++      │   │
│  │ Unsubscribe    │ unsubscribes++      │   │
│  │ Processed      │ (no-op)             │   │
│  │ Deferred       │ (no-op, retry)      │   │
│  └────────────────┴──────────────────────┘   │
│                                               │
│  Recalculate rates:                          │
│    open_rate = unique_opens / total_sent     │
│    click_rate = unique_clicks / total_sent   │
│    bounce_rate = bounces / total_sent        │
└──────────────────────────────────────────────┘
```

### Querying Email Analytics

```
GET /v1/channels/email/analytics/{activation_id}
    → EmailAnalytics {
        total_sent, delivered, opens, unique_opens,
        clicks, unique_clicks, bounces, spam_reports,
        unsubscribes, open_rate, click_rate, bounce_rate
      }

GET /v1/channels/email/analytics
    → Vec<EmailAnalytics>  (all activations)
```

---

## Inference Provider Pattern

Campaign Express supports multiple hardware backends via the `CoLaNetProvider` trait.
The inference layer is hardware-agnostic — the same model runs on CPU, Groq LPU,
AWS Inferentia, Oracle Ampere Altra, or Tenstorrent RISC-V mesh.

### Provider Selection

At startup, the system selects a backend based on configuration:

```
Config (CAMPAIGN_EXPRESS__NPU__PROVIDER)
    │
    ▼
┌──────────────────────────────────────────────────┐
│  ProviderType enum dispatch                       │
│                                                   │
│  Cpu        → CpuBackend (wraps NpuEngine)       │
│  Groq       → GroqBackend (LPU, max_batch=64)    │
│  Inferentia2→ InferentiaBackend (Neuron SDK)      │
│  Inferentia3→ InferentiaBackend (Neuron SDK v3)   │
│  Ampere     → AmpereBackend (ARM NEON SIMD)       │
│  Tenstorrent→ TenstorrentBackend (RISC-V Mesh)   │
└──────────────────────────────────────────────────┘
```

### Nagle-Style Batching

The InferenceBatcher collects individual requests and flushes as a batch:

```
Individual Requests
    │ │ │
    ▼ ▼ ▼
┌────────────────────────────────┐
│  InferenceBatcher               │
│                                 │
│  Collect requests until:        │
│  • 16 items accumulated        │
│  • 500µs since first item      │
│                                 │
│  Then flush_batch() →           │
│  provider.predict_batch()       │
└────────────────────────────────┘
```

This maximizes throughput on accelerators that prefer batched work.

---

## Recommendation Engine Flow

The personalization crate provides 5 recommendation strategies:

### Strategy Dispatch

```
GET /v1/recommendations/{user_id}?strategy=cf
    │
    ▼
┌─────────────────────────────────────────────┐
│  RecommendationEngine                        │
│                                               │
│  Strategy selection:                         │
│  ├─ cf               → Collaborative Filtering│
│  │                     (co-occurrence matrix)  │
│  ├─ content_based    → Content-Based           │
│  │                     (cosine similarity)     │
│  ├─ frequently_bought→ Frequently Bought       │
│  │   _together         Together (pair counts)  │
│  ├─ trending         → Trending Items          │
│  │                     (time-windowed counts)  │
│  └─ new_arrivals     → New Arrivals            │
│                        (registration recency)  │
│                                               │
│  All strategies return ranked item IDs        │
└─────────────────────────────────────────────┘
```

### Collaborative Filtering Detail

```
User interactions → item_cooccurrence DashMap
                     │
                     ▼
For user's interacted items:
  sum co-occurrence scores → rank by total score
  exclude already-interacted items
  return top-N
```

### Content-Based Detail

```
User feature vector (from interactions)
        │
        ▼
Cosine similarity against all item feature vectors
  dot(user, item) / (|user| × |item|)
        │
        ▼
Rank by similarity, return top-N
```

---

## Campaign Workflow & Approval Flow

Campaigns follow a 9-stage lifecycle managed by the WorkflowEngine:

```
Draft → InReview → Approved → Scheduled → Live → Completed → Archived
             ↓                                        ↑
         Rejected → Draft (resubmit)              Paused → Live (resume)
```

### Approval Process

```
submit_for_approval(campaign_id, approver_ids)
    │
    ▼
┌────────────────────────────────────────┐
│  Match approval rule                    │
│  (Standard: min 1, High Budget: min 2, │
│   Regulated Channel: min 2)            │
└──────────────┬─────────────────────────┘
                │
                ▼
┌────────────────────────────────────────┐
│  Collect approver decisions             │
│                                         │
│  if approved_count >= min_approvals:   │
│    → status = Approved                 │
│    → auto-transition InReview→Approved │
│                                         │
│  if any rejection:                     │
│    → status = Rejected                 │
│    → auto-transition InReview→Rejected │
└────────────────────────────────────────┘
```

---

## Global Suppression Flow

Before any message send, the suppression list is checked:

```
Activation request
    │
    ▼
┌────────────────────────────────────┐
│  SuppressionList::is_suppressed()  │
│                                     │
│  Check by:                         │
│  • User identifier (email/phone)   │
│  • Channel (email/sms/push/all)    │
│  • Expiry (permanent or time-bound)│
│                                     │
│  If suppressed → skip send         │
│  If not → proceed with activation  │
└────────────────────────────────────┘
```

Supports bulk add/remove and automatic expiry cleanup.

---

## OfferFit RL Connector Flow

The RL engine integrates with OfferFit for reinforcement learning optimization:

```
OfferFitClient
    │
    ├── get_recommendation(user_id, context)
    │   → Call OfferFit API for optimal action
    │   → Fallback: Thompson Sampling if API unavailable
    │
    ├── send_reward(user_id, action_id, reward)
    │   → Report outcome back to OfferFit for model update
    │
    └── sync_catalog(items)
        → Push item catalog to OfferFit for selection
```

---

## Integration Adaptor Flows

### Task Management (Asana / Jira)

```
Campaign approval event
    │
    ▼
TaskManagementAdaptor
    ├── Asana: create_task() → Asana API
    └── Jira:  create_issue() → Jira Cloud API

Status sync: poll adaptor → update campaign status
```

### Digital Asset Management (AEM Assets / Bynder / Aprimo)

```
Creative upload/search
    │
    ▼
DamAdaptor
    ├── AEM Assets: sync, search, folder management
    ├── Bynder: asset search, metadata sync
    └── Aprimo: DAM sync, approval workflow
```

### BI Tools (Power BI / Excel)

```
Report generation
    │
    ▼
BiToolsAdaptor
    ├── Power BI: push dataset → create/refresh report
    └── Excel: generate XLSX export → download
```

---

## Data Flow Diagram

### Full System Data Flow

```
                    ┌─────────────────────────────────┐
                    │         EXTERNAL CLIENTS          │
                    │  (SSPs, Exchanges, Mobile Apps)   │
                    └──────────┬──────────┬────────────┘
                               │          │
                    OpenRTB Bid    Omnichannel Events
                    Requests       (purchase, view, etc.)
                               │          │
                    ┌──────────▼──────────▼────────────┐
                    │       HAProxy Ingress             │
                    │  Rate limit → Route → Balance     │
                    └──────────┬──────────┬────────────┘
                               │          │
                 ┌─────────────┴──┐  ┌────┴──────────────┐
                 │                │  │                    │
                 ▼                ▼  ▼                    ▼
           ┌──────────┐    ┌──────────┐           ┌──────────┐
           │ Bid API  │    │ Channel  │           │ Loyalty  │
           │ /v1/bid  │    │ /v1/ch/* │           │ /v1/ly/* │
           └────┬─────┘    └────┬─────┘           └────┬─────┘
                │               │                      │
                ▼               ▼                      ▼
           ┌─────────┐   ┌──────────┐          ┌──────────┐
           │  Cache   │   │  Ingest  │          │ Loyalty  │
           │ L1+L2    │   │Processor │          │ Engine   │
           │ DashMap  │   │          │          │          │
           │ + Redis  │   └────┬─────┘          └────┬─────┘
           └────┬─────┘        │                     │
                │              ▼                     │
                ▼         ┌──────────┐               │
           ┌─────────┐   │Activation│               │
           │  NPU    │   │Dispatcher│               │
           │ Engine  │   │          │               │
           │ CoLaNet │   └──┬───┬──┘               │
           │  SNN    │      │   │                   │
           └────┬─────┘     │   │                   │
                │           │   ▼                   │
                ▼           │ ┌──────────┐          │
           ┌─────────┐     │ │ SendGrid │          │
           │   Bid   │     │ │ Provider │          │
           │Processor│     │ │  (Email) │          │
           │ + Tier  │     │ └────┬─────┘          │
           │  Boost  │     │      │                │
           └────┬─────┘    │      │ Webhooks       │
                │          │      ▼                │
                │     ┌────┴──────────┐            │
                │     │  DSP Router   │            │
                │     │ DV360│Amazon  │            │
                │     │ TTD  │Meta    │            │
                │     └───────────────┘            │
                │                                  │
                └──────────┬───────────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  Analytics   │
                    │  Logger      │
                    │ (mpsc queue) │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  ClickHouse  │
                    │  (batched    │
                    │   NDJSON)    │
                    └──────────────┘
```

---

## Latency Breakdown

### Bid Request Path (target: <10ms end-to-end)

```
Component              Typical Latency    Notes
─────────────────────  ─────────────────  ──────────────────────
HAProxy routing        ~0.1ms             TCP + header inspection
JSON deserialization   ~0.1ms             serde_json
Cache L1 (DashMap)     <0.01ms            Lock-free, in-process
Cache L2 (Redis)       ~1-5ms             Network RTT (cluster)
Feature encoding       ~0.05ms            256-dim array fill
NPU inference          ~0.5-2ms           2-layer SNN forward pass
Loyalty tier boost     ~0.001ms           Simple branch
Winner selection       ~0.01ms            Linear scan over batch
JSON serialization     ~0.1ms             serde_json
Analytics log          ~0.001ms           Non-blocking mpsc send
─────────────────────  ─────────────────  ──────────────────────
Total (L1 hit)         ~1-3ms
Total (L2 hit)         ~3-8ms
Total (cache miss)     ~2-5ms             Uses default profile
```

### Analytics Pipeline Latency (non-blocking, background)

```
Event → mpsc queue       ~1µs     Non-blocking send
Queue → batch buffer     async    Tokio select loop
Buffer → ClickHouse      ~10ms    HTTP POST, NDJSON batch
Batch size: 10,000 events or 1s flush interval
```

---

## Key Design Patterns

### 1. Two-Tier Cache (L1 DashMap + L2 Redis)

**Why:** Eliminates Redis network RTT for hot profiles. DashMap is lock-free and serves reads in <100ns. L1 has half the TTL of L2 to balance freshness vs. hit rate.

```
Read:  L1 → L2 → default
Write: L2 → L1 (write-through)
```

### 2. Non-Blocking Analytics (mpsc + Batch Writer)

**Why:** Bid processing must not block on analytics I/O. Events are fire-and-forget via a bounded mpsc channel (100K capacity). A background Tokio task batches and flushes to ClickHouse.

```
Hot path: log_event() → channel.send() → return immediately
Cold path: BatchWriter → buffer.push() → flush every 1s or 10K events
```

### 3. Shared BidProcessor via Arc

**Why:** All 20 NATS agents + REST handler + gRPC handler share the same `BidProcessor` instance. The NPU model is wrapped in `Arc<RwLock<CoLaNetModel>>` using parking_lot for concurrent reads and safe hot-reload.

### 4. Loyalty Feature Injection

**Why:** Rather than a separate loyalty scoring pass, loyalty state is encoded directly into the SNN feature vector. This lets the model learn tier-aware patterns end-to-end, while the explicit tier boost provides an interpretable baseline.

### 5. Queue-Based Agent Load Balancing

**Why:** NATS queue groups (`bid-agents`) automatically distribute work across agents. Adding pods adds consumers, scaling throughput linearly without coordination.

### 6. Omnichannel Priority Routing

**Why:** Not all channels are equal. In-store displays have instant conversion potential, while email has hours of latency. The dispatcher uses a priority cascade to select the highest-impact channel available for each user context.
