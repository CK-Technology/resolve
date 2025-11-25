//! Advanced Ticketing Features
//!
//! Ticket queues, canned responses, ticket linking/merging, and routing rules.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::{
    AppState, ApiResult, ApiError,
    PaginatedResponse, PaginationParams,
};
use crate::auth::middleware::AuthUser;

// ==================== Ticket Queues ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TicketQueue {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub icon: String,
    pub email_address: Option<String>,
    pub auto_assign: bool,
    pub round_robin: bool,
    pub default_priority: String,
    pub default_sla_policy_id: Option<Uuid>,
    pub default_category_id: Option<Uuid>,
    pub is_private: bool,
    pub is_active: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TicketQueueWithStats {
    #[serde(flatten)]
    pub queue: TicketQueue,
    pub ticket_count: i64,
    pub open_count: i64,
    pub member_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateQueueRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub email_address: Option<String>,
    pub auto_assign: Option<bool>,
    pub round_robin: Option<bool>,
    pub default_priority: Option<String>,
    pub default_sla_policy_id: Option<Uuid>,
    pub default_category_id: Option<Uuid>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMember {
    pub id: Uuid,
    pub queue_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub role: String,
    pub receive_notifications: bool,
    pub can_assign: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddQueueMemberRequest {
    pub user_id: Uuid,
    pub role: Option<String>,
    pub receive_notifications: Option<bool>,
    pub can_assign: Option<bool>,
}

// ==================== Canned Responses ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CannedResponse {
    pub id: Uuid,
    pub name: String,
    pub shortcut: Option<String>,
    pub subject: Option<String>,
    pub content: String,
    pub content_html: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_global: bool,
    pub user_id: Option<Uuid>,
    pub queue_id: Option<Uuid>,
    pub variables: Option<serde_json::Value>,
    pub usage_count: i32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCannedResponseRequest {
    pub name: String,
    pub shortcut: Option<String>,
    pub subject: Option<String>,
    pub content: String,
    pub content_html: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_global: Option<bool>,
    pub queue_id: Option<Uuid>,
    pub variables: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CannedResponseQuery {
    pub category: Option<String>,
    pub queue_id: Option<Uuid>,
    pub search: Option<String>,
    pub include_personal: Option<bool>,
}

// ==================== Ticket Links ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketLink {
    pub id: Uuid,
    pub source_ticket_id: Uuid,
    pub source_ticket_number: i32,
    pub source_ticket_subject: String,
    pub target_ticket_id: Uuid,
    pub target_ticket_number: i32,
    pub target_ticket_subject: String,
    pub link_type: String,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketLinkRequest {
    pub target_ticket_id: Uuid,
    pub link_type: String, // parent, child, related, duplicate, blocks, blocked_by
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MergeTicketsRequest {
    pub source_ticket_ids: Vec<Uuid>, // tickets to merge
    pub target_ticket_id: Uuid,       // ticket to merge into
    pub merge_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MergeResult {
    pub primary_ticket_id: Uuid,
    pub merged_count: i32,
    pub merged_ticket_numbers: Vec<i32>,
}

// ==================== Ticket Tags ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TicketTag {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ==================== Routing Rules ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoutingRule {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub conditions: serde_json::Value,
    pub assign_queue_id: Option<Uuid>,
    pub assign_user_id: Option<Uuid>,
    pub set_priority: Option<String>,
    pub set_category_id: Option<Uuid>,
    pub add_tags: Option<Vec<String>>,
    pub stop_processing: bool,
    pub is_active: bool,
    pub priority: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoutingRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub conditions: serde_json::Value,
    pub assign_queue_id: Option<Uuid>,
    pub assign_user_id: Option<Uuid>,
    pub set_priority: Option<String>,
    pub set_category_id: Option<Uuid>,
    pub add_tags: Option<Vec<String>>,
    pub stop_processing: Option<bool>,
    pub priority: Option<i32>,
}

// ==================== Routes ====================

pub fn ticket_queue_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_queues).post(create_queue))
        .route("/:id", get(get_queue).put(update_queue).delete(delete_queue))
        .route("/:id/members", get(list_queue_members).post(add_queue_member))
        .route("/:id/members/:user_id", delete(remove_queue_member))
        .route("/:id/tickets", get(list_queue_tickets))
}

pub fn canned_response_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_canned_responses).post(create_canned_response))
        .route("/:id", get(get_canned_response).put(update_canned_response).delete(delete_canned_response))
        .route("/:id/use", post(use_canned_response))
        .route("/search", get(search_canned_responses))
}

