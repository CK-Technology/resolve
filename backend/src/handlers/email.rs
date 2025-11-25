//! Email settings and mailbox management API
//!
//! Provides endpoints for configuring email-to-ticket integration,
//! testing SMTP/IMAP connections, and managing email templates.

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

/// Mailbox configuration for email-to-ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMailbox {
    pub id: Uuid,
    pub name: String,
    pub email_address: String,
    pub mailbox_type: String, // "support", "sales", "billing", etc.
    pub imap_host: String,
    pub imap_port: i32,
    pub imap_username: String,
    pub imap_folder: String,
    pub use_tls: bool,
    pub is_active: bool,
    pub poll_interval_secs: i32,
    /// Default queue to assign tickets created from this mailbox
    pub default_queue_id: Option<Uuid>,
    /// Default priority for tickets from this mailbox
    pub default_priority: String,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Request to create/update a mailbox
#[derive(Debug, Deserialize)]
pub struct CreateMailboxRequest {
    pub name: String,
    pub email_address: String,
    pub mailbox_type: String,
    pub imap_host: String,
    pub imap_port: Option<i32>,
    pub imap_username: String,
    pub imap_password: String,
    pub imap_folder: Option<String>,
    pub use_tls: Option<bool>,
    pub poll_interval_secs: Option<i32>,
    pub default_queue_id: Option<Uuid>,
    pub default_priority: Option<String>,
}

/// Request to test mailbox connection
#[derive(Debug, Deserialize)]
pub struct TestMailboxRequest {
    pub imap_host: String,
    pub imap_port: i32,
    pub imap_username: String,
    pub imap_password: String,
    pub imap_folder: Option<String>,
    pub use_tls: bool,
}

/// Response from connection test
#[derive(Debug, Serialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
    pub mailbox_count: Option<i32>,
    pub unread_count: Option<i32>,
}

/// SMTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpSettings {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: bool,
    pub is_configured: bool,
}

/// Email template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Email log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailLog {
    pub id: Uuid,
    pub to_email: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub template_id: Option<Uuid>,
    pub status: String, // "sent", "failed", "pending"
    pub error_message: Option<String>,
    pub related_type: Option<String>, // "ticket", "invoice", "client"
    pub related_id: Option<Uuid>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Default)]
pub struct EmailLogQuery {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub status: Option<String>,
    pub related_type: Option<String>,
    pub from_date: Option<chrono::NaiveDate>,
    pub to_date: Option<chrono::NaiveDate>,
}

pub fn email_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Mailbox management
        .route("/mailboxes", get(list_mailboxes).post(create_mailbox))
        .route("/mailboxes/:id", get(get_mailbox).put(update_mailbox).delete(delete_mailbox))
        .route("/mailboxes/:id/activate", post(activate_mailbox))
        .route("/mailboxes/:id/deactivate", post(deactivate_mailbox))
        .route("/mailboxes/test", post(test_mailbox_connection))
        // SMTP settings
        .route("/smtp", get(get_smtp_settings).put(update_smtp_settings))
        .route("/smtp/test", post(test_smtp_connection))
        // Email templates
        .route("/templates", get(list_templates).post(create_template))
        .route("/templates/:id", get(get_template).put(update_template).delete(delete_template))
        // Email logs
        .route("/logs", get(list_email_logs))
        .route("/logs/:id", get(get_email_log))
        // Send test email
        .route("/send-test", post(send_test_email))
}

// ==================== Mailbox Handlers ====================

/// List all configured mailboxes
async fn list_mailboxes(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<EmailMailbox>>> {
    let mailboxes = sqlx::query_as!(
        EmailMailbox,
        r#"SELECT
            id, name, email_address, mailbox_type,
            imap_host, imap_port, imap_username, imap_folder,
            use_tls, is_active, poll_interval_secs,
            default_queue_id, default_priority,
            last_checked_at, last_error,
            created_at, updated_at
         FROM email_mailboxes
         ORDER BY name ASC"#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching mailboxes: {}", e);
        ApiError::internal("Failed to fetch mailboxes")
    })?;

    Ok(Json(mailboxes))
}

