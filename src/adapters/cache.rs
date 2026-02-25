use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntitlement {
    pub id: String,
    pub tier: String,
    pub quota: Option<u64>,
    pub units: Option<u64>,
    pub tier_type: u8,
    pub expires_at: Option<DateTime<Utc>>,
    pub cached_at: Option<DateTime<Utc>>,
}

impl CachedEntitlement {
    pub fn allowed(&self) -> bool {
        match self.tier_type {
            0 => self.expires_at.map_or(false, |exp| exp > Utc::now()),
            2 => {
                self.quota.map_or(false, |q| q > 0)
                    && self.expires_at.map_or(false, |exp| exp > Utc::now())
            }
            3 => self.units.map_or(false, |u| u > 0),
            _ => false,
        }
    }

    pub fn units(&self) -> Option<u64> {
        self.units
    }

    pub fn quota(&self) -> Option<u64> {
        self.quota
    }
}