pub fn ticket_link_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_ticket_links).post(create_ticket_link))
        .route("/:id", delete(delete_ticket_link))
        .route("/merge", post(merge_tickets))
}

pub fn ticket_tag_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_tags).post(create_tag))
        .route("/:id", put(update_tag).delete(delete_tag))
}

pub fn routing_rule_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_routing_rules).post(create_routing_rule))
        .route("/:id", get(get_routing_rule).put(update_routing_rule).delete(delete_routing_rule))
        .route("/reorder", post(reorder_routing_rules))
}

// ==================== Queue Handlers ====================

async fn list_queues(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<TicketQueueWithStats>>> {
    let queues = sqlx::query_as!(
        TicketQueue,
        r#"SELECT
            id, name, description, color, icon, email_address,
            auto_assign, round_robin, default_priority,
            default_sla_policy_id, default_category_id,
            is_private, is_active, display_order,
            created_at, updated_at
         FROM ticket_queues
         WHERE is_active = true
         ORDER BY display_order, name"#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching queues: {}", e);
        ApiError::internal("Failed to fetch queues")
    })?;

    // Get stats for each queue
    let mut result = Vec::with_capacity(queues.len());
    for queue in queues {
        let stats = sqlx::query!(
            r#"SELECT
                COUNT(*) as "ticket_count!",
                COUNT(*) FILTER (WHERE status NOT IN ('closed', 'resolved')) as "open_count!",
                (SELECT COUNT(*) FROM ticket_queue_members WHERE queue_id = $1 AND is_active = true) as "member_count!"
             FROM tickets WHERE queue_id = $1"#,
            queue.id
        )
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching queue stats: {}", e);
            ApiError::internal("Failed to fetch queue stats")
        })?;

        result.push(TicketQueueWithStats {
            queue,
            ticket_count: stats.ticket_count,
            open_count: stats.open_count,
            member_count: stats.member_count,
        });
    }

    Ok(Json(result))
}

async fn get_queue(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TicketQueueWithStats>> {
    let queue = sqlx::query_as!(
        TicketQueue,
        r#"SELECT
            id, name, description, color, icon, email_address,
            auto_assign, round_robin, default_priority,
            default_sla_policy_id, default_category_id,
            is_private, is_active, display_order,
            created_at, updated_at
         FROM ticket_queues WHERE id = $1"#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching queue: {}", e);
        ApiError::internal("Failed to fetch queue")
    })?
    .ok_or_else(|| ApiError::not_found("Queue not found"))?;

    let stats = sqlx::query!(
        r#"SELECT
            COUNT(*) as "ticket_count!",
            COUNT(*) FILTER (WHERE status NOT IN ('closed', 'resolved')) as "open_count!",
            (SELECT COUNT(*) FROM ticket_queue_members WHERE queue_id = $1 AND is_active = true) as "member_count!"
         FROM tickets WHERE queue_id = $1"#,
        id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching queue stats: {}", e);
        ApiError::internal("Failed to fetch queue stats")
    })?;

    Ok(Json(TicketQueueWithStats {
        queue,
        ticket_count: stats.ticket_count,
        open_count: stats.open_count,
        member_count: stats.member_count,
    }))
}

