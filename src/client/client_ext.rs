use anyhow::{Result, anyhow};
use async_trait::async_trait;
use sui_json_rpc_types::{SuiData, SuiObjectDataOptions, SuiObjectResponseQuery};
use sui_sdk::SuiClient;
use sui_types::base_types::{ObjectID, SuiAddress};

use crate::{
    transactions::provider::ProviderState,
    types::{coin::CoinType, types::TierInfo},
    utils::coin::{extract_coin_type_from_tier_type, extract_price_from_content},
};

#[async_trait]
pub trait SuiClientExt {
    async fn get_tier_info(&self, tier_id: ObjectID) -> Result<TierInfo>;
    async fn get_balance(&self, owner: SuiAddress, coin_type: CoinType) -> Result<u128>;
    async fn provider_state(&self, sender: SuiAddress) -> Result<ProviderState>;
}

#[async_trait]
impl SuiClientExt for SuiClient {
    async fn get_tier_info(&self, tier_id: ObjectID) -> Result<TierInfo> {
        let tier_obj = self
            .read_api()
            .get_object_with_options(
                tier_id,
                SuiObjectDataOptions::new().with_type().with_content(),
            )
            .await?;

        let tier_data = tier_obj
            .data
            .ok_or_else(|| anyhow::anyhow!("Tier object not found"))?;

        let tier_type = tier_data
            .type_
            .ok_or_else(|| anyhow::anyhow!("Could not get tier type"))?;

        let coin_type = extract_coin_type_from_tier_type(&tier_type.to_string())?;

        let price = extract_price_from_content(&tier_data.content)?;

        Ok(TierInfo {
            coin_type,
            price,
            tier_type_string: tier_type.to_string(),
        })
    }

    async fn get_balance(&self, owner: SuiAddress, coin_type: CoinType) -> Result<u128> {
        let balance = self
            .coin_read_api()
            .get_balance(owner, Some(coin_type.to_type_tag()?.to_string()))
            .await?;
        Ok(balance.total_balance)
    }

    async fn provider_state(&self, sender: SuiAddress) -> Result<ProviderState> {
        let objects = self
            .read_api()
            .get_owned_objects(
                sender,
                Some(SuiObjectResponseQuery::new_with_options(
                    SuiObjectDataOptions::new().with_type().with_content(),
                )),
                None,
                None,
            )
            .await?;

        let mut profile = None;
        let mut cap = None;
        let mut service_ids = vec![];

        for obj in objects.data {
            let data = obj.data.unwrap();
            let type_str = data.type_.unwrap().to_string();

            if type_str.contains("ProviderProfile") {
                profile = Some(data.object_id);
                if let Some(content) = data.content {
                    if let Some(obj) = content.try_into_move() {
                        let fields = obj.fields.to_json_value();
                        if let Some(service_vecset) = fields.get("service_ids") {
                            if let Some(contents) =
                                service_vecset.get("contents").and_then(|v| v.as_array())
                            {
                                service_ids = contents
                                    .iter()
                                    .filter_map(|id| {
                                        id.as_str().and_then(|s| ObjectID::from_hex_literal(s).ok())
                                    })
                                    .collect();
                            }
                        }
                    }
                }
            }

            if type_str.contains("ProviderCap") {
                cap = Some(data.object_id);
            }
        }

        let provider_state = ProviderState {
            profile_id: profile.ok_or_else(|| anyhow!("Missing profile"))?,
            cap_id: cap.ok_or_else(|| anyhow!("Missing cap"))?,
            service_ids,
        };

        Ok(provider_state)
    }
}
