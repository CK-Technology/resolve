use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    async_trait,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::error::{ApiError, AppError};
use resolve_shared::User;
use super::jwt;
use super::rbac::{Resource, Action};
use super::api_keys::{ApiKey, ApiKeyScope};

/// Authenticated user extractor
#[derive(Debug, Clone)]
pub struct AuthUser(pub User);

/// Authenticated user with role and permissions loaded
#[derive(Debug, Clone)]
pub struct AuthUserWithRole {
    pub user: User,
    pub role_name: Option<String>,
    pub role_hierarchy: Option<i32>,
    pub permissions: Vec<String>,
}

/// API Key authentication extractor
#[derive(Debug, Clone)]
pub struct AuthApiKey {
    pub key: ApiKey,
    pub user: User,
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Extract Bearer token from Authorization header
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()).into_response())?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".to_string()).into_response())?;

        // Check if this is an API key (starts with "resolve_")
        if token.starts_with("resolve_") {
            return Err(AppError::Unauthorized("Use API key authentication endpoint".to_string()).into_response());
        }

        // Verify JWT token
        let token_data = jwt::verify_jwt(token)
            .map_err(|e| AppError::from(e).into_response())?;

        // Load user from database
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(token_data.claims.sub)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()).into_response())?
        .ok_or_else(|| AppError::Unauthorized("User not found or inactive".to_string()).into_response())?;

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if locked_until > chrono::Utc::now() {
                return Err(AppError::AccountLocked { until: locked_until }.into_response());
            }
        }

        Ok(AuthUser(user))
    }
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthUserWithRole {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // First get the authenticated user
        let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;

        // Load role and permissions if user has a role
        let (role_name, role_hierarchy, permissions) = if let Some(role_id) = user.role_id {
            // Get role info
            let role = sqlx::query!(
                "SELECT name, hierarchy FROM roles WHERE id = $1",
                role_id
            )
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()).into_response())?;

            // Get permissions
            let perms = sqlx::query!(
                r#"
                SELECT p.name
                FROM permissions p
                JOIN role_permissions rp ON rp.permission_id = p.id
                WHERE rp.role_id = $1
                "#,
                role_id
            )
            .fetch_all(&state.db_pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()).into_response())?;

            let permission_names: Vec<String> = perms.into_iter().map(|p| p.name).collect();

            if let Some(r) = role {
                (Some(r.name), Some(r.hierarchy), permission_names)
            } else {
                (None, None, vec![])
            }
        } else {
            (None, None, vec![])
        };

        Ok(AuthUserWithRole {
            user,
            role_name,
            role_hierarchy,
            permissions,
        })
    }
}

impl AuthUserWithRole {
    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        // Admin permission grants everything
        if self.permissions.contains(&"admin.all".to_string()) {
            return true;
        }
        self.permissions.contains(&permission.to_string())
    }

    /// Check if user has permission for a resource and action
    pub fn can(&self, resource: Resource, action: Action) -> bool {
        let permission = format!("{}.{}", resource.as_str(), action.as_str());
        self.has_permission(&permission)
    }

    /// Check if user has any of the given permissions
    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        if self.permissions.contains(&"admin.all".to_string()) {
            return true;
        }
        permissions.iter().any(|p| self.permissions.contains(&p.to_string()))
    }

    /// Check if user has all of the given permissions
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        if self.permissions.contains(&"admin.all".to_string()) {
            return true;
        }
        permissions.iter().all(|p| self.permissions.contains(&p.to_string()))
    }

    /// Check if user's role hierarchy is at least the given level
    pub fn has_hierarchy_level(&self, required: i32) -> bool {
        self.role_hierarchy.map_or(false, |h| h >= required)
    }

    /// Require a specific permission, returning an error if not present
    pub fn require_permission(&self, permission: &str) -> Result<(), AppError> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(AppError::InsufficientPermissions {
                required: permission.to_string(),
            })
        }
    }

    /// Require permission for a resource and action
    pub fn require(&self, resource: Resource, action: Action) -> Result<(), AppError> {
        if self.can(resource.clone(), action.clone()) {
            Ok(())
        } else {
            Err(AppError::InsufficientPermissions {
                required: format!("{}.{}", resource.as_str(), action.as_str()),
            })
        }
    }
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthApiKey {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Extract API key from Authorization header
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()).into_response())?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".to_string()).into_response())?;

        // Verify it's an API key format
        if !token.starts_with("resolve_") {
            return Err(AppError::Unauthorized("Invalid API key format".to_string()).into_response());
        }

        // Extract prefix
        let prefix = super::api_keys::extract_key_prefix(token)
            .ok_or_else(|| AppError::Unauthorized("Invalid API key format".to_string()).into_response())?;

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
        .map_err(|e| AppError::DatabaseError(e.to_string()).into_response())?
        .ok_or_else(|| AppError::Unauthorized("Invalid API key".to_string()).into_response())?;

        // Verify the key hash
        if !super::api_keys::verify_api_key(token, &stored_key.key_hash) {
            return Err(AppError::Unauthorized("Invalid API key".to_string()).into_response());
        }

        // Check expiration
        if let Some(expires_at) = stored_key.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(AppError::Unauthorized("API key has expired".to_string()).into_response());
            }
        }

        // Check IP whitelist if configured
        let allowed_ips: Vec<String> = stored_key.allowed_ips.clone().unwrap_or_default();
        if !allowed_ips.is_empty() {
            // Get client IP from headers (X-Forwarded-For or X-Real-IP)
            let client_ip = parts
                .headers
                .get("x-forwarded-for")
                .or_else(|| parts.headers.get("x-real-ip"))
                .and_then(|h| h.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());

            if let Some(ip) = client_ip {
                if !super::api_keys::is_ip_allowed(&ip, &allowed_ips) {
                    return Err(AppError::Forbidden(format!("IP {} not allowed", ip)).into_response());
                }
            }
        }

        // Load the user who owns this key
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(stored_key.user_id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()).into_response())?
        .ok_or_else(|| AppError::Unauthorized("API key owner not found".to_string()).into_response())?;

        // Update last used timestamp
        sqlx::query!(
            "UPDATE api_keys SET last_used_at = NOW(), usage_count = usage_count + 1 WHERE id = $1",
            stored_key.id
        )
        .execute(&state.db_pool)
        .await
        .ok();

        // Parse scopes
        let scopes: Vec<ApiKeyScope> = serde_json::from_value(stored_key.scopes.unwrap_or_default())
            .unwrap_or_default();

        let api_key = ApiKey {
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
        };

        Ok(AuthApiKey { key: api_key, user })
    }
}

