module infrapass::pricing;

use infrapass::registry::{Self, ServiceListing, ProviderCap, ServiceRegistry};
use std::string::{Self, String};
use sui::clock::{Self, Clock};
use sui::event;

const ENotAuthorized: u64 = 1;
const EServiceNotActive: u64 = 2;

const TIER_SUBSCRIPTION: u8 = 0;
const TIER_QUOTA: u8 = 1;
const TIER_USAGE_BASED: u8 = 2;

public enum TierConfig has copy, drop, store {
    /// Unlimited requests for a fixed duration
    Subscription {
        duration_ms: u64,
    },
    /// Limited requests within a fixed duration
    Quota {
        quota_limit: u64,
        duration_ms: u64,
    },
    /// Pay per request (deposit-based or postpaid)
    UsageBased {
        price_per_unit: u64,
    },
}

public struct PricingTier<phantom CoinType> has key, store {
    id: UID,
    service_id: ID,
    provider: ID,
    tier_name: String,
    price: u64,
    /// Configuration for different tier types (e.g., subscription, quota, usage-based)
    inner: TierConfig,
    active: bool,
    created_at: u64,
}

public struct TierCreated has copy, drop {
    tier_id: ID,
    service_id: ID,
    tier_name: String,
    price: u64,
    tier_type: u8,
    timestamp: u64,
}

public struct TierPriceUpdated has copy, drop {
    tier_id: ID,
    new_price: u64,
    timestamp: u64,
}

public struct TierDeactivated has copy, drop {
    tier_id: ID,
    timestamp: u64,
}

public struct TierReactivated has copy, drop {
    tier_id: ID,
    timestamp: u64,
}

public fun create_pricing_tier<CoinType>(
    service: &mut ServiceListing,
    provider_cap: &ProviderCap,
    service_registry: &ServiceRegistry,
    tier_name: vector<u8>,
    price: u64,
    inner: TierConfig,
    clock: &Clock,
    ctx: &mut TxContext,
): PricingTier<CoinType> {
    let provider = verify_sender(provider_cap, service, tx_context::sender(ctx), service_registry);

    assert!(registry::is_service_active(service), EServiceNotActive);

    let timestamp = clock::timestamp_ms(clock);
    let service_id = registry::get_service_id(service);

    let tier_id = object::new(ctx);
    let tier_id_inner = object::uid_to_inner(&tier_id);

    let tier_type = match (&inner) {
        TierConfig::Subscription { .. } => TIER_SUBSCRIPTION,
        TierConfig::Quota { .. } => TIER_QUOTA,
        TierConfig::UsageBased { .. } => TIER_USAGE_BASED,
    };

    let tier: PricingTier<CoinType> = PricingTier {
        id: tier_id,
        service_id,
        provider,
        tier_name: string::utf8(tier_name),
        price,
        inner,
        active: true,
        created_at: timestamp,
    };

    registry::add_tier_id(service, tier_id_inner, clock::timestamp_ms(clock));

    event::emit(TierCreated {
        tier_id: tier_id_inner,
        service_id,
        tier_name: tier.tier_name,
        price,
        tier_type,
        timestamp,
    });

    tier
}

entry fun create_pricing_tier_entry<CoinType>(
    service: &mut ServiceListing,
    provider_cap: &ProviderCap,
    registry: &ServiceRegistry,
    tier_name: vector<u8>,
    price: u64,
    tier_type: u8,
    duration_ms: Option<u64>,
    quota_limit: Option<u64>,
    unit_price: Option<u64>,
    clock: &Clock,
    ctx: &mut TxContext,
) {
    let inner = match (tier_type) {
        0 => TierConfig::Subscription {
            duration_ms: *option::borrow(&duration_ms),
        },
        1 => TierConfig::Quota {
            quota_limit: *option::borrow(&quota_limit),
            duration_ms: *option::borrow(&duration_ms),
        },
        2 => TierConfig::UsageBased {
            price_per_unit: *option::borrow(&unit_price),
        },
        _ => abort ENotAuthorized,
    };
    let tier = create_pricing_tier<CoinType>(
        service,
        provider_cap,
        registry,
        tier_name,
        price,
        inner,
        clock,
        ctx,
    );
    let sender = tx_context::sender(ctx);
    transfer::transfer(tier, sender);
}

entry fun add_tier_to_service(
    service: &mut ServiceListing,
    service_registry: &ServiceRegistry,
    provider_cap: &ProviderCap,
    tier_id: ID,
    clock: &Clock,
    ctx: &TxContext,
) {
    let _ = verify_sender(provider_cap, service, tx_context::sender(ctx), service_registry);

    registry::add_tier_id(service, tier_id, clock::timestamp_ms(clock));
}

