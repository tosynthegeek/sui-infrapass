use std::{sync::Arc, time::Duration};

use anyhow::Result;
use dotenvy::dotenv;
use infrapass::{
    backend::{router::build_router, settlement::settlement_worker},
    db::{create_pool, repository::Repository, run_migrations},
    events::{listener::EventListener, types::EventPayload, worker::EventWorker},
};
use sui_sdk::SuiClientBuilder;
use tokio::{signal, sync::mpsc};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    init_tracing();

    info!("Starting Infrapass");

    let config = load_config();
    let pool = Arc::new(create_pool(&config.database_url).await?);
    run_migrations(&pool).await?;

    let repo = Arc::new(Repository::new(pool));
    let redis_client = redis::Client::open(config.redis_url)?;

    let sui_client = Arc::new(SuiClientBuilder::default().build(&config.grpc_url).await?);

    let app = build_router(repo.clone())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(10)));

    let tcp_listener = tokio::net::TcpListener::bind(&config.addr).await?;
    info!("Validator API listening on {}", config.addr);

    let (tx, rx) = mpsc::channel::<EventPayload>(256);

    let listener = EventListener::new(sui_client.clone(), &config.grpc_url, tx).await?;
    let worker = EventWorker::new(repo.clone(), rx, redis_client).await?;

    let server_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(tcp_listener, app).await {
            tracing::error!("HTTP server error: {}", e);
        }
    });

    let listener_handle = tokio::spawn(async move {
        if let Err(e) = listener.run().await {
            tracing::error!("Event listener failed: {}", e);
        }
    });

    let worker_handle = tokio::spawn(async move {
        if let Err(e) = worker.run().await {
            tracing::error!("Event worker failed: {}", e);
        }
    });

    let settlement_repo = repo.clone();
    let settlement_client = sui_client.clone();
    let settlement_handle = tokio::spawn(async move {
        if let Err(e) = settlement_worker(
            settlement_repo,
            settlement_client,
            config.settlement_interval,
        )
        .await
        {
            error!("Settlement worker failed: {}", e);
        }
    });

    info!("All services running");

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        result = server_handle => {
            match result {
                Ok(_) => info!("HTTP server stopped"),
                Err(e) => tracing::error!("HTTP server panicked: {}", e),
            }
        }
        result = listener_handle => {
            match result {
                Ok(_) => info!("Event listener stopped"),
                Err(e) => tracing::error!("Event listener panicked: {}", e),
            }
        }
        result = worker_handle => {
            match result {
                Ok(_) => info!("Event worker stopped"),
                Err(e) => tracing::error!("Event worker panicked: {}", e),
            }
        }

        result = settlement_handle => tracing::error!("Settlement worker stopped: {:?}", result),
    }

    info!("Shutting down gracefully");
    Ok(())
}

struct IConfig {
    grpc_url: String,
    database_url: String,
    redis_url: String,
    addr: String,
    settlement_interval: u64,
}

fn load_config() -> IConfig {
    std::env::var("API_KEY").expect("API_KEY must be set");
    IConfig {
        grpc_url: std::env::var("GRPC_URL").expect("GRPC_URL must be set"),
        database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        redis_url: std::env::var("BACKEND_REDIS_URL").expect("BACKEND_REDIS_URL must be set"),
        addr: format!(
            "0.0.0.0:{}",
            std::env::var("API_PORT").unwrap_or_else(|_| "8088".to_string())
        ),
        settlement_interval: std::env::var("SETTLEMENT_INTERVAL")
            .expect("SETTLEMENT_INTERVAL must be set")
            .parse::<u64>()
            .expect("SETTLEMENT_INTERVAL must be a valid number"),
    }
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true),
        )
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,infrapass=debug".into()),
        )
        .init();
}
