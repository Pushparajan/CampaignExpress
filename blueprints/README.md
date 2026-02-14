# Campaign Express Blueprints

Starter templates for building applications integrated with the Campaign Express platform. These blueprints provide production-ready patterns for common use cases.

## Available Blueprints

### Web App (`web-app/`)

A **Next.js 14 + React 18** web application with:

- **API Client** — Typed REST client for all Campaign Express endpoints (campaigns, loyalty, monitoring, omnichannel)
- **Event Tracker** — Client-side event tracking SDK with batched delivery, session management, and anonymous ID persistence
- **React Hooks** — TanStack React Query hooks for campaigns, monitoring, and loyalty
- **UI Components** — Stats cards, campaign tables, loyalty cards with Tailwind CSS
- **Pages** — Dashboard, Campaigns (CRUD), Loyalty, Login

**Quick Start:**
```bash
cd blueprints/web-app
cp .env.example .env.local
npm install
npm run dev
```

### Mobile App (`mobile-app/`)

A **React Native (Expo)** mobile application with:

- **Mobile API Client** — Typed REST client with SecureStore token persistence
- **Mobile SDK** — Event tracking, session management, push notification registration, geofence support
- **Screens** — Dashboard (real-time metrics), Campaigns (list with actions), Loyalty (tier + rewards), Settings
- **Navigation** — Tab-based navigation with React Navigation

**Quick Start:**
```bash
cd blueprints/mobile-app
cp .env.example .env
npm install
npx expo start
```

## Architecture

Both blueprints connect to Campaign Express via:

```
+------------------+         +------------------------+
|  Web/Mobile App  | ------> |  Campaign Express API  |
|                  |         |  (Axum / Port 8080)    |
|  - API Client    |         |                        |
|  - Event Tracker |         |  /api/v1/management/*  |
|  - React Query   |         |  /v1/loyalty/*         |
+------------------+         |  /v1/channels/*        |
                             |  /v1/bid               |
                             +------------------------+
```

## API Endpoints Used

| Endpoint                                  | Method | Description               |
|-------------------------------------------|--------|---------------------------|
| `/api/v1/management/auth/login`           | POST   | Authenticate user         |
| `/api/v1/management/campaigns`            | GET    | List campaigns            |
| `/api/v1/management/campaigns`            | POST   | Create campaign           |
| `/api/v1/management/campaigns/{id}/pause` | POST   | Pause campaign            |
| `/api/v1/management/campaigns/{id}/resume`| POST   | Resume campaign           |
| `/api/v1/management/monitoring/overview`  | GET    | Real-time platform stats  |
| `/v1/loyalty/balance/{user_id}`           | GET    | Get loyalty balance       |
| `/v1/loyalty/earn`                        | POST   | Earn loyalty stars        |
| `/v1/loyalty/redeem`                      | POST   | Redeem loyalty stars      |
| `/v1/channels/ingest`                     | POST   | Ingest behaviour events   |
| `/v1/channels/activate`                   | POST   | Activate channel message  |
| `/api/v1/management/billing/plans`        | GET    | List pricing plans        |

## Customization

Both blueprints are designed to be extended:

1. **Add new pages/screens** — Follow the existing patterns in `pages/` or `screens/`
2. **Add new API methods** — Extend `api-client.ts` with additional endpoints
3. **Track custom events** — Use `tracker.trackCustomEvent()` or `ceSdk.trackEvent()`
4. **Integrate DCO/Journeys** — Add hooks for DCO templates and journey orchestration
5. **Add experiments** — Hook into the A/B testing API for variant assignment
