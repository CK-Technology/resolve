use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, patch},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct TicketCreate {
    pub client_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub subject: String,
    pub details: String,
    pub priority: Option<String>,
    pub source: Option<String>,
    pub billable: Option<bool>,
    pub estimated_hours: Option<rust_decimal::Decimal>,
}

#[derive(Serialize, Deserialize)]
pub struct TicketUpdate {
    pub subject: Option<String>,
    pub details: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub billable: Option<bool>,
    pub estimated_hours: Option<rust_decimal::Decimal>,
}

#[derive(Serialize, Deserialize)]
pub struct TicketReplyCreate {
    pub details: String,
    pub reply_type: Option<String>, // reply, note, status_change
    pub time_worked: Option<i32>,   // minutes
    pub billable: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct TicketQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub sla_breached: Option<bool>,
    pub search: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TicketWithDetails {
    pub id: Uuid,
    pub number: i32,
    pub client_id: Uuid,
    pub client_name: String,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub asset_id: Option<Uuid>,
    pub asset_name: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_name: Option<String>,
    pub opened_by: Uuid,
    pub opened_by_name: String,
    pub subject: String,
    pub details: String,
    pub status: String,
    pub priority: String,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub sla_id: Option<Uuid>,
    pub response_due_at: Option<DateTime<Utc>>,
    pub resolution_due_at: Option<DateTime<Utc>>,
    pub first_response_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub sla_breached: bool,
    pub billable: bool,
    pub estimated_hours: Option<rust_decimal::Decimal>,
    pub actual_hours: Option<rust_decimal::Decimal>,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct TicketReply {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub reply_type: String,
    pub details: String,
    pub time_worked: i32,
    pub billable: bool,
    pub created_at: DateTime<Utc>,
}

pub fn ticket_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_tickets).post(create_ticket))
        .route("/:id", get(get_ticket).put(update_ticket))
        .route("/:id/assign", patch(assign_ticket))
        .route("/:id/escalate", patch(escalate_ticket))
        .route("/:id/replies", get(get_ticket_replies).post(add_reply))
        .route("/:id/replies/:reply_id", put(update_reply))
        .route("/categories", get(get_categories))
        .route("/stats", get(get_ticket_stats))
}

