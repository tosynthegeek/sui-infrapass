use redis::{Client as RedisClient, aio::MultiplexedConnection};
use tracing::info;

use crate::{
    events::types::EntitlementPurchased,
    pubsub::types::{EntitlementUpdateEvent, PubSubAction, PubSubEvent, TierEntitlement},
    utils::{error::InfrapassError, get_channel, logs_fmt::abbrev},
};

pub struct PubSubPublisher {
    pub redis_client: RedisClient,
    redis: MultiplexedConnection,
}

impl PubSubPublisher {
    pub async fn new(redis_client: RedisClient) -> Result<Self, InfrapassError> {
        let redis = redis_client.get_multiplexed_async_connection().await?;
        Ok(Self {
            redis_client,
            redis,
        })
    }

    pub async fn publish_refresh(
        &self,
        provider_id: &str,
        event: &EntitlementPurchased,
    ) -> Result<(), InfrapassError> {
        let channel = get_channel(provider_id);
        let tier_type = event.inner.type_u8();
        let tier_id = event.tier_id.bytes.to_string();
        let ent_id = event.entitlement_id.bytes.to_string();
        let user = event.buyer.to_string();
        let service = event.service_id.bytes.to_string();
        let inner = TierEntitlement::from_u8(
            &tier_type,
            &event.inner.expires_at(),
            &event.inner.quota(),
            &event.inner.units(),
        )?;
        let ent = EntitlementUpdateEvent::new(ent_id, tier_id, tier_type, inner);
        let pubsub_event = PubSubEvent {
            user,
            service,
            action: PubSubAction::Refresh(ent),
        };

        let message = serde_json::to_string(&pubsub_event)?;
        let mut conn = self.redis.clone();
        let _: i64 = redis::cmd("PUBLISH")
            .arg(&channel)
            .arg(message)
            .query_async(&mut conn)
            .await?;

        info!(
            event = "ent.published",
            provider_id = %abbrev(&provider_id),
            user = %abbrev(&pubsub_event.user),
            service = %abbrev(&pubsub_event.service),
        );
        Ok(())
    }
}
