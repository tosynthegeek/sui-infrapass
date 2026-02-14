use anyhow::{Ok, Result};
use shared_crypto::intent::Intent;
use sui_json_rpc_types::{SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions};
use sui_keys::key_identity::KeyIdentity;
use sui_sdk::{SuiClient, types::transaction::Transaction, wallet_context::WalletContext};
use sui_types::{
    base_types::SuiAddress,
    transaction::{ProgrammableTransaction, TransactionData},
    transaction_driver_types::ExecuteTransactionRequestType,
};

pub async fn sign_and_execute_tx(
    client: &SuiClient,
    tx_data: TransactionData,
    mut wallet: WalletContext,
) -> Result<SuiTransactionBlockResponse, anyhow::Error> {
    let sender = wallet.active_address()?;
    let key = KeyIdentity::Address(sender);

    let signature = wallet
        .sign_secure(&key, &tx_data, Intent::sui_transaction())
        .await?;

    let tx = Transaction::from_data(tx_data, vec![signature]);

    let response = client
        .quorum_driver_api()
        .execute_transaction_block(
            tx,
            SuiTransactionBlockResponseOptions::full_content(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    Ok(response)
}

pub async fn build_tx_data(
    pt: ProgrammableTransaction,
    client: &SuiClient,
    sender: SuiAddress,
) -> Result<TransactionData> {
    let gas_coins = client
        .coin_read_api()
        .get_coins(sender, None, None, None)
        .await?;

    let gas_coin = gas_coins
        .data
        .first()
        .ok_or_else(|| anyhow::anyhow!("No gas coins available for sender"))?;

    let gas_object = (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest);

    let gas_price = client.read_api().get_reference_gas_price().await?;

    let tx_data =
        TransactionData::new_programmable(sender, vec![gas_object], pt, 10_000_000, gas_price);

    Ok(tx_data)
}
