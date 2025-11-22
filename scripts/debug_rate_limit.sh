#!/bin/bash

echo "Stopping any running proxy..."
pkill -f "target/release/pace" 2>/dev/null
sleep 1

echo "Starting proxy with debug logging..."
RUST_LOG=info ./target/release/pace > /tmp/pace_debug.log 2>&1 &
PROXY_PID=$!

echo "Proxy started (PID: $PROXY_PID)"
sleep 2

echo ""
echo "Sending 15 requests to test rate limiting..."
echo "============================================="

for i in {1..15}; do
    code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/test 2>/dev/null)
    if [ "$code" == "429" ]; then
        echo "Request $i: HTTP $code ← RATE LIMITED!"
    else
        echo "Request $i: HTTP $code"
    fi
    sleep 0.05
done

echo ""
echo "Checking proxy logs for rate limit messages..."
echo "=============================================="
grep -i "rate limit" /tmp/pace_debug.log | tail -10

echo ""
echo "Killing proxy..."
kill $PROXY_PID 2>/dev/null
wait $PROXY_PID 2>/dev/null
