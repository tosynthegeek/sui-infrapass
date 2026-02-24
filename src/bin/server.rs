use std::sync::Arc;

use anyhow::Result;
use dotenvy::dotenv;
use infrapass::{
    db::{create_pool, repository::Repository, run_migrations},
    events::{listener::EventListener, types::EventPayload, worker::EventWorker},
};
use tokio::{signal, sync::mpsc};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

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

    info!("Starting Infrapass Event Listener");

    let grpc_url = std::env::var("GRPC_URL").expect("GRPC_URL environment variable must be set");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = create_pool(&database_url).await?;

    run_migrations(&pool).await?;

    let (tx, rx) = mpsc::channel::<EventPayload>(256);

    let listener = EventListener::new(&grpc_url, tx).await?;
    let pool = Arc::new(pool);
    let repo = Repository::new(pool);
    let worker = EventWorker::new(repo, rx);
    let listener_handle = tokio::spawn(async move {
        if let Err(e) = listener.run().await {
            tracing::error!("Event listener failed: {}", e);
        }
    });

    info!("Processing events...");
    let worker_handle = tokio::spawn(async move {
        if let Err(e) = worker.run().await {
            tracing::error!("Event worker failed: {}", e);
        }
    });

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        result = listener_handle => {
            match result {
                Ok(_) => info!("Listener task completed"),
                Err(e) => tracing::error!("Listener task panicked: {}", e),
            }
        }
        result = worker_handle => {
            match result {
                Ok(_) => info!("Worker task completed"),
                Err(e) => tracing::error!("Worker task panicked: {}", e),
            }
        }
    }

    info!("Shutting down gracefully...");

    Ok(())
}
