use anyhow::{Ok, Result};
use clap::Subcommand;
use sui_sdk::SuiClient;
use sui_types::base_types::ObjectID;

use crate::{
    client::client_ext::SuiClientExt,
    transactions::pricing::{
        add_tier_to_service_tx, create_pricing_tier_tx, deactivate_tier_tx, reactivate_tier_tx,
        remove_tier_from_service_tx, update_tier_price_tx,
    },
    types::types::TierConfigInput,
    utils::{
        config::{default_wallet_config, load_wallet_context},
        handle_response,
    },
};

#[derive(Subcommand)]
pub enum PricingCommands {
    /// Create a new pricing tier
    CreateTier {
        /// Service object ID
        #[arg(short, long)]
        service_id: String,

        /// Tier name
        #[arg(short, long)]
        name: String,

        /// tier type
        #[arg(short, long)]
        tier: u8,

        /// Price in smallest unit
        #[arg(short, long)]
        price: u64,

        /// Coin type (0=SUI, 1=WAL, 2=USDC, 3=USDT)
        #[arg(short, long)]
        coin_type: u8,

        /// Duration in days (for subscription)
        #[arg(long)]
        duration: Option<u64>,

        /// Quota (for subscription or PAYG)
        #[arg(long)]
        quota: Option<u64>,

        /// Unit price (for PAYG)
        #[arg(long)]
        unit_price: Option<u64>,
    },

    /// Add tier to service
    AddToService {
        /// Service object ID
        #[arg(short, long)]
        service_id: String,

        /// Tier object ID
        #[arg(short, long)]
        tier_id: String,
    },

    /// Update tier price
    UpdatePrice {
        /// Tier object ID
        #[arg(short, long)]
        tier_id: String,

        /// New price in smallest unit
        #[arg(short, long)]
        new_price: u64,

        /// Coin type (0=SUI, 1=WAL, 2=USDC, 3=USDT)
        #[arg(short, long)]
        coin_type: u8,
    },

    /// Deactivate a tier
    Deactivate {
        /// Tier object ID
        #[arg(short, long)]
        tier_id: String,

        /// Coin type (0=SUI, 1=WAL, 2=USDC, 3=USDT)
        #[arg(short, long)]
        coin_type: u8,
    },

    /// Reactivate a tier
    Reactivate {
        /// Tier object ID
        #[arg(short, long)]
        tier_id: String,

        /// Coin type (0=SUI, 1=WAL, 2=USDC, 3=USDT)
        #[arg(short, long)]
        coin_type: u8,
    },

    /// Remove tier from service
    RemoveFromService {
        /// Tier object ID
        #[arg(short, long)]
        tier_id: String,

        /// Service object ID
        #[arg(short, long)]
        service_id: String,
    },
}

impl PricingCommands {
    pub async fn execute(&self, client: &SuiClient) -> Result<()> {
        match self {
            PricingCommands::CreateTier {
                service_id,
                name,
                tier,
                price,
                coin_type,
                duration,
                quota,
                unit_price,
            } => {
                let default_path = default_wallet_config()?;
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;
                let service = ObjectID::from_hex_literal(&service_id)?;
                let config = TierConfigInput::from_u8(tier, duration, quota, unit_price)?;
                let tx_data = create_pricing_tier_tx(
                    &client,
                    sender,
                    service,
                    name.to_string(),
                    *price,
                    config,
                    *coin_type,
                )
                .await?;
                let resp = client.sign_and_execute_tx(tx_data, wallet).await?;
                handle_response(&resp);

                Ok(())
            }
            PricingCommands::AddToService {
                service_id,
                tier_id,
            } => {
                let default_path = default_wallet_config()?;
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;
                let service = ObjectID::from_hex_literal(&service_id)?;
                let tier = ObjectID::from_hex_literal(&tier_id)?;

                let tx_data = add_tier_to_service_tx(&client, sender, service, tier).await?;
                let resp = client.sign_and_execute_tx(tx_data, wallet).await?;
                handle_response(&resp);
                Ok(())
            }
            PricingCommands::UpdatePrice {
                tier_id,
                new_price,
                coin_type,
            } => {
                let default_path = default_wallet_config()?;
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;

                let tier = ObjectID::from_hex_literal(&tier_id)?;

                let tx_data =
                    update_tier_price_tx(&client, sender, *new_price, tier, *coin_type).await?;

                let resp = client.sign_and_execute_tx(tx_data, wallet).await?;
                handle_response(&resp);
                Ok(())
            }
            PricingCommands::Deactivate { tier_id, coin_type } => {
                let default_path = default_wallet_config()?;
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;
                let tier = ObjectID::from_hex_literal(&tier_id)?;
                let tx_data = deactivate_tier_tx(&client, sender, tier, *coin_type).await?;

                let _ = client.sign_and_execute_tx(tx_data, wallet).await?;
                Ok(())
            }
            PricingCommands::Reactivate { tier_id, coin_type } => {
                let default_path = default_wallet_config()?;
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;

                let tier = ObjectID::from_hex_literal(&tier_id)?;
                let tx_data = reactivate_tier_tx(&client, sender, tier, *coin_type).await?;
                let resp = client.sign_and_execute_tx(tx_data, wallet).await?;
                handle_response(&resp);
                Ok(())
            }
            PricingCommands::RemoveFromService {
                tier_id,
                service_id,
            } => {
                let default_path = default_wallet_config()?;
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;
                let service = ObjectID::from_hex_literal(&service_id)?;
                let tier = ObjectID::from_hex_literal(&tier_id)?;

                let tx_data = remove_tier_from_service_tx(&client, sender, tier, service).await?;

                let resp = client.sign_and_execute_tx(tx_data, wallet).await?;
                handle_response(&resp);
                Ok(())
            }
        }
    }
}
