CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TYPE tier_type AS ENUM ('subscription', 'quota', 'usage_based');

CREATE TABLE IF NOT EXISTS providers (
    profile_id TEXT PRIMARY KEY,
    provider_address TEXT NOT NULL,
    metadata_uri TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS services (
    service_id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL REFERENCES providers(profile_id),
    service_type TEXT NOT NULL,
    metadata_uri TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS pricing_tiers (
    tier_id TEXT PRIMARY KEY,
    service_id TEXT NOT NULL REFERENCES services(service_id),
    tier_name TEXT NOT NULL,
    price BIGINT NOT NULL,
    coin_type TEXT NOT NULL,
    tier_type tier_type NOT NULL,
    duration_ms BIGINT,
    quota_limit BIGINT,   
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT subscription_has_duration CHECK (
        tier_type != 'subscription' OR duration_ms IS NOT NULL
    ),
    CONSTRAINT quota_has_duration_and_limit CHECK (
        tier_type != 'quota' OR (duration_ms IS NOT NULL AND quota_limit IS NOT NULL)
    ),
    CONSTRAINT usage_based_no_extras CHECK (
        tier_type != 'usage_based' OR (duration_ms IS NULL AND quota_limit IS NULL)
    )
);

CREATE TABLE IF NOT EXISTS entitlements (
    entitlement_id TEXT PRIMARY KEY,
    buyer TEXT NOT NULL,
    service_id TEXT NOT NULL REFERENCES services(service_id),
    tier_id TEXT NOT NULL REFERENCES pricing_tiers(tier_id),
    price_paid BIGINT NOT NULL,
    expires_at TIMESTAMPTZ,
    quota BIGINT,
    units BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS blockchain_events (
    id BIGSERIAL,
    event_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    checkpoint_number BIGINT NOT NULL,
    transaction_digest TEXT,
    event_type TEXT NOT NULL,
    package_id TEXT NOT NULL,
    module TEXT NOT NULL,
    event_data JSONB NOT NULL,

    provider_id TEXT,
    service_id TEXT,
    tier_id TEXT,
    entitlement_id TEXT,
    
    PRIMARY KEY (event_time, id)
);

SELECT create_hypertable('blockchain_events', 'event_time', if_not_exists => TRUE);

CREATE TABLE IF NOT EXISTS api_requests (
    id BIGSERIAL,
    request_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    entitlement_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    status_code SMALLINT NOT NULL,
    response_time_ms INT NOT NULL,
    units_consumed INT NOT NULL DEFAULT 1,
    
    user_agent TEXT,
    ip_address INET,
    request_size_bytes INT,
    response_size_bytes INT,
    
    PRIMARY KEY (request_time, id)
);

SELECT create_hypertable('api_requests', 'request_time', if_not_exists => TRUE);