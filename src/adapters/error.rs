use axum::Json;
use axum::http::StatusCode;
use axum::http::status::InvalidStatusCode;
use axum::response::{IntoResponse, Response};
use hmac::digest::InvalidLength;
use redis::RedisError;

use crate::utils::error::InfrapassError;

#[derive(Debug)]
pub enum ProxyError {
    InvalidRequest(String),
    InternalError(String),
    NotFound(String),
    Unauthorized(String),
    BadGateway(String),
    ServiceUnavailable(String),
    RedisConnectionError(RedisError),
    ReqwestError(reqwest::Error),
    SerdeError(serde_json::Error),
    AxumError(axum::Error),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::InvalidRequest(msg) => write!(f, "Invalid Request: {}", msg),
            ProxyError::InternalError(msg) => write!(f, "Internal Server Error: {}", msg),
            ProxyError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ProxyError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ProxyError::BadGateway(msg) => write!(f, "Bad Gateway: {}", msg),
            ProxyError::ServiceUnavailable(msg) => write!(f, "Service Unavailable: {}", msg),
            ProxyError::RedisConnectionError(err) => write!(f, "Redis Connection Error: {}", err),
            ProxyError::ReqwestError(err) => write!(f, "Reqwest Error: {}", err),
            ProxyError::SerdeError(err) => write!(f, "Serde Error: {}", err),
            ProxyError::AxumError(err) => write!(f, "Axum Error: {}", err),
        }
    }
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ProxyError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ProxyError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            ProxyError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ProxyError::BadGateway(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            ProxyError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            ProxyError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ProxyError::RedisConnectionError(e) => (StatusCode::SERVICE_UNAVAILABLE, e.to_string()),
            ProxyError::ReqwestError(e) => (StatusCode::BAD_GATEWAY, e.to_string()),
            ProxyError::SerdeError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ProxyError::AxumError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl std::error::Error for ProxyError {}

impl From<RedisError> for ProxyError {
    fn from(err: RedisError) -> Self {
        ProxyError::RedisConnectionError(err)
    }
}

impl From<reqwest::Error> for ProxyError {
    fn from(err: reqwest::Error) -> Self {
        ProxyError::ReqwestError(err)
    }
}

impl From<serde_json::Error> for ProxyError {
    fn from(err: serde_json::Error) -> Self {
        ProxyError::SerdeError(err)
    }
}

impl From<axum::Error> for ProxyError {
    fn from(err: axum::Error) -> Self {
        ProxyError::AxumError(err)
    }
}

impl From<axum::http::Error> for ProxyError {
    fn from(err: axum::http::Error) -> Self {
        ProxyError::InternalError(format!("HTTP error: {}", err))
    }
}

impl From<InvalidStatusCode> for ProxyError {
    fn from(err: InvalidStatusCode) -> Self {
        ProxyError::InternalError(format!("Invalid status code: {}", err))
    }
}

impl From<std::io::Error> for ProxyError {
    fn from(err: std::io::Error) -> Self {
        ProxyError::InternalError(format!("IO error: {}", err))
    }
}

impl From<InvalidLength> for ProxyError {
    fn from(err: InvalidLength) -> Self {
        ProxyError::InternalError(format!("HMAC error : {}", err))
    }
}

impl From<InfrapassError> for ProxyError {
    fn from(err: InfrapassError) -> Self {
        ProxyError::InternalError(format!("Infrapass error: {}", err))
    }
}
