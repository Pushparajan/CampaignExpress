#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Development Environment Setup
#
# Sets up everything needed to build and run Campaign Express locally:
#   1. Verifies system prerequisites (Rust, Docker, kubectl, etc.)
#   2. Installs Rust toolchain components
#   3. Builds the workspace
#   4. Optionally starts infrastructure via Docker Compose
#   5. Runs tests to verify the setup
#
# Usage:
#   ./scripts/setup.sh              # Full setup
#   ./scripts/setup.sh --check      # Only verify prerequisites
#   ./scripts/setup.sh --no-docker  # Skip Docker infrastructure
#   ./scripts/setup.sh --no-build   # Skip cargo build
# =============================================================================

set -euo pipefail

# --- Configuration ---
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MIN_RUST_VERSION="1.75.0"
COMPOSE_FILE="$PROJECT_ROOT/deploy/docker/docker-compose.yml"

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# --- Flags ---
CHECK_ONLY=false
SKIP_DOCKER=false
SKIP_BUILD=false

for arg in "$@"; do
    case "$arg" in
        --check)      CHECK_ONLY=true ;;
        --no-docker)  SKIP_DOCKER=true ;;
        --no-build)   SKIP_BUILD=true ;;
        --help|-h)
            echo "Usage: $0 [--check] [--no-docker] [--no-build]"
            echo ""
            echo "Options:"
            echo "  --check      Only verify prerequisites, don't install or build"
            echo "  --no-docker  Skip starting Docker Compose infrastructure"
            echo "  --no-build   Skip cargo build step"
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            exit 1
            ;;
    esac
done

