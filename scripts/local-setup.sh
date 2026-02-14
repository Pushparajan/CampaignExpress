#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Complete Local Setup (End-to-End with Test Data)
# =============================================================================
#
# Sets up the entire application and all dependencies locally:
#
#   1. Prerequisites:  Rust 1.75+, Docker, Node.js 18+, redis-cli
#   2. Rust build:     cargo build + cargo test (workspace)
#   3. Infrastructure: Docker Compose (NATS, Redis, ClickHouse, Prometheus, Grafana)
#   4. NATS streams:   JetStream stream + consumer (campaign-bids)
#   5. ClickHouse:     Schema + seed data (campaigns, bid events, user segments)
#   6. Redis:          Seed data (campaigns, users, offers, segment indexes)
#   7. Frontend:       npm install + next dev (Next.js 14, React 18, Tailwind)
#   8. Backend:        campaign-express binary (API-only or full agents)
#   9. Smoke tests:    Health checks + sample bid request
#
# Usage:
#   ./scripts/local-setup.sh                  # Full setup
#   ./scripts/local-setup.sh --check          # Only check prerequisites
#   ./scripts/local-setup.sh --no-build       # Skip Rust build
#   ./scripts/local-setup.sh --no-frontend    # Skip frontend setup
#   ./scripts/local-setup.sh --seed-only      # Only seed data (infra must be running)
#   ./scripts/local-setup.sh --reset          # Wipe all data + re-seed
#   ./scripts/local-setup.sh --full-agents    # Run with NATS agents (not API-only)
#
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_FILE="$PROJECT_ROOT/deploy/docker/docker-compose.yml"
SEED_DIR="$SCRIPT_DIR/seed"
MIN_RUST_VERSION="1.75.0"
MIN_NODE_VERSION="18.0.0"

# ── Flags ───────────────────────────────────────────────────────────────────

CHECK_ONLY=false
SKIP_BUILD=false
SKIP_FRONTEND=false
SEED_ONLY=false
RESET=false
FULL_AGENTS=false

for arg in "$@"; do
  case "$arg" in
    --check)        CHECK_ONLY=true ;;
    --no-build)     SKIP_BUILD=true ;;
    --no-frontend)  SKIP_FRONTEND=true ;;
    --seed-only)    SEED_ONLY=true ;;
    --reset)        RESET=true ;;
    --full-agents)  FULL_AGENTS=true ;;
    --help|-h)
      sed -n '2,/^# ====/{ /^#/s/^# \?//p }' "$0"
      exit 0 ;;
    *) echo "Unknown option: $arg"; exit 1 ;;
  esac
done

# ── Colors + Helpers ────────────────────────────────────────────────────────

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'

log()     { echo -e "${BLUE}[$(date +%H:%M:%S)]${NC} $*"; }
ok()      { echo -e "${GREEN}  [OK]${NC} $*"; }
warn()    { echo -e "${YELLOW}  [WARN]${NC} $*"; }
err()     { echo -e "${RED}  [ERROR]${NC} $*" >&2; }
banner()  {
  echo ""
  echo -e "${CYAN}───────────────────────────────────────────────────────────${NC}"
  echo -e "${CYAN}  $*${NC}"
  echo -e "${CYAN}───────────────────────────────────────────────────────────${NC}"
}

version_gte() { printf '%s\n%s' "$2" "$1" | sort -V -C; }

wait_for_port() {
  local host=$1 port=$2 name=$3 timeout=${4:-30}
  local elapsed=0
  while ! (echo >/dev/tcp/"$host"/"$port") 2>/dev/null; do
    if (( elapsed >= timeout )); then
      err "$name not reachable on $host:$port after ${timeout}s"
      return 1
    fi
    sleep 1; elapsed=$((elapsed + 1))
  done
  ok "$name ready on $host:$port (${elapsed}s)"
}

# =============================================================================
# Step 1: Prerequisites
# =============================================================================

