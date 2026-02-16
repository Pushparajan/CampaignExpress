#!/usr/bin/env bash
# =============================================================================
# Campaign Express — End User Frontend Startup
# =============================================================================
#
# Starts the Next.js frontend configured for the End User (consumer) persona:
#   - Personalized offer/deal browsing experience
#   - Loyalty program dashboard (points, tiers, redemption)
#   - Preference management and consent settings
#   - Campaign opt-in/opt-out controls
#   - Channel delivery preferences (email, SMS, push)
#   - No access to: campaign management, admin, ops, billing, experiments
#
# This is the consumer-facing portal where end users interact with
# personalized offers, manage their loyalty accounts, and control
# their communication preferences.
#
# Environment:
#   NEXT_PUBLIC_API_URL       Backend REST API base URL
#   NEXT_PUBLIC_USER_ROLE     Role identifier for UI feature gating
#
# Port: 3003
#
# Usage:
#   ./scripts/startup/frontend/enduser.sh                      # Dev server
#   ./scripts/startup/frontend/enduser.sh --prod               # Production build
#   ./scripts/startup/frontend/enduser.sh --port 3005          # Custom port
#   ./scripts/startup/frontend/enduser.sh --api-url <url>      # Custom backend URL
#
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"

# ── Flags ────────────────────────────────────────────────────────────────────

PROD_MODE=false
PORT=3003
API_URL="http://localhost:8082"

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

banner "End User" "Frontend"

section "Checking Prerequisites"
check_node

section "Preparing End User Frontend"
ensure_npm_deps

# Write role-specific .env.local
cd "$UI_DIR"
cat > .env.local <<ENVEOF
# End User Frontend Configuration (auto-generated)
NEXT_PUBLIC_API_URL=${API_URL}
NEXT_PUBLIC_WS_URL=ws://localhost:8082/ws
NEXT_PUBLIC_USER_ROLE=enduser
NEXT_PUBLIC_FEATURES_ENABLED=loyalty,personalization,channels
ENVEOF
ok "Wrote ui/.env.local (role=enduser)"

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
wait_for_port localhost "$PORT" "End User Frontend" 30

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo -e "${CYAN}==================================================================${NC}"
echo -e "${CYAN}  End User Frontend Running${NC}"
echo -e "${CYAN}==================================================================${NC}"
cat <<EOF

  Frontend        http://localhost:${PORT}
  Backend API     ${API_URL}
  Mode            $(if $PROD_MODE; then echo "Production"; else echo "Development (hot-reload)"; fi)
  Role            enduser

  Available Sections:
    /                   Personalized offers & deals
    /login              User authentication

  End User Features:
    - Browse personalized offers and deals
    - View loyalty points, tier status, and rewards
    - Redeem loyalty points for rewards
    - Manage communication preferences (email, SMS, push)
    - Update consent and privacy settings
    - View offer history and saved deals

  Press Ctrl+C to stop.

EOF

setup_shutdown_trap "$FRONTEND_PID"
wait "$FRONTEND_PID" 2>/dev/null || true
