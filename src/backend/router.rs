use std::sync::Arc;

use crate::{
    backend::{
        handlers::{record_usage_handler, validate_entitlements_handler},
        middleware::api_key_auth,
    },
    db::repository::Repository,
};
use axum::{
    Router,
    middleware::{self},
    routing,
};

pub fn build_router(repo: Arc<Repository>) -> Router {
    Router::new()
        .route("/validate", routing::post(validate_entitlements_handler))
        .route("/record_usage", routing::post(record_usage_handler))
        .route_layer(middleware::from_fn(api_key_auth))
        .with_state(repo)
}
