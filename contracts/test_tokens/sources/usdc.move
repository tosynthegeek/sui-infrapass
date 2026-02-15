module test_tokens::usdc;

use sui::coin::{Self, Coin, TreasuryCap};
use sui::coin_registry;

public struct USDC has drop {}

fun init(witness: USDC, ctx: &mut TxContext) {
    let (builder, treasury_cap) = coin_registry::new_currency_with_otw(
        witness,
        6,
        "TUSDC",
        "Test USDC",
        "Test USDC for development",
        "https://azure-purring-partridge-223.mypinata.cloud/ipfs/bafkreifnw6zaeypnjae5kzyxt3wdjaxzyhe4tha3j336npetmzunl53yim",
        ctx,
    );

    let metadata_cap = builder.finalize(ctx);
    transfer::public_transfer(treasury_cap, ctx.sender());
    transfer::public_transfer(metadata_cap, ctx.sender());
}

entry fun mint(
    treasury: &mut TreasuryCap<USDC>,
    amount: u64,
    recipient: address,
    ctx: &mut TxContext,
) {
    let coin = coin::mint(treasury, amount, ctx);
    transfer::public_transfer(coin, recipient);
}

entry fun burn(treasury: &mut TreasuryCap<USDC>, coin: Coin<USDC>) {
    coin::burn(treasury, coin);
}
