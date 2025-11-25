use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put, delete},
    Router,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::AppState;
use resolve_shared::Domain;

pub fn domain_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_domains).post(create_domain))
        .route("/:id", get(get_domain).put(update_domain).delete(delete_domain))
        .route("/expiring", get(get_expiring_domains))
        .route("/:id/dns", get(get_dns_records).put(update_dns_records))
}

#[derive(Debug, Deserialize)]
pub struct ListDomainsQuery {
    pub client_id: Option<Uuid>,
    pub search: Option<String>,
    pub expiring_days: Option<i32>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDomainRequest {
    pub client_id: Uuid,
    pub name: String,
    pub registrar: Option<String>,
    pub nameservers: Option<Vec<String>>,
    pub registration_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub auto_renew: Option<bool>,
    pub dns_records: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DomainWithExpiry {
    #[serde(flatten)]
    pub domain: Domain,
    pub days_until_expiry: Option<i32>,
    pub is_expired: bool,
}

async fn list_domains(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListDomainsQuery>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    let domains = if let Some(client_id) = query.client_id {
        sqlx::query_as!(
            Domain,
            r#"
            SELECT id, client_id, name, registrar, nameservers, registration_date,
                   expiry_date, auto_renew, dns_records, notes, created_at, updated_at
            FROM domains
            WHERE client_id = $1
            ORDER BY name ASC
            LIMIT $2 OFFSET $3
            "#,
            client_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as!(
            Domain,
            r#"
            SELECT id, client_id, name, registrar, nameservers, registration_date,
                   expiry_date, auto_renew, dns_records, notes, created_at, updated_at
            FROM domains
            ORDER BY name ASC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    // Add expiry information
    let domains_with_expiry: Vec<DomainWithExpiry> = domains.into_iter().map(|domain| {
        let (days_until_expiry, is_expired) = if let Some(expiry_date) = domain.expiry_date {
            let today = chrono::Utc::now().date_naive();
            let days = (expiry_date - today).num_days() as i32;
            (Some(days), days < 0)
        } else {
            (None, false)
        };

        DomainWithExpiry {
            domain,
            days_until_expiry,
            is_expired,
        }
    }).collect();

    Ok(Json(domains_with_expiry))
}

async fn get_domain(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let domain = sqlx::query_as!(
        Domain,
        r#"
        SELECT id, client_id, name, registrar, nameservers, registration_date,
               expiry_date, auto_renew, dns_records, notes, created_at, updated_at
        FROM domains
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(domain))
}

async fn create_domain(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateDomainRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = Uuid::new_v4();
    let nameservers = req.nameservers.unwrap_or_default();
    let dns_records = req.dns_records.unwrap_or_else(|| serde_json::json!({}));

    sqlx::query!(
        r#"
        INSERT INTO domains (
            id, client_id, name, registrar, nameservers, registration_date,
            expiry_date, auto_renew, dns_records, notes, created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NULL
        )
        "#,
        id,
        req.client_id,
        req.name,
        req.registrar,
        &nameservers,
        req.registration_date,
        req.expiry_date,
        req.auto_renew.unwrap_or(false),
        dns_records,
        req.notes
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log the creation
    log_audit_action(&state.db_pool, auth.0.id, "CREATE", "domain", id).await;

    Ok(Json(serde_json::json!({ "id": id, "message": "Domain created successfully" })))
}

async fn update_domain(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(req): Json<CreateDomainRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let nameservers = req.nameservers.unwrap_or_default();
    let dns_records = req.dns_records.unwrap_or_else(|| serde_json::json!({}));

    let result = sqlx::query!(
        r#"
        UPDATE domains SET
            client_id = $2, name = $3, registrar = $4, nameservers = $5,
            registration_date = $6, expiry_date = $7, auto_renew = $8,
            dns_records = $9, notes = $10, updated_at = NOW()
        WHERE id = $1
        "#,
        id,
        req.client_id,
        req.name,
        req.registrar,
        &nameservers,
        req.registration_date,
        req.expiry_date,
        req.auto_renew.unwrap_or(false),
        dns_records,
        req.notes
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Log the update
    log_audit_action(&state.db_pool, auth.0.id, "UPDATE", "domain", id).await;

    Ok(Json(serde_json::json!({ "message": "Domain updated successfully" })))
}

async fn delete_domain(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!("DELETE FROM domains WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Log the deletion
    log_audit_action(&state.db_pool, auth.0.id, "DELETE", "domain", id).await;

    Ok(Json(serde_json::json!({ "message": "Domain deleted successfully" })))
}

async fn get_expiring_domains(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListDomainsQuery>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let days = query.expiring_days.unwrap_or(30);
    
    let domains = sqlx::query_as!(
        Domain,
        r#"
        SELECT id, client_id, name, registrar, nameservers, registration_date,
               expiry_date, auto_renew, dns_records, notes, created_at, updated_at
        FROM domains
        WHERE expiry_date <= CURRENT_DATE + ($1 || ' days')::INTERVAL
        ORDER BY expiry_date ASC
        "#,
        days
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Add expiry information
    let domains_with_expiry: Vec<DomainWithExpiry> = domains.into_iter().map(|domain| {
        let (days_until_expiry, is_expired) = if let Some(expiry_date) = domain.expiry_date {
            let today = chrono::Utc::now().date_naive();
            let days = (expiry_date - today).num_days() as i32;
            (Some(days), days < 0)
        } else {
            (None, false)
        };

        DomainWithExpiry {
            domain,
            days_until_expiry,
            is_expired,
        }
    }).collect();

    Ok(Json(domains_with_expiry))
}

async fn get_dns_records(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let domain = sqlx::query!(
        "SELECT dns_records FROM domains WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(domain.dns_records))
}

async fn update_dns_records(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(dns_records): Json<serde_json::Value>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!(
        "UPDATE domains SET dns_records = $2, updated_at = NOW() WHERE id = $1",
        id,
        dns_records
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Log the DNS update
    log_audit_action(&state.db_pool, auth.0.id, "UPDATE_DNS", "domain", id).await;

    Ok(Json(serde_json::json!({ "message": "DNS records updated successfully" })))
}

async fn log_audit_action(
    db_pool: &sqlx::PgPool,
    user_id: Uuid,
    action: &str,
    entity_type: &str,
    entity_id: Uuid,
) {
    let _ = sqlx::query!(
        r#"
        INSERT INTO audit_logs (user_id, action, entity_type, entity_id, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        user_id,
        action,
        entity_type,
        entity_id
    )
    .execute(db_pool)
    .await;
}