async fn create_queue(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateQueueRequest>,
) -> ApiResult<Json<TicketQueue>> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query!(
        r#"INSERT INTO ticket_queues (
            id, name, description, color, icon, email_address,
            auto_assign, round_robin, default_priority,
            default_sla_policy_id, default_category_id,
            is_private, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#,
        id,
        req.name,
        req.description,
        req.color.unwrap_or_else(|| "#6b7280".to_string()),
        req.icon.unwrap_or_else(|| "inbox".to_string()),
        req.email_address,
        req.auto_assign.unwrap_or(false),
        req.round_robin.unwrap_or(false),
        req.default_priority.unwrap_or_else(|| "medium".to_string()),
        req.default_sla_policy_id,
        req.default_category_id,
        req.is_private.unwrap_or(false),
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating queue: {}", e);
        ApiError::internal("Failed to create queue")
    })?;

    // Add creator as queue admin
    sqlx::query!(
        "INSERT INTO ticket_queue_members (queue_id, user_id, role) VALUES ($1, $2, 'admin')",
        id,
        auth.0.id
    )
    .execute(&state.db_pool)
    .await
    .ok();

    let queue = sqlx::query_as!(
        TicketQueue,
        r#"SELECT
            id, name, description, color, icon, email_address,
            auto_assign, round_robin, default_priority,
            default_sla_policy_id, default_category_id,
            is_private, is_active, display_order,
            created_at, updated_at
         FROM ticket_queues WHERE id = $1"#,
        id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching created queue: {}", e);
        ApiError::internal("Failed to fetch created queue")
    })?;

    Ok(Json(queue))
}

async fn update_queue(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateQueueRequest>,
) -> ApiResult<Json<TicketQueue>> {
    let result = sqlx::query!(
        r#"UPDATE ticket_queues SET
            name = $2, description = $3, color = COALESCE($4, color),
            icon = COALESCE($5, icon), email_address = $6,
            auto_assign = COALESCE($7, auto_assign),
            round_robin = COALESCE($8, round_robin),
            default_priority = COALESCE($9, default_priority),
            default_sla_policy_id = $10, default_category_id = $11,
            is_private = COALESCE($12, is_private),
            updated_at = NOW()
         WHERE id = $1"#,
        id,
        req.name,
        req.description,
        req.color,
        req.icon,
        req.email_address,
        req.auto_assign,
        req.round_robin,
        req.default_priority,
        req.default_sla_policy_id,
        req.default_category_id,
        req.is_private
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating queue: {}", e);
        ApiError::internal("Failed to update queue")
    })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Queue not found"));
    }

    let queue = sqlx::query_as!(
        TicketQueue,
        r#"SELECT
            id, name, description, color, icon, email_address,
            auto_assign, round_robin, default_priority,
            default_sla_policy_id, default_category_id,
            is_private, is_active, display_order,
            created_at, updated_at
         FROM ticket_queues WHERE id = $1"#,
        id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::internal("Failed to fetch updated queue"))?;

    Ok(Json(queue))
}

async fn delete_queue(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    // Soft delete - just mark as inactive
    let result = sqlx::query!(
        "UPDATE ticket_queues SET is_active = false, updated_at = NOW() WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error deleting queue: {}", e);
        ApiError::internal("Failed to delete queue")
    })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Queue not found"));
    }

    Ok(())
}

async fn list_queue_members(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(queue_id): Path<Uuid>,
) -> ApiResult<Json<Vec<QueueMember>>> {
    let members = sqlx::query_as!(
        QueueMember,
        r#"SELECT
            qm.id, qm.queue_id, qm.user_id,
            u.first_name || ' ' || u.last_name as "user_name!",
            u.email as "user_email!",
            qm.role, qm.receive_notifications, qm.can_assign,
            qm.is_active, qm.created_at
         FROM ticket_queue_members qm
         JOIN users u ON qm.user_id = u.id
         WHERE qm.queue_id = $1 AND qm.is_active = true
         ORDER BY qm.role DESC, u.first_name"#,
        queue_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching queue members: {}", e);
        ApiError::internal("Failed to fetch queue members")
    })?;

    Ok(Json(members))
}

