#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Marketer Backend Startup
# =============================================================================
#
# Starts the backend optimized for marketing team workflows:
#   - Campaign management, creative, and journey APIs
#   - Analytics endpoints (ClickHouse queries)
#   - DCO (Dynamic Creative Optimization) and personalization
#   - Experiment / A-B testing endpoints
#   - API-only mode (no NATS agents — marketers don't run bid agents)
#   - Standard logging level
#
# Ports:
#   HTTP API     :8081    REST endpoints (marketing subset)
#   gRPC         :9091    gRPC service
#   Metrics      :9093    Prometheus scrape endpoint
#   Redis        :6379    Cache (shared)
#   ClickHouse   :8123    Analytics queries
#
# Usage:
#   ./scripts/startup/backend/marketer.sh                  # Standard start
#   ./scripts/startup/backend/marketer.sh --no-seed        # Skip data seeding
#   ./scripts/startup/backend/marketer.sh --skip-build     # Skip cargo build
#   ./scripts/startup/backend/marketer.sh --with-agents    # Enable NATS agents
#   ./scripts/startup/backend/marketer.sh --debug          # Verbose logging
#
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"

# ── Flags ────────────────────────────────────────────────────────────────────

NO_SEED=false
SKIP_BUILD=false
WITH_AGENTS=false
DEBUG_MODE=false

for arg in "$@"; do
  case "$arg" in
    --no-seed)     NO_SEED=true ;;
    --skip-build)  SKIP_BUILD=true ;;
    --with-agents) WITH_AGENTS=true ;;
    --debug)       DEBUG_MODE=true ;;
    --help|-h)
      sed -n '2,/^# ====/{ /^#/s/^# \?//p }' "$0"
      exit 0 ;;
    *) echo "Unknown option: $arg"; exit 1 ;;
  esac
done

# ── Main ─────────────────────────────────────────────────────────────────────

banner "Marketer" "Backend"

section "Checking Prerequisites"
check_rust
check_docker

# Build
if ! $SKIP_BUILD; then
  section "Building Workspace"
  cd "$PROJECT_ROOT"
  log "cargo build --workspace ..."
  cargo build --workspace 2>&1
  ok "Build complete"
fi

# Infrastructure — core only (no monitoring for marketers by default)
start_core_infra

if $WITH_AGENTS; then
  setup_nats
fi

# Seed
if ! $NO_SEED; then
  seed_data
fi

# Marketer-specific environment
export CAMPAIGN_EXPRESS__NODE_ID="marketer-node-01"
export CAMPAIGN_EXPRESS__AGENTS_PER_NODE="${CAMPAIGN_EXPRESS__AGENTS_PER_NODE:-2}"

if $DEBUG_MODE; then
  export RUST_LOG="campaign_express=debug,tower_http=debug"
else
  export RUST_LOG="campaign_express=info,tower_http=warn"
fi

# Build extra flags
EXTRA_FLAGS=()
if ! $WITH_AGENTS; then
  EXTRA_FLAGS+=(--api-only)
fi

if $WITH_AGENTS; then
  export CAMPAIGN_EXPRESS__NATS__URLS="nats://localhost:4222"
fi

# Marketer backend on offset ports to allow co-running with admin
start_rust_backend "marketer" 8081 9191 9093 "${EXTRA_FLAGS[@]}"

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo -e "${CYAN}==================================================================${NC}"
echo -e "${CYAN}  Marketer Backend Running${NC}"
echo -e "${CYAN}==================================================================${NC}"
cat <<'EOF'

  Core Services:
    REST API        http://localhost:8081     (campaign & analytics endpoints)
    gRPC            localhost:9191
    Metrics         http://localhost:9093/metrics

  Infrastructure:
    Redis           redis://localhost:6379
    ClickHouse      http://localhost:8123

  Marketer Endpoints:
    GET  /health                    Health check
    POST /v1/bid                    Test bid (preview mode)
    *    /api/campaigns/*           Campaign management
    *    /api/loyalty/*             Loyalty programs
    *    /api/dsp/*                 DSP preview
    *    /api/channels/*            Channel configuration

  Use Cases:
    - Create / edit / schedule campaigns
    - Manage creative assets and DCO variants
    - View analytics dashboards (ClickHouse)
    - Configure journey orchestration
    - Run A/B experiments
    - Manage loyalty program rules

  Press Ctrl+C to stop.

EOF

# Keep alive
setup_shutdown_trap "$BACKEND_PID"
wait "$BACKEND_PID" 2>/dev/null || true
