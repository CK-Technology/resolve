//! OIDC HTTP handlers
//!
//! Provides REST endpoints for OpenID Connect authentication.
//! Supports Azure AD, Google, and generic OIDC providers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::jwt;
use super::oidc::{
    generate_code_verifier, generate_nonce, generate_state, OidcProviderConfig,
    OidcProviderType, RoleMapping,
};
use crate::error::{AppError, ApiResult};
use crate::AppState;
use resolve_shared::User;

/// Query params for the OAuth callback
#[derive(Debug, Deserialize)]
pub struct OidcCallbackQuery {
    pub code: String,
    pub state: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

/// Response for listing available OIDC providers
#[derive(Debug, Serialize)]
pub struct OidcProviderInfo {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub provider_type: String,
    pub logo_url: Option<String>,
}

pub fn oidc_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/providers", get(list_oidc_providers))
        .route("/login/:provider", get(oidc_login))
        .route("/callback", get(oidc_callback))
}

/// List all enabled OIDC providers
async fn list_oidc_providers(
    State(state): State<Arc<AppState>>,
) -> ApiResult<impl IntoResponse> {
    let providers = sqlx::query!(
        r#"
        SELECT id, name, display_name, provider_type
        FROM auth_providers
        WHERE provider_type IN ('oidc', 'oauth2') AND enabled = true
        ORDER BY display_name
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let providers: Vec<OidcProviderInfo> = providers
        .into_iter()
        .map(|p| {
            let logo_url = match p.provider_type.as_str() {
                "oidc" if p.name.contains("azure") => {
                    Some("https://aadcdn.msftauthimages.net/dbd5a2dd-ugwgqqaweuzrv5dxv0zb5q8k9nwlbhlqldl3hlx-kkm/logintenantbranding/0/bannerlogo".to_string())
                }
                "oidc" if p.name.contains("google") => {
                    Some("https://www.gstatic.com/firebasejs/ui/2.0.0/images/auth/google.svg".to_string())
                }
                _ => None,
            };
            OidcProviderInfo {
                id: p.id,
                name: p.name,
                display_name: p.display_name,
                provider_type: p.provider_type,
                logo_url,
            }
        })
        .collect();

    Ok(Json(providers))
}

/// Start OIDC login flow
async fn oidc_login(
    State(state): State<Arc<AppState>>,
    Path(provider_name): Path<String>,
) -> ApiResult<impl IntoResponse> {
    // Fetch provider configuration
    let provider_record = sqlx::query!(
        r#"
        SELECT
            id, name, provider_type, client_id, client_secret, tenant_id,
            auth_url, token_url, userinfo_url, issuer_url, jwks_url,
            scopes, allowed_domains, default_role_id, role_mapping
        FROM auth_providers
        WHERE name = $1 AND enabled = true
        "#,
        provider_name
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::ProviderNotFound(provider_name.clone()))?;

    // Determine provider type
    let provider_type = match provider_record.name.to_lowercase() {
        n if n.contains("azure") || n.contains("microsoft") => OidcProviderType::AzureAd,
        n if n.contains("google") => OidcProviderType::Google,
        _ => OidcProviderType::Generic,
    };

    // Build provider config
    let config = OidcProviderConfig {
        provider_id: provider_record.id,
        provider_type: provider_type.clone(),
        client_id: provider_record.client_id,
        client_secret: provider_record.client_secret.unwrap_or_default(),
        tenant_id: provider_record.tenant_id,
        issuer_url: provider_record.issuer_url,
        auth_url: provider_record.auth_url,
        token_url: provider_record.token_url,
        userinfo_url: provider_record.userinfo_url,
        jwks_url: provider_record.jwks_url,
        scopes: provider_record.scopes.unwrap_or_else(|| vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ]),
        allowed_domains: provider_record.allowed_domains.unwrap_or_default(),
        role_mapping: provider_record
            .role_mapping
            .and_then(|v| serde_json::from_value(v).ok()),
    };

    // Generate PKCE challenge
    let code_verifier = generate_code_verifier();
    let code_challenge = super::oidc::generate_code_challenge(&code_verifier);

    // Generate state and nonce
    let state_value = generate_state();
    let nonce = generate_nonce();

    // Store state for verification
    let state_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::minutes(10);

    sqlx::query!(
        r#"
        INSERT INTO oauth_states (id, state, provider_id, provider_type, code_verifier, nonce, expires_at)
        VALUES ($1, $2, $3, 'oidc', $4, $5, $6)
        "#,
        state_id,
        state_value,
        config.provider_id,
        code_verifier,
        nonce,
        expires_at
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Build authorization URL
    let redirect_uri = std::env::var("OAUTH_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:8080/api/v1/auth/oidc/callback".to_string());

    let auth_url = config.auth_url.as_ref().unwrap_or(
        &super::oidc::get_azure_auth_url(config.tenant_id.as_deref().unwrap_or("common")),
    );

    let scopes = config.scopes.join(" ");

    let authorization_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&nonce={}&code_challenge={}&code_challenge_method=S256&response_mode=query",
        auth_url,
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&scopes),
        urlencoding::encode(&state_value),
        urlencoding::encode(&nonce),
        urlencoding::encode(&code_challenge)
    );

    // Add Azure-specific parameters
    let authorization_url = if matches!(config.provider_type, OidcProviderType::AzureAd) {
        // Add prompt parameter to ensure account selection
        format!("{}&prompt=select_account", authorization_url)
    } else {
        authorization_url
    };

    Ok(Redirect::to(&authorization_url))
}

