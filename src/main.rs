use async_trait::async_trait;
use log::{error, info, warn};
use once_cell::sync::Lazy;
use pingora::prelude::*;
use pingora::proxy::{ProxyHttp, Session};
use pingora::server::configuration::Opt;
use pingora::upstreams::peer::HttpPeer;
use prometheus::{Encoder, HistogramOpts, HistogramVec, Registry, TextEncoder};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

    fn check_rate_limit(&self, client