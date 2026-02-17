use anyhow::Result;
use futures::StreamExt;
use sui_json_rpc_types::EventFilter;
use sui_sdk::{SuiClient, rpc_types::SuiEvent};
use sui_types::base_types::ObjectID;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{events::types::ProtocolEvent, utils::constants::PACKAGE_ID};

// All the Move event type strings we care about
const EVENT_TYPES: &[&str] = &[
    // Registry
    "registry::ProviderRegistered",
    "registry::ServiceCreated",
    "registry::ServiceUpdated",
    "registry::TierAddedToService",
    "registry::TierRemovedFromService",
    // Pricing
    "pricing::TierCreated",
    "pricing::TierPriceUpdated",
    "pricing::TierDeactivated",
    "pricing::TierReactivated",
    // Payments
    "payments::EntitlementPurchased",
    "payments::QuotaConsumed",
];

pub struct EventListener {
    client: SuiClient,
    pub package_id: ObjectID,
    /// Sends parsed events to whoever is listening
    event_tx: mpsc::Sender<ProtocolEvent>,
}

impl EventListener {
    pub fn new(client: SuiClient, event_tx: mpsc::Sender<ProtocolEvent>) -> Result<Self> {
        let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;
        Ok(Self {
            client,
            package_id,
            event_tx,
        })
    }

    /// Spawn one subscription per event type and fan-in to a single channel
    pub async fn run(self) -> Result<()> {
        info!("Starting event listener for package: {}", PACKAGE_ID);

        let mut handles = vec![];

        for event_type_suffix in EVENT_TYPES {
            let full_type = format!("{}::{}", PACKAGE_ID, event_type_suffix);
            let filter = EventFilter::MoveEventType(full_type.parse()?);

            let client = self.client.clone();
            let tx = self.event_tx.clone();
            let type_label = event_type_suffix.to_string();

            let handle = tokio::spawn(async move {
                subscribe_to_event(client, filter, type_label, tx).await;
            });

            handles.push(handle);
        }

        info!("Subscribed to {} event types", EVENT_TYPES.len());

        futures::future::join_all(handles).await;

        Ok(())
    }
}

async fn subscribe_to_event(
    client: SuiClient,
    filter: EventFilter,
    label: String,
    tx: mpsc::Sender<ProtocolEvent>,
) {
    loop {
        info!("Subscribing to [{}]", label);

        match client.event_api().subscribe_event(filter.clone()).await {
            Err(e) => {
                error!("[{}] Failed to subscribe: {}", label, e);
            }
            Ok(mut stream) => {
                info!("[{}] Stream connected", label);

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(event) => {
                            debug!("[{}] Received event: {:?}", label, event.id);
                            match parse_event(&event, &label) {
                                Some(parsed) => {
                                    if tx.send(parsed).await.is_err() {
                                        // Receiver dropped — server is shutting down
                                        return;
                                    }
                                }
                                None => {
                                    warn!("[{}] Could not parse event: {:?}", label, event);
                                }
                            }
                        }
                        Err(e) => {
                            error!("[{}] Stream error: {}", label, e);
                            break; // Reconnect
                        }
                    }
                }
            }
        }

        warn!("[{}] Stream dropped, reconnecting in 5s...", label);
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

fn parse_event(event: &SuiEvent, label: &str) -> Option<ProtocolEvent> {
    let data = &event.parsed_json;

    match label {
        "registry::ProviderRegistered" => {
            let inner: crate::events::types::ProviderRegistered =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::ProviderRegistered(inner))
        }
        "registry::ServiceCreated" => {
            let inner: crate::events::types::ServiceCreated =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::ServiceCreated(inner))
        }
        "registry::ServiceUpdated" => {
            let inner: crate::events::types::ServiceUpdated =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::ServiceUpdated(inner))
        }
        "registry::TierAddedToService" => {
            let inner: crate::events::types::TierAddedToService =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::TierAddedToService(inner))
        }
        "registry::TierRemovedFromService" => {
            let inner: crate::events::types::TierRemovedFromService =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::TierRemovedFromService(inner))
        }
        // ── Pricing ───────────────────────────────────────────────
        "pricing::TierCreated" => {
            let inner: crate::events::types::TierCreated =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::TierCreated(inner))
        }
        "pricing::TierPriceUpdated" => {
            let inner: crate::events::types::TierPriceUpdated =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::TierPriceUpdated(inner))
        }
        "pricing::TierDeactivated" => {
            let inner: crate::events::types::TierDeactivated =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::TierDeactivated(inner))
        }
        "pricing::TierReactivated" => {
            let inner: crate::events::types::TierReactivated =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::TierReactivated(inner))
        }
        // ── Payments ──────────────────────────────────────────────
        "payments::EntitlementPurchased" => {
            let inner: crate::events::types::EntitlementPurchased =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::EntitlementPurchased(inner))
        }
        "payments::QuotaConsumed" => {
            let inner: crate::events::types::QuotaConsumed =
                serde_json::from_value(data.clone()).ok()?;
            Some(ProtocolEvent::QuotaConsumed(inner))
        }
        _ => {
            warn!("Unhandled event type: {}", label);
            None
        }
    }
}
