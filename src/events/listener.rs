use std::{sync::Arc, time::Duration};

use crate::{
    events::{
        metrics::EventMetrics,
        types::{ProtocolEvent, ProviderRegistered, ServiceCreated},
    },
    utils::constants::PACKAGE_ID,
};
use anyhow::Result;
use futures::StreamExt;
use prost_types::{FieldMask, Value as ProstValue, value::Kind};
use serde_json::Value as JsonValue;
use sui_grpc::{
    Client,
    proto::sui::rpc::v2::{
        Checkpoint, Event, SubscribeCheckpointsRequest,
        subscription_service_client::SubscriptionServiceClient,
    },
};
use sui_json_rpc_types::CheckpointId;
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::base_types::ObjectID;
use tokio::{
    sync::{RwLock, mpsc},
    time::Instant,
};
use tonic::transport::Channel;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct EventListener {
    pub sui_client: SuiClient,
    pub client: Client,
    pub package_id: String,
    /// Sends parsed events to whoever is listening
    pub event_tx: mpsc::Sender<ProtocolEvent>,
    metrics: Arc<RwLock<EventMetrics>>,
}

impl EventListener {
    pub async fn new(grpc_url: &str, event_tx: mpsc::Sender<ProtocolEvent>) -> Result<Self> {
        let client = Client::new(grpc_url.to_string())?;
        let sui_client = SuiClientBuilder::default()
            .build(grpc_url.to_string())
            .await?;

        Ok(Self {
            client,
            sui_client,
            package_id: PACKAGE_ID.to_string(),
            event_tx,
            metrics: Arc::new(RwLock::new(EventMetrics::default())),
        })
    }

