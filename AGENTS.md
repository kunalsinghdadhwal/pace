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

## Reference Links
### Core Pingora Docs (GitHub: https://github.com/cloudflare/pingora/tree/main/docs)
- Quick Start: https://github.com/cloudflare/pingora/blob/main/docs/quick_start.md
- User Guide Index: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/index.md
- Start/Stop: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/start_stop.md
- Graceful Restart/Shutdown: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/graceful.md
- Configuration: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/conf.md
- Daemonization: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/daemon.md
- Systemd Integration: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/systemd.md
- Panic Handling: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/panic.md
- Error Logging: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/error_log.md
- Prometheus Metrics: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/prom.md

### Building Proxies
- Request Lifecycle/Phases: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/phase.md
- Upstream Peers: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/peer.md
- CTX State Sharing: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/ctx.md
- Returning Errors: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/errors.md
- Modify Requests: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/modify_filter.md
- Connection Pooling: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/pooling.md
- Failover/Handling Failures: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/failover.md
- Rate Limiter Quickstart: https://github.com/cloudflare/pingora/blob/main/docs/user_guide/rate_limiter.md

### Advanced (Optional)
- Internals: https://github.com/cloudflare/pingora/blob/main/docs/advanced/internals.md
- BoringSSL Usage: (WIP in docs; see repo examples)
- Async Runtime/Threading: (WIP; use Tokio for extensions)

### External References
- Pingora Crate Docs: https://docs.rs/pingora/latest/pingora/
- Pingora Proxy Module: https://docs.rs/pingora-proxy/latest/pingora_proxy/
- Prometheus Rust Client: https://docs.rs/prometheus/latest/prometheus/
- Rust Async Best Practices: https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html
- TOML Parsing with Serde: https://serde.rs/tomls.html
- Testing Tools: Apache Bench (`ab`): https://httpd.apache.org/docs/2.4/programs/ab.html

## Troubleshooting
- API Changes: Check https://github.com/cloudflare/pingora/releases (post-0.6 updates).
- Build Errors: Ensure `rustup update`; add `lazy_static = "1.4"` if needed for metrics.
- Runtime Panics: Hook via `panic.md` example.
- Extensions: Add TLS (BoringSSL), health checks (background service), or tracing after core.

Implement step-by-step; validate each phase with tests. Share repo for feedback!
