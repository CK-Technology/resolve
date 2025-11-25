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

pub fn stripe_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/customers", get(list_stripe_customers))
        .route("/subscriptions", get(list_stripe_subscriptions))
        .route("/invoices", get(list_stripe_invoices))
        .route("/payments", get(list_stripe_payments))
        .route("/products", get(list_stripe_products))
        .route("/balance", get(get_stripe_balance))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StripeCredentials {
    pub secret_key: String,
    pub publishable_key: String,
    pub webhook_endpoint_secret: Option<String>,
}

async fn list_stripe_customers(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

async fn list_stripe_subscriptions(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

async fn list_stripe_invoices(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

async fn list_stripe_payments(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

async fn list_stripe_products(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!([])))
}

async fn get_stripe_balance(_state: State<Arc<AppState>>, _query: Query<serde_json::Value>, _auth: AuthUser) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(serde_json::json!({})))
}

pub async fn sync_stripe_integration(
    _db_pool: &sqlx::PgPool,
    _integration: &Integration,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::json!({"status": "placeholder"}))
}

pub async fn test_stripe_connection(
    _integration: &Integration,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::json!({"status": "placeholder"}))
}