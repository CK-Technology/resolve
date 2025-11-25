use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::Json,
    routing::{get, post, put},
    Router,
};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct PortalLoginRequest {
    pub email: String,
    pub password: String,
    pub client_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortalLoginResponse {
    pub token: String,
    pub contact: PortalContact,
    pub client: PortalClient,
    pub portal_settings: PortalSettings,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PortalContact {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PortalClient {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PortalSettings {
    pub id: Uuid,
    pub client_id: Uuid,
    pub is_enabled: bool,
    pub logo_url: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub custom_domain: Option<String>,
    pub welcome_message: Option<String>,
    pub show_tickets: bool,
    pub show_invoices: bool,
    pub show_assets: bool,
    pub show_knowledge_base: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PortalTicket {
    pub id: Uuid,
    pub number: i32,
    pub subject: String,
    pub details: String,
    pub status: String,
    pub priority: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
    pub last_reply_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PortalInvoice {
    pub id: Uuid,
    pub number: String,
    pub date: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub total: rust_decimal::Decimal,
    pub balance: rust_decimal::Decimal,
    pub status: String,
    pub pdf_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PortalAsset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub status: String,
    pub warranty_expire: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PortalTimeEntry {
    pub id: Uuid,
    pub date: chrono::NaiveDate,
    pub duration: i32,
    pub description: String,
    pub billable: bool,
    pub hourly_rate: Option<rust_decimal::Decimal>,
    pub task_name: Option<String>,
    pub project_name: Option<String>,
    pub technician_name: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestLoginRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestLoginResponse {
    pub access_token: String,
    pub client: PortalClient,
    pub expires_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortalDashboard {
    pub open_tickets: i64,
    pub pending_invoices: i64,
    pub total_assets: i64,
    pub outstanding_balance: rust_decimal::Decimal,
    pub monthly_hours: rust_decimal::Decimal,
    pub billable_hours_mtd: rust_decimal::Decimal,
    pub recent_tickets: Vec<PortalTicket>,
    pub recent_invoices: Vec<PortalInvoice>,
    pub recent_time_entries: Vec<PortalTimeEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePortalTicket {
    pub subject: String,
    pub details: String,
    pub priority: Option<String>,
    pub asset_id: Option<Uuid>,
}

pub fn portal_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Authentication
        .route("/login", post(portal_login))
        .route("/logout", post(portal_logout))
        .route("/verify", get(verify_portal_token))
        
        // Dashboard
        .route("/dashboard", get(get_portal_dashboard))
        
        // Tickets
        .route("/tickets", get(list_portal_tickets).post(create_portal_ticket))
        .route("/tickets/:id", get(get_portal_ticket))
        .route("/tickets/:id/replies", get(get_ticket_replies).post(add_ticket_reply))
        
        // Invoices
        .route("/invoices", get(list_portal_invoices))
        .route("/invoices/:id", get(get_portal_invoice))
        .route("/invoices/:id/pdf", get(download_invoice_pdf))
        
        // Assets
        .route("/assets", get(list_portal_assets))
        .route("/assets/:id", get(get_portal_asset))
        
        // Time Tracking
        .route("/time-entries", get(list_portal_time_entries))
        
        // Guest Access
        .route("/guest/login", post(guest_login))
        .route("/guest/tickets", get(list_guest_tickets))
        .route("/guest/time-entries", get(list_guest_time_entries))
        
        // Profile
        .route("/profile", get(get_portal_profile).put(update_portal_profile))
        .route("/profile/password", put(change_portal_password))
}

async fn portal_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PortalLoginRequest>,
) -> Result<Json<PortalLoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Find contact by email
    let contact = sqlx::query!(
        "SELECT c.*, cl.id as client_id, cl.name as client_name, 
         cl.email as client_email, cl.phone as client_phone
         FROM contacts c
         JOIN clients cl ON c.client_id = cl.id
         WHERE c.email = $1 AND c.archived_at IS NULL",
        payload.email
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error finding contact: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"}))
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid email or password"}))
        )
    })?;
    
    // Verify password (assuming we have a password_hash field on contacts)
    // For now, we'll create a simple token
    
    // Get or create portal settings
    let settings = sqlx::query_as::<_, PortalSettings>(
        "SELECT * FROM portal_settings WHERE client_id = $1"
    )
    .bind(contact.client_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching portal settings: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"}))
        )
    })?;
    
    let settings = match settings {
        Some(s) => s,
        None => {
            // Create default settings
            let settings_id = Uuid::new_v4();
            sqlx::query_as::<_, PortalSettings>(
                "INSERT INTO portal_settings (id, client_id, is_enabled, primary_color, secondary_color)
                 VALUES ($1, $2, true, '#3B82F6', '#1E40AF')
                 RETURNING *"
            )
            .bind(settings_id)
            .bind(contact.client_id)
            .fetch_one(&state.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Error creating portal settings: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Database error"}))
                )
            })?
        }
    };
    
    if !settings.is_enabled {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Portal access is disabled for this client"}))
        ));
    }
    
    // Generate portal access token
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(24);
    
    sqlx::query(
        "INSERT INTO portal_access_tokens (contact_id, token, expires_at)
         VALUES ($1, $2, $3)"
    )
    .bind(contact.id)
    .bind(&token)
    .bind(expires_at)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating access token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create session"}))
        )
    })?;
    
    Ok(Json(PortalLoginResponse {
        token,
        contact: PortalContact {
            id: contact.id,
            name: contact.name,
            email: contact.email,
            phone: contact.phone,
            title: contact.title,
        },
        client: PortalClient {
            id: contact.client_id,
            name: contact.client_name,
            email: contact.client_email,
            phone: contact.client_phone,
        },
        portal_settings: settings,
    }))
}

