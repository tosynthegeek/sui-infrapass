use anyhow::Result;
use serde::Deserialize;

use crate::sidecar::{error::ProxyError, middleware::AuthMode};

#[derive(Debug, Clone, Deserialize)]
pub struct SidecarConfig {
    /// Port the sidecar listens on (default 8080)
    #[serde(default = "default_port")]
    pub port: u16,

    pub redis_url: String,

    /// Your provider's actual service URL — sidecar forwards here after validation
    pub upstream_url: String,

    /// Your Sui protocol's validation API
    pub validator_api_url: String,

    /// Shared secret so your validator API knows this is a legit sidecar
    pub validator_api_key: String,

    /// The provider ID this sidecar is protecting (registered in your protocol)
    pub provider_id: String,

    #[serde(default)]
    pub auth_mode: AuthMode,

    /// Expected value for ApiKey or BearerToken modes
    pub auth_secret: Option<String>,

    /// How long to cache a VALID entitlement locally (milliseconds)
    /// Trades off real-time accuracy vs latency. 10-30s is a good default.
    #[serde(default = "default_cache_ttl_ms")]
    pub cache_ttl_ms: u64,

    /// Max cache entries (one per unique user address)
    #[serde(default = "default_cache_max_entries")]
    pub cache_max_entries: u64,

    /// Per-request timeout in ms before sidecar returns 504
    #[serde(default = "default_timeout_ms")]
    pub request_timeout_ms: u64,

    /// Header name where clients send their Sui wallet address
    /// e.g. "X-Sui-Address"
    #[serde(default = "default_address_header")]
    pub address_header: String,

    /// Header name where clients send the service ID they're accessing
    /// e.g. "X-Service-Id"
    #[serde(default = "default_service_header")]
    pub service_header: String,

    /// Header name for passing the cost of the request (optional; depends on your pricing model)
    /// e.g. "X-Request-Cost"
    /// If not provided, sidecar assumes a default cost of 1 for all requests
    #[serde(default = "default_cost_header")]
    pub cost_header: String,

    /// If true, on validator API failure → ALLOW request (fail open)
    /// If false, on failure → REJECT request (fail closed)  
    /// Fail closed is safer; fail open is better for availability
    #[serde(default)]
    pub fail_open: bool,

    /// Webhook URL to notify your provider when quota events occur
    pub provider_webhook_url: Option<String>,

    /// HMAC secret for signing webhook payloads
    pub provider_webhook_secret: Option<String>,
}

impl SidecarConfig {
    pub fn load() -> Result<Self, ProxyError> {
        dotenvy::dotenv().ok();

        let cfg: SidecarConfig = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?
            .try_deserialize()?;

        match cfg.auth_mode {
            AuthMode::None => {}
            AuthMode::ApiKey | AuthMode::BearerToken => {
                if cfg.auth_secret.as_deref().unwrap_or("").is_empty() {
                    return Err(ProxyError::ConfigError(
                        "auth_secret must be set when auth_mode is api_key or bearer_token"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(cfg)
    }

    pub fn validate(&self) -> Result<(), ProxyError> {
        Ok(())
    }
}

fn default_port() -> u16 {
    8080
}
fn default_cache_ttl_ms() -> u64 {
    15_000
}
fn default_cache_max_entries() -> u64 {
    10_000
}
fn default_timeout_ms() -> u64 {
    5_000
}
fn default_address_header() -> String {
    "X-Infrapass-Address".to_string()
}

fn default_cost_header() -> String {
    "X-Infrapass-Cost".to_string()
}

fn default_service_header() -> String {
    "X-Infrapass-Service-Id".to_string()
}
