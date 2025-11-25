use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;
use crate::AppState;
use crate::auth::{extract_token, verify_token};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetCreate {
    pub client_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub asset_type: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub os: Option<String>,
    pub ip: Option<String>,
    pub mac: Option<String>,
    pub uri: Option<String>,
    pub status: String,
    pub location_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub purchase_date: Option<chrono::DateTime<Utc>>,
    pub warranty_expire: Option<chrono::DateTime<Utc>>,
    pub install_date: Option<chrono::DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub asset_type: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub os: Option<String>,
    pub ip: Option<String>,
    pub mac: Option<String>,
    pub uri: Option<String>,
    pub status: Option<String>,
    pub location_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub purchase_date: Option<chrono::DateTime<Utc>>,
    pub warranty_expire: Option<chrono::DateTime<Utc>>,
    pub install_date: Option<chrono::DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub client_id: Option<Uuid>,
    pub asset_type: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AssetWithDetails {
    pub id: Uuid,
    pub client_id: Uuid,
    pub client_name: String,
    pub name: String,
    pub description: Option<String>,
    pub asset_type: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub os: Option<String>,
    pub ip: Option<String>,
    pub mac: Option<String>,
    pub uri: Option<String>,
    pub status: String,
    pub location_id: Option<Uuid>,
    pub location_name: Option<String>,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub purchase_date: Option<chrono::DateTime<Utc>>,
    pub warranty_expire: Option<chrono::DateTime<Utc>>,
    pub install_date: Option<chrono::DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

pub fn asset_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_assets).post(create_asset))
        .route("/:id", get(get_asset).put(update_asset).delete(delete_asset))
        .route("/:id/monitoring", get(get_asset_monitoring))
        .route("/types", get(get_asset_types))
}

async fn list_assets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AssetQuery>,
) -> Result<Json<Vec<AssetWithDetails>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    // Use a simpler query for now
    let assets = sqlx::query_as::<_, AssetWithDetails>(&format!(
        "SELECT a.id, a.client_id, c.name as client_name, a.name, a.description, 
         a.asset_type, a.make, a.model, a.serial, a.os, a.ip, a.mac, a.uri, 
         a.status, a.location_id, l.name as location_name, a.contact_id, ct.name as contact_name,
         a.purchase_date, a.warranty_expire, a.install_date, a.notes, a.created_at, a.updated_at
         FROM assets a
         LEFT JOIN clients c ON a.client_id = c.id
         LEFT JOIN locations l ON a.location_id = l.id
         LEFT JOIN contacts ct ON a.contact_id = ct.id
         WHERE a.archived_at IS NULL
         ORDER BY a.name
         LIMIT {} OFFSET {}", limit, offset))
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching assets: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(assets))
}

async fn create_asset(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<AssetCreate>,
) -> Result<(StatusCode, Json<AssetWithDetails>), StatusCode> {
    // Extract user from token
    let token = extract_token(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let _token_data = verify_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let asset_id = Uuid::new_v4();
    let now = Utc::now();
    
    sqlx::query(
        "INSERT INTO assets (
            id, client_id, name, description, asset_type, make, model, serial,
            os, ip, mac, uri, status, location_id, contact_id, purchase_date,
            warranty_expire, install_date, notes, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)"
    )
    .bind(asset_id)
    .bind(payload.client_id)
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.asset_type)
    .bind(payload.make)
    .bind(payload.model)
    .bind(payload.serial)
    .bind(payload.os)
    .bind(payload.ip)
    .bind(payload.mac)
    .bind(payload.uri)
    .bind(payload.status)
    .bind(payload.location_id)
    .bind(payload.contact_id)
    .bind(payload.purchase_date)
    .bind(payload.warranty_expire)
    .bind(payload.install_date)
    .bind(payload.notes)
    .bind(now)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating asset: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Fetch the created asset
    let asset = get_asset_by_id(&state, asset_id).await?;
    Ok((StatusCode::CREATED, Json(asset)))
}

async fn get_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetWithDetails>, StatusCode> {
    let asset = get_asset_by_id(&state, id).await?;
    Ok(Json(asset))
}

