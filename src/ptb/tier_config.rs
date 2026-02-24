use anyhow::Result;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::Argument;

use crate::types::types::TierConfigInput;

pub fn build_tier_config_args(
    ptb: &mut ProgrammableTransactionBuilder,
    config: TierConfigInput,
) -> Result<(Argument, Argument, Argument)> {
    match config {
        TierConfigInput::Subscription { duration_ms } => Ok((
            ptb.pure(0u8)?,
            ptb.pure(Some(duration_ms))?,
            ptb.pure(None::<u64>)?,
        )),

        TierConfigInput::Quota {
            quota_limit,
            duration_ms,
        } => Ok((
            ptb.pure(1u8)?,
            ptb.pure(Some(duration_ms))?,
            ptb.pure(Some(quota_limit))?,
        )),

        TierConfigInput::UsageBased {} => Ok((
            ptb.pure(2u8)?,
            ptb.pure(None::<u64>)?,
            ptb.pure(None::<u64>)?,
        )),
    }
}
