use sui_types::id::ID;

#[derive(Debug, Clone)]
pub struct UsageSettlement {
    pub entitlement_id: ID,
    pub amount: u64,
}

impl UsageSettlement {
    pub fn new(entitlement_id: ID, amount: u64) -> Self {
        Self {
            entitlement_id,
            amount,
        }
    }
}
