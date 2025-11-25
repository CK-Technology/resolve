use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    AuthUrl, TokenUrl, basic::BasicClient, TokenResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use resolve_shared::{AuthProvider, User};
use super::OAuthCallbackQuery;

#[derive(Debug, Serialize, Deserialize)]
struct OAuthUserInfo {
    id: String,
    email: String,
    name: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    avatar_url: Option<String>,
}

pub async fn get_authorization_url(
    db_pool: &sqlx::PgPool,
    provider_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let provider = sqlx::query_as::<_, AuthProvider>(
        "SELECT * FROM auth_providers WHERE name = $1 AND enabled = true"
    )
    .bind(provider_name)
    .fetch_optional(db_pool)
    .await?
    .ok_or("Provider not found or disabled")?;

    let client = create_oauth_client(&provider)?;
    
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(provider.scopes.iter().map(|s| Scope::new(s.clone())))
        .url();

    Ok(auth_url.to_string())
}

pub async fn handle_oauth_callback(
    db_pool: &sqlx::PgPool,
    callback: OAuthCallbackQuery,
) -> Result<User, Box<dyn std::error::Error>> {
    let provider = sqlx::query_as::<_, AuthProvider>(
        "SELECT * FROM auth_providers WHERE name = $1 AND enabled = true"
    )
    .bind(&callback.provider)
    .fetch_optional(db_pool)
    .await?
    .ok_or("Provider not found or disabled")?;

    let client = create_oauth_client(&provider)?;
    
    // Exchange authorization code for access token
    let token_result = client
        .exchange_code(AuthorizationCode::new(callback.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await?;

    // Get user info from the OAuth provider
    let user_info = get_user_info(&provider, token_result.access_token().secret()).await?;

    // Find or create user
    let user = find_or_create_oauth_user(db_pool, &provider, &user_info).await?;

    Ok(user)
}

async fn get_user_info(
    provider: &AuthProvider,
    access_token: &str,
) -> Result<OAuthUserInfo, Box<dyn std::error::Error>> {
    let userinfo_url = provider.userinfo_url.as_ref()
        .ok_or("Provider has no userinfo URL configured")?;

    let client = reqwest::Client::new();
    let response = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await?;

    let user_data: serde_json::Value = response.json().await?;

    // Map different provider response formats to our standard format
    let user_info = match provider.name.as_str() {
        "Google" => map_google_user_info(&user_data)?,
        "Microsoft Azure" => map_azure_user_info(&user_data)?,
        "GitHub" => map_github_user_info(&user_data, access_token).await?,
        _ => return Err(format!("Unsupported provider: {}", provider.name).into()),
    };

    Ok(user_info)
}

fn map_google_user_info(data: &serde_json::Value) -> Result<OAuthUserInfo, Box<dyn std::error::Error>> {
    Ok(OAuthUserInfo {
        id: data.get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing Google user ID")?
            .to_string(),
        email: data.get("email")
            .and_then(|v| v.as_str())
            .ok_or("Missing Google email")?
            .to_string(),
        name: data.get("name").and_then(|v| v.as_str()).map(String::from),
        first_name: data.get("given_name").and_then(|v| v.as_str()).map(String::from),
        last_name: data.get("family_name").and_then(|v| v.as_str()).map(String::from),
        avatar_url: data.get("picture").and_then(|v| v.as_str()).map(String::from),
    })
}

fn map_azure_user_info(data: &serde_json::Value) -> Result<OAuthUserInfo, Box<dyn std::error::Error>> {
    Ok(OAuthUserInfo {
        id: data.get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing Azure user ID")?
            .to_string(),
        email: data.get("mail")
            .or_else(|| data.get("userPrincipalName"))
            .and_then(|v| v.as_str())
            .ok_or("Missing Azure email")?
            .to_string(),
        name: data.get("displayName").and_then(|v| v.as_str()).map(String::from),
        first_name: data.get("givenName").and_then(|v| v.as_str()).map(String::from),
        last_name: data.get("surname").and_then(|v| v.as_str()).map(String::from),
        avatar_url: None, // Azure doesn't provide avatar URL in basic profile
    })
}

async fn map_github_user_info(
    data: &serde_json::Value,
    access_token: &str,
) -> Result<OAuthUserInfo, Box<dyn std::error::Error>> {
    let id = data.get("id")
        .and_then(|v| v.as_i64())
        .ok_or("Missing GitHub user ID")?
        .to_string();

    // GitHub API doesn't always return email in the user endpoint
    let mut email = data.get("email").and_then(|v| v.as_str()).map(String::from);
    
    // If no email, fetch from the emails endpoint
    if email.is_none() {
        let client = reqwest::Client::new();
        let emails_response = client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("User-Agent", "Resolve")
            .send()
            .await?;

        if emails_response.status().is_success() {
            let emails: Vec<serde_json::Value> = emails_response.json().await?;
            email = emails
                .iter()
                .find(|e| e.get("primary").and_then(|v| v.as_bool()).unwrap_or(false))
                .and_then(|e| e.get("email"))
                .and_then(|v| v.as_str())
                .map(String::from);
        }
    }

    let full_name = data.get("name").and_then(|v| v.as_str()).map(String::from);
    let (first_name, last_name) = if let Some(name) = &full_name {
        let parts: Vec<&str> = name.splitn(2, ' ').collect();
        (
            Some(parts[0].to_string()),
            if parts.len() > 1 { Some(parts[1].to_string()) } else { None }
        )
    } else {
        (None, None)
    };

    Ok(OAuthUserInfo {
        id,
        email: email.ok_or("No email available from GitHub")?,
        name: full_name,
        first_name,
        last_name,
        avatar_url: data.get("avatar_url").and_then(|v| v.as_str()).map(String::from),
    })
}

async fn find_or_create_oauth_user(
    db_pool: &sqlx::PgPool,
    provider: &AuthProvider,
    user_info: &OAuthUserInfo,
) -> Result<User, Box<dyn std::error::Error>> {
    // First, try to find user by OAuth provider and ID
    if let Some(user) = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE oauth_provider = $1 AND oauth_id = $2 AND is_active = true"
    )
    .bind(&provider.name)
    .bind(&user_info.id)
    .fetch_optional(db_pool)
    .await?
    {
        // Update last login
        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(user.id)
            .execute(db_pool)
            .await?;

        return Ok(user);
    }

    // Try to find user by email (for linking existing accounts)
    if let Some(mut user) = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 AND is_active = true"
    )
    .bind(&user_info.email)
    .fetch_optional(db_pool)
    .await?
    {
        // Link OAuth account to existing user
        sqlx::query(
            "UPDATE users SET oauth_provider = $1, oauth_id = $2, last_login_at = NOW() WHERE id = $3"
        )
        .bind(&provider.name)
        .bind(&user_info.id)
        .bind(user.id)
        .execute(db_pool)
        .await?;

        user.oauth_provider = Some(provider.name.clone());
        user.oauth_id = Some(user_info.id.clone());

        return Ok(user);
    }

    // Create new user
    let user_id = Uuid::new_v4();
    let first_name = user_info.first_name.clone()
        .or_else(|| user_info.name.clone())
        .unwrap_or_else(|| "User".to_string());
    let last_name = user_info.last_name.clone().unwrap_or_else(|| "".to_string());

    sqlx::query(
        "INSERT INTO users (
            id, email, first_name, last_name, timezone, is_active, mfa_enabled,
            oauth_provider, oauth_id, failed_login_attempts, avatar_url, last_login_at
        ) VALUES ($1, $2, $3, $4, 'UTC', true, false, $5, $6, 0, $7, NOW())"
    )
    .bind(user_id)
    .bind(&user_info.email)
    .bind(&first_name)
    .bind(&last_name)
    .bind(&provider.name)
    .bind(&user_info.id)
    .bind(&user_info.avatar_url)
    .execute(db_pool)
    .await?;

    // Fetch the newly created user
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(db_pool)
    .await?;

    Ok(user)
}

fn create_oauth_client(provider: &AuthProvider) -> Result<BasicClient, Box<dyn std::error::Error>> {
    let client_id = provider.client_id.as_ref()
        .ok_or("Provider has no client ID configured")?;
    let client_secret = provider.client_secret.as_ref()
        .ok_or("Provider has no client secret configured")?;
    let auth_url = provider.auth_url.as_ref()
        .ok_or("Provider has no auth URL configured")?;
    let token_url = provider.token_url.as_ref()
        .ok_or("Provider has no token URL configured")?;

    let redirect_url = std::env::var("OAUTH_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:3000/api/v1/auth/oauth/callback".to_string());

    Ok(BasicClient::new(
        ClientId::new(client_id.clone()),
        Some(ClientSecret::new(client_secret.clone())),
        AuthUrl::new(auth_url.clone())?,
        Some(TokenUrl::new(token_url.clone())?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url)?))
}