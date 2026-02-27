use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::Response,
};
use chrono::Utc;
use redis::{Client as RedisClient, aio::MultiplexedConnection};
use std::sync::Arc;
use tracing::{instrument, warn};

use crate::{
    sidecar::{
        cache::CachedEntitlement,
        config::SidecarConfig,
        error::ProxyError,
        metrics::METRICS,
        validator::{ProviderNotification, ValidatorClient, to_cached},
    },
    utils::constants::LUA_ATOMIC_CHECK_AND_DECREMENT,
};

use hmac::{Hmac, Mac};
use sha2::Sha256;
pub type HmacSha256 = Hmac<Sha256>;

pub struct ProxyState {
    pub cfg: SidecarConfig,
    pub validator: ValidatorClient,
    pub http_client: reqwest::Client,
    pub redis: MultiplexedConnection,
    pub redis_client: RedisClient,
}

impl ProxyState {
    pub async fn new(cfg: SidecarConfig) -> Result<Self, ProxyError> {
        let validator =
            ValidatorClient::new(cfg.validator_api_url.clone(), cfg.validator_api_key.clone());

        let http_client = reqwest::Client::builder()
            .pool_max_idle_per_host(100)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()?;

        let redis_client = RedisClient::open(cfg.redis_url.clone())?;
        let redis = redis_client.get_multiplexed_async_connection().await?;

        Ok(Self {
            cfg,
            validator,
            http_client,
            redis,
            redis_client,
        })
    }

    fn entitlement_key(&self, user: &str, service: &str) -> String {
        format!("entitlement:{}:{}", user, service)
    }

    fn quota_key(&self, user: &str, service: &str) -> String {
        format!("quota:{}:{}", user, service)
    }

    pub async fn get_entitlement(&self, user: &str, service: &str) -> Option<CachedEntitlement> {
        let mut conn = self.redis.clone();
        let json: Option<String> = redis::cmd("GET")
            .arg(&self.entitlement_key(user, service))
            .query_async(&mut conn)
            .await
            .ok()?;
        json.and_then(|j| serde_json::from_str(&j).ok())
    }

    pub async fn set_entitlement(
        &self,
        user: &str,
        service: &str,
        ent: &CachedEntitlement,
        ttl_secs: u64,
    ) -> Result<(), ProxyError> {
        let mut conn = self.redis.clone();
        let json = serde_json::to_string(&ent)?;
        let _: () = redis::pipe()
            .set(&self.entitlement_key(user, service), json)
            .expire(&self.entitlement_key(user, service), ttl_secs as i64)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn set_quota(
        &self,
        user: &str,
        service: &str,
        remaining: i64,
        ttl_secs: u64,
    ) -> Result<(), ProxyError> {
        let mut conn = self.redis.clone();
        let _: Option<()> = redis::cmd("SET")
            .arg(&self.quota_key(user, service))
            .arg(remaining)
            .arg("NX")
            .arg("EX")
            .arg(ttl_secs)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn invalidate_entitlement(
        &self,
        user: &str,
        service: &str,
    ) -> Result<(), ProxyError> {
        let mut conn = self.redis.clone();
        let _: () = redis::cmd("DEL")
            .arg(&self.entitlement_key(user, service))
            .query_async(&mut conn)
            .await?;

        Ok(())
    }
}

#[instrument(skip(state, req), fields(path = %req.uri().path()))]
pub async fn proxy_handler(
    State(state): State<Arc<ProxyState>>,
    req: Request,
) -> Result<Response, ProxyError> {
    let timer = std::time::Instant::now();

    let user_address = match req.headers().get(&state.cfg.address_header) {
        Some(val) => match val.to_str() {
            Ok(addr) => addr.to_string(),
            Err(_) => {
                return Ok(deny_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_address_header",
                )?);
            }
        },
        None => {
            METRICS.requests_denied.inc();
            return Ok(deny_response(
                StatusCode::UNAUTHORIZED,
                "missing_sui_address",
            )?);
        }
    };

    let cost = match req.headers().get(&state.cfg.cost_header) {
        Some(val) => match val.to_str() {
            Ok(cost_str) => match cost_str.parse::<u64>() {
                Ok(c) => c,
                Err(_) => {
                    return Ok(deny_response(
                        StatusCode::BAD_REQUEST,
                        "invalid_cost_header",
                    )?);
                }
            },
            Err(_) => {
                return Ok(deny_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_cost_header",
                )?);
            }
        },
        None => 1,
    };

