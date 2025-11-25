//! Enhanced Billing and Invoicing Features
//!
//! Time-to-invoice workflow, recurring invoices, payment tracking, and credit notes.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post, put, delete},
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

// ==================== Structs ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecurringInvoiceTemplate {
    pub id: Uuid,
    pub client_id: Uuid,
    pub contract_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub frequency: String,
    pub interval_count: i32,
    pub day_of_month: Option<i32>,
    pub day_of_week: Option<i32>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub next_run_date: NaiveDate,
    pub last_run_date: Option<NaiveDate>,
    pub payment_terms: String,
    pub due_days: i32,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub subtotal: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub include_unbilled_time: bool,
    pub include_unbilled_expenses: bool,
    pub auto_send: bool,
    pub is_active: bool,
    pub run_count: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecurringTemplateWithDetails {
    #[serde(flatten)]
    pub template: RecurringInvoiceTemplate,
    pub client_name: String,
    pub contract_name: Option<String>,
    pub line_items: Vec<RecurringLineItem>,
    pub last_invoice_amount: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecurringLineItem {
    pub id: Uuid,
    pub template_id: Uuid,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub tax_rate: Option<Decimal>,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecurringTemplateRequest {
    pub client_id: Uuid,
    pub contract_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub frequency: String, // weekly, biweekly, monthly, quarterly, yearly
    pub interval_count: Option<i32>,
    pub day_of_month: Option<i32>,
    pub day_of_week: Option<i32>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub due_days: Option<i32>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub include_unbilled_time: Option<bool>,
    pub include_unbilled_expenses: Option<bool>,
    pub auto_send: Option<bool>,
    pub line_items: Vec<CreateRecurringLineItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecurringLineItemRequest {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub tax_rate: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecurringInvoiceRun {
    pub id: Uuid,
    pub template_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub run_date: NaiveDate,
    pub status: String,
    pub error_message: Option<String>,
    pub time_entries_count: i32,
    pub time_entries_amount: Decimal,
    pub fixed_items_amount: Decimal,
    pub total_amount: Decimal,
    pub created_at: DateTime<Utc>,
}

// ==================== Time to Invoice ====================

#[derive(Debug, Deserialize)]
pub struct UnbilledTimeQuery {
    pub client_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnbilledTimeSummary {
    pub client_id: Uuid,
    pub client_name: String,
    pub total_entries: i64,
    pub total_hours: Decimal,
    pub total_amount: Decimal,
    pub entries: Vec<UnbilledTimeEntry>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UnbilledTimeEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub ticket_id: Option<Uuid>,
    pub ticket_number: Option<i32>,
    pub ticket_subject: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub description: Option<String>,
    pub hourly_rate: Option<Decimal>,
    pub total_amount: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceFromTimeRequest {
    pub client_id: Uuid,
    pub time_entry_ids: Vec<Uuid>,
    pub invoice_date: NaiveDate,
    pub due_date: NaiveDate,
    pub payment_terms: Option<String>,
    pub notes: Option<String>,
    pub group_by: Option<String>, // "entry", "project", "ticket", "user"
    pub additional_line_items: Option<Vec<AdditionalLineItem>>,
}

#[derive(Debug, Deserialize)]
pub struct AdditionalLineItem {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub tax_rate: Option<Decimal>,
}

// ==================== Payment Methods ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub payment_type: String,
    pub provider: Option<String>,
    pub instructions: Option<String>,
    pub is_online: bool,
    pub is_default: bool,
    pub is_active: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

// ==================== Credit Notes ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CreditNote {
    pub id: Uuid,
    pub number: String,
    pub client_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub amount: Decimal,
    pub reason: Option<String>,
    pub status: String,
    pub applied_amount: Decimal,
    pub remaining_amount: Option<Decimal>,
    pub issued_date: Option<NaiveDate>,
    pub issued_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditNoteWithDetails {
    #[serde(flatten)]
    pub credit_note: CreditNote,
    pub client_name: String,
    pub invoice_number: Option<String>,
    pub issued_by_name: Option<String>,
    pub applications: Vec<CreditNoteApplication>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CreditNoteApplication {
    pub id: Uuid,
    pub credit_note_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_number: String,
    pub amount: Decimal,
    pub applied_at: DateTime<Utc>,
    pub applied_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCreditNoteRequest {
    pub client_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub amount: Decimal,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyCreditRequest {
    pub invoice_id: Uuid,
    pub amount: Decimal,
}

// ==================== Routes ====================

pub fn billing_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Time to Invoice
        .route("/unbilled-time", get(list_unbilled_time))
        .route("/unbilled-time/summary", get(get_unbilled_time_summary))
        .route("/create-from-time", post(create_invoice_from_time))
        // Recurring Invoices
        .route("/recurring", get(list_recurring_templates).post(create_recurring_template))
        .route("/recurring/:id", get(get_recurring_template).put(update_recurring_template).delete(delete_recurring_template))
        .route("/recurring/:id/run", post(run_recurring_invoice))
        .route("/recurring/:id/history", get(get_recurring_history))
        .route("/recurring/due", get(get_due_recurring_invoices))
        // Payment Methods
        .route("/payment-methods", get(list_payment_methods).post(create_payment_method))
        .route("/payment-methods/:id", put(update_payment_method).delete(delete_payment_method))
        // Credit Notes
        .route("/credit-notes", get(list_credit_notes).post(create_credit_note))
        .route("/credit-notes/:id", get(get_credit_note))
        .route("/credit-notes/:id/issue", post(issue_credit_note))
        .route("/credit-notes/:id/apply", post(apply_credit_note))
}

// ==================== Time to Invoice Handlers ====================

async fn list_unbilled_time(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(params): Query<UnbilledTimeQuery>,
) -> ApiResult<Json<Vec<UnbilledTimeEntry>>> {
    let entries = sqlx::query_as!(
        UnbilledTimeEntry,
        r#"SELECT
            te.id, te.user_id,
            COALESCE(u.first_name || ' ' || u.last_name, 'Unknown') as "user_name!",
            te.ticket_id, t.number as ticket_number, t.subject as ticket_subject,
            te.project_id, p.name as project_name,
            te.start_time, te.end_time, te.duration_minutes,
            te.description, te.hourly_rate, te.total_amount
         FROM time_entries te
         LEFT JOIN users u ON te.user_id = u.id
         LEFT JOIN tickets t ON te.ticket_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         WHERE te.billable = true
           AND te.billed = false
           AND te.end_time IS NOT NULL
           AND ($1::uuid IS NULL OR COALESCE(t.client_id, p.client_id) = $1)
           AND ($2::uuid IS NULL OR te.project_id = $2)
           AND ($3::uuid IS NULL OR te.user_id = $3)
           AND ($4::date IS NULL OR te.start_time::date >= $4)
           AND ($5::date IS NULL OR te.start_time::date <= $5)
         ORDER BY te.start_time DESC"#,
        params.client_id,
        params.project_id,
        params.user_id,
        params.from_date,
        params.to_date
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching unbilled time: {}", e);
        ApiError::internal("Failed to fetch unbilled time entries")
    })?;

    Ok(Json(entries))
}

async fn get_unbilled_time_summary(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<Vec<UnbilledTimeSummary>>> {
    // Get summary grouped by client
    let summaries = sqlx::query!(
        r#"SELECT
            COALESCE(t.client_id, p.client_id) as client_id,
            c.name as client_name,
            COUNT(te.id)::bigint as "total_entries!",
            COALESCE(SUM(te.duration_minutes), 0)::decimal / 60.0 as "total_hours!",
            COALESCE(SUM(te.total_amount), 0) as "total_amount!"
         FROM time_entries te
         LEFT JOIN tickets t ON te.ticket_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         LEFT JOIN clients c ON COALESCE(t.client_id, p.client_id) = c.id
         WHERE te.billable = true
           AND te.billed = false
           AND te.end_time IS NOT NULL
           AND COALESCE(t.client_id, p.client_id) IS NOT NULL
         GROUP BY COALESCE(t.client_id, p.client_id), c.name
         ORDER BY "total_amount!" DESC"#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching unbilled time summary: {}", e);
        ApiError::internal("Failed to fetch unbilled time summary")
    })?;

    let result: Vec<UnbilledTimeSummary> = summaries
        .into_iter()
        .filter_map(|row| {
            Some(UnbilledTimeSummary {
                client_id: row.client_id?,
                client_name: row.client_name.unwrap_or_else(|| "Unknown".to_string()),
                total_entries: row.total_entries,
                total_hours: row.total_hours,
                total_amount: row.total_amount,
                entries: vec![], // Entries fetched separately if needed
            })
        })
        .collect();

    Ok(Json(result))
}

async fn create_invoice_from_time(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateInvoiceFromTimeRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if payload.time_entry_ids.is_empty() {
        return Err(ApiError::validation_single("time_entry_ids", "At least one time entry is required"));
    }

    let mut tx = state.db_pool.begin().await.map_err(|e| {
        tracing::error!("Error starting transaction: {}", e);
        ApiError::internal("Failed to start transaction")
    })?;

    // Verify all time entries exist, are unbilled, and belong to the client
    let entries = sqlx::query!(
        r#"SELECT te.id, te.description, te.duration_minutes, te.hourly_rate, te.total_amount,
                  t.subject as ticket_subject, p.name as project_name,
                  u.first_name || ' ' || u.last_name as user_name,
                  COALESCE(t.client_id, p.client_id) as client_id
           FROM time_entries te
           LEFT JOIN tickets t ON te.ticket_id = t.id
           LEFT JOIN projects p ON te.project_id = p.id
           LEFT JOIN users u ON te.user_id = u.id
           WHERE te.id = ANY($1)
             AND te.billable = true
             AND te.billed = false"#,
        &payload.time_entry_ids
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching time entries: {}", e);
        ApiError::internal("Failed to fetch time entries")
    })?;

    if entries.len() != payload.time_entry_ids.len() {
        return Err(ApiError::validation_single(
            "time_entry_ids",
            "Some time entries are invalid, already billed, or not billable"
        ));
    }

    // Verify all entries belong to the same client
    for entry in &entries {
        if entry.client_id != Some(payload.client_id) {
            return Err(ApiError::validation_single(
                "client_id",
                "All time entries must belong to the specified client"
            ));
        }
    }

    // Calculate totals
    let mut subtotal = Decimal::ZERO;
    let mut line_items_data: Vec<(String, Decimal, Decimal)> = Vec::new();

    let group_by = payload.group_by.as_deref().unwrap_or("entry");

    match group_by {
        "entry" => {
            // Each time entry becomes a line item
            for entry in &entries {
                let hours = Decimal::from(entry.duration_minutes.unwrap_or(0)) / Decimal::from(60);
                let rate = entry.hourly_rate.unwrap_or(Decimal::from(75));
                let amount = entry.total_amount.unwrap_or(hours * rate);
                let desc = format!(
                    "{} - {} ({:.2} hrs)",
                    entry.ticket_subject.as_deref()
                        .or(entry.project_name.as_deref())
                        .unwrap_or("General"),
                    entry.description.as_deref().unwrap_or("Time entry"),
                    hours
                );
                line_items_data.push((desc, hours, rate));
                subtotal += amount;
            }
        }
        _ => {
            // Aggregate all entries into one line item
            let total_minutes: i32 = entries.iter()
                .filter_map(|e| e.duration_minutes)
                .sum();
            let hours = Decimal::from(total_minutes) / Decimal::from(60);
            let total_amount: Decimal = entries.iter()
                .filter_map(|e| e.total_amount)
                .sum();
            let avg_rate = if hours > Decimal::ZERO {
                total_amount / hours
            } else {
                Decimal::from(75)
            };

            line_items_data.push((
                format!("Professional Services ({:.2} hours)", hours),
                hours,
                avg_rate
            ));
            subtotal = total_amount;
        }
    }

    // Add any additional line items
    if let Some(additional) = &payload.additional_line_items {
        for item in additional {
            let line_total = item.quantity * item.unit_price;
            line_items_data.push((item.description.clone(), item.quantity, item.unit_price));
            subtotal += line_total;
        }
    }

    // Generate invoice number
    let invoice_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM invoices")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Error counting invoices: {}", e);
            ApiError::internal("Failed to generate invoice number")
        })?
        .unwrap_or(0);

    let invoice_number = format!("INV-{:05}", invoice_count + 1);
    let invoice_id = Uuid::new_v4();

    // Create the invoice
    sqlx::query!(
        r#"INSERT INTO invoices (
            id, client_id, number, date, due_date,
            subtotal, tax_amount, total, balance,
            status, payment_terms, notes, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, 0, $6, $6, 'draft', $7, $8, NOW())"#,
        invoice_id,
        payload.client_id,
        invoice_number,
        payload.invoice_date,
        payload.due_date,
        subtotal,
        payload.payment_terms.as_deref().unwrap_or("net_30"),
        payload.notes
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error creating invoice: {}", e);
        ApiError::internal("Failed to create invoice")
    })?;

    // Create line items
    for (i, (description, quantity, unit_price)) in line_items_data.iter().enumerate() {
        let line_item_id = Uuid::new_v4();
        let line_total = *quantity * *unit_price;

        sqlx::query!(
            r#"INSERT INTO invoice_line_items (
                id, invoice_id, description, quantity, unit_price, line_total, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, NOW())"#,
            line_item_id,
            invoice_id,
            description,
            quantity,
            unit_price,
            line_total
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Error creating invoice line item: {}", e);
            ApiError::internal("Failed to create invoice line item")
        })?;
    }

    // Mark time entries as billed
    sqlx::query!(
        "UPDATE time_entries SET billed = true, invoice_id = $1, updated_at = NOW() WHERE id = ANY($2)",
        invoice_id,
        &payload.time_entry_ids
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error marking time entries as billed: {}", e);
        ApiError::internal("Failed to update time entries")
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("Error committing transaction: {}", e);
        ApiError::internal("Failed to commit transaction")
    })?;

    Ok(Json(serde_json::json!({
        "invoice_id": invoice_id,
        "invoice_number": invoice_number,
        "subtotal": subtotal,
        "time_entries_billed": payload.time_entry_ids.len()
    })))
}

