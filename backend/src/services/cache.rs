use sqlx::PgPool;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value as JsonValue;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type CacheResult<T> = Result<T, CacheError>;

/// Database-backed caching service
/// Uses the cache_entries table and PostgreSQL functions
pub struct CacheService {
    pool: PgPool,
}

impl CacheService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get a value from cache, or compute and store it if not present
    pub async fn get_or_set<T, F, Fut>(
        &self,
        key: &str,
        ttl_seconds: i32,
        compute: F,
    ) -> CacheResult<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CacheResult<T>>,
    {
        // Try to get from cache first
        if let Some(cached) = self.get::<T>(key).await? {
            return Ok(cached);
        }

        // Compute the value
        let value = compute().await?;

        // Store in cache
        self.set(key, &value, ttl_seconds).await?;

        Ok(value)
    }

    /// Get a value from cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> CacheResult<Option<T>> {
        let result: Option<(JsonValue,)> = sqlx::query_as(
            r#"
            SELECT value FROM cache_entries
            WHERE key = $1 AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some((value,)) => {
                // Update hit count
                sqlx::query("UPDATE cache_entries SET hit_count = hit_count + 1 WHERE key = $1")
                    .bind(key)
                    .execute(&self.pool)
                    .await?;

                let parsed = serde_json::from_value(value)?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Set a value in cache
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_seconds: i32) -> CacheResult<()> {
        let json_value = serde_json::to_value(value)?;

        sqlx::query(
            r#"
            INSERT INTO cache_entries (key, value, expires_at)
            VALUES ($1, $2, NOW() + ($3 || ' seconds')::interval)
            ON CONFLICT (key) DO UPDATE
            SET value = $2, expires_at = NOW() + ($3 || ' seconds')::interval, updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(json_value)
        .bind(ttl_seconds.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete a specific cache entry
    pub async fn delete(&self, key: &str) -> CacheResult<bool> {
        let result = sqlx::query("DELETE FROM cache_entries WHERE key = $1")
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Invalidate cache entries matching a pattern (SQL LIKE pattern)
    pub async fn invalidate_pattern(&self, pattern: &str) -> CacheResult<u64> {
        let result = sqlx::query("DELETE FROM cache_entries WHERE key LIKE $1")
            .bind(pattern)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Clean up expired cache entries
    pub async fn cleanup_expired(&self) -> CacheResult<u64> {
        let result = sqlx::query("DELETE FROM cache_entries WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheResult<CacheStats> {
        let stats: (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_entries,
                COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at < NOW()) as expired_entries,
                COALESCE(SUM(hit_count), 0) as total_hits
            FROM cache_entries
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(CacheStats {
            total_entries: stats.0,
            expired_entries: stats.1,
            total_hits: stats.2,
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CacheStats {
    pub total_entries: i64,
    pub expired_entries: i64,
    pub total_hits: i64,
}

/// Cache key builders for common patterns
pub mod cache_keys {
    use uuid::Uuid;

    pub fn dashboard_stats() -> String {
        "dashboard:stats".to_string()
    }

    pub fn client(id: Uuid) -> String {
        format!("client:{}", id)
    }

    pub fn client_list(page: u32) -> String {
        format!("clients:list:page:{}", page)
    }

    pub fn ticket(id: Uuid) -> String {
        format!("ticket:{}", id)
    }

    pub fn ticket_list(client_id: Option<Uuid>, page: u32) -> String {
        match client_id {
            Some(cid) => format!("tickets:client:{}:page:{}", cid, page),
            None => format!("tickets:all:page:{}", page),
        }
    }

    pub fn user(id: Uuid) -> String {
        format!("user:{}", id)
    }

    pub fn sla_metrics(client_id: Option<Uuid>) -> String {
        match client_id {
            Some(cid) => format!("sla:metrics:client:{}", cid),
            None => "sla:metrics:all".to_string(),
        }
    }

    pub fn analytics_utilization(start: &str, end: &str) -> String {
        format!("analytics:utilization:{}:{}", start, end)
    }

    pub fn analytics_profitability(start: &str, end: &str) -> String {
        format!("analytics:profitability:{}:{}", start, end)
    }

    /// Pattern to invalidate all client-related caches
    pub fn client_pattern(id: Uuid) -> String {
        format!("client:{}%", id)
    }

    /// Pattern to invalidate all ticket-related caches
    pub fn ticket_pattern(id: Uuid) -> String {
        format!("ticket:{}%", id)
    }

    /// Pattern to invalidate all analytics caches
    pub fn analytics_pattern() -> String {
        "analytics:%".to_string()
    }
}

/// Default TTL values in seconds
pub mod ttl {
    pub const SHORT: i32 = 60;           // 1 minute
    pub const MEDIUM: i32 = 300;         // 5 minutes
    pub const LONG: i32 = 900;           // 15 minutes
    pub const DASHBOARD: i32 = 120;      // 2 minutes
    pub const ANALYTICS: i32 = 600;      // 10 minutes
    pub const STATIC: i32 = 3600;        // 1 hour
}
