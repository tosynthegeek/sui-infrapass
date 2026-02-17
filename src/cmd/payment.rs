use anyhow::Result;
use clap::Subcommand;
use sui_sdk::SuiClient;
use sui_types::base_types::ObjectID;

use crate::{
    client::client_ext::SuiClientExt,
    transactions::payments::purchase_entitlement_tx,
    utils::{
        config::{default_wallet_config, load_wallet_context},
        handle_response,
    },
};

#[derive(Subcommand)]
pub enum PaymentCommands {
    /// Purchase an entitlement
    Purchase {
        /// Service object ID
        #[arg(short, long)]
        service_id: String,

        /// Tier object ID
        #[arg(short, long)]
        tier_id: String,

        /// Payment amount in smallest unit
        #[arg(short, long)]
        amount: u64,
    },
}

impl PaymentCommands {
    pub async fn execute(self, client: &SuiClient) -> Result<()> {
        match self {
            PaymentCommands::Purchase {
                service_id,
                tier_id,
                amount,
            } => {
                let default_path = default_wallet_config()?;
                // TODO: find a way to cache this
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;
                let service = ObjectID::from_hex_literal(&service_id)?;
                let tier = ObjectID::from_hex_literal(&tier_id)?;
                let tx_data =
                    purchase_entitlement_tx(client, sender, service, tier, amount).await?;
                let resp = client.sign_and_execute_tx(tx_data, wallet).await?;

                handle_response(&resp);

                Ok(())
            }
        }
    }
}