/// Handle OIDC callback
async fn oidc_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OidcCallbackQuery>,
) -> ApiResult<impl IntoResponse> {
    // Check for error response from IdP
    if let Some(error) = query.error {
        let description = query.error_description.unwrap_or_else(|| "Unknown error".to_string());
        return Err(AppError::OAuthError(format!("{}: {}", error, description)));
    }

    // Verify state and get stored data
    let stored_state = sqlx::query!(
        r#"
        SELECT id, provider_id, code_verifier, nonce
        FROM oauth_states
        WHERE state = $1 AND expires_at > NOW()
        "#,
        query.state
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::OAuthError("Invalid or expired state".to_string()))?;

    // Delete used state
    sqlx::query!("DELETE FROM oauth_states WHERE id = $1", stored_state.id)
        .execute(&state.db_pool)
        .await
        .ok();

    // Get provider config
    let provider_record = sqlx::query!(
        r#"
        SELECT
            id, name, provider_type, client_id, client_secret, tenant_id,
            auth_url, token_url, userinfo_url, issuer_url, jwks_url,
            scopes, allowed_domains, auto_create_users, default_role_id, role_mapping
        FROM auth_providers
        WHERE id = $1 AND enabled = true
        "#,
        stored_state.provider_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::ProviderDisabled("Provider".to_string()))?;

    // Determine provider type
    let provider_type = match provider_record.name.to_lowercase() {
        n if n.contains("azure") || n.contains("microsoft") => OidcProviderType::AzureAd,
        n if n.contains("google") => OidcProviderType::Google,
        _ => OidcProviderType::Generic,
    };

    // Exchange code for tokens
    let redirect_uri = std::env::var("OAUTH_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:8080/api/v1/auth/oidc/callback".to_string());

    let token_url = provider_record.token_url.as_ref().unwrap_or(
        &super::oidc::get_azure_token_url(
            provider_record.tenant_id.as_deref().unwrap_or("common"),
        ),
    );

    let client = reqwest::Client::new();
    let token_response = client
        .post(token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", &provider_record.client_id),
            (
                "client_secret",
                provider_record.client_secret.as_deref().unwrap_or(""),
            ),
            ("code", &query.code),
            ("redirect_uri", &redirect_uri),
            (
                "code_verifier",
                stored_state.code_verifier.as_deref().unwrap_or(""),
            ),
        ])
        .send()
        .await
        .map_err(|e| AppError::OAuthError(format!("Token exchange failed: {}", e)))?;

    if !token_response.status().is_success() {
        let error_body = token_response.text().await.unwrap_or_default();
        return Err(AppError::OAuthError(format!(
            "Token exchange failed: {}",
            error_body
        )));
    }

    let tokens: TokenResponse = token_response
        .json()
        .await
        .map_err(|e| AppError::OAuthError(format!("Failed to parse token response: {}", e)))?;

    // Decode and validate ID token
    let id_token_claims = decode_id_token(&tokens.id_token, &stored_state.nonce)?;

    // Check allowed domains
    if !provider_record.allowed_domains.as_ref().map_or(true, |domains| {
        domains.is_empty()
            || id_token_claims
                .email
                .as_ref()
                .map_or(false, |email| {
                    domains.iter().any(|d| email.ends_with(&format!("@{}", d)))
                })
    }) {
        return Err(AppError::Forbidden(
            "Email domain not allowed for this provider".to_string(),
        ));
    }

    // Find or create user
    let user = find_or_create_oidc_user(
        &state.db_pool,
        &provider_record.id,
        &provider_record.name,
        &id_token_claims,
        provider_record.auto_create_users,
        provider_record.default_role_id,
    )
    .await?;

    // Store OAuth connection
    sqlx::query!(
        r#"
        INSERT INTO user_oauth_connections (
            user_id, provider_type, provider_id, external_id, external_email,
            access_token, refresh_token, token_expires_at, last_login_at
        )
        VALUES ($1, 'oidc', $2, $3, $4, $5, $6, $7, NOW())
        ON CONFLICT (provider_type, provider_id, external_id)
        DO UPDATE SET
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            token_expires_at = EXCLUDED.token_expires_at,
            last_login_at = NOW()
        "#,
        user.id,
        provider_record.id,
        id_token_claims.sub,
        id_token_claims.email,
        tokens.access_token,
        tokens.refresh_token,
        tokens.expires_in.map(|s| Utc::now() + Duration::seconds(s as i64))
    )
    .execute(&state.db_pool)
    .await
    .ok();

    // Generate JWT for our application
    let token_data = jwt::create_jwt(&user).map_err(|e| AppError::InternalError(e.to_string()))?;

    // Redirect to frontend with token
    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "/".to_string());
    let redirect_url = format!(
        "{}?token={}&expires_at={}",
        frontend_url, token_data.token, token_data.expires_at
    );

    Ok(Redirect::to(&redirect_url))
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<i64>,
    refresh_token: Option<String>,
    id_token: String,
    scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct IdTokenClaims {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
    preferred_username: Option<String>,
    // Azure-specific
    oid: Option<String>,
    tid: Option<String>,
    // Groups (if configured)
    groups: Option<Vec<String>>,
}

