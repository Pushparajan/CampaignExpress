#!/usr/bin/env bash
# =============================================================================
# Set up NATS JetStream streams for Campaign Express
# Requires: nats CLI tool
# =============================================================================

set -euo pipefail

NATS_URL="${1:-nats://localhost:4222}"

echo "Setting up NATS JetStream streams at $NATS_URL"

# Create the bid request stream
nats --server="$NATS_URL" stream add campaign-bids \
    --subjects="campaign-bids.>" \
    --storage=file \
    --retention=work \
    --max-msgs=-1 \
    --max-bytes=10GB \
    --max-age=1h \
    --max-msg-size=1MB \
    --discard=old \
    --replicas=3 \
    --dupe-window=30s \
    --no-allow-rollup \
    --deny-delete \
    --deny-purge || echo "Stream may already exist"

# Create consumer for bid agents
nats --server="$NATS_URL" consumer add campaign-bids bid-agents \
    --filter="campaign-bids.bid-requests" \
    --deliver=all \
    --ack=explicit \
    --max-deliver=3 \
    --max-pending=1000 \
    --wait=5s \
    --replay=instant \
    --pull || echo "Consumer may already exist"

echo "NATS JetStream setup complete"
nats --server="$NATS_URL" stream info campaign-bids