// ==================== Recurring Invoice Handlers ====================

async fn list_recurring_templates(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<Vec<RecurringTemplateWithDetails>>> {
    let templates = sqlx::query_as!(
        RecurringInvoiceTemplate,
        r#"SELECT id, client_id, contract_id, name, description,
                  frequency, interval_count, day_of_month, day_of_week,
                  start_date, end_date, next_run_date, last_run_date,
                  payment_terms, due_days, notes, terms,
                  subtotal, tax_rate,
                  include_unbilled_time, include_unbilled_expenses,
                  auto_send, is_active, run_count, created_by, created_at, updated_at
           FROM recurring_invoice_templates
           WHERE is_active = true
           ORDER BY next_run_date ASC"#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching recurring templates: {}", e);
        ApiError::internal("Failed to fetch recurring templates")
    })?;

    let mut result: Vec<RecurringTemplateWithDetails> = Vec::new();

    for template in templates {
        // Get client name
        let client = sqlx::query!("SELECT name FROM clients WHERE id = $1", template.client_id)
            .fetch_optional(&state.db_pool)
            .await
            .ok()
            .flatten();

        // Get line items
        let line_items = sqlx::query_as!(
            RecurringLineItem,
            r#"SELECT id, template_id, description, quantity, unit_price, tax_rate, display_order, created_at
               FROM recurring_invoice_line_items
               WHERE template_id = $1
               ORDER BY display_order"#,
            template.id
        )
        .fetch_all(&state.db_pool)
        .await
        .unwrap_or_default();

        // Get last invoice amount
        let last_run = sqlx::query_scalar!(
            "SELECT total_amount FROM recurring_invoice_runs WHERE template_id = $1 ORDER BY run_date DESC LIMIT 1",
            template.id
        )
        .fetch_optional(&state.db_pool)
        .await
        .ok()
        .flatten()
        .flatten();

        result.push(RecurringTemplateWithDetails {
            client_name: client.map(|c| c.name).unwrap_or_else(|| "Unknown".to_string()),
            contract_name: None,
            line_items,
            last_invoice_amount: last_run,
            template,
        });
    }

    Ok(Json(result))
}