impl AuthApiKey {
    /// Check if API key has a specific scope
    pub fn has_scope(&self, scope: &ApiKeyScope) -> bool {
        super::api_keys::has_scope(&self.key.scopes, scope)
    }

    /// Check if API key has any of the given scopes
    pub fn has_any_scope(&self, scopes: &[ApiKeyScope]) -> bool {
        super::api_keys::has_any_scope(&self.key.scopes, scopes)
    }

    /// Require a specific scope, returning an error if not present
    pub fn require_scope(&self, scope: &ApiKeyScope) -> Result<(), AppError> {
        if self.has_scope(scope) {
            Ok(())
        } else {
            Err(AppError::InsufficientPermissions {
                required: format!("{:?}", scope),
            })
        }
    }
}

/// Optional authentication - returns None if no auth provided instead of error
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<User>);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for OptionalAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Try to extract Bearer token
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|header| header.to_str().ok());

        if let Some(header) = auth_header {
            if let Some(token) = header.strip_prefix("Bearer ") {
                // Skip API keys
                if token.starts_with("resolve_") {
                    return Ok(OptionalAuthUser(None));
                }
                // Try to verify token and load user
                if let Ok(token_data) = jwt::verify_jwt(token) {
                    if let Ok(Some(user)) = sqlx::query_as::<_, User>(
                        "SELECT * FROM users WHERE id = $1 AND is_active = true"
                    )
                    .bind(token_data.claims.sub)
                    .fetch_optional(&state.db_pool)
                    .await
                    {
                        return Ok(OptionalAuthUser(Some(user)));
                    }
                }
            }
        }

        Ok(OptionalAuthUser(None))
    }
}

/// Combined auth extractor - accepts either JWT token or API key
#[derive(Debug, Clone)]
pub enum AuthEither {
    User(User),
    ApiKey(AuthApiKey),
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthEither {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()).into_response())?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".to_string()).into_response())?;

        if token.starts_with("resolve_") {
            // API key authentication
            let api_key = AuthApiKey::from_request_parts(parts, state).await?;
            Ok(AuthEither::ApiKey(api_key))
        } else {
            // JWT authentication
            let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;
            Ok(AuthEither::User(user))
        }
    }
}

impl AuthEither {
    /// Get the user associated with this auth (either the user directly or the API key owner)
    pub fn user(&self) -> &User {
        match self {
            AuthEither::User(u) => u,
            AuthEither::ApiKey(k) => &k.user,
        }
    }

    /// Check if this is API key auth
    pub fn is_api_key(&self) -> bool {
        matches!(self, AuthEither::ApiKey(_))
    }

    /// Get the API key if this is API key auth
    pub fn api_key(&self) -> Option<&ApiKey> {
        match self {
            AuthEither::ApiKey(k) => Some(&k.key),
            _ => None,
        }
    }
}