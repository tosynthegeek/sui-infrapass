use anyhow::{Ok, Result};
use sui_types::base_types::SuiAddress;

use crate::utils::config::{default_wallet_config, load_wallet_context};

pub fn get_sender_address() -> Result<SuiAddress> {
    // TODO: Check from cache or memory where we store
    let default_path = default_wallet_config()?;

    let mut wallet = load_wallet_context(default_path)?;

    Ok(wallet.active_address()?)
}
