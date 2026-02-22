use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "tier_type", rename_all = "snake_case")]
pub enum TierType {
    Subscription,
    Quota,
    UsageBased,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Provider {
    pub profile_id: String,
    pub provider_address: String,
    pub metadata_uri: String,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Service {
    pub service_id: String,
    pub provider_id: String,
    pub service_type: String,
    pub metadata_uri: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PricingTier {
    pub tier_id: String,
    pub service_id: String,
    pub tier_name: String,
    pub price: i64,
    pub coin_type: String,
    pub tier_type: TierType,
    pub duration_ms: Option<i64>,
    pub quota_limit: Option<i64>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PricingTier {
    pub fn duration_days(&self) -> Option<f64> {
        self.duration_ms
            .map(|ms| ms as f64 / (1000.0 * 60.0 * 60.0 * 24.0))
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Entitlement {
    pub entitlement_id: String,
    pub buyer: String,
    pub service_id: String,
    pub tier_id: String,
    pub price_paid: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub quota: Option<i64>,
    pub units: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BlockchainEvent {
    pub id: i64,
    pub event_time: DateTime<Utc>,
    pub checkpoint_number: i64,
    pub transaction_digest: Option<String>,
    pub event_type: String,
    pub package_id: String,
    pub module: String,
    pub event_data: serde_json::Value,
    pub provider_id: Option<String>,
    pub service_id: Option<String>,
    pub tier_id: Option<String>,
    pub entitlement_id: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ApiRequest {
    pub id: i64,
    pub request_time: DateTime<Utc>,
    pub entitlement_id: String,
    pub service_id: String,
    pub endpoint: String,
    pub method: String,
    pub status_code: i16,
    pub response_time_ms: i32,
    pub units_consumed: i32,
    pub user_agent: Option<String>,
    pub ip_address: Option<std::net::IpAddr>,
    pub request_size_bytes: Option<i32>,
    pub response_size_bytes: Option<i32>,
}
