#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Admin Frontend Startup
# =============================================================================
#
# Starts the Next.js frontend configured for the Admin persona:
#   - Full dashboard with all navigation sections
#   - Platform administration (users, tenants, RBAC)
#   - Ops dashboard (incidents, SLA, backups, status page)
#   - Billing management
#   - System monitoring links (Grafana, Prometheus)
#   - All campaign, creative, journey, and experiment management
#   - CDP, DCO, and segmentation views
#
# Environment:
#   NEXT_PUBLIC_API_URL       Backend REST API base URL
#   NEXT_PUBLIC_WS_URL        WebSocket URL for real-time updates
#   NEXT_PUBLIC_USER_ROLE     Role identifier for UI feature gating
#   NEXT_PUBLIC_GRAFANA_URL   Grafana dashboard URL
#   NEXT_PUBLIC_PROMETHEUS_URL Prometheus URL
#
# Port: 3001 (offset from Grafana on 3000)
#
# Usage:
#   ./scripts/startup/frontend/admin.sh                    # Dev server
#   ./scripts/startup/frontend/admin.sh --prod             # Production build + start
#   ./scripts/startup/frontend/admin.sh --port 3005        # Custom port
#   ./scripts/startup/frontend/admin.sh --api-url <url>    # Custom backend URL
#
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"

# ── Flags ────────────────────────────────────────────────────────────────────

PROD_MODE=false
PORT=3001
API_URL="http://localhost:8080"

i=0
args=("$@")
while (( i < ${#args[@]} )); do
  case "${args[$i]}" in
    --prod)      PROD_MODE=true ;;
    --port)      i=$((i + 1)); PORT="${args[$i]}" ;;
    --api-url)   i=$((i + 1)); API_URL="${args[$i]}" ;;
    --help|-h)
      sed -n '2,/^# ====/{ /^#/s/^# \?//p }' "$0"
      exit 0 ;;
    *) echo "Unknown option: ${args[$i]}"; exit 1 ;;
  esac
  i=$((i + 1))
done

# ── Main ─────────────────────────────────────────────────────────────────────

banner "Admin" "Frontend"

section "Checking Prerequisites"
check_node

section "Preparing Admin Frontend"
ensure_npm_deps

# Write role-specific .env.local
cd "$UI_DIR"
cat > .env.local <<ENVEOF
# Admin Frontend Configuration (auto-generated)
NEXT_PUBLIC_API_URL=${API_URL}
NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws
NEXT_PUBLIC_USER_ROLE=admin
NEXT_PUBLIC_FEATURES_ENABLED=campaigns,creatives,journeys,experiments,dco,cdp,analytics,billing,platform,ops,users,loyalty,dsp,channels,segmentation,personalization
NEXT_PUBLIC_GRAFANA_URL=http://localhost:3000
NEXT_PUBLIC_PROMETHEUS_URL=http://localhost:9092
NEXT_PUBLIC_NATS_MONITOR_URL=http://localhost:8222
NEXT_PUBLIC_CLICKHOUSE_URL=http://localhost:8123
ENVEOF
ok "Wrote ui/.env.local (role=admin)"

# Start
if $PROD_MODE; then
  section "Building Production Frontend"
  log "npm run build ..."
  npm run build 2>&1
  ok "Build complete"

  section "Starting Production Server (port $PORT)"
  PORT=$PORT npm start &
  FRONTEND_PID=$!
else
  section "Starting Development Server (port $PORT)"
  PORT=$PORT npm run dev &
  FRONTEND_PID=$!
fi

# Wait for frontend
sleep 3
wait_for_port localhost "$PORT" "Admin Frontend" 30

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo -e "${CYAN}==================================================================${NC}"
echo -e "${CYAN}  Admin Frontend Running${NC}"
echo -e "${CYAN}==================================================================${NC}"
cat <<EOF

  Frontend        http://localhost:${PORT}
  Backend API     ${API_URL}
  Mode            $(if $PROD_MODE; then echo "Production"; else echo "Development (hot-reload)"; fi)
  Role            admin

  Available Sections:
    /                   Dashboard (full system overview)
    /campaigns          Campaign management (CRUD + governance)
    /experiments        A/B testing & experimentation
    /journeys           Journey orchestration
    /dco                Dynamic Creative Optimization
    /cdp                Customer Data Platform
    /billing            Billing & usage
    /platform           User management, tenants, RBAC
    /ops                Incidents, SLA, backups, status page
    /users              User administration
    /login              Authentication

  Monitoring Links:
    Grafana             http://localhost:3000
    Prometheus          http://localhost:9092
    NATS Monitor        http://localhost:8222

  Press Ctrl+C to stop.

EOF

setup_shutdown_trap "$FRONTEND_PID"
wait "$FRONTEND_PID" 2>/dev/null || true
