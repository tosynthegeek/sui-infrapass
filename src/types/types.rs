use anyhow::{Ok, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::{db::models::TierType, types::coin::CoinType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TierConfigInput {
    Subscription { expires_at: u64 },
    Quota { quota_limit: u64, expires_at: u64 },
    UsageBased {},
}

#[derive(Debug, Clone)]
pub struct TierInfo {
    pub coin_type: CoinType,
    pub price: u64,
    pub tier_type_string: String,
}

impl TierConfigInput {
    pub fn from_u8(tier: &u8, expires_at: &Option<u64>, quota: &Option<u64>) -> Result<Self> {
        match tier {
            0 => {
                let expires_at = expires_at.ok_or_else(|| anyhow!("invalid duration provided"))?;

                Ok(TierConfigInput::Subscription { expires_at })
            }
            1 => {
                let quota_limit = quota.ok_or_else(|| anyhow!("invalid quota limit provided"))?;
                let expires_at = expires_at.ok_or_else(|| anyhow!("invalid duration provided"))?;

                Ok(TierConfigInput::Quota {
                    quota_limit,
                    expires_at,
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

    pub fn as_tier_type_string(&self) -> String {
        match self {
            TierConfigInput::Subscription { .. } => "subscription".to_string(),
            TierConfigInput::Quota { .. } => "quota".to_string(),
            TierConfigInput::UsageBased {} => "usage_based".to_string(),
        }
    }

    pub fn duration(&self) -> Option<u64> {
        match self {
            TierConfigInput::Subscription { expires_at } => Some(*expires_at),
            TierConfigInput::Quota { expires_at, .. } => Some(*expires_at),
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
