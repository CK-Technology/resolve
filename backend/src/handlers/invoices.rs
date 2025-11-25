use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::Json,
    routing::{get, post, put, patch},
    Router,
};
use chrono::{Utc, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;
use crate::AppState;
use crate::auth::{extract_token, verify_token};

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceCreate {
    pub client_id: Uuid,
    pub contract_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub number: String,
    pub date: NaiveDate,
    pub due_date: NaiveDate,
    pub payment_terms: String,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub line_items: Vec<InvoiceLineItemCreate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceLineItemCreate {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub tax_rate: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceUpdate {
    pub date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub payment_terms: Option<String>,
    pub notes: Option<String>,
    pub terms: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub client_id: Option<Uuid>,
    pub status: Option<String>,
    pub overdue: Option<bool>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InvoiceWithDetails {
    pub id: Uuid,
    pub client_id: Uuid,
    pub client_name: String,
    pub contract_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub number: String,
    pub date: NaiveDate,
    pub due_date: NaiveDate,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
    pub balance: Decimal,
    pub status: String,
    pub payment_terms: String,
    pub late_fee_percentage: Option<Decimal>,
    pub discount_percentage: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub days_overdue: Option<i32>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InvoiceLineItem {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub line_total: Decimal,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentCreate {
    pub amount: Decimal,
    pub payment_date: NaiveDate,
    pub payment_method: Option<String>,
    pub reference_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub amount: Decimal,
    pub payment_date: NaiveDate,
    pub payment_method: Option<String>,
    pub reference_number: Option<String>,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

pub fn invoice_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_invoices).post(create_invoice))
        .route("/:id", get(get_invoice).put(update_invoice))
        .route("/:id/line-items", get(get_invoice_line_items))
        .route("/:id/payments", get(get_invoice_payments).post(add_payment))
        .route("/:id/send", patch(send_invoice))
        .route("/:id/pdf", get(generate_invoice_pdf))
        .route("/stats", get(get_invoice_stats))
        .route("/overdue", get(get_overdue_invoices))
}

async fn list_invoices(
    State(state): State<Arc<AppState>>,
    Query(params): Query<InvoiceQuery>,
) -> Result<Json<Vec<InvoiceWithDetails>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let invoices = sqlx::query_as::<_, InvoiceWithDetails>(&format!(
        "SELECT 
            i.id, i.client_id, c.name as client_name,
            i.contract_id, i.project_id, p.name as project_name,
            i.number, i.date, i.due_date,
            i.subtotal, i.tax_amount, i.total, i.balance,
            i.status, i.payment_terms,
            i.late_fee_percentage, i.discount_percentage, i.discount_amount,
            i.notes, i.terms,
            CASE WHEN i.due_date < CURRENT_DATE AND i.status != 'paid' 
                 THEN EXTRACT(days FROM CURRENT_DATE - i.due_date)::int
                 ELSE NULL END as days_overdue,
            i.created_at, i.updated_at
         FROM invoices i
         LEFT JOIN clients c ON i.client_id = c.id
         LEFT JOIN projects p ON i.project_id = p.id
         ORDER BY i.created_at DESC
         LIMIT {} OFFSET {}", limit, offset))
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching invoices: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(invoices))
}

async fn create_invoice(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<InvoiceCreate>,
) -> Result<(StatusCode, Json<InvoiceWithDetails>), StatusCode> {
    // Extract user from token
    let token = extract_token(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let _token_data = verify_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let invoice_id = Uuid::new_v4();
    let now = Utc::now();
    
    // Calculate totals from line items
    let mut subtotal = Decimal::ZERO;
    let mut tax_amount = Decimal::ZERO;
    
    for item in &payload.line_items {
        let line_total = item.quantity * item.unit_price;
        subtotal += line_total;
        
        if let Some(tax_rate) = item.tax_rate {
            tax_amount += line_total * (tax_rate / Decimal::from(100));
        }
    }
    
    let total = subtotal + tax_amount;
    
    // Start transaction
    let mut tx = state.db_pool.begin().await.map_err(|e| {
        tracing::error!("Error starting transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Insert invoice
    sqlx::query(
        "INSERT INTO invoices (
            id, client_id, contract_id, project_id, number, date, due_date,
            subtotal, tax_amount, total, balance, status, payment_terms,
            notes, terms, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)"
    )
    .bind(invoice_id)
    .bind(payload.client_id)
    .bind(payload.contract_id)
    .bind(payload.project_id)
    .bind(payload.number)
    .bind(payload.date)
    .bind(payload.due_date)
    .bind(subtotal)
    .bind(tax_amount)
    .bind(total)
    .bind(total) // initial balance equals total
    .bind("draft")
    .bind(payload.payment_terms)
    .bind(payload.notes)
    .bind(payload.terms)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error creating invoice: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Insert line items
    for item in payload.line_items {
        let line_item_id = Uuid::new_v4();
        let line_total = item.quantity * item.unit_price;
        let item_tax_amount = if let Some(tax_rate) = item.tax_rate {
            Some(line_total * (tax_rate / Decimal::from(100)))
        } else {
            None
        };
        
        sqlx::query(
            "INSERT INTO invoice_line_items (
                id, invoice_id, description, quantity, unit_price, 
                line_total, tax_rate, tax_amount
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(line_item_id)
        .bind(invoice_id)
        .bind(item.description)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(line_total)
        .bind(item.tax_rate)
        .bind(item_tax_amount)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Error creating invoice line item: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }
    
    tx.commit().await.map_err(|e| {
        tracing::error!("Error committing transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Fetch the created invoice
    let invoice = get_invoice_by_id(&state, invoice_id).await?;
    Ok((StatusCode::CREATED, Json(invoice)))
}

async fn get_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<InvoiceWithDetails>, StatusCode> {
    let invoice = get_invoice_by_id(&state, id).await?;
    Ok(Json(invoice))
}

async fn update_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<InvoiceUpdate>,
) -> Result<Json<InvoiceWithDetails>, StatusCode> {
    sqlx::query(
        "UPDATE invoices SET 
         date = COALESCE($2, date),
         due_date = COALESCE($3, due_date),
         status = COALESCE($4, status),
         payment_terms = COALESCE($5, payment_terms),
         notes = COALESCE($6, notes),
         terms = COALESCE($7, terms),
         updated_at = NOW()
         WHERE id = $1"
    )
    .bind(id)
    .bind(payload.date)
    .bind(payload.due_date)
    .bind(payload.status)
    .bind(payload.payment_terms)
    .bind(payload.notes)
    .bind(payload.terms)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating invoice: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let invoice = get_invoice_by_id(&state, id).await?;
    Ok(Json(invoice))
}

async fn get_invoice_line_items(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<InvoiceLineItem>>, StatusCode> {
    let line_items = sqlx::query_as::<_, InvoiceLineItem>(
        "SELECT id, invoice_id, description, quantity, unit_price, 
         line_total, tax_rate, tax_amount
         FROM invoice_line_items 
         WHERE invoice_id = $1 
         ORDER BY created_at"
    )
    .bind(id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching invoice line items: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(line_items))
}

async fn get_invoice_payments(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Payment>>, StatusCode> {
    let payments = sqlx::query_as::<_, Payment>(
        "SELECT id, invoice_id, amount, payment_date, payment_method,
         reference_number, notes, created_at
         FROM payments 
         WHERE invoice_id = $1 
         ORDER BY payment_date DESC"
    )
    .bind(id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching invoice payments: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(payments))
}

async fn add_payment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<PaymentCreate>,
) -> Result<(StatusCode, Json<Payment>), StatusCode> {
    // Extract user from token
    let token = extract_token(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let _token_data = verify_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let payment_id = Uuid::new_v4();
    let now = Utc::now();
    
    // Start transaction
    let mut tx = state.db_pool.begin().await.map_err(|e| {
        tracing::error!("Error starting transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Insert payment
    sqlx::query(
        "INSERT INTO payments (
            id, invoice_id, amount, payment_date, payment_method,
            reference_number, notes, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(payment_id)
    .bind(id)
    .bind(payload.amount)
    .bind(payload.payment_date)
    .bind(payload.payment_method)
    .bind(payload.reference_number)
    .bind(payload.notes)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error adding payment: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Update invoice balance
    sqlx::query(
        "UPDATE invoices SET 
         balance = balance - $2,
         status = CASE WHEN balance - $2 <= 0 THEN 'paid' ELSE 'partial' END,
         updated_at = NOW()
         WHERE id = $1"
    )
    .bind(id)
    .bind(payload.amount)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error updating invoice balance: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    tx.commit().await.map_err(|e| {
        tracing::error!("Error committing transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Fetch the created payment
    let payment = sqlx::query_as::<_, Payment>(
        "SELECT id, invoice_id, amount, payment_date, payment_method,
         reference_number, notes, created_at
         FROM payments WHERE id = $1"
    )
    .bind(payment_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching payment: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok((StatusCode::CREATED, Json(payment)))
}

async fn send_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Update invoice status to sent
    sqlx::query("UPDATE invoices SET status = 'sent', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error sending invoice: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // TODO: Actually send email to client
    
    Ok(StatusCode::OK)
}

async fn generate_invoice_pdf(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement PDF generation
    Ok(Json(serde_json::json!({
        "message": "PDF generation not yet implemented",
        "url": "/api/v1/invoices/{}/pdf"
    })))
}

#[derive(Debug, Serialize)]
pub struct InvoiceStats {
    pub total_invoices: i64,
    pub draft_invoices: i64,
    pub sent_invoices: i64,
    pub paid_invoices: i64,
    pub overdue_invoices: i64,
    pub total_outstanding: Decimal,
    pub total_overdue: Decimal,
    pub average_days_to_pay: Option<i32>,
}

async fn get_invoice_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<InvoiceStats>, StatusCode> {
    let stats = sqlx::query!(
        "SELECT 
            COUNT(*) as total_invoices,
            COUNT(*) FILTER (WHERE status = 'draft') as draft_invoices,
            COUNT(*) FILTER (WHERE status = 'sent') as sent_invoices,
            COUNT(*) FILTER (WHERE status = 'paid') as paid_invoices,
            COUNT(*) FILTER (WHERE due_date < CURRENT_DATE AND status != 'paid') as overdue_invoices,
            COALESCE(SUM(balance) FILTER (WHERE status != 'paid'), 0) as total_outstanding,
            COALESCE(SUM(balance) FILTER (WHERE due_date < CURRENT_DATE AND status != 'paid'), 0) as total_overdue
         FROM invoices"
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching invoice stats: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let result = InvoiceStats {
        total_invoices: stats.total_invoices.unwrap_or(0),
        draft_invoices: stats.draft_invoices.unwrap_or(0),
        sent_invoices: stats.sent_invoices.unwrap_or(0),
        paid_invoices: stats.paid_invoices.unwrap_or(0),
        overdue_invoices: stats.overdue_invoices.unwrap_or(0),
        total_outstanding: stats.total_outstanding.unwrap_or_default(),
        total_overdue: stats.total_overdue.unwrap_or_default(),
        average_days_to_pay: None, // TODO: Calculate actual average
    };
    
    Ok(Json(result))
}

async fn get_overdue_invoices(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<InvoiceWithDetails>>, StatusCode> {
    let invoices = sqlx::query_as::<_, InvoiceWithDetails>(
        "SELECT 
            i.id, i.client_id, c.name as client_name,
            i.contract_id, i.project_id, p.name as project_name,
            i.number, i.date, i.due_date,
            i.subtotal, i.tax_amount, i.total, i.balance,
            i.status, i.payment_terms,
            i.late_fee_percentage, i.discount_percentage, i.discount_amount,
            i.notes, i.terms,
            EXTRACT(days FROM CURRENT_DATE - i.due_date)::int as days_overdue,
            i.created_at, i.updated_at
         FROM invoices i
         LEFT JOIN clients c ON i.client_id = c.id
         LEFT JOIN projects p ON i.project_id = p.id
         WHERE i.due_date < CURRENT_DATE AND i.status != 'paid'
         ORDER BY i.due_date ASC"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching overdue invoices: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(invoices))
}

// Helper functions
async fn get_invoice_by_id(state: &AppState, id: Uuid) -> Result<InvoiceWithDetails, StatusCode> {
    sqlx::query_as::<_, InvoiceWithDetails>(
        "SELECT 
            i.id, i.client_id, c.name as client_name,
            i.contract_id, i.project_id, p.name as project_name,
            i.number, i.date, i.due_date,
            i.subtotal, i.tax_amount, i.total, i.balance,
            i.status, i.payment_terms,
            i.late_fee_percentage, i.discount_percentage, i.discount_amount,
            i.notes, i.terms,
            CASE WHEN i.due_date < CURRENT_DATE AND i.status != 'paid' 
                 THEN EXTRACT(days FROM CURRENT_DATE - i.due_date)::int
                 ELSE NULL END as days_overdue,
            i.created_at, i.updated_at
         FROM invoices i
         LEFT JOIN clients c ON i.client_id = c.id
         LEFT JOIN projects p ON i.project_id = p.id
         WHERE i.id = $1"
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error fetching invoice: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })
}