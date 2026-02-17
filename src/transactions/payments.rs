use anyhow::Result;
use sui_sdk::{SuiClient, sui_sdk_types::bcs::ToBcs};
use sui_types::{
    Identifier,
    base_types::{ObjectID, SuiAddress},
    id::ID,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Command as SuiCommand, TransactionData},
};

use crate::{
    client::client_ext::SuiClientExt,
    ptb::{clock::clock_arg, object_ext::ObjectIDExt},
    types::settlement::UsageSettlement,
    utils::{
        coin::prepare_payment_coin,
        constants::{ENTITLEMENT_STORE_ID, PACKAGE_ID, REGISTRY_ID, USAGE_RELAYER_ID},
    },
};

pub async fn purchase_entitlement_tx(
    client: &SuiClient,
    sender: SuiAddress,
    service_id: ObjectID,
    tier_id: ObjectID,
    payment_amount: u64,
) -> Result<TransactionData> {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let tier_obj = client.get_tier_info(tier_id).await?;

    if payment_amount < tier_obj.price {
        anyhow::bail!(
            "Payment amount {} is less than tier price {}",
            tier_obj.coin_type.format_amount(payment_amount),
            tier_obj.coin_type.format_amount(tier_obj.price)
        );
    }

    let coin_type = tier_obj.coin_type;
    let coin_type_tag = coin_type.to_type_tag()?;

    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;
    let registry_id = ObjectID::from_hex_literal(REGISTRY_ID)?;
    let store_id = ObjectID::from_hex_literal(ENTITLEMENT_STORE_ID)?;

    let store_arg = store_id.to_shared_mut_ptb_arg(client, &mut ptb).await?;
    let service_arg = service_id.to_owned_ptb_arg(client, &mut ptb).await?;
    let registry_arg = registry_id.to_shared_imm_ptb_arg(client, &mut ptb).await?;
    let tier_arg = tier_id.to_owned_ptb_arg(client, &mut ptb).await?;
    let clock_arg = clock_arg(client, &mut ptb).await?;

    let payment_arg =
        prepare_payment_coin(&mut ptb, client, sender, coin_type, payment_amount).await?;

    ptb.command(SuiCommand::move_call(
        package_id,
        Identifier::new("payments")?,
        Identifier::new("purchase_entitlement")?,
        vec![coin_type_tag],
        vec![
            store_arg,
            service_arg,
            registry_arg,
            tier_arg,
            payment_arg,
            clock_arg,
        ],
    ));

    let pt = ptb.finish();
    client.build_tx_data(pt, sender).await
}

pub async fn settle_usage_batch_tx(
    client: &SuiClient,
    sender: SuiAddress,
    settlements: Vec<UsageSettlement>,
) -> Result<TransactionData> {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;
    let relayer_cap_id = ObjectID::from_hex_literal(USAGE_RELAYER_ID)?;
    let store_id = ObjectID::from_hex_literal(ENTITLEMENT_STORE_ID)?;

    if settlements.is_empty() {
        anyhow::bail!("No settlements provided");
    }

    let entitlement_ids: Vec<ID> = settlements
        .iter()
        .map(|s| s.entitlement_id.clone())
        .collect();

    let consumptions: Vec<u64> = settlements.iter().map(|s| s.amount).collect();

    println!("Settling {} entitlements", settlements.len());
    for settlement in &settlements {
        println!(
            "  - Entitlement {:?}: {} units",
            settlement.entitlement_id, settlement.amount
        );
    }

    let ent_ids = entitlement_ids.to_bcs()?;
    let consumption_ids = consumptions.to_bcs()?;

    let relayer_cap_arg = relayer_cap_id.to_owned_ptb_arg(client, &mut ptb).await?;
    let store_arg = store_id.to_shared_mut_ptb_arg(client, &mut ptb).await?;

    let ids_arg = ptb.pure(ent_ids)?;
    let amounts_arg = ptb.pure(consumption_ids)?;

    let clock_arg = clock_arg(client, &mut ptb).await?;

    ptb.command(SuiCommand::move_call(
        package_id,
        Identifier::new("payments")?,
        Identifier::new("settle_usage_batch")?,
        vec![],
        vec![relayer_cap_arg, store_arg, ids_arg, amounts_arg, clock_arg],
    ));

    let pt = ptb.finish();
    client.build_tx_data(pt, sender).await
}
