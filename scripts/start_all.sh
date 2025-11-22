#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=================================================="
echo "Starting Pingora Reverse Proxy Demo"
echo "=================================================="
echo ""

cleanup() {
    echo ""
    echo "Shutting down all services..."
    kill $(jobs -p) 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

cd "$PROJECT_ROOT"

if [ ! -f "target/release/pace" ]; then
    echo "Building proxy server..."
    cargo build --release
    echo ""
fi

echo "Starting Backend 1 on port 8000..."
python3 scripts/backend1.py &
BACKEND1_PID=$!
sleep 1

echo "Starting Backend 2 on port 8001..."
python3 scripts/backend2.py &
BACKEND2_PID=$!
sleep 1

echo "Starting Pingora Proxy on port 8080..."
echo ""
echo "=================================================="
echo "All services started successfully!"
echo "=================================================="
echo ""
echo "  Backend 1: http://127.0.0.1:8000 (PID: $BACKEND1_PID)"
echo "  Backend 2: http://127.0.0.1:8001 (PID: $BACKEND2_PID)"
echo "  Proxy:     http://127.0.0.1:8080"
echo "  Metrics:   http://127.0.0.1:8080/metrics"
echo ""
echo "=================================================="
echo ""
echo "Press Ctrl+C to stop all services"
echo ""
echo "Run tests in another terminal:"
echo "  bash scripts/test_proxy.sh"
echo ""
echo "=================================================="
echo ""

RUST_LOG=info ./target/release/pace

wait
