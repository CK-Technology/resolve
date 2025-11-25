//! API Key HTTP handlers
//!
//! Provides REST endpoints for API key management.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::api_keys::{
    generate_api_key, has_scope, validate_create_request, ApiKey, ApiKeyScope,
    CreateApiKeyRequest, CreateApiKeyResponse,
};
use super::middleware::AuthUser;
use crate::error::{AppError, ApiResult};
use crate::AppState;

/// Response for listing API keys (doesn't include the actual key)
#[derive(Debug, Serialize)]
pub struct ApiKeyListItem {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub key_prefix: String,
    pub scopes: Vec<ApiKeyScope>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub last_used_at: Option<chrono::DateTime<Utc>>,
    pub usage_count: i64,
}

/// Request to update an API key
#[derive(Debug, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scopes: Option<Vec<ApiKeyScope>>,
    pub allowed_ips: Option<Vec<String>>,
    pub rate_limit: Option<u32>,
    pub is_active: Option<bool>,
}

pub fn api_key_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_api_keys).post(create_api_key))
        .route("/:id", get(get_api_key).delete(revoke_api_key))
        .route("/:id/regenerate", post(regenerate_api_key))
}

/// List all API keys for the current user
async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
) -> ApiResult<impl IntoResponse> {
    let keys = sqlx::query_as!(
        ApiKeyListItem,
        r#"
        SELECT
            id, name, description, key_prefix,
            scopes as "scopes: sqlx::types::Json<Vec<ApiKeyScope>>",
            expires_at, is_active, created_at, last_used_at, usage_count
        FROM api_keys
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user.id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Map to response format
    let keys: Vec<ApiKeyListItem> = keys
        .into_iter()
        .map(|k| ApiKeyListItem {
            id: k.id,
            name: k.name,
            description: k.description,
            key_prefix: k.key_prefix,
            scopes: k.scopes,
            expires_at: k.expires_at,
            is_active: k.is_active,
            created_at: k.created_at,
            last_used_at: k.last_used_at,
            usage_count: k.usage_count,
        })
        .collect();

    Ok(Json(keys))
}

/// Create a new API key
async fn create_api_key(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateApiKeyRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate request
    validate_create_request(&req)?;

    // Generate the key
    let (key, prefix, hash) = generate_api_key();

    // Calculate expiration
    let expires_at = req
        .expires_in_days
        .map(|days| Utc::now() + Duration::days(days as i64));

    let key_id = Uuid::new_v4();
    let now = Utc::now();

    // Store in database
    sqlx::query!(
        r#"
        INSERT INTO api_keys (
            id, user_id, name, description, key_hash, key_prefix,
            scopes, expires_at, allowed_ips, rate_limit, is_active,
            created_at, usage_count
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, $11, 0)
        "#,
        key_id,
        user.id,
        req.name,
        req.description,
        hash,
        prefix,
        serde_json::to_value(&req.scopes).unwrap(),
        expires_at,
        &req.allowed_ips,
        req.rate_limit.unwrap_or(0) as i32,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let response = CreateApiKeyResponse {
        id: key_id,
        name: req.name,
        key, // Only time the actual key is returned!
        key_prefix: prefix,
        scopes: req.scopes,
        expires_at,
        created_at: now,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get details of a specific API key
async fn get_api_key(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(key_id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    let key = sqlx::query_as!(
        ApiKeyListItem,
        r#"
        SELECT
            id, name, description, key_prefix,
            scopes as "scopes: sqlx::types::Json<Vec<ApiKeyScope>>",
            expires_at, is_active, created_at, last_used_at, usage_count
        FROM api_keys
        WHERE id = $1 AND user_id = $2
        "#,
        key_id,
        user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("API key".to_string()))?;

    Ok(Json(key))
}

/// Revoke (delete) an API key
async fn revoke_api_key(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(key_id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    let result = sqlx::query!(
        "DELETE FROM api_keys WHERE id = $1 AND user_id = $2",
        key_id,
        user.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("API key".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Regenerate an API key (creates new key, invalidates old one)
async fn regenerate_api_key(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(key_id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    // First, get the existing key details
    let existing = sqlx::query!(
        r#"
        SELECT name, description, scopes, expires_at, allowed_ips, rate_limit
        FROM api_keys
        WHERE id = $1 AND user_id = $2
        "#,
        key_id,
        user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("API key".to_string()))?;

    // Generate new key
    let (key, prefix, hash) = generate_api_key();
    let now = Utc::now();

    // Update the key in database
    sqlx::query!(
        r#"
        UPDATE api_keys
        SET key_hash = $1, key_prefix = $2, created_at = $3, usage_count = 0, last_used_at = NULL
        WHERE id = $4 AND user_id = $5
        "#,
        hash,
        prefix,
        now,
        key_id,
        user.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let scopes: Vec<ApiKeyScope> = serde_json::from_value(existing.scopes.unwrap_or_default())
        .unwrap_or_default();

    let response = CreateApiKeyResponse {
        id: key_id,
        name: existing.name,
        key,
        key_prefix: prefix,
        scopes,
        expires_at: existing.expires_at,
        created_at: now,
    };

    Ok(Json(response))
}

/// Middleware to validate API key from Authorization header
pub async fn validate_api_key_header(
    state: &AppState,
    headers: &HeaderMap,
    required_scope: Option<&ApiKeyScope>,
) -> ApiResult<ApiKey> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized("Invalid authorization format".to_string()));
    }

    let key = &auth_header[7..];

    // Extract prefix for lookup
    let prefix = super::api_keys::extract_key_prefix(key)
        .ok_or_else(|| AppError::Unauthorized("Invalid API key format".to_string()))?;

    // Look up key by prefix
    let stored_key = sqlx::query!(
        r#"
        SELECT
            id, user_id, name, description, key_hash, key_prefix,
            scopes, expires_at, allowed_ips, rate_limit, is_active,
            created_at, last_used_at, usage_count
        FROM api_keys
        WHERE key_prefix = $1 AND is_active = true
        "#,
        prefix
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::Unauthorized("Invalid API key".to_string()))?;

    // Verify the key hash
    if !super::api_keys::verify_api_key(key, &stored_key.key_hash) {
        return Err(AppError::Unauthorized("Invalid API key".to_string()));
    }

    // Check expiration
    if let Some(expires_at) = stored_key.expires_at {
        if expires_at < Utc::now() {
            return Err(AppError::Unauthorized("API key has expired".to_string()));
        }
    }

    // Parse scopes
    let scopes: Vec<ApiKeyScope> = serde_json::from_value(stored_key.scopes.unwrap_or_default())
        .unwrap_or_default();

    // Check required scope if specified
    if let Some(required) = required_scope {
        if !has_scope(&scopes, required) {
            return Err(AppError::InsufficientPermissions {
                required: format!("{:?}", required),
            });
        }
    }

    // Update last used timestamp
    sqlx::query!(
        "UPDATE api_keys SET last_used_at = NOW(), usage_count = usage_count + 1 WHERE id = $1",
        stored_key.id
    )
    .execute(&state.db_pool)
    .await
    .ok(); // Don't fail the request if this update fails

    let allowed_ips: Vec<String> = stored_key.allowed_ips.unwrap_or_default();

    Ok(ApiKey {
        id: stored_key.id,
        user_id: stored_key.user_id,
        name: stored_key.name,
        description: stored_key.description,
        key_hash: stored_key.key_hash,
        key_prefix: stored_key.key_prefix,
        scopes,
        expires_at: stored_key.expires_at,
        allowed_ips,
        rate_limit: stored_key.rate_limit.unwrap_or(0) as u32,
        is_active: stored_key.is_active,
        created_at: stored_key.created_at,
        last_used_at: stored_key.last_used_at,
        usage_count: stored_key.usage_count as u64,
    })
}
