#!/bin/bash

set -e

echo "=================================================="
echo "Pingora Reverse Proxy Test Suite"
echo "=================================================="
echo ""

PROXY_URL="http://localhost:8080"
METRICS_URL="http://localhost:8080/metrics"

echo "[1] Testing basic proxy functionality..."
echo "-------------------------------------------"
response=$(curl -s -w "\n%{http_code}" $PROXY_URL/test)
http_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | head -n -1)

if [ "$http_code" -eq 200 ]; then
    echo "✓ Basic request successful (HTTP $http_code)"
    echo "Response: $body" | head -c 200
    echo ""
else
    echo "✗ Basic request failed (HTTP $http_code)"
fi

echo ""
echo "[2] Testing round-robin load balancing..."
echo "-------------------------------------------"
echo "Making 6 requests to observe backend distribution:"
for i in {1..6}; do
    response=$(curl -s $PROXY_URL/balance-test)
    backend=$(echo "$response" | grep -o '"backend":"[^"]*"' | cut -d'"' -f4)
    echo "  Request $i: Backend = $backend"
done

echo ""
echo "[3] Testing X-Proxy header injection..."
echo "-------------------------------------------"
headers=$(curl -s -D - $PROXY_URL/header-test -o /dev/null)
if echo "$headers" | grep -q "X-Backend:"; then
    backend_header=$(echo "$headers" | grep "X-Backend:" | tr -d '\r\n')
    echo "✓ X-Backend header present: $backend_header"
else
    echo "✗ X-Backend header not found"
fi

echo ""
echo "[4] Testing Prometheus metrics endpoint..."
echo "-------------------------------------------"
metrics=$(curl -s $METRICS_URL)
if echo "$metrics" | grep -q "http_requests_duration_seconds"; then
    echo "✓ Metrics endpoint accessible"
    echo "Sample metrics:"
    echo "$metrics" | grep "http_requests_duration_seconds" | head -3
else
    echo "✗ Metrics endpoint not working"
fi

echo ""
echo "[5] Testing rate limiting (100 req/60s)..."
echo "-------------------------------------------"
echo "Sending 105 rapid requests to trigger rate limit..."

rate_limited=false
for i in {1..105}; do
    http_code=$(curl -s -o /dev/null -w "%{http_code}" $PROXY_URL/rate-limit-test)
    if [ "$http_code" -eq 429 ]; then
        echo "✓ Rate limit triggered at request $i (HTTP 429)"
        rate_limited=true
        break
    fi
done

if [ "$rate_limited" = false ]; then
    echo "⚠ Rate limit not triggered (might need more requests or check config)"
fi

echo ""
echo "[6] Testing POST requests..."
echo "-------------------------------------------"
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"test":"data","value":123}' \
    $PROXY_URL/api/submit)
echo "POST response received:"
echo "$response" | head -c 200

echo ""
echo ""
echo "[7] Testing failover (requires stopping a backend)..."
echo "-------------------------------------------"
echo "To test failover manually:"
echo "  1. Stop one backend server (e.g., kill backend1)"
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
