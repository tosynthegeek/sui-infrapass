use anyhow::Result;
use sui_sdk::SuiClient;
use sui_types::{
    TypeTag,
    base_types::{ObjectID, SuiAddress},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, Command as SuiCommand, ObjectArg},
};

use crate::types::coin::CoinType;

pub async fn find_coin_object(
    client: &SuiClient,
    owner: SuiAddress,
    coin_type: &TypeTag,
    required_amount: u64,
) -> Result<ObjectID> {
    let coins = client
        .coin_read_api()
        .get_coins(owner, Some(coin_type.to_string()), None, None)
        .await?;

    for coin in coins.data {
        if coin.balance >= required_amount {
            return Ok(coin.coin_object_id);
        }
    }

    Err(anyhow::anyhow!("Insufficient balance"))
}

pub async fn prepare_payment_coin(
    ptb: &mut ProgrammableTransactionBuilder,
    client: &SuiClient,
    sender: SuiAddress,
    coin_type: CoinType,
    exact_amount: u64,
) -> Result<Argument> {
    if coin_type.to_u8()? == 0 {
        let amount_arg = ptb.pure(exact_amount)?;
        return Ok(ptb.command(SuiCommand::SplitCoins(Argument::GasCoin, vec![amount_arg])));
    }

    let coins = client
        .coin_read_api()
        .get_coins(
            sender,
            Some(coin_type.to_type_tag()?.to_string()),
            None,
            None,
        )
        .await?;

    if coins.data.is_empty() {
        anyhow::bail!("No {} coins found in wallet", coin_type.name());
    }

    let total_balance: u64 = coins.data.iter().map(|c| c.balance).sum();
    if total_balance < exact_amount {
        anyhow::bail!(
            "Insufficient {} balance\nRequired: {}\nAvailable: {}",
            coin_type.name(),
            coin_type.format_amount(exact_amount),
            coin_type.format_amount(total_balance)
        );
    }

    if let Some(coin) = coins.data.iter().find(|c| c.balance >= exact_amount) {
        let coin_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(coin.object_ref()))?;

        if coin.balance == exact_amount {
            return Ok(coin_arg);
        } else {
            let amount_arg = ptb.pure(exact_amount)?;
            return Ok(ptb.command(SuiCommand::SplitCoins(coin_arg, vec![amount_arg])));
        }
    }

    println!(
        "Merging {} coin objects to create payment",
        coins.data.len()
    );

    let primary_coin = &coins.data[0];
    let primary_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(primary_coin.object_ref()))?;

    if coins.data.len() > 1 {
        let merge_args: Vec<Argument> = coins.data[1..]
            .iter()
            .map(|coin| ptb.obj(ObjectArg::ImmOrOwnedObject(coin.object_ref())))
            .collect::<Result<Vec<_>, _>>()?;

        ptb.command(SuiCommand::MergeCoins(primary_arg, merge_args));
    }

    let amount_arg = ptb.pure(exact_amount)?;

    Ok(ptb.command(SuiCommand::SplitCoins(primary_arg, vec![amount_arg])))
}

pub fn extract_coin_type_from_tier_type(tier_type: &str) -> Result<CoinType> {
    if tier_type.contains("0x2::sui::SUI>") {
        Ok(CoinType::SUI)
    } else if tier_type
        .contains("356a26eb9e012a68958082340d4c4116e7f55615cf27affcff209cf0ae544f59::wal::WAL>")
    {
        Ok(CoinType::WAL)
    } else if tier_type
        .contains("dba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC>")
    {
        Ok(CoinType::USDC)
    } else if tier_type
        .contains("375f70cf2ae4c00bf37117d0c85a2c71545e6ee05c4a5c7d282cd66a4504b068::usdt::USDT>")
    {
        Ok(CoinType::USDT)
    } else {
        Err(anyhow::anyhow!("Unknown coin type in tier: {}", tier_type))
    }
}

pub fn extract_price_from_content(
    content: &Option<sui_json_rpc_types::SuiParsedData>,
) -> Result<u64> {
    let content = content
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No content in tier"))?;

    if let sui_json_rpc_types::SuiParsedData::MoveObject(move_obj) = content {
        let fields = move_obj.fields.clone().to_json_value();
        if let Some(price_value) = fields.get("price") {
            if let Some(price_str) = price_value.as_str() {
                return Ok(price_str.parse()?);
            }
        }
    }

    Err(anyhow::anyhow!("Could not extract price from tier"))
}
