pub mod models;
pub mod repository;

use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    info!("Connecting to database: {}", mask_password(database_url));

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(database_url)
        .await?;

    info!("Database connection pool created");

    let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await?;

    info!("Database connection verified: {}", row.0);

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");

    sqlx::migrate!("src/db/migrations").run(pool).await?;

    info!("Database migrations completed");
    Ok(())
}

fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}

pub async fn health_check(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}