async fn add_queue_member(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(queue_id): Path<Uuid>,
    Json(req): Json<AddQueueMemberRequest>,
) -> ApiResult<Json<QueueMember>> {
    let id = Uuid::new_v4();

    sqlx::query!(
        r#"INSERT INTO ticket_queue_members (id, queue_id, user_id, role, receive_notifications, can_assign)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (queue_id, user_id) DO UPDATE SET
            role = EXCLUDED.role,
            receive_notifications = EXCLUDED.receive_notifications,
            can_assign = EXCLUDED.can_assign,
            is_active = true"#,
        id,
        queue_id,
        req.user_id,
        req.role.unwrap_or_else(|| "member".to_string()),
        req.receive_notifications.unwrap_or(true),
        req.can_assign.unwrap_or(true)
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error adding queue member: {}", e);
        ApiError::internal("Failed to add queue member")
    })?;

    let member = sqlx::query_as!(
        QueueMember,
        r#"SELECT
            qm.id, qm.queue_id, qm.user_id,
            u.first_name || ' ' || u.last_name as "user_name!",
            u.email as "user_email!",
            qm.role, qm.receive_notifications, qm.can_assign,
            qm.is_active, qm.created_at
         FROM ticket_queue_members qm
         JOIN users u ON qm.user_id = u.id
         WHERE qm.queue_id = $1 AND qm.user_id = $2"#,
        queue_id,
        req.user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::internal("Failed to fetch added member"))?;

    Ok(Json(member))
}

async fn remove_queue_member(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((queue_id, user_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<()> {
    sqlx::query!(
        "UPDATE ticket_queue_members SET is_active = false WHERE queue_id = $1 AND user_id = $2",
        queue_id,
        user_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error removing queue member: {}", e);
        ApiError::internal("Failed to remove queue member")
    })?;

    Ok(())
}

async fn list_queue_tickets(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(queue_id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<serde_json::Value>> {
    // Return basic ticket list for the queue
    let tickets = sqlx::query!(
        r#"SELECT
            t.id, t.number, t.subject, t.status, t.priority,
            c.name as client_name, t.created_at
         FROM tickets t
         LEFT JOIN clients c ON t.client_id = c.id
         WHERE t.queue_id = $1 AND t.is_merged = false
         ORDER BY t.created_at DESC
         LIMIT $2 OFFSET $3"#,
        queue_id,
        params.limit(),
        params.offset()
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching queue tickets: {}", e);
        ApiError::internal("Failed to fetch queue tickets")
    })?;

    Ok(Json(serde_json::json!(tickets)))
}

// ==================== Canned Response Handlers ====================

async fn list_canned_responses(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<CannedResponseQuery>,
) -> ApiResult<Json<Vec<CannedResponse>>> {
    let responses = sqlx::query_as!(
        CannedResponse,
        r#"SELECT
            id, name, shortcut, subject, content, content_html,
            category, tags, is_global, user_id, queue_id,
            variables, usage_count, last_used_at, is_active,
            created_by, created_at, updated_at
         FROM canned_responses
         WHERE is_active = true
           AND (is_global = true OR user_id = $1 OR ($2::uuid IS NOT NULL AND queue_id = $2))
           AND ($3::text IS NULL OR category = $3)
           AND ($4::text IS NULL OR name ILIKE '%' || $4 || '%' OR content ILIKE '%' || $4 || '%')
         ORDER BY usage_count DESC, name"#,
        auth.0.id,
        params.queue_id,
        params.category,
        params.search
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching canned responses: {}", e);
        ApiError::internal("Failed to fetch canned responses")
    })?;

    Ok(Json(responses))
}

async fn get_canned_response(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CannedResponse>> {
    let response = sqlx::query_as!(
        CannedResponse,
        r#"SELECT
            id, name, shortcut, subject, content, content_html,
            category, tags, is_global, user_id, queue_id,
            variables, usage_count, last_used_at, is_active,
            created_by, created_at, updated_at
         FROM canned_responses WHERE id = $1"#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching canned response: {}", e);
        ApiError::internal("Failed to fetch canned response")
    })?
    .ok_or_else(|| ApiError::not_found("Canned response not found"))?;

    Ok(Json(response))
}

async fn create_canned_response(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateCannedResponseRequest>,
) -> ApiResult<Json<CannedResponse>> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let is_global = req.is_global.unwrap_or(false);

    // If not global, assign to the user
    let user_id = if is_global { None } else { Some(auth.0.id) };

    sqlx::query!(
        r#"INSERT INTO canned_responses (
            id, name, shortcut, subject, content, content_html,
            category, tags, is_global, user_id, queue_id,
            variables, created_by, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"#,
        id,
        req.name,
        req.shortcut,
        req.subject,
        req.content,
        req.content_html,
        req.category,
        req.tags.as_deref(),
        is_global,
        user_id,
        req.queue_id,
        serde_json::to_value(&req.variables).ok(),
        auth.0.id,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating canned response: {}", e);
        ApiError::internal("Failed to create canned response")
    })?;

    get_canned_response(State(state), auth, Path(id)).await
}