async fn portal_logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let token = extract_portal_token(&headers)?;
    
    sqlx::query("DELETE FROM portal_access_tokens WHERE token = $1")
        .bind(token)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting token: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(StatusCode::OK)
}

async fn verify_portal_token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<PortalContact>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (contact_id, _client_id) = verify_token(&state, &token).await?;
    
    let contact = sqlx::query_as::<_, PortalContact>(
        "SELECT id, name, email, phone, title FROM contacts WHERE id = $1"
    )
    .bind(contact_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching contact: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(contact))
}

async fn get_portal_dashboard(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<PortalDashboard>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (_contact_id, client_id) = verify_token(&state, &token).await?;
    
    // Get dashboard stats with simpler queries
    let open_tickets: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tickets WHERE client_id = $1 AND status IN ('open', 'in_progress')"
    )
    .bind(client_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);
    
    let pending_invoices: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM invoices WHERE client_id = $1 AND status != 'paid'"
    )
    .bind(client_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);
    
    let total_assets: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM assets WHERE client_id = $1 AND archived_at IS NULL"
    )
    .bind(client_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);
    
    let outstanding_balance: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(balance), 0) FROM invoices WHERE client_id = $1 AND status != 'paid'"
    )
    .bind(client_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or_default();
    
    let monthly_hours: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(duration), 0)::decimal / 60.0 FROM time_entries WHERE client_id = $1 AND date >= date_trunc('month', CURRENT_DATE)"
    )
    .bind(client_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or_default();
    
    let billable_hours_mtd: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(duration), 0)::decimal / 60.0 FROM time_entries WHERE client_id = $1 AND billable = true AND date >= date_trunc('month', CURRENT_DATE)"
    )
    .bind(client_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or_default();
    
    // Get recent tickets
    let recent_tickets = sqlx::query_as::<_, PortalTicket>(
        "SELECT id, number, subject, details, status, priority, created_at, updated_at, 
         (SELECT MAX(created_at) FROM ticket_replies WHERE ticket_id = tickets.id) as last_reply_at
         FROM tickets 
         WHERE client_id = $1 
         ORDER BY created_at DESC 
         LIMIT 5"
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching recent tickets: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Get recent invoices
    let recent_invoices = sqlx::query_as::<_, PortalInvoice>(
        "SELECT id, number, date, due_date, total, balance, status, NULL as pdf_url
         FROM invoices 
         WHERE client_id = $1 
         ORDER BY created_at DESC 
         LIMIT 5"
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching recent invoices: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Get recent time entries
    let recent_time_entries = sqlx::query_as::<_, PortalTimeEntry>(
        "SELECT te.id, te.date, te.duration, te.description, te.billable, te.hourly_rate,
         t.name as task_name, p.name as project_name,
         (u.first_name || ' ' || u.last_name) as technician_name, te.created_at
         FROM time_entries te
         LEFT JOIN tasks t ON te.task_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         LEFT JOIN users u ON te.user_id = u.id
         WHERE te.client_id = $1
         ORDER BY te.date DESC, te.created_at DESC
         LIMIT 10"
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching recent time entries: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(PortalDashboard {
        open_tickets,
        pending_invoices,
        total_assets,
        outstanding_balance,
        monthly_hours,
        billable_hours_mtd,
        recent_tickets,
        recent_invoices,
        recent_time_entries,
    }))
}

async fn list_portal_tickets(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<PortalTicket>>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (contact_id, client_id) = verify_token(&state, &token).await?;
    
    let tickets = sqlx::query_as::<_, PortalTicket>(
        "SELECT id, number, subject, details, status, priority, created_at, updated_at,
         (SELECT MAX(created_at) FROM ticket_replies WHERE ticket_id = tickets.id) as last_reply_at
         FROM tickets 
         WHERE client_id = $1 AND (contact_id = $2 OR contact_id IS NULL)
         ORDER BY created_at DESC"
    )
    .bind(client_id)
    .bind(contact_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching tickets: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(tickets))
}

async fn create_portal_ticket(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreatePortalTicket>,
) -> Result<(StatusCode, Json<PortalTicket>), StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (contact_id, client_id) = verify_token(&state, &token).await?;
    
    let ticket_id = Uuid::new_v4();
    let ticket_number = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM tickets WHERE client_id = $1",
        client_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error getting ticket number: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(1);
    
    let priority = payload.priority.unwrap_or_else(|| "medium".to_string());
    let now = Utc::now();
    
    let ticket = sqlx::query_as::<_, PortalTicket>(
        "INSERT INTO tickets (
            id, client_id, contact_id, asset_id, number, subject, details,
            status, priority, source, opened_by, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'open', $8, 'portal', $3, $9)
        RETURNING id, number, subject, details, status, priority, created_at, updated_at, NULL as last_reply_at"
    )
    .bind(ticket_id)
    .bind(client_id)
    .bind(contact_id)
    .bind(payload.asset_id)
    .bind(ticket_number)
    .bind(payload.subject)
    .bind(payload.details)
    .bind(priority)
    .bind(now)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating ticket: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Send notification to support team
    state.broadcast_notification(
        "portal_ticket_created",
        serde_json::json!({
            "ticket_id": ticket_id,
            "ticket_number": ticket_number,
            "subject": payload.subject,
            "client_id": client_id,
            "contact_id": contact_id
        })
    ).await;
    
    Ok((StatusCode::CREATED, Json(ticket)))
}

