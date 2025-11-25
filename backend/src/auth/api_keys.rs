//! API Key management for machine-to-machine authentication
//!
//! Provides secure API key generation, validation, and management for:
//! - Third-party integrations
//! - Automation scripts
//! - CI/CD pipelines
//! - External applications

use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;

use crate::error::{AppError, ApiResult};

/// API Key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    /// User who owns this key
    pub user_id: Uuid,
    /// Human-readable name for the key
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Hashed key value (original is only shown once on creation)
    #[serde(skip_serializing)]
    pub key_hash: String,
    /// Key prefix for identification (first 8 chars)
    pub key_prefix: String,
    /// Permissions/scopes granted to this key
    pub scopes: Vec<ApiKeyScope>,
    /// Optional expiration date
    pub expires_at: Option<DateTime<Utc>>,
    /// IP whitelist (empty = allow all)
    pub allowed_ips: Vec<String>,
    /// Rate limit (requests per minute, 0 = unlimited)
    pub rate_limit: u32,
    /// Whether key is currently active
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    /// Number of times this key has been used
    pub usage_count: u64,
}

/// API Key scope/permission
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyScope {
    // Read scopes
    ReadClients,
    ReadTickets,
    ReadAssets,
    ReadPasswords,
    ReadDocumentation,
    ReadInvoices,
    ReadReports,

    // Write scopes
    WriteClients,
    WriteTickets,
    WriteAssets,
    WritePasswords,
    WriteDocumentation,
    WriteInvoices,

    // Admin scopes
    ManageUsers,
    ManageSettings,
    ManageIntegrations,

    // Special scopes
    FullAccess,
    WebhooksOnly,
}

impl ApiKeyScope {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::ReadClients => "Read client information",
            Self::ReadTickets => "Read tickets",
            Self::ReadAssets => "Read assets",
            Self::ReadPasswords => "Read passwords (requires additional auth)",
            Self::ReadDocumentation => "Read documentation and KB",
            Self::ReadInvoices => "Read invoices and billing",
            Self::ReadReports => "Read reports and analytics",
            Self::WriteClients => "Create and update clients",
            Self::WriteTickets => "Create and update tickets",
            Self::WriteAssets => "Create and update assets",
            Self::WritePasswords => "Create and update passwords",
            Self::WriteDocumentation => "Create and update documentation",
            Self::WriteInvoices => "Create and update invoices",
            Self::ManageUsers => "Manage user accounts",
            Self::ManageSettings => "Manage system settings",
            Self::ManageIntegrations => "Manage integrations",
            Self::FullAccess => "Full API access (all permissions)",
            Self::WebhooksOnly => "Receive and send webhooks only",
        }
    }

    /// Check if this scope grants a specific permission
    pub fn grants(&self, required: &ApiKeyScope) -> bool {
        if self == &Self::FullAccess {
            return true;
        }
        self == required
    }
}

/// Request to create a new API key
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<ApiKeyScope>,
    /// Expiration in days (None = never expires)
    pub expires_in_days: Option<u32>,
    pub allowed_ips: Vec<String>,
    pub rate_limit: Option<u32>,
}

/// Response when creating an API key (includes the actual key)
#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    /// The actual API key - only shown once!
    pub key: String,
    pub key_prefix: String,
    pub scopes: Vec<ApiKeyScope>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Generate a new API key
///
/// Key format: `resolve_<prefix>_<random>`
/// Example: `resolve_abc12345_x7k9m2p4q8r1s5t6u3v0w`
pub fn generate_api_key() -> (String, String, String) {
    let mut prefix_bytes = [0u8; 4];
    let mut secret_bytes = [0u8; 24];

    rand::thread_rng().fill_bytes(&mut prefix_bytes);
    rand::thread_rng().fill_bytes(&mut secret_bytes);

    let prefix = hex::encode(prefix_bytes);
    let secret = base64_url_safe_encode(&secret_bytes);

    let key = format!("resolve_{}_{}", prefix, secret);
    let hash = hash_api_key(&key);

    (key, prefix, hash)
}

/// Hash an API key for storage
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verify an API key against a stored hash
pub fn verify_api_key(key: &str, stored_hash: &str) -> bool {
    let key_hash = hash_api_key(key);
    // Use constant-time comparison to prevent timing attacks
    constant_time_eq(key_hash.as_bytes(), stored_hash.as_bytes())
}

