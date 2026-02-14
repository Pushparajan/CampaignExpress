#!/usr/bin/env bash
# =============================================================================
# Campaign Express — One-Command Quickstart
# =============================================================================
#
# Gets the entire platform running locally with a single command.
#
# What this does:
#   1. Checks prerequisites (Rust, Docker, Node.js)
#   2. Builds the Rust workspace (~2-4 min on first run)
#   3. Starts infrastructure (NATS, Redis, ClickHouse, Prometheus, Grafana)
#   4. Seeds test data (campaigns, users, bid events)
#   5. Starts the backend API server
#   6. Runs smoke tests to verify everything works
#   7. Prints service URLs and next steps
#
# Usage:
#   ./quickstart.sh                 # Full setup + start
#   ./quickstart.sh --check         # Just check if prerequisites are installed
#   ./quickstart.sh --no-build      # Skip Rust build (if already built)
#   ./quickstart.sh --no-frontend   # Skip frontend npm install
#   ./quickstart.sh --reset         # Wipe all data and start fresh
#
# Prerequisites:
#   - Rust 1.77+     https://rustup.rs
#   - Docker         https://docs.docker.com/get-docker/
#   - Node.js 18+    https://nodejs.org (optional, for management UI)
#
# Detailed install instructions: docs/PREREQUISITES.md
# Full deployment guide:         docs/LOCAL_DEPLOYMENT.md
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Pass all arguments through to the full setup script
exec "$SCRIPT_DIR/scripts/local-setup.sh" "$@"
