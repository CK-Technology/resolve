use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct ClientCreate {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub billing_address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ClientUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub billing_address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ClientQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

pub fn client_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_clients).post(create_client))
        .route("/:id", get(get_client).put(update_client).delete(delete_client))
        .route("/:id/contacts", get(get_client_contacts))
        .route("/:id/assets", get(get_client_assets))
        .route("/:id/tickets", get(get_client_tickets))
}

async fn list_clients(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ClientQuery>,
) -> Result<Json<Vec<resolve_shared::Client>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let query = if let Some(search) = params.search {
        sqlx::query_as!(
            resolve_shared::Client,
            "SELECT id, name, email, phone, address, city, state, zip, billing_address, notes, 
             created_at, updated_at, archived_at 
             FROM clients 
             WHERE name ILIKE $1 OR email ILIKE $1
             ORDER BY name 
             LIMIT $2 OFFSET $3",
            format!("%{}%", search),
            limit,
            offset
        )
    } else {
        sqlx::query_as!(
            resolve_shared::Client,
            "SELECT id, name, email, phone, address, city, state, zip, billing_address, notes, 
             created_at, updated_at, archived_at 
             FROM clients 
             ORDER BY name 
             LIMIT $1 OFFSET $2",
            limit,
            offset
        )
    };

    match query.fetch_all(&state.db_pool).await {
        Ok(clients) => Ok(Json(clients)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_client(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ClientCreate>,
) -> Result<(StatusCode, Json<resolve_shared::Client>), StatusCode> {
    let client_id = Uuid::new_v4();
    
    match sqlx::query_as!(
        resolve_shared::Client,
        "INSERT INTO clients (id, name, email, phone, address, city, state, zip, billing_address, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         RETURNING id, name, email, phone, address, city, state, zip, billing_address, notes, 
                   created_at, updated_at, archived_at",
        client_id,
        payload.name,
        payload.email,
        payload.phone,
        payload.address,
        payload.city,
        payload.state,
        payload.zip,
        payload.billing_address,
        payload.notes
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(client) => Ok((StatusCode::CREATED, Json(client))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_client(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<resolve_shared::Client>, StatusCode> {
    match sqlx::query_as!(
        resolve_shared::Client,
        "SELECT id, name, email, phone, address, city, state, zip, billing_address, notes, 
         created_at, updated_at, archived_at 
         FROM clients WHERE id = $1",
        id
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(client) => Ok(Json(client)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_client(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ClientUpdate>,
) -> Result<Json<resolve_shared::Client>, StatusCode> {
    // This is a simplified update - in production you'd want to build dynamic SQL
    match sqlx::query_as!(
        resolve_shared::Client,
        "UPDATE clients SET 
         name = COALESCE($2, name),
         email = COALESCE($3, email),
         phone = COALESCE($4, phone),
         address = COALESCE($5, address),
         city = COALESCE($6, city),
         state = COALESCE($7, state),
         zip = COALESCE($8, zip),
         billing_address = COALESCE($9, billing_address),
         notes = COALESCE($10, notes),
         updated_at = NOW()
         WHERE id = $1
         RETURNING id, name, email, phone, address, city, state, zip, billing_address, notes, 
                   created_at, updated_at, archived_at",
        id,
        payload.name,
        payload.email,
        payload.phone,
        payload.address,
        payload.city,
        payload.state,
        payload.zip,
        payload.billing_address,
        payload.notes
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(client) => Ok(Json(client)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_client(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match sqlx::query!("DELETE FROM clients WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_client_contacts(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<resolve_shared::Contact>>, StatusCode> {
    match sqlx::query_as!(
        resolve_shared::Contact,
        "SELECT id, client_id, name, title, email, phone, extension, mobile, department, notes, 
         is_primary as primary, created_at, updated_at, archived_at 
         FROM contacts WHERE client_id = $1 ORDER BY is_primary DESC, name",
        id
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(contacts) => Ok(Json(contacts)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_client_assets(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<resolve_shared::Asset>>, StatusCode> {
    match sqlx::query_as!(
        resolve_shared::Asset,
        "SELECT id, client_id, name, description, asset_type, make, model, serial, os, 
         ip::TEXT as ip, mac::TEXT as mac, uri, status, location_id, contact_id, 
         purchase_date, warranty_expire, install_date, notes, created_at, updated_at, archived_at
         FROM assets WHERE client_id = $1 ORDER BY name",
        id
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(assets) => Ok(Json(assets)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_client_tickets(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<resolve_shared::Ticket>>, StatusCode> {
    match sqlx::query_as!(
        resolve_shared::Ticket,
        "SELECT id, client_id, contact_id, asset_id, number, subject, details, status, priority, 
         assigned_to, billable, opened_by, created_at, updated_at, closed_at
         FROM tickets WHERE client_id = $1 ORDER BY created_at DESC",
        id
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(tickets) => Ok(Json(tickets)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}