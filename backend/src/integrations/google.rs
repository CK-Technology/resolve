use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::AppState;
use resolve_shared::Integration;
use super::decrypt_json;

pub fn google_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/workspace/users", get(list_google_workspace_users))
        .route("/workspace/groups", get(list_google_workspace_groups))
        .route("/workspace/domains", get(list_google_workspace_domains))
        .route("/cloud/projects", get(list_google_cloud_projects))
        .route("/cloud/resources", get(list_google_cloud_resources))
        .route("/drive/files", get(list_google_drive_files))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleCredentials {
    pub service_account_key: String,
    pub project_id: String,
    pub customer_id: Option<String>,
    pub domain: Option<String>,
}

async fn list_google_workspace_users(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

async fn list_google_workspace_groups(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

async fn list_google_workspace_domains(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

async fn list_google_cloud_projects(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

async fn list_google_cloud_resources(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

async fn list_google_drive_files(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

pub async fn sync_google_integration(
    _db_pool: &sqlx::PgPool,
    _integration: &Integration,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::json!({"status": "placeholder"}))
}

pub async fn test_google_connection(
    _integration: &Integration,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::json!({"status": "placeholder"}))
}