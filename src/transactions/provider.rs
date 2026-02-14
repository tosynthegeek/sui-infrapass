use anyhow::Result;
use serde::{Deserialize, Serialize};
use sui_json_rpc_types::{SuiData, SuiObjectDataOptions};
use sui_sdk::SuiClient;
use sui_types::base_types::{ObjectID, SuiAddress};

use crate::client::client_ext::SuiClientExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProviderState {
    pub profile_id: ObjectID,
    pub cap_id: ObjectID,
    pub service_ids: Vec<ObjectID>,
}

pub async fn get_provider_state(client: &SuiClient, sender: SuiAddress) -> Result<ProviderState> {
    // TODO: get provider state from cache / storage
    // if not available, fallback to query

    client.provider_state(sender).await
}

pub async fn fetch_tiers_for_service(
    client: &SuiClient,
    service_id: ObjectID,
) -> Result<Vec<ObjectID>> {
    let obj = client
        .read_api()
        .get_object_with_options(service_id, SuiObjectDataOptions::new().with_content())
        .await?;

    let mut tier_ids = vec![];

    if let Some(content) = obj.data.unwrap().content {
        if let Some(obj) = content.try_into_move() {
            let fields = obj.fields.to_json_value();
            if let Some(tiers) = fields.get("pricing_tier_ids") {
                if let Some(contents) = tiers.get("contents").and_then(|v| v.as_array()) {
                    tier_ids = contents
                        .iter()
                        .filter_map(|id| {
                            id.as_str().and_then(|s| ObjectID::from_hex_literal(s).ok())
                        })
                        .collect();
                }
            }
        }
    }

    Ok(tier_ids)
}