async fn list_tickets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TicketQuery>,
) -> Result<Json<Vec<TicketWithDetails>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let mut where_clauses = vec!["1=1".to_string()];
    let mut param_count = 1;
    
    if let Some(status) = &params.status {
        where_clauses.push(format!("t.status = ${}", param_count));
        param_count += 1;
    }
    
    if let Some(priority) = &params.priority {
        where_clauses.push(format!("t.priority = ${}", param_count));
        param_count += 1;
    }
    
    if let Some(assigned_to) = &params.assigned_to {
        where_clauses.push(format!("t.assigned_to = ${}", param_count));
        param_count += 1;
    }
    
    if let Some(client_id) = &params.client_id {
        where_clauses.push(format!("t.client_id = ${}", param_count));
        param_count += 1;
    }
    
    if let Some(category_id) = &params.category_id {
        where_clauses.push(format!("t.category_id = ${}", param_count));
        param_count += 1;
    }
    
    if let Some(sla_breached) = &params.sla_breached {
        where_clauses.push(format!("t.sla_breached = ${}", param_count));
        param_count += 1;
    }
    
    if let Some(search) = &params.search {
        where_clauses.push(format!("(t.subject ILIKE ${} OR t.details ILIKE ${})", param_count, param_count));
        param_count += 1;
    }
    
    let where_clause = where_clauses.join(" AND ");
    
    let query = format!(
        "SELECT 
            t.id, t.number, t.client_id, c.name as client_name,
            t.contact_id, ct.name as contact_name,
            t.asset_id, a.name as asset_name,
            t.assigned_to, u1.first_name || ' ' || u1.last_name as assigned_name,
            t.opened_by, u2.first_name || ' ' || u2.last_name as opened_by_name,
            t.subject, t.details, t.status, t.priority,
            t.category_id, tc.name as category_name,
            t.sla_id, t.response_due_at, t.resolution_due_at,
            t.first_response_at, t.resolved_at, t.sla_breached,
            t.billable, t.estimated_hours, t.actual_hours, t.source,
            t.created_at, t.updated_at, t.closed_at
         FROM tickets t
         LEFT JOIN clients c ON t.client_id = c.id
         LEFT JOIN contacts ct ON t.contact_id = ct.id
         LEFT JOIN assets a ON t.asset_id = a.id
         LEFT JOIN users u1 ON t.assigned_to = u1.id
         LEFT JOIN users u2 ON t.opened_by = u2.id
         LEFT JOIN ticket_categories tc ON t.category_id = tc.id
         WHERE {}
         ORDER BY t.created_at DESC
         LIMIT ${} OFFSET ${}",
        where_clause, param_count, param_count + 1
    );
    
    // This is a simplified implementation - in production you'd use a query builder
    // For now, let's return a basic query result
    match sqlx::query_as!(
        TicketWithDetails,
        "SELECT 
            t.id, t.number, t.client_id, c.name as client_name,
            t.contact_id, ct.name as contact_name,
            t.asset_id, a.name as asset_name,
            t.assigned_to, 
            CASE WHEN u1.id IS NOT NULL THEN u1.first_name || ' ' || u1.last_name ELSE NULL END as assigned_name,
            t.opened_by, u2.first_name || ' ' || u2.last_name as opened_by_name,
            t.subject, t.details, t.status, t.priority,
            t.category_id, tc.name as category_name,
            t.sla_id, t.response_due_at, t.resolution_due_at,
            t.first_response_at, t.resolved_at, t.sla_breached,
            t.billable, t.estimated_hours, t.actual_hours, t.source,
            t.created_at, t.updated_at, t.closed_at
         FROM tickets t
         LEFT JOIN clients c ON t.client_id = c.id
         LEFT JOIN contacts ct ON t.contact_id = ct.id
         LEFT JOIN assets a ON t.asset_id = a.id
         LEFT JOIN users u1 ON t.assigned_to = u1.id
         LEFT JOIN users u2 ON t.opened_by = u2.id
         LEFT JOIN ticket_categories tc ON t.category_id = tc.id
         ORDER BY t.created_at DESC
         LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(tickets) => Ok(Json(tickets)),
        Err(e) => {
            tracing::error!("Error fetching tickets: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TicketCreate>,
) -> Result<(StatusCode, Json<TicketWithDetails>), StatusCode> {
    let ticket_id = Uuid::new_v4();
    
    // Get the next ticket number
    let next_number = match sqlx::query_scalar!(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM tickets"
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(num) => num.unwrap_or(1),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // TODO: Calculate SLA due dates based on client contract
    // For now, set basic defaults
    let now = Utc::now();
    let response_due = now + chrono::Duration::hours(4); // 4 hour response SLA
    let resolution_due = now + chrono::Duration::hours(24); // 24 hour resolution SLA
    
    let priority = payload.priority.unwrap_or_else(|| "medium".to_string());
    let source = payload.source.unwrap_or_else(|| "manual".to_string());
    let billable = payload.billable.unwrap_or(true);
    
    // TODO: Get current user from auth context - for now use a dummy UUID
    let current_user_id = Uuid::new_v4();
    
    match sqlx::query!(
        "INSERT INTO tickets (
            id, number, client_id, contact_id, asset_id, category_id,
            subject, details, status, priority, source, billable,
            estimated_hours, opened_by, response_due_at, resolution_due_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
        ticket_id,
        next_number,
        payload.client_id,
        payload.contact_id,
        payload.asset_id,
        payload.category_id,
        payload.subject,
        payload.details,
        "open",
        priority,
        source,
        billable,
        payload.estimated_hours,
        current_user_id,
        response_due,
        resolution_due
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(_) => {
            // Fetch the created ticket with all details
            match get_ticket_by_id(&state, ticket_id).await {
                Ok(ticket) => Ok((StatusCode::CREATED, Json(ticket))),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(e) => {
            tracing::error!("Error creating ticket: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<TicketWithDetails>, StatusCode> {
    match get_ticket_by_id(&state, id).await {
        Ok(ticket) => Ok(Json(ticket)),
        Err(StatusCode::NOT_FOUND) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_ticket_by_id(state: &AppState, id: Uuid) -> Result<TicketWithDetails, StatusCode> {
    match sqlx::query_as!(
        TicketWithDetails,
        "SELECT 
            t.id, t.number, t.client_id, c.name as client_name,
            t.contact_id, ct.name as contact_name,
            t.asset_id, a.name as asset_name,
            t.assigned_to, 
            CASE WHEN u1.id IS NOT NULL THEN u1.first_name || ' ' || u1.last_name ELSE NULL END as assigned_name,
            t.opened_by, u2.first_name || ' ' || u2.last_name as opened_by_name,
            t.subject, t.details, t.status, t.priority,
            t.category_id, tc.name as category_name,
            t.sla_id, t.response_due_at, t.resolution_due_at,
            t.first_response_at, t.resolved_at, t.sla_breached,
            t.billable, t.estimated_hours, t.actual_hours, t.source,
            t.created_at, t.updated_at, t.closed_at
         FROM tickets t
         LEFT JOIN clients c ON t.client_id = c.id
         LEFT JOIN contacts ct ON t.contact_id = ct.id
         LEFT JOIN assets a ON t.asset_id = a.id
         LEFT JOIN users u1 ON t.assigned_to = u1.id
         LEFT JOIN users u2 ON t.opened_by = u2.id
         LEFT JOIN ticket_categories tc ON t.category_id = tc.id
         WHERE t.id = $1",
        id
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(ticket) => Ok(ticket),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Error fetching ticket: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TicketUpdate>,
) -> Result<Json<TicketWithDetails>, StatusCode> {
    // Update ticket - simplified version
    match sqlx::query!(
        "UPDATE tickets SET 
         subject = COALESCE($2, subject),
         details = COALESCE($3, details),
         status = COALESCE($4, status),
         priority = COALESCE($5, priority),
         assigned_to = COALESCE($6, assigned_to),
         category_id = COALESCE($7, category_id),
         billable = COALESCE($8, billable),
         estimated_hours = COALESCE($9, estimated_hours),
         updated_at = NOW()
         WHERE id = $1",
        id,
        payload.subject,
        payload.details,
        payload.status,
        payload.priority,
        payload.assigned_to,
        payload.category_id,
        payload.billable,
        payload.estimated_hours
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                match get_ticket_by_id(&state, id).await {
                    Ok(ticket) => Ok(Json(ticket)),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Error updating ticket: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn assign_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TicketWithDetails>, StatusCode> {
    let assigned_to = payload.get("assigned_to")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    
    match sqlx::query!(
        "UPDATE tickets SET assigned_to = $2, updated_at = NOW() WHERE id = $1",
        id,
        assigned_to
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                match get_ticket_by_id(&state, id).await {
                    Ok(ticket) => Ok(Json(ticket)),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Error assigning ticket: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn escalate_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TicketWithDetails>, StatusCode> {
    let escalated_to = payload.get("escalated_to")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    
    match sqlx::query!(
        "UPDATE tickets SET 
         escalated_to = $2, 
         escalated_at = NOW(),
         priority = CASE WHEN priority = 'low' THEN 'medium'
                         WHEN priority = 'medium' THEN 'high'
                         WHEN priority = 'high' THEN 'critical'
                         ELSE priority END,
         updated_at = NOW() 
         WHERE id = $1",
        id,
        escalated_to
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                match get_ticket_by_id(&state, id).await {
                    Ok(ticket) => Ok(Json(ticket)),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Error escalating ticket: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_ticket_replies(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<TicketReply>>, StatusCode> {
    match sqlx::query_as!(
        TicketReply,
        "SELECT 
            tr.id, tr.ticket_id, tr.user_id, 
            u.first_name || ' ' || u.last_name as user_name,
            tr.type as reply_type, tr.details, tr.time_worked, tr.billable,
            tr.created_at
         FROM ticket_replies tr
         LEFT JOIN users u ON tr.user_id = u.id
         WHERE tr.ticket_id = $1
         ORDER BY tr.created_at ASC",
        id
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(replies) => Ok(Json(replies)),
        Err(e) => {
            tracing::error!("Error fetching ticket replies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn add_reply(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TicketReplyCreate>,
) -> Result<(StatusCode, Json<TicketReply>), StatusCode> {
    let reply_id = Uuid::new_v4();
    // TODO: Get current user from auth context
    let current_user_id = Uuid::new_v4();
    
    let reply_type = payload.reply_type.unwrap_or_else(|| "reply".to_string());
    let time_worked = payload.time_worked.unwrap_or(0);
    let billable = payload.billable.unwrap_or(false);
    
    match sqlx::query!(
        "INSERT INTO ticket_replies (
            id, ticket_id, user_id, type, details, time_worked, billable
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        reply_id,
        id,
        current_user_id,
        reply_type,
        payload.details,
        time_worked,
        billable
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(_) => {
            // Update first response time if this is the first reply
            if reply_type == "reply" {
                let _ = sqlx::query!(
                    "UPDATE tickets SET 
                     first_response_at = COALESCE(first_response_at, NOW()),
                     updated_at = NOW()
                     WHERE id = $1",
                    id
                ).execute(&state.db_pool).await;
            }
            
            // Fetch the created reply
            match sqlx::query_as!(
                TicketReply,
                "SELECT 
                    tr.id, tr.ticket_id, tr.user_id, 
                    u.first_name || ' ' || u.last_name as user_name,
                    tr.type as reply_type, tr.details, tr.time_worked, tr.billable,
                    tr.created_at
                 FROM ticket_replies tr
                 LEFT JOIN users u ON tr.user_id = u.id
                 WHERE tr.id = $1",
                reply_id
            )
            .fetch_one(&state.db_pool)
            .await
            {
                Ok(reply) => Ok((StatusCode::CREATED, Json(reply))),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(e) => {
            tracing::error!("Error creating ticket reply: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_reply(
    State(state): State<Arc<AppState>>,
    Path((ticket_id, reply_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<TicketReplyCreate>,
) -> Result<Json<TicketReply>, StatusCode> {
    match sqlx::query!(
        "UPDATE ticket_replies SET 
         details = $3,
         time_worked = COALESCE($4, time_worked),
         billable = COALESCE($5, billable)
         WHERE id = $1 AND ticket_id = $2",
        reply_id,
        ticket_id,
        payload.details,
        payload.time_worked,
        payload.billable
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                match sqlx::query_as!(
                    TicketReply,
                    "SELECT 
                        tr.id, tr.ticket_id, tr.user_id, 
                        u.first_name || ' ' || u.last_name as user_name,
                        tr.type as reply_type, tr.details, tr.time_worked, tr.billable,
                        tr.created_at
                     FROM ticket_replies tr
                     LEFT JOIN users u ON tr.user_id = u.id
                     WHERE tr.id = $1",
                    reply_id
                )
                .fetch_one(&state.db_pool)
                .await
                {
                    Ok(reply) => Ok(Json(reply)),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Error updating ticket reply: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<resolve_shared::TicketCategory>>, StatusCode> {
    match sqlx::query_as!(
        resolve_shared::TicketCategory,
        "SELECT id, name, color, default_priority, default_sla_id, created_at 
         FROM ticket_categories 
         ORDER BY name"
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(categories) => Ok(Json(categories)),
        Err(e) => {
            tracing::error!("Error fetching ticket categories: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
pub struct TicketStats {
    pub total_tickets: i64,
    pub open_tickets: i64,
    pub overdue_tickets: i64,
    pub sla_breached: i64,
    pub avg_response_time_hours: Option<f64>,
    pub avg_resolution_time_hours: Option<f64>,
}

async fn get_ticket_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TicketStats>, StatusCode> {
    // This would be more complex in production with proper date ranges, filters, etc.
    let stats = match sqlx::query!(
        "SELECT 
            COUNT(*) as total_tickets,
            COUNT(*) FILTER (WHERE status IN ('open', 'in_progress')) as open_tickets,
            COUNT(*) FILTER (WHERE resolution_due_at < NOW() AND status NOT IN ('closed', 'resolved')) as overdue_tickets,
            COUNT(*) FILTER (WHERE sla_breached = true) as sla_breached
         FROM tickets"
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(row) => TicketStats {
            total_tickets: row.total_tickets.unwrap_or(0),
            open_tickets: row.open_tickets.unwrap_or(0),
            overdue_tickets: row.overdue_tickets.unwrap_or(0),
            sla_breached: row.sla_breached.unwrap_or(0),
            avg_response_time_hours: None, // TODO: Calculate actual averages
            avg_resolution_time_hours: None,
        },
        Err(e) => {
            tracing::error!("Error fetching ticket stats: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    Ok(Json(stats))
}