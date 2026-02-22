use tracing::{info, warn};

use crate::events::types::{EntitlementConfig, ProtocolEvent};

pub async fn handle_event(event: ProtocolEvent) {
    match event {
        ProtocolEvent::ProviderRegistered(e) => {
            info!(
                address = %e.provider_address,
                profile_id = ?e.profile_id,
                timestamp = e.timestamp,
                "Provider registered"
            );
        }

        ProtocolEvent::ServiceCreated(e) => {
            let service_type = String::from_utf8_lossy(&e.service_type);
            let metadata_uri = String::from_utf8_lossy(&e.metadata_uri);
            info!(
                service_id = ?e.service_id,
                service_type = %service_type,
                metadata_uri = %metadata_uri,
                "Service created"
            );
        }

        ProtocolEvent::ServiceUpdated(e) => {
            let metadata_uri = String::from_utf8_lossy(&e.metadata_uri);
            info!(
                service_id = ?e.service_id,
                metadata_uri = %metadata_uri,
                "Service updated"
            );
        }

        ProtocolEvent::TierAddedToService(e) => {
            info!(
                service_id = ?e.service_id,
                tier_id = ?e.tier_id,
                "Tier added to service"
            );
        }

        ProtocolEvent::TierRemovedFromService(e) => {
            info!(
                service_id = ?e.service_id,
                tier_id = ?e.tier_id,
                "Tier removed from service"
            );
        }

        ProtocolEvent::TierCreated(e) => {
            let name = String::from_utf8_lossy(&e.tier_name);
            info!(
                tier_id = ?e.tier_id,
                service_id = ?e.service_id,
                name = %name,
                price = e.price,
                "Tier created"
            );
        }

        ProtocolEvent::TierPriceUpdated(e) => {
            info!(
                tier_id = ?e.tier_id,
                new_price = e.new_price,
                "Tier price updated"
            );
        }

        ProtocolEvent::TierDeactivated(e) => {
            warn!(tier_id = ?e.tier_id, "Tier deactivated");
        }

        ProtocolEvent::TierReactivated(e) => {
            info!(tier_id = ?e.tier_id, "Tier reactivated");
        }

        ProtocolEvent::EntitlementPurchased(e) => match &e.inner {
            EntitlementConfig::Subscription { expires_at } => {
                info!(
                    entitlement_id = ?e.entitlement_id,
                    buyer = %e.buyer,
                    tier_id = ?e.tier_id,
                    price_paid = e.price_paid,
                    expires_at = expires_at,
                    "Entitlement purchased [Subscription]"
                );
            }
            EntitlementConfig::Quota { expires_at, quota } => {
                info!(
                    entitlement_id = ?e.entitlement_id,
                    buyer = %e.buyer,
                    tier_id = ?e.tier_id,
                    price_paid = e.price_paid,
                    expires_at = expires_at,
                    quota = quota,
                    "Entitlement purchased [Quota]"
                );
            }
            EntitlementConfig::UsageBased { units } => {
                info!(
                    entitlement_id = ?e.entitlement_id,
                    buyer = %e.buyer,
                    tier_id = ?e.tier_id,
                    price_paid = e.price_paid,
                    units = units,
                    "Entitlement purchased [UsageBased]"
                );
            }
        },

        ProtocolEvent::QuotaConsumed(e) => {
            info!(
                entitlement_id = ?e.entitlement_id,
                amount = e.amount,
                "Quota consumed"
            );
        }
    }
}
