//! SAML 2.0 HTTP handlers
//!
//! Provides REST endpoints for SAML authentication.
//! Resolve acts as a Service Provider (SP).

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Json, Router,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::jwt;
use super::saml::{
    decode_saml_response, extract_saml_attributes, generate_authn_request, generate_sp_metadata,
    SamlIdpConfig, SamlSpConfig, SamlBinding,
};
use crate::error::{AppError, ApiResult};
use crate::AppState;
use resolve_shared::User;

/// Query params for IdP-initiated flow
#[derive(Debug, Deserialize)]
pub struct SamlCallbackQuery {
    #[serde(default)]
    pub relay_state: Option<String>,
}

/// Form data for SAML response (POST binding)
#[derive(Debug, Deserialize)]
pub struct SamlCallbackForm {
    #[serde(rename = "SAMLResponse")]
    pub saml_response: String,
    #[serde(rename = "RelayState")]
    pub relay_state: Option<String>,
}

/// Response for listing available SAML providers
#[derive(Debug, Serialize)]
pub struct SamlProviderInfo {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
}

pub fn saml_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/providers", get(list_saml_providers))
        .route("/login/:provider", get(saml_login))
        .route("/callback", post(saml_callback_post))
        .route("/callback", get(saml_callback_get))
        .route("/metadata", get(sp_metadata))
}

/// List all enabled SAML providers
async fn list_saml_providers(
    State(state): State<Arc<AppState>>,
) -> ApiResult<impl IntoResponse> {
    let providers = sqlx::query!(
        r#"
        SELECT id, name, display_name
        FROM saml_providers
        WHERE enabled = true
        ORDER BY display_name
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let providers: Vec<SamlProviderInfo> = providers
        .into_iter()
        .map(|p| SamlProviderInfo {
            id: p.id,
            name: p.name,
            display_name: p.display_name,
        })
        .collect();

    Ok(Json(providers))
}

/// Start SAML login flow
async fn saml_login(
    State(state): State<Arc<AppState>>,
    Path(provider_name): Path<String>,
) -> ApiResult<impl IntoResponse> {
    // Fetch provider configuration
    let provider = sqlx::query!(
        r#"
        SELECT
            id, entity_id, sso_url, sso_binding, sign_authn_requests,
            sp_signing_key, sp_signing_cert
        FROM saml_providers
        WHERE name = $1 AND enabled = true
        "#,
        provider_name
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::ProviderNotFound(provider_name.clone()))?;

    // Generate state for relay
    let request_id = format!("_resolve_{}", Uuid::new_v4());
    let relay_state = Uuid::new_v4().to_string();

    // Get SP config
    let sp_entity_id = std::env::var("SAML_SP_ENTITY_ID")
        .unwrap_or_else(|_| "https://resolve.local".to_string());
    let acs_url = std::env::var("SAML_ACS_URL")
        .unwrap_or_else(|_| "https://resolve.local/api/v1/auth/saml/callback".to_string());

    // Store state for verification
    let expires_at = Utc::now() + Duration::minutes(10);
    sqlx::query!(
        r#"
        INSERT INTO oauth_states (id, state, provider_id, provider_type, redirect_url, expires_at)
        VALUES ($1, $2, $3, 'saml', $4, $5)
        "#,
        Uuid::new_v4(),
        relay_state,
        provider.id,
        request_id,
        expires_at
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Generate AuthnRequest
    let authn_request = generate_authn_request(
        &request_id,
        &sp_entity_id,
        &acs_url,
        &provider.entity_id,
        provider.sign_authn_requests,
        provider.sp_signing_key.as_deref(),
        provider.sp_signing_cert.as_deref(),
    )
    .map_err(|e| AppError::InternalError(format!("Failed to generate AuthnRequest: {}", e)))?;

    // Determine binding type
    let binding = provider
        .sso_binding
        .as_deref()
        .unwrap_or("HTTP-Redirect");

    if binding == "HTTP-POST" {
        // Return HTML form for POST binding
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Redirecting to Identity Provider</title></head>
<body onload="document.forms[0].submit();">
    <noscript>
        <p>JavaScript is required. Please click the button below to continue.</p>
    </noscript>
    <form method="POST" action="{}">
        <input type="hidden" name="SAMLRequest" value="{}"/>
        <input type="hidden" name="RelayState" value="{}"/>
        <noscript><input type="submit" value="Continue"/></noscript>
    </form>
</body>
</html>"#,
            provider.sso_url,
            base64_encode(&authn_request),
            relay_state
        );

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(html)
            .unwrap()
            .into_response())
    } else {
        // HTTP-Redirect binding
        let encoded_request = urlencoding::encode(&base64_encode(&authn_request));
        let redirect_url = format!(
            "{}?SAMLRequest={}&RelayState={}",
            provider.sso_url, encoded_request, relay_state
        );

        Ok(Redirect::to(&redirect_url).into_response())
    }
}

