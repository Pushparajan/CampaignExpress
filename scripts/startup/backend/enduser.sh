#!/usr/bin/env bash
# =============================================================================
# Campaign Express — End User Backend Startup
# =============================================================================
#
# Starts a lightweight backend optimized for consumer-facing API traffic:
#   - Bid/offer serving endpoint (high-throughput, low-latency)
#   - Personalization and segmentation APIs
#   - Loyalty point balance / redemption endpoints
#   - Channel delivery (SMS, push, email triggers)
#   - API-only mode (no management endpoints needed)
#   - Minimal logging (warn level) for maximum throughput
#
# This profile is tuned for end-user traffic patterns:
#   - NPU inference enabled for real-time personalization
#   - Higher agent count for throughput
#   - Read-heavy cache configuration
#
# Ports:
#   HTTP API     :8082    REST endpoints (consumer-facing subset)
#   gRPC         :9292    gRPC service
#   Metrics      :9094    Prometheus scrape endpoint
#   Redis        :6379    Cache (shared)
#   ClickHouse   :8123    Event logging (write-only from end-user perspective)
#
# Usage:
#   ./scripts/startup/backend/enduser.sh                   # Standard start
#   ./scripts/startup/backend/enduser.sh --no-seed         # Skip data seeding
#   ./scripts/startup/backend/enduser.sh --skip-build      # Skip cargo build
#   ./scripts/startup/backend/enduser.sh --with-agents     # Enable NATS agents
#   ./scripts/startup/backend/enduser.sh --npu <device>    # NPU device (cpu|xdna|gpu)
#   ./scripts/startup/backend/enduser.sh --high-throughput # Tuned for load testing
#
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"

# ── Flags ────────────────────────────────────────────────────────────────────

NO_SEED=false
SKIP_BUILD=false
WITH_AGENTS=false
NPU_DEVICE="cpu"
HIGH_THROUGHPUT=false

for arg in "$@"; do
  case "$arg" in
    --no-seed)          NO_SEED=true ;;
    --skip-build)       SKIP_BUILD=true ;;
    --with-agents)      WITH_AGENTS=true ;;
    --high-throughput)  HIGH_THROUGHPUT=true ;;
    --npu)              shift_npu=true ;;
    --help|-h)
      sed -n '2,/^# ====/{ /^#/s/^# \?//p }' "$0"
      exit 0 ;;
    *)
      if [ "${shift_npu:-false}" = "true" ]; then
        NPU_DEVICE="$arg"
        shift_npu=false
      else
        echo "Unknown option: $arg"; exit 1
      fi
      ;;
  esac
done

# ── Main ─────────────────────────────────────────────────────────────────────

banner "End User" "Backend"

section "Checking Prerequisites"
check_rust
check_docker

# Build
if ! $SKIP_BUILD; then
  section "Building Workspace (release mode for end-user throughput)"
  cd "$PROJECT_ROOT"
  if $HIGH_THROUGHPUT; then
    log "cargo build --release --workspace ..."
    cargo build --release --workspace 2>&1
  else
    log "cargo build --workspace ..."
    cargo build --workspace 2>&1
  fi
  ok "Build complete"
fi

# Infrastructure — core only
start_core_infra

if $WITH_AGENTS; then
  setup_nats
fi

# Seed (end-user needs campaigns, offers, segments in cache)
if ! $NO_SEED; then
  seed_data
fi

# End-user-specific environment — tuned for throughput
export CAMPAIGN_EXPRESS__NODE_ID="enduser-node-01"
export CAMPAIGN_EXPRESS__NPU__DEVICE="$NPU_DEVICE"
export RUST_LOG="campaign_express=warn,tower_http=warn"

if $HIGH_THROUGHPUT; then
  export CAMPAIGN_EXPRESS__AGENTS_PER_NODE="${CAMPAIGN_EXPRESS__AGENTS_PER_NODE:-20}"
  export CAMPAIGN_EXPRESS__NPU__BATCH_SIZE="${CAMPAIGN_EXPRESS__NPU__BATCH_SIZE:-64}"
  export CAMPAIGN_EXPRESS__NPU__NUM_THREADS="${CAMPAIGN_EXPRESS__NPU__NUM_THREADS:-8}"
  export CAMPAIGN_EXPRESS__REDIS__POOL_SIZE="${CAMPAIGN_EXPRESS__REDIS__POOL_SIZE:-32}"
  export CAMPAIGN_EXPRESS__CLICKHOUSE__BATCH_SIZE="${CAMPAIGN_EXPRESS__CLICKHOUSE__BATCH_SIZE:-5000}"
  log "High-throughput mode: 20 agents, batch=64, 8 threads, pool=32"
else
  export CAMPAIGN_EXPRESS__AGENTS_PER_NODE="${CAMPAIGN_EXPRESS__AGENTS_PER_NODE:-4}"
fi

# Build extra flags
EXTRA_FLAGS=()
if ! $WITH_AGENTS; then
  EXTRA_FLAGS+=(--api-only)
fi

if $WITH_AGENTS; then
  export CAMPAIGN_EXPRESS__NATS__URLS="nats://localhost:4222"
fi

# End-user backend on offset ports
start_rust_backend "enduser" 8082 9292 9094 "${EXTRA_FLAGS[@]}"

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo -e "${CYAN}==================================================================${NC}"
echo -e "${CYAN}  End User Backend Running${NC}"
echo -e "${CYAN}==================================================================${NC}"
cat <<EOF

  Core Services:
    REST API        http://localhost:8082     (consumer-facing endpoints)
    gRPC            localhost:9292
    Metrics         http://localhost:9094/metrics
    NPU Device      $NPU_DEVICE

  Infrastructure:
    Redis           redis://localhost:6379
    ClickHouse      http://localhost:8123

  End User Endpoints:
    GET  /health                    Health check
    POST /v1/bid                    Real-time bid/offer serving
    *    /api/loyalty/*             Loyalty balance & redemption
    *    /api/channels/*            Channel delivery triggers

  Traffic Profile:
    - Real-time offer personalization (NPU-accelerated)
    - High-throughput bid serving (<10ms p99 target)
    - Segment-based targeting lookups
    - Event logging to ClickHouse (async, batched)

  Load Test:
    ./scripts/load-test.sh http://localhost:8082 50 1000

  Press Ctrl+C to stop.

EOF

# Keep alive
setup_shutdown_trap "$BACKEND_PID"
wait "$BACKEND_PID" 2>/dev/null || true
