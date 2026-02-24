use serde::{Deserialize, Serialize};
use sui_types::{base_types::SuiAddress, id::ID};

use crate::types::types::TierConfigInput;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRegistered {
    pub provider_address: SuiAddress,
    pub profile_id: ID,
    pub metadata: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCreated {
    pub service_id: ID,
    pub provider: ID,
    pub service_type: Vec<u8>,
    pub metadata_uri: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceUpdated {
    pub service_id: ID,
    pub metadata_uri: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierCreated {
    pub tier_id: ID,
    pub service_id: ID,
    pub tier_name: Vec<u8>,
    pub price: u64,
    pub inner: TierConfigInput,
    pub coin_type: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierPriceUpdated {
    pub tier_id: ID,
    pub new_price: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDeactivated {
    pub tier_id: ID,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierReactivated {
    pub tier_id: ID,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierAddedToService {
    pub service_id: ID,
    pub tier_id: ID,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRemovedFromService {
    pub service_id: ID,
    pub tier_id: ID,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementPurchased {
    pub entitlement_id: ID,
    pub buyer: String,
    pub service_id: ID,
    pub tier_id: ID,
    pub price_paid: u64,
    pub timestamp: u64,
    pub inner: EntitlementConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConsumed {
    pub entitlement_id: ID,
    pub amount: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    pub event: ProtocolEvent,
    pub tx_digest: Option<String>,
    pub checkpoint: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolEvent {
    // Registry
    ProviderRegistered(ProviderRegistered),
    ServiceCreated(ServiceCreated),
    ServiceUpdated(ServiceUpdated),
    // Pricing
    TierCreated(TierCreated),
    TierPriceUpdated(TierPriceUpdated),
    TierDeactivated(TierDeactivated),
    TierReactivated(TierReactivated),
    // TierAddedToService(TierAddedToService),
    // TierRemovedFromService(TierRemovedFromService),
    // Payments
    EntitlementPurchased(EntitlementPurchased),
    // QuotaConsumed(QuotaConsumed),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EntitlementConfig {
    Subscription { expires_at: u64 },
    Quota { expires_at: u64, quota: u64 },
    UsageBased { units: u64 },
}

impl EntitlementConfig {
    pub fn expires_at(&self) -> Option<u64> {
        match self {
            EntitlementConfig::Subscription { expires_at } => Some(*expires_at),
            EntitlementConfig::Quota { expires_at, .. } => Some(*expires_at),
            EntitlementConfig::UsageBased { .. } => None,
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            EntitlementConfig::Subscription { .. } => "Subscription",
            EntitlementConfig::Quota { .. } => "Quota",
            EntitlementConfig::UsageBased { .. } => "UsageBased",
        }
    }
}
