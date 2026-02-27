use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{sidecar::cache::CachedEntitlement, utils::error::InfrapassError};

#[derive(Debug, Serialize, Deserialize)]
pub struct PubSubEvent {
    pub user: String,
    pub service: String,
    pub action: PubSubAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PubSubAction {
    Invalidate,
    Refresh(EntitlementUpdateEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntitlementUpdateEvent {
    ent_id: String,
    tier_id: String,
    tier_type: u8,
    inner: TierEntitlement,
}

impl EntitlementUpdateEvent {
    pub fn new(ent_id: String, tier_id: String, tier_type: u8, inner: TierEntitlement) -> Self {
        Self {
            ent_id,
            tier_id,
            tier_type,
            inner,
        }
    }

    pub fn tier_type(&self) -> u8 {
        self.tier_type
    }

    pub fn inner(&self) -> &TierEntitlement {
        &self.inner
    }

    pub fn to_cached_entitlement(&self) -> Result<CachedEntitlement, InfrapassError> {
        match self.tier_type {
            0 => Ok(CachedEntitlement {
                id: self.ent_id.clone(),
                tier: self.tier_id.clone(),
                quota: None,
                units: None,
                tier_type: self.tier_type,
                expires_at: self
                    .inner
                    .expires_at()
                    .map(|ts| {
                        DateTime::<Utc>::from_timestamp_millis(ts as i64).ok_or_else(|| {
                            InfrapassError::ValidationError("invalid timestamp".into())
                        })
                    })
                    .transpose()?,
                cached_at: Some(chrono::Utc::now()),
            }),
            1 => Ok(CachedEntitlement {
                id: self.ent_id.clone(),
                tier: self.tier_id.clone(),
                quota: self.inner.quota(),
                units: None,
                tier_type: self.tier_type,
                expires_at: self
                    .inner
                    .expires_at()
                    .map(|ts| {
                        DateTime::<Utc>::from_timestamp_millis(ts as i64).ok_or_else(|| {
                            InfrapassError::ValidationError("invalid timestamp".into())
                        })
                    })
                    .transpose()?,
                cached_at: Some(chrono::Utc::now()),
            }),
            2 => Ok(CachedEntitlement {
                id: self.ent_id.clone(),
                tier: self.tier_id.clone(),
                quota: None,
                units: self.inner.units(),
                tier_type: self.tier_type,
                expires_at: None,
                cached_at: Some(chrono::Utc::now()),
            }),
            _ => Err(InfrapassError::Other(format!("invalid tier type"))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TierEntitlement {
    Subscription { expires_at: u64 },
    Quota { quota_limit: u64, expires_at: u64 },
    UsageBased { units: u64 },
}

impl TierEntitlement {
    pub fn from_u8(
        tier: &u8,
        expires_at: &Option<u64>,
        quota: &Option<u64>,
        units: &Option<u64>,
    ) -> Result<Self, InfrapassError> {
        match tier {
            0 => {
                let expires_at = expires_at
                    .ok_or_else(|| InfrapassError::Other(format!("expires at not set")))?;

                Ok(TierEntitlement::Subscription { expires_at })
            }
            1 => {
                let quota_limit =
                    quota.ok_or_else(|| InfrapassError::Other(format!("quota limit not set")))?;
                let expires_at = expires_at
                    .ok_or_else(|| InfrapassError::Other(format!("expires at not set")))?;

                Ok(TierEntitlement::Quota {
                    quota_limit,
                    expires_at,
                })
            }
            2 => {
                let units = units.ok_or_else(|| InfrapassError::Other(format!("units not set")))?;
                Ok(TierEntitlement::UsageBased { units })
            }
            _ => Err(InfrapassError::Other(format!("invalid tier type"))),
        }
    }

    pub fn as_tier_type_string(&self) -> String {
        match self {
            TierEntitlement::Subscription { .. } => "subscription".to_string(),
            TierEntitlement::Quota { .. } => "quota".to_string(),
            TierEntitlement::UsageBased { .. } => "usage_based".to_string(),
        }
    }

    pub fn expires_at(&self) -> Option<u64> {
        match self {
            TierEntitlement::Subscription { expires_at } => Some(*expires_at),
            TierEntitlement::Quota { expires_at, .. } => Some(*expires_at),
            TierEntitlement::UsageBased { .. } => None,
        }
    }

    pub fn quota(&self) -> Option<u64> {
        match self {
            TierEntitlement::Subscription { .. } => None,
            TierEntitlement::Quota { quota_limit, .. } => Some(*quota_limit),
            TierEntitlement::UsageBased { .. } => None,
        }
    }

    pub fn units(&self) -> Option<u64> {
        match self {
            TierEntitlement::Subscription { .. } => None,
            TierEntitlement::Quota { .. } => None,
            TierEntitlement::UsageBased { units } => Some(*units),
        }
    }
}
