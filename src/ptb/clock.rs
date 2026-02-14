use anyhow::{Result, anyhow};
use sui_json_rpc_types::SuiObjectDataOptions;
use sui_sdk::SuiClient;
use sui_types::transaction::{Argument, ObjectArg, SharedObjectMutability};
use sui_types::{
    base_types::ObjectID, object::Owner,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
};

use crate::utils::constants::CLOCK_OBJECT_ID;

pub async fn clock_arg(
    client: &SuiClient,
    ptb: &mut ProgrammableTransactionBuilder,
) -> Result<Argument> {
    let clock_id = ObjectID::from_hex_literal(CLOCK_OBJECT_ID)?;

    let obj = client
        .read_api()
        .get_object_with_options(clock_id, SuiObjectDataOptions::new().with_owner())
        .await?;

    let data = obj
        .data
        .ok_or_else(|| anyhow!("Missing clock object data"))?;

    let owner = data.owner.ok_or_else(|| anyhow!("Clock missing owner"))?;

    let initial_shared_version = match owner {
        Owner::Shared {
            initial_shared_version,
        } => initial_shared_version,
        _ => return Err(anyhow!("Clock is not shared")),
    };

    Ok(ptb.obj(ObjectArg::SharedObject {
        id: clock_id,
        initial_shared_version,
        mutability: SharedObjectMutability::Immutable,
    })?)
}