async fn get_portal_ticket(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<PortalTicket>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (_contact_id, client_id) = verify_token(&state, &token).await?;
    
    let ticket = sqlx::query_as::<_, PortalTicket>(
        "SELECT id, number, subject, details, status, priority, created_at, updated_at,
         (SELECT MAX(created_at) FROM ticket_replies WHERE ticket_id = tickets.id) as last_reply_at
         FROM tickets 
         WHERE id = $1 AND client_id = $2"
    )
    .bind(id)
    .bind(client_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error fetching ticket: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    
    Ok(Json(ticket))
}

async fn get_ticket_replies(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (_contact_id, client_id) = verify_token(&state, &token).await?;
    
    // Verify ticket belongs to client
    let _ticket = sqlx::query!(
        "SELECT id FROM tickets WHERE id = $1 AND client_id = $2",
        id,
        client_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error verifying ticket: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    
    let replies = sqlx::query!(
        "SELECT tr.id, tr.details as message, tr.created_at,
         (u.first_name || ' ' || u.last_name) as author_name,
         'support' as author_type
         FROM ticket_replies tr
         LEFT JOIN users u ON tr.user_id = u.id
         WHERE tr.ticket_id = $1
         ORDER BY tr.created_at ASC",
        id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching replies: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let result: Vec<serde_json::Value> = replies
        .into_iter()
        .map(|r| serde_json::json!({
            "id": r.id,
            "message": r.message,
            "created_at": r.created_at,
            "author_name": r.author_name,
            "author_type": r.author_type
        }))
        .collect();
    
    Ok(Json(result))
}

async fn add_ticket_reply(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (contact_id, client_id) = verify_token(&state, &token).await?;
    
    // Verify ticket belongs to client
    let _ticket = sqlx::query!(
        "SELECT id FROM tickets WHERE id = $1 AND client_id = $2",
        id,
        client_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error verifying ticket: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    
    let message = payload.get("message")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let reply_id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO ticket_replies (id, ticket_id, contact_id, message, internal_note)
         VALUES ($1, $2, $3, $4, false)"
    )
    .bind(reply_id)
    .bind(id)
    .bind(contact_id)
    .bind(message)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating reply: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Update ticket status if closed
    sqlx::query(
        "UPDATE tickets SET status = 'open', updated_at = NOW() 
         WHERE id = $1 AND status = 'closed'"
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Send notification
    state.broadcast_notification(
        "portal_ticket_reply",
        serde_json::json!({
            "ticket_id": id,
            "reply_id": reply_id,
            "contact_id": contact_id
        })
    ).await;
    
    Ok(StatusCode::CREATED)
}

async fn list_portal_invoices(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<PortalInvoice>>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (_contact_id, client_id) = verify_token(&state, &token).await?;
    
    let invoices = sqlx::query_as::<_, PortalInvoice>(
        "SELECT id, number, date, due_date, total, balance, status, NULL as pdf_url
         FROM invoices 
         WHERE client_id = $1
         ORDER BY created_at DESC"
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching invoices: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(invoices))
}

async fn get_portal_invoice(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<PortalInvoice>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (_contact_id, client_id) = verify_token(&state, &token).await?;
    
    let invoice = sqlx::query_as::<_, PortalInvoice>(
        "SELECT id, number, date, due_date, total, balance, status, NULL as pdf_url
         FROM invoices 
         WHERE id = $1 AND client_id = $2"
    )
    .bind(id)
    .bind(client_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error fetching invoice: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    
    Ok(Json(invoice))
}

async fn download_invoice_pdf(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _token = extract_portal_token(&headers)?;
    
    // TODO: Implement PDF generation
    Ok(Json(serde_json::json!({
        "message": "PDF download not yet implemented"
    })))
}

async fn list_portal_assets(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<PortalAsset>>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (_contact_id, client_id) = verify_token(&state, &token).await?;
    
    let assets = sqlx::query_as::<_, PortalAsset>(
        "SELECT id, name, asset_type, make, model, serial, status, warranty_expire
         FROM assets 
         WHERE client_id = $1 AND archived_at IS NULL
         ORDER BY name"
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching assets: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(assets))
}

async fn get_portal_asset(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<PortalAsset>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (_contact_id, client_id) = verify_token(&state, &token).await?;
    
    let asset = sqlx::query_as::<_, PortalAsset>(
        "SELECT id, name, asset_type, make, model, serial, status, warranty_expire
         FROM assets 
         WHERE id = $1 AND client_id = $2"
    )
    .bind(id)
    .bind(client_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error fetching asset: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    
    Ok(Json(asset))
}

async fn get_portal_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<PortalContact>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (contact_id, _client_id) = verify_token(&state, &token).await?;
    
    let contact = sqlx::query_as::<_, PortalContact>(
        "SELECT id, name, email, phone, title FROM contacts WHERE id = $1"
    )
    .bind(contact_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching profile: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(contact))
}

async fn update_portal_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<PortalContact>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (contact_id, _client_id) = verify_token(&state, &token).await?;
    
    let name = payload.get("name").and_then(|v| v.as_str());
    let phone = payload.get("phone").and_then(|v| v.as_str());
    let title = payload.get("title").and_then(|v| v.as_str());
    
    let contact = sqlx::query_as::<_, PortalContact>(
        "UPDATE contacts SET 
         name = COALESCE($2, name),
         phone = COALESCE($3, phone),
         title = COALESCE($4, title),
         updated_at = NOW()
         WHERE id = $1
         RETURNING id, name, email, phone, title"
    )
    .bind(contact_id)
    .bind(name)
    .bind(phone)
    .bind(title)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating profile: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(contact))
}

async fn change_portal_password(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(_payload): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let _token = extract_portal_token(&headers)?;
    
    // TODO: Implement password change
    Ok(StatusCode::OK)
}

async fn list_portal_time_entries(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<PortalTimeEntry>>, StatusCode> {
    let token = extract_portal_token(&headers)?;
    let (_contact_id, client_id) = verify_token(&state, &token).await?;
    
    let time_entries = sqlx::query_as::<_, PortalTimeEntry>(
        "SELECT te.id, te.date, te.duration, te.description, te.billable, te.hourly_rate,
         t.name as task_name, p.name as project_name,
         (u.first_name || ' ' || u.last_name) as technician_name, te.created_at
         FROM time_entries te
         LEFT JOIN tasks t ON te.task_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         LEFT JOIN users u ON te.user_id = u.id
         WHERE te.client_id = $1
         ORDER BY te.date DESC, te.created_at DESC"
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching time entries: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(time_entries))
}

async fn guest_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GuestLoginRequest>,
) -> Result<Json<GuestLoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Verify client portal token with simpler query
    let client_token_result: Option<(Uuid, chrono::DateTime<Utc>)> = sqlx::query_as(
        "SELECT client_id, expires_at FROM client_portal_tokens 
         WHERE token = $1 AND expires_at > NOW() AND is_active = true"
    )
    .bind(&payload.token)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error verifying client token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"}))
        )
    })?;
    
    let (client_id, _expires_at) = client_token_result.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or expired guest token"}))
        )
    })?;
    
    // Get client information
    let client = sqlx::query_as::<_, PortalClient>(
        "SELECT id, name, email, phone FROM clients WHERE id = $1"
    )
    .bind(client_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"}))
        )
    })?;
    
    // Generate guest access token
    let access_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(8); // Shorter session for guests
    
    sqlx::query(
        "INSERT INTO guest_access_tokens (client_id, token, expires_at, created_from_portal_token)
         VALUES ($1, $2, $3, $4)"
    )
    .bind(client_id)
    .bind(&access_token)
    .bind(expires_at)
    .bind(&payload.token)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating guest access token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create session"}))
        )
    })?;
    
    Ok(Json(GuestLoginResponse {
        access_token,
        client,
        expires_at,
    }))
}