/// Extract the prefix from an API key for lookup
pub fn extract_key_prefix(key: &str) -> Option<String> {
    // Key format: resolve_<prefix>_<secret>
    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() == 3 && parts[0] == "resolve" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Validate API key format
pub fn validate_key_format(key: &str) -> bool {
    let parts: Vec<&str> = key.split('_').collect();
    parts.len() == 3
        && parts[0] == "resolve"
        && parts[1].len() == 8  // prefix is 8 hex chars
        && parts[2].len() == 32 // secret is 32 base64 chars
}

/// Check if a key has a specific scope
pub fn has_scope(key_scopes: &[ApiKeyScope], required: &ApiKeyScope) -> bool {
    key_scopes.iter().any(|s| s.grants(required))
}

/// Check if a key has all required scopes
pub fn has_all_scopes(key_scopes: &[ApiKeyScope], required: &[ApiKeyScope]) -> bool {
    required.iter().all(|r| has_scope(key_scopes, r))
}

/// Check if a key has any of the required scopes
pub fn has_any_scope(key_scopes: &[ApiKeyScope], required: &[ApiKeyScope]) -> bool {
    required.iter().any(|r| has_scope(key_scopes, r))
}

/// Validate API key request
pub fn validate_create_request(req: &CreateApiKeyRequest) -> ApiResult<()> {
    if req.name.is_empty() {
        return Err(AppError::BadRequest("API key name is required".to_string()));
    }

    if req.name.len() > 100 {
        return Err(AppError::BadRequest("API key name is too long".to_string()));
    }

    if req.scopes.is_empty() {
        return Err(AppError::BadRequest("At least one scope is required".to_string()));
    }

    // Validate IP addresses if provided
    for ip in &req.allowed_ips {
        if !is_valid_ip_or_cidr(ip) {
            return Err(AppError::BadRequest(format!("Invalid IP address: {}", ip)));
        }
    }

    Ok(())
}

/// Check if an IP is in the allowed list
pub fn is_ip_allowed(client_ip: &str, allowed_ips: &[String]) -> bool {
    if allowed_ips.is_empty() {
        return true; // No restrictions
    }

    for allowed in allowed_ips {
        if allowed.contains('/') {
            // CIDR notation
            if ip_in_cidr(client_ip, allowed) {
                return true;
            }
        } else if allowed == client_ip {
            return true;
        }
    }

    false
}

// Helper: URL-safe base64 encoding without padding
fn base64_url_safe_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

// Helper: Constant-time string comparison
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

// Helper: Validate IP or CIDR notation
fn is_valid_ip_or_cidr(ip: &str) -> bool {
    if ip.contains('/') {
        // CIDR notation
        let parts: Vec<&str> = ip.split('/').collect();
        if parts.len() != 2 {
            return false;
        }
        parts[0].parse::<std::net::IpAddr>().is_ok()
            && parts[1].parse::<u8>().is_ok()
    } else {
        ip.parse::<std::net::IpAddr>().is_ok()
    }
}

// Helper: Check if IP is in CIDR range (simplified)
fn ip_in_cidr(ip: &str, cidr: &str) -> bool {
    // Simplified implementation - in production use ipnetwork crate
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    let network_ip = match parts[0].parse::<std::net::Ipv4Addr>() {
        Ok(ip) => ip,
        Err(_) => return false,
    };

    let client_ip = match ip.parse::<std::net::Ipv4Addr>() {
        Ok(ip) => ip,
        Err(_) => return false,
    };

    let prefix_len: u8 = match parts[1].parse() {
        Ok(len) if len <= 32 => len,
        _ => return false,
    };

    let mask = if prefix_len == 0 {
        0u32
    } else {
        !0u32 << (32 - prefix_len)
    };

    let network_bits = u32::from(network_ip) & mask;
    let client_bits = u32::from(client_ip) & mask;

    network_bits == client_bits
}

/// Rate limiter for API keys
pub struct ApiKeyRateLimiter {
    /// Map of key_prefix -> (request_count, window_start)
    state: std::sync::RwLock<std::collections::HashMap<String, (u32, DateTime<Utc>)>>,
}

impl ApiKeyRateLimiter {
    pub fn new() -> Self {
        Self {
            state: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Check if a request is allowed and update counter
    /// Returns Ok(remaining) or Err with retry_after seconds
    pub fn check_rate_limit(&self, key_prefix: &str, limit: u32) -> Result<u32, u64> {
        if limit == 0 {
            return Ok(u32::MAX); // No limit
        }

        let now = Utc::now();
        let window_duration = chrono::Duration::minutes(1);

        let mut state = self.state.write().unwrap();
        let entry = state.entry(key_prefix.to_string()).or_insert((0, now));

        // Check if we're in a new window
        if now - entry.1 > window_duration {
            *entry = (1, now);
            return Ok(limit - 1);
        }

        // Check if limit exceeded
        if entry.0 >= limit {
            let retry_after = (entry.1 + window_duration - now).num_seconds().max(0) as u64;
            return Err(retry_after);
        }

        // Increment counter
        entry.0 += 1;
        Ok(limit - entry.0)
    }

    /// Clean up old entries
    pub fn cleanup(&self) {
        let now = Utc::now();
        let window_duration = chrono::Duration::minutes(2); // Keep a bit longer than window

        let mut state = self.state.write().unwrap();
        state.retain(|_, (_, window_start)| now - *window_start < window_duration);
    }
}

impl Default for ApiKeyRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let (key, prefix, hash) = generate_api_key();

        assert!(key.starts_with("resolve_"));
        assert_eq!(prefix.len(), 8);
        assert!(validate_key_format(&key));
        assert!(verify_api_key(&key, &hash));
    }

    #[test]
    fn test_extract_prefix() {
        let prefix = extract_key_prefix("resolve_abc12345_secretsecretsecretsecret");
        assert_eq!(prefix, Some("abc12345".to_string()));

        let invalid = extract_key_prefix("invalid_key");
        assert_eq!(invalid, None);
    }

    #[test]
    fn test_scope_grants() {
        assert!(ApiKeyScope::FullAccess.grants(&ApiKeyScope::ReadClients));
        assert!(ApiKeyScope::ReadClients.grants(&ApiKeyScope::ReadClients));
        assert!(!ApiKeyScope::ReadClients.grants(&ApiKeyScope::WriteClients));
    }

    #[test]
    fn test_ip_cidr() {
        assert!(ip_in_cidr("192.168.1.100", "192.168.1.0/24"));
        assert!(!ip_in_cidr("192.168.2.100", "192.168.1.0/24"));
        assert!(ip_in_cidr("10.0.0.1", "10.0.0.0/8"));
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = ApiKeyRateLimiter::new();

        // First request should succeed
        let result = limiter.check_rate_limit("test_key", 3);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        // Second and third should also succeed
        assert!(limiter.check_rate_limit("test_key", 3).is_ok());
        assert!(limiter.check_rate_limit("test_key", 3).is_ok());

        // Fourth should fail
        let result = limiter.check_rate_limit("test_key", 3);
        assert!(result.is_err());
    }
}