# --- Helpers ---
info()    { echo -e "${BLUE}[INFO]${NC}  $*"; }
success() { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $*"; }
fail()    { echo -e "${RED}[FAIL]${NC}  $*"; }

check_cmd() {
    if command -v "$1" &>/dev/null; then
        success "$1 found: $(command -v "$1")"
        return 0
    else
        fail "$1 not found"
        return 1
    fi
}

version_gte() {
    # Returns 0 if $1 >= $2 (semantic version comparison)
    printf '%s\n%s' "$2" "$1" | sort -V -C
}

# =============================================================================
echo ""
echo "============================================="
echo "  Campaign Express — Development Setup"
echo "============================================="
echo ""

# --- Step 1: Check prerequisites ---
info "Checking prerequisites..."
echo ""

ERRORS=0

# Rust toolchain
if check_cmd rustc; then
    RUST_VER=$(rustc --version | awk '{print $2}')
    if version_gte "$RUST_VER" "$MIN_RUST_VERSION"; then
        success "Rust version $RUST_VER (>= $MIN_RUST_VERSION)"
    else
        fail "Rust version $RUST_VER is below minimum $MIN_RUST_VERSION"
        ERRORS=$((ERRORS + 1))
    fi
else
    fail "Rust is not installed. Install from https://rustup.rs"
    ERRORS=$((ERRORS + 1))
fi

check_cmd cargo || ERRORS=$((ERRORS + 1))

# Docker (optional but recommended)
DOCKER_AVAILABLE=false
if check_cmd docker; then
    if docker info &>/dev/null; then
        success "Docker daemon is running"
        DOCKER_AVAILABLE=true
    else
        warn "Docker is installed but the daemon is not running"
    fi
else
    warn "Docker not found — needed for compose-based local dev"
fi

# Docker Compose
if $DOCKER_AVAILABLE; then
    if docker compose version &>/dev/null; then
        success "Docker Compose (plugin) available"
    elif check_cmd docker-compose; then
        success "docker-compose (standalone) available"
    else
        warn "Docker Compose not found — needed for local infrastructure"
    fi
fi

# Optional tools
echo ""
info "Optional tools:"
check_cmd kubectl   || warn "kubectl not found — needed for K8s deployment"
check_cmd kustomize || warn "kustomize not found — can use 'kubectl -k' instead"
check_cmd nats      || warn "nats CLI not found — needed for stream setup script"
check_cmd redis-cli || warn "redis-cli not found — useful for debugging cache"
check_cmd curl      || warn "curl not found — needed for load testing"

echo ""

if [ "$ERRORS" -gt 0 ]; then
    fail "$ERRORS required tool(s) missing. Please install them and re-run."
    exit 1
fi

success "All required prerequisites met."

if $CHECK_ONLY; then
    echo ""
    info "Check-only mode — exiting."
    exit 0
fi

# --- Step 2: Rust toolchain components ---
echo ""
info "Setting up Rust toolchain components..."

if command -v rustup &>/dev/null; then
    rustup component add clippy rustfmt 2>/dev/null && \
        success "clippy and rustfmt installed" || \
        warn "Could not install clippy/rustfmt via rustup"
else
    warn "rustup not available — skipping component install"
fi

# --- Step 3: Create .env from example if missing ---
echo ""
if [ ! -f "$PROJECT_ROOT/.env" ] && [ -f "$PROJECT_ROOT/.env.example" ]; then
    cp "$PROJECT_ROOT/.env.example" "$PROJECT_ROOT/.env"
    success "Created .env from .env.example"
else
    if [ -f "$PROJECT_ROOT/.env" ]; then
        success ".env already exists"
    else
        warn "No .env.example found to copy"
    fi
fi

# --- Step 4: Build the workspace ---
if ! $SKIP_BUILD; then
    echo ""
    info "Building the workspace..."
    cd "$PROJECT_ROOT"

    if cargo build --workspace 2>&1; then
        success "Workspace build succeeded"
    else
        fail "Workspace build failed"
        exit 1
    fi

    echo ""
    info "Running tests..."
    if cargo test --workspace 2>&1; then
        success "All tests passed"
    else
        fail "Tests failed"
        exit 1
    fi
else
    info "Skipping build (--no-build)"
fi

# --- Step 5: Start Docker infrastructure ---
if ! $SKIP_DOCKER && $DOCKER_AVAILABLE; then
    echo ""
    info "Starting local infrastructure via Docker Compose..."
    info "  Services: NATS, Redis, ClickHouse, Prometheus, Grafana"

    cd "$PROJECT_ROOT"

    # Start just the infrastructure (not the app itself — we'll run that natively)
    docker compose -f "$COMPOSE_FILE" up -d \
        nats redis clickhouse prometheus grafana 2>&1 && \
        success "Infrastructure containers started" || \
        warn "Docker Compose failed — you can start infra manually later"

    echo ""
    info "Waiting for services to become healthy..."
    sleep 5

    # Check health of each service
    for svc in nats redis clickhouse; do
        if docker compose -f "$COMPOSE_FILE" ps "$svc" 2>/dev/null | grep -q "healthy"; then
            success "$svc is healthy"
        elif docker compose -f "$COMPOSE_FILE" ps "$svc" 2>/dev/null | grep -q "Up"; then
            success "$svc is up (health check pending)"
        else
            warn "$svc may not be ready yet"
        fi
    done
elif $SKIP_DOCKER; then
    info "Skipping Docker infrastructure (--no-docker)"
else
    warn "Docker not available — skipping infrastructure setup"
    warn "You'll need NATS, Redis, and ClickHouse running to use the full platform"
fi

# --- Done ---
echo ""
echo "============================================="
echo "  Setup Complete"
echo "============================================="
echo ""
info "Quick start commands:"
echo ""
echo "  # Run the API server locally (no external deps needed):"
echo "  make run-local"
echo ""
echo "  # Run with full infrastructure (Docker required):"
echo "  make compose-up"
echo ""
echo "  # Run tests:"
echo "  make test"
echo ""
echo "  # Run linter:"
echo "  make lint"
echo ""
echo "  # Send a test bid request:"
echo "  curl -X POST http://localhost:8080/v1/bid \\"
echo "    -H 'Content-Type: application/json' \\"
echo "    -d '{\"id\":\"test-1\",\"imp\":[{\"id\":\"imp-1\",\"banner\":{\"w\":300,\"h\":250},\"bidfloor\":0.5}],\"user\":{\"id\":\"user-1\"},\"tmax\":100}'"
echo ""

if $DOCKER_AVAILABLE && ! $SKIP_DOCKER; then
    echo "  Services running:"
    echo "    NATS:       nats://localhost:4222  (monitor: http://localhost:8222)"
    echo "    Redis:      redis://localhost:6379"
    echo "    ClickHouse: http://localhost:8123"
    echo "    Prometheus:  http://localhost:9092"
    echo "    Grafana:     http://localhost:3000  (admin / campaign-express)"
    echo ""
fi

success "Campaign Express is ready for development."