async fn update_canned_response(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateCannedResponseRequest>,
) -> ApiResult<Json<CannedResponse>> {
    let result = sqlx::query!(
        r#"UPDATE canned_responses SET
            name = $2, shortcut = $3, subject = $4,
            content = $5, content_html = $6, category = $7,
            tags = $8, queue_id = $9, variables = $10,
            updated_at = NOW()
         WHERE id = $1"#,
        id,
        req.name,
        req.shortcut,
        req.subject,
        req.content,
        req.content_html,
        req.category,
        req.tags.as_deref(),
        req.queue_id,
        serde_json::to_value(&req.variables).ok()
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating canned response: {}", e);
        ApiError::internal("Failed to update canned response")
    })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Canned response not found"));
    }

    get_canned_response(State(state), auth, Path(id)).await
}

async fn delete_canned_response(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    sqlx::query!(
        "UPDATE canned_responses SET is_active = false, updated_at = NOW() WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error deleting canned response: {}", e);
        ApiError::internal("Failed to delete canned response")
    })?;

    Ok(())
}

async fn use_canned_response(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CannedResponse>> {
    // Increment usage count
    sqlx::query!(
        "UPDATE canned_responses SET usage_count = usage_count + 1, last_used_at = NOW() WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await
    .ok();

    let response = sqlx::query_as!(
        CannedResponse,
        r#"SELECT
            id, name, shortcut, subject, content, content_html,
            category, tags, is_global, user_id, queue_id,
            variables, usage_count, last_used_at, is_active,
            created_by, created_at, updated_at
         FROM canned_responses WHERE id = $1"#,
        id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::internal("Failed to fetch canned response"))?;

    Ok(Json(response))
}

async fn search_canned_responses(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<CannedResponseQuery>,
) -> ApiResult<Json<Vec<CannedResponse>>> {
    list_canned_responses(State(state), auth, Query(params)).await
}

// ==================== Ticket Link Handlers ====================

async fn list_ticket_links(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Query(ticket_id): Query<Option<Uuid>>,
) -> ApiResult<Json<Vec<TicketLink>>> {
    let links = if let Some(tid) = ticket_id {
        sqlx::query_as!(
            TicketLink,
            r#"SELECT
                tl.id,
                tl.source_ticket_id, ts.number as source_ticket_number, ts.subject as source_ticket_subject,
                tl.target_ticket_id, tt.number as target_ticket_number, tt.subject as target_ticket_subject,
                tl.link_type, tl.notes, tl.created_by,
                u.first_name || ' ' || u.last_name as created_by_name,
                tl.created_at
             FROM ticket_links tl
             JOIN tickets ts ON tl.source_ticket_id = ts.id
             JOIN tickets tt ON tl.target_ticket_id = tt.id
             LEFT JOIN users u ON tl.created_by = u.id
             WHERE tl.source_ticket_id = $1 OR tl.target_ticket_id = $1
             ORDER BY tl.created_at DESC"#,
            tid
        )
        .fetch_all(&state.db_pool)
        .await
    } else {
        sqlx::query_as!(
            TicketLink,
            r#"SELECT
                tl.id,
                tl.source_ticket_id, ts.number as source_ticket_number, ts.subject as source_ticket_subject,
                tl.target_ticket_id, tt.number as target_ticket_number, tt.subject as target_ticket_subject,
                tl.link_type, tl.notes, tl.created_by,
                u.first_name || ' ' || u.last_name as created_by_name,
                tl.created_at
             FROM ticket_links tl
             JOIN tickets ts ON tl.source_ticket_id = ts.id
             JOIN tickets tt ON tl.target_ticket_id = tt.id
             LEFT JOIN users u ON tl.created_by = u.id
             ORDER BY tl.created_at DESC
             LIMIT 100"#
        )
        .fetch_all(&state.db_pool)
        .await
    }
    .map_err(|e| {
        tracing::error!("Error fetching ticket links: {}", e);
        ApiError::internal("Failed to fetch ticket links")
    })?;

    Ok(Json(links))
}

async fn create_ticket_link(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(source_ticket_id): Path<Uuid>,
    Json(req): Json<CreateTicketLinkRequest>,
) -> ApiResult<Json<TicketLink>> {
    // Validate link type
    let valid_types = ["parent", "child", "related", "duplicate", "blocks", "blocked_by"];
    if !valid_types.contains(&req.link_type.as_str()) {
        return Err(ApiError::validation_single("link_type", "Invalid link type"));
    }

    let id = Uuid::new_v4();

    sqlx::query!(
        r#"INSERT INTO ticket_links (id, source_ticket_id, target_ticket_id, link_type, notes, created_by)
         VALUES ($1, $2, $3, $4, $5, $6)"#,
        id,
        source_ticket_id,
        req.target_ticket_id,
        req.link_type,
        req.notes,
        auth.0.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating ticket link: {}", e);
        if e.to_string().contains("duplicate") {
            ApiError::conflict("This link already exists")
        } else {
            ApiError::internal("Failed to create ticket link")
        }
    })?;

    let link = sqlx::query_as!(
        TicketLink,
        r#"SELECT
            tl.id,
            tl.source_ticket_id, ts.number as source_ticket_number, ts.subject as source_ticket_subject,
            tl.target_ticket_id, tt.number as target_ticket_number, tt.subject as target_ticket_subject,
            tl.link_type, tl.notes, tl.created_by,
            u.first_name || ' ' || u.last_name as created_by_name,
            tl.created_at
         FROM ticket_links tl
         JOIN tickets ts ON tl.source_ticket_id = ts.id
         JOIN tickets tt ON tl.target_ticket_id = tt.id
         LEFT JOIN users u ON tl.created_by = u.id
         WHERE tl.id = $1"#,
        id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::internal("Failed to fetch created link"))?;

    Ok(Json(link))
}

async fn delete_ticket_link(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    let result = sqlx::query!("DELETE FROM ticket_links WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting ticket link: {}", e);
            ApiError::internal("Failed to delete ticket link")
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Ticket link not found"));
    }

    Ok(())
}

