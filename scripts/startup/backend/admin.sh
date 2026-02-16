#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Admin Backend Startup
# =============================================================================
#
# Starts the full backend stack with all services enabled for administrators:
#   - Full bid processing pipeline with NATS agents
#   - All REST + gRPC endpoints (campaigns, DSP, loyalty, channels, management)
#   - Monitoring stack (Prometheus + Grafana)
#   - Debug-level logging for system diagnostics
#   - User management, tenant admin, billing, and ops endpoints
#
# Ports:
#   HTTP API     :8080    REST endpoints (all routes)
#   gRPC         :9090    gRPC service
#   Metrics      :9091    Prometheus scrape endpoint
#   NATS         :4222    Agent message broker
#   NATS Monitor :8222    NATS management UI
#   Redis        :6379    Cache (L2)
#   ClickHouse   :8123    Analytics queries
#   Prometheus   :9092    Metrics dashboard
#   Grafana      :3000    Visualization (admin/campaign-express)
#
# Usage:
#   ./scripts/startup/backend/admin.sh                 # Full start
#   ./scripts/startup/backend/admin.sh --no-seed       # Skip data seeding
#   ./scripts/startup/backend/admin.sh --no-monitor    # Skip Prometheus/Grafana
#   ./scripts/startup/backend/admin.sh --skip-build    # Skip cargo build
#   ./scripts/startup/backend/admin.sh --debug         # Extra verbose logging
#
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"

# ── Flags ────────────────────────────────────────────────────────────────────

NO_SEED=false
NO_MONITOR=false
SKIP_BUILD=false
DEBUG_MODE=false

for arg in "$@"; do
  case "$arg" in
    --no-seed)     NO_SEED=true ;;
    --no-monitor)  NO_MONITOR=true ;;
    --skip-build)  SKIP_BUILD=true ;;
    --debug)       DEBUG_MODE=true ;;
    --help|-h)
      sed -n '2,/^# ====/{ /^#/s/^# \?//p }' "$0"
      exit 0 ;;
    *) echo "Unknown option: $arg"; exit 1 ;;
  esac
done

# ── Main ─────────────────────────────────────────────────────────────────────

banner "Admin" "Backend"

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

# Infrastructure
start_core_infra
setup_nats

if ! $NO_MONITOR; then
  start_monitoring
fi

# Seed
if ! $NO_SEED; then
  seed_data
fi

# Admin-specific environment
export CAMPAIGN_EXPRESS__NODE_ID="admin-node-01"
export CAMPAIGN_EXPRESS__AGENTS_PER_NODE="${CAMPAIGN_EXPRESS__AGENTS_PER_NODE:-8}"
export CAMPAIGN_EXPRESS__NATS__URLS="nats://localhost:4222"

if $DEBUG_MODE; then
  export RUST_LOG="campaign_express=debug,tower_http=debug,hyper=info"
else
  export RUST_LOG="campaign_express=info,tower_http=info"
fi

# Start with full agents (admin gets the complete pipeline)
start_rust_backend "admin" 8080 9090 9091

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo -e "${CYAN}==================================================================${NC}"
echo -e "${CYAN}  Admin Backend Running${NC}"
echo -e "${CYAN}==================================================================${NC}"
cat <<'EOF'

  Core Services:
    REST API        http://localhost:8080     (all endpoints)
    gRPC            localhost:9090
    Metrics         http://localhost:9091/metrics

  Infrastructure:
    NATS            nats://localhost:4222     (monitor: http://localhost:8222)
    Redis           redis://localhost:6379
    ClickHouse      http://localhost:8123
EOF

if ! $NO_MONITOR; then
  cat <<'EOF'

  Monitoring:
    Prometheus      http://localhost:9092
    Grafana         http://localhost:3000     (admin / campaign-express)
EOF
fi

cat <<'EOF'

  Admin Endpoints:
    GET  /health                    Health check
    GET  /ready                     Readiness probe
    GET  /metrics                   Prometheus metrics
    POST /v1/bid                    Bid processing
    *    /api/campaigns/*           Campaign CRUD
    *    /api/loyalty/*             Loyalty management
    *    /api/dsp/*                 DSP integrations
    *    /api/channels/*            Channel management

  Press Ctrl+C to stop.

EOF

# Keep alive
setup_shutdown_trap "$BACKEND_PID"
wait "$BACKEND_PID" 2>/dev/null || true
