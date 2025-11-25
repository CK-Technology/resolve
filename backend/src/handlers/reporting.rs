use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;
use crate::AppState;
use crate::auth::middleware::AuthUser;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Report {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub report_type: String,
    pub data_sources: serde_json::Value,
    pub base_query: Option<String>,
    pub filters: serde_json::Value,
    pub parameters: serde_json::Value,
    pub chart_type: Option<String>,
    pub chart_config: serde_json::Value,
    pub layout_config: serde_json::Value,
    pub visibility: String,
    pub allowed_users: Vec<Uuid>,
    pub allowed_roles: Vec<String>,
    pub client_accessible: bool,
    pub cache_duration_minutes: i32,
    pub last_cached: Option<chrono::DateTime<Utc>>,
    pub cache_data: Option<serde_json::Value>,
    pub view_count: i32,
    pub last_viewed: Option<chrono::DateTime<Utc>>,
    pub is_active: bool,
    pub is_template: bool,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct KPI {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub calculation_query: String,
    pub calculation_frequency: String,
    pub unit: Option<String>,
    pub format_pattern: Option<String>,
    pub target_value: Option<rust_decimal::Decimal>,
    pub warning_threshold: Option<rust_decimal::Decimal>,
    pub critical_threshold: Option<rust_decimal::Decimal>,
    pub good_direction: String,
    pub chart_type: String,
    pub color_good: String,
    pub color_warning: String,
    pub color_critical: String,
    pub is_active: bool,
    pub last_calculated: Option<chrono::DateTime<Utc>>,
    pub current_value: Option<rust_decimal::Decimal>,
    pub previous_value: Option<rust_decimal::Decimal>,
    pub trend: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ClientHealthScore {
    pub id: Uuid,
    pub client_id: Uuid,
    pub client_name: Option<String>,
    pub overall_score: i32,
    pub score_trend: Option<String>,
    pub asset_health_score: i32,
    pub ticket_satisfaction_score: i32,
    pub financial_health_score: i32,
    pub communication_score: i32,
    pub security_score: i32,
    pub risk_level: String,
    pub risk_factors: Option<Vec<String>>,
    pub recommendations: Option<Vec<String>>,
    pub calculation_date: chrono::NaiveDate,
    pub data_completeness_percent: i32,
    pub calculation_version: String,
    pub alert_sent: bool,
    pub alert_sent_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub category: Option<String>,
    pub report_type: Option<String>,
    pub visibility: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteReportRequest {
    pub parameters: Option<serde_json::Value>,
    pub filters: Option<serde_json::Value>,
    pub format: Option<String>,
}

pub fn reporting_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/reports", get(list_reports))
        .route("/reports/:id", get(get_report))
        .route("/reports/:id/execute", post(execute_report))
        .route("/reports/:id/data", get(get_report_data))
        .route("/kpis", get(list_kpis))
        .route("/kpis/:id", get(get_kpi))
        .route("/client-health", get(get_client_health_scores))
        .route("/client-health/:client_id", get(get_client_health_score))
        .route("/dashboard/stats", get(get_dashboard_stats))
        .route("/dashboard/widgets", get(get_dashboard_widgets))
}

async fn list_reports(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ReportQuery>,
    auth: AuthUser,
) -> Result<Json<Vec<Report>>, StatusCode> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let reports = sqlx::query_as!(
        Report,
        r#"
        SELECT id, name, description, category, report_type, data_sources, base_query,
               filters, parameters, chart_type, chart_config, layout_config,
               visibility, allowed_users, allowed_roles, client_accessible,
               cache_duration_minutes, last_cached, cache_data, view_count,
               last_viewed, is_active, is_template, created_by, created_at, updated_at
        FROM reports
        WHERE is_active = true
        AND (visibility = 'company' OR $1 = ANY(allowed_users) OR created_by = $1)
        ORDER BY category, name
        LIMIT $2 OFFSET $3
        "#,
        auth.0.id,
        limit,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(reports))
}

async fn get_report(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<Report>, StatusCode> {
    let report = sqlx::query_as!(
        Report,
        r#"
        SELECT id, name, description, category, report_type, data_sources, base_query,
               filters, parameters, chart_type, chart_config, layout_config,
               visibility, allowed_users, allowed_roles, client_accessible,
               cache_duration_minutes, last_cached, cache_data, view_count,
               last_viewed, is_active, is_template, created_by, created_at, updated_at
        FROM reports
        WHERE id = $1
        AND (visibility = 'company' OR $2 = ANY(allowed_users) OR created_by = $2)
        "#,
        id,
        auth.0.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Increment view count
    let _ = sqlx::query!(
        "UPDATE reports SET view_count = view_count + 1, last_viewed = NOW() WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await;

    Ok(Json(report))
}

async fn execute_report(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(req): Json<ExecuteReportRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let report = sqlx::query!(
        "SELECT base_query, cache_data, last_cached, cache_duration_minutes FROM reports WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Check cache validity
    let cache_valid = if let (Some(cached_data), Some(last_cached)) = (&report.cache_data, &report.last_cached) {
        let cache_age = Utc::now() - *last_cached;
        cache_age.num_minutes() < report.cache_duration_minutes as i64
    } else {
        false
    };

    if cache_valid && report.cache_data.is_some() {
        return Ok(Json(report.cache_data.unwrap()));
    }

    // Execute query (simplified for demo)
    let data = match report.base_query.as_deref() {
        Some(query) if query.contains("bi_metrics_daily") => {
            // Sample dashboard data
            serde_json::json!({
                "data": [
                    {"metric_date": "2024-01-07", "revenue_total": 15000, "tickets_created": 25, "tickets_resolved": 22},
                    {"metric_date": "2024-01-06", "revenue_total": 12000, "tickets_created": 18, "tickets_resolved": 20},
                    {"metric_date": "2024-01-05", "revenue_total": 18000, "tickets_created": 30, "tickets_resolved": 28}
                ],
                "summary": {
                    "total_revenue": 45000,
                    "total_tickets": 73,
                    "resolution_rate": 95.9
                }
            })
        },
        _ => {
            serde_json::json!({
                "data": [],
                "message": "Report executed successfully",
                "generated_at": Utc::now()
            })
        }
    };

    // Cache the result
    let _ = sqlx::query!(
        "UPDATE reports SET cache_data = $1, last_cached = NOW() WHERE id = $2",
        &data,
        id
    )
    .execute(&state.db_pool)
    .await;

    Ok(Json(data))
}

async fn get_report_data(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let req = ExecuteReportRequest {
        parameters: None,
        filters: None,
        format: None,
    };
    execute_report(State(state), Path(id), auth, Json(req)).await
}

async fn list_kpis(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<KPI>>, StatusCode> {
    let kpis = sqlx::query_as!(
        KPI,
        r#"
        SELECT id, name, description, category, calculation_query, calculation_frequency,
               unit, format_pattern, target_value, warning_threshold, critical_threshold,
               good_direction, chart_type, color_good, color_warning, color_critical,
               is_active, last_calculated, current_value, previous_value, trend,
               created_by, created_at, updated_at
        FROM kpis
        WHERE is_active = true
        ORDER BY category, name
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(kpis))
}

async fn get_kpi(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<KPI>, StatusCode> {
    let kpi = sqlx::query_as!(
        KPI,
        r#"
        SELECT id, name, description, category, calculation_query, calculation_frequency,
               unit, format_pattern, target_value, warning_threshold, critical_threshold,
               good_direction, chart_type, color_good, color_warning, color_critical,
               is_active, last_calculated, current_value, previous_value, trend,
               created_by, created_at, updated_at
        FROM kpis
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(kpi))
}

async fn get_client_health_scores(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<ClientHealthScore>>, StatusCode> {
    let scores = sqlx::query_as!(
        ClientHealthScore,
        r#"
        SELECT chs.*, c.name as client_name
        FROM client_health_scores chs
        JOIN clients c ON c.id = chs.client_id
        WHERE chs.calculation_date = (
            SELECT MAX(calculation_date) 
            FROM client_health_scores chs2 
            WHERE chs2.client_id = chs.client_id
        )
        ORDER BY chs.overall_score ASC
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(scores))
}

async fn get_client_health_score(
    State(state): State<Arc<AppState>>,
    Path(client_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<ClientHealthScore>, StatusCode> {
    let score = sqlx::query_as!(
        ClientHealthScore,
        r#"
        SELECT chs.*, c.name as client_name
        FROM client_health_scores chs
        JOIN clients c ON c.id = chs.client_id
        WHERE chs.client_id = $1
        ORDER BY chs.calculation_date DESC
        LIMIT 1
        "#,
        client_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(score))
}

async fn get_dashboard_stats(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Sample dashboard stats for demo
    let stats = serde_json::json!({
        "overview": {
            "total_clients": 42,
            "active_tickets": 18,
            "monthly_revenue": 125000.00,
            "unbilled_time": 47.5,
            "overdue_invoices": 3
        },
        "tickets": {
            "open": 8,
            "in_progress": 10,
            "pending": 5,
            "resolved_today": 7,
            "sla_breached": 2,
            "avg_response_time_hours": 3.4
        },
        "financials": {
            "mrr": 89500.00,
            "arr": 1074000.00,
            "gross_margin": 78.5,
            "client_ltv": 45600.00
        },
        "team": {
            "utilization_rate": 76.8,
            "billable_hours_today": 52.5,
            "efficiency_score": 82.3
        },
        "health_scores": {
            "average_client_health": 76,
            "clients_at_risk": 5,
            "trending_up": 12,
            "trending_down": 3
        }
    });

    Ok(Json(stats))
}

async fn get_dashboard_widgets(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    // Sample widget data for demo
    let widgets = vec![
        serde_json::json!({
            "id": "revenue-chart",
            "type": "chart",
            "title": "Revenue Trend",
            "position": {"x": 0, "y": 0, "width": 6, "height": 4},
            "data": {
                "chart_type": "line",
                "data": [
                    {"date": "2024-01-01", "value": 95000},
                    {"date": "2024-01-02", "value": 98000},
                    {"date": "2024-01-03", "value": 102000},
                    {"date": "2024-01-04", "value": 99000},
                    {"date": "2024-01-05", "value": 108000},
                    {"date": "2024-01-06", "value": 112000},
                    {"date": "2024-01-07", "value": 125000}
                ]
            }
        }),
        serde_json::json!({
            "id": "ticket-status",
            "type": "donut",
            "title": "Ticket Status",
            "position": {"x": 6, "y": 0, "width": 3, "height": 4},
            "data": {
                "chart_type": "donut",
                "data": [
                    {"label": "Open", "value": 8, "color": "#ef4444"},
                    {"label": "In Progress", "value": 10, "color": "#f59e0b"},
                    {"label": "Pending", "value": 5, "color": "#8b5cf6"},
                    {"label": "Resolved", "value": 25, "color": "#10b981"}
                ]
            }
        }),
        serde_json::json!({
            "id": "client-health",
            "type": "metric",
            "title": "Avg Client Health",
            "position": {"x": 9, "y": 0, "width": 3, "height": 2},
            "data": {
                "value": 76,
                "unit": "score",
                "trend": "up",
                "change": "+2.3"
            }
        }),
        serde_json::json!({
            "id": "sla-compliance",
            "type": "metric",
            "title": "SLA Compliance",
            "position": {"x": 9, "y": 2, "width": 3, "height": 2},
            "data": {
                "value": 94.2,
                "unit": "%",
                "trend": "down",
                "change": "-1.8"
            }
        })
    ];

    Ok(Json(widgets))
}