async fn create_recurring_template(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateRecurringTemplateRequest>,
) -> ApiResult<Json<RecurringInvoiceTemplate>> {
    // Validate frequency
    let valid_frequencies = ["weekly", "biweekly", "monthly", "quarterly", "yearly"];
    if !valid_frequencies.contains(&payload.frequency.as_str()) {
        return Err(ApiError::validation_single("frequency", "Invalid frequency"));
    }

    let mut tx = state.db_pool.begin().await.map_err(|e| {
        tracing::error!("Error starting transaction: {}", e);
        ApiError::internal("Failed to start transaction")
    })?;

    let template_id = Uuid::new_v4();

    // Calculate subtotal from line items
    let subtotal: Decimal = payload.line_items.iter()
        .map(|item| item.quantity * item.unit_price)
        .sum();

    sqlx::query!(
        r#"INSERT INTO recurring_invoice_templates (
            id, client_id, contract_id, name, description,
            frequency, interval_count, day_of_month, day_of_week,
            start_date, end_date, next_run_date,
            payment_terms, due_days, notes, terms,
            subtotal, include_unbilled_time, include_unbilled_expenses,
            auto_send, created_by, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $10, $12, $13, $14, $15, $16, $17, $18, $19, $20, NOW())"#,
        template_id,
        payload.client_id,
        payload.contract_id,
        payload.name,
        payload.description,
        payload.frequency,
        payload.interval_count.unwrap_or(1),
        payload.day_of_month,
        payload.day_of_week,
        payload.start_date,
        payload.end_date,
        payload.payment_terms.as_deref().unwrap_or("net_30"),
        payload.due_days.unwrap_or(30),
        payload.notes,
        payload.terms,
        subtotal,
        payload.include_unbilled_time.unwrap_or(true),
        payload.include_unbilled_expenses.unwrap_or(true),
        payload.auto_send.unwrap_or(false),
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error creating recurring template: {}", e);
        ApiError::internal("Failed to create recurring template")
    })?;

    // Insert line items
    for (i, item) in payload.line_items.iter().enumerate() {
        sqlx::query!(
            r#"INSERT INTO recurring_invoice_line_items (
                template_id, description, quantity, unit_price, tax_rate, display_order
            ) VALUES ($1, $2, $3, $4, $5, $6)"#,
            template_id,
            item.description,
            item.quantity,
            item.unit_price,
            item.tax_rate,
            i as i32
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Error creating recurring line item: {}", e);
            ApiError::internal("Failed to create recurring line item")
        })?;
    }

    tx.commit().await.map_err(|e| {
        tracing::error!("Error committing transaction: {}", e);
        ApiError::internal("Failed to commit transaction")
    })?;

    let template = sqlx::query_as!(
        RecurringInvoiceTemplate,
        "SELECT * FROM recurring_invoice_templates WHERE id = $1",
        template_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching created template: {}", e);
        ApiError::internal("Failed to fetch created template")
    })?;

    Ok(Json(template))
}

