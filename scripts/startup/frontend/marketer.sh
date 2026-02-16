#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Marketer Frontend Startup
# =============================================================================
#
# Starts the Next.js frontend configured for the Marketer persona:
#   - Campaign creation, editing, scheduling, and governance
#   - Creative asset management and DCO variants
#   - Journey builder and orchestration
#   - Experiment (A/B test) management
#   - Analytics dashboards and reporting
#   - Loyalty program configuration
#   - Segmentation and audience management
#   - No access to: platform admin, ops, billing, user management
#
# Environment:
#   NEXT_PUBLIC_API_URL       Backend REST API base URL
#   NEXT_PUBLIC_USER_ROLE     Role identifier for UI feature gating
#
# Port: 3002
#
# Usage:
#   ./scripts/startup/frontend/marketer.sh                     # Dev server
#   ./scripts/startup/frontend/marketer.sh --prod              # Production build
#   ./scripts/startup/frontend/marketer.sh --port 3005         # Custom port
#   ./scripts/startup/frontend/marketer.sh --api-url <url>     # Custom backend URL
#
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"

# ── Flags ────────────────────────────────────────────────────────────────────

PROD_MODE=false
PORT=3002
API_URL="http://localhost:8081"

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

banner "Marketer" "Frontend"

section "Checking Prerequisites"
check_node

section "Preparing Marketer Frontend"
ensure_npm_deps

# Write role-specific .env.local
cd "$UI_DIR"
cat > .env.local <<ENVEOF
# Marketer Frontend Configuration (auto-generated)
NEXT_PUBLIC_API_URL=${API_URL}
NEXT_PUBLIC_WS_URL=ws://localhost:8081/ws
NEXT_PUBLIC_USER_ROLE=marketer
NEXT_PUBLIC_FEATURES_ENABLED=campaigns,creatives,journeys,experiments,dco,cdp,analytics,loyalty,segmentation,personalization,channels
ENVEOF
ok "Wrote ui/.env.local (role=marketer)"

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
wait_for_port localhost "$PORT" "Marketer Frontend" 30

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo -e "${CYAN}==================================================================${NC}"
echo -e "${CYAN}  Marketer Frontend Running${NC}"
echo -e "${CYAN}==================================================================${NC}"
cat <<EOF

  Frontend        http://localhost:${PORT}
  Backend API     ${API_URL}
  Mode            $(if $PROD_MODE; then echo "Production"; else echo "Development (hot-reload)"; fi)
  Role            marketer

  Available Sections:
    /                   Dashboard (campaign performance overview)
    /campaigns          Campaign management
    /experiments        A/B testing
    /journeys           Journey orchestration
    /dco                Dynamic Creative Optimization
    /cdp                Customer Data Platform
    /login              Authentication

  Marketer Workflows:
    - Create and schedule campaigns with budget controls
    - Design creative variants for DCO
    - Build customer journeys with triggers
    - Run experiments and analyze results
    - View audience segments and targeting rules
    - Manage loyalty program rules and rewards
    - Track campaign analytics and ROI

  Press Ctrl+C to stop.

EOF

setup_shutdown_trap "$FRONTEND_PID"
wait "$FRONTEND_PID" 2>/dev/null || true
