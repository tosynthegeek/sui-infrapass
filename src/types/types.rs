use anyhow::{Ok, Result, anyhow};

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

impl TierConfigInput {
    pub fn from_u8(
        tier: &u8,
        duration: &Option<u64>,
        quota: &Option<u64>,
        unit_price: &Option<u64>,
    ) -> Result<Self> {
        match tier {
            0 => {
                let duration_ms = duration.ok_or_else(|| anyhow!("invalid duration provided"))?;

                Ok(TierConfigInput::Subscription { duration_ms })
            }
            1 => {
                let quota_limit = quota.ok_or_else(|| anyhow!("invalid quota limit provided"))?;
                let duration_ms = duration.ok_or_else(|| anyhow!("invalid duration provided"))?;

                Ok(TierConfigInput::Quota {
                    quota_limit,
                    duration_ms,
                })
            }
            2 => {
                let price_per_unit =
                    unit_price.ok_or_else(|| anyhow!("invalid unit price provided"))?;
                Ok(TierConfigInput::UsageBased { price_per_unit })
            }
            _ => Err(anyhow!("Invalid tier selected")),
        }
    }
}
