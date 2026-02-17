use anyhow::Result;
use sui_sdk::SuiClient;
use sui_types::{
    Identifier,
    base_types::{ObjectID, SuiAddress},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Command as SuiCommand, TransactionData},
};

use crate::{
    client::client_ext::SuiClientExt, ptb::{clock::clock_arg, object_ext::ObjectIDExt, tier_config::build_tier_config_args}, transactions::provider::get_provider_state, types::{coin::CoinType, types::TierConfigInput}, utils::constants::{PACKAGE_ID, REGISTRY_ID}
};

pub async fn create_pricing_tier_tx(
    client: &SuiClient,
    sender: SuiAddress,
    service_id: ObjectID,
    tier_name: String,
    price: u64,
    config: TierConfigInput,
    coin_type: u8,
) -> Result<TransactionData> {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;
    let registry_id = ObjectID::from_hex_literal(REGISTRY_ID)?;

    let provider_state = get_provider_state(client, sender).await?;

    let service_arg = service_id.to_owned_ptb_arg(client, &mut ptb).await?;

    let cap_arg = provider_state
        .cap_id
        .to_owned_ptb_arg(client, &mut ptb)
        .await?;

    let registry_arg = registry_id.to_shared_imm_ptb_arg(client, &mut ptb).await?;

    let clock_arg = clock_arg(client, &mut ptb).await?;

    let (tier_type_arg, duration_arg, quota_arg, unit_price_arg) =
        build_tier_config_args(&mut ptb, config)?;

    let name_arg = ptb.pure(tier_name.into_bytes())?;
    let price_arg = ptb.pure(price)?;

    let coin_type_tag = CoinType::u8_to_typetag(coin_type)?;

    ptb.command(SuiCommand::move_call(
        package_id,
        Identifier::new("pricing")?,
        Identifier::new("create_pricing_tier_entry")?,
        vec![coin_type_tag],
        vec![
            service_arg,
            cap_arg,
            registry_arg,
            name_arg,
            price_arg,
            tier_type_arg,
            duration_arg,
            quota_arg,
            unit_price_arg,
            clock_arg,
        ],
    ));

    let pt = ptb.finish();

    client.build_tx_data(pt, sender).await
}

pub async fn add_tier_to_service_tx(
    client: &SuiClient,
    sender: SuiAddress,
    service_id: ObjectID,
    tier_id: ObjectID,
) -> Result<TransactionData> {
    let registry_id = ObjectID::from_hex_literal(REGISTRY_ID)?;
    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let provider_state = get_provider_state(client, sender).await?;

    let registry_arg = registry_id.to_shared_imm_ptb_arg(client, &mut ptb).await?;

    let service_arg = service_id.to_owned_ptb_arg(client, &mut ptb).await?;

    let tier_arg = tier_id.to_owned_ptb_arg(client, &mut ptb).await?;

    let provider_cap_arg = provider_state
        .cap_id
        .to_owned_ptb_arg(client, &mut ptb)
        .await?;

    let clock_arg = clock_arg(client, &mut ptb).await?;

    ptb.command(SuiCommand::move_call(
        package_id,
        Identifier::new("pricing")?,
        Identifier::new("add_tier_to_service")?,
        vec![],
        vec![
            service_arg,
            registry_arg,
            provider_cap_arg,
            tier_arg,
            clock_arg,
        ],
    ));

    let pt = ptb.finish();

    client.build_tx_data(pt, sender).await
}

pub async fn update_tier_price_tx(
    client: &SuiClient,
    sender: SuiAddress,
    new_price: u64,
    tier_id: ObjectID,
    coin_type: u8,
) -> Result<TransactionData> {
    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let provider_state = get_provider_state(client, sender).await?;

    let tier_arg = tier_id.to_owned_ptb_arg(client, &mut ptb).await?;

    let provider_cap_arg = provider_state
        .cap_id
        .to_owned_ptb_arg(client, &mut ptb)
        .await?;

    let clock_arg = clock_arg(client, &mut ptb).await?;

    let price_arg = ptb.pure(new_price)?;
    let coin_type_tag = CoinType::u8_to_typetag(coin_type)?;

    ptb.command(SuiCommand::move_call(
        package_id,
        Identifier::new("pricing")?,
        Identifier::new("update_tier_price")?,
        vec![coin_type_tag],
        vec![tier_arg, provider_cap_arg, price_arg, clock_arg],
    ));

    let pt = ptb.finish();

    client.build_tx_data(pt, sender).await
}

pub async fn deactivate_tier_tx(
    client: &SuiClient,
    sender: SuiAddress,
    tier_id: ObjectID,
    coin_type: u8,
) -> Result<TransactionData> {
    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let provider_state = get_provider_state(client, sender).await?;

    let tier_arg = tier_id.to_owned_ptb_arg(client, &mut ptb).await?;

    let provider_cap_arg = provider_state
        .cap_id
        .to_owned_ptb_arg(client, &mut ptb)
        .await?;

    let clock_arg = clock_arg(client, &mut ptb).await?;

    let coin_type_tag = CoinType::u8_to_typetag(coin_type)?;

    ptb.command(SuiCommand::move_call(
        package_id,
        Identifier::new("pricing")?,
        Identifier::new("deactivate_tier")?,
        vec![coin_type_tag],
        vec![tier_arg, provider_cap_arg, clock_arg],
    ));

    let pt = ptb.finish();

    client.build_tx_data(pt, sender).await
}

pub async fn reactivate_tier_tx(
    client: &SuiClient,
    sender: SuiAddress,
    tier_id: ObjectID,
    coin_type: u8,
) -> Result<TransactionData> {
    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let provider_state = get_provider_state(client, sender).await?;

    let tier_arg = tier_id.to_owned_ptb_arg(client, &mut ptb).await?;

    let provider_cap_arg = provider_state
        .cap_id
        .to_owned_ptb_arg(client, &mut ptb)
        .await?;

    let clock_arg = clock_arg(client, &mut ptb).await?;

    let coin_type_tag = CoinType::u8_to_typetag(coin_type)?;

    ptb.command(SuiCommand::move_call(
        package_id,
        Identifier::new("pricing")?,
        Identifier::new("reactivate_tier")?,
        vec![coin_type_tag],
        vec![tier_arg, provider_cap_arg, clock_arg],
    ));

    let pt = ptb.finish();

    client.build_tx_data(pt, sender).await
}

pub async fn remove_tier_from_service_tx(
    client: &SuiClient,
    sender: SuiAddress,
    tier_id: ObjectID,
    service_id: ObjectID,
) -> Result<TransactionData> {
    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;
    let registry_id = ObjectID::from_hex_literal(REGISTRY_ID)?;
    let mut ptb = ProgrammableTransactionBuilder::new();

    let provider_state = get_provider_state(client, sender).await?;
    let service_arg = service_id.to_owned_ptb_arg(client, &mut ptb).await?;
    let tier_arg = ptb.pure(tier_id)?;
    let registry_arg = registry_id.to_shared_imm_ptb_arg(client, &mut ptb).await?;

    let provider_cap_arg = provider_state
        .cap_id
        .to_owned_ptb_arg(client, &mut ptb)
        .await?;

    let clock_arg = clock_arg(client, &mut ptb).await?;

    ptb.command(SuiCommand::move_call(
        package_id,
        Identifier::new("pricing")?,
        Identifier::new("remove_tier_from_service")?,
        vec![],
        vec![
            service_arg,
            provider_cap_arg,
            registry_arg,
            tier_arg,
            clock_arg,
        ],
    ));

    let pt = ptb.finish();

    client.build_tx_data(pt, sender).await
}
