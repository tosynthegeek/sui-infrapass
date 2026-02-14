use anyhow::{Result, anyhow};
use async_trait::async_trait;
use sui_json_rpc_types::SuiObjectDataOptions;
use sui_sdk::SuiClient;
use sui_types::transaction::{Argument, ObjectArg, SharedObjectMutability};
use sui_types::{
    base_types::ObjectID, object::Owner,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
};

#[async_trait]
pub trait ObjectIDExt {
    async fn to_owned_ptb_arg(
        &self,
        client: &SuiClient,
        ptb: &mut ProgrammableTransactionBuilder,
    ) -> Result<Argument>;

    async fn to_shared_mut_ptb_arg(
        &self,
        client: &SuiClient,
        ptb: &mut ProgrammableTransactionBuilder,
    ) -> Result<Argument>;

    async fn to_shared_imm_ptb_arg(
        &self,
        client: &SuiClient,
        ptb: &mut ProgrammableTransactionBuilder,
    ) -> Result<Argument>;
}

#[async_trait]
impl ObjectIDExt for ObjectID {
    async fn to_owned_ptb_arg(
        &self,
        client: &SuiClient,
        ptb: &mut ProgrammableTransactionBuilder,
    ) -> Result<Argument> {
        let obj = client
            .read_api()
            .get_object_with_options(*self, SuiObjectDataOptions::new().with_owner())
            .await?;

        let data = obj.data.ok_or_else(|| anyhow!("Missing object data"))?;

        Ok(ptb.obj(ObjectArg::ImmOrOwnedObject(data.object_ref()))?)
    }

    async fn to_shared_mut_ptb_arg(
        &self,
        client: &SuiClient,
        ptb: &mut ProgrammableTransactionBuilder,
    ) -> Result<Argument> {
        let obj = client
            .read_api()
            .get_object_with_options(*self, SuiObjectDataOptions::new().with_owner())
            .await?;

        let data = obj.data.ok_or_else(|| anyhow!("Missing object data"))?;

        let owner = data
            .owner
            .ok_or_else(|| anyhow!("Shared object missing owner"))?;

        let initial_shared_version = match owner {
            Owner::Shared {
                initial_shared_version,
            } => initial_shared_version,
            _ => return Err(anyhow!("Object is not shared")),
        };

        Ok(ptb.obj(ObjectArg::SharedObject {
            id: *self,
            initial_shared_version,
            mutability: SharedObjectMutability::Mutable,
        })?)
    }

    async fn to_shared_imm_ptb_arg(
        &self,
        client: &SuiClient,
        ptb: &mut ProgrammableTransactionBuilder,
    ) -> Result<Argument> {
        let obj = client
            .read_api()
            .get_object_with_options(*self, SuiObjectDataOptions::new().with_owner())
            .await?;

        let data = obj.data.ok_or_else(|| anyhow!("Missing object data"))?;

        let owner = data
            .owner
            .ok_or_else(|| anyhow!("Shared object missing owner"))?;

        let initial_shared_version = match owner {
            Owner::Shared {
                initial_shared_version,
            } => initial_shared_version,
            _ => return Err(anyhow!("Object is not shared")),
        };

        Ok(ptb.obj(ObjectArg::SharedObject {
            id: *self,
            initial_shared_version,
            mutability: SharedObjectMutability::Immutable,
        })?)
    }
}
