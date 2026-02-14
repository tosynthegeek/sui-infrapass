module infrapass::payments;

use infrapass::pricing::{Self, PricingTier};
use infrapass::registry::{Self, ServiceListing, ServiceRegistry};
use std::string::String;
use sui::bag::{Self, Bag};
use sui::clock::{Self, Clock};
use sui::coin::{Self, Coin};
use sui::event;

const EInsufficientPayment: u64 = 1;
const EServiceNotActive: u64 = 2;
const ETierNotInService: u64 = 3;
const ETierNotActive: u64 = 4;
const EInvalidBatchSize: u64 = 5;
const ENoExpiry: u64 = 6;
const EExpired: u64 = 7;
const EQuotaExceeded: u64 = 8;

public struct EntitlementStore has key {
    id: UID,
    entitlements: Bag,
}

public enum EntitlementConfig has copy, drop, store {
    Subscription {
        expires_at: u64,
    },
    Quota {
        expires_at: u64,
        quota: u64,
    },
    UsageBased {
        units: u64,
    },
}

public struct UsageRelayerCap has key {
    id: UID,
}

public struct Entitlement has key, store {
    id: UID,
    holder: address,
    service_id: ID,
    tier_id: ID,
    tier_name: String,
    purchased_at: u64,
    inner: EntitlementConfig,
}

public struct EntitlementPurchased has copy, drop {
    entitlement_id: ID,
    buyer: address,
    service_id: ID,
    tier_id: ID,
    price_paid: u64,
    timestamp: u64,
    inner: EntitlementConfig,
}

public struct QuotaConsumed has copy, drop {
    entitlement_id: ID,
    amount: u64,
    inner: EntitlementConfig,
    timestamp: u64,
}

fun init(ctx: &mut TxContext) {
    transfer::transfer(
        UsageRelayerCap {
            id: object::new(ctx),
        },
        tx_context::sender(ctx),
    );
    let store = EntitlementStore {
        id: object::new(ctx),
        entitlements: bag::new(ctx),
    };
    transfer::share_object(store);
}

entry fun purchase_entitlement<CoinType>(
    store: &mut EntitlementStore,
    service: &ServiceListing,
    registry: &ServiceRegistry,
    tier: &PricingTier<CoinType>,
    mut payment: Coin<CoinType>,
    clock: &Clock,
    ctx: &mut TxContext,
) {
    let buyer = tx_context::sender(ctx);
    let timestamp = clock::timestamp_ms(clock);
    let service_id = registry::get_service_id(service);

    assert!(registry::is_service_active(service), EServiceNotActive);

    assert!(pricing::get_tier_service_id(tier) == service_id, ETierNotInService);

    assert!(pricing::is_tier_active(tier), ETierNotActive);

    let tier_price = pricing::get_tier_price(tier);
    let payment_amount = coin::value(&payment);
    assert!(payment_amount >= tier_price, EInsufficientPayment);

    if (!pricing::is_usage_based(tier)) {
        if (payment_amount > tier_price) {
            let change = coin::split(&mut payment, payment_amount - tier_price, ctx);
            transfer::public_transfer(change, buyer);
        };
    };

    let (expires_at, quota_limit) = pricing::calculate_entitlement_details(
        tier,
        timestamp,
        payment_amount,
    );

    let entitlement_id = object::new(ctx);

    let ent_config = get_entitlement_config(expires_at, quota_limit, tier);

    let entitlement = Entitlement {
        id: entitlement_id,
        holder: buyer,
        service_id,
        tier_id: pricing::get_tier_id(tier),
        tier_name: pricing::get_tier_name(tier),
        purchased_at: timestamp,
        inner: ent_config,
    };

    let entitlement_id = object::uid_to_inner(&entitlement.id);

    event::emit(EntitlementPurchased {
        entitlement_id,
        buyer,
        service_id,
        tier_id: entitlement.tier_id,
        price_paid: payment_amount,
        timestamp,
        inner: ent_config,
    });

    bag::add(&mut store.entitlements, entitlement_id, entitlement);
    let provider = registry::get_service_provider_address(registry, service);

    transfer::public_transfer(payment, provider);

    // Optional: auto-transfer to buyer or keep in store for them to claim later
    // transfer::transfer(entitlement, buyer);
}

