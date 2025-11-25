use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub display_name: String,
    pub provider_type: String,
    pub auth_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub scopes: Vec<String>,
    pub icon: String,
    pub color: String,
}

pub fn get_provider_configs() -> HashMap<String, ProviderConfig> {
    let mut providers = HashMap::new();

    providers.insert("google".to_string(), ProviderConfig {
        name: "google".to_string(),
        display_name: "Google".to_string(),
        provider_type: "oauth2".to_string(),
        auth_url: "https://accounts.google.com/o/oauth2/auth".to_string(),
        token_url: "https://oauth2.googleapis.com/token".to_string(),
        userinfo_url: "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
        scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        icon: "google".to_string(),
        color: "#db4437".to_string(),
    });

    providers.insert("microsoft".to_string(), ProviderConfig {
        name: "microsoft".to_string(),
        display_name: "Microsoft".to_string(),
        provider_type: "oauth2".to_string(),
        auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
        token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
        userinfo_url: "https://graph.microsoft.com/v1.0/me".to_string(),
        scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        icon: "microsoft".to_string(),
        color: "#0078d4".to_string(),
    });

    providers.insert("github".to_string(), ProviderConfig {
        name: "github".to_string(),
        display_name: "GitHub".to_string(),
        provider_type: "oauth2".to_string(),
        auth_url: "https://github.com/login/oauth/authorize".to_string(),
        token_url: "https://github.com/login/oauth/access_token".to_string(),
        userinfo_url: "https://api.github.com/user".to_string(),
        scopes: vec!["user:email".to_string()],
        icon: "github".to_string(),
        color: "#333333".to_string(),
    });

    providers.insert("azure".to_string(), ProviderConfig {
        name: "azure".to_string(),
        display_name: "Azure AD".to_string(),
        provider_type: "oauth2".to_string(),
        auth_url: "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize".to_string(),
        token_url: "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token".to_string(),
        userinfo_url: "https://graph.microsoft.com/v1.0/me".to_string(),
        scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        icon: "microsoft".to_string(),
        color: "#0078d4".to_string(),
    });

    providers
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub provider_type: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: Option<String>,
    pub token_url: Option<String>,
    pub userinfo_url: Option<String>,
    pub scopes: Vec<String>,
    pub enabled: bool,
}

impl CreateProviderRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Provider name is required".to_string());
        }

        if self.client_id.is_empty() {
            return Err("Client ID is required".to_string());
        }

        if self.client_secret.is_empty() {
            return Err("Client secret is required".to_string());
        }

        match self.provider_type.as_str() {
            "oauth2" | "oidc" => {
                if self.auth_url.is_none() || self.token_url.is_none() {
                    return Err("Auth URL and Token URL are required for OAuth2/OIDC".to_string());
                }
            },
            "saml" => {
                // SAML validation would go here
            },
            _ => {
                return Err("Invalid provider type".to_string());
            }
        }

        Ok(())
    }
}