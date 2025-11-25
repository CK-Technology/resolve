use axum::http::{HeaderMap, HeaderValue, header::AUTHORIZATION};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestClaims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

impl TestClaims {
    pub fn new(user_id: Uuid, email: String, role: String) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::hours(1)).timestamp() as usize;
        let iat = now.timestamp() as usize;
        
        Self {
            sub: user_id.to_string(),
            email,
            role,
            exp,
            iat,
        }
    }
}

pub fn create_test_jwt(user_id: Uuid, email: &str, role: &str) -> String {
    let claims = TestClaims::new(user_id, email.to_string(), role.to_string());
    let secret = "test_secret_key_for_testing_only";
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Failed to create test JWT")
}

pub fn create_auth_headers(user_id: Uuid, email: &str, role: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let token = create_test_jwt(user_id, email, role);
    let auth_value = format!("Bearer {}", token);
    
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value).expect("Failed to create auth header"),
    );
    
    headers
}

pub fn create_admin_headers() -> HeaderMap {
    create_auth_headers(
        Uuid::new_v4(),
        "admin@resolve.test",
        "admin"
    )
}

pub fn create_user_headers() -> HeaderMap {
    create_auth_headers(
        Uuid::new_v4(),
        "user@resolve.test",
        "user"
    )
}

// Database test helpers
pub async fn count_table_rows(pool: &sqlx::PgPool, table: &str) -> i64 {
    let query = format!("SELECT COUNT(*) as count FROM {}", table);
    sqlx::query_scalar::<_, i64>(&query)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

pub async fn table_exists(pool: &sqlx::PgPool, table: &str) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = $1)"
    )
    .bind(table)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

// Mock external service helpers
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

pub async fn create_mock_m365_server() -> MockServer {
    let mock_server = MockServer::start().await;
    
    // Mock Microsoft Graph API token endpoint
    Mock::given(method("POST"))
        .and(path("/oauth2/v2.0/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_access_token",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;
    
    // Mock users endpoint
    Mock::given(method("GET"))
        .and(path("/v1.0/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "value": [
                {
                    "id": "12345678-1234-1234-1234-123456789012",
                    "userPrincipalName": "test@contoso.com",
                    "displayName": "Test User",
                    "accountEnabled": true,
                    "mail": "test@contoso.com"
                }
            ]
        })))
        .mount(&mock_server)
        .await;
    
    mock_server
}

pub async fn create_mock_azure_server() -> MockServer {
    let mock_server = MockServer::start().await;
    
    // Mock Azure Resource Manager API
    Mock::given(method("GET"))
        .and(path("/subscriptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "value": [
                {
                    "subscriptionId": "12345678-1234-1234-1234-123456789012",
                    "displayName": "Test Subscription",
                    "state": "Enabled",
                    "subscriptionPolicies": {
                        "spendingLimit": "Off"
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;
    
    mock_server
}

// Performance testing helpers
use std::time::Instant;

pub struct PerformanceTracker {
    start_time: Instant,
    name: String,
}

impl PerformanceTracker {
    pub fn new(name: &str) -> Self {
        Self {
            start_time: Instant::now(),
            name: name.to_string(),
        }
    }
    
    pub fn elapsed_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }
    
    pub fn assert_max_duration(&self, max_ms: u128) {
        let elapsed = self.elapsed_ms();
        assert!(
            elapsed <= max_ms,
            "{} took {}ms, expected max {}ms",
            self.name, elapsed, max_ms
        );
    }
}

impl Drop for PerformanceTracker {
    fn drop(&mut self) {
        let elapsed = self.elapsed_ms();
        if elapsed > 1000 {
            println!("⚠️  {} took {}ms (>1s)", self.name, elapsed);
        }
    }
}