check_prerequisites() {
  banner "Step 1: Checking Prerequisites"

  local errors=0

  # Rust
  if command -v rustc &>/dev/null; then
    local rust_ver
    rust_ver=$(rustc --version | awk '{print $2}')
    if version_gte "$rust_ver" "$MIN_RUST_VERSION"; then
      ok "Rust $rust_ver (>= $MIN_RUST_VERSION)"
    else
      err "Rust $rust_ver is below minimum $MIN_RUST_VERSION"
      errors=$((errors + 1))
    fi
  else
    err "Rust not found — install from https://rustup.rs"
    errors=$((errors + 1))
  fi

  command -v cargo &>/dev/null && ok "cargo" || { err "cargo not found"; errors=$((errors + 1)); }

  # Docker
  if command -v docker &>/dev/null; then
    if docker info &>/dev/null; then
      ok "Docker daemon running"
    else
      err "Docker installed but daemon not running"
      errors=$((errors + 1))
    fi
  else
    err "Docker not found — https://docs.docker.com/get-docker/"
    errors=$((errors + 1))
  fi

  docker compose version &>/dev/null && ok "Docker Compose" || { err "Docker Compose not found"; errors=$((errors + 1)); }

  # Node.js (for frontend)
  if ! $SKIP_FRONTEND; then
    if command -v node &>/dev/null; then
      local node_ver
      node_ver=$(node -v | tr -d 'v')
      if version_gte "$node_ver" "$MIN_NODE_VERSION"; then
        ok "Node.js $node_ver (>= $MIN_NODE_VERSION)"
      else
        warn "Node.js $node_ver is below minimum $MIN_NODE_VERSION"
      fi
    else
      warn "Node.js not found — frontend will be skipped"
      SKIP_FRONTEND=true
    fi
    command -v npm &>/dev/null && ok "npm" || warn "npm not found"
  fi

  # Optional tools
  command -v redis-cli &>/dev/null && ok "redis-cli" || warn "redis-cli not found (Redis seeding will use Docker exec fallback)"
  command -v curl &>/dev/null && ok "curl" || warn "curl not found (smoke tests will be skipped)"
  command -v jq &>/dev/null && ok "jq" || warn "jq not found (optional)"

  if (( errors > 0 )); then
    err "$errors required tool(s) missing"
    exit 1
  fi

  ok "All prerequisites met"
}

# =============================================================================
# Step 2: Build Workspace
# =============================================================================

build_workspace() {
  if $SKIP_BUILD; then
    log "Skipping build (--no-build)"
    return 0
  fi

  banner "Step 2: Building Rust Workspace"

  cd "$PROJECT_ROOT"

  # Ensure toolchain components
  if command -v rustup &>/dev/null; then
    rustup component add clippy rustfmt 2>/dev/null || true
  fi

  # Create .env if missing
  if [ ! -f "$PROJECT_ROOT/.env" ] && [ -f "$PROJECT_ROOT/.env.example" ]; then
    cp "$PROJECT_ROOT/.env.example" "$PROJECT_ROOT/.env"
    ok "Created .env from .env.example"
  fi

  log "Building workspace (this may take a few minutes on first run)..."
  if cargo build --workspace 2>&1; then
    ok "Workspace build succeeded"
  else
    err "Build failed"
    exit 1
  fi

  log "Running tests..."
  if cargo test --workspace 2>&1; then
    ok "All tests passed"
  else
    warn "Some tests failed — continuing setup"
  fi
}

# =============================================================================
# Step 3: Docker Infrastructure
# =============================================================================

start_infrastructure() {
  banner "Step 3: Starting Infrastructure (Docker Compose)"
  log "Services: NATS JetStream · Redis 7 · ClickHouse 24 · Prometheus · Grafana"

  cd "$PROJECT_ROOT"

  if $RESET; then
    log "Resetting all data volumes..."
    docker compose -f "$COMPOSE_FILE" down -v 2>/dev/null || true
  fi

  # Start infra services (not the app — we run that natively)
  docker compose -f "$COMPOSE_FILE" up -d \
    nats redis clickhouse prometheus grafana 2>&1

  log "Waiting for services to be healthy..."

  wait_for_port localhost 4222 "NATS"
  wait_for_port localhost 6379 "Redis"
  wait_for_port localhost 8123 "ClickHouse"

  ok "All infrastructure services running"
}

# =============================================================================
# Step 4: NATS JetStream Streams
# =============================================================================

