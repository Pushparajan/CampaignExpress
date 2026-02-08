#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Load Test Script
# Sends synthetic OpenRTB bid requests to the API server.
# Requires: curl, jq
# Usage: ./scripts/load-test.sh [HOST] [CONCURRENCY] [TOTAL_REQUESTS]
# =============================================================================

set -euo pipefail

HOST="${1:-http://localhost:8080}"
CONCURRENCY="${2:-50}"
TOTAL="${3:-10000}"

echo "=== Campaign Express Load Test ==="
echo "Target:      $HOST"
echo "Concurrency: $CONCURRENCY"
echo "Total:       $TOTAL requests"
echo ""

BID_REQUEST='{
  "id": "req-XXXXX",
  "imp": [{
    "id": "imp-1",
    "banner": {"w": 300, "h": 250},
    "bidfloor": 0.5
  }],
  "site": {
    "id": "site-1",
    "domain": "example.com",
    "page": "https://example.com/article"
  },
  "device": {
    "ua": "Mozilla/5.0",
    "ip": "203.0.113.1",
    "devicetype": 2,
    "os": "iOS"
  },
  "user": {
    "id": "user-XXXXX"
  },
  "tmax": 100,
  "at": 1,
  "cur": ["USD"]
}'

send_request() {
    local i=$1
    local req_id="req-$(printf '%05d' "$i")"
    local user_id="user-$(printf '%05d' $((i % 1000)))"

    local payload
    payload=$(echo "$BID_REQUEST" | sed "s/req-XXXXX/$req_id/g" | sed "s/user-XXXXX/$user_id/g")

    local start
    start=$(date +%s%N)

    local http_code
    http_code=$(curl -s -o /dev/null -w "%{http_code}" \
        -X POST "$HOST/v1/bid" \
        -H "Content-Type: application/json" \
        -d "$payload" \
        --max-time 5)

    local end
    end=$(date +%s%N)
    local latency_ms=$(( (end - start) / 1000000 ))

    echo "$i,$http_code,$latency_ms"
}

export -f send_request
export HOST BID_REQUEST

echo "request_num,status_code,latency_ms" > /tmp/campaign-express-load-test.csv

seq 1 "$TOTAL" | xargs -P "$CONCURRENCY" -I {} bash -c 'send_request {}' >> /tmp/campaign-express-load-test.csv

echo ""
echo "=== Results ==="
echo "Output: /tmp/campaign-express-load-test.csv"

# Quick summary
total_lines=$(wc -l < /tmp/campaign-express-load-test.csv)
success=$(grep -c ",200," /tmp/campaign-express-load-test.csv || true)
errors=$((total_lines - success - 1))

echo "Total:   $((total_lines - 1))"
echo "Success: $success"
echo "Errors:  $errors"

if command -v awk &> /dev/null; then
    avg_latency=$(awk -F',' 'NR>1 {sum+=$3; count++} END {print sum/count}' /tmp/campaign-express-load-test.csv)
    echo "Avg latency: ${avg_latency}ms"
fi