entry fun update_tier_price<CoinType>(
    tier: &mut PricingTier<CoinType>,
    provider_cap: &ProviderCap,
    new_price: u64,
    clock: &Clock,
    _ctx: &mut TxContext,
) {
    assert!(tier.provider == registry::get_provider_profile_id(provider_cap), ENotAuthorized);
    tier.price = new_price;

    event::emit(TierPriceUpdated {
        tier_id: object::uid_to_inner(&tier.id),
        new_price,
        timestamp: clock::timestamp_ms(clock),
    });
}

entry fun deactivate_tier<CoinType>(
    tier: &mut PricingTier<CoinType>,
    provider_cap: &ProviderCap,
    clock: &Clock,
    _ctx: &mut TxContext,
) {
    assert!(tier.provider == registry::get_provider_profile_id(provider_cap), ENotAuthorized);
    tier.active = false;

    event::emit(TierDeactivated {
        tier_id: object::uid_to_inner(&tier.id),
        timestamp: clock::timestamp_ms(clock),
    });
}

entry fun reactivate_tier<CoinType>(
    tier: &mut PricingTier<CoinType>,
    provider_cap: &ProviderCap,
    clock: &Clock,
    _ctx: &mut TxContext,
) {
    assert!(tier.provider == registry::get_provider_profile_id(provider_cap), ENotAuthorized);
    tier.active = true;

    event::emit(TierReactivated {
        tier_id: object::uid_to_inner(&tier.id),
        timestamp: clock::timestamp_ms(clock),
    });
}

entry fun remove_tier_from_service(
    service: &mut ServiceListing,
    provider_cap: &ProviderCap,
    registry: &ServiceRegistry,
    tier_id: ID,
    clock: &Clock,
    ctx: &TxContext,
) {
    let _ = verify_sender(provider_cap, service, tx_context::sender(ctx), registry);

    registry::remove_tier_id(service, tier_id, clock::timestamp_ms(clock));
}

public fun get_tier_id<CoinType>(tier: &PricingTier<CoinType>): ID {
    object::uid_to_inner(&tier.id)
}

public fun get_tier_provider<CoinType>(tier: &PricingTier<CoinType>): ID {
    tier.provider
}

public fun get_tier_price<CoinType>(tier: &PricingTier<CoinType>): u64 {
    tier.price
}

public fun get_tier_service_id<CoinType>(tier: &PricingTier<CoinType>): ID {
    tier.service_id
}

public fun get_tier_details<CoinType>(tier: &PricingTier<CoinType>): (String, u64, TierConfig) {
    (tier.tier_name, tier.price, tier.inner)
}

public fun get_tier_config<CoinType>(tier: &PricingTier<CoinType>): &TierConfig {
    &tier.inner
}

public fun get_tier_name<CoinType>(tier: &PricingTier<CoinType>): String {
    tier.tier_name
}

public fun is_subscription<CoinType>(tier: &PricingTier<CoinType>): bool {
    match (&tier.inner) {
        TierConfig::Subscription { .. } => true,
        _ => false,
    }
}

public fun is_quota<CoinType>(tier: &PricingTier<CoinType>): bool {
    match (&tier.inner) {
        TierConfig::Quota { .. } => true,
        _ => false,
    }
}

public fun is_usage_based<CoinType>(tier: &PricingTier<CoinType>): bool {
    match (&tier.inner) {
        TierConfig::UsageBased { .. } => true,
        _ => false,
    }
}

public fun get_subscription_duration<CoinType>(tier: &PricingTier<CoinType>): Option<u64> {
    match (&tier.inner) {
        TierConfig::Subscription { duration_ms } => option::some(*duration_ms),
        _ => option::none(),
    }
}

public fun is_tier_active<CoinType>(tier: &PricingTier<CoinType>): bool {
    tier.active
}

/// Calculate entitlement details based on tier config
/// Returns: (expires_at, remaining_quota)
public fun calculate_entitlement_details<CoinType>(
    tier: &PricingTier<CoinType>,
    current_time: u64,
    amount_paid: u64,
): (Option<u64>, Option<u64>) {
    match (&tier.inner) {
        TierConfig::Subscription { duration_ms } => {
            let expires_at = current_time + *duration_ms;
            (option::some(expires_at), option::none())
        },
        TierConfig::Quota { quota_limit, duration_ms } => {
            let expires_at = current_time + *duration_ms;
            (option::some(expires_at), option::some(*quota_limit))
        },
        TierConfig::UsageBased { price_per_unit } => {
            let requests_available = amount_paid / *price_per_unit;

            (option::none(), option::some(requests_available))
        },
    }
}

/// Verify that the sender is authorized to manage the service (i.e., is the provider)
fun verify_sender(
    provider_cap: &ProviderCap,
    service: &ServiceListing,
    sender: address,
    registry: &ServiceRegistry,
): ID {
    let sender_profile_id = registry::get_address_id(sender, registry);

    assert!(sender_profile_id == registry::get_service_provider_id(service), ENotAuthorized);
    assert!(
        registry::verify_provider_cap(
            provider_cap,
            registry::get_provider_profile_id(provider_cap),
        ),
        ENotAuthorized,
    );

    sender_profile_id
}
