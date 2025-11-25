//! OpenID Connect (OIDC) authentication support
//!
//! Implements OIDC authentication for:
//! - Microsoft Entra ID (Azure AD) - Multi-tenant support
//! - Google Workspace
//! - Generic OIDC providers

use openidconnect::{
    core::{
        CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreProviderMetadata,
        CoreResponseType, CoreTokenResponse,
    },
    reqwest::async_http_client,
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl,
    Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{AppError, ApiResult};

/// OIDC Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProviderConfig {
    pub provider_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub provider_type: OidcProviderType,
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: String,
    /// For Azure AD: "common", "organizations", "consumers", or specific tenant ID
    pub tenant_id: Option<String>,
    /// Additional scopes beyond openid, profile, email
    pub additional_scopes: Vec<String>,
    pub enabled: bool,
    /// Allow new user registration via this provider
    pub allow_registration: bool,
    /// Restrict to specific email domains (empty = allow all)
    pub allowed_domains: Vec<String>,
    /// Map provider groups/roles to Resolve roles
    pub role_mapping: Option<RoleMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OidcProviderType {
    /// Microsoft Entra ID (Azure AD)
    AzureAd,
    /// Google Workspace / Google Cloud Identity
    Google,
    /// Generic OIDC provider
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleMapping {
    /// Azure AD group ID -> Resolve role ID
    pub group_to_role: std::collections::HashMap<String, Uuid>,
    /// Default role if no mapping matches
    pub default_role_id: Option<Uuid>,
}

/// OIDC authentication state stored during auth flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcAuthState {
    pub provider_id: Uuid,
    pub csrf_token: String,
    pub nonce: String,
    pub pkce_verifier: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Where to redirect after successful auth
    pub return_url: Option<String>,
}

/// User info extracted from OIDC claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcUserInfo {
    /// Provider's unique identifier for the user
    pub subject: String,
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    /// Azure AD: user's tenant ID
    pub tenant_id: Option<String>,
    /// Azure AD: group memberships (if groups claim is requested)
    pub groups: Vec<String>,
    /// Raw claims for custom processing
    pub raw_claims: serde_json::Value,
}

/// Cached OIDC client for a provider
pub struct OidcClientCache {
    clients: RwLock<std::collections::HashMap<Uuid, Arc<CachedOidcClient>>>,
}

struct CachedOidcClient {
    client: CoreClient,
    provider_metadata: CoreProviderMetadata,
    cached_at: chrono::DateTime<chrono::Utc>,
}

impl OidcClientCache {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Get or create an OIDC client for a provider
    pub async fn get_client(
        &self,
        config: &OidcProviderConfig,
        redirect_url: &str,
    ) -> ApiResult<Arc<CachedOidcClient>> {
        // Check cache first
        {
            let clients = self.clients.read().await;
            if let Some(cached) = clients.get(&config.provider_id) {
                // Cache for 1 hour
                if cached.cached_at + chrono::Duration::hours(1) > chrono::Utc::now() {
                    return Ok(Arc::clone(cached));
                }
            }
        }

        // Create new client
        let client = create_oidc_client(config, redirect_url).await?;
        let cached = Arc::new(client);

        // Store in cache
        {
            let mut clients = self.clients.write().await;
            clients.insert(config.provider_id, Arc::clone(&cached));
        }

        Ok(cached)
    }

    /// Clear cached client (e.g., when config changes)
    pub async fn invalidate(&self, provider_id: Uuid) {
        let mut clients = self.clients.write().await;
        clients.remove(&provider_id);
    }
}

impl Default for OidcClientCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Create an OIDC client from provider configuration
async fn create_oidc_client(
    config: &OidcProviderConfig,
    redirect_url: &str,
) -> ApiResult<CachedOidcClient> {
    let issuer_url = get_issuer_url(config)?;

    // Discover provider metadata
    let provider_metadata =
        CoreProviderMetadata::discover_async(issuer_url.clone(), async_http_client)
            .await
            .map_err(|e| AppError::OAuthError(format!("Failed to discover OIDC metadata: {}", e)))?;

    // Create client
    let client = CoreClient::from_provider_metadata(
        provider_metadata.clone(),
        ClientId::new(config.client_id.clone()),
        Some(ClientSecret::new(config.client_secret.clone())),
    )
    .set_redirect_uri(
        RedirectUrl::new(redirect_url.to_string())
            .map_err(|e| AppError::OAuthError(format!("Invalid redirect URL: {}", e)))?,
    );

    Ok(CachedOidcClient {
        client,
        provider_metadata,
        cached_at: chrono::Utc::now(),
    })
}