    let service_id = match req.headers().get(&state.cfg.service_header) {
        Some(val) => match val.to_str() {
            Ok(sid) => sid.to_string(),
            Err(_) => {
                return Ok(deny_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_service_header",
                )?);
            }
        },
        None => {
            METRICS.requests_denied.inc();
            return Ok(deny_response(
                StatusCode::BAD_REQUEST,
                "missing_service_id",
            )?);
        }
    };

    let (has_entitlement, entitlement) =
        if let Some(cached) = state.get_entitlement(&user_address, &service_id).await {
            METRICS.cache_hits.inc();
            (cached.allowed(), cached)
        } else {
            METRICS.cache_misses.inc();
            let resp = match state
                .validator
                .validate(&user_address, &service_id, cost)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    METRICS.validator_errors.inc();
                    warn!(error = ?e, "Validator API error");
                    if state.cfg.fail_open {
                        warn!("Failing open due to validator error");
                        return Ok(deny_response(
                            StatusCode::OK,
                            "validator_error, failing_open",
                        )?);
                    } else {
                        warn!("Failing closed due to validator error");
                    }
                    return Ok(deny_response(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "validator_error",
                    )?);
                }
            };
            let resp_to_cache_type = to_cached(&resp);
            let allowed = resp_to_cache_type.allowed();
            let ttl_secs: u64 = match resp_to_cache_type.expires_at {
                Some(exp) => {
                    let now = Utc::now();
                    let remaining = (exp - now).num_seconds();
                    if remaining > 0 { remaining as u64 } else { 0 }
                }
                None => state.cfg.cache_ttl_ms / 1000,
            };
            let _ = state
                .set_entitlement(&user_address, &service_id, &resp_to_cache_type, ttl_secs)
                .await;

            if allowed {
                // In your cache miss path after set_entitlement
                match resp_to_cache_type.tier_type {
                    0 => {
                        // Subscription — no quota key needed, expiry is enforced by allowed()
                    }
                    2 => {
                        // Quota-within-window — seed from quota field
                        if let Some(quota) = resp_to_cache_type.quota {
                            let _ = state
                                .set_quota(&user_address, &service_id, quota as i64, ttl_secs)
                                .await;
                        }
                    }
                    3 => {
                        // Pay-per-request — seed from units field
                        if let Some(units) = resp_to_cache_type.units {
                            let _ = state
                                .set_quota(&user_address, &service_id, units as i64, ttl_secs)
                                .await;
                        }
                    }
                    _ => {
                        warn!(
                            tier_type = resp_to_cache_type.tier_type,
                            "Unknown tier type during quota seeding"
                        );
                    }
                }
            }

            (allowed, resp_to_cache_type)
        };

    if !has_entitlement {
        METRICS.requests_denied.inc();
        return Ok(deny_response(
            StatusCode::FORBIDDEN,
            "access_denied, no entitlement",
        )?);
    }

    let mut conn = state.redis.clone();

    if (entitlement.tier_type != 0)
        && (entitlement.quota().is_some() || entitlement.units().is_some())
    {
        let result: i64 = redis::Script::new(LUA_ATOMIC_CHECK_AND_DECREMENT)
            .key(&state.quota_key(&user_address, &service_id))
            .arg(cost as i64)
            .arg(entitlement.tier_type as i64)
            .invoke_async(&mut conn)
            .await?;

        match result {
            0 => {} // subscription — allowed, no counter
            -1 => {
                METRICS.requests_denied.inc();
                return Ok(deny_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    "quota_exceeded",
                )?);
            }
            -2 => {
                METRICS.requests_denied.inc();
                warn!(
                    user = %user_address,
                    tier_type = entitlement.tier_type,
                    "Quota key not initialized"
                );
                return Ok(deny_response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "quota_not_ready",
                )?);
            }
            -3 => {
                METRICS.requests_denied.inc();
                warn!(
                    user = %user_address,
                    tier_type = entitlement.tier_type,
                    "Unknown tier type in Lua script"
                );
                return Ok(deny_response(StatusCode::BAD_REQUEST, "unknown_tier_type")?);
            }
            n => {
                if n < 10 {
                    warn!(
                        user = %user_address,
                        service = %service_id,
                        remaining = n,
                        "Low quota"
                    );
                }
            }
        }
    }

    METRICS.requests_allowed.inc();

    let path_and_query = req
        .uri()
        .path_and_query()
        .ok_or_else(|| ProxyError::InvalidRequest("Missing path and query".into()))?
        .as_str();
    let upstream_url = format!("{}{}", state.cfg.upstream_url, path_and_query);

    let mut upstream_req = state
        .http_client
        .request(req.method().clone(), &upstream_url);

    for (name, value) in req.headers().iter() {
        upstream_req = upstream_req.header(name, value);
    }

    upstream_req = upstream_req.header("X-Infrapass-User-Address", &user_address);
    upstream_req = upstream_req.header("X-Infrapass-Validated", "true");

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX).await?;

    upstream_req = upstream_req.body(body_bytes);

    let upstream_resp = match upstream_req.send().await {
        Ok(r) => r,
        Err(e) => {
            warn!(error = %e, "Upstream request failed");
            return Ok(deny_response(StatusCode::BAD_GATEWAY, "upstream_error")?);
        }
    };

    let state_clone = state.clone();
    let addr = user_address.clone();
    let ent = entitlement.id.clone();
    tokio::spawn(async move {
        let _ = state_clone.validator.record_usage(&addr, &ent, cost).await;
    });

    METRICS
        .request_duration
        .observe(timer.elapsed().as_secs_f64());

    let status = StatusCode::from_u16(upstream_resp.status().as_u16())?;
    let headers = upstream_resp.headers().clone();
    let body = upstream_resp.bytes().await?;

    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    for (name, value) in headers.iter() {
        response.headers_mut().insert(name, value.clone());
    }

    Ok(response)
}

pub fn deny_response(status: StatusCode, reason: &str) -> Result<Response, ProxyError> {
    let body = serde_json::json!({
        "error": reason,
        "status": status.as_u16(),
    });
    Ok(Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))?)
}

pub async fn deliver_notification(
    state: &ProxyState,
    notification: ProviderNotification,
) -> Result<(), ProxyError> {
    let (webhook_url, secret) = match (
        &state.cfg.provider_webhook_url,
        &state.cfg.provider_webhook_secret,
    ) {
        (Some(url), Some(secret)) => (url.clone(), secret.clone()),
        _ => {
            // TODO: consider metrics for missed notifications due to misconfiguration
            warn!("Provider webhook URL or secret not configured; skipping notification");
            return Ok(());
        }
    };

    let payload = match serde_json::to_vec(&notification) {
        Ok(p) => p,
        Err(_) => {
            warn!(notification = ?notification, "Failed to serialize notification payload");
            return Ok(());
        }
    };

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
    mac.update(&payload);
    let sig = hex::encode(mac.finalize().into_bytes());

    let _ = state
        .http_client
        .post(&webhook_url)
        .header("Content-Type", "application/json")
        .header("X-Infrapass-Signature", sig)
        .body(payload)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await;

    Ok(())
}