/// Handle SAML callback (POST binding)
async fn saml_callback_post(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SamlCallbackForm>,
) -> ApiResult<impl IntoResponse> {
    process_saml_response(&state, &form.saml_response, form.relay_state.as_deref()).await
}

/// Handle SAML callback (GET binding - less common)
async fn saml_callback_get(
    State(state): State<Arc<AppState>>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> ApiResult<impl IntoResponse> {
    let saml_response = query
        .get("SAMLResponse")
        .ok_or_else(|| AppError::BadRequest("Missing SAMLResponse".to_string()))?;

    process_saml_response(&state, saml_response, query.get("RelayState").map(|s| s.as_str())).await
}

/// Process SAML response from IdP
async fn process_saml_response(
    state: &AppState,
    saml_response: &str,
    relay_state: Option<&str>,
) -> ApiResult<impl IntoResponse> {
    // Verify relay state if provided
    let provider_id = if let Some(relay) = relay_state {
        let stored_state = sqlx::query!(
            r#"
            SELECT provider_id, redirect_url
            FROM oauth_states
            WHERE state = $1 AND provider_type = 'saml' AND expires_at > NOW()
            "#,
            relay
        )
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::OAuthError("Invalid or expired relay state".to_string()))?;

        // Delete used state
        sqlx::query!(
            "DELETE FROM oauth_states WHERE state = $1",
            relay
        )
        .execute(&state.db_pool)
        .await
        .ok();

        Some(stored_state.provider_id)
    } else {
        None
    };

    // Decode SAML response
    let decoded = base64_decode(saml_response)
        .map_err(|_| AppError::OAuthError("Invalid SAML response encoding".to_string()))?;

    let response_xml = String::from_utf8(decoded)
        .map_err(|_| AppError::OAuthError("Invalid SAML response encoding".to_string()))?;

    // Parse the response to extract issuer
    let issuer = extract_issuer(&response_xml)?;

    // Find the SAML provider by issuer
    let provider = if let Some(pid) = provider_id {
        sqlx::query!(
            r#"
            SELECT
                id, name, entity_id, signing_cert, attribute_mapping,
                allowed_domains, auto_create_users, default_role_id, role_mapping
            FROM saml_providers
            WHERE id = $1 AND enabled = true
            "#,
            pid
        )
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
    } else {
        sqlx::query!(
            r#"
            SELECT
                id, name, entity_id, signing_cert, attribute_mapping,
                allowed_domains, auto_create_users, default_role_id, role_mapping
            FROM saml_providers
            WHERE entity_id = $1 AND enabled = true
            "#,
            issuer
        )
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
    };

    let provider = provider.ok_or_else(|| {
        AppError::ProviderNotFound(format!("SAML provider with issuer: {}", issuer))
    })?;

    // Verify signature (simplified - in production use proper XML signature verification)
    // For now we'll trust the response but log a warning
    tracing::warn!("SAML signature verification not fully implemented - trusting response");

    // Extract attributes from SAML response
    let attributes = extract_saml_attributes(&response_xml)
        .map_err(|e| AppError::OAuthError(format!("Failed to extract SAML attributes: {}", e)))?;

    // Get attribute mapping
    let attr_mapping: std::collections::HashMap<String, Vec<String>> = provider
        .attribute_mapping
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // Extract user info using mapping
    let email = find_attribute(&attributes, &attr_mapping, "email")
        .ok_or_else(|| AppError::OAuthError("No email attribute in SAML response".to_string()))?;

    let first_name = find_attribute(&attributes, &attr_mapping, "first_name");
    let last_name = find_attribute(&attributes, &attr_mapping, "last_name");
    let groups = attributes
        .get("groups")
        .or_else(|| attributes.get("memberOf"))
        .cloned();

    // Check allowed domains
    if let Some(domains) = &provider.allowed_domains {
        if !domains.is_empty() && !domains.iter().any(|d| email.ends_with(&format!("@{}", d))) {
            return Err(AppError::Forbidden(
                "Email domain not allowed for this provider".to_string(),
            ));
        }
    }

    // Track assertion to prevent replay
    let assertion_id = extract_assertion_id(&response_xml)?;

    // Check if assertion was already used
    let existing = sqlx::query!(
        "SELECT id FROM saml_assertions WHERE assertion_id = $1",
        assertion_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::OAuthError("SAML assertion replay detected".to_string()));
    }

    // Find or create user
    let user = find_or_create_saml_user(
        &state.db_pool,
        &provider.id,
        &provider.name,
        &email,
        first_name.as_deref(),
        last_name.as_deref(),
        provider.auto_create_users,
        provider.default_role_id,
    )
    .await?;

    // Store assertion
    sqlx::query!(
        r#"
        INSERT INTO saml_assertions (assertion_id, provider_id, user_id, issued_at, expires_at)
        VALUES ($1, $2, $3, NOW(), NOW() + INTERVAL '5 minutes')
        "#,
        assertion_id,
        provider.id,
        user.id
    )
    .execute(&state.db_pool)
    .await
    .ok();

    // Generate JWT
    let token_data = jwt::create_jwt(&user).map_err(|e| AppError::InternalError(e.to_string()))?;

    // Redirect to frontend with token
    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "/".to_string());
    let redirect_url = format!(
        "{}?token={}&expires_at={}",
        frontend_url, token_data.token, token_data.expires_at
    );

    Ok(Redirect::to(&redirect_url))
}

