use anyhow::Result;
use sui_sdk::SuiClient;
use sui_types::{
    Identifier,
    base_types::{ObjectID, SequenceNumber, SuiAddress},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Command, ObjectArg, SharedObjectMutability, TransactionData},
};

use crate::{
    client::client_ext::SuiClientExt,
    ptb::{clock::clock_arg, object_ext::ObjectIDExt},
    transactions::provider::get_provider_state,
    utils::constants::{CLOCK_OBJECT_ID, PACKAGE_ID, REGISTRY_ID},
};

pub async fn register_provider_tx(
    client: &SuiClient,
    sender: SuiAddress,
    metadata_uri: String,
) -> Result<TransactionData> {
    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;
    let registry_id = ObjectID::from_hex_literal(REGISTRY_ID)?;
    let clock_id = ObjectID::from_hex_literal(CLOCK_OBJECT_ID)?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let registry_arg = registry_id.to_shared_mut_ptb_arg(client, &mut ptb).await?;

    let metadata_bytes: Vec<u8> = metadata_uri.into_bytes();
    let metadata_arg = ptb.pure(metadata_bytes)?;

    let clock_arg = ptb.obj(ObjectArg::SharedObject {
        id: clock_id,
        initial_shared_version: SequenceNumber::from_u64(1),
        mutability: SharedObjectMutability::Immutable,
    })?;

    ptb.command(Command::move_call(
        package_id,
        Identifier::new("registry")?,
        Identifier::new("register_provider_entry")?,
        vec![],
        vec![registry_arg, metadata_arg, clock_arg],
    ));

    let pt = ptb.finish();

    let tx_data = client.build_tx_data(pt, sender).await?;

    Ok(tx_data)
}

pub async fn provider_create_service(
    client: &SuiClient,
    sender: SuiAddress,
    service_type: String,
    metadata_uri: String,
) -> Result<TransactionData> {
    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;
    let registry_id = ObjectID::from_hex_literal(REGISTRY_ID)?;

    let provider_state = get_provider_state(client, sender).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let registry_arg = registry_id.to_shared_mut_ptb_arg(client, &mut ptb).await?;

    let provider_profile_arg = provider_state
        .profile_id
        .to_owned_ptb_arg(client, &mut ptb)
        .await?;

    let provider_cap_arg = provider_state
        .cap_id
        .to_owned_ptb_arg(client, &mut ptb)
        .await?;

    let service_type_arg = ptb.pure(service_type.into_bytes())?;
    let metadata_arg = ptb.pure(metadata_uri.into_bytes())?;

    let clock_arg = clock_arg(client, &mut ptb).await?;

    ptb.command(Command::move_call(
        package_id,
        Identifier::new("registry")?,
        Identifier::new("create_service_entry")?,
        vec![],
        vec![
            registry_arg,
            provider_profile_arg,
            provider_cap_arg,
            service_type_arg,
            metadata_arg,
            clock_arg,
        ],
    ));

    let pt = ptb.finish();

    client.build_tx_data(pt, sender).await
}

pub async fn set_service_active_tx(
    client: &SuiClient,
    sender: SuiAddress,
    service_id: ObjectID,
) -> Result<TransactionData> {
    let registry_id = ObjectID::from_hex_literal(REGISTRY_ID)?;
    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let registry_arg = registry_id.to_shared_imm_ptb_arg(client, &mut ptb).await?;

    let service_arg = service_id.to_owned_ptb_arg(client, &mut ptb).await?;

    let clock_arg = clock_arg(client, &mut ptb).await?;

    ptb.command(Command::move_call(
        package_id,
        Identifier::new("registry")?,
        Identifier::new("set_service_active_entry")?,
        vec![],
        vec![registry_arg, service_arg, clock_arg],
    ));

    let pt = ptb.finish();

    client.build_tx_data(pt, sender).await
}

pub async fn update_service_metadata_tx(
    client: &SuiClient,
    sender: SuiAddress,
    service_id: ObjectID,
    metadata_uri: String,
) -> Result<TransactionData> {
    let registry_id = ObjectID::from_hex_literal(REGISTRY_ID)?;
    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let registry_arg = registry_id.to_shared_imm_ptb_arg(client, &mut ptb).await?;

    let service_arg = service_id.to_owned_ptb_arg(client, &mut ptb).await?;
    let metadata_arg = ptb.pure(metadata_uri.into_bytes())?;
    let clock_arg = clock_arg(client, &mut ptb).await?;

    ptb.command(Command::move_call(
        package_id,
        Identifier::new("registry")?,
        Identifier::new("update_service_metadata_entry")?,
        vec![],
        vec![registry_arg, service_arg, metadata_arg, clock_arg],
    ));

    let pt = ptb.finish();

    client.build_tx_data(pt, sender).await
}

// pub async fn update_provider_address_tx(
//     client: &SuiClient,
//     sender: SuiAddress,
//     service_id: ObjectID,
//     provider_id: ObjectID,
// ) -> Result<TransactionData> {
//     let registry_id = ObjectID::from_hex_literal(REGISTRY_ID)?;
//     let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;

//     let mut ptb = ProgrammableTransactionBuilder::new();

//     let registry_arg = registry_id.to_shared_imm_ptb_arg(client, &mut ptb).await?;

//     let service_arg = service_id.to_owned_ptb_arg(client, &mut ptb).await?;

//     let metadata_arg = ptb.pure(metadata_uri.into_bytes())?;

//     let clock_arg = clock_arg(client, &mut ptb).await?;

//     ptb.command(Command::move_call(
//         package_id,
//         Identifier::new("registry")?,
//         Identifier::new("update_provider_address_entry")?,
//         vec![],
//         vec![registry_arg, service_arg, metadata_arg, clock_arg],
//     ));

//     let pt = ptb.finish();

//     client.build_tx_data(pt, sender).await
// }