async fn get_recurring_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<RecurringTemplateWithDetails>> {
    let template = sqlx::query_as!(
        RecurringInvoiceTemplate,
        "SELECT * FROM recurring_invoice_templates WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching recurring template: {}", e);
        ApiError::internal("Failed to fetch recurring template")
    })?
    .ok_or_else(|| ApiError::not_found("Recurring template not found"))?;

    let client = sqlx::query!("SELECT name FROM clients WHERE id = $1", template.client_id)
        .fetch_optional(&state.db_pool)
        .await
        .ok()
        .flatten();

    let line_items = sqlx::query_as!(
        RecurringLineItem,
        "SELECT * FROM recurring_invoice_line_items WHERE template_id = $1 ORDER BY display_order",
        id
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    let last_run = sqlx::query_scalar!(
        "SELECT total_amount FROM recurring_invoice_runs WHERE template_id = $1 ORDER BY run_date DESC LIMIT 1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .ok()
    .flatten()
    .flatten();

    Ok(Json(RecurringTemplateWithDetails {
        client_name: client.map(|c| c.name).unwrap_or_else(|| "Unknown".to_string()),
        contract_name: None,
        line_items,
        last_invoice_amount: last_run,
        template,
    }))
}

async fn update_recurring_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateRecurringTemplateRequest>,
) -> ApiResult<Json<RecurringInvoiceTemplate>> {
    let subtotal: Decimal = payload.line_items.iter()
        .map(|item| item.quantity * item.unit_price)
        .sum();

    let mut tx = state.db_pool.begin().await.map_err(|e| ApiError::internal("Transaction error"))?;

    sqlx::query!(
        r#"UPDATE recurring_invoice_templates SET
            client_id = $2, contract_id = $3, name = $4, description = $5,
            frequency = $6, interval_count = $7, day_of_month = $8, day_of_week = $9,
            start_date = $10, end_date = $11, payment_terms = $12, due_days = $13,
            notes = $14, terms = $15, subtotal = $16,
            include_unbilled_time = $17, include_unbilled_expenses = $18,
            auto_send = $19, updated_at = NOW()
           WHERE id = $1"#,
        id,
        payload.client_id,
        payload.contract_id,
        payload.name,
        payload.description,
        payload.frequency,
        payload.interval_count.unwrap_or(1),
        payload.day_of_month,
        payload.day_of_week,
        payload.start_date,
        payload.end_date,
        payload.payment_terms.as_deref().unwrap_or("net_30"),
        payload.due_days.unwrap_or(30),
        payload.notes,
        payload.terms,
        subtotal,
        payload.include_unbilled_time.unwrap_or(true),
        payload.include_unbilled_expenses.unwrap_or(true),
        payload.auto_send.unwrap_or(false)
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error updating recurring template: {}", e);
        ApiError::internal("Failed to update recurring template")
    })?;

    // Replace line items
    sqlx::query!("DELETE FROM recurring_invoice_line_items WHERE template_id = $1", id)
        .execute(&mut *tx)
        .await?;

    for (i, item) in payload.line_items.iter().enumerate() {
        sqlx::query!(
            r#"INSERT INTO recurring_invoice_line_items (
                template_id, description, quantity, unit_price, tax_rate, display_order
            ) VALUES ($1, $2, $3, $4, $5, $6)"#,
            id, item.description, item.quantity, item.unit_price, item.tax_rate, i as i32
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let template = sqlx::query_as!(
        RecurringInvoiceTemplate,
        "SELECT * FROM recurring_invoice_templates WHERE id = $1",
        id
    )
    .fetch_one(&state.db_pool)
    .await?;

    Ok(Json(template))
}

