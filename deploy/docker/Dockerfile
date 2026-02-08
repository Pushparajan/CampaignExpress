# =============================================================================
# Campaign Express — Multi-stage Docker build
# =============================================================================
# Stage 1: Build the Rust binary
# Stage 2: Minimal runtime image
# =============================================================================

# -- Builder stage --
FROM rust:1.77-bookworm AS builder

WORKDIR /build

# Install protobuf compiler for gRPC proto compilation
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first for dependency caching
COPY Cargo.toml Cargo.lock* ./
COPY crates/core/Cargo.toml crates/core/Cargo.toml
COPY crates/npu-engine/Cargo.toml crates/npu-engine/Cargo.toml
COPY crates/agents/Cargo.toml crates/agents/Cargo.toml
COPY crates/cache/Cargo.toml crates/cache/Cargo.toml
COPY crates/analytics/Cargo.toml crates/analytics/Cargo.toml
COPY crates/api-server/Cargo.toml crates/api-server/Cargo.toml
COPY crates/wasm-edge/Cargo.toml crates/wasm-edge/Cargo.toml
COPY src/campaign-express/Cargo.toml src/campaign-express/Cargo.toml

# Create dummy source files for dependency caching
RUN mkdir -p crates/core/src && echo "pub fn _dummy() {}" > crates/core/src/lib.rs && \
    mkdir -p crates/npu-engine/src && echo "pub fn _dummy() {}" > crates/npu-engine/src/lib.rs && \
    mkdir -p crates/agents/src && echo "pub fn _dummy() {}" > crates/agents/src/lib.rs && \
    mkdir -p crates/cache/src && echo "pub fn _dummy() {}" > crates/cache/src/lib.rs && \
    mkdir -p crates/analytics/src && echo "pub fn _dummy() {}" > crates/analytics/src/lib.rs && \
    mkdir -p crates/api-server/src && echo "pub fn _dummy() {}" > crates/api-server/src/lib.rs && \
    mkdir -p crates/wasm-edge/src && echo "pub fn _dummy() {}" > crates/wasm-edge/src/lib.rs && \
    mkdir -p src/campaign-express/src && echo "fn main() {}" > src/campaign-express/src/main.rs && \
    mkdir -p crates/api-server/src/generated && touch crates/api-server/src/generated/.gitkeep

# Cache dependencies
RUN cargo build --release --workspace 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/
COPY src/ src/
COPY proto/ proto/

# Build the actual binary
RUN cargo build --release --bin campaign-express

# -- Runtime stage --
FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r campaign && useradd -r -g campaign campaign

# Create directories
RUN mkdir -p /app /models /config && chown -R campaign:campaign /app /models /config

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/campaign-express /app/campaign-express

# Default configuration
ENV CAMPAIGN_EXPRESS__NODE_ID=node-01 \
    CAMPAIGN_EXPRESS__AGENTS_PER_NODE=20 \
    CAMPAIGN_EXPRESS__API__HOST=0.0.0.0 \
    CAMPAIGN_EXPRESS__API__HTTP_PORT=8080 \
    CAMPAIGN_EXPRESS__API__GRPC_PORT=9090 \
    CAMPAIGN_EXPRESS__METRICS__PORT=9091 \
    CAMPAIGN_EXPRESS__NPU__MODEL_PATH=/models/colanet.onnx \
    CAMPAIGN_EXPRESS__NPU__DEVICE=cpu \
    RUST_LOG=campaign_express=info

USER campaign

EXPOSE 8080 9090 9091

HEALTHCHECK --interval=10s --timeout=3s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["/app/campaign-express"]