async fn list_guest_tickets(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<PortalTicket>>, StatusCode> {
    let token = extract_guest_token(&headers)?;
    let client_id = verify_guest_token(&state, &token).await?;
    
    let tickets = sqlx::query_as::<_, PortalTicket>(
        "SELECT id, number, subject, details, status, priority, created_at, updated_at,
         (SELECT MAX(created_at) FROM ticket_replies WHERE ticket_id = tickets.id) as last_reply_at
         FROM tickets 
         WHERE client_id = $1
         ORDER BY created_at DESC"
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching guest tickets: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(tickets))
}

async fn list_guest_time_entries(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<PortalTimeEntry>>, StatusCode> {
    let token = extract_guest_token(&headers)?;
    let client_id = verify_guest_token(&state, &token).await?;
    
    let time_entries = sqlx::query_as::<_, PortalTimeEntry>(
        "SELECT te.id, te.date, te.duration, te.description, te.billable, te.hourly_rate,
         t.name as task_name, p.name as project_name,
         (u.first_name || ' ' || u.last_name) as technician_name, te.created_at
         FROM time_entries te
         LEFT JOIN tasks t ON te.task_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         LEFT JOIN users u ON te.user_id = u.id
         WHERE te.client_id = $1 AND te.billable = true
         ORDER BY te.date DESC, te.created_at DESC"
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching guest time entries: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(time_entries))
}

// Helper functions
fn extract_portal_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    headers
        .get("X-Portal-Token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn verify_token(state: &Arc<AppState>, token: &str) -> Result<(Uuid, Uuid), StatusCode> {
    let result = sqlx::query!(
        "SELECT pat.contact_id, c.client_id 
         FROM portal_access_tokens pat
         JOIN contacts c ON pat.contact_id = c.id
         WHERE pat.token = $1 AND pat.expires_at > NOW()",
        token
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error verifying token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    match result {
        Some(record) => {
            // Update last used time
            let _ = sqlx::query!(
                "UPDATE portal_access_tokens SET last_used_at = NOW() WHERE token = $1",
                token
            )
            .execute(&state.db_pool)
            .await;
            
            Ok((record.contact_id, record.client_id))
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

fn extract_guest_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    headers
        .get("X-Guest-Token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn verify_guest_token(state: &Arc<AppState>, token: &str) -> Result<Uuid, StatusCode> {
    let result: Option<Uuid> = sqlx::query_scalar(
        "SELECT client_id FROM guest_access_tokens 
         WHERE token = $1 AND expires_at > NOW()"
    )
    .bind(token)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error verifying guest token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    match result {
        Some(client_id) => {
            // Update last used time
            let _ = sqlx::query(
                "UPDATE guest_access_tokens SET last_used_at = NOW() WHERE token = $1"
            )
            .bind(token)
            .execute(&state.db_pool)
            .await;
            
            Ok(client_id)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}