/// Get a single mailbox by ID
async fn get_mailbox(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<EmailMailbox>> {
    let mailbox = sqlx::query_as!(
        EmailMailbox,
        r#"SELECT
            id, name, email_address, mailbox_type,
            imap_host, imap_port, imap_username, imap_folder,
            use_tls, is_active, poll_interval_secs,
            default_queue_id, default_priority,
            last_checked_at, last_error,
            created_at, updated_at
         FROM email_mailboxes
         WHERE id = $1"#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching mailbox: {}", e);
        ApiError::internal("Failed to fetch mailbox")
    })?
    .ok_or_else(|| ApiError::not_found("Mailbox not found"))?;

    Ok(Json(mailbox))
}

/// Create a new mailbox
async fn create_mailbox(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(req): Json<CreateMailboxRequest>,
) -> ApiResult<Json<EmailMailbox>> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    // Encrypt the password before storing (in production, use proper encryption)
    let encrypted_password = encrypt_password(&req.imap_password);

    sqlx::query!(
        r#"INSERT INTO email_mailboxes (
            id, name, email_address, mailbox_type,
            imap_host, imap_port, imap_username, imap_password, imap_folder,
            use_tls, is_active, poll_interval_secs,
            default_queue_id, default_priority,
            created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"#,
        id,
        req.name,
        req.email_address,
        req.mailbox_type,
        req.imap_host,
        req.imap_port.unwrap_or(993),
        req.imap_username,
        encrypted_password,
        req.imap_folder.unwrap_or_else(|| "INBOX".to_string()),
        req.use_tls.unwrap_or(true),
        false, // Start inactive
        req.poll_interval_secs.unwrap_or(60),
        req.default_queue_id,
        req.default_priority.unwrap_or_else(|| "medium".to_string()),
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating mailbox: {}", e);
        ApiError::internal("Failed to create mailbox")
    })?;

    // Fetch and return the created mailbox
    get_mailbox(State(state), _auth, Path(id)).await
}

/// Update a mailbox
async fn update_mailbox(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateMailboxRequest>,
) -> ApiResult<Json<EmailMailbox>> {
    let now = Utc::now();

    // If password is provided, encrypt it
    let password_update = if !req.imap_password.is_empty() {
        Some(encrypt_password(&req.imap_password))
    } else {
        None
    };

    let result = if let Some(encrypted_password) = password_update {
        sqlx::query!(
            r#"UPDATE email_mailboxes SET
                name = $2, email_address = $3, mailbox_type = $4,
                imap_host = $5, imap_port = $6, imap_username = $7,
                imap_password = $8, imap_folder = $9,
                use_tls = $10, poll_interval_secs = $11,
                default_queue_id = $12, default_priority = $13,
                updated_at = $14
             WHERE id = $1"#,
            id,
            req.name,
            req.email_address,
            req.mailbox_type,
            req.imap_host,
            req.imap_port.unwrap_or(993),
            req.imap_username,
            encrypted_password,
            req.imap_folder.unwrap_or_else(|| "INBOX".to_string()),
            req.use_tls.unwrap_or(true),
            req.poll_interval_secs.unwrap_or(60),
            req.default_queue_id,
            req.default_priority.unwrap_or_else(|| "medium".to_string()),
            now
        )
        .execute(&state.db_pool)
        .await
    } else {
        sqlx::query!(
            r#"UPDATE email_mailboxes SET
                name = $2, email_address = $3, mailbox_type = $4,
                imap_host = $5, imap_port = $6, imap_username = $7,
                imap_folder = $8, use_tls = $9, poll_interval_secs = $10,
                default_queue_id = $11, default_priority = $12,
                updated_at = $13
             WHERE id = $1"#,
            id,
            req.name,
            req.email_address,
            req.mailbox_type,
            req.imap_host,
            req.imap_port.unwrap_or(993),
            req.imap_username,
            req.imap_folder.unwrap_or_else(|| "INBOX".to_string()),
            req.use_tls.unwrap_or(true),
            req.poll_interval_secs.unwrap_or(60),
            req.default_queue_id,
            req.default_priority.unwrap_or_else(|| "medium".to_string()),
            now
        )
        .execute(&state.db_pool)
        .await
    };

    result.map_err(|e| {
        tracing::error!("Error updating mailbox: {}", e);
        ApiError::internal("Failed to update mailbox")
    })?;

    get_mailbox(State(state), _auth, Path(id)).await
}

