use std::sync::Arc;

use anyhow::Result;
use sqlx::PgPool;

use crate::{
    db::models::{BlockchainEvent, PricingTier, Provider, Service, TierType},
    events::types::ProtocolEvent,
};

pub struct Repository {
    pool: Arc<PgPool>,
}

impl Repository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn create_provider(
        &self,
        profile_id: String,
        provider_address: String,
        metadata: &str,
    ) -> Result<Provider> {
        let provider = sqlx::query_as::<_, Provider>(
            r#"
            INSERT INTO providers (profile_id, provider_address, metadata_uri)
            VALUES ($1, $2, $3)
            ON CONFLICT (profile_id) DO UPDATE
            SET provider_address = EXCLUDED.provider_address, updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(profile_id)
        .bind(provider_address)
        .bind(metadata)
        .fetch_one(&*self.pool)
        .await?;

        Ok(provider)
    }

    pub async fn get_provider(&self, profile_id: &str) -> Result<Option<Provider>> {
        let provider = sqlx::query_as!(
            Provider,
            "SELECT * FROM providers WHERE profile_id = $1",
            profile_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(provider)
    }

    pub async fn get_provider_by_address(&self, address: &str) -> Result<Option<Provider>> {
        let provider = sqlx::query_as!(
            Provider,
            "SELECT * FROM providers WHERE provider_address = $1",
            address
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(provider)
    }

    pub async fn list_providers(&self, limit: i64) -> Result<Vec<Provider>> {
        let providers = sqlx::query_as!(
            Provider,
            "SELECT * FROM providers WHERE is_active = true ORDER BY created_at DESC LIMIT $1",
            limit
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(providers)
    }

    pub async fn create_service(
        &self,
        service_id: &str,
        provider_id: &str,
        service_type: &str,
        metadata_uri: Option<String>,
    ) -> Result<Service> {
        let service = sqlx::query_as!(
            Service,
            r#"
            INSERT INTO services (service_id, provider_id, service_type, metadata_uri)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (service_id) DO UPDATE
            SET metadata_uri = EXCLUDED.metadata_uri, updated_at = NOW()
            RETURNING *
            "#,
            service_id,
            provider_id,
            service_type,
            metadata_uri
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(service)
    }

    pub async fn get_service(&self, service_id: &str) -> Result<Option<Service>> {
        let service = sqlx::query_as!(
            Service,
            "SELECT * FROM services WHERE service_id = $1",
            service_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(service)
    }

    pub async fn list_services_by_provider(&self, provider_id: &str) -> Result<Vec<Service>> {
        let services = sqlx::query_as!(
            Service,
            "SELECT * FROM services WHERE provider_id = $1 ORDER BY created_at DESC",
            provider_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(services)
    }

    pub async fn list_services(&self, limit: i64) -> Result<Vec<Service>> {
        let services = sqlx::query_as!(
            Service,
            "SELECT * FROM services WHERE is_active = true ORDER BY created_at DESC LIMIT $1",
            limit
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(services)
    }

    pub async fn create_tier(
        &self,
        tier_id: &str,
        service_id: &str,
        tier_name: &str,
        price: i64,
        coin_type: &str,
        tier_type: TierType,
        duration_ms: Option<i64>,
        quota_limit: Option<i64>,
    ) -> Result<PricingTier> {
        let tier = sqlx::query_as::<_, PricingTier>(
            r#"
            INSERT INTO pricing_tiers 
            (tier_id, service_id, tier_name, price, coin_type, tier_type, duration_ms, quota_limit)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (tier_id) DO UPDATE
            SET price = EXCLUDED.price, 
                duration_ms = EXCLUDED.duration_ms,
                quota_limit = EXCLUDED.quota_limit,
                updated_at = NOW()
            RETURNING 
                tier_id, service_id, tier_name, price, coin_type,
                tier_type as "tier_type!: TierType",
                duration_ms, quota_limit, is_active, created_at, updated_at
            "#,
        )
        .bind(tier_id)
        .bind(service_id)
        .bind(tier_name)
        .bind(price)
        .bind(coin_type)
        .bind(tier_type)
        .bind(duration_ms)
        .bind(quota_limit)
        .fetch_one(&*self.pool)
        .await?;

        Ok(tier)
    }

    pub async fn get_tier(&self, tier_id: &str) -> Result<Option<PricingTier>> {
        let tier = sqlx::query_as!(
            PricingTier,
            r#"
            SELECT 
                tier_id, service_id, tier_name, price, coin_type,
                tier_type as "tier_type!: TierType",
                duration_ms, quota_limit, is_active, created_at, updated_at
            FROM pricing_tiers 
            WHERE tier_id = $1
            "#,
            tier_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(tier)
    }

    pub async fn list_tiers_by_service(&self, service_id: &str) -> Result<Vec<PricingTier>> {
        let tiers = sqlx::query_as!(
            PricingTier,
            r#"
            SELECT 
                tier_id, service_id, tier_name, price, coin_type,
                tier_type as "tier_type!: TierType",
                duration_ms, quota_limit, is_active, created_at, updated_at
            FROM pricing_tiers 
            WHERE service_id = $1 AND is_active = true
            ORDER BY price ASC
            "#,
            service_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(tiers)
    }

    pub async fn store_event(
        &self,
        event: &ProtocolEvent,
        checkpoint: u64,
        tx_digest: Option<String>,
    ) -> Result<()> {
        match event {
            ProtocolEvent::ProviderRegistered(e) => {
                let prof_id = e.profile_id.bytes.to_string();
                sqlx::query!(
                    r#"
                    INSERT INTO blockchain_events 
                    (checkpoint_number, transaction_digest, event_type, package_id, module, event_data, provider_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                    checkpoint as i64,
                    tx_digest,
                    "ProviderRegistered",
                    crate::utils::constants::PACKAGE_ID,
                    "registry",
                    serde_json::to_value(e)?,
                    prof_id
                )
                .execute(&*self.pool)
                .await?;

                self.create_provider(prof_id, e.provider_address.to_string(), &e.metadata)
                    .await?;
            }

            ProtocolEvent::ServiceCreated(e) => {
                let service_type = String::from_utf8_lossy(&e.service_type).to_string();
                let metadata_uri = String::from_utf8_lossy(&e.metadata_uri).to_string();
                let prof_id = e.provider.bytes.to_string();
                let serv = e.service_id.bytes.to_string();

                sqlx::query!(
                    r#"
                    INSERT INTO blockchain_events 
                    (checkpoint_number, transaction_digest, event_type, package_id, module, event_data, provider_id, service_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    "#,
                    checkpoint as i64,
                    tx_digest,
                    "ServiceCreated",
                    crate::utils::constants::PACKAGE_ID,
                    "registry",
                    serde_json::to_value(e)?,
                    prof_id,
                    serv
                )
                .execute(&*self.pool)
                .await?;

                self.create_service(&serv, &prof_id, &service_type, Some(metadata_uri))
                    .await?;
            }

            ProtocolEvent::TierCreated(e) => {
                let tier_name = String::from_utf8_lossy(&e.tier_name).to_string();
                let tier_id = e.tier_id.bytes.to_string();
                let serv = e.service_id.bytes.to_string();
                let coin_type = &e.coin_type;

                sqlx::query!(
                    r#"
                    INSERT INTO blockchain_events 
                    (checkpoint_number, transaction_digest, event_type, package_id, module, event_data, service_id, tier_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    "#,
                    checkpoint as i64,
                    tx_digest,
                    "TierCreated",
                    crate::utils::constants::PACKAGE_ID,
                    "pricing",
                    serde_json::to_value(e)?,
                    serv,
                    tier_id
                )
                .execute(&*self.pool)
                .await?;

                self.create_tier(
                    &tier_id,
                    &serv,
                    &tier_name,
                    e.price as i64,
                    coin_type,
                    e.inner.as_tier_type(),
                    e.inner.duration().map(|d| d as i64),
                    e.inner.quota().map(|q| q as i64),
                )
                .await?;
            }

            _ => {
                sqlx::query!(
                    r#"
                    INSERT INTO blockchain_events 
                    (checkpoint_number, transaction_digest, event_type, package_id, module, event_data)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    checkpoint as i64,
                    tx_digest,
                    format!("{:?}", event),
                    crate::utils::constants::PACKAGE_ID,
                    "unknown",
                    serde_json::to_value(event)?
                )
                .execute(&*self.pool)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn get_recent_events(&self, limit: i64) -> Result<Vec<BlockchainEvent>> {
        let events = sqlx::query_as::<_, BlockchainEvent>(
            r#"SELECT * FROM blockchain_events ORDER BY event_time DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&*self.pool)
        .await?;

        Ok(events)
    }
}
