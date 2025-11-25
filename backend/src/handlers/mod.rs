use axum::{http::StatusCode, response::Json, routing::get, Router, extract::State};
use serde_json::json;
use serde::Serialize;
use std::sync::Arc;
use chrono::{Utc, Datelike};
use rust_decimal::Decimal;
use crate::AppState;

pub mod clients;
pub mod tickets;
pub mod ticket_advanced;
pub mod assets;
pub mod invoices;
pub mod time_tracking;
pub mod projects;
pub mod knowledge_base;
pub mod portal;
pub mod passwords;
pub mod asset_layouts;
pub mod asset_relationships;
pub mod sla_management;
pub mod network_topology;
pub mod forticloud;
pub mod license_alerts;
pub mod documentation;
pub mod reporting;
pub mod email;
pub mod billing;
pub mod analytics;
pub mod teams;

pub use clients::client_routes;
pub use tickets::ticket_routes;
pub use ticket_advanced::{ticket_queue_routes, canned_response_routes, ticket_link_routes, ticket_tag_routes, routing_rule_routes};
pub use assets::asset_routes;
pub use invoices::invoice_routes;
pub use time_tracking::time_tracking_routes;
pub use projects::project_routes;
pub use knowledge_base::knowledge_base_routes;
pub use portal::portal_routes;
pub use passwords::password_routes;
pub use asset_layouts::asset_layout_routes;
pub use asset_relationships::asset_relationship_routes;
pub use sla_management::sla_routes;
pub use network_topology::network_topology_routes;
pub use forticloud::forticloud_routes;
pub use license_alerts::license_alert_routes;
pub use documentation::documentation_routes;
pub use reporting::reporting_routes;
pub use email::email_routes;
pub use billing::billing_routes;
pub use analytics::analytics_routes;
pub use teams::teams_routes;

// Add user routes function
pub fn user_routes() -> axum::Router<std::sync::Arc<crate::AppState>> {
    use axum::routing::get;
    axum::Router::new()
        .route("/", get(|| async { "Users endpoint" }))
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub overview: OverviewStats,
    pub tickets: TicketStats,
    pub time: TimeStats,
    pub invoices: InvoiceStats,
    pub clients: ClientStats,
    pub assets: AssetStats,
}

#[derive(Debug, Serialize)]
pub struct OverviewStats {
    pub total_clients: i64,
    pub active_tickets: i64,
    pub monthly_revenue: Decimal,
    pub unbilled_time: Decimal,
    pub overdue_invoices: i64,
}

#[derive(Debug, Serialize)]
pub struct TicketStats {
    pub open: i64,
    pub in_progress: i64,
    pub pending: i64,
    pub resolved_today: i64,
    pub sla_breached: i64,
    pub avg_response_time_hours: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct TimeStats {
    pub hours_today: Decimal,
    pub billable_hours_today: Decimal,
    pub hours_this_week: Decimal,
    pub active_timers: i64,
    pub team_utilization: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceStats {
    pub outstanding_amount: Decimal,
    pub overdue_amount: Decimal,
    pub draft_count: i64,
    pub paid_this_month: Decimal,
    pub collection_ratio: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ClientStats {
    pub total_clients: i64,
    pub new_this_month: i64,
    pub top_clients_by_revenue: Vec<TopClient>,
}

#[derive(Debug, Serialize)]
pub struct TopClient {
    pub name: String,
    pub revenue: Decimal,
}

#[derive(Debug, Serialize)]
pub struct AssetStats {
    pub total_assets: i64,
    pub critical_alerts: i64,
    pub warranty_expiring: i64,
    pub online_percentage: Option<f64>,
}

pub async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({"status": "healthy", "service": "resolve-api"})))
}

pub async fn dashboard_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DashboardStats>, StatusCode> {
    let now = Utc::now();
    let today = now.date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let month_start = today.with_day(1).unwrap();
    
    // Simplified dashboard stats for compilation
    let dashboard = DashboardStats {
        overview: OverviewStats {
            total_clients: 0,
            active_tickets: 0,
            monthly_revenue: Decimal::ZERO,
            unbilled_time: Decimal::ZERO,
            overdue_invoices: 0,
        },
        tickets: TicketStats {
            open: 0,
            in_progress: 0,
            pending: 0,
            resolved_today: 0,
            sla_breached: 0,
            avg_response_time_hours: None,
        },
        time: TimeStats {
            hours_today: Decimal::ZERO,
            billable_hours_today: Decimal::ZERO,
            hours_this_week: Decimal::ZERO,
            active_timers: 0,
            team_utilization: None,
        },
        invoices: InvoiceStats {
            outstanding_amount: Decimal::ZERO,
            overdue_amount: Decimal::ZERO,
            draft_count: 0,
            paid_this_month: Decimal::ZERO,
            collection_ratio: None,
        },
        clients: ClientStats {
            total_clients: 0,
            new_this_month: 0,
            top_clients_by_revenue: vec![],
        },
        assets: AssetStats {
            total_assets: 0,
            critical_alerts: 0,
            warranty_expiring: 0,
            online_percentage: None,
        },
    };
    
    Ok(Json(dashboard))
}