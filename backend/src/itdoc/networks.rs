use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::AppState;
use resolve_shared::Network;

pub fn network_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_networks).post(create_network))
        .route("/:id", get(get_network).put(update_network).delete(delete_network))
}

#[derive(Debug, Deserialize)]
pub struct CreateNetworkRequest {
    pub client_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub network_type: String,
    pub ip_range: String,
    pub subnet_mask: String,
    pub gateway: Option<String>,
    pub dns_servers: Option<Vec<String>>,
    pub vlan_id: Option<i32>,
    pub location_id: Option<Uuid>,
}

async fn list_networks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<serde_json::Value>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let client_id = query.get("client_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok());

    let networks = if let Some(client_id) = client_id {
        sqlx::query_as!(
            Network,
            r#"
            SELECT id, client_id, name, description, network_type, ip_range,
                   subnet_mask, gateway, dns_servers, vlan_id, location_id,
                   created_at, updated_at
            FROM networks WHERE client_id = $1 ORDER BY name ASC
            "#,
            client_id
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as!(
            Network,
            r#"
            SELECT id, client_id, name, description, network_type, ip_range,
                   subnet_mask, gateway, dns_servers, vlan_id, location_id,
                   created_at, updated_at
            FROM networks ORDER BY name ASC
            "#
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Json(networks))
}

async fn get_network(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let network = sqlx::query_as!(
        Network,
        r#"
        SELECT id, client_id, name, description, network_type, ip_range,
               subnet_mask, gateway, dns_servers, vlan_id, location_id,
               created_at, updated_at
        FROM networks WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(network))
}

async fn create_network(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateNetworkRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = Uuid::new_v4();
    let dns_servers = req.dns_servers.unwrap_or_default();

    sqlx::query!(
        r#"
        INSERT INTO networks (
            id, client_id, name, description, network_type, ip_range,
            subnet_mask, gateway, dns_servers, vlan_id, location_id,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NULL)
        "#,
        id, req.client_id, req.name, req.description, req.network_type,
        req.ip_range, req.subnet_mask, req.gateway, &dns_servers,
        req.vlan_id, req.location_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    log_audit_action(&state.db_pool, auth.0.id, "CREATE", "network", id).await;

    Ok(Json(serde_json::json!({ "id": id, "message": "Network created successfully" })))
}

async fn update_network(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(req): Json<CreateNetworkRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let dns_servers = req.dns_servers.unwrap_or_default();

    let result = sqlx::query!(
        r#"
        UPDATE networks SET
            client_id = $2, name = $3, description = $4, network_type = $5,
            ip_range = $6, subnet_mask = $7, gateway = $8, dns_servers = $9,
            vlan_id = $10, location_id = $11, updated_at = NOW()
        WHERE id = $1
        "#,
        id, req.client_id, req.name, req.description, req.network_type,
        req.ip_range, req.subnet_mask, req.gateway, &dns_servers,
        req.vlan_id, req.location_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    log_audit_action(&state.db_pool, auth.0.id, "UPDATE", "network", id).await;

    Ok(Json(serde_json::json!({ "message": "Network updated successfully" })))
}

async fn delete_network(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!("DELETE FROM networks WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    log_audit_action(&state.db_pool, auth.0.id, "DELETE", "network", id).await;

    Ok(Json(serde_json::json!({ "message": "Network deleted successfully" })))
}

async fn log_audit_action(
    db_pool: &sqlx::PgPool,
    user_id: Uuid,
    action: &str,
    entity_type: &str,
    entity_id: Uuid,
) {
    let _ = sqlx::query!(
        r#"
        INSERT INTO audit_logs (user_id, action, entity_type, entity_id, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        user_id, action, entity_type, entity_id
    )
    .execute(db_pool)
    .await;
}