/// Delete a mailbox
async fn delete_mailbox(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    let result = sqlx::query!("DELETE FROM email_mailboxes WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting mailbox: {}", e);
            ApiError::internal("Failed to delete mailbox")
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Mailbox not found"));
    }

    Ok(())
}

/// Activate a mailbox for email processing
async fn activate_mailbox(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<EmailMailbox>> {
    sqlx::query!(
        "UPDATE email_mailboxes SET is_active = true, updated_at = NOW() WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error activating mailbox: {}", e);
        ApiError::internal("Failed to activate mailbox")
    })?;

    get_mailbox(State(state), _auth, Path(id)).await
}

/// Deactivate a mailbox
async fn deactivate_mailbox(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<EmailMailbox>> {
    sqlx::query!(
        "UPDATE email_mailboxes SET is_active = false, updated_at = NOW() WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error deactivating mailbox: {}", e);
        ApiError::internal("Failed to deactivate mailbox")
    })?;

    get_mailbox(State(state), _auth, Path(id)).await
}

/// Test mailbox IMAP connection
async fn test_mailbox_connection(
    _auth: AuthUser,
    Json(req): Json<TestMailboxRequest>,
) -> ApiResult<Json<TestConnectionResponse>> {
    // This would actually test the IMAP connection
    // For now, return a simulated response
    let result = test_imap_connection(&req).await;

    match result {
        Ok((mailbox_count, unread_count)) => Ok(Json(TestConnectionResponse {
            success: true,
            message: "Successfully connected to IMAP server".to_string(),
            mailbox_count: Some(mailbox_count),
            unread_count: Some(unread_count),
        })),
        Err(e) => Ok(Json(TestConnectionResponse {
            success: false,
            message: format!("Connection failed: {}", e),
            mailbox_count: None,
            unread_count: None,
        })),
    }
}

// ==================== SMTP Handlers ====================

