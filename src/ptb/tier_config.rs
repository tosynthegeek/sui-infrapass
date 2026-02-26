use anyhow::Result;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::Argument;

use crate::types::types::TierConfigInput;

pub fn build_tier_config_args(
    ptb: &mut ProgrammableTransactionBuilder,
    config: TierConfigInput,
) -> Result<(Argument, Argument, Argument)> {
    match config {
        TierConfigInput::Subscription { expires_at } => Ok((
            ptb.pure(0u8)?,
            ptb.pure(Some(expires_at))?,
            ptb.pure(None::<u64>)?,
        )),

        TierConfigInput::Quota {
            quota_limit,
            expires_at,
        } => Ok((
            ptb.pure(1u8)?,
            ptb.pure(Some(expires_at))?,
            ptb.pure(Some(quota_limit))?,
        )),

        TierConfigInput::UsageBased {} => Ok((
            ptb.pure(2u8)?,
            ptb.pure(None::<u64>)?,
            ptb.pure(None::<u64>)?,
        )),
    }
}