/// Batch-settle usage by providing entitlement object IDs + the actual mutable objects.
/// Caller must own/pass all entitlements being settled.
entry fun settle_usage_batch(
    _cap: &UsageRelayerCap,
    store: &mut EntitlementStore,
    entitlement_ids: vector<ID>,
    consumptions: vector<u64>,
    clock: &Clock,
    _ctx: &mut TxContext,
) {
    assert!(vector::length(&entitlement_ids) == vector::length(&consumptions), EInvalidBatchSize);
    // assert!(tx_context::sender(ctx) == trusted_relayer_address, ENotAuthorized);

    let timestamp = clock::timestamp_ms(clock);
    let mut i = 0;
    let len = vector::length(&entitlement_ids);

    while (i < len) {
        let ent_id = *vector::borrow(&entitlement_ids, i);

        let amount = *vector::borrow(&consumptions, i);
        let ent: &mut Entitlement = bag::borrow_mut(&mut store.entitlements, ent_id);

        if (has_expiry(ent)) { assert!(timestamp < get_expiry(ent), EExpired); };

        let mut should_emit = false;
        match (&mut ent.inner) {
            EntitlementConfig::Quota { .., quota } => {
                assert!(*quota >= amount, EQuotaExceeded);
                *quota = *quota - amount;
                should_emit = true
            },
            EntitlementConfig::UsageBased { units } => {
                assert!(*units >= amount, EQuotaExceeded);
                *units = *units - amount;
                should_emit = true
            },
            _ => {},
        };
        if (should_emit) {
            event::emit(QuotaConsumed {
                entitlement_id: ent_id,
                amount,
                inner: ent.inner,
                timestamp,
            });
        };

        // transfer::transfer(ent, ent.holder);

        i = i + 1;
    };
}

/// Returns whether the entitlement **appears valid based on the last on-chain settlement**.
/// This may be stale if off-chain usage has occurred but not yet settled.
public fun is_entitlement_appears_valid(ent: &Entitlement, clock: &Clock): bool {
    let now = clock::timestamp_ms(clock);

    match (&ent.inner) {
        EntitlementConfig::Subscription { expires_at } => now < *expires_at,
        EntitlementConfig::Quota { expires_at, quota } => {
            if (now >= *expires_at) {
                false
            } else {
                *quota > 0
            }
        },
        EntitlementConfig::UsageBased { units } => {
            *units > 0
        },
    }
}

public fun get_entitlement_config<CoinType>(
    expires_at: Option<u64>,
    remaining_quota: Option<u64>,
    tier: &PricingTier<CoinType>,
): EntitlementConfig { if (pricing::is_subscription(tier)) {
        assert!(option::is_some(&expires_at), 0);
        let expires = *option::borrow(&expires_at);
        EntitlementConfig::Subscription {
            expires_at: expires,
        }
    } else if (pricing::is_quota(tier)) {
        assert!(option::is_some(&expires_at), 0);
        assert!(option::is_some(&remaining_quota), 0);
        let expires = *option::borrow(&expires_at);
        let quota = *option::borrow(&remaining_quota);
        EntitlementConfig::Quota {
            expires_at: expires,
            quota,
        }
    } else {
        assert!(option::is_some(&remaining_quota), 0);
        let units = *option::borrow(&remaining_quota);
        EntitlementConfig::UsageBased {
            units,
        }
    } }

fun has_expiry(ent: &Entitlement): bool {
    match (&ent.inner) {
        EntitlementConfig::Subscription { .. } => true,
        EntitlementConfig::Quota { .. } => true,
        EntitlementConfig::UsageBased { .. } => false,
    }
}

fun get_expiry(ent: &Entitlement): u64 {
    match (&ent.inner) {
        EntitlementConfig::Subscription { expires_at } => *expires_at,
        EntitlementConfig::Quota { expires_at, .. } => *expires_at,
        _ => abort ENoExpiry,
    }
}

public fun get_remaining(config: &EntitlementConfig): Option<u64> {
    match (config) {
        EntitlementConfig::Quota { .., quota } => option::some(*quota),
        EntitlementConfig::UsageBased { units } => option::some(*units),
        _ => option::none(),
    }
}
