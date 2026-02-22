module infrapass::registry;

use std::string::{Self, String};
use sui::clock::{Self, Clock};
use sui::event;
use sui::table::{Self, Table};
use sui::vec_set::{Self, VecSet};

const ENotAuthorized: u64 = 1;
const EProviderNotRegistered: u64 = 2;
const EDuplicateProvider: u64 = 3;

public struct ServiceRegistry has key {
    id: UID,
    providers: Table<address, ID>,
    providers_by_id: Table<ID, address>,
    provider_count: u64,
    service_count: u64,
    admin: address,
}

public struct ProviderProfile has key, store {
    id: UID,
    provider_address: address,
    service_ids: VecSet<ID>,
    verified: bool,
    created_at: u64,
    metadata_uri: String,
}

public struct ProviderCap has key, store {
    id: UID,
    profile_id: ID,
}

public struct ServiceListing has key, store {
    id: UID,
    provider_profile_id: ID,
    service_type: String,
    metadata_uri: String,
    pricing_tier_ids: VecSet<ID>,
    verified: bool,
    active: bool,
    created_at: u64,
    updated_at: u64,
}

public struct AdminCap has key {
    id: UID,
}

public struct ProviderRegistered has copy, drop {
    provider_address: address,
    profile_id: ID,
    metadata: String,
    timestamp: u64,
}

public struct ServiceCreated has copy, drop {
    service_id: ID,
    provider: ID,
    service_type: String,
    metadata_uri: String,
    timestamp: u64,
}

public struct ServiceVerified has copy, drop {
    service_id: ID,
    timestamp: u64,
}

public struct ServiceUpdated has copy, drop {
    service_id: ID,
    metadata_uri: String,
    timestamp: u64,
}

public struct TierAddedToService has copy, drop {
    service_id: ID,
    tier_id: ID,
    timestamp: u64,
}

public struct TierRemovedFromService has copy, drop {
    service_id: ID,
    tier_id: ID,
    timestamp: u64,
}

public struct ProviderAddressUpdated has copy, drop {
    provider_address: address,
    old_address: address,
    profile_id: ID,
    timestamp: u64,
}

fun init(ctx: &mut TxContext) {
    let admin = tx_context::sender(ctx);

    let registry = ServiceRegistry {
        id: object::new(ctx),
        providers: table::new(ctx),
        providers_by_id: table::new(ctx),
        provider_count: 0,
        service_count: 0,
        admin,
    };
    transfer::share_object(registry);

    let admin_cap = AdminCap {
        id: object::new(ctx),
    };
    transfer::transfer(admin_cap, admin);
}

public fun register_provider(
    registry: &mut ServiceRegistry,
    metadata_uri: vector<u8>,
    clock: &Clock,
    ctx: &mut TxContext,
): (ProviderProfile, ProviderCap) {
    let sender = tx_context::sender(ctx);
    assert!(!table::contains(&registry.providers, sender), EDuplicateProvider);

    let timestamp = clock::timestamp_ms(clock);
    let profile_id = object::new(ctx);
    let profile_id_inner = object::uid_to_inner(&profile_id);

    let profile = ProviderProfile {
        id: profile_id,
        provider_address: sender,
        service_ids: vec_set::empty(),
        verified: false,
        created_at: timestamp,
        metadata_uri: string::utf8(metadata_uri),
    };

    let provider_cap = ProviderCap {
        id: object::new(ctx),
        profile_id: profile_id_inner,
    };

    table::add(&mut registry.providers, sender, profile_id_inner);
    table::add(&mut registry.providers_by_id, profile_id_inner, sender);
    registry.provider_count = registry.provider_count + 1;

    event::emit(ProviderRegistered {
        provider_address: sender,
        profile_id: profile_id_inner,
        metadata: string::utf8(metadata_uri),
        timestamp,
    });

    (profile, provider_cap)
}

