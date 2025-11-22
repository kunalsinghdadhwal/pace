# Pingora Reverse Proxy Project

## Project Overview
This project builds a configurable HTTP reverse proxy (API gateway) using Cloudflare's Pingora framework in Rust. It demonstrates mastery of Pingora's core concepts by integrating configuration, load balancing, rate limiting, request/response modification, error handling, metrics, and graceful operations. The proxy balances traffic across upstream backends, limits requests per IP, adds headers, fails over on errors, logs issues, exposes Prometheus metrics, and supports zero-downtime shutdowns.

**Goal**: Create a runnable binary testable with `curl` and `ab`. Commit to GitHub with README.md (setup, run, test commands).


**Success Criteria**:
- Loads TOML config dynamically.
- Round-robin balances to 2+ backends with pooling.
- Rate limits at 100 req/60s per IP (429 on exceed).
- Adds proxy/backend headers.
- Failover on upstream failure (e.g., kill a backend).
- Metrics at `/metrics` show latency histograms.
- Graceful SIGTERM handles in-flight requests.
- Logs errors to stdout/file.

## Key Constraints
- Use Rust 1.75+; no blocking code in async phases (use `tokio::fs` if needed).
- Share state via `CTX` (e.g., backend index, failures)—no globals.
- Handle async peers correctly; connect with `connect_async`.
- No external installs in runtime; deps via Cargo.toml only.
- Debug with `RUST_LOG=debug cargo run`.

## File Structure
- `Cargo.toml`: Exact deps as in prompt.
- `src/config.rs`: TOML structs and `load_config` fn.
- `src/main.rs`: Server bootstrap, `ProxyListener` impl (with `ProxyHttp`), phases (`request_filter`, `response_filter`, `upstream_peer`, `on_connect_error`).
- `config/proxy.toml`: Sample config.
- `README.md`: Setup (prereqs, build/run), tests (curl/ab examples), systemd unit.
- Optional: `tests/integration.rs` for basic proxy checks.

### Core Pingora Docs
- Quick Start: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/quick_start.md
- User Guide Index: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/index.md
- Start/Stop: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/start_stop.md
- Graceful Restart/Shutdown: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/graceful.md
- Configuration: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/conf.md
- Daemonization: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/daemon.md
- Systemd Integration: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/systemd.md
- Panic Handling: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/panic.md
- Error Logging: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/error_log.md
- Prometheus Metrics: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/prom.md

### Building Proxies
- Request Lifecycle/Phases: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/phase.md
- Upstream Peers: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/peer.md
- CTX State Sharing: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/ctx.md
- Returning Errors: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/errors.md
- Modify Requests: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/modify_filter.md
- Connection Pooling: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/pooling.md
- Failover/Handling Failures: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/failover.md
- Rate Limiter Quickstart: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/user_guide/rate_limiter.md

### Advanced (Optional)
- Internals: https://raw.githubusercontent.com/cloudflare/pingora/refs/heads/main/docs/advanced/internals.md

### External References (Unchanged)
- Pingora Crate Docs: https://docs.rs/pingora/latest/pingora/
- Pingora Proxy Module: https://docs.rs/pingora-proxy/latest/pingora_proxy/
- Prometheus Rust Client: https://docs.rs/prometheus/latest/prometheus/
- Rust Async Book: https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html
- TOML + Serde: https://serde.rs/tomls.html
- Apache Bench (ab): https://httpd.apache.org/docs/2.4/programs/ab.html
- Pingora Releases: https://github.com/cloudflare/pingora/releases