async fn delete_recurring_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    sqlx::query!("UPDATE recurring_invoice_templates SET is_active = false, updated_at = NOW() WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting recurring template: {}", e);
            ApiError::internal("Failed to delete recurring template")
        })?;

    Ok(())
}

async fn run_recurring_invoice(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // Fetch template
    let template = sqlx::query_as!(
        RecurringInvoiceTemplate,
        "SELECT * FROM recurring_invoice_templates WHERE id = $1 AND is_active = true",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::internal("Failed to fetch template"))?
    .ok_or_else(|| ApiError::not_found("Template not found"))?;

    // Start transaction
    let mut tx = state.db_pool.begin().await.map_err(|e| ApiError::internal("Transaction error"))?;

    // Get line items
    let line_items = sqlx::query_as!(
        RecurringLineItem,
        "SELECT * FROM recurring_invoice_line_items WHERE template_id = $1 ORDER BY display_order",
        id
    )
    .fetch_all(&mut *tx)
    .await?;

    // Calculate fixed items total
    let fixed_items_amount: Decimal = line_items.iter()
        .map(|item| item.quantity * item.unit_price)
        .sum();

    // Get unbilled time entries if enabled
    let mut time_entries_count = 0i32;
    let mut time_entries_amount = Decimal::ZERO;
    let time_entry_ids: Vec<Uuid> = if template.include_unbilled_time {
        let entries = sqlx::query!(
            r#"SELECT id, total_amount
               FROM time_entries te
               LEFT JOIN tickets t ON te.ticket_id = t.id
               LEFT JOIN projects p ON te.project_id = p.id
               WHERE te.billable = true
                 AND te.billed = false
                 AND te.end_time IS NOT NULL
                 AND COALESCE(t.client_id, p.client_id) = $1"#,
            template.client_id
        )
        .fetch_all(&mut *tx)
        .await?;

        time_entries_count = entries.len() as i32;
        time_entries_amount = entries.iter()
            .filter_map(|e| e.total_amount)
            .sum();

        entries.into_iter().map(|e| e.id).collect()
    } else {
        vec![]
    };

    let total_amount = fixed_items_amount + time_entries_amount;

    // Generate invoice number
    let invoice_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM invoices")
        .fetch_one(&mut *tx)
        .await?
        .unwrap_or(0);

    let invoice_number = format!("INV-{:05}", invoice_count + 1);
    let invoice_id = Uuid::new_v4();
    let today = Utc::now().date_naive();
    let due_date = today + chrono::Duration::days(template.due_days as i64);

    // Create invoice
    sqlx::query!(
        r#"INSERT INTO invoices (
            id, client_id, contract_id, number, date, due_date,
            subtotal, tax_amount, total, balance,
            status, payment_terms, notes, terms, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 0, $7, $7, 'draft', $8, $9, $10, NOW())"#,
        invoice_id,
        template.client_id,
        template.contract_id,
        invoice_number,
        today,
        due_date,
        total_amount,
        template.payment_terms,
        template.notes,
        template.terms
    )
    .execute(&mut *tx)
    .await?;

    // Create line items from fixed items
    for item in &line_items {
        let line_total = item.quantity * item.unit_price;
        sqlx::query!(
            r#"INSERT INTO invoice_line_items (invoice_id, description, quantity, unit_price, line_total, tax_rate)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
            invoice_id, item.description, item.quantity, item.unit_price, line_total, item.tax_rate
        )
        .execute(&mut *tx)
        .await?;
    }

    // Add time entries as line item if applicable
    if time_entries_count > 0 {
        let total_hours = time_entries_amount / Decimal::from(75); // Approximate
        sqlx::query!(
            r#"INSERT INTO invoice_line_items (invoice_id, description, quantity, unit_price, line_total)
               VALUES ($1, $2, $3, $4, $5)"#,
            invoice_id,
            format!("Professional Services ({} time entries)", time_entries_count),
            total_hours,
            Decimal::from(75),
            time_entries_amount
        )
        .execute(&mut *tx)
        .await?;

        // Mark time entries as billed
        sqlx::query!(
            "UPDATE time_entries SET billed = true, invoice_id = $1, updated_at = NOW() WHERE id = ANY($2)",
            invoice_id,
            &time_entry_ids
        )
        .execute(&mut *tx)
        .await?;
    }

    // Record the run
    let run_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO recurring_invoice_runs (
            id, template_id, invoice_id, run_date, status,
            time_entries_count, time_entries_amount, fixed_items_amount, total_amount
        ) VALUES ($1, $2, $3, $4, 'success', $5, $6, $7, $8)"#,
        run_id, id, invoice_id, today, time_entries_count, time_entries_amount, fixed_items_amount, total_amount
    )
    .execute(&mut *tx)
    .await?;

    // Update template: next_run_date, last_run_date, run_count
    sqlx::query!(
        r#"UPDATE recurring_invoice_templates SET
            last_run_date = $2,
            next_run_date = calculate_next_run_date(frequency, interval_count, $2, day_of_month, day_of_week),
            run_count = run_count + 1,
            updated_at = NOW()
           WHERE id = $1"#,
        id, today
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(serde_json::json!({
        "invoice_id": invoice_id,
        "invoice_number": invoice_number,
        "total_amount": total_amount,
        "fixed_items_amount": fixed_items_amount,
        "time_entries_count": time_entries_count,
        "time_entries_amount": time_entries_amount
    })))
}

