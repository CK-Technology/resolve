//! Analytics and Reporting - Phase 7
//!
//! Technician utilization, client profitability, and SLA compliance reports.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate, Datelike};
use rust_decimal::Decimal;
use crate::{
    AppState, ApiResult, ApiError,
    PaginatedResponse, PaginationParams,
};
use crate::auth::middleware::AuthUser;

// ==================== Query Parameters ====================

#[derive(Debug, Clone, Deserialize)]
pub struct DateRangeQuery {
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub group_by: Option<String>, // day, week, month
}

impl DateRangeQuery {
    pub fn get_range(&self) -> (NaiveDate, NaiveDate) {
        let today = Utc::now().date_naive();
        let from = self.from_date.unwrap_or_else(|| today - chrono::Duration::days(30));
        let to = self.to_date.unwrap_or(today);
        (from, to)
    }
}

// ==================== Technician Utilization ====================

#[derive(Debug, Clone, Serialize)]
pub struct TechnicianUtilization {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub avatar_url: Option<String>,

    // Time metrics
    pub total_hours: Decimal,
    pub billable_hours: Decimal,
    pub non_billable_hours: Decimal,
    pub utilization_rate: Decimal, // billable / total target hours

    // Target and capacity
    pub target_hours: Decimal, // expected hours for period (e.g., 40hrs/week)
    pub capacity_used: Decimal, // total hours / target hours

    // Billing metrics
    pub total_billed: Decimal,
    pub effective_rate: Decimal, // total billed / billable hours

    // Ticket metrics
    pub tickets_worked: i64,
    pub tickets_resolved: i64,
    pub avg_resolution_time_hours: Option<Decimal>,

