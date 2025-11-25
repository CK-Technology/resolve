//! Microsoft Teams Integration Handlers
//!
//! API endpoints for managing Teams webhook configurations and sending notifications.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use crate::{
    AppState, ApiResult, ApiError,
    PaginatedResponse, PaginationParams,
};
use crate::auth::middleware::AuthUser;
use crate::services::teams_integration::{
    TeamsNotificationService, TicketNotification, DailySummary,
};

// ==================== Structs ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TeamsIntegration {
    pub id: Uuid,
    pub name: String,
    pub webhook_url: String,
    pub channel_name: Option<String>,
    pub is_active: bool,
    // Notification preferences
    pub notify_ticket_created: bool,
    pub notify_ticket_assigned: bool,
    pub notify_ticket_resolved: bool,
    pub notify_sla_breach: bool,
    pub notify_daily_summary: bool,
    // Filters
    pub filter_priorities: Option<Vec<String>>, // only notify for these priorities
    pub filter_queue_ids: Option<Vec<Uuid>>, // only notify for these queues
    pub filter_client_ids: Option<Vec<Uuid>>, // only notify for these clients
    // Stats
    pub last_notification_at: Option<DateTime<Utc>>,
    pub notification_count: i32,
    pub error_count: i32,
    pub last_error: Option<String>,
    // Metadata
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamsIntegrationRequest {
    pub name: String,
    pub webhook_url: String,
    pub channel_name: Option<String>,
    pub notify_ticket_created: Option<bool>,
    pub notify_ticket_assigned: Option<bool>,
    pub notify_ticket_resolved: Option<bool>,
    pub notify_sla_breach: Option<bool>,
    pub notify_daily_summary: Option<bool>,
    pub filter_priorities: Option<Vec<String>>,
    pub filter_queue_ids: Option<Vec<Uuid>>,
    pub filter_client_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct SendTestNotificationRequest {
    pub notification_type: String, // ticket_created, sla_breach, daily_summary
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TeamsNotificationLog {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub notification_type: String,
    pub ticket_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub success: bool,
    pub error_message: Option<String>,
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationLogQuery {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub integration_id: Option<Uuid>,
    pub success: Option<bool>,
}

// ==================== Routes ====================

pub fn teams_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_integrations).post(create_integration))
        .route("/:id", get(get_integration).put(update_integration).delete(delete_integration))
        .route("/:id/test", post(send_test_notification))
        .route("/:id/toggle", post(toggle_integration))
        .route("/logs", get(list_notification_logs))
        .route("/notify/ticket/:ticket_id", post(notify_ticket))
        .route("/notify/daily-summary", post(send_daily_summary))
}

// ==================== Handlers ====================

async fn list_integrations(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<Vec<TeamsIntegration>>> {
    let integrations = sqlx::query_as!(
        TeamsIntegration,
        r#"SELECT
            id, name, webhook_url, channel_name, is_active,
            notify_ticket_created, notify_ticket_assigned, notify_ticket_resolved,
            notify_sla_breach, notify_daily_summary,
            filter_priorities, filter_queue_ids, filter_client_ids,
            last_notification_at, notification_count, error_count, last_error,
            created_by, created_at, updated_at
         FROM notification_integrations
         WHERE integration_type = 'teams'
         ORDER BY name"#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching Teams integrations: {}", e);
        ApiError::internal("Failed to fetch Teams integrations")
    })?;

    Ok(Json(integrations))
}

async fn create_integration(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateTeamsIntegrationRequest>,
) -> ApiResult<Json<TeamsIntegration>> {
    // Validate webhook URL format
    if !payload.webhook_url.contains("webhook.office.com") &&
       !payload.webhook_url.contains("outlook.office.com") {
        return Err(ApiError::validation_single(
            "webhook_url",
            "Invalid Teams webhook URL. Must be a Microsoft Teams webhook URL."
        ));
    }

    let id = Uuid::new_v4();

    sqlx::query!(
        r#"INSERT INTO notification_integrations (
            id, name, integration_type, webhook_url, channel_id,
            notify_on, queue_ids, is_active, created_by
        ) VALUES (
            $1, $2, 'teams', $3, $4,
            jsonb_build_object(
                'ticket_created', $5,
                'ticket_assigned', $6,
                'ticket_resolved', $7,
                'sla_breach', $8,
                'daily_summary', $9
            ),
            $10, true, $11
        )"#,
        id,
        payload.name,
        payload.webhook_url,
        payload.channel_name,
        payload.notify_ticket_created.unwrap_or(true),
        payload.notify_ticket_assigned.unwrap_or(true),
        payload.notify_ticket_resolved.unwrap_or(true),
        payload.notify_sla_breach.unwrap_or(true),
        payload.notify_daily_summary.unwrap_or(false),
        payload.filter_queue_ids.as_deref(),
        user.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating Teams integration: {}", e);
        ApiError::internal("Failed to create Teams integration")
    })?;

    // Fetch the created integration
    let integration = get_integration_by_id(&state, id).await?;
    Ok(Json(integration))
}

