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
use aes_gcm::{Aes256Gcm, Key};

use crate::auth::middleware::AuthUser;
use crate::AppState;
use resolve_shared::Credential;

pub fn credential_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_credentials).post(create_credential))
        .route("/:id", get(get_credential).put(update_credential).delete(delete_credential))
        .route("/:id/access", post(record_credential_access))
}

#[derive(Debug, Deserialize)]
pub struct ListCredentialsQuery {
    pub client_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub search: Option<String>,
    pub tags: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCredentialRequest {
    pub client_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub certificate: Option<String>,
    pub uri: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn list_credentials(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListCredentialsQuery>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 0;

    if let Some(client_id) = query.client_id {
        param_count += 1;
        conditions.push(format!("client_id = ${}", param_count));
        params.push(Box::new(client_id));
    }

    if let Some(asset_id) = query.asset_id {
        param_count += 1;
        conditions.push(format!("asset_id = ${}", param_count));
        params.push(Box::new(asset_id));
    }

    if let Some(search) = &query.search {
        param_count += 1;
        conditions.push(format!("(name ILIKE ${} OR username ILIKE ${} OR uri ILIKE ${})", param_count, param_count, param_count));
        params.push(Box::new(format!("%{}%", search)));
    }

    if let Some(tags_str) = &query.tags {
        let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
        if !tags.is_empty() {
            param_count += 1;
            conditions.push(format!("tags && ${}", param_count));
            params.push(Box::new(tags));
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // For simplicity, using a basic query without dynamic parameters
    // In a production app, you'd use a query builder or similar approach
    let credentials = if query.client_id.is_some() {
        sqlx::query_as!(
            Credential,
            r#"
            SELECT id, client_id, asset_id, name, username, password, private_key,
                   public_key, certificate, uri, notes, tags, last_accessed,
                   expires_at, created_at, updated_at
            FROM credentials
            WHERE client_id = $1
            ORDER BY name ASC
            LIMIT $2 OFFSET $3
            "#,
            query.client_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as!(
            Credential,
            r#"
            SELECT id, client_id, asset_id, name, username, password, private_key,
                   public_key, certificate, uri, notes, tags, last_accessed,
                   expires_at, created_at, updated_at
            FROM credentials
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

    // Remove sensitive data before returning
    let safe_credentials: Vec<_> = credentials.into_iter().map(|mut cred| {
        cred.password = cred.password.map(|_| "***ENCRYPTED***".to_string());
        cred.private_key = cred.private_key.map(|_| "***ENCRYPTED***".to_string());
        cred
    }).collect();

    Ok(Json(safe_credentials))
}

async fn get_credential(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let credential = sqlx::query_as!(
        Credential,
        r#"
        SELECT id, client_id, asset_id, name, username, password, private_key,
               public_key, certificate, uri, notes, tags, last_accessed,
               expires_at, created_at, updated_at
        FROM credentials
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // For security, don't return decrypted passwords/keys by default
    // This would need a separate "reveal" endpoint with additional authentication
    let mut safe_credential = credential;
    safe_credential.password = safe_credential.password.map(|_| "***ENCRYPTED***".to_string());
    safe_credential.private_key = safe_credential.private_key.map(|_| "***ENCRYPTED***".to_string());

    Ok(Json(safe_credential))
}

async fn create_credential(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateCredentialRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = Uuid::new_v4();

    // Encrypt sensitive data before storing
    let encrypted_password = if let Some(password) = &req.password {
        Some(encrypt_data(password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
    } else {
        None
    };

    let encrypted_private_key = if let Some(private_key) = &req.private_key {
        Some(encrypt_data(private_key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
    } else {
        None
    };

    let tags = req.tags.unwrap_or_default();

    sqlx::query!(
        r#"
        INSERT INTO credentials (
            id, client_id, asset_id, name, username, password, private_key,
            public_key, certificate, uri, notes, tags, expires_at,
            created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NULL
        )
        "#,
        id,
        req.client_id,
        req.asset_id,
        req.name,
        req.username,
        encrypted_password,
        encrypted_private_key,
        req.public_key,
        req.certificate,
        req.uri,
        req.notes,
        &tags,
        req.expires_at
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log the creation in audit log
    log_audit_action(&state.db_pool, auth.0.id, "CREATE", "credential", id, None, None).await;

    Ok(Json(serde_json::json!({ "id": id, "message": "Credential created successfully" })))
}

async fn update_credential(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(req): Json<CreateCredentialRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get the current credential for audit logging
    let current = sqlx::query_as!(
        Credential,
        "SELECT * FROM credentials WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Encrypt sensitive data if provided
    let encrypted_password = if let Some(password) = &req.password {
        if password != "***ENCRYPTED***" {
            Some(encrypt_data(password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        } else {
            current.password // Keep existing
        }
    } else {
        None
    };

    let encrypted_private_key = if let Some(private_key) = &req.private_key {
        if private_key != "***ENCRYPTED***" {
            Some(encrypt_data(private_key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        } else {
            current.private_key // Keep existing
        }
    } else {
        None
    };

    let tags = req.tags.unwrap_or_default();

    sqlx::query!(
        r#"
        UPDATE credentials SET
            client_id = $2, asset_id = $3, name = $4, username = $5,
            password = $6, private_key = $7, public_key = $8, certificate = $9,
            uri = $10, notes = $11, tags = $12, expires_at = $13, updated_at = NOW()
        WHERE id = $1
        "#,
        id,
        req.client_id,
        req.asset_id,
        req.name,
        req.username,
        encrypted_password,
        encrypted_private_key,
        req.public_key,
        req.certificate,
        req.uri,
        req.notes,
        &tags,
        req.expires_at
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log the update in audit log
    log_audit_action(&state.db_pool, auth.0.id, "UPDATE", "credential", id, None, None).await;

    Ok(Json(serde_json::json!({ "message": "Credential updated successfully" })))
}

async fn delete_credential(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!("DELETE FROM credentials WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Log the deletion in audit log
    log_audit_action(&state.db_pool, auth.0.id, "DELETE", "credential", id, None, None).await;

    Ok(Json(serde_json::json!({ "message": "Credential deleted successfully" })))
}

async fn record_credential_access(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    sqlx::query!(
        "UPDATE credentials SET last_accessed = NOW() WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log the access in audit log
    log_audit_action(&state.db_pool, auth.0.id, "ACCESS", "credential", id, None, None).await;

    Ok(StatusCode::OK)
}

// Encryption helper functions
fn encrypt_data(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    // This is a simplified encryption implementation
    // In production, use proper encryption with AES-GCM or similar
    use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
    use rand::RngCore;

    let key = get_encryption_key();
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, data.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Combine nonce and ciphertext
    let mut encrypted = Vec::with_capacity(12 + ciphertext.len());
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);
    
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted))
}

fn get_encryption_key() -> Key<Aes256Gcm> {
    
    let key_env = std::env::var("CREDENTIAL_ENCRYPTION_KEY").unwrap_or_else(|_| {
        tracing::warn!("CREDENTIAL_ENCRYPTION_KEY not set, using default (insecure for production)");
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()
    });
    
    let key_bytes = hex::decode(&key_env).unwrap_or_else(|_| {
        tracing::error!("Invalid CREDENTIAL_ENCRYPTION_KEY format, using default");
        vec![0u8; 32]
    });
    
    if key_bytes.len() != 32 {
        tracing::error!("CREDENTIAL_ENCRYPTION_KEY must be 32 bytes (64 hex chars), using default");
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
    old_values: Option<serde_json::Value>,
    new_values: Option<serde_json::Value>,
) {
    let _ = sqlx::query!(
        r#"
        INSERT INTO audit_logs (user_id, action, entity_type, entity_id, old_values, new_values, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        "#,
        user_id,
        action,
        entity_type,
        entity_id,
        old_values,
        new_values
    )
    .execute(db_pool)
    .await;
}