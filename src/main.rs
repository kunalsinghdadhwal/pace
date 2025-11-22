use async_trait::async_trait;
use log::{error, info, warn};
use once_cell::sync::Lazy;
use pingora_core::prelude::*;
use pingora_core::server::configuration::Opt;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_http::{RequestHeader, ResponseHeader};
use pingora_proxy::{FailToProxy, ProxyHttp, Session};
use prometheus::{Encoder, HistogramOpts, HistogramVec, Registry, TextEncoder};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

mod config;
use config::{load_config, Config};

static METRICS_REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

static REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "http_requests_duration_seconds",
        "HTTP request latency in seconds",
    )
    .buckets(vec![
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
    ]);

    let histogram = HistogramVec::new(opts, &["method", "status"]).unwrap();
    METRICS_REGISTRY
        .register(Box::new(histogram.clone()))
        .unwrap();
    histogram
});

#[derive(Clone)]
struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    max_requests: u64,
    window_seconds: u64,
}

impl RateLimiter {
    fn new(max_requests: u64, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_seconds,
        }
    }

    fn check_rate_limit(&self, client_ip: &str) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut requests = self.requests.write().unwrap();
        let entry = requests
            .entry(client_ip.to_string())
            .or_insert_with(Vec::new);

        entry.retain(|&timestamp| now - timestamp < self.window_seconds);

        if entry.len() >= self.max_requests as usize {
            false
        } else {
            entry.push(now);
            true
        }
    }
}

pub struct ProxyContext {
    backend_index: usize,
    failure_count: usize,
    selected_backend: Option<String>,
    start_time: Instant,
}

impl ProxyContext {
    fn new() -> Self {
        Self {
            backend_index: 0,
            failure_count: 0,
            selected_backend: None,
            start_time: Instant::now(),
        }
    }
}

pub struct ReverseProxy {
    backends: Vec<String>,
    round_robin_index: Arc<AtomicUsize>,
    rate_limiter: RateLimiter,
    config: Config,
}

impl ReverseProxy {
    fn new(config: Config) -> Self {
        let rate_limiter = RateLimiter::new(
            config.rate_limit.max_requests,
            config.rate_limit.window_seconds,
        );

        Self {
            backends: config.upstreams.backends.clone(),
            round_robin_index: Arc::new(AtomicUsize::new(0)),
            rate_limiter,
            config,
        }
    }

    fn get_next_backend(&self) -> String {
        let index = self.round_robin_index.fetch_add(1, Ordering::Relaxed);
        let backend_index = index % self.backends.len();
        self.backends[backend_index].clone()
    }

    fn get_backend_by_index(&self, index: usize) -> String {
        self.backends[index % self.backends.len()].clone()
    }
}

#[async_trait]
impl ProxyHttp for ReverseProxy {
    type CTX = ProxyContext;

