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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AssetFieldType {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub validation_rules: Option<serde_json::Value>,
    pub ui_component: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AssetLayout {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_system_layout: bool,
    pub is_active: bool,
    pub display_order: i32,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AssetLayoutField {
    pub id: Uuid,
    pub layout_id: Uuid,
    pub field_type_id: Uuid,
    pub field_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_required: bool,
    pub is_searchable: bool,
    pub is_shown_in_list: bool,
    pub display_order: i32,
    pub default_value: Option<String>,
    pub placeholder: Option<String>,
    pub validation_rules: Option<serde_json::Value>,
    pub field_options: Option<serde_json::Value>,
    pub help_text: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetLayoutWithFields {
    #[serde(flatten)]
    pub layout: AssetLayout,
    pub fields: Vec<AssetLayoutFieldWithType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetLayoutFieldWithType {
    #[serde(flatten)]
    pub field: AssetLayoutField,
    pub field_type: AssetFieldType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAssetLayoutRequest {
    pub name: String,
    pub asset_type: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_active: Option<bool>,
    pub display_order: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLayoutFieldRequest {
    pub field_type_id: Uuid,
    pub field_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_required: Option<bool>,
    pub is_searchable: Option<bool>,
    pub is_shown_in_list: Option<bool>,
    pub display_order: Option<i32>,
    pub default_value: Option<String>,
    pub placeholder: Option<String>,
    pub validation_rules: Option<serde_json::Value>,
    pub field_options: Option<serde_json::Value>,
    pub help_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AssetFieldValue {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub field_id: Uuid,
    pub field_value: Option<String>,
    pub field_value_encrypted: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetWithCustomFields {
    pub asset_id: Uuid,
    pub layout: AssetLayout,
    pub field_values: Vec<AssetFieldValueWithField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetFieldValueWithField {
    #[serde(flatten)]
    pub value: AssetFieldValue,
    pub field: AssetLayoutField,
    pub field_type: AssetFieldType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAssetFieldValuesRequest {
    pub field_values: std::collections::HashMap<Uuid, String>, // field_id -> value
}

pub fn asset_layout_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/field-types", get(list_field_types))
        .route("/layouts", get(list_layouts).post(create_layout))
        .route("/layouts/:id", get(get_layout).put(update_layout).delete(delete_layout))
        .route("/layouts/:id/fields", get(list_layout_fields).post(create_layout_field))
        .route("/layouts/:layout_id/fields/:field_id", put(update_layout_field).delete(delete_layout_field))
        .route("/assets/:asset_id/custom-fields", get(get_asset_custom_fields).put(update_asset_custom_fields))
}

async fn list_field_types(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AssetFieldType>>, StatusCode> {
    let field_types = sqlx::query_as::<_, AssetFieldType>(
        "SELECT id, name, display_name, description, validation_rules, ui_component 
         FROM asset_field_types 
         ORDER BY display_name"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching asset field types: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(field_types))
}

async fn list_layouts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AssetLayout>>, StatusCode> {
    let mut query = "SELECT * FROM asset_layouts WHERE is_active = true".to_string();
    
    if let Some(asset_type) = params.get("asset_type") {
        query.push_str(&format!(" AND asset_type = '{}'", asset_type));
    }
    
    query.push_str(" ORDER BY display_order, name");
    
    let layouts = sqlx::query_as::<_, AssetLayout>(&query)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching asset layouts: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(layouts))
}

async fn get_layout(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetLayoutWithFields>, StatusCode> {
    let layout = sqlx::query_as::<_, AssetLayout>(
        "SELECT * FROM asset_layouts WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error fetching asset layout: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    
    let fields = sqlx::query_as::<_, (AssetLayoutField, AssetFieldType)>(
        "SELECT 
            f.id, f.layout_id, f.field_type_id, f.field_name, f.display_name, f.description,
            f.is_required, f.is_searchable, f.is_shown_in_list, f.display_order,
            f.default_value, f.placeholder, f.validation_rules, f.field_options, f.help_text, f.created_at,
            t.id, t.name, t.display_name, t.description, t.validation_rules, t.ui_component
         FROM asset_layout_fields f
         JOIN asset_field_types t ON f.field_type_id = t.id
         WHERE f.layout_id = $1
         ORDER BY f.display_order, f.display_name"
    )
    .bind(id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching layout fields: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let fields_with_types: Vec<AssetLayoutFieldWithType> = fields
        .into_iter()
        .map(|(field, field_type)| AssetLayoutFieldWithType { field, field_type })
        .collect();
    
    Ok(Json(AssetLayoutWithFields {
        layout,
        fields: fields_with_types,
    }))
}

async fn create_layout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateAssetLayoutRequest>,
) -> Result<(StatusCode, Json<AssetLayout>), StatusCode> {
    let token = extract_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let token_data = verify_token(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let layout_id = Uuid::new_v4();
    let now = Utc::now();
    
    sqlx::query(
        "INSERT INTO asset_layouts 
         (id, name, asset_type, description, icon, color, is_active, display_order, created_by, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(layout_id)
    .bind(&payload.name)
    .bind(&payload.asset_type)
    .bind(&payload.description)
    .bind(&payload.icon)
    .bind(&payload.color)
    .bind(payload.is_active.unwrap_or(true))
    .bind(payload.display_order.unwrap_or(0))
    .bind(token_data.claims.sub.parse::<Uuid>().map_err(|_| StatusCode::UNAUTHORIZED)?)
    .bind(now)
    .bind(now)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating asset layout: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let layout = sqlx::query_as::<_, AssetLayout>("SELECT * FROM asset_layouts WHERE id = $1")
        .bind(layout_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching created layout: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok((StatusCode::CREATED, Json(layout)))
}

async fn update_layout(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateAssetLayoutRequest>,
) -> Result<Json<AssetLayout>, StatusCode> {
    sqlx::query(
        "UPDATE asset_layouts 
         SET name = $2, description = $3, icon = $4, color = $5, 
             is_active = $6, display_order = $7, updated_at = NOW()
         WHERE id = $1"
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.icon)
    .bind(&payload.color)
    .bind(payload.is_active.unwrap_or(true))
    .bind(payload.display_order.unwrap_or(0))
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating asset layout: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let layout = sqlx::query_as::<_, AssetLayout>("SELECT * FROM asset_layouts WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => {
                tracing::error!("Error fetching updated layout: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    Ok(Json(layout))
}

async fn delete_layout(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE asset_layouts SET is_active = false WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deactivating asset layout: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(StatusCode::NO_CONTENT)
}

async fn list_layout_fields(
    State(state): State<Arc<AppState>>,
    Path(layout_id): Path<Uuid>,
) -> Result<Json<Vec<AssetLayoutFieldWithType>>, StatusCode> {
    let fields = sqlx::query_as::<_, (AssetLayoutField, AssetFieldType)>(
        "SELECT 
            f.id, f.layout_id, f.field_type_id, f.field_name, f.display_name, f.description,
            f.is_required, f.is_searchable, f.is_shown_in_list, f.display_order,
            f.default_value, f.placeholder, f.validation_rules, f.field_options, f.help_text, f.created_at,
            t.id, t.name, t.display_name, t.description, t.validation_rules, t.ui_component
         FROM asset_layout_fields f
         JOIN asset_field_types t ON f.field_type_id = t.id
         WHERE f.layout_id = $1
         ORDER BY f.display_order, f.display_name"
    )
    .bind(layout_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching layout fields: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let fields_with_types: Vec<AssetLayoutFieldWithType> = fields
        .into_iter()
        .map(|(field, field_type)| AssetLayoutFieldWithType { field, field_type })
        .collect();
    
    Ok(Json(fields_with_types))
}

async fn create_layout_field(
    State(state): State<Arc<AppState>>,
    Path(layout_id): Path<Uuid>,
    Json(payload): Json<CreateLayoutFieldRequest>,
) -> Result<(StatusCode, Json<AssetLayoutField>), StatusCode> {
    let field_id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO asset_layout_fields 
         (id, layout_id, field_type_id, field_name, display_name, description, 
          is_required, is_searchable, is_shown_in_list, display_order, 
          default_value, placeholder, validation_rules, field_options, help_text) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"
    )
    .bind(field_id)
    .bind(layout_id)
    .bind(payload.field_type_id)
    .bind(&payload.field_name)
    .bind(&payload.display_name)
    .bind(&payload.description)
    .bind(payload.is_required.unwrap_or(false))
    .bind(payload.is_searchable.unwrap_or(true))
    .bind(payload.is_shown_in_list.unwrap_or(false))
    .bind(payload.display_order.unwrap_or(0))
    .bind(&payload.default_value)
    .bind(&payload.placeholder)
    .bind(&payload.validation_rules)
    .bind(&payload.field_options)
    .bind(&payload.help_text)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating layout field: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let field = sqlx::query_as::<_, AssetLayoutField>("SELECT * FROM asset_layout_fields WHERE id = $1")
        .bind(field_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching created field: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok((StatusCode::CREATED, Json(field)))
}

async fn update_layout_field(
    State(state): State<Arc<AppState>>,
    Path((layout_id, field_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateLayoutFieldRequest>,
) -> Result<Json<AssetLayoutField>, StatusCode> {
    sqlx::query(
        "UPDATE asset_layout_fields 
         SET field_type_id = $3, field_name = $4, display_name = $5, description = $6, 
             is_required = $7, is_searchable = $8, is_shown_in_list = $9, display_order = $10,
             default_value = $11, placeholder = $12, validation_rules = $13, 
             field_options = $14, help_text = $15
         WHERE id = $2 AND layout_id = $1"
    )
    .bind(layout_id)
    .bind(field_id)
    .bind(payload.field_type_id)
    .bind(&payload.field_name)
    .bind(&payload.display_name)
    .bind(&payload.description)
    .bind(payload.is_required.unwrap_or(false))
    .bind(payload.is_searchable.unwrap_or(true))
    .bind(payload.is_shown_in_list.unwrap_or(false))
    .bind(payload.display_order.unwrap_or(0))
    .bind(&payload.default_value)
    .bind(&payload.placeholder)
    .bind(&payload.validation_rules)
    .bind(&payload.field_options)
    .bind(&payload.help_text)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating layout field: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let field = sqlx::query_as::<_, AssetLayoutField>("SELECT * FROM asset_layout_fields WHERE id = $1")
        .bind(field_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => {
                tracing::error!("Error fetching updated field: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    Ok(Json(field))
}

async fn delete_layout_field(
    State(state): State<Arc<AppState>>,
    Path((layout_id, field_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM asset_layout_fields WHERE id = $2 AND layout_id = $1")
        .bind(layout_id)
        .bind(field_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting layout field: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(StatusCode::NO_CONTENT)
}

async fn get_asset_custom_fields(
    State(state): State<Arc<AppState>>,
    Path(asset_id): Path<Uuid>,
) -> Result<Json<AssetWithCustomFields>, StatusCode> {
    // First get the asset's type and corresponding layout
    let asset_type: (String,) = sqlx::query_as("SELECT asset_type FROM assets WHERE id = $1")
        .bind(asset_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => {
                tracing::error!("Error fetching asset: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    let layout = sqlx::query_as::<_, AssetLayout>(
        "SELECT * FROM asset_layouts WHERE asset_type = $1 AND is_active = true LIMIT 1"
    )
    .bind(&asset_type.0)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching asset layout: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;
    
    // Get field values with field definitions
    let field_values = sqlx::query_as::<_, (AssetFieldValue, AssetLayoutField, AssetFieldType)>(
        "SELECT 
            v.id, v.asset_id, v.field_id, v.field_value, v.field_value_encrypted, v.created_at, v.updated_at,
            f.id, f.layout_id, f.field_type_id, f.field_name, f.display_name, f.description,
            f.is_required, f.is_searchable, f.is_shown_in_list, f.display_order,
            f.default_value, f.placeholder, f.validation_rules, f.field_options, f.help_text, f.created_at,
            t.id, t.name, t.display_name, t.description, t.validation_rules, t.ui_component
         FROM asset_field_values v
         JOIN asset_layout_fields f ON v.field_id = f.id
         JOIN asset_field_types t ON f.field_type_id = t.id
         WHERE v.asset_id = $1
         ORDER BY f.display_order, f.display_name"
    )
    .bind(asset_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching asset field values: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let values_with_fields: Vec<AssetFieldValueWithField> = field_values
        .into_iter()
        .map(|(value, field, field_type)| AssetFieldValueWithField { value, field, field_type })
        .collect();
    
    Ok(Json(AssetWithCustomFields {
        asset_id,
        layout,
        field_values: values_with_fields,
    }))
}

async fn update_asset_custom_fields(
    State(state): State<Arc<AppState>>,
    Path(asset_id): Path<Uuid>,
    Json(payload): Json<UpdateAssetFieldValuesRequest>,
) -> Result<StatusCode, StatusCode> {
    // Update field values in a transaction
    let mut tx = state.db_pool.begin().await.map_err(|e| {
        tracing::error!("Error starting transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    for (field_id, value) in payload.field_values {
        sqlx::query(
            "INSERT INTO asset_field_values (asset_id, field_id, field_value, updated_at)
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (asset_id, field_id)
             DO UPDATE SET field_value = $3, updated_at = NOW()"
        )
        .bind(asset_id)
        .bind(field_id)
        .bind(&value)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Error updating field value: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }
    
    tx.commit().await.map_err(|e| {
        tracing::error!("Error committing transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(StatusCode::NO_CONTENT)
}