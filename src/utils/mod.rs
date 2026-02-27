use sui_json_rpc_types::{
    SuiExecutionStatus, SuiTransactionBlockEffectsAPI, SuiTransactionBlockResponse,
};
use tracing::{error, info};

pub mod address;
pub mod coin;
pub mod config;
pub mod constants;
pub mod error;
pub mod logs_fmt;

pub fn handle_response(resp: &SuiTransactionBlockResponse) {
    match resp.status_ok() {
        Some(true) => {
            let tx_digest = resp.digest;
            info!(
                "Transaction digest: {} waiting for tx to be indexed...",
                tx_digest
            );
        }
        Some(false) => {
            if let Some(effects) = &resp.effects {
                let status = effects.status();
                match status {
                    SuiExecutionStatus::Failure { error } => {
                        error!("Transaction failed with error: {:?}", error);
                    }
                    _ => {
                        error!("Transaction failed for unknown reason: {:?}", status);
                    }
                }
            } else {
                error!("Transaction failed and no effects were returned");
            }
        }

        None => {
            println!("No execution status returned");
        }
    }
}

pub async fn get_checkpoint_with_retry(
    client: &sui_sdk::SuiClient,
    tx_digest: sui_types::base_types::TransactionDigest,
    max_retries: u32,
    delay_ms: u64,
) -> Option<u64> {
    for attempt in 0..max_retries {
        match client
            .read_api()
            .get_transaction_with_options(
                tx_digest,
                sui_json_rpc_types::SuiTransactionBlockResponseOptions::new()
                    .with_effects()
                    .with_events(),
            )
            .await
        {
            Ok(resp) => {
                if let Some(checkpoint) = resp.checkpoint {
                    info!("Transaction executed in checkpoint: {}", checkpoint);
                    return Some(checkpoint);
                } else {
                    info!(
                        "Attempt {}: Checkpoint not yet available for transaction {}",
                        attempt + 1,
                        tx_digest
                    );
                }
            }
            Err(e) => {
                info!(
                    "Attempt {}: Error fetching transaction {}: {}",
                    attempt + 1,
                    tx_digest,
                    e
                );
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }
    None
}

pub fn get_channel(provider_id: &str) -> String {
    format!("infrapass:{provider_id}:events")
}