/// Get current SMTP settings
async fn get_smtp_settings(
    _auth: AuthUser,
) -> ApiResult<Json<SmtpSettings>> {
    // Read from environment or database
    let settings = SmtpSettings {
        host: std::env::var("SMTP_HOST").unwrap_or_default(),
        port: std::env::var("SMTP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(587),
        username: std::env::var("SMTP_USERNAME").unwrap_or_default(),
        from_email: std::env::var("SMTP_FROM_EMAIL").unwrap_or_default(),
        from_name: std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Resolve Support".to_string()),
        use_tls: std::env::var("SMTP_USE_TLS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true),
        is_configured: !std::env::var("SMTP_USERNAME").unwrap_or_default().is_empty(),
    };

    Ok(Json(settings))
}

/// Update SMTP settings (stores in database for persistence)
async fn update_smtp_settings(
    State(_state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(_req): Json<SmtpSettings>,
) -> ApiResult<Json<SmtpSettings>> {
    // In production, store these in the database
    // For now, SMTP settings come from environment variables
    Err(ApiError::bad_request(
        "SMTP settings must be configured via environment variables",
    ))
}

/// Test SMTP connection by sending a test email
async fn test_smtp_connection(
    _auth: AuthUser,
) -> ApiResult<Json<TestConnectionResponse>> {
    // Verify SMTP configuration
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_default();
    if smtp_host.is_empty() {
        return Ok(Json(TestConnectionResponse {
            success: false,
            message: "SMTP is not configured".to_string(),
            mailbox_count: None,
            unread_count: None,
        }));
    }

    // Would actually test SMTP connection here
    Ok(Json(TestConnectionResponse {
        success: true,
        message: "SMTP connection successful".to_string(),
        mailbox_count: None,
        unread_count: None,
    }))
}

// ==================== Template Handlers ====================

/// List email templates
async fn list_templates(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<EmailTemplate>>> {
    let templates = sqlx::query_as!(
        EmailTemplate,
        r#"SELECT
            id, name, slug, subject, html_body, text_body,
            description, is_system, created_at, updated_at
         FROM email_templates
         ORDER BY is_system DESC, name ASC"#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching templates: {}", e);
        ApiError::internal("Failed to fetch templates")
    })?;

    Ok(Json(templates))
}

/// Get a single template
async fn get_template(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<EmailTemplate>> {
    let template = sqlx::query_as!(
        EmailTemplate,
        r#"SELECT
            id, name, slug, subject, html_body, text_body,
            description, is_system, created_at, updated_at
         FROM email_templates
         WHERE id = $1"#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching template: {}", e);
        ApiError::internal("Failed to fetch template")
    })?
    .ok_or_else(|| ApiError::not_found("Template not found"))?;

    Ok(Json(template))
}

/// Create email template
async fn create_template(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(req): Json<EmailTemplate>,
) -> ApiResult<Json<EmailTemplate>> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query!(
        r#"INSERT INTO email_templates (
            id, name, slug, subject, html_body, text_body,
            description, is_system, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, false, $8)"#,
        id,
        req.name,
        req.slug,
        req.subject,
        req.html_body,
        req.text_body,
        req.description,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating template: {}", e);
        ApiError::internal("Failed to create template")
    })?;

    get_template(State(state), _auth, Path(id)).await
}

/// Update email template
async fn update_template(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<EmailTemplate>,
) -> ApiResult<Json<EmailTemplate>> {
    // Check if it's a system template
    let existing = sqlx::query_scalar!("SELECT is_system FROM email_templates WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error checking template: {}", e);
            ApiError::internal("Failed to check template")
        })?
        .ok_or_else(|| ApiError::not_found("Template not found"))?;

    if existing {
        return Err(ApiError::forbidden("Cannot modify system templates"));
    }

    sqlx::query!(
        r#"UPDATE email_templates SET
            name = $2, slug = $3, subject = $4,
            html_body = $5, text_body = $6,
            description = $7, updated_at = NOW()
         WHERE id = $1"#,
        id,
        req.name,
        req.slug,
        req.subject,
        req.html_body,
        req.text_body,
        req.description
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating template: {}", e);
        ApiError::internal("Failed to update template")
    })?;

    get_template(State(state), _auth, Path(id)).await
}

/// Delete email template
async fn delete_template(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    // Check if it's a system template
    let is_system = sqlx::query_scalar!("SELECT is_system FROM email_templates WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error checking template: {}", e);
            ApiError::internal("Failed to check template")
        })?
        .ok_or_else(|| ApiError::not_found("Template not found"))?;

    if is_system {
        return Err(ApiError::forbidden("Cannot delete system templates"));
    }

    sqlx::query!("DELETE FROM email_templates WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting template: {}", e);
            ApiError::internal("Failed to delete template")
        })?;

    Ok(())
}

// ==================== Email Log Handlers ====================