    fn new_ctx(&self) -> Self::CTX {
        ProxyContext::new()
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        ctx.start_time = Instant::now();

        if session.req_header().uri.path() == self.config.metrics.endpoint
            && self.config.metrics.enabled
        {
            let encoder = TextEncoder::new();
            let metric_families = METRICS_REGISTRY.gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();

            let response_body = String::from_utf8(buffer).unwrap();

            let mut header = ResponseHeader::build(200, None).unwrap();
            header
                .insert_header("Content-Type", "text/plain; version=0.0.4")
                .unwrap();
            header
                .insert_header("Content-Length", &response_body.len().to_string())
                .unwrap();

            session
                .write_response_header(Box::new(header), false)
                .await?;
            session
                .write_response_body(Some(response_body.into()), true)
                .await?;

            return Ok(true);
        }

        let client_ip = session
            .client_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if !self.rate_limiter.check_rate_limit(&client_ip) {
            warn!("Rate limit exceeded for client: {}", client_ip);

            let body = "Too Many Requests";
            let mut header = ResponseHeader::build(429, None).unwrap();
            header.insert_header("Content-Type", "text/plain").unwrap();
            header
                .insert_header("Content-Length", &body.len().to_string())
                .unwrap();

            session
                .write_response_header(Box::new(header), false)
                .await?;
            session
                .write_response_body(Some(body.as_bytes().to_vec().into()), true)
                .await?;

            return Ok(true);
        }

        session
            .req_header_mut()
            .insert_header("X-Proxy", "Pingora-Response")
            .unwrap();

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let backend_url = if ctx.failure_count > 0 && ctx.failure_count < self.backends.len() {
            let retry_index = (ctx.backend_index + ctx.failure_count) % self.backends.len();
            self.get_backend_by_index(retry_index)
        } else {
            let backend = self.get_next_backend();
            ctx.backend_index = self
                .round_robin_index
                .load(Ordering::Relaxed)
                .wrapping_sub(1)
                % self.backends.len();
            backend
        };

        ctx.selected_backend = Some(backend_url.clone());

        let url = backend_url.trim_start_matches("http://");
        let peer = Box::new(HttpPeer::new(url, false, String::new()));

        info!("Selected upstream peer: {}", backend_url);
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request
            .insert_header("X-Forwarded-Proto", "http")
            .unwrap();
        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if let Some(backend) = &ctx.selected_backend {
            let backend_name = backend.replace("http://", "").replace(":", "_");
            upstream_response
                .insert_header("X-Backend", &format!("backend_{}", backend_name))
                .unwrap();
        }

        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora_core::Error>,
        ctx: &mut Self::CTX,
    ) {
        let duration = ctx.start_time.elapsed().as_secs_f64();

        let method = session.req_header().method.as_str();
        let status = session
            .response_written()
            .map(|resp| resp.status.as_u16().to_string())
            .unwrap_or_else(|| "0".to_string());

        REQUEST_DURATION
            .with_label_values(&[method, &status])
            .observe(duration);

        info!(
            "{} {} - Status: {} - Duration: {:.3}s - Backend: {:?}",
            method,
            session.req_header().uri.path(),
            status,
            duration,
            ctx.selected_backend
        );
    }

    fn fail_to_connect(
        &self,
        _session: &mut Session,
        _peer: &HttpPeer,
        ctx: &mut Self::CTX,
        e: Box<pingora_core::Error>,
    ) -> Box<pingora_core::Error> {
        ctx.failure_count += 1;
        error!(
            "Failed to connect to upstream (attempt {}): {}",
            ctx.failure_count, e
        );
        e
    }

    fn error_while_proxy(
        &self,
        _peer: &HttpPeer,
        _session: &mut Session,
        e: Box<pingora_core::Error>,
        _ctx: &mut Self::CTX,
        _client_reused: bool,
    ) -> Box<pingora_core::Error> {
        error!("Error while proxying: {}", e);
        e
    }

    async fn fail_to_proxy(
        &self,
        _session: &mut Session,
        e: &pingora_core::Error,
        ctx: &mut Self::CTX,
    ) -> FailToProxy
    where
        Self::CTX: Send + Sync,
    {
        error!("Failed to proxy request: {}", e);

        if ctx.failure_count > 0 && ctx.failure_count < self.backends.len() {
            warn!(
                "Retrying with different backend (failure count: {})",
                ctx.failure_count
            );
            return FailToProxy {
                error_code: 0,
                can_reuse_downstream: false,
            };
        }

        FailToProxy {
            error_code: 503,
            can_reuse_downstream: false,
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = load_config("config/proxy.toml").expect("Failed to load config");

    info!("Loaded configuration: {:?}", config);
    info!("Starting proxy on {}", config.server.listen);
    info!("Backends: {:?}", config.upstreams.backends);
    info!(
        "Rate limit: {} requests per {} seconds",
        config.rate_limit.max_requests, config.rate_limit.window_seconds
    );

    let mut server =
        pingora_core::server::Server::new(Some(Opt::default())).expect("Failed to create server");
    server.bootstrap();

    let proxy_service = ReverseProxy::new(config.clone());

    let mut proxy_service_http =
        pingora_proxy::http_proxy_service(&server.configuration, proxy_service);
    proxy_service_http.add_tcp(&config.server.listen);

    server.add_service(proxy_service_http);

    info!("Proxy server starting with graceful shutdown support...");
    server.run_forever();
}
