CREATE INDEX IF NOT EXISTS idx_tiers_service ON pricing_tiers (service_id);
CREATE INDEX IF NOT EXISTS idx_tiers_type ON pricing_tiers (tier_type);
CREATE INDEX IF NOT EXISTS idx_tiers_active ON pricing_tiers (is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_tiers_price ON pricing_tiers (price);

CREATE INDEX IF NOT EXISTS idx_tiers_quota_limit ON pricing_tiers (quota_limit) 
WHERE tier_type = 'quota';

CREATE INDEX IF NOT EXISTS idx_tiers_duration ON pricing_tiers (duration_ms) 
WHERE tier_type IN ('subscription', 'quota');

CREATE INDEX IF NOT EXISTS idx_events_checkpoint ON blockchain_events (checkpoint_number DESC);
CREATE INDEX IF NOT EXISTS idx_events_type ON blockchain_events (event_type, event_time DESC);
CREATE INDEX IF NOT EXISTS idx_events_provider ON blockchain_events (provider_id, event_time DESC) WHERE provider_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_service ON blockchain_events (service_id, event_time DESC) WHERE service_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_tier ON blockchain_events (tier_id, event_time DESC) WHERE tier_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_entitlement ON blockchain_events (entitlement_id, event_time DESC) WHERE entitlement_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_tx ON blockchain_events (transaction_digest) WHERE transaction_digest IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_data ON blockchain_events USING GIN (event_data);

CREATE INDEX IF NOT EXISTS idx_api_entitlement ON api_requests (entitlement_id, request_time DESC);
CREATE INDEX IF NOT EXISTS idx_api_service ON api_requests (service_id, request_time DESC);
CREATE INDEX IF NOT EXISTS idx_api_status ON api_requests (status_code, request_time DESC);

CREATE INDEX IF NOT EXISTS idx_services_provider ON services (provider_id);
CREATE INDEX IF NOT EXISTS idx_tiers_service ON pricing_tiers (service_id);
CREATE INDEX IF NOT EXISTS idx_entitlements_buyer ON entitlements (buyer);
CREATE INDEX IF NOT EXISTS idx_entitlements_service ON entitlements (service_id);