/// Get the issuer URL for a provider
fn get_issuer_url(config: &OidcProviderConfig) -> ApiResult<IssuerUrl> {
    let url = match config.provider_type {
        OidcProviderType::AzureAd => {
            let tenant = config.tenant_id.as_deref().unwrap_or("common");
            format!("https://login.microsoftonline.com/{}/v2.0", tenant)
        }
        OidcProviderType::Google => "https://accounts.google.com".to_string(),
        OidcProviderType::Generic => config.issuer_url.clone(),
    };

    IssuerUrl::new(url).map_err(|e| AppError::OAuthError(format!("Invalid issuer URL: {}", e)))
}

/// Generate authorization URL for OIDC login
pub async fn generate_auth_url(
    cache: &OidcClientCache,
    config: &OidcProviderConfig,
    redirect_url: &str,
    return_url: Option<String>,
) -> ApiResult<(String, OidcAuthState)> {
    let cached_client = cache.get_client(config, redirect_url).await?;

    // Generate PKCE challenge
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate CSRF token and nonce
    let csrf_token = CsrfToken::new_random();
    let nonce = Nonce::new_random();

    // Build authorization URL
    let mut auth_request = cached_client
        .client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            || csrf_token.clone(),
            || nonce.clone(),
        )
        .set_pkce_challenge(pkce_challenge)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()));

    // Add provider-specific scopes
    match config.provider_type {
        OidcProviderType::AzureAd => {
            // Request offline_access for refresh tokens
            auth_request = auth_request.add_scope(Scope::new("offline_access".to_string()));
            // Request group memberships (requires Azure AD app permission)
            // auth_request = auth_request.add_scope(Scope::new("GroupMember.Read.All".to_string()));
        }
        OidcProviderType::Google => {
            // Google-specific scopes if needed
        }
        OidcProviderType::Generic => {}
    }

    // Add additional configured scopes
    for scope in &config.additional_scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.clone()));
    }

    let (auth_url, _, _) = auth_request.url();

    let state = OidcAuthState {
        provider_id: config.provider_id,
        csrf_token: csrf_token.secret().clone(),
        nonce: nonce.secret().clone(),
        pkce_verifier: pkce_verifier.secret().clone(),
        created_at: chrono::Utc::now(),
        return_url,
    };

    Ok((auth_url.to_string(), state))
}

/// Exchange authorization code for tokens and extract user info
pub async fn exchange_code(
    cache: &OidcClientCache,
    config: &OidcProviderConfig,
    redirect_url: &str,
    code: &str,
    state: &OidcAuthState,
) -> ApiResult<(OidcUserInfo, CoreTokenResponse)> {
    let cached_client = cache.get_client(config, redirect_url).await?;

    // Exchange code for tokens
    let token_response = cached_client
        .client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(PkceCodeVerifier::new(state.pkce_verifier.clone()))
        .request_async(async_http_client)
        .await
        .map_err(|e| AppError::OAuthError(format!("Token exchange failed: {}", e)))?;

    // Extract and verify ID token
    let id_token = token_response
        .id_token()
        .ok_or_else(|| AppError::OAuthError("No ID token in response".to_string()))?;

    // Create verifier with nonce
    let verifier: CoreIdTokenVerifier<'_> = cached_client
        .client
        .id_token_verifier();

    let nonce = Nonce::new(state.nonce.clone());
    let claims = id_token
        .claims(&verifier, &nonce)
        .map_err(|e| AppError::OAuthError(format!("ID token verification failed: {}", e)))?;

    // Extract user info from claims
    let user_info = extract_user_info(config, claims)?;

    // Validate email domain if restrictions are configured
    if !config.allowed_domains.is_empty() {
        let email_domain = user_info
            .email
            .split('@')
            .nth(1)
            .ok_or_else(|| AppError::OAuthError("Invalid email format".to_string()))?;

        if !config.allowed_domains.iter().any(|d| d == email_domain) {
            return Err(AppError::Forbidden(format!(
                "Email domain '{}' is not allowed for this provider",
                email_domain
            )));
        }
    }

    Ok((user_info, token_response))
}

