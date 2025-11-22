#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PROXY_URL="http://localhost:8080"
METRICS_URL="http://localhost:8080/metrics"

echo "=================================================="
echo "Pingora Reverse Proxy Test Suite"
echo "=================================================="
echo ""

echo "[1] Testing basic proxy functionality..."
echo "-------------------------------------------"
response=$(curl -s -w "\n%{http_code}" $PROXY_URL/test)
http_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | head -n -1)

if [ "$http_code" -eq 200 ]; then
    echo -e "${GREEN}✓ Basic request successful (HTTP $http_code)${NC}"
    echo "Response: $body" | head -c 200
    echo ""
else
    echo -e "${RED}✗ Basic request failed (HTTP $http_code)${NC}"
fi

echo ""
echo "[2] Testing round-robin load balancing..."
echo "-------------------------------------------"
echo "Making 6 requests to observe backend distribution:"
for i in {1..6}; do
    response=$(curl -s $PROXY_URL/balance-test)
    backend=$(echo "$response" | grep -oP '"backend":\s*"\K[^"]+' || echo "$response" | sed -n 's/.*"backend":\s*"\([^"]*\)".*/\1/p')
    if [ -z "$backend" ]; then
        backend="unknown"
    fi
    echo "  Request $i: Backend = $backend"
done

echo ""
echo "[3] Testing X-Backend header injection..."
echo "-------------------------------------------"
headers=$(curl -s -D - $PROXY_URL/header-test -o /dev/null)
if echo "$headers" | grep -q "X-Backend:"; then
    backend_header=$(echo "$headers" | grep "X-Backend:" | tr -d '\r\n')
    echo -e "${GREEN}✓ X-Backend header present: $backend_header${NC}"
else
    echo -e "${RED}✗ X-Backend header not found${NC}"
fi

echo ""
echo "[4] Testing Prometheus metrics endpoint..."
echo "-------------------------------------------"
metrics=$(curl -s $METRICS_URL)
if echo "$metrics" | grep -q "http_requests_duration_seconds"; then
    echo -e "${GREEN}✓ Metrics endpoint accessible${NC}"
    echo "Sample metrics:"
    echo "$metrics" | grep "http_requests_duration_seconds" | head -3
else
    echo -e "${RED}✗ Metrics endpoint not working${NC}"
fi

echo ""
echo "[5] Testing rate limiting (10 req/60s)..."
echo "-------------------------------------------"
echo "Sending 15 rapid requests to trigger rate limit..."

rate_limited=false
success_count=0
for i in {1..15}; do
    http_code=$(curl -s -o /dev/null -w "%{http_code}" $PROXY_URL/rate-limit-test)
    if [ "$http_code" -eq 429 ]; then
        echo -e "${GREEN}✓ Rate limit triggered at request $i (HTTP 429)${NC}"
        rate_limited=true
        break
    elif [ "$http_code" -eq 200 ]; then
        success_count=$((success_count + 1))
    fi
    # Small delay to ensure requests are counted
    sleep 0.01
done

if [ "$rate_limited" = false ]; then
    echo -e "${RED}✗ Rate limit not triggered after 15 requests (got $success_count successful)${NC}"
    echo -e "${YELLOW}⚠ Check config: rate limit should be 10 req/60s${NC}"
fi

echo ""
echo "[6] Testing POST requests..."
echo "-------------------------------------------"
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"test":"data","value":123}' \
    $PROXY_URL/api/submit)
if [ -n "$response" ]; then
    echo -e "${GREEN}✓ POST request successful${NC}"
    echo "POST response received:"
    echo "$response" | head -c 200
else
    echo -e "${RED}✗ POST request failed${NC}"
fi

echo ""
echo ""
echo "[7] Testing failover (requires stopping a backend)..."
echo "-------------------------------------------"
echo "To test failover manually:"
echo "  1. Stop one backend server (e.g., pkill -f backend1.py)"
echo "  2. Run: curl -v $PROXY_URL/failover-test"
echo "  3. Proxy should failover to the remaining backend"
echo "  4. Check logs for retry attempts"

echo ""
echo "=================================================="
echo "Test Suite Complete"
echo "=================================================="
echo ""
echo "Additional manual tests:"
echo "  - Load test: ab -n 1000 -c 10 $PROXY_URL/"
echo "  - View metrics: curl $METRICS_URL"
echo "  - Check logs: RUST_LOG=debug ./target/release/pace"
echo ""
