use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde::Deserialize;

use crate::sidecar::{
    error::ProxyError,
    proxy::{ProxyState, deny_response},
};

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    #[default]
    None, // only entitlement check
    ApiKey,      // require X-Api-Key header
    BearerToken, // require Authorization: Bearer <token>
}

pub async fn auth_middleware(
    State(state): State<Arc<ProxyState>>,
    req: Request,
    next: Next,
) -> Result<Response, ProxyError> {
    match state.cfg.auth_mode {
        AuthMode::None => Ok(next.run(req).await),

        AuthMode::ApiKey => {
            let expected = state
                .cfg
                .auth_secret
                .as_deref()
                .ok_or_else(|| ProxyError::ConfigError("auth_secret missing".into()))?;

            let provided = req
                .headers()
                .get("X-Api-Key")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if provided == expected {
                Ok(next.run(req).await)
            } else {
                Ok(deny_response(StatusCode::UNAUTHORIZED, "invalid_api_key")?)
            }
        }

        AuthMode::BearerToken => {
            let expected = state
                .cfg
                .auth_secret
                .as_deref()
                .ok_or_else(|| ProxyError::ConfigError("auth_secret missing".into()))?;

            let provided = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .unwrap_or("");

            if provided == expected {
                Ok(next.run(req).await)
            } else {
                Ok(deny_response(
                    StatusCode::UNAUTHORIZED,
                    "invalid_bearer_token",
                )?)
            }
        }
    }
}