    pub async fn run(mut self) -> Result<()> {
        info!(
            "Starting checkpoint subscription for package: {}",
            self.package_id
        );

        let metrics_clone = self.metrics.clone();
        tokio::spawn(async move {
            Self::health_monitor(metrics_clone).await;
        });

        loop {
            {
                let mut metrics = self.metrics.write().await;
                metrics.connection_healthy = false;
            }

            match self.subscribe_and_process().await {
                Ok(_) => {
                    warn!("Checkpoint stream ended normally");
                }
                Err(e) => {
                    error!("Checkpoint stream error: {}", e);
                }
            }

            warn!("Reconnecting in 5s...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    pub async fn subscribe_and_process(&mut self) -> Result<()> {
        info!("Connecting to: {}", self.client.uri());

        let tls_config = tonic::transport::ClientTlsConfig::new().with_enabled_roots();

        let channel = Channel::from_shared(self.client.uri().to_string())?
            .tls_config(tls_config)?
            .connect()
            .await?;

        let mut client = SubscriptionServiceClient::new(channel);

        let mut req_msg = SubscribeCheckpointsRequest::default();
        req_msg.read_mask = Some(FieldMask {
            paths: vec![
                "events".to_string(),
                "effects".to_string(),
                "transactions".to_string(),
            ],
        });
        let request = tonic::Request::new(req_msg);

        let response = client.subscribe_checkpoints(request).await?;
        let mut stream = response.into_inner();

        info!("Checkpoint stream connected");

        {
            let mut metrics = self.metrics.write().await;
            metrics.connection_healthy = true;
        }

        while let Some(result) = stream.next().await {
            match result {
                Ok(checkpoint_response) => {
                    if checkpoint_response.cursor.is_some() {
                        let mut metrics = self.metrics.write().await;
                        metrics.last_checkpoint_received = checkpoint_response.cursor;
                        metrics.last_checkpoint_received_at = Some(Instant::now());
                        metrics.total_checkpoints_processed += 1;
                    };
                    if let Some(checkpoint) = checkpoint_response.checkpoint {
                        self.process_checkpoint(&checkpoint, checkpoint_response.cursor)
                            .await;
                    }
                }
                Err(e) => {
                    error!("Checkpoint error: {}", e);
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    pub async fn process_checkpoint(
        &mut self,
        checkpoint: &Checkpoint,
        checkpoint_cursor: Option<u64>,
    ) {
        for tx in &checkpoint.transactions {
            if let Some(tx_events) = &tx.events {
                for event in tx_events.events() {
                    if let Some(event_package_id) = &event.package_id {
                        if event_package_id != self.package_id.as_str() {
                            continue;
                        }
                    }

                    match self.parse_event(event) {
                        Some(parsed) => {
                            {
                                let mut metrics = self.metrics.write().await;
                                metrics.last_checkpoint_with_event = checkpoint_cursor;
                                metrics.last_event_seen_at = Some(Instant::now());
                                metrics.total_events_processed += 1;
                            }

                            if self.event_tx.send(parsed).await.is_err() {
                                warn!("Event receiver dropped, shutting down");
                                return;
                            }
                        }
                        None => {
                            warn!(
                                "Failed to parse event of type {:?} in checkpoint {:?}",
                                event.event_type, checkpoint_cursor
                            );
                        }
                    }
                }
            } else {
                continue;
            }
        }
    }

    pub fn parse_event(&self, event: &Event) -> Option<ProtocolEvent> {
        let event_type = &event.event_type.as_ref()?;

        let parts: Vec<&str> = event_type.split("::").collect();
        if parts.len() != 3 {
            warn!("Invalid event type format: {}", event_type);
            return None;
        }

        let module = parts[1];
        let event_name = parts[2];
        let label = format!("{}::{}", module, event_name);

        let bcs_contents = event.contents.as_ref()?;
        let bcs_bytes = bcs_contents.value.as_ref()?;

        match label.as_str() {
            "registry::ProviderRegistered" => {
                let inner: ProviderRegistered = bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::ProviderRegistered(inner))
            }
            "registry::ServiceCreated" => {
                let inner: ServiceCreated = bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::ServiceCreated(inner))
            }
            "registry::ServiceUpdated" => {
                let inner: crate::events::types::ServiceUpdated =
                    bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::ServiceUpdated(inner))
            }
            "registry::TierAddedToService" => {
                let inner: crate::events::types::TierAddedToService =
                    bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::TierAddedToService(inner))
            }
            "registry::TierRemovedFromService" => {
                let inner: crate::events::types::TierRemovedFromService =
                    bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::TierRemovedFromService(inner))
            }
            "pricing::TierCreated" => {
                let inner: crate::events::types::TierCreated = bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::TierCreated(inner))
            }
            "pricing::TierPriceUpdated" => {
                let inner: crate::events::types::TierPriceUpdated =
                    bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::TierPriceUpdated(inner))
            }
            "pricing::TierDeactivated" => {
                let inner: crate::events::types::TierDeactivated =
                    bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::TierDeactivated(inner))
            }
            "pricing::TierReactivated" => {
                let inner: crate::events::types::TierReactivated =
                    bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::TierReactivated(inner))
            }
            "payments::EntitlementPurchased" => {
                let inner: crate::events::types::EntitlementPurchased =
                    bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::EntitlementPurchased(inner))
            }
            "payments::QuotaConsumed" => {
                let inner: crate::events::types::QuotaConsumed = bcs::from_bytes(bcs_bytes).ok()?;
                Some(ProtocolEvent::QuotaConsumed(inner))
            }
            _ => {
                warn!("Unhandled event type: {}", label);
                None
            }
        }
    }

    pub async fn process_rpc_checkpoint(
        &self,
        checkpoint_id: CheckpointId,
        tx_digest: &str,
    ) -> Result<()> {
        let checkpoint = self
            .sui_client
            .read_api()
            .get_checkpoint(checkpoint_id)
            .await?;

        let expected_package_id = ObjectID::from_hex_literal(&self.package_id)?;

        for tx in &checkpoint.transactions {
            if tx.base58_encode() != tx_digest {
                continue;
            }

            let full_tx = self
                .sui_client
                .read_api()
                .get_transaction_with_options(
                    *tx,
                    sui_json_rpc_types::SuiTransactionBlockResponseOptions::new()
                        .with_effects()
                        .with_events(),
                )
                .await?;

            if let Some(tx_events) = &full_tx.events {
                for event in &tx_events.data {
                    if event.package_id != expected_package_id {
                        continue;
                    }
                }
            } else {
                continue;
            }
        }

        Ok(())
    }

    async fn health_monitor(health: Arc<RwLock<EventMetrics>>) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            let metrics = health.read().await;
            let now = Instant::now();

            let checkpoint_status = if !metrics.connection_healthy {
                "disconnected".to_string()
            } else {
                match metrics.last_checkpoint_received_at {
                    Some(t) => {
                        let elapsed = now.duration_since(t).as_secs();
                        if elapsed < 60 {
                            format!("healthy ({:.0}s ago)", elapsed)
                        } else {
                            format!("stalled ({:.0}s ago)", elapsed)
                        }
                    }
                    None => "waiting".to_string(),
                }
            };

            let event_info = match metrics.last_event_seen_at {
                Some(t) => {
                    let elapsed = now.duration_since(t).as_secs();
                    format!("{:.0}s ago", elapsed)
                }
                None => "never".to_string(),
            };

            info!(
                target: "health",
                "Health | Connection: {} | Last CP: {} | Last Event: {} (cp #{:?}) | Totals: {} checkpoints, {} events",
                checkpoint_status,
                metrics.last_checkpoint_received
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                event_info,
                metrics.last_checkpoint_with_event,
                metrics.total_checkpoints_processed,
                metrics.total_events_processed
            );

            if metrics.connection_healthy {
                if let Some(last_time) = metrics.last_checkpoint_received_at {
                    let elapsed = now.duration_since(last_time).as_secs();
                    if elapsed > 120 {
                        error!("ALERT: No checkpoint received in {}s", elapsed);
                    }
                }
            }
        }
    }
}

pub fn prost_value_to_json(value: &ProstValue) -> JsonValue {
    match &value.kind {
        Some(Kind::NullValue(_)) | None => JsonValue::Null,
        Some(Kind::BoolValue(b)) => JsonValue::Bool(*b),
        Some(Kind::NumberValue(n)) => JsonValue::Number(
            serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0)),
        ),
        Some(Kind::StringValue(s)) => JsonValue::String(s.clone()),
        Some(Kind::ListValue(list)) => {
            JsonValue::Array(list.values.iter().map(prost_value_to_json).collect())
        }
        Some(Kind::StructValue(s)) => JsonValue::Object(
            s.fields
                .iter()
                .map(|(k, v)| (k.clone(), prost_value_to_json(v)))
                .collect(),
        ),
    }
}
