# Campaign Express — API Reference

**Version:** 0.1.0
**Base URL:** `http://localhost:8080`
**gRPC:** `localhost:9090`
**Metrics:** `http://localhost:9091/metrics`

---

## Table of Contents

1. [Authentication](#1-authentication)
2. [Core Bidding](#2-core-bidding)
3. [Health & Operations](#3-health--operations)
4. [Loyalty Program](#4-loyalty-program)
5. [DSP Integration](#5-dsp-integration)
6. [Omnichannel](#6-omnichannel)
7. [Campaign Management](#7-campaign-management)
8. [Creative Management](#8-creative-management)
9. [Journey Orchestration](#9-journey-orchestration)
10. [DCO Templates](#10-dco-templates)
11. [CDP Integration](#11-cdp-integration)
12. [Experiments](#12-experiments)
13. [Platform & Tenants](#13-platform--tenants)
14. [User Management](#14-user-management)
15. [RBAC & Compliance](#15-rbac--compliance)
16. [Billing](#16-billing)
17. [Operations & SLA](#17-operations--sla)
18. [Monitoring](#18-monitoring)
19. [Workflows & Approvals](#19-workflows--approvals)
20. [Brand & Assets](#20-brand--assets)
21. [Reporting](#21-reporting)
22. [Recommendations](#22-recommendations)
23. [Integrations](#23-integrations)
24. [Inference Providers](#24-inference-providers)
25. [Segmentation](#25-segmentation)
26. [Intelligent Delivery](#26-intelligent-delivery)
27. [gRPC API](#27-grpc-api)
28. [Error Handling](#28-error-handling)

---

## 1. Authentication

Management endpoints under `/api/v1/management/` require a Bearer token obtained via login.

### POST /api/v1/management/auth/login

Authenticate and obtain a Bearer token.

**Request:**
```json
{
  "username": "string",
  "password": "string"
}
```

**Response (200):**
```json
{
  "token": "string",
  "user": "string",
  "expires_at": "2026-02-15T00:00:00Z"
}
```

**Response (401):**
```json
{
  "error": "unauthorized",
  "message": "Invalid credentials"
}
```

**Usage:**
```bash
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/management/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"campaign2024"}' | jq -r '.token')

curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/management/campaigns
```

---

## 2. Core Bidding

### POST /v1/bid

Process an OpenRTB 2.6 bid request. This is the high-throughput endpoint targeting 50M offers/hour.

**Auth:** None

**Request:**
```json
{
  "id": "req-001",
  "imp": [
    {
      "id": "imp-1",
      "bidfloor": 0.50,
      "banner": { "w": 300, "h": 250 }
    }
  ],
  "site": { "domain": "example.com" },
  "app": null,
  "device": { "ua": "Mozilla/5.0...", "ip": "203.0.113.1" },
  "user": { "id": "user-123" },
  "tmax": 100,
  "at": 1,
  "cur": ["USD"],
  "ext": {}
}
```

**Response (200):**
```json
{
  "id": "req-001",
  "seatbid": [
    {
      "bid": [
        {
          "id": "bid-001",
          "impid": "imp-1",
          "price": 1.25,
          "adid": "creative-456",
          "nurl": "https://example.com/win",
          "adm": "<div>...</div>",
          "w": 300,
          "h": 250
        }
      ],
      "seat": "campaign-express"
    }
  ]
}
```

**Metrics:** `api.errors` (counter, on failure)

---

## 3. Health & Operations

### GET /health

Detailed health status with node info and uptime.

**Auth:** None

**Response (200):**
```json
{
  "status": "healthy",
  "node_id": "node-01",
  "uptime_secs": 3600
}
```

### GET /ready

Kubernetes readiness probe.

**Auth:** None | **Response:** 200 OK or 503 Service Unavailable

### GET /live

Kubernetes liveness probe.

**Auth:** None | **Response:** 200 OK

---

## 4. Loyalty Program

### POST /v1/loyalty/earn

Earn stars from a purchase. Earn rate multiplied by tier (Green 1.0x, Gold 1.2x, Reserve 1.7x).

**Auth:** None

**Request:**
```json
{
  "user_id": "user-123",
  "amount": 25.50,
  "category": "coffee",
  "channel": "mobile_app"
}
```

**Response (200):**
```json
{
  "user_id": "user-123",
  "stars_earned": 51,
  "new_balance": 350,
  "tier_after": "gold"
}
```

**Metrics:** `loyalty.api.earn_stars`

### POST /v1/loyalty/redeem

Redeem stars for a reward.

**Auth:** None

**Request:**
```json
{
  "user_id": "user-123",
  "stars_to_spend": 100,
  "reward_id": "free-drink"
}
```

**Response (200):**
```json
{
  "success": true,
  "stars_redeemed": 100,
  "new_balance": 250,
  "reward_value": 5.50
}
```

**Metrics:** `loyalty.api.redemptions` (on success)

### GET /v1/loyalty/balance/{user_id}

Get loyalty balance, tier, and progress.

**Auth:** None

**Response (200):**
```json
{
  "user_id": "user-123",
  "tier": "gold",
  "stars_balance": 250,
  "stars_qualifying": 750,
  "tier_progress": 0.30,
  "effective_earn_rate": 1.2,
  "lifetime_stars": 5000,
  "total_redemptions": 15
}
```

### POST /v1/loyalty/reward-signal

Record an RL reward signal for SNN training (non-blocking via NATS).

**Auth:** None

**Request:**
```json
{
  "signal_type": "positive",
  "user_id": "user-123",
  "metadata": { "offer_id": "offer-456", "action": "clicked" }
}
```

**Response:** 202 Accepted

**Metrics:** `loyalty.reward_signals` (with `type` tag)

---

## 5. DSP Integration

### POST /v1/dsp/bid

Route a bid request to multiple DSP platforms.

**Auth:** None

**Request:**
```json
{
  "request_id": "req-001",
  "openrtb_json": "{...serialized OpenRTB...}",
  "impression_ids": ["imp-1", "imp-2"]
}
```

**Response (200):**
```json
{
  "request_id": "req-001",
  "dsp_responses": 4,
  "bids_received": 3,
  "responses": [
    {
      "dsp_platform": "google_dv360",
      "no_bid": false,
      "bid_price": 1.75,
      "seat_id": "seat-001"
    },
    {
      "dsp_platform": "thetradedesk",
      "no_bid": false,
      "bid_price": 2.10,
      "seat_id": "seat-002"
    }
  ]
}
```

### POST /v1/dsp/win

Record a DSP win notification.

**Auth:** None

**Request:**
```json
{
  "platform": "google_dv360",
  "win_price": 1.75
}
```

**Response:** 200 OK | **Metrics:** `dsp.wins` (with `platform` tag)

### GET /v1/dsp/status

Get DSP routing status.

**Auth:** None

**Response (200):**
```json
{
  "active_dsps": 4
}
```

---

## 6. Omnichannel

### POST /v1/channels/ingest

Process a real-time ingest event from any source.

**Auth:** None

**Sources:** `mobile_app`, `pos`, `kiosk`, `web`, `call_center`, `partner_api`, `iot_device`

**Event Types:** `purchase`, `product_view`, `cart_add`, `cart_abandon`, `app_open`, `page_view`, `search`, `wishlist_add`, `store_visit`, `loyalty_swipe`, `check_in`, `feedback`

**Request:**
```json
{
  "event_id": "evt-001",
  "source": "mobile_app",
  "event_type": "purchase",
  "user_id": "user-123",
  "device_id": "device-456",
  "session_id": "session-789",
  "payload": { "product_id": "prod-001", "amount": 25.50 },
  "location": { "lat": 37.7749, "lon": -122.4194, "accuracy_m": 10.0 },
  "occurred_at": "2026-02-14T10:30:00Z",
  "received_at": "2026-02-14T10:30:01Z"
}
```

**Response (200):**
```json
{
  "event_id": "evt-001",
  "user_id": "user-123",
  "should_activate": true,
  "loyalty_relevant": true
}
```

**Metrics:** `channels.ingest.processed` (with `source` tag)

### POST /v1/channels/activate

Dispatch an activation to a channel.

**Auth:** None

**Channels:** `push_notification`, `sms`, `email`, `in_app_message`, `web_personalization`, `whatsapp`, `web_push`, `content_card`, `webhook`, `facebook_meta`, `thetradedesk`, `google_dv360`, `amazon_dsp`, `digital_signage`, `kiosk_display`

**Request:**
```json
{
  "user_id": "user-123",
  "channel": "email",
  "message_template": "welcome_offer",
  "personalization_context": { "offer_name": "20% Off", "user_name": "Jane" }
}
```

**Response (200):**
```json
{
  "activation_id": "act-001",
  "channel": "email",
  "status": "queued",
  "user_id": "user-123"
}
```

### POST /v1/webhooks/sendgrid

SendGrid delivery webhook receiver.

**Auth:** None

**Request:** `Vec<EmailWebhookEvent>` — array of SendGrid event objects

**Response:** 200 OK | **Metrics:** `sendgrid.webhooks_received`

### GET /v1/channels/email/analytics/{activation_id}

Get email analytics for a specific activation.

**Auth:** None

**Response (200):**
```json
{
  "activation_id": "act-001",
  "opens": 150,
  "clicks": 42,
  "bounces": 3,
  "unsubscribes": 1,
  "sent": 200
}
```

### GET /v1/channels/email/analytics

Get analytics for all email activations.

**Auth:** None | **Response:** Array of `EmailAnalytics` objects

---

## 7. Campaign Management

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/campaigns` | List all campaigns |
| POST | `/api/v1/management/campaigns` | Create a campaign |
| GET | `/api/v1/management/campaigns/{id}` | Get campaign by ID |
| PUT | `/api/v1/management/campaigns/{id}` | Update campaign |
| DELETE | `/api/v1/management/campaigns/{id}` | Delete campaign |
| POST | `/api/v1/management/campaigns/{id}/pause` | Pause campaign |
| POST | `/api/v1/management/campaigns/{id}/resume` | Resume campaign |

### POST /api/v1/management/campaigns — Create Campaign

**Request:**
```json
{
  "name": "Summer Sale 2026",
  "budget": 50000.00,
  "daily_budget": 2500.00,
  "pacing": "even",
  "targeting": {
    "geo_regions": ["US-CA", "US-NY"],
    "segments": [101, 102],
    "devices": ["mobile", "desktop"],
    "floor_price": 0.50,
    "max_bid": 5.00,
    "frequency_cap_hourly": 3,
    "frequency_cap_daily": 10,
    "loyalty_tiers": ["gold", "reserve"],
    "dsp_platforms": ["google_dv360", "thetradedesk"]
  },
  "schedule_start": "2026-06-01T00:00:00Z",
  "schedule_end": "2026-08-31T23:59:59Z"
}
```

**Response (201):** Full `Campaign` object with generated `id`, `status: "draft"`, and timestamps.

**Metrics:** `management.campaigns.created`

---

## 8. Creative Management

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/creatives` | List all creatives |
| POST | `/api/v1/management/creatives` | Create a creative |
| GET | `/api/v1/management/creatives/{id}` | Get creative by ID |
| PUT | `/api/v1/management/creatives/{id}` | Update creative |
| DELETE | `/api/v1/management/creatives/{id}` | Delete creative |

### POST /api/v1/management/creatives — Create Creative

**Request:**
```json
{
  "campaign_id": "uuid",
  "name": "Summer Banner 300x250",
  "format": "banner",
  "asset_url": "https://cdn.example.com/banners/summer.png",
  "width": 300,
  "height": 250,
  "metadata": { "theme": "summer", "version": "2" }
}
```

**Formats:** `banner`, `native`, `video`, `html5`, `rich`

**Response (201):** Full `Creative` object | **Metrics:** `management.creatives.created`

---

## 9. Journey Orchestration

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/journeys` | List all journeys |
| POST | `/api/v1/management/journeys` | Create a journey |
| GET | `/api/v1/management/journeys/{id}` | Get journey by ID |
| DELETE | `/api/v1/management/journeys/{id}` | Delete journey |
| GET | `/api/v1/management/journeys/{id}/stats` | Journey performance stats |

---

## 10. DCO Templates

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/dco/templates` | List DCO templates |
| POST | `/api/v1/management/dco/templates` | Create DCO template |
| GET | `/api/v1/management/dco/templates/{id}` | Get DCO template |
| DELETE | `/api/v1/management/dco/templates/{id}` | Delete DCO template |

---

## 11. CDP Integration

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/cdp/platforms` | List CDP platform connectors |
| GET | `/api/v1/management/cdp/sync-history` | Get CDP sync history |

**Supported Platforms:** Salesforce, Adobe, Segment, Tealium, Hightouch

---

## 12. Experiments

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/experiments` | List all experiments |
| POST | `/api/v1/management/experiments` | Create an experiment |
| GET | `/api/v1/management/experiments/{id}` | Get experiment by ID |

---

## 13. Platform & Tenants

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/platform/tenants` | List tenants |
| POST | `/api/v1/management/platform/tenants` | Create tenant |
| GET | `/api/v1/management/platform/tenants/{id}` | Get tenant |
| PUT | `/api/v1/management/platform/tenants/{id}` | Update tenant |
| DELETE | `/api/v1/management/platform/tenants/{id}` | Delete tenant |
| POST | `/api/v1/management/platform/tenants/{id}/suspend` | Suspend tenant |
| POST | `/api/v1/management/platform/tenants/{id}/activate` | Activate tenant |

---

## 14. User Management

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/users` | List users |
| POST | `/api/v1/management/users` | Create user |
| GET | `/api/v1/management/users/{id}` | Get user |
| DELETE | `/api/v1/management/users/{id}` | Delete user |
| POST | `/api/v1/management/users/{id}/disable` | Disable user |
| POST | `/api/v1/management/users/{id}/enable` | Enable user |
| PUT | `/api/v1/management/users/{id}/role` | Update user role |
| GET | `/api/v1/management/invitations` | List invitations |
| POST | `/api/v1/management/invitations` | Create invitation |
| DELETE | `/api/v1/management/invitations/{id}` | Revoke invitation |

---

## 15. RBAC & Compliance

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/platform/roles` | List roles and permissions |
| GET | `/api/v1/management/platform/compliance` | Compliance status report |
| GET | `/api/v1/management/platform/privacy/dsrs` | List Data Subject Requests |
| GET | `/api/v1/management/audit-log` | Get audit log entries |

### GET /api/v1/management/audit-log

**Response (200):**
```json
[
  {
    "id": "uuid",
    "user": "admin",
    "action": "create",
    "resource_type": "campaign",
    "resource_id": "uuid",
    "details": {},
    "timestamp": "2026-02-14T10:30:00Z"
  }
]
```

**Actions:** `create`, `update`, `delete`, `pause`, `resume`, `model_reload`, `login`

---

## 16. Billing

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/billing/plans` | List billing plans |
| GET | `/api/v1/management/billing/subscriptions/{tenant_id}` | Get tenant subscription |
| GET | `/api/v1/management/billing/invoices` | List invoices |
| GET | `/api/v1/management/billing/usage/{tenant_id}` | Get tenant usage metrics |
| GET | `/api/v1/management/billing/onboarding/{tenant_id}` | Get onboarding status |

---

## 17. Operations & SLA

All endpoints require `Authorization: Bearer <token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/management/ops/status` | Operational status overview |
| GET | `/api/v1/management/ops/incidents` | List incidents |
| GET | `/api/v1/management/ops/sla` | SLA compliance report |
| GET | `/api/v1/management/ops/backups` | List backups |

---

## 18. Monitoring

### GET /api/v1/management/monitoring/overview

**Auth:** Bearer token

**Response (200):**
```json
{
  "total_campaigns": 150,
  "active_campaigns": 42,
  "total_impressions": 1500000,
  "total_clicks": 45000,
  "total_spend": 75000.00,
  "avg_ctr": 0.03,
  "avg_latency_us": 2500,
  "active_pods": 20,
  "offers_per_hour": 50000000,
  "cache_hit_rate": 0.92,
  "no_bid_rate": 0.15,
  "error_rate": 0.001
}
```

### GET /api/v1/management/monitoring/campaigns/{id}/stats

**Auth:** Bearer token

**Response (200):**
```json
{
  "impressions": 50000,
  "clicks": 1500,
  "conversions": 120,
  "spend": 2500.00,
  "ctr": 0.03,
  "win_rate": 0.45,
  "avg_bid": 1.25,
  "avg_win_price": 0.85,
  "hourly_data": [...]
}
```

### POST /api/v1/management/models/reload

Trigger NPU model reloading. **Auth:** Bearer token | **Response:** Status confirmation

---

## 19. Workflows & Approvals

**Auth:** Bearer token

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/workflows/calendar` | Get workflow calendar |

---

## 20. Brand & Assets

**Auth:** Bearer token

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/brand/assets` | List brand assets |

---

## 21. Reporting

**Auth:** Bearer token

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/reports/templates` | List report templates |

**Report Types:** Campaign performance, audience insights, channel attribution, budget utilization, conversion funnel, A/B test results, loyalty analytics, DSP performance, journey completion, creative performance

**Export Formats:** CSV, JSON, Excel

---

## 22. Recommendations

### GET /api/v1/recommendations/{user_id}

Get personalized recommendations for a user.

**Auth:** None

**Query Parameters:**
- `strategy` — `cf` (collaborative filtering), `content_based`, `trending`, `new_arrivals`, `frequently_bought_together`

**Example:**
```bash
curl http://localhost:8080/api/v1/recommendations/user-123?strategy=cf
```

---

## 23. Integrations

**Auth:** Bearer token

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/integrations/dam/search?query=banner` | Search DAM assets |

**Supported Integrations:**
- **Project Management:** Asana, Jira
- **Digital Asset Management:** AEM Assets, Bynder, Aprimo
- **Business Intelligence:** Power BI, Excel

---

## 24. Inference Providers

### GET /api/v1/inference/providers

List available inference hardware backends.

**Auth:** None

**Response (200):**
```json
[
  { "name": "cpu", "status": "active", "latency_p99_us": 5000 },
  { "name": "groq_lpu", "status": "available", "latency_p99_us": 1200 },
  { "name": "aws_inferentia", "status": "available", "latency_p99_us": 800 },
  { "name": "ampere_altra", "status": "available", "latency_p99_us": 1500 },
  { "name": "tenstorrent", "status": "available", "latency_p99_us": 900 }
]
```

---

## 25. Segmentation

Audience segmentation with rule-based real-time evaluation. Endpoints available under management APIs.

---

## 26. Intelligent Delivery

Smart delivery optimization with global per-channel suppression lists (with expiry). Endpoints available under management APIs.

---

## 27. gRPC API

**Port:** 9090 | **Framework:** Tonic 0.12 | **Service:** `BiddingService`

### RPC process_bid

Unary bid request processing.

**Request:**
```protobuf
message BidRequestProto {
  string openrtb_json = 1;
  string request_id = 2;
  string user_id = 3;
  uint32 timeout_ms = 4;
}
```

**Response:**
```protobuf
message BidResponseProto {
  string openrtb_json = 1;
  string request_id = 2;
  bool has_bid = 3;
  uint64 processing_time_us = 4;
  string agent_id = 5;
}
```

### RPC health_check

Health check for gRPC clients.

**Request:**
```protobuf
message HealthCheckRequest {
  string service = 1;
}
```

**Response:**
```protobuf
message HealthCheckResponse {
  ServingStatus status = 1;  // Unknown=0, Serving=1, NotServing=2
  string node_id = 2;
  uint32 active_agents = 3;
  uint64 uptime_secs = 4;
}
```

### RPC stream_bids

Bidirectional streaming for high-throughput batch processing.

**Stream:** `stream BidRequestProto` -> `stream BidResponseProto`
**Buffer Size:** 128 internal channel

---

## 28. Error Handling

All error responses follow a consistent format:

```json
{
  "error": "error_code",
  "message": "Human-readable description"
}
```

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created |
| 202 | Accepted (async processing) |
| 204 | No Content (successful delete) |
| 400 | Bad Request (validation error) |
| 401 | Unauthorized (missing/invalid token) |
| 404 | Not Found |
| 429 | Too Many Requests (rate limited) |
| 500 | Internal Server Error |
| 503 | Service Unavailable (not ready) |

### Middleware

- **Compression:** gzip via `CompressionLayer`
- **CORS:** Permissive (all origins)
- **Tracing:** Request-level tracing via `TraceLayer`

---

## Metrics Summary

All metrics are exposed via Prometheus on port 9091.

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `api.errors` | Counter | — | Bid processing failures |
| `loyalty.api.earn_stars` | Counter | — | Star earning events |
| `loyalty.api.redemptions` | Counter | — | Successful redemptions |
| `loyalty.reward_signals` | Counter | `type` | SNN training signals |
| `channels.ingest.processed` | Counter | `source` | Ingest events by source |
| `sendgrid.webhooks_received` | Counter | — | Email delivery webhooks |
| `dsp.wins` | Counter | `platform` | DSP win notifications |
| `management.campaigns.created` | Counter | — | Campaign creations |
| `management.campaigns.deleted` | Counter | — | Campaign deletions |
| `management.creatives.created` | Counter | — | Creative creations |

---

## Service Ports

| Service | Port | Protocol |
|---------|------|----------|
| REST API | 8080 | HTTP |
| gRPC | 9090 | HTTP/2 |
| Prometheus Metrics | 9091 | HTTP |
| NATS Client | 4222 | TCP |
| NATS Management | 8222 | HTTP |
| Redis | 6379 | TCP |
| ClickHouse HTTP | 8123 | HTTP |
| ClickHouse Native | 9000 | TCP |
| Grafana | 3000 | HTTP |
