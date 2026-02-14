use crate::types::coin::CoinType;

pub enum TierConfigInput {
    Subscription { duration_ms: u64 },
    Quota { quota_limit: u64, duration_ms: u64 },
    UsageBased { price_per_unit: u64 },
}

#[derive(Debug, Clone)]
pub struct TierInfo {
    pub coin_type: CoinType,
    pub price: u64,
    pub tier_type_string: String,
}