/// Extract user info from ID token claims
fn extract_user_info(
    config: &OidcProviderConfig,
    claims: &CoreIdTokenClaims,
) -> ApiResult<OidcUserInfo> {
    let subject = claims.subject().to_string();

    let email = claims
        .email()
        .map(|e| e.to_string())
        .ok_or_else(|| AppError::OAuthError("Email claim missing from ID token".to_string()))?;

    let email_verified = claims.email_verified().unwrap_or(false);

    let name = claims
        .name()
        .and_then(|n| n.get(None))
        .map(|n| n.to_string());

    let given_name = claims
        .given_name()
        .and_then(|n| n.get(None))
        .map(|n| n.to_string());

    let family_name = claims
        .family_name()
        .and_then(|n| n.get(None))
        .map(|n| n.to_string());

    let picture = claims
        .picture()
        .and_then(|p| p.get(None))
        .map(|p| p.to_string());

    // Extract provider-specific claims
    let (tenant_id, groups) = match config.provider_type {
        OidcProviderType::AzureAd => {
            // Azure AD includes tenant ID (tid) and groups in additional claims
            let additional = claims.additional_claims();
            let tid = additional
                .get("tid")
                .and_then(|v| v.as_str())
                .map(String::from);
            let groups = additional
                .get("groups")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default();
            (tid, groups)
        }
        _ => (None, Vec::new()),
    };

    // Serialize raw claims for custom processing
    let raw_claims = serde_json::to_value(claims.additional_claims())
        .unwrap_or_else(|_| serde_json::Value::Null);

    Ok(OidcUserInfo {
        subject,
        email,
        email_verified,
        name,
        given_name,
        family_name,
        picture,
        tenant_id,
        groups,
        raw_claims,
    })
}

/// Well-known OIDC endpoints for documentation
pub mod endpoints {
    pub const AZURE_AD_COMMON: &str = "https://login.microsoftonline.com/common/v2.0";
    pub const AZURE_AD_ORGANIZATIONS: &str = "https://login.microsoftonline.com/organizations/v2.0";
    pub const AZURE_AD_CONSUMERS: &str = "https://login.microsoftonline.com/consumers/v2.0";
    pub const GOOGLE: &str = "https://accounts.google.com";

    /// Get Azure AD issuer URL for a specific tenant
    pub fn azure_ad_tenant(tenant_id: &str) -> String {
        format!("https://login.microsoftonline.com/{}/v2.0", tenant_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issuer_url_azure() {
        let config = OidcProviderConfig {
            provider_id: Uuid::new_v4(),
            name: "azure".to_string(),
            display_name: "Azure AD".to_string(),
            provider_type: OidcProviderType::AzureAd,
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            issuer_url: String::new(),
            tenant_id: Some("my-tenant-id".to_string()),
            additional_scopes: vec![],
            enabled: true,
            allow_registration: true,
            allowed_domains: vec![],
            role_mapping: None,
        };

        let url = get_issuer_url(&config).unwrap();
        assert_eq!(
            url.as_str(),
            "https://login.microsoftonline.com/my-tenant-id/v2.0"
        );
    }

    #[test]
    fn test_issuer_url_google() {
        let config = OidcProviderConfig {
            provider_id: Uuid::new_v4(),
            name: "google".to_string(),
            display_name: "Google".to_string(),
            provider_type: OidcProviderType::Google,
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            issuer_url: String::new(),
            tenant_id: None,
            additional_scopes: vec![],
            enabled: true,
            allow_registration: true,
            allowed_domains: vec![],
            role_mapping: None,
        };

        let url = get_issuer_url(&config).unwrap();
        assert_eq!(url.as_str(), "https://accounts.google.com");
    }
}