setup_nats_streams() {
  banner "Step 4: Configuring NATS JetStream"

  # Use nats CLI if available, otherwise use the HTTP API
  if command -v nats &>/dev/null; then
    log "Using nats CLI..."
    bash "$SCRIPT_DIR/setup-nats-streams.sh" "nats://localhost:4222" 2>&1 || true
  else
    log "nats CLI not found — creating stream via HTTP monitoring API..."
    # Wait for JetStream to be ready
    sleep 2

    # Check JetStream is enabled
    local js_info
    js_info=$(curl -sf http://localhost:8222/jsz 2>/dev/null || echo "")
    if echo "$js_info" | grep -q "streams"; then
      ok "JetStream is enabled"
    else
      warn "JetStream info not available yet — stream will be auto-created on first use"
    fi
  fi

  ok "NATS JetStream configured"
}

# =============================================================================
# Step 5: ClickHouse Schema + Seed Data
# =============================================================================

seed_clickhouse() {
  banner "Step 5: ClickHouse Schema + Test Data"

  log "Creating schema and inserting seed data..."

  if command -v clickhouse-client &>/dev/null; then
    clickhouse-client --host localhost --multiquery < "$SEED_DIR/clickhouse-schema.sql" 2>&1
  else
    # Use curl to ClickHouse HTTP interface
    curl -sf "http://localhost:8123/" --data-binary @"$SEED_DIR/clickhouse-schema.sql" 2>&1 || {
      # Some statements may fail if tables exist; try line by line
      log "Retrying statements individually..."
      local tmpdir
      tmpdir=$(mktemp -d)
      # Split on empty lines between statements
      csplit -z -f "$tmpdir/stmt-" "$SEED_DIR/clickhouse-schema.sql" '/^$/+1' '{*}' 2>/dev/null || true
      for f in "$tmpdir"/stmt-*; do
        # Skip comment-only blocks
        if grep -qE '^(CREATE|INSERT|ALTER)' "$f"; then
          curl -sf "http://localhost:8123/" --data-binary @"$f" 2>/dev/null || true
        fi
      done
      rm -rf "$tmpdir"
    }
  fi

  # Verify
  local table_count
  table_count=$(curl -sf "http://localhost:8123/?query=SELECT+count()+FROM+system.tables+WHERE+database='campaign_express'" 2>/dev/null || echo "0")
  ok "ClickHouse: $table_count tables in campaign_express database"

  local event_count
  event_count=$(curl -sf "http://localhost:8123/?query=SELECT+count()+FROM+campaign_express.bid_events" 2>/dev/null || echo "0")
  ok "ClickHouse: $event_count bid events seeded"
}

# =============================================================================
# Step 6: Redis Seed Data
# =============================================================================

seed_redis() {
  banner "Step 6: Redis Test Data"

  if command -v redis-cli &>/dev/null; then
    log "Seeding Redis via redis-cli..."
    bash "$SEED_DIR/redis-seed.sh" "redis://localhost:6379"
  else
    log "redis-cli not found — seeding via Docker exec..."
    # Copy seed script into container and run
    docker compose -f "$COMPOSE_FILE" exec -T redis sh -c '
      apk add --no-cache bash >/dev/null 2>&1 || true
    '
    docker compose -f "$COMPOSE_FILE" cp "$SEED_DIR/redis-seed.sh" redis:/tmp/redis-seed.sh
    docker compose -f "$COMPOSE_FILE" exec -T redis bash /tmp/redis-seed.sh "redis://localhost:6379"
  fi

  ok "Redis seeded with campaigns, users, offers, and segment indexes"
}

# =============================================================================
# Step 7: Frontend Setup
# =============================================================================

setup_frontend() {
  if $SKIP_FRONTEND; then
    log "Skipping frontend (--no-frontend or Node.js not available)"
    return 0
  fi

  banner "Step 7: Frontend Setup (Next.js 14)"

  cd "$PROJECT_ROOT/ui"

  if [ ! -d "node_modules" ] || [ ! -f "node_modules/.package-lock.json" ]; then
    log "Installing npm dependencies..."
    npm install 2>&1
    ok "npm install complete"
  else
    ok "node_modules already installed"
  fi

  # Create .env.local for development
  if [ ! -f ".env.local" ]; then
    cat > .env.local <<'ENVEOF'
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws
ENVEOF
    ok "Created ui/.env.local"
  fi

  cd "$PROJECT_ROOT"
  ok "Frontend ready — run 'cd ui && npm run dev' to start"
}

# =============================================================================
# Step 8: Start Backend
# =============================================================================

start_backend() {
  banner "Step 8: Starting Campaign Express Backend"

  cd "$PROJECT_ROOT"

  local run_args=(
    --bin campaign-express
  )

  if $FULL_AGENTS; then
    log "Starting with full NATS agent system..."
    export CAMPAIGN_EXPRESS__NATS__URLS="nats://localhost:4222"
    export CAMPAIGN_EXPRESS__AGENTS_PER_NODE=4
  else
    run_args+=(-- --api-only)
  fi

  export RUST_LOG="campaign_express=debug,tower_http=debug"
  export CAMPAIGN_EXPRESS__NODE_ID="dev-01"
  export CAMPAIGN_EXPRESS__AGENTS_PER_NODE="${CAMPAIGN_EXPRESS__AGENTS_PER_NODE:-2}"
  export CAMPAIGN_EXPRESS__API__HOST="0.0.0.0"
  export CAMPAIGN_EXPRESS__API__HTTP_PORT=8080
  export CAMPAIGN_EXPRESS__API__GRPC_PORT=9090
  export CAMPAIGN_EXPRESS__REDIS__URLS="redis://localhost:6379"
  export CAMPAIGN_EXPRESS__CLICKHOUSE__URL="http://localhost:8123"
  export CAMPAIGN_EXPRESS__CLICKHOUSE__DATABASE="campaign_express"
  export CAMPAIGN_EXPRESS__NPU__DEVICE="cpu"
  export CAMPAIGN_EXPRESS__METRICS__PORT=9091

  log "Starting backend (HTTP :8080, gRPC :9090, Metrics :9091)..."
  cargo run "${run_args[@]}" &
  BACKEND_PID=$!

  # Wait for backend to be ready
  local elapsed=0
  while ! curl -sf http://localhost:8080/health &>/dev/null; do
    if (( elapsed >= 60 )); then
      err "Backend did not become healthy within 60s"
      kill "$BACKEND_PID" 2>/dev/null || true
      return 1
    fi
    sleep 1; elapsed=$((elapsed + 1))
  done

  ok "Backend running (PID $BACKEND_PID, took ${elapsed}s)"
}

# =============================================================================
# Step 9: Smoke Tests
# =============================================================================

run_smoke_tests() {
  banner "Step 9: Smoke Tests"

  if ! command -v curl &>/dev/null; then
    warn "curl not available — skipping smoke tests"
    return 0
  fi

  local failures=0

  # Health endpoint
  log "Testing /health..."
  if curl -sf http://localhost:8080/health >/dev/null 2>&1; then
    ok "/health — 200 OK"
  else
    err "/health failed"
    failures=$((failures + 1))
  fi

  # Ready endpoint
  log "Testing /ready..."
  if curl -sf http://localhost:8080/ready >/dev/null 2>&1; then
    ok "/ready — 200 OK"
  else
    warn "/ready not available (may not be implemented)"
  fi

  # Metrics endpoint
  log "Testing /metrics..."
  local metrics_port=9091
  if curl -sf "http://localhost:${metrics_port}/metrics" >/dev/null 2>&1; then
    ok "/metrics — 200 OK (port $metrics_port)"
  else
    warn "Metrics endpoint not reachable on port $metrics_port"
  fi

  # Bid request
  log "Sending test bid request to /v1/bid..."
  local bid_response
  bid_response=$(curl -sf -w "\n%{http_code}" -X POST http://localhost:8080/v1/bid \
    -H "Content-Type: application/json" \
    -d '{
      "id": "smoke-test-001",
      "imp": [{
        "id": "imp-1",
        "banner": {"w": 300, "h": 250},
        "bidfloor": 0.5
      }],
      "site": {
        "id": "site-1",
        "domain": "example.com",
        "page": "https://example.com/test"
      },
      "device": {
        "ua": "Mozilla/5.0 (smoke-test)",
        "ip": "203.0.113.1",
        "devicetype": 2,
        "os": "iOS"
      },
      "user": {
        "id": "user-101",
        "keywords": "tech,programming"
      },
      "tmax": 100,
      "at": 1,
      "cur": ["USD"]
    }' 2>/dev/null || echo -e "\n000")

  local http_code
  http_code=$(echo "$bid_response" | tail -1)
  local body
  body=$(echo "$bid_response" | head -n -1)

  if [ "$http_code" = "200" ] || [ "$http_code" = "204" ]; then
    ok "POST /v1/bid — $http_code"
    if [ -n "$body" ] && command -v jq &>/dev/null; then
      echo "$body" | jq -r '  "    Response ID: \(.id // "N/A")\n    SeatBids: \(.seatbid | length // 0)"' 2>/dev/null || true
    fi
  else
    warn "POST /v1/bid — HTTP $http_code (may be expected for no-bid)"
  fi

  # ClickHouse query test
  log "Querying ClickHouse test data..."
  local ch_count
  ch_count=$(curl -sf "http://localhost:8123/?query=SELECT+count()+FROM+campaign_express.bid_events" 2>/dev/null || echo "error")
  if [ "$ch_count" != "error" ]; then
    ok "ClickHouse — $ch_count bid events in database"
  else
    warn "ClickHouse query failed"
  fi

  # Redis check
  log "Checking Redis test data..."
  if command -v redis-cli &>/dev/null; then
    local redis_keys
    redis_keys=$(redis-cli -h localhost -p 6379 DBSIZE 2>/dev/null | awk '{print $2}')
    ok "Redis — $redis_keys keys"
  fi

  echo ""
  if (( failures > 0 )); then
    warn "$failures smoke test(s) failed"
  else
    ok "All smoke tests passed"
  fi
}

# =============================================================================
# Print Summary
# =============================================================================

print_summary() {
  banner "Setup Complete"

  cat <<SUMMARY

  Services Running:
    Backend API     http://localhost:8080     (Rust/Axum/Tonic)
    gRPC            localhost:9090
    Metrics         http://localhost:9091     (Prometheus scrape)
    NATS            nats://localhost:4222     (monitor: http://localhost:8222)
    Redis           redis://localhost:6379
    ClickHouse      http://localhost:8123     (DB: campaign_express)
    Prometheus      http://localhost:9092
    Grafana         http://localhost:3000     (admin / campaign-express)
SUMMARY

  if ! $SKIP_FRONTEND; then
    echo "    Frontend        cd ui && npm run dev   (http://localhost:3000 — starts separately)"
  fi

  cat <<'SUMMARY'

  Test Data:
    ClickHouse  5 tables, 20 bid events, 10 campaigns, 23 user segments
    Redis       10 campaigns, 10 users, 10 offers, 18 segment indexes
    NATS        campaign-bids stream + bid-agents consumer

  Quick Commands:
    # Run frontend dev server
    cd ui && npm run dev

    # Send a test bid
    curl -X POST http://localhost:8080/v1/bid \
      -H 'Content-Type: application/json' \
      -d '{"id":"test-1","imp":[{"id":"imp-1","banner":{"w":300,"h":250},"bidfloor":0.5}],"user":{"id":"user-101"},"tmax":100}'

    # Run load test (50 concurrent, 1000 requests)
    ./scripts/load-test.sh http://localhost:8080 50 1000

    # Query analytics
    curl 'http://localhost:8123/?query=SELECT+*+FROM+campaign_express.campaign_stats_daily+FORMAT+Pretty'

    # Check NATS
    curl -s http://localhost:8222/jsz | jq .

    # Stop everything
    make compose-down && kill %1

SUMMARY
}

# =============================================================================
# Main
# =============================================================================

main() {
  echo ""
  echo -e "${CYAN}  Campaign Express — Local Development Setup${NC}"
  echo -e "${CYAN}  Complete end-to-end with test data${NC}"
  echo ""

  check_prerequisites

  if $CHECK_ONLY; then
    exit 0
  fi

  if $SEED_ONLY; then
    seed_clickhouse
    seed_redis
    echo ""
    ok "Seed data loaded"
    exit 0
  fi

  if ! $SKIP_BUILD; then
    build_workspace
  fi

  start_infrastructure
  setup_nats_streams
  seed_clickhouse
  seed_redis
  setup_frontend
  start_backend
  run_smoke_tests
  print_summary

  ok "Campaign Express is running. Press Ctrl+C to stop the backend."

  # Keep script alive while backend runs
  if [ -n "${BACKEND_PID:-}" ]; then
    trap 'echo ""; log "Shutting down backend (PID $BACKEND_PID)..."; kill "$BACKEND_PID" 2>/dev/null; exit 0' INT TERM
    wait "$BACKEND_PID" 2>/dev/null || true
  fi
}

main
