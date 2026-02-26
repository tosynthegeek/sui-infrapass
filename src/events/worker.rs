// worker.rs
use anyhow::Result;
use redis::Client as RedisClient;
use tokio::sync::mpsc::Receiver;
use tracing::{error, info};

use crate::events::types::{EventPayload, ProtocolEvent};

use crate::db::repository::Repository;
use crate::pubsub::publisher::PubSubPublisher;
use crate::utils::error::InfrapassError;

pub struct EventWorker {
    repo: Repository,
    pub publisher: PubSubPublisher,
    rx: Receiver<EventPayload>,
}

impl EventWorker {
    pub async fn new(
        repo: Repository,
        rx: Receiver<EventPayload>,
        redis_client: RedisClient,
    ) -> Result<Self, InfrapassError> {
        let publisher = PubSubPublisher::new(redis_client.clone()).await?;
        Ok(Self {
            repo,
            rx,
            publisher,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        info!("Event worker started");
        while let Some(payload) = self.rx.recv().await {
            if let Err(e) = self.handle_event(&payload).await {
                error!("Failed to handle payload {:?}: {}", payload, e);
            }
        }
        info!("Event worker stopped");
        Ok(())
    }

    pub async fn handle_event(&self, payload: &EventPayload) -> Result<()> {
        match &payload.event {
            ProtocolEvent::ProviderRegistered(e) => {
                let profile_id = e.profile_id.bytes.to_string();
                let provider_address = e.provider_address.to_string();

                self.repo
                    .store_event(
                        &payload.event,
                        payload.checkpoint,
                        payload.tx_digest.clone(),
                    )
                    .await?;

                info!(
                    provider_id = %profile_id,
                    provider_address = %provider_address,
                    "Provider registered"
                );

                Ok(())
            }

            ProtocolEvent::ServiceCreated(e) => {
                let service_id = e.service_id.bytes.to_string();
                let provider_id = e.provider.bytes.to_string();

                self.repo
                    .store_event(
                        &payload.event,
                        payload.checkpoint,
                        payload.tx_digest.clone(),
                    )
                    .await?;

                info!(
                    service_id = ?service_id,
                    provider_id = ?provider_id,
                    "Service created"
                );

                Ok(())
            }

            ProtocolEvent::ServiceUpdated(e) => {
                let metadata_uri = String::from_utf8_lossy(&e.metadata_uri);

                let service_id = e.service_id.bytes.to_string();
                let updated_service = self
                    .repo
                    .update_service_metadata(&service_id, &metadata_uri)
                    .await?;

                info!(
                    service_id = ?updated_service.service_id,
                    metadata_uri = %metadata_uri,
                    "Service updated"
                );

                Ok(())
            }

            ProtocolEvent::TierCreated(e) => {
                let name = String::from_utf8_lossy(&e.tier_name);

                self.repo
                    .store_event(
                        &payload.event,
                        payload.checkpoint,
                        payload.tx_digest.clone(),
                    )
                    .await?;

                info!(
                    tier_id = ?e.tier_id,
                    service_id = ?e.service_id,
                    name = %name,
                    price = e.price,
                    "Tier created"
                );

                Ok(())
            }

            ProtocolEvent::TierPriceUpdated(e) => {
                let tier_id = e.tier_id.bytes.to_string();
                let tier = self
                    .repo
                    .update_tier_price(&tier_id, e.new_price as i64)
                    .await?;
                info!(
                    tier_id = ?tier.tier_id,
                    new_price = e.new_price,
                    "Tier price updated"
                );

                Ok(())
            }

            ProtocolEvent::TierDeactivated(e) => {
                let tier_id = e.tier_id.bytes.to_string();
                let tier = self.repo.deactivate_tier(&tier_id).await?;
                info!(tier_id = ?tier.tier_id, "Tier deactivated");

                Ok(())
            }

            ProtocolEvent::TierReactivated(e) => {
                let tier_id = e.tier_id.bytes.to_string();
                let tier = self.repo.reactivate_tier(&tier_id).await?;
                info!(tier_id = ?tier.tier_id, "Tier reactivated");

                Ok(())
            }

            ProtocolEvent::EntitlementPurchased(e) => {
                let ent = self.repo.create_entitlement(&e).await?;
                info!(
                    entitlement_id = ?e.entitlement_id,
                    buyer = %e.buyer,
                    service_id = ?e.service_id,
                    tier_id = ?e.tier_id,
                    price_paid = e.price_paid,
                    "Entitlement purchased"
                );

                self.publisher.publish_refresh(&ent.provider_id, e).await?;

                Ok(())
            }
        }
    }
}
