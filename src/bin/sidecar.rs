use axum::{Json, Router, extract::State, response::IntoResponse};
use infrapass::{
    adapters::{
        config::SidecarConfig,
        metrics,
        proxy::{self, ProxyState},
    },
    pubsub::subscriber::PubSubSubscriber,
};
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "infrapass_sidecar=info,tower_http=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json()) // JSON logs for prod
        .init();

    let cfg = SidecarConfig::load()?;
    info!(
        upstream = %cfg.upstream_url,
        validator = %cfg.validator_api_url,
        port = cfg.port,
        "Infrapass sidecar starting"
    );

    let state = Arc::new(ProxyState::new(cfg.clone()).await?);
    let pubsub_state = state.clone();

    let app = Router::new()
        .route("/metrics", axum::routing::get(metrics::metrics_handler))
        .route("/healthz", axum::routing::get(health_handler))
        .fallback(proxy::proxy_handler)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_millis(
            cfg.request_timeout_ms,
        )))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let subscriber = PubSubSubscriber::new(pubsub_state);

    tokio::spawn(async move {
        if let Err(e) = subscriber.run().await {
            tracing::error!(error = %e, "PubSub listener crashed");
        }
    });

    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_handler(State(state): State<Arc<ProxyState>>) -> impl IntoResponse {
    let redis_ok = state.redis.clone().ping::<String>().await.is_ok();
    let status = if redis_ok { "ok" } else { "degraded" };
    Json(serde_json::json!({
        "status": status,
        "redis": redis_ok,
        "service": "infrapass-sidecar"
    }))
}
