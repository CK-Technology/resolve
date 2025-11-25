use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use std::sync::Arc;
use std::net::SocketAddr;
use uuid::Uuid;
use chrono::Utc;

use crate::AppState;
use crate::services::{MetricsService, RequestLog, Timer, metric_names};

/// Middleware layer for request observability
/// Tracks request timing, status codes, and errors
pub async fn observability_layer(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let request_id = Uuid::new_v4();
    let timer = Timer::start();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|q| {
        serde_json::json!({ "raw": q })
    });

    // Execute the request
    let response = next.run(request).await;

    let status_code = response.status().as_u16() as i32;
    let response_time_ms = timer.elapsed_ms();

    // Log the request asynchronously (don't block the response)
    let pool = state.db_pool.clone();
    let method_clone = method.clone();
    let path_clone = path.clone();

    tokio::spawn(async move {
        let metrics = MetricsService::new(pool);

        let mut request_log = RequestLog::new(request_id, &method_clone, &path_clone);
        request_log.query_params = query;
        request_log.ip_address = Some(addr.ip());
        request_log.status_code = Some(status_code);
        request_log.response_time_ms = Some(response_time_ms);
        request_log.completed_at = Some(Utc::now());

        if status_code >= 400 {
            request_log.error_code = Some(format!("HTTP_{}", status_code));
        }

        // Log the request
        let _ = metrics.log_request(request_log).await;

        // Record metrics
        let labels = serde_json::json!({
            "method": method_clone,
            "path": normalize_path(&path_clone),
            "status": status_code
        });

        let _ = metrics.increment(metric_names::HTTP_REQUESTS_TOTAL, Some(labels.clone())).await;
        let _ = metrics.histogram(metric_names::HTTP_REQUEST_DURATION_MS, response_time_ms as f64, Some(labels.clone())).await;

        if status_code >= 400 {
            let _ = metrics.increment(metric_names::HTTP_ERRORS_TOTAL, Some(labels)).await;
        }
    });

    response
}

/// Normalize path to group similar endpoints (e.g., /api/v1/tickets/123 -> /api/v1/tickets/:id)
fn normalize_path(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').collect();
    let normalized: Vec<String> = segments
        .iter()
        .map(|s| {
            if Uuid::parse_str(s).is_ok() {
                ":id".to_string()
            } else if s.parse::<i64>().is_ok() {
                ":id".to_string()
            } else {
                s.to_string()
            }
        })
        .collect();
    normalized.join("/")
}

/// Health check endpoint with detailed status
pub async fn detailed_health_check(
    State(state): State<Arc<AppState>>,
) -> Result<axum::Json<HealthCheckResponse>, StatusCode> {
    let metrics = MetricsService::new(state.db_pool.clone());
    let timer = Timer::start();

    // Check database connection
    let db_status = match sqlx::query("SELECT 1").execute(&state.db_pool).await {
        Ok(_) => ServiceStatus {
            status: "healthy".to_string(),
            response_time_ms: Some(timer.elapsed_ms()),
            details: None,
        },
        Err(e) => ServiceStatus {
            status: "unhealthy".to_string(),
            response_time_ms: Some(timer.elapsed_ms()),
            details: Some(serde_json::json!({ "error": e.to_string() })),
        },
    };

    // Record health check
    let _ = metrics
        .record_health_check(
            "database",
            if db_status.status == "healthy" {
                crate::services::HealthStatus::Healthy
            } else {
                crate::services::HealthStatus::Unhealthy
            },
            db_status.response_time_ms,
            db_status.details.clone(),
        )
        .await;

    let overall_status = if db_status.status == "healthy" {
        "healthy"
    } else {
        "unhealthy"
    };

    let response = HealthCheckResponse {
        status: overall_status.to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services: vec![("database".to_string(), db_status)]
            .into_iter()
            .collect(),
    };

    if overall_status == "healthy" {
        Ok(axum::Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub services: std::collections::HashMap<String, ServiceStatus>,
}

#[derive(Debug, serde::Serialize)]
pub struct ServiceStatus {
    pub status: String,
    pub response_time_ms: Option<i32>,
    pub details: Option<serde_json::Value>,
}

/// Metrics endpoint for observability
pub async fn metrics_endpoint(
    State(state): State<Arc<AppState>>,
) -> Result<axum::Json<MetricsResponse>, StatusCode> {
    let metrics = MetricsService::new(state.db_pool.clone());

    let request_stats = metrics
        .get_request_stats(24)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let slowest_endpoints = metrics
        .get_slowest_endpoints(24, 10)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let health_status = metrics
        .get_health_status()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(axum::Json(MetricsResponse {
        request_stats,
        slowest_endpoints,
        health_status,
        timestamp: Utc::now(),
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct MetricsResponse {
    pub request_stats: crate::services::RequestStats,
    pub slowest_endpoints: Vec<crate::services::metrics::EndpointStats>,
    pub health_status: Vec<crate::services::metrics::ServiceHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
