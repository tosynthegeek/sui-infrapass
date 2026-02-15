module test_tokens::usdt;

use sui::coin::{Self, Coin, TreasuryCap};
use sui::coin_registry;

public struct USDT has drop {}

fun init(witness: USDT, ctx: &mut TxContext) {
    let (builder, treasury_cap) = coin_registry::new_currency_with_otw(
        witness,
        6,
        "TUSDT",
        "Test USDT",
        "Test USDT for development",
        "https://azure-purring-partridge-223.mypinata.cloud/ipfs/bafkreifnw6zaeypnjae5kzyxt3wdjaxzyhe4tha3j336npetmzunl53yim",
        ctx,
    );

    let metadata_cap = builder.finalize(ctx);
    transfer::public_transfer(treasury_cap, ctx.sender());
    transfer::public_transfer(metadata_cap, ctx.sender());
}

entry fun mint(
    treasury: &mut TreasuryCap<USDT>,
    amount: u64,
    recipient: address,
    ctx: &mut TxContext,
) {
    let coin = coin::mint(treasury, amount, ctx);
    transfer::public_transfer(coin, recipient);
}

entry fun burn(treasury: &mut TreasuryCap<USDT>, coin: Coin<USDT>) {
    coin::burn(treasury, coin);
}
