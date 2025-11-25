pub mod azure;
pub mod cloudflare;
pub mod github;
pub mod google;
pub mod stripe;

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
use resolve_shared::Integration;

pub fn integration_routes() -> Router<Arc<AppState>> {
    Router::new()
        // General integration management
        .route("/", get(list_integrations).post(create_integration))
        .route("/:id", get(get_integration).put(update_integration).delete(delete_integration))
        .route("/:id/sync", post(sync_integration))
        .route("/:id/test", post(test_integration))
        
        // Specific integration routes
        .nest("/azure", azure::azure_routes())
        .nest("/cloudflare", cloudflare::cloudflare_routes())
        .nest("/github", github::github_routes())
        .nest("/google", google::google_routes())
        .nest("/stripe", stripe::stripe_routes())
}

#[derive(Debug, Deserialize)]
pub struct CreateIntegrationRequest {
    pub name: String,
    pub integration_type: String,
    pub config: serde_json::Value,
    pub credentials: serde_json::Value,
    pub enabled: bool,
}

async fn list_integrations(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integrations = sqlx::query_as!(
        Integration,
        r#"
        SELECT id, name, integration_type, config, credentials, enabled,
               last_sync, created_at, updated_at
        FROM integrations
        ORDER BY name ASC
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Remove sensitive credential data before returning
    let safe_integrations: Vec<_> = integrations.into_iter().map(|mut integration| {
        integration.credentials = serde_json::json!({ "configured": !integration.credentials.is_null() });
        integration
    }).collect();

    Ok(Json(safe_integrations))
}

async fn get_integration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let mut integration = sqlx::query_as!(
        Integration,
        r#"
        SELECT id, name, integration_type, config, credentials, enabled,
               last_sync, created_at, updated_at
        FROM integrations
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Remove sensitive credential data
    integration.credentials = serde_json::json!({ "configured": !integration.credentials.is_null() });

    Ok(Json(integration))
}

async fn create_integration(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateIntegrationRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = Uuid::new_v4();

    // Encrypt credentials before storing
    let encrypted_credentials = encrypt_json(&req.credentials)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query!(
        r#"
        INSERT INTO integrations (
            id, name, integration_type, config, credentials, enabled,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NULL)
        "#,
        id,
        req.name,
        req.integration_type,
        req.config,
        encrypted_credentials,
        req.enabled
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log the creation
    log_audit_action(&state.db_pool, auth.0.id, "CREATE", "integration", id).await;

    Ok(Json(serde_json::json!({ 
        "id": id, 
        "message": "Integration created successfully" 
    })))
}

async fn update_integration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(req): Json<CreateIntegrationRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get current integration for credential handling
    let current = sqlx::query!(
        "SELECT credentials FROM integrations WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Handle credential encryption
    let encrypted_credentials = if req.credentials.get("configured").is_some() {
        current.credentials // Keep existing if placeholder
    } else {
        encrypt_json(&req.credentials).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let result = sqlx::query!(
        r#"
        UPDATE integrations SET
            name = $2, integration_type = $3, config = $4, credentials = $5,
            enabled = $6, updated_at = NOW()
        WHERE id = $1
        "#,
        id,
        req.name,
        req.integration_type,
        req.config,
        encrypted_credentials,
        req.enabled
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    log_audit_action(&state.db_pool, auth.0.id, "UPDATE", "integration", id).await;

    Ok(Json(serde_json::json!({ "message": "Integration updated successfully" })))
}

async fn delete_integration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!("DELETE FROM integrations WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    log_audit_action(&state.db_pool, auth.0.id, "DELETE", "integration", id).await;

    Ok(Json(serde_json::json!({ "message": "Integration deleted successfully" })))
}

async fn sync_integration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration = sqlx::query_as!(
        Integration,
        r#"
        SELECT id, name, integration_type, config, credentials, enabled,
               last_sync, created_at, updated_at
        FROM integrations
        WHERE id = $1 AND enabled = true
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let sync_result = match integration.integration_type.as_str() {
        "azure" => azure::sync_azure_integration(&state.db_pool, &integration).await,
        "cloudflare" => cloudflare::sync_cloudflare_integration(&state.db_pool, &integration).await,
        "github" => github::sync_github_integration(&state.db_pool, &integration).await,
        "google" => google::sync_google_integration(&state.db_pool, &integration).await,
        _ => Err("Unsupported integration type".into()),
    };

    match sync_result {
        Ok(sync_info) => {
            // Update last sync time
            sqlx::query!(
                "UPDATE integrations SET last_sync = NOW() WHERE id = $1",
                id
            )
            .execute(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            log_audit_action(&state.db_pool, auth.0.id, "SYNC", "integration", id).await;

            Ok(Json(serde_json::json!({
                "message": "Integration synchronized successfully",
                "sync_info": sync_info
            })))
        }
        Err(error) => {
            tracing::error!("Integration sync failed: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn test_integration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration = sqlx::query_as!(
        Integration,
        r#"
        SELECT id, name, integration_type, config, credentials, enabled,
               last_sync, created_at, updated_at
        FROM integrations
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let test_result = match integration.integration_type.as_str() {
        "azure" => azure::test_azure_connection(&integration).await,
        "cloudflare" => cloudflare::test_cloudflare_connection(&integration).await,
        "github" => github::test_github_connection(&integration).await,
        "google" => google::test_google_connection(&integration).await,
        _ => Err("Unsupported integration type".into()),
    };

    match test_result {
        Ok(test_info) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": "Integration test successful",
            "test_info": test_info
        }))),
        Err(error) => Ok(Json(serde_json::json!({
            "status": "error",
            "message": "Integration test failed",
            "error": error.to_string()
        })))
    }
}

fn encrypt_json(data: &serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
    use rand::RngCore;

    let json_str = serde_json::to_string(data)?;
    
    let key = get_integration_encryption_key();
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, json_str.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Combine nonce and ciphertext
    let mut encrypted = Vec::with_capacity(12 + ciphertext.len());
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);
    
    let encrypted_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted);
    
    Ok(serde_json::json!({ "encrypted": encrypted_b64 }))
}

pub fn decrypt_json(encrypted_data: &serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};

    let encrypted_str = encrypted_data.get("encrypted")
        .and_then(|v| v.as_str())
        .ok_or("Invalid encrypted data format")?;
    
    let encrypted = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encrypted_str)?;
    
    if encrypted.len() < 12 {
        return Err("Invalid encrypted data".into());
    }
    
    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let key = get_integration_encryption_key();
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    let json_str = String::from_utf8(plaintext)?;
    
    Ok(serde_json::from_str(&json_str)?)
}

fn get_integration_encryption_key() -> Key<Aes256Gcm> {
    use aes_gcm::Key;
    
    let key_env = std::env::var("INTEGRATION_ENCRYPTION_KEY").unwrap_or_else(|_| {
        tracing::warn!("INTEGRATION_ENCRYPTION_KEY not set, using default (insecure for production)");
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()
    });
    
    let key_bytes = hex::decode(&key_env).unwrap_or_else(|_| {
        tracing::error!("Invalid INTEGRATION_ENCRYPTION_KEY format, using default");
        vec![0u8; 32]
    });
    
    if key_bytes.len() != 32 {
        tracing::error!("INTEGRATION_ENCRYPTION_KEY must be 32 bytes (64 hex chars), using default");
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