public fun create_service(
    registry: &mut ServiceRegistry,
    provider_profile: &mut ProviderProfile,
    provider_cap: &ProviderCap,
    service_type: vector<u8>,
    metadata_uri: vector<u8>,
    clock: &Clock,
    ctx: &mut TxContext,
): ServiceListing {
    let sender = tx_context::sender(ctx);
    let sender_profile_id = table::borrow(&registry.providers, sender);
    assert!(provider_cap.profile_id == sender_profile_id, ENotAuthorized);
    assert!(provider_profile.provider_address == sender, ENotAuthorized);

    assert!(
        *sender_profile_id == object::uid_to_inner(&provider_profile.id),
        EProviderNotRegistered,
    );

    let timestamp = clock::timestamp_ms(clock);
    let service_id = object::new(ctx);
    let service_id_inner = object::uid_to_inner(&service_id);

    let service = ServiceListing {
        id: service_id,
        provider_profile_id: provider_cap.profile_id,
        service_type: string::utf8(service_type),
        metadata_uri: string::utf8(metadata_uri),
        pricing_tier_ids: vec_set::empty(),
        verified: false,
        active: true,
        created_at: timestamp,
        updated_at: timestamp,
    };

    vec_set::insert(&mut provider_profile.service_ids, service_id_inner);
    registry.service_count = registry.service_count + 1;

    event::emit(ServiceCreated {
        service_id: service_id_inner,
        provider: provider_cap.profile_id,
        service_type: service.service_type,
        metadata_uri: service.metadata_uri,
        timestamp,
    });

    service
}

public fun update_service_metadata(
    service: &mut ServiceListing,
    new_metadata_uri: vector<u8>,
    timestamp: u64,
) {
    service.metadata_uri = string::utf8(new_metadata_uri);
    service.updated_at = timestamp;

    event::emit(ServiceUpdated {
        service_id: object::uid_to_inner(&service.id),
        metadata_uri: service.metadata_uri,
        timestamp,
    });
}

public fun add_tier_id(service: &mut ServiceListing, tier_id: ID, timestamp: u64) {
    vec_set::insert(&mut service.pricing_tier_ids, tier_id);

    event::emit(TierAddedToService {
        service_id: object::uid_to_inner(&service.id),
        tier_id,
        timestamp,
    });
}

public fun remove_tier_id(service: &mut ServiceListing, tier_id: ID, timestamp: u64) {
    vec_set::remove(&mut service.pricing_tier_ids, &tier_id);

    event::emit(TierRemovedFromService {
        service_id: object::uid_to_inner(&service.id),
        tier_id,
        timestamp,
    });
}

public fun get_service_provider_id(service: &ServiceListing): ID {
    service.provider_profile_id
}

public fun get_service_provider_address(
    registry: &ServiceRegistry,
    service: &ServiceListing,
): address {
    let provider_id = service.provider_profile_id;
    *table::borrow(&registry.providers_by_id, provider_id)
}

public fun get_service_id(service: &ServiceListing): ID {
    object::uid_to_inner(&service.id)
}

public fun get_provider_profile_id(cap: &ProviderCap): ID {
    cap.profile_id
}

public fun check_provider_id_by_address(registry: &ServiceRegistry, addr: address): bool {
    table::contains(&registry.providers, addr)
}

public fun is_service_active(service: &ServiceListing): bool {
    service.active
}

public fun get_service_tier_ids(service: &ServiceListing): vector<ID> {
    vec_set::into_keys(service.pricing_tier_ids)
}

public fun get_address_id(addr: address, registry: &ServiceRegistry): ID {
    *table::borrow(&registry.providers, addr)
}

public fun verify_provider_cap(cap: &ProviderCap, expected_id: ID): bool {
    cap.profile_id == expected_id
}

public fun verify_sender_is_provider(registry: &ServiceRegistry, sender: address) {
    let sender_exists = check_provider_id_by_address(registry, sender);
    assert!(sender_exists, ENotAuthorized);
}

