use std::sync::OnceLock;

use axum::{
    extract::{Json, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn api_key_auth(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    static API_KEY: OnceLock<String> = OnceLock::new();
    let expected = API_KEY.get_or_init(|| std::env::var("API_KEY").expect("API_KEY must be set"));

    let provided = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match provided {
        Some(key) if key == expected => Ok(next.run(req).await),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "invalid or missing API key" })),
        )),
    }
}
