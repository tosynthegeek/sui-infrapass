module test_tokens::test_tokens;

use sui::coin::{Self, Coin, TreasuryCap};
use sui::coin_registry;

public struct USDC has drop, store {}
public struct USDT has drop, store {}
public struct WAL has drop, store {}

fun init(ctx: &mut TxContext) {
    let (usdc_treasury, _usdc_metadata) = coin_registry::new_currency_with_otw<USDC>(
        USDC {},
        6,
        "TUSDC",
        "Test USDC",
        "Test USDC for development",
        "https://azure-purring-partridge-223.mypinata.cloud/ipfs/bafkreifnw6zaeypnjae5kzyxt3wdjaxzyhe4tha3j336npetmzunl53yim",
        ctx,
    );

    let (usdt_treasury, _usdt_metadata) = coin_registry::new_currency_with_otw<USDT>(
        USDT {},
        6,
        "TUSDT",
        "Test USDT",
        "Test USDT for development",
        "https://azure-purring-partridge-223.mypinata.cloud/ipfs/bafkreifnw6zaeypnjae5kzyxt3wdjaxzyhe4tha3j336npetmzunl53yim",
        ctx,
    );

    let (wal_treasury, _wal_metadata) = coin_registry::new_currency_with_otw<WAL>(
        WAL {},
        9,
        "TWAL",
        "Test WAL",
        "Test WAL for development",
        "https://azure-purring-partridge-223.mypinata.cloud/ipfs/bafkreifnw6zaeypnjae5kzyxt3wdjaxzyhe4tha3j336npetmzunl53yim",
        ctx,
    );
}

entry fun mint<T>(
    treasury: &mut TreasuryCap<T>,
    amount: u64,
    recipient: address,
    ctx: &mut TxContext,
) {
    let coin = coin::mint(treasury, amount, ctx);
    transfer::public_transfer(coin, recipient);
}

entry fun burn<T>(treasury: &mut TreasuryCap<T>, coin: Coin<T>) {
    coin::burn(treasury, coin);
}