async fn merge_tickets(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<MergeTicketsRequest>,
) -> ApiResult<Json<MergeResult>> {
    if req.source_ticket_ids.is_empty() {
        return Err(ApiError::validation_single("source_ticket_ids", "At least one source ticket is required"));
    }

    if req.source_ticket_ids.contains(&req.target_ticket_id) {
        return Err(ApiError::validation_single("target_ticket_id", "Target ticket cannot be in source list"));
    }

    let mut merged_numbers = Vec::new();

    for source_id in &req.source_ticket_ids {
        // Get source ticket info
        let source = sqlx::query!(
            "SELECT number, subject FROM tickets WHERE id = $1 AND is_merged = false",
            source_id
        )
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| ApiError::internal("Failed to fetch source ticket"))?
        .ok_or_else(|| ApiError::not_found("Source ticket not found or already merged"))?;

        // Record the merge
        sqlx::query!(
            r#"INSERT INTO ticket_merges (primary_ticket_id, merged_ticket_id, merged_ticket_number, merged_ticket_subject, merge_reason, merged_by)
             VALUES ($1, $2, $3, $4, $5, $6)"#,
            req.target_ticket_id,
            source_id,
            source.number,
            source.subject,
            req.merge_reason,
            auth.0.id
        )
        .execute(&state.db_pool)
        .await
        .map_err(|e| ApiError::internal("Failed to record merge"))?;

        // Mark source ticket as merged
        sqlx::query!(
            "UPDATE tickets SET is_merged = true, merged_into_id = $2, updated_at = NOW() WHERE id = $1",
            source_id,
            req.target_ticket_id
        )
        .execute(&state.db_pool)
        .await
        .map_err(|e| ApiError::internal("Failed to mark ticket as merged"))?;

        // Move replies from source to target
        sqlx::query!(
            "UPDATE ticket_replies SET ticket_id = $2 WHERE ticket_id = $1",
            source_id,
            req.target_ticket_id
        )
        .execute(&state.db_pool)
        .await
        .ok();

        // Move time entries from source to target
        sqlx::query!(
            "UPDATE time_entries SET ticket_id = $2 WHERE ticket_id = $1",
            source_id,
            req.target_ticket_id
        )
        .execute(&state.db_pool)
        .await
        .ok();

        merged_numbers.push(source.number);
    }

    // Add a note to the target ticket about the merge
    let merge_note = format!(
        "Merged tickets: #{}\n{}",
        merged_numbers.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", #"),
        req.merge_reason.as_deref().unwrap_or("")
    );

    sqlx::query!(
        "INSERT INTO ticket_replies (id, ticket_id, user_id, type, details) VALUES ($1, $2, $3, 'note', $4)",
        Uuid::new_v4(),
        req.target_ticket_id,
        auth.0.id,
        merge_note
    )
    .execute(&state.db_pool)
    .await
    .ok();

    Ok(Json(MergeResult {
        primary_ticket_id: req.target_ticket_id,
        merged_count: merged_numbers.len() as i32,
        merged_ticket_numbers: merged_numbers,
    }))
}

