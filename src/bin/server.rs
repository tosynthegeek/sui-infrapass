use anyhow::Result;
use infrapass::events::{handlers::handle_event, listener::EventListener, types::ProtocolEvent};
use sui_sdk::SuiClientBuilder;
use tokio::sync::mpsc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
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

    let ws_url = std::env::var("SUI_RPC_URL")
        .unwrap_or_else(|_| "wss://fullnode.testnet.sui.io:443".to_string());
    let rpc_url = std::env::var("SUI_RPC_URL")
        .unwrap_or_else(|_| "https://fullnode.testnet.sui.io:443".to_string());

    info!("Connecting to Sui RPC: {}", rpc_url);

    let client = SuiClientBuilder::default()
        .ws_url(&ws_url)
        .build(&rpc_url)
        .await?;

    let (tx, mut rx) = mpsc::channel::<ProtocolEvent>(256);

    let listener = EventListener::new(client, tx)?;
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
