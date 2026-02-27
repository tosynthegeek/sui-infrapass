use once_cell::sync::Lazy;
use prometheus::{Counter, Histogram, HistogramOpts, Registry, TextEncoder};

pub struct SidecarMetrics {
    pub requests_allowed: Counter,
    pub requests_denied: Counter,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
    pub validator_errors: Counter,
    pub request_duration: Histogram,
    registry: Registry,
}

impl SidecarMetrics {
    fn new() -> Self {
        let registry = Registry::new();

        let requests_allowed = Counter::new(
            "infrapass_sidecar_requests_allowed_total",
            "Requests allowed through",
        )
        .unwrap();
        let requests_denied = Counter::new(
            "infrapass_sidecar_requests_denied_total",
            "Requests denied by entitlement check",
        )
        .unwrap();
        let cache_hits = Counter::new(
            "infrapass_sidecar_cache_hits_total",
            "Entitlement cache hits",
        )
        .unwrap();
        let cache_misses = Counter::new(
            "infrapass_sidecar_cache_misses_total",
            "Entitlement cache misses",
        )
        .unwrap();
        let validator_errors = Counter::new(
            "infrapass_sidecar_validator_errors_total",
            "Validator API errors",
        )
        .unwrap();
        let request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "infrapass_sidecar_request_duration_seconds",
                "End-to-end request duration",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
        )
        .unwrap();

        registry
            .register(Box::new(requests_allowed.clone()))
            .unwrap();
        registry
            .register(Box::new(requests_denied.clone()))
            .unwrap();
        registry.register(Box::new(cache_hits.clone())).unwrap();
        registry.register(Box::new(cache_misses.clone())).unwrap();
        registry
            .register(Box::new(validator_errors.clone()))
            .unwrap();
        registry
            .register(Box::new(request_duration.clone()))
            .unwrap();

        Self {
            requests_allowed,
            requests_denied,
            cache_hits,
            cache_misses,
            validator_errors,
            request_duration,
            registry,
        }
    }

    pub fn encode(&self) -> String {
        let encoder = TextEncoder::new();
        let families = self.registry.gather();
        encoder.encode_to_string(&families).unwrap_or_default()
    }
}

pub static METRICS: Lazy<SidecarMetrics> = Lazy::new(SidecarMetrics::new);

pub async fn metrics_handler() -> String {
    METRICS.encode()
}
