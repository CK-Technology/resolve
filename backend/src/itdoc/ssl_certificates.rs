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

use crate::auth::middleware::AuthUser;
use crate::AppState;
use resolve_shared::SslCertificate;

pub fn ssl_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_ssl_certificates).post(create_ssl_certificate))
        .route("/:id", get(get_ssl_certificate).put(update_ssl_certificate).delete(delete_ssl_certificate))
        .route("/expiring", get(get_expiring_ssl_certificates))
}

#[derive(Debug, Deserialize)]
pub struct ListSslQuery {
    pub client_id: Option<Uuid>,
    pub domain_id: Option<Uuid>,
    pub search: Option<String>,
    pub expiring_days: Option<i32>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSslCertificateRequest {
    pub domain_id: Option<Uuid>,
    pub client_id: Uuid,
    pub name: String,
    pub common_name: String,
    pub subject_alt_names: Option<Vec<String>>,
    pub issuer: String,
    pub issued_date: NaiveDate,
    pub expiry_date: NaiveDate,
    pub certificate_chain: Option<String>,
    pub private_key: Option<String>,
    pub auto_renew: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SslCertificateWithExpiry {
    #[serde(flatten)]
    pub certificate: SslCertificate,
    pub days_until_expiry: i32,
    pub is_expired: bool,
    pub is_expiring_soon: bool,
}

async fn list_ssl_certificates(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListSslQuery>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    let certificates = if let Some(client_id) = query.client_id {
        sqlx::query_as!(
            SslCertificate,
            r#"
            SELECT id, domain_id, client_id, name, common_name, subject_alt_names,
                   issuer, issued_date, expiry_date, certificate_chain, private_key,
                   auto_renew, status, created_at, updated_at
            FROM ssl_certificates
            WHERE client_id = $1
            ORDER BY expiry_date ASC
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
            SslCertificate,
            r#"
            SELECT id, domain_id, client_id, name, common_name, subject_alt_names,
                   issuer, issued_date, expiry_date, certificate_chain, private_key,
                   auto_renew, status, created_at, updated_at
            FROM ssl_certificates
            ORDER BY expiry_date ASC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    // Add expiry information and hide private keys
    let certificates_with_expiry: Vec<SslCertificateWithExpiry> = certificates.into_iter().map(|mut cert| {
        // Hide private key for security
        cert.private_key = cert.private_key.map(|_| "***ENCRYPTED***".to_string());

        let today = chrono::Utc::now().date_naive();
        let days_until_expiry = (cert.expiry_date - today).num_days() as i32;
        let is_expired = days_until_expiry < 0;
        let is_expiring_soon = days_until_expiry <= 30 && days_until_expiry >= 0;

        SslCertificateWithExpiry {
            certificate: cert,
            days_until_expiry,
            is_expired,
            is_expiring_soon,
        }
    }).collect();

    Ok(Json(certificates_with_expiry))
}

async fn get_ssl_certificate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let mut certificate = sqlx::query_as!(
        SslCertificate,
        r#"
        SELECT id, domain_id, client_id, name, common_name, subject_alt_names,
               issuer, issued_date, expiry_date, certificate_chain, private_key,
               auto_renew, status, created_at, updated_at
        FROM ssl_certificates
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Hide private key for security
    certificate.private_key = certificate.private_key.map(|_| "***ENCRYPTED***".to_string());

    Ok(Json(certificate))
}

async fn create_ssl_certificate(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateSslCertificateRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = Uuid::new_v4();
    let subject_alt_names = req.subject_alt_names.unwrap_or_default();

    // Encrypt private key if provided
    let encrypted_private_key = if let Some(private_key) = &req.private_key {
        Some(encrypt_private_key(private_key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
    } else {
        None
    };

    sqlx::query!(
        r#"
        INSERT INTO ssl_certificates (
            id, domain_id, client_id, name, common_name, subject_alt_names,
            issuer, issued_date, expiry_date, certificate_chain, private_key,
            auto_renew, status, created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NULL
        )
        "#,
        id,
        req.domain_id,
        req.client_id,
        req.name,
        req.common_name,
        &subject_alt_names,
        req.issuer,
        req.issued_date,
        req.expiry_date,
        req.certificate_chain,
        encrypted_private_key,
        req.auto_renew.unwrap_or(false),
        req.status.unwrap_or_else(|| "active".to_string())
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log the creation
    log_audit_action(&state.db_pool, auth.0.id, "CREATE", "ssl_certificate", id).await;

    Ok(Json(serde_json::json!({ "id": id, "message": "SSL certificate created successfully" })))
}

async fn update_ssl_certificate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(req): Json<CreateSslCertificateRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let subject_alt_names = req.subject_alt_names.unwrap_or_default();

    // Get current certificate for private key handling
    let current = sqlx::query!(
        "SELECT private_key FROM ssl_certificates WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Handle private key encryption
    let encrypted_private_key = if let Some(private_key) = &req.private_key {
        if private_key != "***ENCRYPTED***" {
            Some(encrypt_private_key(private_key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        } else {
            current.private_key // Keep existing
        }
    } else {
        None
    };

    let result = sqlx::query!(
        r#"
        UPDATE ssl_certificates SET
            domain_id = $2, client_id = $3, name = $4, common_name = $5,
            subject_alt_names = $6, issuer = $7, issued_date = $8,
            expiry_date = $9, certificate_chain = $10, private_key = $11,
            auto_renew = $12, status = $13, updated_at = NOW()
        WHERE id = $1
        "#,
        id,
        req.domain_id,
        req.client_id,
        req.name,
        req.common_name,
        &subject_alt_names,
        req.issuer,
        req.issued_date,
        req.expiry_date,
        req.certificate_chain,
        encrypted_private_key,
        req.auto_renew.unwrap_or(false),
        req.status.unwrap_or_else(|| "active".to_string())
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Log the update
    log_audit_action(&state.db_pool, auth.0.id, "UPDATE", "ssl_certificate", id).await;

    Ok(Json(serde_json::json!({ "message": "SSL certificate updated successfully" })))
}

async fn delete_ssl_certificate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!("DELETE FROM ssl_certificates WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Log the deletion
    log_audit_action(&state.db_pool, auth.0.id, "DELETE", "ssl_certificate", id).await;

    Ok(Json(serde_json::json!({ "message": "SSL certificate deleted successfully" })))
}

async fn get_expiring_ssl_certificates(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListSslQuery>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let days = query.expiring_days.unwrap_or(30);
    
    let certificates = sqlx::query_as!(
        SslCertificate,
        r#"
        SELECT id, domain_id, client_id, name, common_name, subject_alt_names,
               issuer, issued_date, expiry_date, certificate_chain, private_key,
               auto_renew, status, created_at, updated_at
        FROM ssl_certificates
        WHERE expiry_date <= CURRENT_DATE + $1::interval
        ORDER BY expiry_date ASC
        "#,
        format!("{} days", days)
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Add expiry information and hide private keys
    let certificates_with_expiry: Vec<SslCertificateWithExpiry> = certificates.into_iter().map(|mut cert| {
        // Hide private key for security
        cert.private_key = cert.private_key.map(|_| "***ENCRYPTED***".to_string());

        let today = chrono::Utc::now().date_naive();
        let days_until_expiry = (cert.expiry_date - today).num_days() as i32;
        let is_expired = days_until_expiry < 0;
        let is_expiring_soon = days_until_expiry <= 30 && days_until_expiry >= 0;

        SslCertificateWithExpiry {
            certificate: cert,
            days_until_expiry,
            is_expired,
            is_expiring_soon,
        }
    }).collect();

    Ok(Json(certificates_with_expiry))
}

fn encrypt_private_key(private_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
    use rand::RngCore;

    let key = get_ssl_encryption_key();
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, private_key.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Combine nonce and ciphertext
    let mut encrypted = Vec::with_capacity(12 + ciphertext.len());
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);
    
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted))
}

fn get_ssl_encryption_key() -> Key<Aes256Gcm> {
    use aes_gcm::Key;
    
    let key_env = std::env::var("SSL_ENCRYPTION_KEY").unwrap_or_else(|_| {
        tracing::warn!("SSL_ENCRYPTION_KEY not set, using default (insecure for production)");
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()
    });
    
    let key_bytes = hex::decode(&key_env).unwrap_or_else(|_| {
        tracing::error!("Invalid SSL_ENCRYPTION_KEY format, using default");
        vec![0u8; 32]
    });
    
    if key_bytes.len() != 32 {
        tracing::error!("SSL_ENCRYPTION_KEY must be 32 bytes (64 hex chars), using default");
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
        user_id,
        action,
        entity_type,
        entity_id
    )
    .execute(db_pool)
    .await;
}