use anyhow::{Ok, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::{db::models::TierType, types::coin::CoinType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TierConfigInput {
    Subscription { duration_ms: u64 },
    Quota { quota_limit: u64, duration_ms: u64 },
    UsageBased {},
}

#[derive(Debug, Clone)]
pub struct TierInfo {
    pub coin_type: CoinType,
    pub price: u64,
    pub tier_type_string: String,
}

impl TierConfigInput {
    pub fn from_u8(tier: &u8, duration: &Option<u64>, quota: &Option<u64>) -> Result<Self> {
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
            2 => Ok(TierConfigInput::UsageBased {}),
            _ => Err(anyhow!("Invalid tier selected")),
        }
    }

    pub fn as_tier_type(&self) -> TierType {
        match self {
            TierConfigInput::Subscription { .. } => TierType::Subscription,
            TierConfigInput::Quota { .. } => TierType::Quota,
            TierConfigInput::UsageBased {} => TierType::UsageBased,
        }
    }

    pub fn duration(&self) -> Option<u64> {
        match self {
            TierConfigInput::Subscription { duration_ms } => Some(*duration_ms),
            TierConfigInput::Quota { duration_ms, .. } => Some(*duration_ms),
            TierConfigInput::UsageBased {} => None,
        }
    }

    pub fn quota(&self) -> Option<u64> {
        match self {
            TierConfigInput::Subscription { .. } => None,
            TierConfigInput::Quota { quota_limit, .. } => Some(*quota_limit),
            TierConfigInput::UsageBased {} => None,
        }
    }
}
