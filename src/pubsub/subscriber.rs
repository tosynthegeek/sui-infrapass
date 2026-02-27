use std::sync::Arc;

use futures::StreamExt;
use redis::aio::PubSub;
use tracing::{info, warn};

use crate::{
    sidecar::{error::ProxyError, proxy::ProxyState},
    pubsub::types::{PubSubAction, PubSubEvent},
    utils::{get_channel, logs_fmt::abbrev},
};

pub struct PubSubSubscriber {
    state: Arc<ProxyState>,
}

impl PubSubSubscriber {
    pub fn new(state: Arc<ProxyState>) -> Self {
        Self { state }
    }

    pub async fn run(&self) -> Result<(), ProxyError> {
        run_pubsub_listener(self.state.clone()).await
    }
}

pub async fn run_pubsub_listener(state: Arc<ProxyState>) -> Result<(), ProxyError> {
    let mut pubsub_conn: PubSub = state.redis_client.get_async_pubsub().await?;

    let channel = get_channel(state.cfg.provider_id.as_str());

    pubsub_conn.subscribe(&channel).await?;

    info!(channel = %abbrev(&channel), "Subscribed");

    let mut stream = pubsub_conn.on_message();

    while let Some(msg) = stream.next().await {
        let payload = msg.get_payload::<String>()?;
        let event: PubSubEvent = serde_json::from_str(&payload)?;

        match event.action {
            PubSubAction::Invalidate => {
                let _ = state
                    .invalidate_entitlement(&event.user, &event.service)
                    .await;

                info!(
                    user = %abbrev(&event.user),
                    service = %abbrev(&event.service),
                    "Cache invalidated"
                );
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
                    None => state.cfg.cache_ttl_ms / 1000,
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
                }

                let ent = match state.get_entitlement(&event.user, &event.service).await {
                    Some(ent) => ent,
                    None => {
                        warn!(user = %event.user, service = %event.service, "Failed to retrieve entitlement after refresh");
                        continue;
                    }
                };

                info!(
                    event = "cache.refresh",
                    user = %abbrev(&event.user),
                    service = %abbrev(&event.service),
                    entitlement_id = %abbrev(&ent.id),
                    "Cache refreshed"
                );
            }
        }
    }

    warn!("Pub/Sub stream ended unexpectedly");
    Ok(())
}
