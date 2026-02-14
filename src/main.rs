use sui_sdk::SuiClientBuilder;

use crate::{
    transactions::{registry::provider_create_service, tx_builder::sign_and_execute_tx},
    utils::config::{default_wallet_config, load_wallet_context},
};

pub mod client;
pub mod ptb;
pub mod transactions;
pub mod types;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let default_path = default_wallet_config()?;

    let mut wallet = load_wallet_context(default_path)?;

    let metadata_uri = "https://provider.json";
    let service = "Random Service";

    let client = SuiClientBuilder::default()
        .build("https://fullnode.testnet.sui.io:443")
        .await?;

    let sender = wallet.active_address()?;

    let tx_data = provider_create_service(
        &client,
        sender,
        service.to_string(),
        metadata_uri.to_string(),
    )
    .await?;

    let response = sign_and_execute_tx(&client, tx_data, wallet).await?;

    println!("{:#?}", response);

    Ok(())
}