// ==================== Tag Handlers ====================

async fn list_tags(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<TicketTag>>> {
    let tags = sqlx::query_as!(
        TicketTag,
        "SELECT id, name, color, description, is_active, created_at FROM ticket_tags WHERE is_active = true ORDER BY name"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching tags: {}", e);
        ApiError::internal("Failed to fetch tags")
    })?;

    Ok(Json(tags))
}

async fn create_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(req): Json<TicketTag>,
) -> ApiResult<Json<TicketTag>> {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO ticket_tags (id, name, color, description) VALUES ($1, $2, $3, $4)",
        id,
        req.name,
        req.color,
        req.description
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating tag: {}", e);
        if e.to_string().contains("duplicate") {
            ApiError::conflict("Tag with this name already exists")
        } else {
            ApiError::internal("Failed to create tag")
        }
    })?;

    let tag = sqlx::query_as!(
        TicketTag,
        "SELECT id, name, color, description, is_active, created_at FROM ticket_tags WHERE id = $1",
        id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::internal("Failed to fetch created tag"))?;

    Ok(Json(tag))
}

async fn update_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<TicketTag>,
) -> ApiResult<Json<TicketTag>> {
    sqlx::query!(
        "UPDATE ticket_tags SET name = $2, color = $3, description = $4 WHERE id = $1",
        id,
        req.name,
        req.color,
        req.description
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating tag: {}", e);
        ApiError::internal("Failed to update tag")
    })?;

    let tag = sqlx::query_as!(
        TicketTag,
        "SELECT id, name, color, description, is_active, created_at FROM ticket_tags WHERE id = $1",
        id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::internal("Failed to fetch updated tag"))?;

    Ok(Json(tag))
}

