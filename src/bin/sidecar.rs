use axum::{Json, Router, extract::State, middleware, response::IntoResponse};
use infrapass::{
    pubsub::subscriber::PubSubSubscriber,
    sidecar::{
        config::SidecarConfig,
        metrics,
        middleware::auth_middleware,
        proxy::{self, ProxyState},
    },
    utils::logs_fmt::UptimeSeconds,
};
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cfg = SidecarConfig::load()?;
    cfg.validate()?;
    info!(upstream = %cfg.upstream_url, port = cfg.port, "Sidecar starting");

    let state = Arc::new(ProxyState::new(cfg.clone()).await?);
    let pubsub_state = state.clone();

    let app = Router::new()
        .route("/metrics", axum::routing::get(metrics::metrics_handler))
        .route("/healthz", axum::routing::get(health_handler))
        .fallback(proxy::proxy_handler)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
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

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("infrapass_sidecar=info,infrapass=info,tower_http=warn")
    });

    let is_json = std::env::var("LOG_FORMAT").unwrap_or_default() == "json";

    let fmt_layer = if is_json {
        fmt::layer()
            .json()
            .with_current_span(false)
            .with_span_list(false)
            .with_ansi(true)
            .with_span_events(FmtSpan::NONE)
            .event_format(
                fmt::format()
                    .compact()
                    .with_level(true)
                    .with_timer(UptimeSeconds),
            )
            .boxed()
    } else {
        fmt::layer()
            .compact()
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .with_thread_names(false)
            .with_ansi(true)
            .with_span_events(FmtSpan::NONE)
            .event_format(
                fmt::format()
                    .compact()
                    .with_level(true)
                    .with_timer(UptimeSeconds),
            )
            .boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