async fn get_integration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TeamsIntegration>> {
    let integration = get_integration_by_id(&state, id).await?;
    Ok(Json(integration))
}

async fn update_integration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateTeamsIntegrationRequest>,
) -> ApiResult<Json<TeamsIntegration>> {
    sqlx::query!(
        r#"UPDATE notification_integrations SET
            name = $2, webhook_url = $3, channel_id = $4,
            notify_on = jsonb_build_object(
                'ticket_created', $5,
                'ticket_assigned', $6,
                'ticket_resolved', $7,
                'sla_breach', $8,
                'daily_summary', $9
            ),
            queue_ids = $10,
            updated_at = NOW()
           WHERE id = $1 AND integration_type = 'teams'"#,
        id,
        payload.name,
        payload.webhook_url,
        payload.channel_name,
        payload.notify_ticket_created.unwrap_or(true),
        payload.notify_ticket_assigned.unwrap_or(true),
        payload.notify_ticket_resolved.unwrap_or(true),
        payload.notify_sla_breach.unwrap_or(true),
        payload.notify_daily_summary.unwrap_or(false),
        payload.filter_queue_ids.as_deref()
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating Teams integration: {}", e);
        ApiError::internal("Failed to update Teams integration")
    })?;

    let integration = get_integration_by_id(&state, id).await?;
    Ok(Json(integration))
}

async fn delete_integration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    sqlx::query!(
        "DELETE FROM notification_integrations WHERE id = $1 AND integration_type = 'teams'",
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error deleting Teams integration: {}", e);
        ApiError::internal("Failed to delete Teams integration")
    })?;

    Ok(())
}

async fn toggle_integration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TeamsIntegration>> {
    sqlx::query!(
        "UPDATE notification_integrations SET is_active = NOT is_active, updated_at = NOW() WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await?;

    let integration = get_integration_by_id(&state, id).await?;
    Ok(Json(integration))
}

async fn send_test_notification(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SendTestNotificationRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let integration = get_integration_by_id(&state, id).await?;
    let service = TeamsNotificationService::new();
    let portal_url = std::env::var("PORTAL_BASE_URL").unwrap_or_else(|_| "https://app.example.com".to_string());

    let result = match payload.notification_type.as_str() {
        "ticket_created" => {
            let test_ticket = TicketNotification {
                id: Uuid::new_v4(),
                number: 12345,
                subject: "Test Ticket - Integration Verification".to_string(),
                description: "This is a test notification to verify the Teams integration is working correctly.".to_string(),
                priority: "high".to_string(),
                status: "open".to_string(),
                client_id: Uuid::new_v4(),
                client_name: "Test Client".to_string(),
                category: Some("General".to_string()),
                assigned_to: None,
                created_at: Utc::now(),
            };
            service.notify_ticket_created(&integration.webhook_url, &test_ticket, &portal_url).await
        }
        "sla_breach" => {
            let test_ticket = TicketNotification {
                id: Uuid::new_v4(),
                number: 12345,
                subject: "Test SLA Breach Notification".to_string(),
                description: "This is a test SLA breach notification.".to_string(),
                priority: "critical".to_string(),
                status: "open".to_string(),
                client_id: Uuid::new_v4(),
                client_name: "Test Client".to_string(),
                category: Some("Support".to_string()),
                assigned_to: Some("Test Technician".to_string()),
                created_at: Utc::now(),
            };
            service.notify_sla_breach(&integration.webhook_url, &test_ticket, "First Response", 45, &portal_url).await
        }
        "daily_summary" => {
            let summary = DailySummary {
                date: Utc::now().date_naive(),
                tickets_created: 15,
                tickets_resolved: 12,
                open_tickets: 23,
                sla_breaches: 2,
                sla_compliance: 94.5,
                billable_hours: 48.5,
            };
            service.notify_daily_summary(&integration.webhook_url, &summary, &portal_url).await
        }
        _ => {
            service.send_simple_message(&integration.webhook_url, "Test notification from Resolve MSP Platform").await
        }
    };

    match result {
        Ok(_) => {
            // Update success stats
            sqlx::query!(
                "UPDATE notification_integrations SET last_notification_at = NOW(), notification_count = notification_count + 1 WHERE id = $1",
                id
            )
            .execute(&state.db_pool)
            .await?;

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Test notification sent successfully"
            })))
        }
        Err(e) => {
            // Update error stats
            sqlx::query!(
                "UPDATE notification_integrations SET error_count = error_count + 1, last_error = $2 WHERE id = $1",
                id,
                e.to_string()
            )
            .execute(&state.db_pool)
            .await?;

            Err(ApiError::internal(format!("Failed to send notification: {}", e)))
        }
    }
}

