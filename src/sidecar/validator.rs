use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, warn};

use crate::sidecar::cache::CachedEntitlement;

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub user_address: String,
    pub service_id: String,
    pub request_cost: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateResponse {
    pub entitlement_id: String,
    pub tier: String,
    pub quota: Option<u64>,
    pub units: Option<u64>,
    pub tier_type: u8,
    pub expires_at: Option<DateTime<Utc>>,
    pub notify_provider: Option<ProviderNotification>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProviderNotification {
    pub event: String,
    pub user_address: String,
    pub service_id: String,
    pub detail: serde_json::Value,
}

pub struct ValidatorClient {
    client: Client,
    api_url: String,
    api_key: String,
}

impl ValidatorClient {
    pub fn new(api_url: String, api_key: String) -> Self {
        let client = Client::builder()
            // Connection pool: keeps TCP connections alive to your validator API
            // This alone saves ~3-5ms per request (no TCP handshake overhead)
            .pool_max_idle_per_host(50)
            .pool_idle_timeout(Duration::from_secs(90))
            // Per-call timeout (separate from the sidecar's overall request timeout)
            .timeout(Duration::from_millis(500))
            // Use rustls (pure Rust TLS) — no OpenSSL dependency
            .use_rustls_tls()
            .build()
            .expect("Failed to build validator HTTP client");

        Self {
            client,
            api_url,
            api_key,
        }
    }

    pub async fn validate(
        &self,
        user_address: &str,
        service_id: &str,
        cost: u64,
    ) -> Result<ValidateResponse, ValidatorError> {
        let url = format!("{}/validate", self.api_url);

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&ValidateRequest {
                user_address: user_address.to_string(),
                service_id: service_id.to_string(),
                request_cost: cost,
            })
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Validator API unreachable");
                ValidatorError::Unreachable(e.to_string())
            })?;

        if !resp.status().is_success() {
            warn!(status = %resp.status(), "Validator API returned non-2xx");
            return Err(ValidatorError::ApiError(resp.status().as_u16()));
        }

        resp.json::<ValidateResponse>().await.map_err(|e| {
            error!(error = %e, "Failed to parse validator response");
            ValidatorError::ParseError(e.to_string())
        })
    }

    pub async fn record_usage(
        &self,
        user_address: &str,
        entitlement_id: &str,
        cost: u64,
    ) -> Result<(), ValidatorError> {
        let url = format!("{}/record_usage", self.api_url);

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "user_address": user_address,
                "entitlement_id": entitlement_id,
                "cost": cost,
            }))
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Validator API unreachable");
                ValidatorError::Unreachable(e.to_string())
            })?;

        if !resp.status().is_success() {
            warn!(status = %resp.status(), "Validator API returned non-2xx on record_usage");
            return Err(ValidatorError::ApiError(resp.status().as_u16()));
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidatorError {
    #[error("Validator API unreachable: {0}")]
    Unreachable(String),
    #[error("Validator API error: HTTP {0}")]
    ApiError(u16),
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl ValidatorError {
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            ValidatorError::Unreachable(_) | ValidatorError::ApiError(500..=599)
        )
    }
}

/// Convert a validator response into something cacheable
pub fn to_cached(resp: &ValidateResponse) -> CachedEntitlement {
    CachedEntitlement {
        id: resp.entitlement_id.clone(),
        tier: resp.tier.clone(),
        quota: resp.quota,
        units: resp.units,
        tier_type: resp.tier_type,
        expires_at: resp.expires_at,
        cached_at: None,
    }
}
