use std::{sync::Arc, time::Duration};
use sui_types::base_types::ObjectID;
use tracing::{error, info};

use sui_sdk::SuiClient;
use uuid::Uuid;

use crate::{
    client::client_ext::SuiClientExt,
    db::repository::Repository,
    transactions::payments::settle_usage_batch_tx,
    types::settlement::UsageSettlement,
    utils::{
        config::{default_wallet_config, load_wallet_context},
        error::InfrapassError,
    },
};

pub async fn settlement_worker(
    repo: Arc<Repository>,
    client: Arc<SuiClient>,
    interval_secs: u64,
) -> Result<(), InfrapassError> {
    let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
    let default_path = default_wallet_config()?;
    let mut wallet = load_wallet_context(default_path)?;
    let sender = wallet.active_address()?;

    loop {
        ticker.tick().await;

        let pending = match repo.get_unsettled_aggregated().await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to fetch pending settlements: {}", e);
                continue;
            }
        };

        if pending.is_empty() {
            continue;
        }

        let settlements: Vec<UsageSettlement> = pending
            .iter()
            .filter_map(|p| match ObjectID::from_hex_literal(&p.entitlement_id) {
                Ok(oid) => Some(UsageSettlement {
                    entitlement_id: sui_types::id::ID::new(oid),
                    amount: p.total_amount as u64,
                }),
                Err(e) => {
                    error!("Invalid entitlement_id {}: {}", p.entitlement_id, e);
                    None
                }
            })
            .collect();

        if settlements.is_empty() {
            continue;
        }

        match settle_usage_batch_tx(&client, sender, settlements).await {
            Ok(tx_data) => match client.sign_and_execute_tx(tx_data, &mut wallet).await {
                Ok(digest) => {
                    info!("Settled batch digest={}", digest);
                    let ids: Vec<Uuid> = pending
                        .iter()
                        .flat_map(|p| p.event_ids.iter().copied())
                        .collect();
                    if let Err(e) = repo.mark_settled(&ids).await {
                        error!("Settled onchain but failed to mark in DB: {}", e);
                    }
                }
                Err(e) => error!("Tx execution failed: {}", e),
            },
            Err(e) => error!("Tx build failed: {}", e),
        }
    }
}
