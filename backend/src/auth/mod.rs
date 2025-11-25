pub mod oauth;
pub mod jwt;
pub mod middleware;
pub mod totp;
pub mod providers;
pub mod oidc;
pub mod oidc_handlers;
pub mod saml;
pub mod saml_handlers;
pub mod api_keys;
pub mod api_key_handlers;
pub mod rbac;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Redirect},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use crate::AppState;
use resolve_shared::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub mfa_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role_id: Option<Uuid>,
    pub avatar_url: Option<String>,
    pub mfa_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
    pub provider: String,
}

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Local authentication
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/refresh", post(refresh_token))
        // Legacy OAuth (kept for backwards compatibility)
        .route("/oauth/providers", get(get_oauth_providers))
        .route("/oauth/:provider", get(oauth_login))
        .route("/oauth/callback", get(oauth_callback))
        // MFA
        .route("/mfa/setup", post(setup_mfa))
        .route("/mfa/verify", post(verify_mfa))
        .route("/mfa/disable", post(disable_mfa))
        // OIDC (Azure AD, Google, etc.)
        .nest("/oidc", oidc_handlers::oidc_routes())
        // SAML 2.0
        .nest("/saml", saml_handlers::saml_routes())
        // API Keys
        .nest("/api-keys", api_key_handlers::api_key_routes())
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // First try to find user by email
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 AND is_active = true"
    )
    .bind(&req.email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = match user {
        Some(u) => u,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Check if account is locked
    if let Some(locked_until) = user.locked_until {
        if locked_until > chrono::Utc::now() {
            return Err(StatusCode::LOCKED);
        }
    }

    // Verify password for local auth users
    if let Some(password_hash) = &user.password_hash {
        let parsed_hash = argon2::PasswordHash::new(password_hash)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let valid = argon2::Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .is_ok();

        if !valid {
            // Increment failed login attempts
            sqlx::query(
                "UPDATE users SET failed_login_attempts = failed_login_attempts + 1,
                 locked_until = CASE WHEN failed_login_attempts >= 4 THEN NOW() + INTERVAL '15 minutes' ELSE NULL END
                 WHERE id = $1"
            )
            .bind(user.id)
            .execute(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        // User only has OAuth login, password login not allowed
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check MFA if enabled
    if user.mfa_enabled {
        if let Some(mfa_code) = req.mfa_code {
            if let Some(mfa_secret) = &user.mfa_secret {
                let decrypted_secret = crate::auth::totp::decrypt_mfa_secret(mfa_secret)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                
                if !crate::auth::totp::verify_totp(&decrypted_secret, &mfa_code) {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            } else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        } else {
            // MFA required but not provided
            return Ok(Json(serde_json::json!({
                "error": "mfa_required",
                "message": "MFA code required"
            })).into_response());
        }
    }

    // Reset failed login attempts on successful login
    sqlx::query(
        "UPDATE users SET failed_login_attempts = 0, locked_until = NULL, last_login_at = NOW() WHERE id = $1"
    )
    .bind(user.id)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate JWT token
    let token_data = jwt::create_jwt(&user).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = LoginResponse {
        token: token_data.token,
        user: UserResponse {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role_id: user.role_id,
            avatar_url: user.avatar_url,
            mfa_enabled: user.mfa_enabled,
        },
        expires_at: token_data.expires_at,
    };

    Ok(Json(response))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check if user already exists
    let existing_user = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    // Hash password
    use argon2::password_hash::rand_core::OsRng;
    use argon2::password_hash::SaltString;
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    // Create user
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, password_hash, timezone, is_active, mfa_enabled, failed_login_attempts)
         VALUES ($1, $2, $3, $4, $5, 'UTC', true, false, 0)"
    )
    .bind(user_id)
    .bind(&req.email)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(password_hash)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

async fn logout() -> impl IntoResponse {
    // In a more sophisticated implementation, you'd maintain a token blacklist
    StatusCode::OK
}

async fn me(
    middleware::AuthUser(user): middleware::AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let response = UserResponse {
        id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        role_id: user.role_id,
        avatar_url: user.avatar_url,
        mfa_enabled: user.mfa_enabled,
    };

    Ok(Json(response))
}

async fn refresh_token(
    middleware::AuthUser(user): middleware::AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let token_data = jwt::create_jwt(&user).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "token": token_data.token,
        "expires_at": token_data.expires_at
    })))
}