fun verify_service_internal(
    service: &mut ServiceListing,
    timestamp: u64,
    registry: &ServiceRegistry,
    ctx: &TxContext,
) {
    let sender = tx_context::sender(ctx);
    assert!(sender == registry.admin, ENotAuthorized);
    service.verified = true;
    service.updated_at = timestamp;

    event::emit(ServiceVerified {
        service_id: object::uid_to_inner(&service.id),
        timestamp,
    });
}

public fun verify_sender_owns_service(
    registry: &ServiceRegistry,
    service: &ServiceListing,
    sender: address,
) {
    let sender_profile_id = get_address_id(sender, registry);

    assert!(sender_profile_id == service.provider_profile_id, ENotAuthorized);
}

entry fun verify_service(
    service: &mut ServiceListing,
    clock: &Clock,
    registry: &ServiceRegistry,
    ctx: &TxContext,
) {
    verify_service_internal(service, clock::timestamp_ms(clock), registry, ctx);
}

entry fun register_provider_entry(
    registry: &mut ServiceRegistry,
    metadata_uri: vector<u8>,
    clock: &Clock,
    ctx: &mut TxContext,
) {
    let sender = tx_context::sender(ctx);

    let (profile, provider_cap) = register_provider(registry, metadata_uri, clock, ctx);

    transfer::transfer(profile, sender);
    transfer::transfer(provider_cap, sender);
}

entry fun create_service_entry(
    registry: &mut ServiceRegistry,
    provider_profile: &mut ProviderProfile,
    provider_cap: &ProviderCap,
    service_type: vector<u8>,
    metadata_uri: vector<u8>,
    clock: &Clock,
    ctx: &mut TxContext,
) {
    let service = create_service(
        registry,
        provider_profile,
        provider_cap,
        service_type,
        metadata_uri,
        clock,
        ctx,
    );

    let sender = tx_context::sender(ctx);
    transfer::transfer(service, sender);
}

entry fun update_service_metadata_entry(
    registry: &ServiceRegistry,
    service: &mut ServiceListing,
    new_metadata_uri: vector<u8>,
    clock: &Clock,
    ctx: &TxContext,
) {
    verify_sender_is_provider(registry, tx_context::sender(ctx));
    verify_sender_owns_service(registry, service, tx_context::sender(ctx));

    update_service_metadata(service, new_metadata_uri, clock::timestamp_ms(clock));
}

entry fun update_provider_address_entry(
    registry: &mut ServiceRegistry,
    provider_profile: &mut ProviderProfile,
    service: &ServiceListing,
    old_address: address,
    new_address: address,
    clock: &Clock,
    ctx: &TxContext,
) {
    verify_sender_is_provider(registry, tx_context::sender(ctx));
    verify_sender_owns_service(registry, service, tx_context::sender(ctx));
    let profile_id = table::borrow(&registry.providers, old_address);
    assert!(*profile_id == object::uid_to_inner(&provider_profile.id), ENotAuthorized);

    let profile_id = table::remove(&mut registry.providers, old_address);
    let _ = table::remove(&mut registry.providers_by_id, profile_id);
    table::add(&mut registry.providers, new_address, profile_id);
    table::add(&mut registry.providers_by_id, profile_id, new_address);
    provider_profile.provider_address = new_address;

    event::emit(ProviderAddressUpdated {
        provider_address: new_address,
        old_address,
        profile_id: profile_id,
        timestamp: clock::timestamp_ms(clock),
    });
}

entry fun set_service_active_entry(
    registry: &ServiceRegistry,
    service: &mut ServiceListing,
    clock: &Clock,
    ctx: &TxContext,
) {
    verify_sender_is_provider(registry, tx_context::sender(ctx));
    verify_sender_owns_service(registry, service, tx_context::sender(ctx));

    service.active = true;
    service.updated_at = clock::timestamp_ms(clock);

    event::emit(ServiceUpdated {
        service_id: object::uid_to_inner(&service.id),
        metadata_uri: service.metadata_uri,
        timestamp: service.updated_at,
    });
}

#[test_only]
public fun init_for_testing(ctx: &mut TxContext) {
    init(ctx);
}