async fn get_recurring_history(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<RecurringInvoiceRun>>> {
    let runs = sqlx::query_as!(
        RecurringInvoiceRun,
        "SELECT * FROM recurring_invoice_runs WHERE template_id = $1 ORDER BY run_date DESC LIMIT 50",
        id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching recurring history: {}", e);
        ApiError::internal("Failed to fetch recurring history")
    })?;

    Ok(Json(runs))
}

async fn get_due_recurring_invoices(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<RecurringTemplateWithDetails>>> {
    let today = Utc::now().date_naive();

    let templates = sqlx::query_as!(
        RecurringInvoiceTemplate,
        r#"SELECT * FROM recurring_invoice_templates
           WHERE is_active = true AND next_run_date <= $1
             AND (end_date IS NULL OR end_date >= $1)
           ORDER BY next_run_date ASC"#,
        today
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching due recurring invoices: {}", e);
        ApiError::internal("Failed to fetch due recurring invoices")
    })?;

    let mut result = Vec::new();
    for template in templates {
        let client = sqlx::query!("SELECT name FROM clients WHERE id = $1", template.client_id)
            .fetch_optional(&state.db_pool)
            .await.ok().flatten();

        result.push(RecurringTemplateWithDetails {
            client_name: client.map(|c| c.name).unwrap_or("Unknown".to_string()),
            contract_name: None,
            line_items: vec![],
            last_invoice_amount: None,
            template,
        });
    }

    Ok(Json(result))
}

