use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::AppState;
use resolve_shared::Notification;

pub fn notification_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_notifications))
        .route("/:id/read", put(mark_as_read))
        .route("/read-all", put(mark_all_as_read))
        .route("/:id", delete(delete_notification))
        .route("/unread-count", get(get_unread_count))
}

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub read: Option<bool>,
    pub notification_type: Option<String>,
    pub entity_type: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    #[serde(flatten)]
    pub notification: Notification,
    pub relative_time: String,
}

#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub unread_count: i64,
}

async fn list_notifications(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListNotificationsQuery>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    let mut sql = String::from(
        r#"
        SELECT id, user_id, title, message, notification_type,
               entity_type, entity_id, read, created_at
        FROM notifications
        WHERE user_id = $1
        "#
    );

    let mut conditions = Vec::new();
    let mut param_count = 1;

    if let Some(read) = query.read {
        param_count += 1;
        conditions.push(format!("read = ${}", param_count));
    }

    if let Some(notification_type) = &query.notification_type {
        param_count += 1;
        conditions.push(format!("notification_type = ${}", param_count));
    }

    if let Some(entity_type) = &query.entity_type {
        param_count += 1;
        conditions.push(format!("entity_type = ${}", param_count));
    }

    if !conditions.is_empty() {
        sql.push_str(&format!(" AND {}", conditions.join(" AND ")));
    }

    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));

    // For simplicity, use a basic query. In production, use a query builder
    let notifications = sqlx::query_as!(
        Notification,
        r#"
        SELECT id, user_id, title, message, notification_type,
               entity_type, entity_id, read, created_at
        FROM notifications
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        auth.0.id,
        limit as i64,
        offset as i64
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Add relative time information
    let notification_responses: Vec<NotificationResponse> = notifications.into_iter().map(|notification| {
        NotificationResponse {
            relative_time: format_relative_time(notification.created_at),
            notification,
        }
    }).collect();

    Ok(Json(notification_responses))
}

async fn mark_as_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!(
        "UPDATE notifications SET read = true WHERE id = $1 AND user_id = $2",
        id,
        auth.0.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({ "message": "Notification marked as read" })))
}

async fn mark_all_as_read(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!(
        "UPDATE notifications SET read = true WHERE user_id = $1 AND read = false",
        auth.0.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ 
        "message": "All notifications marked as read",
        "updated_count": result.rows_affected()
    })))
}

async fn delete_notification(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!(
        "DELETE FROM notifications WHERE id = $1 AND user_id = $2",
        id,
        auth.0.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({ "message": "Notification deleted" })))
}

async fn get_unread_count(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let unread_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read = false",
        auth.0.id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .unwrap_or(0);

    Ok(Json(UnreadCountResponse { unread_count }))
}

// Utility functions for creating notifications
pub async fn create_notification(
    db_pool: &sqlx::PgPool,
    user_id: Uuid,
    title: String,
    message: String,
    notification_type: String,
    entity_type: Option<String>,
    entity_id: Option<Uuid>,
) -> Result<Uuid, sqlx::Error> {
    let notification_id = Uuid::new_v4();
    
    sqlx::query!(
        r#"
        INSERT INTO notifications (
            id, user_id, title, message, notification_type,
            entity_type, entity_id, read, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, false, NOW())
        "#,
        notification_id,
        user_id,
        title,
        message,
        notification_type,
        entity_type,
        entity_id
    )
    .execute(db_pool)
    .await?;

    Ok(notification_id)
}

// Bulk notification creation for multiple users
pub async fn create_notifications_for_users(
    db_pool: &sqlx::PgPool,
    user_ids: Vec<Uuid>,
    title: String,
    message: String,
    notification_type: String,
    entity_type: Option<String>,
    entity_id: Option<Uuid>,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let mut notification_ids = Vec::new();
    
    for user_id in user_ids {
        let notification_id = create_notification(
            db_pool,
            user_id,
            title.clone(),
            message.clone(),
            notification_type.clone(),
            entity_type.clone(),
            entity_id,
        ).await?;
        
        notification_ids.push(notification_id);
    }
    
    Ok(notification_ids)
}