async fn delete_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    sqlx::query!("UPDATE ticket_tags SET is_active = false WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting tag: {}", e);
            ApiError::internal("Failed to delete tag")
        })?;

    Ok(())
}

// ==================== Routing Rule Handlers ====================

async fn list_routing_rules(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<RoutingRule>>> {
    let rules = sqlx::query_as!(
        RoutingRule,
        r#"SELECT
            id, name, description, conditions,
            assign_queue_id, assign_user_id, set_priority, set_category_id,
            add_tags, stop_processing, is_active, priority,
            created_by, created_at, updated_at
         FROM ticket_routing_rules
         ORDER BY priority DESC, name"#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching routing rules: {}", e);
        ApiError::internal("Failed to fetch routing rules")
    })?;

    Ok(Json(rules))
}

async fn get_routing_rule(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<RoutingRule>> {
    let rule = sqlx::query_as!(
        RoutingRule,
        r#"SELECT
            id, name, description, conditions,
            assign_queue_id, assign_user_id, set_priority, set_category_id,
            add_tags, stop_processing, is_active, priority,
            created_by, created_at, updated_at
         FROM ticket_routing_rules WHERE id = $1"#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching routing rule: {}", e);
        ApiError::internal("Failed to fetch routing rule")
    })?
    .ok_or_else(|| ApiError::not_found("Routing rule not found"))?;

    Ok(Json(rule))
}

async fn create_routing_rule(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateRoutingRuleRequest>,
) -> ApiResult<Json<RoutingRule>> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query!(
        r#"INSERT INTO ticket_routing_rules (
            id, name, description, conditions,
            assign_queue_id, assign_user_id, set_priority, set_category_id,
            add_tags, stop_processing, priority, created_by, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#,
        id,
        req.name,
        req.description,
        req.conditions,
        req.assign_queue_id,
        req.assign_user_id,
        req.set_priority,
        req.set_category_id,
        req.add_tags.as_deref(),
        req.stop_processing.unwrap_or(true),
        req.priority.unwrap_or(0),
        auth.0.id,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating routing rule: {}", e);
        ApiError::internal("Failed to create routing rule")
    })?;

    get_routing_rule(State(state), auth, Path(id)).await
}

async fn update_routing_rule(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateRoutingRuleRequest>,
) -> ApiResult<Json<RoutingRule>> {
    sqlx::query!(
        r#"UPDATE ticket_routing_rules SET
            name = $2, description = $3, conditions = $4,
            assign_queue_id = $5, assign_user_id = $6,
            set_priority = $7, set_category_id = $8,
            add_tags = $9, stop_processing = COALESCE($10, stop_processing),
            priority = COALESCE($11, priority),
            updated_at = NOW()
         WHERE id = $1"#,
        id,
        req.name,
        req.description,
        req.conditions,
        req.assign_queue_id,
        req.assign_user_id,
        req.set_priority,
        req.set_category_id,
        req.add_tags.as_deref(),
        req.stop_processing,
        req.priority
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating routing rule: {}", e);
        ApiError::internal("Failed to update routing rule")
    })?;

    get_routing_rule(State(state), auth, Path(id)).await
}

async fn delete_routing_rule(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    sqlx::query!("UPDATE ticket_routing_rules SET is_active = false, updated_at = NOW() WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting routing rule: {}", e);
            ApiError::internal("Failed to delete routing rule")
        })?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ReorderRulesRequest {
    pub rule_ids: Vec<Uuid>,
}

async fn reorder_routing_rules(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(req): Json<ReorderRulesRequest>,
) -> ApiResult<()> {
    for (index, rule_id) in req.rule_ids.iter().enumerate() {
        sqlx::query!(
            "UPDATE ticket_routing_rules SET priority = $2, updated_at = NOW() WHERE id = $1",
            rule_id,
            (req.rule_ids.len() - index) as i32
        )
        .execute(&state.db_pool)
        .await
        .ok();
    }

    Ok(())
}