async fn get_oauth_providers(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let providers = sqlx::query_as::<_, resolve_shared::AuthProvider>(
        "SELECT * FROM auth_providers WHERE enabled = true ORDER BY name"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(providers))
}

async fn oauth_login(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(provider): axum::extract::Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_url = oauth::get_authorization_url(&state.db_pool, &provider)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Redirect::to(&auth_url))
}

async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = oauth::handle_oauth_callback(&state.db_pool, query)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token_data = jwt::create_jwt(&user).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Redirect to frontend with token (in a real app, this would be more secure)
    let redirect_url = format!("/auth/callback?token={}", token_data.token);
    Ok(Redirect::to(&redirect_url))
}

async fn setup_mfa(
    State(state): State<Arc<AppState>>,
    middleware::AuthUser(user): middleware::AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    if user.mfa_enabled {
        return Err(StatusCode::BAD_REQUEST);
    }

    let secret = totp::generate_secret();
    let encrypted_secret = totp::encrypt_mfa_secret(&secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let qr_code = totp::generate_qr_code(&user.email, &secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Store the secret temporarily (user needs to verify it to enable MFA)
    sqlx::query("UPDATE users SET mfa_secret = $1 WHERE id = $2")
        .bind(encrypted_secret)
        .bind(user.id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "secret": secret,
        "qr_code": qr_code
    })))
}

async fn verify_mfa(
    State(state): State<Arc<AppState>>,
    middleware::AuthUser(user): middleware::AuthUser,
    Json(req): Json<serde_json::Value>,
) -> Result<impl IntoResponse, StatusCode> {
    let mfa_code = req.get("code")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if let Some(mfa_secret) = &user.mfa_secret {
        let decrypted_secret = totp::decrypt_mfa_secret(mfa_secret)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if totp::verify_totp(&decrypted_secret, mfa_code) {
            // Enable MFA
            sqlx::query("UPDATE users SET mfa_enabled = true WHERE id = $1")
                .bind(user.id)
                .execute(&state.db_pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "MFA enabled successfully"
            })))
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn disable_mfa(
    State(state): State<Arc<AppState>>,
    middleware::AuthUser(user): middleware::AuthUser,
    Json(req): Json<serde_json::Value>,
) -> Result<impl IntoResponse, StatusCode> {
    let mfa_code = req.get("code")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if !user.mfa_enabled {
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Some(mfa_secret) = &user.mfa_secret {
        let decrypted_secret = totp::decrypt_mfa_secret(mfa_secret)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if totp::verify_totp(&decrypted_secret, mfa_code) {
            // Disable MFA and clear secret
            sqlx::query("UPDATE users SET mfa_enabled = false, mfa_secret = NULL WHERE id = $1")
                .bind(user.id)
                .execute(&state.db_pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "MFA disabled successfully"
            })))
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

fn generate_salt() -> Vec<u8> {
    use rand::RngCore;
    let mut salt = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}
// Simple token extraction for handlers that need basic verification
pub fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let auth_header = headers.get("authorization")?;
    let auth_str = auth_header.to_str().ok()?;
    
    if auth_str.starts_with("Bearer ") {
        Some(auth_str[7..].to_string())
    } else {
        None
    }
}

// Simple token verification
pub fn verify_token(token: &str) -> Result<jwt::Claims, axum::http::StatusCode> {
    jwt::verify_jwt(token)
        .map(|token_data| token_data.claims)
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)
}
