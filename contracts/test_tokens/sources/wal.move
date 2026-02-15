module test_tokens::wal;

use sui::coin::{Self, Coin, TreasuryCap};
use sui::coin_registry;

public struct WAL has drop {}

fun init(witness: WAL, ctx: &mut TxContext) {
    let (builder, treasury_cap) = coin_registry::new_currency_with_otw(
        witness,
        9,
        "TWAL",
        "Test WAL",
        "Test WAL for development",
        "https://azure-purring-partridge-223.mypinata.cloud/ipfs/bafkreifnw6zaeypnjae5kzyxt3wdjaxzyhe4tha3j336npetmzunl53yim",
        ctx,
    );

    let metadata_cap = builder.finalize(ctx);
    transfer::public_transfer(treasury_cap, ctx.sender());
    transfer::public_transfer(metadata_cap, ctx.sender());
}

entry fun mint(
    treasury: &mut TreasuryCap<WAL>,
    amount: u64,
    recipient: address,
    ctx: &mut TxContext,
) {
    let coin = coin::mint(treasury, amount, ctx);
    transfer::public_transfer(coin, recipient);
}

entry fun burn(treasury: &mut TreasuryCap<WAL>, coin: Coin<WAL>) {
    coin::burn(treasury, coin);
}
