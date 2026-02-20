use anyhow::{Ok, Result};
use clap::Subcommand;
use sui_json_rpc_types::SuiTransactionBlockEffectsAPI;
use sui_sdk::SuiClient;
use sui_types::base_types::ObjectID;
use tracing::info;

use crate::{
    client::client_ext::SuiClientExt,
    transactions::registry::{
        provider_create_service, register_provider_tx, set_service_active_tx,
        update_service_metadata_tx,
    },
    utils::{
        config::{default_wallet_config, load_wallet_context},
        handle_response,
    },
};

#[derive(Subcommand)]
pub enum RegistryCommands {
    /// Register as a provider
    Register {
        /// Metadata URI for the provider
        #[arg(short, long)]
        metadata_uri: String,
    },

    /// Create a new service
    CreateService {
        /// Type of service
        #[arg(short, long)]
        service_type: String,

        /// Metadata URI for the service
        #[arg(short, long)]
        metadata_uri: String,
    },

    /// Update service metadata
    UpdateServiceMetadata {
        /// Service object ID
        #[arg(short, long)]
        service_id: String,

        /// New metadata URI
        #[arg(short, long)]
        metadata_uri: String,
    },

    /// Set service as active
    SetServiceActive {
        /// Service object ID
        #[arg(short, long)]
        service_id: String,
    },
}

impl RegistryCommands {
    pub async fn execute(self, client: &SuiClient) -> Result<()> {
        match self {
            RegistryCommands::Register { metadata_uri } => {
                let default_path = default_wallet_config()?;
                // TODO: find a way to cache this
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;
                info!("Registering provider with address {} ...", sender);
                let data = register_provider_tx(client, sender, metadata_uri).await?;
                let resp = client.sign_and_execute_tx(data, wallet).await?;

                handle_response(&resp);

                Ok(())
            }
            RegistryCommands::CreateService {
                service_type,
                metadata_uri,
            } => {
                let default_path = default_wallet_config()?;
                // TODO: find a way to cache this
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;
                info!("Creating service with address {} ...", sender);
                let data =
                    provider_create_service(client, sender, service_type, metadata_uri).await?;
                let resp = client.sign_and_execute_tx(data, wallet).await?;

                handle_response(&resp);
                let effects = resp
                    .effects
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Missing transaction effects"))?;

                for created in effects.created() {
                    let object_id = created.reference.object_id;
                    let version = created.reference.version;

                    info!("Created object: {} @ version {}", object_id, version);
                }

                Ok(())
            }
            RegistryCommands::UpdateServiceMetadata {
                service_id,
                metadata_uri,
            } => {
                let default_path = default_wallet_config()?;
                // TODO: find a way to cache this
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;
                info!("Updating service {} metadata...", service_id);

                let service = ObjectID::from_hex_literal(&service_id)?;
                let data =
                    update_service_metadata_tx(client, sender, service, metadata_uri).await?;
                let resp = client.sign_and_execute_tx(data, wallet).await?;
                handle_response(&resp);
                Ok(())
            }
            RegistryCommands::SetServiceActive { service_id } => {
                let default_path = default_wallet_config()?;
                // TODO: find a way to cache this
                let mut wallet = load_wallet_context(default_path)?;
                let sender = wallet.active_address()?;
                info!("Setting service {} to active...", service_id);

                let service = ObjectID::from_hex_literal(&service_id)?;
                let data = set_service_active_tx(client, sender, service).await?;

                let resp = client.sign_and_execute_tx(data, wallet).await?;
                handle_response(&resp);
                Ok(())
            }
        }
    }
}