// Helper to create ticket-related notifications
pub async fn notify_ticket_update(
    db_pool: &sqlx::PgPool,
    ticket_id: Uuid,
    client_id: Uuid,
    action: &str,
    details: &str,
) -> Result<(), sqlx::Error> {
    // Get users who should be notified (assigned user, watchers, etc.)
    let user_ids = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT user_id FROM (
            SELECT assigned_to as user_id FROM tickets WHERE id = $1 AND assigned_to IS NOT NULL
            UNION
            SELECT user_id FROM ticket_watchers WHERE ticket_id = $1
            UNION
            SELECT id as user_id FROM users WHERE role_id IN (
                SELECT id FROM roles WHERE name IN ('admin', 'technician')
            )
        ) AS users
        "#,
        ticket_id
    )
    .fetch_all(db_pool)
    .await?;

    let title = format!("Ticket {}", action);
    let message = format!("Ticket has been {}. {}", action.to_lowercase(), details);

    create_notifications_for_users(
        db_pool,
        user_ids,
        title,
        message,
        "ticket_update".to_string(),
        Some("ticket".to_string()),
        Some(ticket_id),
    ).await?;

    Ok(())
}

// Helper to create expiry notifications
pub async fn notify_expiring_items(
    db_pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    // Notify about expiring domains
    let expiring_domains = sqlx::query!(
        r#"
        SELECT d.id, d.name, d.expiry_date, c.name as client_name
        FROM domains d
        JOIN clients c ON d.client_id = c.id
        WHERE d.expiry_date <= CURRENT_DATE + INTERVAL '7 days'
        AND d.expiry_date > CURRENT_DATE
        "#
    )
    .fetch_all(db_pool)
    .await?;

    for domain in expiring_domains {
        let title = "Domain Expiring Soon".to_string();
        let message = format!(
            "Domain '{}' for client '{}' expires on {}",
            domain.name,
            domain.client_name,
            domain.expiry_date.unwrap().format("%Y-%m-%d")
        );

        // Notify all admins
        let admin_user_ids = sqlx::query_scalar!(
            r#"
            SELECT u.id FROM users u
            JOIN roles r ON u.role_id = r.id
            WHERE r.name = 'admin' AND u.is_active = true
            "#
        )
        .fetch_all(db_pool)
        .await?;

        create_notifications_for_users(
            db_pool,
            admin_user_ids,
            title,
            message,
            "domain_expiry".to_string(),
            Some("domain".to_string()),
            Some(domain.id),
        ).await?;
    }

    // Notify about expiring SSL certificates
    let expiring_certs = sqlx::query!(
        r#"
        SELECT s.id, s.name, s.common_name, s.expiry_date, c.name as client_name
        FROM ssl_certificates s
        JOIN clients c ON s.client_id = c.id
        WHERE s.expiry_date <= CURRENT_DATE + INTERVAL '7 days'
        AND s.expiry_date > CURRENT_DATE
        "#
    )
    .fetch_all(db_pool)
    .await?;

    for cert in expiring_certs {
        let title = "SSL Certificate Expiring Soon".to_string();
        let message = format!(
            "SSL Certificate '{}' (CN: {}) for client '{}' expires on {}",
            cert.name,
            cert.common_name,
            cert.client_name,
            cert.expiry_date.format("%Y-%m-%d")
        );

        // Notify all admins
        let admin_user_ids = sqlx::query_scalar!(
            r#"
            SELECT u.id FROM users u
            JOIN roles r ON u.role_id = r.id
            WHERE r.name = 'admin' AND u.is_active = true
            "#
        )
        .fetch_all(db_pool)
        .await?;

        create_notifications_for_users(
            db_pool,
            admin_user_ids,
            title,
            message,
            "ssl_expiry".to_string(),
            Some("ssl_certificate".to_string()),
            Some(cert.id),
        ).await?;
    }

    Ok(())
}

fn format_relative_time(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(timestamp);

    if duration.num_minutes() < 1 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_days() < 30 {
        format!("{} weeks ago", duration.num_days() / 7)
    } else if duration.num_days() < 365 {
        format!("{} months ago", duration.num_days() / 30)
    } else {
        format!("{} years ago", duration.num_days() / 365)
    }
}