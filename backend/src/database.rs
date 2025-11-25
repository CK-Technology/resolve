use sqlx::{migrate::MigrateDatabase, postgres::PgPoolOptions, PgPool, Postgres};
use std::time::Duration;

/// Database pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Maximum time to wait for a connection
    pub acquire_timeout: Duration,
    /// Maximum idle time before a connection is closed
    pub idle_timeout: Duration,
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 5,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),      // 10 minutes
            max_lifetime: Duration::from_secs(1800),     // 30 minutes
        }
    }
}

impl PoolConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(max) = std::env::var("DB_MAX_CONNECTIONS") {
            if let Ok(n) = max.parse() {
                config.max_connections = n;
            }
        }

        if let Ok(min) = std::env::var("DB_MIN_CONNECTIONS") {
            if let Ok(n) = min.parse() {
                config.min_connections = n;
            }
        }

        if let Ok(timeout) = std::env::var("DB_ACQUIRE_TIMEOUT") {
            if let Ok(n) = timeout.parse() {
                config.acquire_timeout = Duration::from_secs(n);
            }
        }

        if let Ok(idle) = std::env::var("DB_IDLE_TIMEOUT") {
            if let Ok(n) = idle.parse() {
                config.idle_timeout = Duration::from_secs(n);
            }
        }

        if let Ok(lifetime) = std::env::var("DB_MAX_LIFETIME") {
            if let Ok(n) = lifetime.parse() {
                config.max_lifetime = Duration::from_secs(n);
            }
        }

        config
    }

    /// Create production-optimized config
    pub fn production() -> Self {
        Self {
            max_connections: 50,
            min_connections: 10,
            acquire_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300),      // 5 minutes
            max_lifetime: Duration::from_secs(3600),     // 1 hour
        }
    }

    /// Create development config (fewer connections)
    pub fn development() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(60),
            idle_timeout: Duration::from_secs(900),      // 15 minutes
            max_lifetime: Duration::from_secs(7200),     // 2 hours
        }
    }
}

/// Create a database connection pool with default configuration
pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    create_pool_with_config(database_url, PoolConfig::from_env()).await
}

/// Create a database connection pool with custom configuration
pub async fn create_pool_with_config(database_url: &str, config: PoolConfig) -> anyhow::Result<PgPool> {
    // Create database if it doesn't exist
    if !Postgres::database_exists(database_url).await? {
        Postgres::create_database(database_url).await?;
        tracing::info!("Database created successfully");
    }

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .test_before_acquire(true)
        .connect(database_url)
        .await?;

    tracing::info!(
        "Database pool created: max={}, min={}, acquire_timeout={}s",
        config.max_connections,
        config.min_connections,
        config.acquire_timeout.as_secs()
    );

    Ok(pool)
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("Database migrations completed");
    Ok(())
}

/// Check database health
pub async fn health_check(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .is_ok()
}

/// Get pool statistics
#[derive(Debug, serde::Serialize)]
pub struct PoolStats {
    pub size: u32,
    pub idle: u32,
    pub in_use: u32,
}

pub fn get_pool_stats(pool: &PgPool) -> PoolStats {
    PoolStats {
        size: pool.size(),
        idle: pool.num_idle() as u32,
        in_use: pool.size() - pool.num_idle() as u32,
    }
}