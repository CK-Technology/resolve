use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use std::time::Instant;

#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type MetricsResult<T> = Result<T, MetricsError>;

/// Metrics collection service for observability
pub struct MetricsService {
    pool: PgPool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::Gauge => "gauge",
            Self::Histogram => "histogram",
        }
    }
}

impl MetricsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Record a metric value
    pub async fn record(
        &self,
        name: &str,
        metric_type: MetricType,
        value: f64,
        labels: Option<JsonValue>,
    ) -> MetricsResult<()> {
        let labels = labels.unwrap_or(JsonValue::Object(serde_json::Map::new()));

        sqlx::query(
            r#"
            SELECT record_metric($1, $2, $3, $4)
            "#,
        )
        .bind(name)
        .bind(metric_type.as_str())
        .bind(value)
        .bind(labels)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Increment a counter metric
    pub async fn increment(&self, name: &str, labels: Option<JsonValue>) -> MetricsResult<()> {
        self.record(name, MetricType::Counter, 1.0, labels).await
    }

    /// Increment a counter by a specific amount
    pub async fn increment_by(
        &self,
        name: &str,
        amount: f64,
        labels: Option<JsonValue>,
    ) -> MetricsResult<()> {
        self.record(name, MetricType::Counter, amount, labels).await
    }

    /// Set a gauge metric
    pub async fn gauge(&self, name: &str, value: f64, labels: Option<JsonValue>) -> MetricsResult<()> {
        self.record(name, MetricType::Gauge, value, labels).await
    }

    /// Record a histogram value (e.g., response time)
    pub async fn histogram(
        &self,
        name: &str,
        value: f64,
        labels: Option<JsonValue>,
    ) -> MetricsResult<()> {
        self.record(name, MetricType::Histogram, value, labels).await
    }

    /// Get metrics for a time range
    pub async fn get_metrics(
        &self,
        name: &str,
        hours: i32,
    ) -> MetricsResult<Vec<MetricDataPoint>> {
        let entries = sqlx::query_as(
            r#"
            SELECT metric_name, metric_type, value, labels, hour
            FROM metrics_hourly
            WHERE metric_name = $1 AND hour >= NOW() - ($2 || ' hours')::interval
            ORDER BY hour DESC
            "#,
        )
        .bind(name)
        .bind(hours.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Get aggregated metric value
    pub async fn get_aggregate(
        &self,
        name: &str,
        hours: i32,
        aggregation: AggregationType,
    ) -> MetricsResult<Option<f64>> {
        let agg_fn = match aggregation {
            AggregationType::Sum => "SUM(value)",
            AggregationType::Avg => "AVG(value)",
            AggregationType::Min => "MIN(value)",
            AggregationType::Max => "MAX(value)",
            AggregationType::Count => "COUNT(*)",
        };

        let query = format!(
            r#"
            SELECT {} as result
            FROM metrics_hourly
            WHERE metric_name = $1 AND hour >= NOW() - ($2 || ' hours')::interval
            "#,
            agg_fn
        );

        let result: Option<(Option<f64>,)> = sqlx::query_as(&query)
            .bind(name)
            .bind(hours.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.and_then(|r| r.0))
    }

    /// Log an HTTP request
    pub async fn log_request(&self, request: RequestLog) -> MetricsResult<Uuid> {
        let id: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO request_logs (
                request_id, method, path, query_params,
                user_id, api_key_id, ip_address,
                status_code, response_time_ms, error_code, error_message,
                started_at, completed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id
            "#,
        )
        .bind(request.request_id)
        .bind(&request.method)
        .bind(&request.path)
        .bind(&request.query_params)
        .bind(request.user_id)
        .bind(request.api_key_id)
        .bind(request.ip_address.map(|ip| ip.to_string()))
        .bind(request.status_code)
        .bind(request.response_time_ms)
        .bind(&request.error_code)
        .bind(&request.error_message)
        .bind(request.started_at)
        .bind(request.completed_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(id.0)
    }

    /// Record a health check result
    pub async fn record_health_check(
        &self,
        service: &str,
        status: HealthStatus,
        response_time_ms: Option<i32>,
        details: Option<JsonValue>,
    ) -> MetricsResult<()> {
        sqlx::query(
            r#"
            INSERT INTO health_check_history (service, status, response_time_ms, details)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(service)
        .bind(status.as_str())
        .bind(response_time_ms)
        .bind(details)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get latest health check for all services
    pub async fn get_health_status(&self) -> MetricsResult<Vec<ServiceHealth>> {
        let entries = sqlx::query_as(
            r#"
            SELECT DISTINCT ON (service)
                service, status, response_time_ms, details, checked_at
            FROM health_check_history
            ORDER BY service, checked_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Get request statistics
    pub async fn get_request_stats(&self, hours: i32) -> MetricsResult<RequestStats> {
        let stats: RequestStatsRow = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_requests,
                COUNT(*) FILTER (WHERE status_code >= 400) as error_count,
                AVG(response_time_ms) as avg_response_time,
                MAX(response_time_ms) as max_response_time,
                PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms) as p95_response_time,
                PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY response_time_ms) as p99_response_time
            FROM request_logs
            WHERE started_at >= NOW() - ($1 || ' hours')::interval
            "#,
        )
        .bind(hours.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(RequestStats {
            total_requests: stats.total_requests,
            error_count: stats.error_count,
            error_rate: if stats.total_requests > 0 {
                (stats.error_count as f64 / stats.total_requests as f64) * 100.0
            } else {
                0.0
            },
            avg_response_time_ms: stats.avg_response_time,
            max_response_time_ms: stats.max_response_time.unwrap_or(0),
            p95_response_time_ms: stats.p95_response_time,
            p99_response_time_ms: stats.p99_response_time,
        })
    }

    /// Get slowest endpoints
    pub async fn get_slowest_endpoints(&self, hours: i32, limit: i32) -> MetricsResult<Vec<EndpointStats>> {
        let entries = sqlx::query_as(
            r#"
            SELECT
                path,
                COUNT(*) as request_count,
                AVG(response_time_ms) as avg_response_time,
                MAX(response_time_ms) as max_response_time,
                COUNT(*) FILTER (WHERE status_code >= 400) as error_count
            FROM request_logs
            WHERE started_at >= NOW() - ($1 || ' hours')::interval
            GROUP BY path
            ORDER BY avg_response_time DESC
            LIMIT $2
            "#,
        )
        .bind(hours.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Clean up old request logs (keep last N hours)
    pub async fn cleanup_request_logs(&self, keep_hours: i32) -> MetricsResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM request_logs
            WHERE started_at < NOW() - ($1 || ' hours')::interval
            "#,
        )
        .bind(keep_hours.to_string())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AggregationType {
    Sum,
    Avg,
    Min,
    Max,
    Count,
}

#[derive(Debug, Clone, Copy)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MetricDataPoint {
    pub metric_name: String,
    pub metric_type: String,
    pub value: rust_decimal::Decimal,
    pub labels: JsonValue,
    pub hour: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct RequestStatsRow {
    total_requests: i64,
    error_count: i64,
    avg_response_time: Option<f64>,
    max_response_time: Option<i32>,
    p95_response_time: Option<f64>,
    p99_response_time: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RequestStats {
    pub total_requests: i64,
    pub error_count: i64,
    pub error_rate: f64,
    pub avg_response_time_ms: Option<f64>,
    pub max_response_time_ms: i32,
    pub p95_response_time_ms: Option<f64>,
    pub p99_response_time_ms: Option<f64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EndpointStats {
    pub path: String,
    pub request_count: i64,
    pub avg_response_time: Option<f64>,
    pub max_response_time: Option<i32>,
    pub error_count: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ServiceHealth {
    pub service: String,
    pub status: String,
    pub response_time_ms: Option<i32>,
    pub details: Option<JsonValue>,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct RequestLog {
    pub request_id: Uuid,
    pub method: String,
    pub path: String,
    pub query_params: Option<JsonValue>,
    pub user_id: Option<Uuid>,
    pub api_key_id: Option<Uuid>,
    pub ip_address: Option<std::net::IpAddr>,
    pub status_code: Option<i32>,
    pub response_time_ms: Option<i32>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl RequestLog {
    pub fn new(request_id: Uuid, method: &str, path: &str) -> Self {
        Self {
            request_id,
            method: method.to_string(),
            path: path.to_string(),
            query_params: None,
            user_id: None,
            api_key_id: None,
            ip_address: None,
            status_code: None,
            response_time_ms: None,
            error_code: None,
            error_message: None,
            started_at: chrono::Utc::now(),
            completed_at: None,
        }
    }
}

/// Helper struct for timing operations
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> i32 {
        self.start.elapsed().as_millis() as i32
    }
}

/// Common metric names
pub mod metric_names {
    pub const HTTP_REQUESTS_TOTAL: &str = "http_requests_total";
    pub const HTTP_REQUEST_DURATION_MS: &str = "http_request_duration_ms";
    pub const HTTP_ERRORS_TOTAL: &str = "http_errors_total";
    pub const DB_CONNECTIONS_ACTIVE: &str = "db_connections_active";
    pub const DB_QUERY_DURATION_MS: &str = "db_query_duration_ms";
    pub const CACHE_HITS: &str = "cache_hits";
    pub const CACHE_MISSES: &str = "cache_misses";
    pub const TICKETS_CREATED: &str = "tickets_created";
    pub const TICKETS_RESOLVED: &str = "tickets_resolved";
    pub const TIME_ENTRIES_LOGGED: &str = "time_entries_logged";
    pub const INVOICES_CREATED: &str = "invoices_created";
    pub const INVOICES_PAID: &str = "invoices_paid";
    pub const SLA_BREACHES: &str = "sla_breaches";
    pub const ACTIVE_USERS: &str = "active_users";
    pub const API_KEY_USAGE: &str = "api_key_usage";
}
