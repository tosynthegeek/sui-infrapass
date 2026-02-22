use anyhow::Result;
use dotenvy::dotenv;
use infrapass::{
    db::{create_pool, run_migrations},
    events::{handlers::handle_event, listener::EventListener, types::ProtocolEvent},
};
use tokio::sync::mpsc;
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

    let (tx, mut rx) = mpsc::channel::<ProtocolEvent>(256);

    let listener = EventListener::new(&grpc_url, tx).await?;
    tokio::spawn(async move {
        if let Err(e) = listener.run().await {
            tracing::error!("Event listener failed: {}", e);
        }
    });

    info!("Processing events...");
    while let Some(event) = rx.recv().await {
        handle_event(event).await;
    }

    Ok(())
}
