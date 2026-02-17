use anyhow::Result;
use clap::Parser;
use infrapass::cmd::{Cli, Commands};
use sui_sdk::SuiClientBuilder;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let rpc_url = cli
        .rpc_url
        .unwrap_or_else(|| "https://fullnode.testnet.sui.io:443".to_string());

    info!("Connecting to Sui RPC: {}", rpc_url);

    let client = SuiClientBuilder::default().build(&rpc_url).await?;

    match cli.command {
        Commands::Provider(cmd) => cmd.execute(&client).await?,
        Commands::Pricing(cmd) => cmd.execute(&client).await?,
        Commands::Payment(cmd) => cmd.execute(&client).await?,
        Commands::Query(cmd) => cmd.execute(&client).await?,
    }

    Ok(())
}
