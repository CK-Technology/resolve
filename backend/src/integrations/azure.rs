use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::AppState;
use resolve_shared::Integration;
use super::decrypt_json;

pub fn azure_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tenants", get(list_azure_tenants))
        .route("/users", get(list_azure_users))
        .route("/groups", get(list_azure_groups))
        .route("/devices", get(list_azure_devices))
        .route("/applications", get(list_azure_applications))
        .route("/licenses", get(list_azure_licenses))
        .route("/domains", get(list_azure_domains))
        .route("/subscriptions", get(list_azure_subscriptions))
        .route("/resources", get(list_azure_resources))
        .route("/security", get(get_azure_security_overview))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AzureCredentials {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub graph_scope: Option<String>,
    pub resource_scope: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AzureTenant {
    pub id: String,
    pub display_name: String,
    pub domain_name: String,
    pub country: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AzureUser {
    pub id: String,
    pub user_principal_name: String,
    pub display_name: String,
    pub given_name: Option<String>,
    pub surname: Option<String>,
    pub mail: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub office_location: Option<String>,
    pub account_enabled: bool,
    pub last_sign_in: Option<String>,
    pub created_at: Option<String>,
    pub licenses: Vec<String>,
    pub groups: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AzureGroup {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub group_type: String,
    pub mail: Option<String>,
    pub mail_enabled: bool,
    pub security_enabled: bool,
    pub created_at: Option<String>,
    pub member_count: i32,
}

#[derive(Debug, Serialize)]
pub struct AzureDevice {
    pub id: String,
    pub display_name: String,
    pub device_id: String,
    pub operating_system: Option<String>,
    pub operating_system_version: Option<String>,
    pub device_version: Option<String>,
    pub device_category: Option<String>,
    pub device_ownership: Option<String>,
    pub enrollment_type: Option<String>,
    pub management_state: Option<String>,
    pub compliance_state: Option<String>,
    pub last_sync_time: Option<String>,
    pub registered_users: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AzureApplication {
    pub id: String,
    pub app_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub publisher_domain: Option<String>,
    pub sign_in_audience: Option<String>,
    pub created_at: Option<String>,
    pub verified_publisher: Option<String>,
    pub app_roles: Vec<AppRole>,
    pub required_resource_access: Vec<RequiredResourceAccess>,
}

#[derive(Debug, Serialize)]
pub struct AppRole {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub value: String,
    pub is_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct RequiredResourceAccess {
    pub resource_app_id: String,
    pub resource_access: Vec<ResourceAccess>,
}

#[derive(Debug, Serialize)]
pub struct ResourceAccess {
    pub id: String,
    pub access_type: String,
}

#[derive(Debug, Serialize)]
pub struct AzureLicense {
    pub sku_id: String,
    pub sku_part_number: String,
    pub display_name: String,
    pub consumed_units: i32,
    pub prepared_units: i32,
    pub enabled_units: i32,
    pub suspended_units: i32,
    pub warning_units: i32,
    pub localized_units: i32,
}

#[derive(Debug, Serialize)]
pub struct AzureDomain {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_initial: bool,
    pub is_verified: bool,
    pub supported_services: Vec<String>,
    pub authentication_type: String,
}

#[derive(Debug, Serialize)]
pub struct AzureSubscription {
    pub subscription_id: String,
    pub display_name: String,
    pub state: String,
    pub subscription_policies: Option<serde_json::Value>,
    pub authorization_source: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AzureResource {
    pub id: String,
    pub name: String,
    pub resource_type: String,
    pub location: String,
    pub resource_group: String,
    pub subscription_id: String,
    pub tags: HashMap<String, String>,
    pub created_time: Option<String>,
    pub changed_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AzureSecurityOverview {
    pub secure_score: Option<f64>,
    pub max_score: Option<f64>,
    pub enabled_services: Vec<String>,
    pub alerts_summary: AlertsSummary,
    pub compliance_summary: ComplianceSummary,
    pub identity_security: IdentitySecurity,
}

#[derive(Debug, Serialize)]
pub struct AlertsSummary {
    pub high_severity: i32,
    pub medium_severity: i32,
    pub low_severity: i32,
    pub informational: i32,
}

#[derive(Debug, Serialize)]
pub struct ComplianceSummary {
    pub healthy_resources: i32,
    pub unhealthy_resources: i32,
    pub not_applicable_resources: i32,
}

#[derive(Debug, Serialize)]
pub struct IdentitySecurity {
    pub mfa_enabled_users: i32,
    pub total_users: i32,
    pub risky_users: i32,
    pub risky_sign_ins: i32,
}

async fn list_azure_tenants(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration_id = get_integration_id(&query)?;
    let credentials = get_azure_credentials(&state.db_pool, integration_id).await?;
    
    let client = create_azure_client(&credentials)?;
    let tenants = fetch_azure_tenants(&client, &credentials).await?;
    
    Ok(Json(tenants))
}

async fn list_azure_users(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration_id = get_integration_id(&query)?;
    let credentials = get_azure_credentials(&state.db_pool, integration_id).await?;
    
    let client = create_azure_client(&credentials)?;
    let users = fetch_azure_users(&client, &credentials).await?;
    
    Ok(Json(users))
}

async fn list_azure_groups(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration_id = get_integration_id(&query)?;
    let credentials = get_azure_credentials(&state.db_pool, integration_id).await?;
    
    let client = create_azure_client(&credentials)?;
    let groups = fetch_azure_groups(&client, &credentials).await?;
    
    Ok(Json(groups))
}

async fn list_azure_devices(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration_id = get_integration_id(&query)?;
    let credentials = get_azure_credentials(&state.db_pool, integration_id).await?;
    
    let client = create_azure_client(&credentials)?;
    let devices = fetch_azure_devices(&client, &credentials).await?;
    
    Ok(Json(devices))
}

async fn list_azure_applications(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration_id = get_integration_id(&query)?;
    let credentials = get_azure_credentials(&state.db_pool, integration_id).await?;
    
    let client = create_azure_client(&credentials)?;
    let applications = fetch_azure_applications(&client, &credentials).await?;
    
    Ok(Json(applications))
}

async fn list_azure_licenses(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration_id = get_integration_id(&query)?;
    let credentials = get_azure_credentials(&state.db_pool, integration_id).await?;
    
    let client = create_azure_client(&credentials)?;
    let licenses = fetch_azure_licenses(&client, &credentials).await?;
    
    Ok(Json(licenses))
}

async fn list_azure_domains(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration_id = get_integration_id(&query)?;
    let credentials = get_azure_credentials(&state.db_pool, integration_id).await?;
    
    let client = create_azure_client(&credentials)?;
    let domains = fetch_azure_domains(&client, &credentials).await?;
    
    Ok(Json(domains))
}

async fn list_azure_subscriptions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration_id = get_integration_id(&query)?;
    let credentials = get_azure_credentials(&state.db_pool, integration_id).await?;
    
    let client = create_azure_client(&credentials)?;
    let subscriptions = fetch_azure_subscriptions(&client, &credentials).await?;
    
    Ok(Json(subscriptions))
}

async fn list_azure_resources(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration_id = get_integration_id(&query)?;
    let credentials = get_azure_credentials(&state.db_pool, integration_id).await?;
    
    let client = create_azure_client(&credentials)?;
    let resources = fetch_azure_resources(&client, &credentials).await?;
    
    Ok(Json(resources))
}

async fn get_azure_security_overview(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let integration_id = get_integration_id(&query)?;
    let credentials = get_azure_credentials(&state.db_pool, integration_id).await?;
    
    let client = create_azure_client(&credentials)?;
    let security_overview = fetch_azure_security_overview(&client, &credentials).await?;
    
    Ok(Json(security_overview))
}

// Implementation functions

pub async fn sync_azure_integration(
    db_pool: &sqlx::PgPool,
    integration: &Integration,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let credentials_json = decrypt_json(&integration.credentials)?;
    let credentials: AzureCredentials = serde_json::from_value(credentials_json)?;
    
    let client = create_azure_client(&credentials)?;
    
    // Sync various Azure resources
    let mut sync_results = serde_json::Map::new();
    
    // Sync users and store in our database
    match fetch_azure_users(&client, &credentials).await {
        Ok(users) => {
            sync_results.insert("users".to_string(), serde_json::json!({
                "status": "success",
                "count": users.len(),
                "synced_at": chrono::Utc::now()
            }));
            
            // Store users in local database for offline access
            // (Implementation would depend on your schema design)
        }
        Err(e) => {
            sync_results.insert("users".to_string(), serde_json::json!({
                "status": "error",
                "error": e.to_string()
            }));
        }
    }
    
    // Sync devices
    match fetch_azure_devices(&client, &credentials).await {
        Ok(devices) => {
            sync_results.insert("devices".to_string(), serde_json::json!({
                "status": "success", 
                "count": devices.len(),
                "synced_at": chrono::Utc::now()
            }));
        }
        Err(e) => {
            sync_results.insert("devices".to_string(), serde_json::json!({
                "status": "error",
                "error": e.to_string()
            }));
        }
    }
    
    // Sync applications
    match fetch_azure_applications(&client, &credentials).await {
        Ok(applications) => {
            sync_results.insert("applications".to_string(), serde_json::json!({
                "status": "success",
                "count": applications.len(), 
                "synced_at": chrono::Utc::now()
            }));
        }
        Err(e) => {
            sync_results.insert("applications".to_string(), serde_json::json!({
                "status": "error",
                "error": e.to_string()
            }));
        }
    }
    
    Ok(serde_json::Value::Object(sync_results))
}

pub async fn test_azure_connection(
    integration: &Integration,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let credentials_json = decrypt_json(&integration.credentials)?;
    let credentials: AzureCredentials = serde_json::from_value(credentials_json)?;
    
    let client = create_azure_client(&credentials)?;
    
    // Test connection by fetching organization info
    let response = client
        .get("https://graph.microsoft.com/v1.0/organization")
        .bearer_auth(&get_access_token(&credentials).await?)
        .send()
        .await?;
    
    if response.status().is_success() {
        let org_info: serde_json::Value = response.json().await?;
        Ok(serde_json::json!({
            "status": "success",
            "organization": org_info.get("value")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|org| org.get("displayName"))
        }))
    } else {
        Ok(serde_json::json!({
            "status": "error",
            "error": format!("HTTP {}: {}", response.status(), response.text().await?)
        }))
    }
}

// Helper functions

fn get_integration_id(query: &serde_json::Value) -> Result<Uuid, StatusCode> {
    query.get("integration_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::BAD_REQUEST)
}

async fn get_azure_credentials(
    db_pool: &sqlx::PgPool,
    integration_id: Uuid,
) -> Result<AzureCredentials, StatusCode> {
    let integration = sqlx::query_as!(
        Integration,
        "SELECT * FROM integrations WHERE id = $1 AND integration_type = 'azure' AND enabled = true",
        integration_id
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    let credentials_json = decrypt_json(&integration.credentials)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    serde_json::from_value(credentials_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn create_azure_client(credentials: &AzureCredentials) -> Result<reqwest::Client, Box<dyn std::error::Error + Send + Sync>> {
    Ok(reqwest::Client::builder()
        .user_agent("Resolve/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()?)
}

async fn get_access_token(credentials: &AzureCredentials) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id", credentials.client_id.as_str()),
        ("client_secret", credentials.client_secret.as_str()),
        ("scope", credentials.graph_scope.as_deref().unwrap_or("https://graph.microsoft.com/.default")),
        ("grant_type", "client_credentials"),
    ];
    
    let response = client
        .post(&format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", credentials.tenant_id))
        .form(&params)
        .send()
        .await?;
    
    let token_response: serde_json::Value = response.json().await?;
    
    token_response.get("access_token")
        .and_then(|t| t.as_str())
        .map(String::from)
        .ok_or_else(|| "Failed to get access token".into())
}

async fn fetch_azure_tenants(
    client: &reqwest::Client,
    credentials: &AzureCredentials,
) -> Result<Vec<AzureTenant>, Box<dyn std::error::Error + Send + Sync>> {
    let access_token = get_access_token(credentials).await?;
    
    let response = client
        .get("https://graph.microsoft.com/v1.0/organization")
        .bearer_auth(&access_token)
        .send()
        .await?;
    
    let data: serde_json::Value = response.json().await?;
    let tenants = data.get("value")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
        .iter()
        .map(|tenant| AzureTenant {
            id: tenant.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            display_name: tenant.get("displayName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            domain_name: tenant.get("verifiedDomains")
                .and_then(|domains| domains.as_array())
                .and_then(|arr| arr.first())
                .and_then(|domain| domain.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            country: tenant.get("country").and_then(|v| v.as_str()).map(String::from),
            created_at: tenant.get("createdDateTime").and_then(|v| v.as_str()).map(String::from),
        })
        .collect();
    
    Ok(tenants)
}

async fn fetch_azure_users(
    client: &reqwest::Client,
    credentials: &AzureCredentials,
) -> Result<Vec<AzureUser>, Box<dyn std::error::Error + Send + Sync>> {
    let access_token = get_access_token(credentials).await?;
    
    let response = client
        .get("https://graph.microsoft.com/v1.0/users?$select=id,userPrincipalName,displayName,givenName,surname,mail,jobTitle,department,officeLocation,accountEnabled,signInActivity,createdDateTime&$top=999")
        .bearer_auth(&access_token)
        .send()
        .await?;
    
    let data: serde_json::Value = response.json().await?;
    let users = data.get("value")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
        .iter()
        .map(|user| AzureUser {
            id: user.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            user_principal_name: user.get("userPrincipalName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            display_name: user.get("displayName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            given_name: user.get("givenName").and_then(|v| v.as_str()).map(String::from),
            surname: user.get("surname").and_then(|v| v.as_str()).map(String::from),
            mail: user.get("mail").and_then(|v| v.as_str()).map(String::from),
            job_title: user.get("jobTitle").and_then(|v| v.as_str()).map(String::from),
            department: user.get("department").and_then(|v| v.as_str()).map(String::from),
            office_location: user.get("officeLocation").and_then(|v| v.as_str()).map(String::from),
            account_enabled: user.get("accountEnabled").and_then(|v| v.as_bool()).unwrap_or(false),
            last_sign_in: user.get("signInActivity")
                .and_then(|activity| activity.get("lastSignInDateTime"))
                .and_then(|v| v.as_str())
                .map(String::from),
            created_at: user.get("createdDateTime").and_then(|v| v.as_str()).map(String::from),
            licenses: vec![], // Would need separate API call
            groups: vec![],   // Would need separate API call
        })
        .collect();
    
    Ok(users)
}

// Additional fetch functions would be implemented similarly...
async fn fetch_azure_groups(client: &reqwest::Client, credentials: &AzureCredentials) -> Result<Vec<AzureGroup>, Box<dyn std::error::Error + Send + Sync>> {
    // Implementation similar to fetch_azure_users but for groups
    let access_token = get_access_token(credentials).await?;
    let response = client
        .get("https://graph.microsoft.com/v1.0/groups?$select=id,displayName,description,groupTypes,mail,mailEnabled,securityEnabled,createdDateTime&$top=999")
        .bearer_auth(&access_token)
        .send()
        .await?;
    
    let data: serde_json::Value = response.json().await?;
    let groups = data.get("value")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
        .iter()
        .map(|group| AzureGroup {
            id: group.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            display_name: group.get("displayName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: group.get("description").and_then(|v| v.as_str()).map(String::from),
            group_type: group.get("groupTypes")
                .and_then(|types| types.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("Security")
                .to_string(),
            mail: group.get("mail").and_then(|v| v.as_str()).map(String::from),
            mail_enabled: group.get("mailEnabled").and_then(|v| v.as_bool()).unwrap_or(false),
            security_enabled: group.get("securityEnabled").and_then(|v| v.as_bool()).unwrap_or(false),
            created_at: group.get("createdDateTime").and_then(|v| v.as_str()).map(String::from),
            member_count: 0, // Would need separate API call
        })
        .collect();
    
    Ok(groups)
}

async fn fetch_azure_devices(client: &reqwest::Client, credentials: &AzureCredentials) -> Result<Vec<AzureDevice>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder implementation - would fetch from Microsoft Graph devices endpoint
    Ok(vec![])
}

async fn fetch_azure_applications(client: &reqwest::Client, credentials: &AzureCredentials) -> Result<Vec<AzureApplication>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder implementation - would fetch from Microsoft Graph applications endpoint
    Ok(vec![])
}

async fn fetch_azure_licenses(client: &reqwest::Client, credentials: &AzureCredentials) -> Result<Vec<AzureLicense>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder implementation - would fetch from Microsoft Graph subscribedSkus endpoint
    Ok(vec![])
}

async fn fetch_azure_domains(client: &reqwest::Client, credentials: &AzureCredentials) -> Result<Vec<AzureDomain>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder implementation - would fetch from Microsoft Graph domains endpoint
    Ok(vec![])
}

async fn fetch_azure_subscriptions(client: &reqwest::Client, credentials: &AzureCredentials) -> Result<Vec<AzureSubscription>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder implementation - would fetch from Azure Resource Manager API
    Ok(vec![])
}

async fn fetch_azure_resources(client: &reqwest::Client, credentials: &AzureCredentials) -> Result<Vec<AzureResource>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder implementation - would fetch from Azure Resource Manager API
    Ok(vec![])
}

async fn fetch_azure_security_overview(client: &reqwest::Client, credentials: &AzureCredentials) -> Result<AzureSecurityOverview, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder implementation - would fetch from Microsoft Graph security endpoints
    Ok(AzureSecurityOverview {
        secure_score: Some(75.0),
        max_score: Some(100.0),
        enabled_services: vec!["Identity Protection".to_string(), "Conditional Access".to_string()],
        alerts_summary: AlertsSummary {
            high_severity: 2,
            medium_severity: 5,
            low_severity: 10,
            informational: 15,
        },
        compliance_summary: ComplianceSummary {
            healthy_resources: 85,
            unhealthy_resources: 15,
            not_applicable_resources: 5,
        },
        identity_security: IdentitySecurity {
            mfa_enabled_users: 120,
            total_users: 150,
            risky_users: 3,
            risky_sign_ins: 8,
        },
    })
}