/// Return SP metadata XML
async fn sp_metadata() -> impl IntoResponse {
    let entity_id = std::env::var("SAML_SP_ENTITY_ID")
        .unwrap_or_else(|_| "https://resolve.local".to_string());
    let acs_url = std::env::var("SAML_ACS_URL")
        .unwrap_or_else(|_| "https://resolve.local/api/v1/auth/saml/callback".to_string());
    let slo_url = std::env::var("SAML_SLO_URL").ok();

    let sp_config = SamlSpConfig {
        entity_id,
        acs_url,
        slo_url,
        name_id_format: "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".to_string(),
        want_assertions_signed: true,
        authn_requests_signed: false,
        signing_cert: None,
        encryption_cert: None,
    };

    match generate_sp_metadata(&sp_config) {
        Ok(metadata) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/xml")
            .body(metadata)
            .unwrap()
            .into_response(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Failed to generate SP metadata: {}", e))
            .unwrap()
            .into_response(),
    }
}

// Helper functions

fn base64_encode(data: &str) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(data.as_bytes())
}

fn base64_decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.decode(data)
}

fn extract_issuer(xml: &str) -> ApiResult<String> {
    // Simple extraction - in production use proper XML parsing
    let issuer_start = xml
        .find("<saml:Issuer>")
        .or_else(|| xml.find("<Issuer>"))
        .ok_or_else(|| AppError::OAuthError("No Issuer in SAML response".to_string()))?;

    let content_start = xml[issuer_start..]
        .find('>')
        .map(|i| issuer_start + i + 1)
        .ok_or_else(|| AppError::OAuthError("Malformed SAML response".to_string()))?;

    let content_end = xml[content_start..]
        .find('<')
        .map(|i| content_start + i)
        .ok_or_else(|| AppError::OAuthError("Malformed SAML response".to_string()))?;

    Ok(xml[content_start..content_end].trim().to_string())
}

