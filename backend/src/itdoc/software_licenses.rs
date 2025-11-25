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
use aes_gcm::{Aes256Gcm, Key};
use rust_decimal::Decimal;

use crate::auth::middleware::AuthUser;
use crate::AppState;
use resolve_shared::SoftwareLicense;

pub fn license_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_software_licenses).post(create_software_license))
        .route("/:id", get(get_software_license).put(update_software_license).delete(delete_software_license))
        .route("/expiring", get(get_expiring_licenses))
        .route("/usage", get(get_license_usage_summary))
}

#[derive(Debug, Deserialize)]
pub struct CreateSoftwareLicenseRequest {
    pub client_id: Uuid,
    pub name: String,
    pub vendor: String,
    pub version: Option<String>,
    pub license_key: Option<String>,
    pub license_type: String,
    pub seats: Option<i32>,
    pub used_seats: Option<i32>,
    pub purchase_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub renewal_date: Option<NaiveDate>,
    pub cost: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SoftwareLicenseWithUsage {
    #[serde(flatten)]
    pub license: SoftwareLicense,
    pub days_until_expiry: Option<i32>,
    pub is_expired: bool,
    pub is_expiring_soon: bool,
    pub usage_percentage: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct LicenseUsageSummary {
    pub total_licenses: i64,
    pub total_seats: i32,
    pub used_seats: i32,
    pub available_seats: i32,
    pub expiring_soon: i64,
    pub expired: i64,
    pub total_cost: Decimal,
}

async fn list_software_licenses(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let client_id = query.get("client_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok());

    let licenses = if let Some(client_id) = client_id {
        sqlx::query_as!(
            SoftwareLicense,
            r#"
            SELECT id, client_id, name, vendor, version, license_key, license_type,
                   seats, used_seats, purchase_date, expiry_date, renewal_date,
                   cost, notes, created_at, updated_at
            FROM software_licenses WHERE client_id = $1 ORDER BY name ASC
            "#,
            client_id
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as!(
            SoftwareLicense,
            r#"
            SELECT id, client_id, name, vendor, version, license_key, license_type,
                   seats, used_seats, purchase_date, expiry_date, renewal_date,
                   cost, notes, created_at, updated_at
            FROM software_licenses ORDER BY name ASC
            "#
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    // Add usage and expiry information, hide license keys
    let licenses_with_usage: Vec<SoftwareLicenseWithUsage> = licenses.into_iter().map(|mut license| {
        // Hide license key for security
        license.license_key = license.license_key.map(|_| "***ENCRYPTED***".to_string());

        let (days_until_expiry, is_expired, is_expiring_soon) = if let Some(expiry_date) = license.expiry_date {
            let today = chrono::Utc::now().date_naive();
            let days = (expiry_date - today).num_days() as i32;
            (Some(days), days < 0, days <= 30 && days >= 0)
        } else {
            (None, false, false)
        };

        let usage_percentage = if let (Some(seats), Some(used_seats)) = (license.seats, license.used_seats) {
            if seats > 0 {
                Some((used_seats as f64 / seats as f64) * 100.0)
            } else {
                None
            }
        } else {
            None
        };

        SoftwareLicenseWithUsage {
            license,
            days_until_expiry,
            is_expired,
            is_expiring_soon,
            usage_percentage,
        }
    }).collect();

    Ok(Json(licenses_with_usage))
}

async fn get_software_license(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let mut license = sqlx::query_as!(
        SoftwareLicense,
        r#"
        SELECT id, client_id, name, vendor, version, license_key, license_type,
               seats, used_seats, purchase_date, expiry_date, renewal_date,
               cost, notes, created_at, updated_at
        FROM software_licenses WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Hide license key for security
    license.license_key = license.license_key.map(|_| "***ENCRYPTED***".to_string());

    Ok(Json(license))
}

async fn create_software_license(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateSoftwareLicenseRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = Uuid::new_v4();

    // Encrypt license key if provided
    let encrypted_license_key = if let Some(license_key) = &req.license_key {
        Some(encrypt_license_key(license_key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
    } else {
        None
    };

    sqlx::query!(
        r#"
        INSERT INTO software_licenses (
            id, client_id, name, vendor, version, license_key, license_type,
            seats, used_seats, purchase_date, expiry_date, renewal_date,
            cost, notes, created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW(), NULL
        )
        "#,
        id, req.client_id, req.name, req.vendor, req.version,
        encrypted_license_key, req.license_type, req.seats,
        req.used_seats.unwrap_or(0), req.purchase_date, req.expiry_date,
        req.renewal_date, req.cost, req.notes
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    log_audit_action(&state.db_pool, auth.0.id, "CREATE", "software_license", id).await;

    Ok(Json(serde_json::json!({ "id": id, "message": "Software license created successfully" })))
}

async fn update_software_license(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(req): Json<CreateSoftwareLicenseRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get current license for key handling
    let current = sqlx::query!(
        "SELECT license_key FROM software_licenses WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Handle license key encryption
    let encrypted_license_key = if let Some(license_key) = &req.license_key {
        if license_key != "***ENCRYPTED***" {
            Some(encrypt_license_key(license_key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        } else {
            current.license_key // Keep existing
        }
    } else {
        None
    };

    let result = sqlx::query!(
        r#"
        UPDATE software_licenses SET
            client_id = $2, name = $3, vendor = $4, version = $5,
            license_key = $6, license_type = $7, seats = $8, used_seats = $9,
            purchase_date = $10, expiry_date = $11, renewal_date = $12,
            cost = $13, notes = $14, updated_at = NOW()
        WHERE id = $1
        "#,
        id, req.client_id, req.name, req.vendor, req.version,
        encrypted_license_key, req.license_type, req.seats,
        req.used_seats.unwrap_or(0), req.purchase_date, req.expiry_date,
        req.renewal_date, req.cost, req.notes
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    log_audit_action(&state.db_pool, auth.0.id, "UPDATE", "software_license", id).await;

    Ok(Json(serde_json::json!({ "message": "Software license updated successfully" })))
}

async fn delete_software_license(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!("DELETE FROM software_licenses WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    log_audit_action(&state.db_pool, auth.0.id, "DELETE", "software_license", id).await;

    Ok(Json(serde_json::json!({ "message": "Software license deleted successfully" })))
}

async fn get_expiring_licenses(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let days = query.get("days").and_then(|v| v.as_i64()).unwrap_or(30);
    
    let licenses = sqlx::query_as!(
        SoftwareLicense,
        r#"
        SELECT id, client_id, name, vendor, version, license_key, license_type,
               seats, used_seats, purchase_date, expiry_date, renewal_date,
               cost, notes, created_at, updated_at
        FROM software_licenses
        WHERE expiry_date <= CURRENT_DATE + $1::interval
        ORDER BY expiry_date ASC
        "#,
        format!("{} days", days)
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Hide license keys and add expiry info
    let safe_licenses: Vec<_> = licenses.into_iter().map(|mut license| {
        license.license_key = license.license_key.map(|_| "***ENCRYPTED***".to_string());
        license
    }).collect();

    Ok(Json(safe_licenses))
}

async fn get_license_usage_summary(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let client_id = query.get("client_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok());

    let where_clause = if client_id.is_some() { "WHERE client_id = $1" } else { "" };
    
    // Get summary statistics
    let summary = if let Some(client_id) = client_id {
        sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_licenses,
                COALESCE(SUM(seats), 0) as total_seats,
                COALESCE(SUM(used_seats), 0) as used_seats,
                COALESCE(SUM(seats - COALESCE(used_seats, 0)), 0) as available_seats,
                COUNT(*) FILTER (WHERE expiry_date <= CURRENT_DATE + INTERVAL '30 days') as expiring_soon,
                COUNT(*) FILTER (WHERE expiry_date < CURRENT_DATE) as expired,
                COALESCE(SUM(cost), 0) as total_cost
            FROM software_licenses 
            WHERE client_id = $1
            "#,
            client_id
        )
        .fetch_one(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_licenses,
                COALESCE(SUM(seats), 0) as total_seats,
                COALESCE(SUM(used_seats), 0) as used_seats,
                COALESCE(SUM(seats - COALESCE(used_seats, 0)), 0) as available_seats,
                COUNT(*) FILTER (WHERE expiry_date <= CURRENT_DATE + INTERVAL '30 days') as expiring_soon,
                COUNT(*) FILTER (WHERE expiry_date < CURRENT_DATE) as expired,
                COALESCE(SUM(cost), 0) as total_cost
            FROM software_licenses
            "#
        )
        .fetch_one(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let usage_summary = LicenseUsageSummary {
        total_licenses: summary.total_licenses.unwrap_or(0),
        total_seats: summary.total_seats.unwrap_or(0) as i32,
        used_seats: summary.used_seats.unwrap_or(0) as i32,
        available_seats: summary.available_seats.unwrap_or(0) as i32,
        expiring_soon: summary.expiring_soon.unwrap_or(0),
        expired: summary.expired.unwrap_or(0),
        total_cost: summary.total_cost.unwrap_or_else(|| Decimal::new(0, 0)),
    };

    Ok(Json(usage_summary))
}

fn encrypt_license_key(license_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
    use rand::RngCore;

    let key = get_license_encryption_key();
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, license_key.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Combine nonce and ciphertext
    let mut encrypted = Vec::with_capacity(12 + ciphertext.len());
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);
    
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted))
}

fn get_license_encryption_key() -> Key<Aes256Gcm> {
    use aes_gcm::Key;
    
    let key_env = std::env::var("LICENSE_ENCRYPTION_KEY").unwrap_or_else(|_| {
        tracing::warn!("LICENSE_ENCRYPTION_KEY not set, using default (insecure for production)");
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()
    });
    
    let key_bytes = hex::decode(&key_env).unwrap_or_else(|_| {
        tracing::error!("Invalid LICENSE_ENCRYPTION_KEY format, using default");
        vec![0u8; 32]
    });
    
    if key_bytes.len() != 32 {
        tracing::error!("LICENSE_ENCRYPTION_KEY must be 32 bytes (64 hex chars), using default");
        return *Key::<Aes256Gcm>::from_slice(&[0u8; 32]);
    }
    
    *Key::<Aes256Gcm>::from_slice(&key_bytes)
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
        user_id, action, entity_type, entity_id
    )
    .execute(db_pool)
    .await;
}