/// Decode ID token (simplified - in production, verify signature with JWKS)
fn decode_id_token(id_token: &str, expected_nonce: &Option<String>) -> ApiResult<IdTokenClaims> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err(AppError::OAuthError("Invalid ID token format".to_string()));
    }

    // Decode payload (middle part)
    let payload = base64_decode_url_safe(parts[1])
        .map_err(|_| AppError::OAuthError("Failed to decode ID token payload".to_string()))?;

    let claims: serde_json::Value = serde_json::from_slice(&payload)
        .map_err(|_| AppError::OAuthError("Failed to parse ID token claims".to_string()))?;

    // Verify nonce if present
    if let Some(expected) = expected_nonce {
        let token_nonce = claims.get("nonce").and_then(|v| v.as_str());
        if token_nonce != Some(expected.as_str()) {
            return Err(AppError::OAuthError("Invalid nonce in ID token".to_string()));
        }
    }

    // Verify token hasn't expired
    if let Some(exp) = claims.get("exp").and_then(|v| v.as_i64()) {
        if exp < Utc::now().timestamp() {
            return Err(AppError::TokenExpired);
        }
    }

    let id_claims: IdTokenClaims = serde_json::from_value(claims)
        .map_err(|_| AppError::OAuthError("Failed to parse ID token claims".to_string()))?;

    Ok(id_claims)
}

fn base64_decode_url_safe(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::{engine::general_purpose, Engine as _};

    // Add padding if needed
    let padded = match input.len() % 4 {
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => input.to_string(),
    };

    // URL-safe base64 decode
    general_purpose::URL_SAFE_NO_PAD
        .decode(input)
        .or_else(|_| general_purpose::URL_SAFE.decode(&padded))
        .or_else(|_| general_purpose::STANDARD.decode(&padded))
}

async fn find_or_create_oidc_user(
    db_pool: &sqlx::PgPool,
    provider_id: &Uuid,
    provider_name: &str,
    claims: &IdTokenClaims,
    auto_create: bool,
    default_role_id: Option<Uuid>,
) -> ApiResult<User> {
    // First check if we have an existing OAuth connection
    let existing_connection = sqlx::query!(
        r#"
        SELECT user_id
        FROM user_oauth_connections
        WHERE provider_type = 'oidc' AND provider_id = $1 AND external_id = $2
        "#,
        provider_id,
        claims.sub
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if let Some(conn) = existing_connection {
        // Get the user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
            .bind(conn.user_id)
            .fetch_optional(db_pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User".to_string()))?;

        // Update last login
        sqlx::query!("UPDATE users SET last_login_at = NOW() WHERE id = $1", user.id)
            .execute(db_pool)
            .await
            .ok();

        return Ok(user);
    }

    // Try to find user by email
    let email = claims
        .email
        .as_ref()
        .or(claims.preferred_username.as_ref())
        .ok_or_else(|| AppError::OAuthError("No email in ID token".to_string()))?;

    if let Some(user) = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 AND is_active = true"
    )
    .bind(email)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    {
        // Update last login
        sqlx::query!("UPDATE users SET last_login_at = NOW() WHERE id = $1", user.id)
            .execute(db_pool)
            .await
            .ok();

        return Ok(user);
    }

    // Create new user if allowed
    if !auto_create {
        return Err(AppError::Forbidden(
            "User registration is not enabled for this provider".to_string(),
        ));
    }

    let user_id = Uuid::new_v4();
    let first_name = claims
        .given_name
        .clone()
        .or_else(|| {
            claims
                .name
                .as_ref()
                .map(|n| n.split_whitespace().next().unwrap_or("User").to_string())
        })
        .unwrap_or_else(|| "User".to_string());

    let last_name = claims.family_name.clone().or_else(|| {
        claims.name.as_ref().and_then(|n| {
            n.split_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ")
                .into()
        })
    });

    sqlx::query!(
        r#"
        INSERT INTO users (
            id, email, first_name, last_name, avatar_url, role_id,
            timezone, is_active, mfa_enabled, failed_login_attempts,
            oauth_provider, oauth_id, last_login_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'UTC', true, false, 0, $7, $8, NOW())
        "#,
        user_id,
        email,
        first_name,
        last_name.as_deref().unwrap_or(""),
        claims.picture,
        default_role_id,
        provider_name,
        claims.sub
    )
    .execute(db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(user)
}