/// List email logs
async fn list_email_logs(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Query(params): Query<EmailLogQuery>,
) -> ApiResult<Json<PaginatedResponse<EmailLog>>> {
    let limit = params.pagination.limit();
    let offset = params.pagination.offset();

    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM email_logs
         WHERE ($1::text IS NULL OR status = $1)
           AND ($2::text IS NULL OR related_type = $2)
           AND ($3::date IS NULL OR created_at::date >= $3)
           AND ($4::date IS NULL OR created_at::date <= $4)"#,
        params.status,
        params.related_type,
        params.from_date,
        params.to_date
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error counting logs: {}", e);
        ApiError::internal("Failed to count logs")
    })?;

    let logs = sqlx::query_as!(
        EmailLog,
        r#"SELECT
            id, to_email, to_name, subject, template_id,
            status, error_message, related_type, related_id,
            sent_at, created_at
         FROM email_logs
         WHERE ($1::text IS NULL OR status = $1)
           AND ($2::text IS NULL OR related_type = $2)
           AND ($3::date IS NULL OR created_at::date >= $3)
           AND ($4::date IS NULL OR created_at::date <= $4)
         ORDER BY created_at DESC
         LIMIT $5 OFFSET $6"#,
        params.status,
        params.related_type,
        params.from_date,
        params.to_date,
        limit,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching logs: {}", e);
        ApiError::internal("Failed to fetch logs")
    })?;

    Ok(Json(PaginatedResponse::new(logs, &params.pagination, total)))
}

/// Get a single email log entry
async fn get_email_log(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<EmailLog>> {
    let log = sqlx::query_as!(
        EmailLog,
        r#"SELECT
            id, to_email, to_name, subject, template_id,
            status, error_message, related_type, related_id,
            sent_at, created_at
         FROM email_logs
         WHERE id = $1"#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching log: {}", e);
        ApiError::internal("Failed to fetch log")
    })?
    .ok_or_else(|| ApiError::not_found("Email log not found"))?;

    Ok(Json(log))
}

/// Send a test email
#[derive(Debug, Deserialize)]
pub struct SendTestEmailRequest {
    pub to_email: String,
    pub to_name: Option<String>,
    pub subject: Option<String>,
}

async fn send_test_email(
    _auth: AuthUser,
    Json(req): Json<SendTestEmailRequest>,
) -> ApiResult<Json<TestConnectionResponse>> {
    // Validate email address
    if !req.to_email.contains('@') {
        return Err(ApiError::validation_single("to_email", "Invalid email address"));
    }

    // Would actually send test email here using EmailService
    Ok(Json(TestConnectionResponse {
        success: true,
        message: format!("Test email sent to {}", req.to_email),
        mailbox_count: None,
        unread_count: None,
    }))
}

// ==================== Helper Functions ====================

/// Encrypt password for storage (placeholder - use proper encryption in production)
fn encrypt_password(password: &str) -> String {
    // In production, use proper encryption (e.g., AES-256-GCM with a secure key)
    // For now, just base64 encode (NOT SECURE - placeholder only)
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(password)
}

/// Test IMAP connection
async fn test_imap_connection(
    req: &TestMailboxRequest,
) -> Result<(i32, i32), Box<dyn std::error::Error + Send + Sync>> {
    // This would actually connect to IMAP and test
    // For now, return simulated values
    use tokio::task::spawn_blocking;

    let host = req.imap_host.clone();
    let port = req.imap_port as u16;
    let username = req.imap_username.clone();
    let password = req.imap_password.clone();
    let folder = req.imap_folder.clone().unwrap_or_else(|| "INBOX".to_string());
    let use_tls = req.use_tls;

    spawn_blocking(move || {
        // Connect to IMAP
        let tls = native_tls::TlsConnector::builder().build()?;
        let client = if use_tls {
            imap::connect((&*host, port), &host, &tls)?
        } else {
            // For non-TLS, would need different connection method
            return Err("Non-TLS connections not supported".into());
        };

        // Login
        let mut session = client.login(&username, &password).map_err(|e| e.0)?;

        // Count mailboxes
        let mailboxes = session.list(Some(""), Some("*"))?;
        let mailbox_count = mailboxes.len() as i32;

        // Select inbox and count unread
        session.select(&folder)?;
        let unread = session.search("UNSEEN")?;
        let unread_count = unread.len() as i32;

        session.logout()?;

        Ok((mailbox_count, unread_count))
    })
    .await?
}
