#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Shared Startup Helpers
# =============================================================================
# Common functions, colors, port helpers, and infrastructure management
# used by all role-based startup scripts.
#
# Source this file — do not execute directly.
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
COMPOSE_FILE="$PROJECT_ROOT/deploy/docker/docker-compose.yml"
SEED_DIR="$PROJECT_ROOT/scripts/seed"
UI_DIR="$PROJECT_ROOT/ui"

# ── Colors + Helpers ─────────────────────────────────────────────────────────

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; MAGENTA='\033[0;35m'; NC='\033[0m'

log()     { echo -e "${BLUE}[$(date +%H:%M:%S)]${NC} $*"; }
ok()      { echo -e "${GREEN}  [OK]${NC} $*"; }
warn()    { echo -e "${YELLOW}  [WARN]${NC} $*"; }
err()     { echo -e "${RED}  [ERROR]${NC} $*" >&2; }
section() { echo -e "\n${MAGENTA}>> $*${NC}"; }

banner() {
  local role=$1 system=$2
  echo ""
  echo -e "${CYAN}==================================================================${NC}"
  echo -e "${CYAN}  Campaign Express — ${role} ${system} Startup${NC}"
  echo -e "${CYAN}==================================================================${NC}"
  echo ""
}

# ── Port Helpers ─────────────────────────────────────────────────────────────

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

check_port_available() {
  local port=$1 name=$2
  if (echo >/dev/tcp/localhost/"$port") 2>/dev/null; then
    warn "Port $port ($name) already in use — service may already be running"
    return 1
  fi
  return 0
}

# ── Infrastructure ───────────────────────────────────────────────────────────

# Start core infrastructure services (NATS, Redis, ClickHouse)
start_core_infra() {
  section "Starting Core Infrastructure (Docker Compose)"
  log "Services: NATS JetStream, Redis 7, ClickHouse 24"

  docker compose -f "$COMPOSE_FILE" up -d nats redis clickhouse 2>&1

  wait_for_port localhost 4222 "NATS"
  wait_for_port localhost 6379 "Redis"
  wait_for_port localhost 8123 "ClickHouse"

  ok "Core infrastructure running"
}

# Start monitoring stack (Prometheus + Grafana)
start_monitoring() {
  section "Starting Monitoring Stack"
  log "Services: Prometheus, Grafana"

  docker compose -f "$COMPOSE_FILE" up -d prometheus grafana 2>&1

  wait_for_port localhost 9092 "Prometheus" 20
  wait_for_port localhost 3000 "Grafana" 20

  ok "Monitoring stack running"
}

# Seed data into ClickHouse and Redis
seed_data() {
  section "Seeding Test Data"

  # ClickHouse
  log "Seeding ClickHouse schema + data..."
  if command -v clickhouse-client &>/dev/null; then
    clickhouse-client --host localhost --multiquery < "$SEED_DIR/clickhouse-schema.sql" 2>&1 || true
  else
    curl -sf "http://localhost:8123/" --data-binary @"$SEED_DIR/clickhouse-schema.sql" 2>/dev/null || {
      log "Retrying ClickHouse statements individually..."
      local tmpdir
      tmpdir=$(mktemp -d)
      csplit -z -f "$tmpdir/stmt-" "$SEED_DIR/clickhouse-schema.sql" '/^$/+1' '{*}' 2>/dev/null || true
      for f in "$tmpdir"/stmt-*; do
        if grep -qE '^(CREATE|INSERT|ALTER)' "$f"; then
          curl -sf "http://localhost:8123/" --data-binary @"$f" 2>/dev/null || true
        fi
      done
      rm -rf "$tmpdir"
    }
  fi
  ok "ClickHouse seeded"

  # Redis
  log "Seeding Redis..."
  if command -v redis-cli &>/dev/null; then
    bash "$SEED_DIR/redis-seed.sh" "redis://localhost:6379" 2>&1 || true
  else
    docker compose -f "$COMPOSE_FILE" exec -T redis sh -c 'apk add --no-cache bash >/dev/null 2>&1 || true'
    docker compose -f "$COMPOSE_FILE" cp "$SEED_DIR/redis-seed.sh" redis:/tmp/redis-seed.sh 2>/dev/null || true
    docker compose -f "$COMPOSE_FILE" exec -T redis bash /tmp/redis-seed.sh "redis://localhost:6379" 2>&1 || true
  fi
  ok "Redis seeded"
}