async fn notify_ticket(
    State(state): State<Arc<AppState>>,
    Path(ticket_id): Path<Uuid>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    // Fetch ticket details
    let ticket = sqlx::query!(
        r#"SELECT t.id, t.number, t.subject, t.description, t.priority, t.status,
                  t.client_id, c.name as client_name, tc.name as category_name,
                  u.first_name || ' ' || u.last_name as assigned_to_name, t.created_at
           FROM tickets t
           JOIN clients c ON t.client_id = c.id
           LEFT JOIN ticket_categories tc ON t.category_id = tc.id
           LEFT JOIN users u ON t.assigned_to = u.id
           WHERE t.id = $1"#,
        ticket_id
    )
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| ApiError::not_found("Ticket not found"))?;

    let ticket_notification = TicketNotification {
        id: ticket.id,
        number: ticket.number,
        subject: ticket.subject,
        description: ticket.description.unwrap_or_default(),
        priority: ticket.priority,
        status: ticket.status,
        client_id: ticket.client_id,
        client_name: ticket.client_name,
        category: ticket.category_name,
        assigned_to: ticket.assigned_to_name,
        created_at: ticket.created_at,
    };

    // Get all active Teams integrations that want this notification
    let integrations = sqlx::query!(
        r#"SELECT id, webhook_url
           FROM notification_integrations
           WHERE integration_type = 'teams'
             AND is_active = true
             AND (notify_on->>'ticket_created')::boolean = true"#
    )
    .fetch_all(&state.db_pool)
    .await?;

    let service = TeamsNotificationService::new();
    let portal_url = std::env::var("PORTAL_BASE_URL").unwrap_or_else(|_| "https://app.example.com".to_string());

    let mut sent_count = 0;
    let mut error_count = 0;

    for integration in integrations {
        if let Some(webhook_url) = integration.webhook_url {
            match service.notify_ticket_created(&webhook_url, &ticket_notification, &portal_url).await {
                Ok(_) => {
                    sent_count += 1;
                    let _ = log_notification(&state, integration.id, "ticket_created", Some(ticket_id), true, None).await;
                }
                Err(e) => {
                    error_count += 1;
                    let _ = log_notification(&state, integration.id, "ticket_created", Some(ticket_id), false, Some(e.to_string())).await;
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "sent": sent_count,
        "errors": error_count
    })))
}

async fn send_daily_summary(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    let today = Utc::now().date_naive();

    // Build summary from database
    let stats = sqlx::query!(
        r#"SELECT
            COUNT(*) FILTER (WHERE created_at::date = $1) as "tickets_created!",
            COUNT(*) FILTER (WHERE resolved_at::date = $1) as "tickets_resolved!",
            COUNT(*) FILTER (WHERE status NOT IN ('resolved', 'closed')) as "open_tickets!",
            COUNT(*) FILTER (WHERE
                created_at::date = $1 AND
                ((sla_response_at IS NOT NULL AND sla_response_at > sla_response_due) OR
                 (resolved_at IS NOT NULL AND resolved_at > sla_resolution_due))
            ) as "sla_breaches!"
         FROM tickets"#,
        today
    )
    .fetch_one(&state.db_pool)
    .await?;

    let time_stats = sqlx::query!(
        r#"SELECT COALESCE(SUM(duration_minutes) FILTER (WHERE billable), 0)::float8 / 60.0 as "billable_hours!"
           FROM time_entries WHERE start_time::date = $1 AND end_time IS NOT NULL"#,
        today
    )
    .fetch_one(&state.db_pool)
    .await?;

    let total_with_sla = stats.tickets_created.max(1);
    let met_sla = stats.tickets_created - stats.sla_breaches;
    let sla_compliance = (met_sla as f64 / total_with_sla as f64) * 100.0;

    let summary = DailySummary {
        date: today,
        tickets_created: stats.tickets_created,
        tickets_resolved: stats.tickets_resolved,
        open_tickets: stats.open_tickets,
        sla_breaches: stats.sla_breaches,
        sla_compliance,
        billable_hours: time_stats.billable_hours,
    };

    // Get all active Teams integrations that want daily summary
    let integrations = sqlx::query!(
        r#"SELECT id, webhook_url
           FROM notification_integrations
           WHERE integration_type = 'teams'
             AND is_active = true
             AND (notify_on->>'daily_summary')::boolean = true"#
    )
    .fetch_all(&state.db_pool)
    .await?;

    let service = TeamsNotificationService::new();
    let portal_url = std::env::var("PORTAL_BASE_URL").unwrap_or_else(|_| "https://app.example.com".to_string());

    let mut sent_count = 0;
    for integration in integrations {
        if let Some(webhook_url) = integration.webhook_url {
            if service.notify_daily_summary(&webhook_url, &summary, &portal_url).await.is_ok() {
                sent_count += 1;
            }
        }
    }

    Ok(Json(serde_json::json!({
        "sent": sent_count,
        "summary": {
            "date": today,
            "tickets_created": summary.tickets_created,
            "tickets_resolved": summary.tickets_resolved,
            "open_tickets": summary.open_tickets,
            "sla_breaches": summary.sla_breaches,
            "sla_compliance": format!("{:.1}%", summary.sla_compliance),
            "billable_hours": format!("{:.1}h", summary.billable_hours)
        }
    })))
}