async fn update_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AssetUpdate>,
) -> Result<Json<AssetWithDetails>, StatusCode> {
    // Build dynamic update query
    let mut set_clauses = Vec::new();
    let mut param_index = 2; // $1 is for id
    
    if payload.name.is_some() { set_clauses.push(format!("name = ${}", param_index)); param_index += 1; }
    if payload.description.is_some() { set_clauses.push(format!("description = ${}", param_index)); param_index += 1; }
    if payload.asset_type.is_some() { set_clauses.push(format!("asset_type = ${}", param_index)); param_index += 1; }
    if payload.make.is_some() { set_clauses.push(format!("make = ${}", param_index)); param_index += 1; }
    if payload.model.is_some() { set_clauses.push(format!("model = ${}", param_index)); param_index += 1; }
    if payload.serial.is_some() { set_clauses.push(format!("serial = ${}", param_index)); param_index += 1; }
    if payload.os.is_some() { set_clauses.push(format!("os = ${}", param_index)); param_index += 1; }
    if payload.ip.is_some() { set_clauses.push(format!("ip = ${}", param_index)); param_index += 1; }
    if payload.mac.is_some() { set_clauses.push(format!("mac = ${}", param_index)); param_index += 1; }
    if payload.uri.is_some() { set_clauses.push(format!("uri = ${}", param_index)); param_index += 1; }
    if payload.status.is_some() { set_clauses.push(format!("status = ${}", param_index)); param_index += 1; }
    if payload.notes.is_some() { set_clauses.push(format!("notes = ${}", param_index)); param_index += 1; }
    
    set_clauses.push(format!("updated_at = NOW()"));
    
    if set_clauses.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let query = format!(
        "UPDATE assets SET {} WHERE id = $1",
        set_clauses.join(", ")
    );
    
    // For simplicity, use a basic update query
    sqlx::query(
        "UPDATE assets SET 
         name = COALESCE($2, name),
         description = COALESCE($3, description),
         asset_type = COALESCE($4, asset_type),
         make = COALESCE($5, make),
         model = COALESCE($6, model),
         serial = COALESCE($7, serial),
         os = COALESCE($8, os),
         ip = COALESCE($9, ip),
         mac = COALESCE($10, mac),
         uri = COALESCE($11, uri),
         status = COALESCE($12, status),
         notes = COALESCE($13, notes),
         updated_at = NOW()
         WHERE id = $1"
    )
    .bind(id)
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.asset_type)
    .bind(payload.make)
    .bind(payload.model)
    .bind(payload.serial)
    .bind(payload.os)
    .bind(payload.ip)
    .bind(payload.mac)
    .bind(payload.uri)
    .bind(payload.status)
    .bind(payload.notes)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating asset: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let asset = get_asset_by_id(&state, id).await?;
    Ok(Json(asset))
}

async fn delete_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE assets SET archived_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error archiving asset: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(StatusCode::NO_CONTENT)
}

async fn get_asset_monitoring(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement asset monitoring data
    Ok(Json(serde_json::json!({
        "status": "up",
        "last_check": Utc::now(),
        "uptime": "99.9%"
    })))
}

async fn get_asset_types(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let types = sqlx::query_scalar::<_, String>("SELECT DISTINCT asset_type FROM assets ORDER BY asset_type")
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching asset types: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(types))
}

// Helper functions
async fn get_asset_by_id(state: &AppState, id: Uuid) -> Result<AssetWithDetails, StatusCode> {
    sqlx::query_as::<_, AssetWithDetails>(
        "SELECT a.id, a.client_id, c.name as client_name, a.name, a.description, 
         a.asset_type, a.make, a.model, a.serial, a.os, a.ip, a.mac, a.uri, 
         a.status, a.location_id, l.name as location_name, a.contact_id, ct.name as contact_name,
         a.purchase_date, a.warranty_expire, a.install_date, a.notes, a.created_at, a.updated_at
         FROM assets a
         LEFT JOIN clients c ON a.client_id = c.id
         LEFT JOIN locations l ON a.location_id = l.id
         LEFT JOIN contacts ct ON a.contact_id = ct.id
         WHERE a.id = $1"
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error fetching asset: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })
}