    // Trend
    pub trend: String, // up, down, stable
    pub trend_change: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct UtilizationSummary {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub total_technicians: i64,
    pub avg_utilization_rate: Decimal,
    pub total_billable_hours: Decimal,
    pub total_revenue: Decimal,
    pub top_performer_id: Option<Uuid>,
    pub top_performer_name: Option<String>,
    pub technicians: Vec<TechnicianUtilization>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UtilizationTrend {
    pub date: NaiveDate,
    pub total_hours: Decimal,
    pub billable_hours: Decimal,
    pub utilization_rate: Decimal,
    pub technician_count: i64,
}

// ==================== Client Profitability ====================

#[derive(Debug, Clone, Serialize)]
pub struct ClientProfitability {
    pub client_id: Uuid,
    pub client_name: String,
    pub client_type: Option<String>,

    // Revenue metrics
    pub total_revenue: Decimal,
    pub recurring_revenue: Decimal,
    pub one_time_revenue: Decimal,
    pub average_monthly_revenue: Decimal,

    // Cost metrics
    pub total_cost: Decimal, // labor cost based on time spent
    pub labor_hours: Decimal,
    pub labor_cost: Decimal,
    pub other_costs: Decimal,

    // Profitability
    pub gross_profit: Decimal,
    pub gross_margin: Decimal, // (revenue - cost) / revenue * 100
    pub profit_per_hour: Decimal,

    // Efficiency metrics
    pub tickets_opened: i64,
    pub tickets_resolved: i64,
    pub avg_resolution_time_hours: Option<Decimal>,
    pub cost_per_ticket: Decimal,

    // Health indicators
    pub payment_score: i32, // 0-100, based on payment history
    pub engagement_score: i32, // 0-100, based on activity
    pub risk_level: String, // low, medium, high

    // Contract info
    pub contract_value: Option<Decimal>,
    pub contract_end_date: Option<NaiveDate>,
    pub months_as_client: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfitabilitySummary {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub total_clients: i64,
    pub total_revenue: Decimal,
    pub total_cost: Decimal,
    pub total_profit: Decimal,
    pub avg_margin: Decimal,
    pub clients: Vec<ClientProfitability>,
    pub top_clients: Vec<ClientProfitability>,
    pub at_risk_clients: Vec<ClientProfitability>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientRevenueTrend {
    pub client_id: Uuid,
    pub client_name: String,
    pub data: Vec<RevenueTrendPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RevenueTrendPoint {
    pub date: NaiveDate,
    pub revenue: Decimal,
    pub cost: Decimal,
    pub profit: Decimal,
}

// ==================== SLA Compliance ====================

#[derive(Debug, Clone, Serialize)]
pub struct SlaComplianceSummary {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,

    // Overall metrics
    pub total_tickets: i64,
    pub tickets_with_sla: i64,
    pub tickets_met_sla: i64,
    pub tickets_breached_sla: i64,
    pub compliance_rate: Decimal,

    // By type
    pub first_response_compliance: Decimal,
    pub resolution_compliance: Decimal,

    // By priority
    pub by_priority: Vec<SlaPriorityBreakdown>,

    // By client
    pub by_client: Vec<SlaClientBreakdown>,

    // Trends
    pub trends: Vec<SlaComplianceTrend>,

    // Recent breaches
    pub recent_breaches: Vec<SlaBreachDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlaPriorityBreakdown {
    pub priority: String,
    pub total_tickets: i64,
    pub met_sla: i64,
    pub breached_sla: i64,
    pub compliance_rate: Decimal,
    pub avg_response_time_minutes: i64,
    pub avg_resolution_time_minutes: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlaClientBreakdown {
    pub client_id: Uuid,
    pub client_name: String,
    pub total_tickets: i64,
    pub met_sla: i64,
    pub breached_sla: i64,
    pub compliance_rate: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlaComplianceTrend {
    pub date: NaiveDate,
    pub total_tickets: i64,
    pub met_sla: i64,
    pub compliance_rate: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlaBreachDetail {
    pub ticket_id: Uuid,
    pub ticket_number: i32,
    pub subject: String,
    pub client_id: Uuid,
    pub client_name: String,
    pub priority: String,
    pub breach_type: String, // first_response, resolution
    pub expected_at: DateTime<Utc>,
    pub actual_at: Option<DateTime<Utc>>,
    pub breach_duration_minutes: i64,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlaPerformanceByTechnician {
    pub user_id: Uuid,
    pub user_name: String,
    pub total_assigned: i64,
    pub met_sla: i64,
    pub breached_sla: i64,
    pub compliance_rate: Decimal,
    pub avg_response_time_minutes: i64,
    pub avg_resolution_time_minutes: i64,
}

// ==================== Routes ====================

pub fn analytics_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Technician Utilization
        .route("/utilization", get(get_utilization_report))
        .route("/utilization/trend", get(get_utilization_trend))
        .route("/utilization/by-technician/:id", get(get_technician_utilization))
        // Client Profitability
        .route("/profitability", get(get_profitability_report))
        .route("/profitability/by-client/:id", get(get_client_profitability))
        .route("/profitability/trend", get(get_profitability_trend))
        .route("/profitability/at-risk", get(get_at_risk_clients))
        // SLA Compliance
        .route("/sla", get(get_sla_compliance))
        .route("/sla/by-priority", get(get_sla_by_priority))
        .route("/sla/by-client", get(get_sla_by_client))
        .route("/sla/by-technician", get(get_sla_by_technician))
        .route("/sla/breaches", get(get_sla_breaches))
        .route("/sla/trend", get(get_sla_trend))
        // Executive Summary
        .route("/executive-summary", get(get_executive_summary))
}

// ==================== Utilization Handlers ====================

async fn get_utilization_report(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<UtilizationSummary>> {
    let (from_date, to_date) = params.get_range();

    // Calculate target hours (assuming 8 hours/day, 5 days/week)
    let days = (to_date - from_date).num_days() as i64;
    let work_days = days * 5 / 7;
    let target_hours_per_tech = Decimal::from(work_days * 8);

    let technicians = sqlx::query!(
        r#"SELECT
            u.id as user_id,
            u.first_name || ' ' || u.last_name as user_name,
            u.email as user_email,
            COALESCE(SUM(te.duration_minutes), 0)::decimal / 60.0 as "total_hours!",
            COALESCE(SUM(te.duration_minutes) FILTER (WHERE te.billable), 0)::decimal / 60.0 as "billable_hours!",
            COALESCE(SUM(te.duration_minutes) FILTER (WHERE NOT te.billable), 0)::decimal / 60.0 as "non_billable_hours!",
            COALESCE(SUM(te.total_amount) FILTER (WHERE te.billable), 0) as "total_billed!",
            COUNT(DISTINCT te.ticket_id) as "tickets_worked!",
            0::bigint as "tickets_resolved!"
         FROM users u
         LEFT JOIN time_entries te ON u.id = te.user_id
            AND te.start_time::date >= $1
            AND te.start_time::date <= $2
            AND te.end_time IS NOT NULL
         WHERE u.is_active = true
         GROUP BY u.id, u.first_name, u.last_name, u.email
         HAVING COALESCE(SUM(te.duration_minutes), 0) > 0
         ORDER BY "billable_hours!" DESC"#,
        from_date,
        to_date
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching utilization: {}", e);
        ApiError::internal("Failed to fetch utilization report")
    })?;

    let mut result_technicians: Vec<TechnicianUtilization> = Vec::new();
    let mut total_billable = Decimal::ZERO;
    let mut total_revenue = Decimal::ZERO;

    for tech in technicians {
        let utilization_rate = if target_hours_per_tech > Decimal::ZERO {
            (tech.billable_hours / target_hours_per_tech) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        let capacity_used = if target_hours_per_tech > Decimal::ZERO {
            (tech.total_hours / target_hours_per_tech) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        let effective_rate = if tech.billable_hours > Decimal::ZERO {
            tech.total_billed / tech.billable_hours
        } else {
            Decimal::ZERO
        };

        total_billable += tech.billable_hours;
        total_revenue += tech.total_billed;

        result_technicians.push(TechnicianUtilization {
            user_id: tech.user_id,
            user_name: tech.user_name.unwrap_or_else(|| "Unknown".to_string()),
            user_email: tech.user_email,
            avatar_url: None,
            total_hours: tech.total_hours,
            billable_hours: tech.billable_hours,
            non_billable_hours: tech.non_billable_hours,
            utilization_rate,
            target_hours: target_hours_per_tech,
            capacity_used,
            total_billed: tech.total_billed,
            effective_rate,
            tickets_worked: tech.tickets_worked,
            tickets_resolved: tech.tickets_resolved,
            avg_resolution_time_hours: None,
            trend: "stable".to_string(),
            trend_change: Decimal::ZERO,
        });
    }

    let total_technicians = result_technicians.len() as i64;
    let avg_utilization_rate = if total_technicians > 0 {
        result_technicians.iter()
            .map(|t| t.utilization_rate)
            .sum::<Decimal>() / Decimal::from(total_technicians)
    } else {
        Decimal::ZERO
    };

    let top = result_technicians.first();

    Ok(Json(UtilizationSummary {
        period_start: from_date,
        period_end: to_date,
        total_technicians,
        avg_utilization_rate,
        total_billable_hours: total_billable,
        total_revenue,
        top_performer_id: top.map(|t| t.user_id),
        top_performer_name: top.map(|t| t.user_name.clone()),
        technicians: result_technicians,
    }))
}

async fn get_utilization_trend(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<Vec<UtilizationTrend>>> {
    let (from_date, to_date) = params.get_range();

    let trends = sqlx::query!(
        r#"SELECT
            te.start_time::date as "date!",
            COALESCE(SUM(te.duration_minutes), 0)::decimal / 60.0 as "total_hours!",
            COALESCE(SUM(te.duration_minutes) FILTER (WHERE te.billable), 0)::decimal / 60.0 as "billable_hours!",
            COUNT(DISTINCT te.user_id) as "technician_count!"
         FROM time_entries te
         WHERE te.start_time::date >= $1
           AND te.start_time::date <= $2
           AND te.end_time IS NOT NULL
         GROUP BY te.start_time::date
         ORDER BY "date!" ASC"#,
        from_date,
        to_date
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching utilization trend: {}", e);
        ApiError::internal("Failed to fetch utilization trend")
    })?;

    let result: Vec<UtilizationTrend> = trends
        .into_iter()
        .map(|row| {
            let target = Decimal::from(row.technician_count * 8);
            let util_rate = if target > Decimal::ZERO {
                (row.billable_hours / target) * Decimal::from(100)
            } else {
                Decimal::ZERO
            };

            UtilizationTrend {
                date: row.date,
                total_hours: row.total_hours,
                billable_hours: row.billable_hours,
                utilization_rate: util_rate,
                technician_count: row.technician_count,
            }
        })
        .collect();

    Ok(Json(result))
}

async fn get_technician_utilization(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<TechnicianUtilization>> {
    let (from_date, to_date) = params.get_range();

    let days = (to_date - from_date).num_days() as i64;
    let work_days = days * 5 / 7;
    let target_hours = Decimal::from(work_days * 8);

    let tech = sqlx::query!(
        r#"SELECT
            u.id as user_id,
            u.first_name || ' ' || u.last_name as user_name,
            u.email as user_email,
            COALESCE(SUM(te.duration_minutes), 0)::decimal / 60.0 as "total_hours!",
            COALESCE(SUM(te.duration_minutes) FILTER (WHERE te.billable), 0)::decimal / 60.0 as "billable_hours!",
            COALESCE(SUM(te.duration_minutes) FILTER (WHERE NOT te.billable), 0)::decimal / 60.0 as "non_billable_hours!",
            COALESCE(SUM(te.total_amount) FILTER (WHERE te.billable), 0) as "total_billed!",
            COUNT(DISTINCT te.ticket_id) as "tickets_worked!"
         FROM users u
         LEFT JOIN time_entries te ON u.id = te.user_id
            AND te.start_time::date >= $2
            AND te.start_time::date <= $3
            AND te.end_time IS NOT NULL
         WHERE u.id = $1
         GROUP BY u.id, u.first_name, u.last_name, u.email"#,
        user_id,
        from_date,
        to_date
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::internal("Failed to fetch technician utilization"))?
    .ok_or_else(|| ApiError::not_found("User not found"))?;

    let utilization_rate = if target_hours > Decimal::ZERO {
        (tech.billable_hours / target_hours) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let capacity_used = if target_hours > Decimal::ZERO {
        (tech.total_hours / target_hours) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let effective_rate = if tech.billable_hours > Decimal::ZERO {
        tech.total_billed / tech.billable_hours
    } else {
        Decimal::ZERO
    };

    Ok(Json(TechnicianUtilization {
        user_id: tech.user_id,
        user_name: tech.user_name.unwrap_or_else(|| "Unknown".to_string()),
        user_email: tech.user_email,
        avatar_url: None,
        total_hours: tech.total_hours,
        billable_hours: tech.billable_hours,
        non_billable_hours: tech.non_billable_hours,
        utilization_rate,
        target_hours,
        capacity_used,
        total_billed: tech.total_billed,
        effective_rate,
        tickets_worked: tech.tickets_worked,
        tickets_resolved: 0,
        avg_resolution_time_hours: None,
        trend: "stable".to_string(),
        trend_change: Decimal::ZERO,
    }))
}

// ==================== Profitability Handlers ====================

async fn get_profitability_report(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<ProfitabilitySummary>> {
    let (from_date, to_date) = params.get_range();

    // Assume $50/hour internal cost for simplicity
    let cost_per_hour = Decimal::from(50);

    let clients = sqlx::query!(
        r#"SELECT
            c.id as client_id,
            c.name as client_name,
            c.type as client_type,
            COALESCE(SUM(inv.total), 0) as "total_revenue!",
            COALESCE(SUM(te.duration_minutes), 0)::decimal / 60.0 as "labor_hours!",
            COUNT(DISTINCT t.id) FILTER (WHERE t.created_at::date >= $1) as "tickets_opened!",
            COUNT(DISTINCT t.id) FILTER (WHERE t.resolved_at::date >= $1) as "tickets_resolved!"
         FROM clients c
         LEFT JOIN invoices inv ON inv.client_id = c.id
            AND inv.date >= $1 AND inv.date <= $2
         LEFT JOIN tickets t ON t.client_id = c.id
         LEFT JOIN time_entries te ON te.ticket_id = t.id
            AND te.start_time::date >= $1
            AND te.start_time::date <= $2
         WHERE c.is_active = true
         GROUP BY c.id, c.name, c.type
         HAVING COALESCE(SUM(inv.total), 0) > 0 OR COALESCE(SUM(te.duration_minutes), 0) > 0
         ORDER BY "total_revenue!" DESC"#,
        from_date,
        to_date
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching profitability: {}", e);
        ApiError::internal("Failed to fetch profitability report")
    })?;

    let mut result_clients: Vec<ClientProfitability> = Vec::new();
    let mut total_revenue = Decimal::ZERO;
    let mut total_cost = Decimal::ZERO;

    for client in clients {
        let labor_cost = client.labor_hours * cost_per_hour;
        let gross_profit = client.total_revenue - labor_cost;
        let gross_margin = if client.total_revenue > Decimal::ZERO {
            (gross_profit / client.total_revenue) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        let profit_per_hour = if client.labor_hours > Decimal::ZERO {
            gross_profit / client.labor_hours
        } else {
            Decimal::ZERO
        };

        let cost_per_ticket = if client.tickets_resolved > 0 {
            labor_cost / Decimal::from(client.tickets_resolved)
        } else {
            Decimal::ZERO
        };

        total_revenue += client.total_revenue;
        total_cost += labor_cost;

        let risk_level = if gross_margin < Decimal::from(20) {
            "high"
        } else if gross_margin < Decimal::from(40) {
            "medium"
        } else {
            "low"
        };

        result_clients.push(ClientProfitability {
            client_id: client.client_id,
            client_name: client.client_name,
            client_type: client.client_type,
            total_revenue: client.total_revenue,
            recurring_revenue: Decimal::ZERO, // Would need contract data
            one_time_revenue: client.total_revenue,
            average_monthly_revenue: client.total_revenue, // Simplified
            total_cost: labor_cost,
            labor_hours: client.labor_hours,
            labor_cost,
            other_costs: Decimal::ZERO,
            gross_profit,
            gross_margin,
            profit_per_hour,
            tickets_opened: client.tickets_opened,
            tickets_resolved: client.tickets_resolved,
            avg_resolution_time_hours: None,
            cost_per_ticket,
            payment_score: 80, // Would calculate from payment history
            engagement_score: 70, // Would calculate from activity
            risk_level: risk_level.to_string(),
            contract_value: None,
            contract_end_date: None,
            months_as_client: 12, // Would calculate from created_at
        });
    }

    let total_profit = total_revenue - total_cost;
    let avg_margin = if total_revenue > Decimal::ZERO {
        (total_profit / total_revenue) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let top_clients: Vec<ClientProfitability> = result_clients.iter()
        .take(5)
        .cloned()
        .collect();

    let at_risk_clients: Vec<ClientProfitability> = result_clients.iter()
        .filter(|c| c.risk_level == "high")
        .cloned()
        .collect();

    Ok(Json(ProfitabilitySummary {
        period_start: from_date,
        period_end: to_date,
        total_clients: result_clients.len() as i64,
        total_revenue,
        total_cost,
        total_profit,
        avg_margin,
        clients: result_clients,
        top_clients,
        at_risk_clients,
    }))
}

async fn get_client_profitability(
    State(state): State<Arc<AppState>>,
    Path(client_id): Path<Uuid>,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<ClientProfitability>> {
    let (from_date, to_date) = params.get_range();
    let cost_per_hour = Decimal::from(50);

    let client = sqlx::query!(
        r#"SELECT
            c.id as client_id,
            c.name as client_name,
            c.type as client_type,
            COALESCE(SUM(inv.total), 0) as "total_revenue!",
            COALESCE(SUM(te.duration_minutes), 0)::decimal / 60.0 as "labor_hours!",
            COUNT(DISTINCT t.id) FILTER (WHERE t.created_at::date >= $2) as "tickets_opened!",
            COUNT(DISTINCT t.id) FILTER (WHERE t.resolved_at::date >= $2) as "tickets_resolved!"
         FROM clients c
         LEFT JOIN invoices inv ON inv.client_id = c.id
            AND inv.date >= $2 AND inv.date <= $3
         LEFT JOIN tickets t ON t.client_id = c.id
         LEFT JOIN time_entries te ON te.ticket_id = t.id
            AND te.start_time::date >= $2
            AND te.start_time::date <= $3
         WHERE c.id = $1
         GROUP BY c.id, c.name, c.type"#,
        client_id,
        from_date,
        to_date
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::internal("Failed to fetch client profitability"))?
    .ok_or_else(|| ApiError::not_found("Client not found"))?;

    let labor_cost = client.labor_hours * cost_per_hour;
    let gross_profit = client.total_revenue - labor_cost;
    let gross_margin = if client.total_revenue > Decimal::ZERO {
        (gross_profit / client.total_revenue) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let profit_per_hour = if client.labor_hours > Decimal::ZERO {
        gross_profit / client.labor_hours
    } else {
        Decimal::ZERO
    };

    let cost_per_ticket = if client.tickets_resolved > 0 {
        labor_cost / Decimal::from(client.tickets_resolved)
    } else {
        Decimal::ZERO
    };

    let risk_level = if gross_margin < Decimal::from(20) {
        "high"
    } else if gross_margin < Decimal::from(40) {
        "medium"
    } else {
        "low"
    };

    Ok(Json(ClientProfitability {
        client_id: client.client_id,
        client_name: client.client_name,
        client_type: client.client_type,
        total_revenue: client.total_revenue,
        recurring_revenue: Decimal::ZERO,
        one_time_revenue: client.total_revenue,
        average_monthly_revenue: client.total_revenue,
        total_cost: labor_cost,
        labor_hours: client.labor_hours,
        labor_cost,
        other_costs: Decimal::ZERO,
        gross_profit,
        gross_margin,
        profit_per_hour,
        tickets_opened: client.tickets_opened,
        tickets_resolved: client.tickets_resolved,
        avg_resolution_time_hours: None,
        cost_per_ticket,
        payment_score: 80,
        engagement_score: 70,
        risk_level: risk_level.to_string(),
        contract_value: None,
        contract_end_date: None,
        months_as_client: 12,
    }))
}

async fn get_profitability_trend(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<Vec<RevenueTrendPoint>>> {
    let (from_date, to_date) = params.get_range();
    let cost_per_hour = Decimal::from(50);

    let trends = sqlx::query!(
        r#"SELECT
            d::date as "date!",
            COALESCE(SUM(inv.total), 0) as "revenue!",
            COALESCE(SUM(te.duration_minutes), 0)::decimal / 60.0 * $3 as "cost!"
         FROM generate_series($1::date, $2::date, '1 day'::interval) d
         LEFT JOIN invoices inv ON inv.date = d::date
         LEFT JOIN time_entries te ON te.start_time::date = d::date AND te.end_time IS NOT NULL
         GROUP BY d::date
         ORDER BY d::date ASC"#,
        from_date,
        to_date,
        cost_per_hour
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching profitability trend: {}", e);
        ApiError::internal("Failed to fetch profitability trend")
    })?;

    let result: Vec<RevenueTrendPoint> = trends
        .into_iter()
        .map(|row| RevenueTrendPoint {
            date: row.date,
            revenue: row.revenue,
            cost: row.cost,
            profit: row.revenue - row.cost,
        })
        .collect();

    Ok(Json(result))
}

async fn get_at_risk_clients(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<Vec<ClientProfitability>>> {
    // Reuse profitability report and filter
    let report = get_profitability_report(
        State(state.clone()),
        AuthUser(crate::auth::middleware::AuthenticatedUser {
            id: Uuid::nil(),
            email: "".to_string(),
            role: "admin".to_string(),
        }),
        Query(params),
    )
    .await?;

    Ok(Json(report.0.at_risk_clients))
}

// ==================== SLA Compliance Handlers ====================

async fn get_sla_compliance(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<SlaComplianceSummary>> {
    let (from_date, to_date) = params.get_range();

    // Get overall SLA stats
    let stats = sqlx::query!(
        r#"SELECT
            COUNT(*) as "total_tickets!",
            COUNT(*) FILTER (WHERE sla_policy_id IS NOT NULL) as "tickets_with_sla!",
            COUNT(*) FILTER (WHERE sla_response_at IS NOT NULL AND sla_response_at <= sla_response_due) as "met_response_sla!",
            COUNT(*) FILTER (WHERE sla_response_at IS NOT NULL AND sla_response_at > sla_response_due) as "breached_response_sla!",
            COUNT(*) FILTER (WHERE resolved_at IS NOT NULL AND resolved_at <= sla_resolution_due) as "met_resolution_sla!",
            COUNT(*) FILTER (WHERE resolved_at IS NOT NULL AND resolved_at > sla_resolution_due) as "breached_resolution_sla!"
         FROM tickets
         WHERE created_at::date >= $1
           AND created_at::date <= $2"#,
        from_date,
        to_date
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching SLA stats: {}", e);
        ApiError::internal("Failed to fetch SLA compliance")
    })?;

    let met_sla = stats.met_response_sla + stats.met_resolution_sla;
    let breached_sla = stats.breached_response_sla + stats.breached_resolution_sla;
    let total_tracked = met_sla + breached_sla;

    let compliance_rate = if total_tracked > 0 {
        Decimal::from(met_sla) / Decimal::from(total_tracked) * Decimal::from(100)
    } else {
        Decimal::from(100)
    };

    let first_response_compliance = if stats.met_response_sla + stats.breached_response_sla > 0 {
        Decimal::from(stats.met_response_sla) /
            Decimal::from(stats.met_response_sla + stats.breached_response_sla) *
            Decimal::from(100)
    } else {
        Decimal::from(100)
    };

    let resolution_compliance = if stats.met_resolution_sla + stats.breached_resolution_sla > 0 {
        Decimal::from(stats.met_resolution_sla) /
            Decimal::from(stats.met_resolution_sla + stats.breached_resolution_sla) *
            Decimal::from(100)
    } else {
        Decimal::from(100)
    };

    Ok(Json(SlaComplianceSummary {
        period_start: from_date,
        period_end: to_date,
        total_tickets: stats.total_tickets,
        tickets_with_sla: stats.tickets_with_sla,
        tickets_met_sla: met_sla,
        tickets_breached_sla: breached_sla,
        compliance_rate,
        first_response_compliance,
        resolution_compliance,
        by_priority: vec![],
        by_client: vec![],
        trends: vec![],
        recent_breaches: vec![],
    }))
}

async fn get_sla_by_priority(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<Vec<SlaPriorityBreakdown>>> {
    let (from_date, to_date) = params.get_range();

    let priorities = sqlx::query!(
        r#"SELECT
            priority,
            COUNT(*) as "total_tickets!",
            COUNT(*) FILTER (WHERE
                (sla_response_at IS NULL OR sla_response_at <= sla_response_due) AND
                (resolved_at IS NULL OR resolved_at <= sla_resolution_due)
            ) as "met_sla!",
            COUNT(*) FILTER (WHERE
                (sla_response_at IS NOT NULL AND sla_response_at > sla_response_due) OR
                (resolved_at IS NOT NULL AND resolved_at > sla_resolution_due)
            ) as "breached_sla!",
            COALESCE(AVG(EXTRACT(EPOCH FROM (sla_response_at - created_at))/60)::bigint, 0) as "avg_response_time!"
         FROM tickets
         WHERE created_at::date >= $1
           AND created_at::date <= $2
           AND sla_policy_id IS NOT NULL
         GROUP BY priority
         ORDER BY priority"#,
        from_date,
        to_date
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching SLA by priority: {}", e);
        ApiError::internal("Failed to fetch SLA by priority")
    })?;

    let result: Vec<SlaPriorityBreakdown> = priorities
        .into_iter()
        .map(|row| {
            let total = row.met_sla + row.breached_sla;
            let rate = if total > 0 {
                Decimal::from(row.met_sla) / Decimal::from(total) * Decimal::from(100)
            } else {
                Decimal::from(100)
            };

            SlaPriorityBreakdown {
                priority: row.priority,
                total_tickets: row.total_tickets,
                met_sla: row.met_sla,
                breached_sla: row.breached_sla,
                compliance_rate: rate,
                avg_response_time_minutes: row.avg_response_time,
                avg_resolution_time_minutes: 0, // Would calculate similarly
            }
        })
        .collect();

    Ok(Json(result))
}

async fn get_sla_by_client(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<Vec<SlaClientBreakdown>>> {
    let (from_date, to_date) = params.get_range();

    let clients = sqlx::query!(
        r#"SELECT
            c.id as client_id,
            c.name as client_name,
            COUNT(*) as "total_tickets!",
            COUNT(*) FILTER (WHERE
                (t.sla_response_at IS NULL OR t.sla_response_at <= t.sla_response_due) AND
                (t.resolved_at IS NULL OR t.resolved_at <= t.sla_resolution_due)
            ) as "met_sla!",
            COUNT(*) FILTER (WHERE
                (t.sla_response_at IS NOT NULL AND t.sla_response_at > t.sla_response_due) OR
                (t.resolved_at IS NOT NULL AND t.resolved_at > t.sla_resolution_due)
            ) as "breached_sla!"
         FROM tickets t
         JOIN clients c ON t.client_id = c.id
         WHERE t.created_at::date >= $1
           AND t.created_at::date <= $2
           AND t.sla_policy_id IS NOT NULL
         GROUP BY c.id, c.name
         ORDER BY "breached_sla!" DESC"#,
        from_date,
        to_date
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching SLA by client: {}", e);
        ApiError::internal("Failed to fetch SLA by client")
    })?;

    let result: Vec<SlaClientBreakdown> = clients
        .into_iter()
        .map(|row| {
            let total = row.met_sla + row.breached_sla;
            let rate = if total > 0 {
                Decimal::from(row.met_sla) / Decimal::from(total) * Decimal::from(100)
            } else {
                Decimal::from(100)
            };

            SlaClientBreakdown {
                client_id: row.client_id,
                client_name: row.client_name,
                total_tickets: row.total_tickets,
                met_sla: row.met_sla,
                breached_sla: row.breached_sla,
                compliance_rate: rate,
            }
        })
        .collect();

    Ok(Json(result))
}

async fn get_sla_by_technician(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<Vec<SlaPerformanceByTechnician>>> {
    let (from_date, to_date) = params.get_range();

    let techs = sqlx::query!(
        r#"SELECT
            u.id as user_id,
            u.first_name || ' ' || u.last_name as user_name,
            COUNT(*) as "total_assigned!",
            COUNT(*) FILTER (WHERE
                (t.sla_response_at IS NULL OR t.sla_response_at <= t.sla_response_due) AND
                (t.resolved_at IS NULL OR t.resolved_at <= t.sla_resolution_due)
            ) as "met_sla!",
            COUNT(*) FILTER (WHERE
                (t.sla_response_at IS NOT NULL AND t.sla_response_at > t.sla_response_due) OR
                (t.resolved_at IS NOT NULL AND t.resolved_at > t.sla_resolution_due)
            ) as "breached_sla!",
            COALESCE(AVG(EXTRACT(EPOCH FROM (t.sla_response_at - t.created_at))/60)::bigint, 0) as "avg_response_time!"
         FROM tickets t
         JOIN users u ON t.assigned_to = u.id
         WHERE t.created_at::date >= $1
           AND t.created_at::date <= $2
           AND t.sla_policy_id IS NOT NULL
         GROUP BY u.id, u.first_name, u.last_name
         ORDER BY "breached_sla!" DESC"#,
        from_date,
        to_date
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching SLA by technician: {}", e);
        ApiError::internal("Failed to fetch SLA by technician")
    })?;

    let result: Vec<SlaPerformanceByTechnician> = techs
        .into_iter()
        .map(|row| {
            let total = row.met_sla + row.breached_sla;
            let rate = if total > 0 {
                Decimal::from(row.met_sla) / Decimal::from(total) * Decimal::from(100)
            } else {
                Decimal::from(100)
            };

            SlaPerformanceByTechnician {
                user_id: row.user_id,
                user_name: row.user_name.unwrap_or_else(|| "Unknown".to_string()),
                total_assigned: row.total_assigned,
                met_sla: row.met_sla,
                breached_sla: row.breached_sla,
                compliance_rate: rate,
                avg_response_time_minutes: row.avg_response_time,
                avg_resolution_time_minutes: 0,
            }
        })
        .collect();

    Ok(Json(result))
}

async fn get_sla_breaches(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<Vec<SlaBreachDetail>>> {
    let (from_date, to_date) = params.get_range();

    let breaches = sqlx::query!(
        r#"SELECT
            t.id as ticket_id,
            t.number as ticket_number,
            t.subject,
            t.client_id,
            c.name as client_name,
            t.priority,
            t.sla_response_due,
            t.sla_response_at,
            t.sla_resolution_due,
            t.resolved_at,
            u.first_name || ' ' || u.last_name as assigned_to
         FROM tickets t
         JOIN clients c ON t.client_id = c.id
         LEFT JOIN users u ON t.assigned_to = u.id
         WHERE t.created_at::date >= $1
           AND t.created_at::date <= $2
           AND (
               (t.sla_response_at IS NOT NULL AND t.sla_response_at > t.sla_response_due) OR
               (t.resolved_at IS NOT NULL AND t.resolved_at > t.sla_resolution_due)
           )
         ORDER BY t.created_at DESC
         LIMIT 50"#,
        from_date,
        to_date
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching SLA breaches: {}", e);
        ApiError::internal("Failed to fetch SLA breaches")
    })?;

    let result: Vec<SlaBreachDetail> = breaches
        .into_iter()
        .filter_map(|row| {
            // Determine breach type
            let (breach_type, expected_at, actual_at) = if let (Some(due), Some(actual)) =
                (row.sla_response_due, row.sla_response_at)
            {
                if actual > due {
                    ("first_response", due, Some(actual))
                } else if let (Some(res_due), Some(res_actual)) =
                    (row.sla_resolution_due, row.resolved_at)
                {
                    if res_actual > res_due {
                        ("resolution", res_due, Some(res_actual))
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else if let (Some(res_due), Some(res_actual)) =
                (row.sla_resolution_due, row.resolved_at)
            {
                if res_actual > res_due {
                    ("resolution", res_due, Some(res_actual))
                } else {
                    return None;
                }
            } else {
                return None;
            };

            let breach_duration = actual_at
                .map(|a| (a - expected_at).num_minutes())
                .unwrap_or(0);

            Some(SlaBreachDetail {
                ticket_id: row.ticket_id,
                ticket_number: row.ticket_number,
                subject: row.subject,
                client_id: row.client_id,
                client_name: row.client_name,
                priority: row.priority,
                breach_type: breach_type.to_string(),
                expected_at,
                actual_at,
                breach_duration_minutes: breach_duration,
                assigned_to: row.assigned_to,
            })
        })
        .collect();

    Ok(Json(result))
}

async fn get_sla_trend(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<Vec<SlaComplianceTrend>>> {
    let (from_date, to_date) = params.get_range();

    let trends = sqlx::query!(
        r#"SELECT
            t.created_at::date as "date!",
            COUNT(*) as "total_tickets!",
            COUNT(*) FILTER (WHERE
                (t.sla_response_at IS NULL OR t.sla_response_at <= t.sla_response_due) AND
                (t.resolved_at IS NULL OR t.resolved_at <= t.sla_resolution_due)
            ) as "met_sla!"
         FROM tickets t
         WHERE t.created_at::date >= $1
           AND t.created_at::date <= $2
           AND t.sla_policy_id IS NOT NULL
         GROUP BY t.created_at::date
         ORDER BY "date!" ASC"#,
        from_date,
        to_date
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching SLA trend: {}", e);
        ApiError::internal("Failed to fetch SLA trend")
    })?;

    let result: Vec<SlaComplianceTrend> = trends
        .into_iter()
        .map(|row| {
            let rate = if row.total_tickets > 0 {
                Decimal::from(row.met_sla) / Decimal::from(row.total_tickets) * Decimal::from(100)
            } else {
                Decimal::from(100)
            };

            SlaComplianceTrend {
                date: row.date,
                total_tickets: row.total_tickets,
                met_sla: row.met_sla,
                compliance_rate: rate,
            }
        })
        .collect();

    Ok(Json(result))
}

// ==================== Executive Summary ====================

#[derive(Debug, Clone, Serialize)]
pub struct ExecutiveSummary {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,

    // Financial
    pub total_revenue: Decimal,
    pub total_cost: Decimal,
    pub gross_profit: Decimal,
    pub gross_margin: Decimal,
    pub revenue_change: Decimal,

    // Operations
    pub total_tickets: i64,
    pub tickets_resolved: i64,
    pub resolution_rate: Decimal,
    pub avg_resolution_hours: Decimal,

    // Team
    pub avg_utilization: Decimal,
    pub total_billable_hours: Decimal,
    pub top_performers: Vec<String>,

    // SLA
    pub sla_compliance_rate: Decimal,
    pub sla_breaches: i64,

    // Clients
    pub total_clients: i64,
    pub clients_at_risk: i64,
    pub avg_client_health: i32,
}

async fn get_executive_summary(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(params): Query<DateRangeQuery>,
) -> ApiResult<Json<ExecutiveSummary>> {
    let (from_date, to_date) = params.get_range();
    let cost_per_hour = Decimal::from(50);

    // Financial metrics
    let financial = sqlx::query!(
        r#"SELECT
            COALESCE(SUM(total), 0) as "revenue!",
            COUNT(DISTINCT client_id) as "client_count!"
         FROM invoices
         WHERE date >= $1 AND date <= $2"#,
        from_date,
        to_date
    )
    .fetch_one(&state.db_pool)
    .await?;

    // Time/cost metrics
    let time_stats = sqlx::query!(
        r#"SELECT
            COALESCE(SUM(duration_minutes), 0)::decimal / 60.0 as "total_hours!",
            COALESCE(SUM(duration_minutes) FILTER (WHERE billable), 0)::decimal / 60.0 as "billable_hours!",
            COUNT(DISTINCT user_id) as "tech_count!"
         FROM time_entries
         WHERE start_time::date >= $1
           AND start_time::date <= $2
           AND end_time IS NOT NULL"#,
        from_date,
        to_date
    )
    .fetch_one(&state.db_pool)
    .await?;

    // Ticket metrics
    let ticket_stats = sqlx::query!(
        r#"SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE resolved_at IS NOT NULL) as "resolved!",
            COUNT(*) FILTER (WHERE
                (sla_response_at IS NOT NULL AND sla_response_at > sla_response_due) OR
                (resolved_at IS NOT NULL AND resolved_at > sla_resolution_due)
            ) as "sla_breaches!"
         FROM tickets
         WHERE created_at::date >= $1
           AND created_at::date <= $2"#,
        from_date,
        to_date
    )
    .fetch_one(&state.db_pool)
    .await?;

    let total_cost = time_stats.total_hours * cost_per_hour;
    let gross_profit = financial.revenue - total_cost;
    let gross_margin = if financial.revenue > Decimal::ZERO {
        (gross_profit / financial.revenue) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let resolution_rate = if ticket_stats.total > 0 {
        Decimal::from(ticket_stats.resolved) / Decimal::from(ticket_stats.total) * Decimal::from(100)
    } else {
        Decimal::from(100)
    };

    let sla_met = ticket_stats.total - ticket_stats.sla_breaches;
    let sla_compliance_rate = if ticket_stats.total > 0 {
        Decimal::from(sla_met) / Decimal::from(ticket_stats.total) * Decimal::from(100)
    } else {
        Decimal::from(100)
    };

    // Calculate utilization (assuming 8 hours/day target per technician)
    let days = (to_date - from_date).num_days() as i64;
    let work_days = days * 5 / 7;
    let target_hours = Decimal::from(work_days * 8 * time_stats.tech_count);
    let avg_utilization = if target_hours > Decimal::ZERO {
        (time_stats.billable_hours / target_hours) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    Ok(Json(ExecutiveSummary {
        period_start: from_date,
        period_end: to_date,
        total_revenue: financial.revenue,
        total_cost,
        gross_profit,
        gross_margin,
        revenue_change: Decimal::ZERO, // Would compare to previous period
        total_tickets: ticket_stats.total,
        tickets_resolved: ticket_stats.resolved,
        resolution_rate,
        avg_resolution_hours: Decimal::ZERO, // Would calculate
        avg_utilization,
        total_billable_hours: time_stats.billable_hours,
        top_performers: vec![],
        sla_compliance_rate,
        sla_breaches: ticket_stats.sla_breaches,
        total_clients: financial.client_count,
        clients_at_risk: 0, // Would calculate from profitability
        avg_client_health: 75, // Would calculate from health scores
    }))
}