async fn list_notification_logs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NotificationLogQuery>,
) -> ApiResult<Json<Vec<TeamsNotificationLog>>> {
    let limit = params.pagination.limit();
    let offset = params.pagination.offset();

    let logs = sqlx::query_as!(
        TeamsNotificationLog,
        r#"SELECT id, integration_id, notification_type, ticket_id, payload, success, error_message, sent_at
           FROM notification_log
           WHERE ($1::uuid IS NULL OR integration_id = $1)
             AND ($2::bool IS NULL OR success = $2)
           ORDER BY sent_at DESC
           LIMIT $3 OFFSET $4"#,
        params.integration_id,
        params.success,
        limit,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching notification logs: {}", e);
        ApiError::internal("Failed to fetch notification logs")
    })?;

    Ok(Json(logs))
}

// ==================== Helper Functions ====================

async fn get_integration_by_id(state: &AppState, id: Uuid) -> Result<TeamsIntegration, ApiError> {
    sqlx::query_as!(
        TeamsIntegration,
        r#"SELECT
            id, name, webhook_url,
            channel_id as channel_name, is_active,
            COALESCE((notify_on->>'ticket_created')::boolean, true) as "notify_ticket_created!",
            COALESCE((notify_on->>'ticket_assigned')::boolean, true) as "notify_ticket_assigned!",
            COALESCE((notify_on->>'ticket_resolved')::boolean, true) as "notify_ticket_resolved!",
            COALESCE((notify_on->>'sla_breach')::boolean, true) as "notify_sla_breach!",
            COALESCE((notify_on->>'daily_summary')::boolean, false) as "notify_daily_summary!",
            priority_filter as filter_priorities,
            queue_ids as filter_queue_ids,
            NULL::uuid[] as filter_client_ids,
            last_notification_at, error_count, error_count as notification_count, last_error,
            created_by, created_at, updated_at
         FROM notification_integrations
         WHERE id = $1 AND integration_type = 'teams'"#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching Teams integration: {}", e);
        ApiError::internal("Failed to fetch Teams integration")
    })?
    .ok_or_else(|| ApiError::not_found("Teams integration not found"))
}

async fn log_notification(
    state: &AppState,
    integration_id: Uuid,
    notification_type: &str,
    ticket_id: Option<Uuid>,
    success: bool,
    error_message: Option<String>,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"INSERT INTO notification_log (integration_id, notification_type, ticket_id, payload, success, error_message)
           VALUES ($1, $2, $3, '{}'::jsonb, $4, $5)"#,
        integration_id,
        notification_type,
        ticket_id,
        success,
        error_message
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error logging notification: {}", e);
        ApiError::internal("Failed to log notification")
    })?;

    // Update integration stats
    if success {
        sqlx::query!(
            "UPDATE notification_integrations SET last_notification_at = NOW(), notification_count = COALESCE(notification_count, 0) + 1 WHERE id = $1",
            integration_id
        )
        .execute(&state.db_pool)
        .await?;
    } else {
        sqlx::query!(
            "UPDATE notification_integrations SET error_count = COALESCE(error_count, 0) + 1, last_error = $2 WHERE id = $1",
            integration_id,
            error_message
        )
        .execute(&state.db_pool)
        .await?;
    }

    Ok(())
}