// ==================== Payment Methods Handlers ====================

async fn list_payment_methods(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<PaymentMethod>>> {
    let methods = sqlx::query_as!(
        PaymentMethod,
        r#"SELECT id, name, type as payment_type, provider, instructions,
                  is_online, is_default, is_active, display_order, created_at
           FROM payment_methods
           WHERE is_active = true
           ORDER BY display_order"#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching payment methods: {}", e);
        ApiError::internal("Failed to fetch payment methods")
    })?;

    Ok(Json(methods))
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentMethodRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub payment_type: String,
    pub provider: Option<String>,
    pub instructions: Option<String>,
    pub is_online: Option<bool>,
    pub is_default: Option<bool>,
}

async fn create_payment_method(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreatePaymentMethodRequest>,
) -> ApiResult<Json<PaymentMethod>> {
    let id = Uuid::new_v4();

    // If setting as default, unset other defaults
    if payload.is_default.unwrap_or(false) {
        sqlx::query!("UPDATE payment_methods SET is_default = false")
            .execute(&state.db_pool)
            .await?;
    }

    sqlx::query!(
        r#"INSERT INTO payment_methods (id, name, type, provider, instructions, is_online, is_default)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        id,
        payload.name,
        payload.payment_type,
        payload.provider,
        payload.instructions,
        payload.is_online.unwrap_or(false),
        payload.is_default.unwrap_or(false)
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating payment method: {}", e);
        ApiError::internal("Failed to create payment method")
    })?;

    let method = sqlx::query_as!(
        PaymentMethod,
        r#"SELECT id, name, type as payment_type, provider, instructions,
                  is_online, is_default, is_active, display_order, created_at
           FROM payment_methods WHERE id = $1"#,
        id
    )
    .fetch_one(&state.db_pool)
    .await?;

    Ok(Json(method))
}

async fn update_payment_method(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreatePaymentMethodRequest>,
) -> ApiResult<Json<PaymentMethod>> {
    if payload.is_default.unwrap_or(false) {
        sqlx::query!("UPDATE payment_methods SET is_default = false WHERE id != $1", id)
            .execute(&state.db_pool)
            .await?;
    }

    sqlx::query!(
        r#"UPDATE payment_methods SET
            name = $2, type = $3, provider = $4, instructions = $5,
            is_online = $6, is_default = $7
           WHERE id = $1"#,
        id,
        payload.name,
        payload.payment_type,
        payload.provider,
        payload.instructions,
        payload.is_online.unwrap_or(false),
        payload.is_default.unwrap_or(false)
    )
    .execute(&state.db_pool)
    .await?;

    let method = sqlx::query_as!(
        PaymentMethod,
        r#"SELECT id, name, type as payment_type, provider, instructions,
                  is_online, is_default, is_active, display_order, created_at
           FROM payment_methods WHERE id = $1"#,
        id
    )
    .fetch_one(&state.db_pool)
    .await?;

    Ok(Json(method))
}

async fn delete_payment_method(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    sqlx::query!("UPDATE payment_methods SET is_active = false WHERE id = $1", id)
        .execute(&state.db_pool)
        .await?;
    Ok(())
}

// ==================== Credit Notes Handlers ====================