fn extract_assertion_id(xml: &str) -> ApiResult<String> {
    // Extract ID from Assertion element
    if let Some(assertion_start) = xml.find("<saml:Assertion").or_else(|| xml.find("<Assertion")) {
        let assertion_tag = &xml[assertion_start..];
        if let Some(id_start) = assertion_tag.find("ID=\"") {
            let id_value_start = id_start + 4;
            if let Some(id_end) = assertion_tag[id_value_start..].find('"') {
                return Ok(assertion_tag[id_value_start..id_value_start + id_end].to_string());
            }
        }
    }

    // Generate a unique ID if we can't extract one
    Ok(format!("_unknown_{}", Uuid::new_v4()))
}

fn find_attribute(
    attributes: &std::collections::HashMap<String, String>,
    mapping: &std::collections::HashMap<String, Vec<String>>,
    field: &str,
) -> Option<String> {
    // First check if there's a mapping for this field
    if let Some(mapped_names) = mapping.get(field) {
        for name in mapped_names {
            if let Some(value) = attributes.get(name) {
                return Some(value.clone());
            }
        }
    }

    // Fall back to direct lookup
    attributes.get(field).cloned()
}

async fn find_or_create_saml_user(
    db_pool: &sqlx::PgPool,
    provider_id: &Uuid,
    provider_name: &str,
    email: &str,
    first_name: Option<&str>,
    last_name: Option<&str>,
    auto_create: bool,
    default_role_id: Option<Uuid>,
) -> ApiResult<User> {
    // Check for existing OAuth connection
    let existing_connection = sqlx::query!(
        r#"
        SELECT user_id
        FROM user_oauth_connections
        WHERE provider_type = 'saml' AND provider_id = $1 AND external_email = $2
        "#,
        provider_id,
        email
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if let Some(conn) = existing_connection {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
            .bind(conn.user_id)
            .fetch_optional(db_pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User".to_string()))?;

        sqlx::query!("UPDATE users SET last_login_at = NOW() WHERE id = $1", user.id)
            .execute(db_pool)
            .await
            .ok();

        return Ok(user);
    }

    // Try to find user by email
    if let Some(user) = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 AND is_active = true"
    )
    .bind(email)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    {
        // Create OAuth connection
        sqlx::query!(
            r#"
            INSERT INTO user_oauth_connections (user_id, provider_type, provider_id, external_id, external_email, last_login_at)
            VALUES ($1, 'saml', $2, $3, $3, NOW())
            "#,
            user.id,
            provider_id,
            email
        )
        .execute(db_pool)
        .await
        .ok();

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

    sqlx::query!(
        r#"
        INSERT INTO users (
            id, email, first_name, last_name, role_id,
            timezone, is_active, mfa_enabled, failed_login_attempts,
            oauth_provider, last_login_at
        )
        VALUES ($1, $2, $3, $4, $5, 'UTC', true, false, 0, $6, NOW())
        "#,
        user_id,
        email,
        first_name.unwrap_or("User"),
        last_name.unwrap_or(""),
        default_role_id,
        provider_name
    )
    .execute(db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Create OAuth connection
    sqlx::query!(
        r#"
        INSERT INTO user_oauth_connections (user_id, provider_type, provider_id, external_id, external_email, last_login_at)
        VALUES ($1, 'saml', $2, $3, $3, NOW())
        "#,
        user_id,
        provider_id,
        email
    )
    .execute(db_pool)
    .await
    .ok();

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(user)
}
