use std::sync::Arc;

use futures::StreamExt;
use redis::aio::PubSub;
use tracing::{info, warn};

use crate::{
    adapters::{error::ProxyError, proxy::ProxyState},
    pubsub::types::{PubSubAction, PubSubEvent},
    utils::get_channel,
};

pub struct PubSubSubscriber {
    state: Arc<ProxyState>,
}

impl PubSubSubscriber {
    pub fn new(state: Arc<ProxyState>) -> Self {
        Self { state }
    }

    pub async fn run(&self) -> Result<(), ProxyError> {
        run_pubsub_listener(&self.state).await
    }
}

async fn run_pubsub_listener(state: &Arc<ProxyState>) -> Result<(), ProxyError> {
    let mut pubsub_conn: PubSub = state.redis_client.get_async_pubsub().await?;

    let channel = get_channel(state.cfg.provider_id.as_str());

    pubsub_conn.subscribe(&channel).await?;

    info!(%channel, "Subscribed to provider event channel");

    let mut stream = pubsub_conn.on_message();

    while let Some(msg) = stream.next().await {
        let payload = msg.get_payload::<String>()?;
        let event: PubSubEvent = serde_json::from_str(&payload)?;

        match event.action {
            PubSubAction::Invalidate => {
                info!(
                    user = %event.user,
                    service = %event.service,
                    "Received cache invalidation event"
                );
                let _ = state
                    .invalidate_entitlement(&event.user, &event.service)
                    .await;
            }
            PubSubAction::Refresh(tier) => {
                let _ = state
                    .invalidate_entitlement(&event.user, &event.service)
                    .await;

                let ent = tier.to_cached_entitlement()?;
                let ttl = match ent.expires_at {
                    Some(exp) => {
                        let now = chrono::Utc::now();
                        let remaining = (exp - now).num_seconds();
                        if remaining > 0 { remaining as u64 } else { 0 }
                    }
                    None => state.cfg.cache_ttl_ms / 1000, // default TTL if no expiration
                };
                let _ = state
                    .set_entitlement(&event.user, &event.service, &ent, ttl)
                    .await;

                if tier.tier_type() != 0 {
                    if let Some(q) = tier.inner().quota() {
                        let _ = state
                            .set_quota(&event.user, &event.service, q as i64, ttl)
                            .await;
                    }
                    // same for units
                }
            }
        }
    }

    warn!("Pub/Sub stream ended unexpectedly");
    Ok(())
}