async fn list_credit_notes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<Vec<CreditNoteWithDetails>>> {
    let notes = sqlx::query_as!(
        CreditNote,
        "SELECT * FROM credit_notes ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching credit notes: {}", e);
        ApiError::internal("Failed to fetch credit notes")
    })?;

    let mut result = Vec::new();
    for note in notes {
        let client = sqlx::query!("SELECT name FROM clients WHERE id = $1", note.client_id)
            .fetch_optional(&state.db_pool)
            .await.ok().flatten();

        result.push(CreditNoteWithDetails {
            client_name: client.map(|c| c.name).unwrap_or("Unknown".to_string()),
            invoice_number: None,
            issued_by_name: None,
            applications: vec![],
            credit_note: note,
        });
    }

    Ok(Json(result))
}

async fn create_credit_note(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateCreditNoteRequest>,
) -> ApiResult<Json<CreditNote>> {
    // Generate credit note number
    let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM credit_notes")
        .fetch_one(&state.db_pool)
        .await?
        .unwrap_or(0);

    let number = format!("CN-{:05}", count + 1);
    let id = Uuid::new_v4();

    sqlx::query!(
        r#"INSERT INTO credit_notes (id, number, client_id, invoice_id, amount, reason, remaining_amount)
           VALUES ($1, $2, $3, $4, $5, $6, $5)"#,
        id, number, payload.client_id, payload.invoice_id, payload.amount, payload.reason
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating credit note: {}", e);
        ApiError::internal("Failed to create credit note")
    })?;

    let note = sqlx::query_as!(CreditNote, "SELECT * FROM credit_notes WHERE id = $1", id)
        .fetch_one(&state.db_pool)
        .await?;

    Ok(Json(note))
}

async fn get_credit_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CreditNoteWithDetails>> {
    let note = sqlx::query_as!(CreditNote, "SELECT * FROM credit_notes WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await?
        .ok_or_else(|| ApiError::not_found("Credit note not found"))?;

    let client = sqlx::query!("SELECT name FROM clients WHERE id = $1", note.client_id)
        .fetch_optional(&state.db_pool)
        .await.ok().flatten();

    let applications = sqlx::query_as!(
        CreditNoteApplication,
        r#"SELECT cna.id, cna.credit_note_id, cna.invoice_id, i.number as "invoice_number!",
                  cna.amount, cna.applied_at, cna.applied_by
           FROM credit_note_applications cna
           JOIN invoices i ON cna.invoice_id = i.id
           WHERE cna.credit_note_id = $1
           ORDER BY cna.applied_at DESC"#,
        id
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    Ok(Json(CreditNoteWithDetails {
        client_name: client.map(|c| c.name).unwrap_or("Unknown".to_string()),
        invoice_number: None,
        issued_by_name: None,
        applications,
        credit_note: note,
    }))
}

async fn issue_credit_note(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CreditNote>> {
    let today = Utc::now().date_naive();

    sqlx::query!(
        "UPDATE credit_notes SET status = 'issued', issued_date = $2, issued_by = $3, updated_at = NOW() WHERE id = $1",
        id, today, user.id
    )
    .execute(&state.db_pool)
    .await?;

    let note = sqlx::query_as!(CreditNote, "SELECT * FROM credit_notes WHERE id = $1", id)
        .fetch_one(&state.db_pool)
        .await?;

    Ok(Json(note))
}

async fn apply_credit_note(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApplyCreditRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify credit note exists and has sufficient remaining amount
    let note = sqlx::query_as!(CreditNote, "SELECT * FROM credit_notes WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await?
        .ok_or_else(|| ApiError::not_found("Credit note not found"))?;

    if note.status != "issued" {
        return Err(ApiError::validation_single("id", "Credit note must be issued before applying"));
    }

    let remaining = note.remaining_amount.unwrap_or(note.amount - note.applied_amount);
    if payload.amount > remaining {
        return Err(ApiError::validation_single("amount", "Amount exceeds remaining credit"));
    }

    let mut tx = state.db_pool.begin().await?;

    // Create application record
    sqlx::query!(
        "INSERT INTO credit_note_applications (credit_note_id, invoice_id, amount, applied_by) VALUES ($1, $2, $3, $4)",
        id, payload.invoice_id, payload.amount, user.id
    )
    .execute(&mut *tx)
    .await?;

    // Update invoice balance
    sqlx::query!(
        r#"UPDATE invoices SET
            balance = balance - $2,
            status = CASE WHEN balance - $2 <= 0 THEN 'paid' ELSE status END,
            updated_at = NOW()
           WHERE id = $1"#,
        payload.invoice_id, payload.amount
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(serde_json::json!({
        "credit_note_id": id,
        "invoice_id": payload.invoice_id,
        "amount_applied": payload.amount
    })))
}
