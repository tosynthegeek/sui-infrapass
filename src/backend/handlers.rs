use std::sync::Arc;

use crate::{
    sidecar::validator::{ValidateRequest, ValidateResponse},
    db::repository::Repository,
    utils::error::InfrapassError,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{info, warn};

#[derive(Debug, serde::Deserialize)]
pub struct RecordUsageRequest {
    pub user_address: String,
    pub entitlement_id: String,
    pub cost: u64,
}

pub async fn validate_entitlements_handler(
    State(repo): State<Arc<Repository>>,
    Json(payload): Json<ValidateRequest>,
) -> Result<impl IntoResponse, InfrapassError> {
    let result = repo
        .get_valid_entitlement_response(
            &payload.user_address,
            &payload.service_id,
            payload.request_cost,
        )
        .await?;

    info!(
        user = %payload.user_address,
        service = %payload.service_id,
        cost = payload.request_cost,
        "Entitlement validation request"
    );

    match result {
        Some(entitlement) => Ok((StatusCode::OK, Json(entitlement))),
        None => Ok((
            StatusCode::FORBIDDEN,
            Json(ValidateResponse {
                entitlement_id: String::new(),
                tier: String::new(),
                quota: None,
                units: None,
                tier_type: 0,
                expires_at: None,
                notify_provider: None,
            }),
        )),
    }
}

pub async fn record_usage_handler(
    State(repo): State<Arc<Repository>>,
    Json(payload): Json<RecordUsageRequest>,
) -> Result<impl IntoResponse, InfrapassError> {
    let timer = std::time::Instant::now();
    info!(
        user = %payload.user_address,
        entitlement_id = %payload.entitlement_id,
        cost = payload.cost,
        "Recording usage"
    );

    if payload.cost == 0 {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "cost must be > 0"})),
        ));
    }

    match repo
        .commit_usage(&payload.entitlement_id, &payload.user_address, payload.cost)
        .await
    {
        Ok(()) => {
            let duration = timer.elapsed().as_secs_f64();

            info!(
                user = %payload.user_address,
                entitlement_id = %payload.entitlement_id,
                cost = payload.cost,
                duration_ms = duration * 1000.0,
                "Usage recorded successfully"
            );

            Ok((
                StatusCode::OK,
                Json(serde_json::json!({"status": "usage recorded"})),
            ))
        }

        Err(e) => {
            warn!(
                error = %e,
                user = %payload.user_address,
                entitlement_id = %payload.entitlement_id,
                "Failed to record usage"
            );

            let status = match &e {
                InfrapassError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::BAD_REQUEST,
            };

            Ok((status, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}
