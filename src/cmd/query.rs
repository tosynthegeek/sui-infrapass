use anyhow::{Ok, Result};
use clap::Subcommand;
use sui_sdk::SuiClient;
use tracing::info;

use crate::{
    transactions::provider::get_provider_state,
    utils::config::{default_wallet_config, load_wallet_context},
};

#[derive(Subcommand)]
pub enum QueryCommands {
    /// Get provider info
    Provider {},
    // /// Get service info
    // Service {
    //     /// Service object ID
    //     #[arg(short, long)]
    //     service_id: String,
    // },

    // /// Get tier info
    // Tier {
    //     /// Tier object ID
    //     #[arg(short, long)]
    //     tier_id: String,
    // },
}

impl QueryCommands {
    pub async fn execute(&self, client: &SuiClient) -> Result<()> {
        match self {
            QueryCommands::Provider {} => {
                let default_path = default_wallet_config()?;
                // TODO: find a way to cache this
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;
                let prov_state = get_provider_state(client, sender).await?;

                info!("{:?}", prov_state);

                Ok(())
            }
        }
    }
}