# Setup NATS JetStream streams
setup_nats() {
  section "Configuring NATS JetStream"
  if command -v nats &>/dev/null; then
    bash "$PROJECT_ROOT/scripts/setup-nats-streams.sh" "nats://localhost:4222" 2>&1 || true
  else
    sleep 2
    local js_info
    js_info=$(curl -sf http://localhost:8222/jsz 2>/dev/null || echo "")
    if echo "$js_info" | grep -q "streams"; then
      ok "JetStream is enabled"
    else
      warn "JetStream info not available — stream will auto-create on first use"
    fi
  fi
  ok "NATS configured"
}

# ── Prerequisite Checks ─────────────────────────────────────────────────────

check_rust() {
  if ! command -v cargo &>/dev/null; then
    err "Rust/cargo not found — install from https://rustup.rs"
    exit 1
  fi
  ok "Rust $(rustc --version | awk '{print $2}')"
}

check_docker() {
  if ! command -v docker &>/dev/null; then
    err "Docker not found"
    exit 1
  fi
  if ! docker info &>/dev/null 2>&1; then
    err "Docker daemon not running"
    exit 1
  fi
  ok "Docker running"
}

check_node() {
  if ! command -v node &>/dev/null; then
    err "Node.js not found — install Node 18+ for frontend"
    exit 1
  fi
  ok "Node.js $(node -v)"
}

# ── Backend Start Helpers ────────────────────────────────────────────────────

# Start the Rust backend with role-specific environment variables.
# Args: $1=role_name, $2=http_port, $3=grpc_port, $4=metrics_port, $5=extra_flags...
start_rust_backend() {
  local role=$1 http_port=$2 grpc_port=$3 metrics_port=$4
  shift 4
  local extra_flags=("$@")

  section "Starting Backend (role: $role)"

  cd "$PROJECT_ROOT"

  export RUST_LOG="${RUST_LOG:-campaign_express=info,tower_http=info}"
  export CAMPAIGN_EXPRESS__NODE_ID="${CAMPAIGN_EXPRESS__NODE_ID:-${role}-node-01}"
  export CAMPAIGN_EXPRESS__API__HOST="0.0.0.0"
  export CAMPAIGN_EXPRESS__API__HTTP_PORT="$http_port"
  export CAMPAIGN_EXPRESS__API__GRPC_PORT="$grpc_port"
  export CAMPAIGN_EXPRESS__REDIS__URLS="${CAMPAIGN_EXPRESS__REDIS__URLS:-redis://localhost:6379}"
  export CAMPAIGN_EXPRESS__CLICKHOUSE__URL="${CAMPAIGN_EXPRESS__CLICKHOUSE__URL:-http://localhost:8123}"
  export CAMPAIGN_EXPRESS__CLICKHOUSE__DATABASE="campaign_express"
  export CAMPAIGN_EXPRESS__NPU__DEVICE="${CAMPAIGN_EXPRESS__NPU__DEVICE:-cpu}"
  export CAMPAIGN_EXPRESS__METRICS__PORT="$metrics_port"

  local run_args=(--bin campaign-express)

  log "Launching: HTTP :$http_port | gRPC :$grpc_port | Metrics :$metrics_port"

  cargo run "${run_args[@]}" -- "${extra_flags[@]}" &
  BACKEND_PID=$!

  # Wait for healthy
  local elapsed=0
  while ! curl -sf "http://localhost:${http_port}/health" &>/dev/null; do
    if (( elapsed >= 90 )); then
      err "Backend did not become healthy within 90s"
      kill "$BACKEND_PID" 2>/dev/null || true
      return 1
    fi
    sleep 1; elapsed=$((elapsed + 1))
  done

  ok "Backend running (PID $BACKEND_PID, HTTP :$http_port, ${elapsed}s startup)"
}

# ── Frontend Start Helpers ───────────────────────────────────────────────────

# Install npm deps if needed
ensure_npm_deps() {
  cd "$UI_DIR"
  if [ ! -d "node_modules" ] || [ ! -f "node_modules/.package-lock.json" ]; then
    log "Installing npm dependencies..."
    npm install 2>&1
    ok "npm install complete"
  else
    ok "node_modules already installed"
  fi
}

# ── Cleanup ──────────────────────────────────────────────────────────────────

# Trap handler for graceful shutdown
setup_shutdown_trap() {
  local pids=("$@")
  trap '
    echo ""
    log "Shutting down..."
    for pid in '"${pids[*]}"'; do
      kill "$pid" 2>/dev/null || true
    done
    log "Done."
    exit 0
  ' INT TERM
}
