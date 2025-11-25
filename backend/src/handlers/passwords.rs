use crate::auth::jwt::Claims;
use crate::models::passwords::*;
use crate::services::{PasswordManagerService, EncryptionService};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

pub fn password_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_passwords).post(create_password))
        .route("/generate", post(generate_password))
        .route("/:id", get(get_password).delete(delete_password))
        .route("/:id/favorite", put(update_password_favorite))
        .route("/folders", post(create_folder))
        .route("/shares", get(list_password_shares).post(create_password_share))
        .route("/shares/:id/deactivate", put(deactivate_password_share))
        .route("/shared", post(access_shared_password))
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PasswordQuery {
    pub client_id: Option<Uuid>,
    pub folder_id: Option<Uuid>,
    pub search: Option<String>,
    pub category: Option<String>,
    pub favorite: Option<bool>,
}

pub async fn create_password(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreatePasswordRequest>,
) -> Result<Json<ApiResponse<Uuid>>, StatusCode> {
    let pool = &state.db;
    
    let encryption_service = match EncryptionService::new() {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            return Ok(Json(ApiResponse::error("Internal server error")));
        }
    };

    let password_manager = PasswordManagerService::new(pool.clone(), encryption_service);

    match password_manager.create_password(request, claims.sub).await {
        Ok(password_id) => {
            info!("Password created successfully: {}", password_id);
            Ok(Json(ApiResponse::success(password_id)))
        }
        Err(e) => {
            error!("Failed to create password: {}", e);
            Ok(Json(ApiResponse::error("Failed to create password")))
        }
    }
}

pub async fn get_password(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(password_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PasswordResponse>>, StatusCode> {
    let pool = &state.db;
    
    let encryption_service = match EncryptionService::new() {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            return Ok(Json(ApiResponse::error("Internal server error")));
        }
    };

    let password_manager = PasswordManagerService::new(pool.clone(), encryption_service);

    match password_manager.get_password(password_id, claims.sub).await {
        Ok(Some(password)) => Ok(Json(ApiResponse::success(password))),
        Ok(None) => Ok(Json(ApiResponse::error("Password not found"))),
        Err(e) => {
            error!("Failed to get password {}: {}", password_id, e);
            Ok(Json(ApiResponse::error("Failed to retrieve password")))
        }
    }
}

pub async fn list_passwords(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Query(params): Query<PasswordQuery>,
) -> Result<Json<ApiResponse<PasswordListResponse>>, StatusCode> {
    let pool = &state.db;
    
    let encryption_service = match EncryptionService::new() {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            return Ok(Json(ApiResponse::error("Internal server error")));
        }
    };

    let password_manager = PasswordManagerService::new(pool.clone(), encryption_service);

    match password_manager.list_passwords(params.client_id, params.folder_id).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            error!("Failed to list passwords: {}", e);
            Ok(Json(ApiResponse::error("Failed to retrieve passwords")))
        }
    }
}

pub async fn generate_password(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<GeneratePasswordRequest>,
) -> Result<Json<ApiResponse<GeneratePasswordResponse>>, StatusCode> {
    let pool = &state.db;
    
    let encryption_service = match EncryptionService::new() {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            return Ok(Json(ApiResponse::error("Internal server error")));
        }
    };

    let password_manager = PasswordManagerService::new(pool.clone(), encryption_service);

    match password_manager.generate_password(request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            error!("Failed to generate password: {}", e);
            Ok(Json(ApiResponse::error("Failed to generate password")))
        }
    }
}

pub async fn create_password_share(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreatePasswordShareRequest>,
) -> Result<Json<ApiResponse<PasswordShareResponse>>, StatusCode> {
    let pool = &state.db;
    
    let encryption_service = match EncryptionService::new() {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            return Ok(Json(ApiResponse::error("Internal server error")));
        }
    };

    let password_manager = PasswordManagerService::new(pool.clone(), encryption_service);
    let base_url = std::env::var("APP_BASE_URL").unwrap_or_else(|_| "https://resolve.local".to_string());

    match password_manager.create_password_share(request, claims.sub, &base_url).await {
        Ok(share) => {
            info!("Password share created: {}", share.id);
            Ok(Json(ApiResponse::success(share)))
        }
        Err(e) => {
            error!("Failed to create password share: {}", e);
            Ok(Json(ApiResponse::error("Failed to create password share")))
        }
    }
}

pub async fn access_shared_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AccessPasswordShareRequest>,
) -> Result<Json<ApiResponse<PasswordShareAccessResponse>>, StatusCode> {
    let pool = &state.db;
    
    let encryption_service = match EncryptionService::new() {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            return Ok(Json(ApiResponse::error("Internal server error")));
        }
    };

    let password_manager = PasswordManagerService::new(pool.clone(), encryption_service);

    match password_manager.access_shared_password(request).await {
        Ok(Some(response)) => Ok(Json(ApiResponse::success(response))),
        Ok(None) => Ok(Json(ApiResponse::error("Share not found or expired"))),
        Err(e) => {
            error!("Failed to access shared password: {}", e);
            Ok(Json(ApiResponse::error("Failed to access shared password")))
        }
    }
}

pub async fn list_password_shares(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<PasswordShareResponse>>>, StatusCode> {
    let pool = &state.db;
    
    let encryption_service = match EncryptionService::new() {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            return Ok(Json(ApiResponse::error("Internal server error")));
        }
    };

    let password_manager = PasswordManagerService::new(pool.clone(), encryption_service);
    
    let password_id = params.get("password_id")
        .and_then(|id| Uuid::parse_str(id).ok());
    
    let created_by = Some(claims.sub);

    match password_manager.list_password_shares(password_id, created_by).await {
        Ok(shares) => Ok(Json(ApiResponse::success(shares))),
        Err(e) => {
            error!("Failed to list password shares: {}", e);
            Ok(Json(ApiResponse::error("Failed to retrieve password shares")))
        }
    }
}

pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateFolderRequest>,
) -> Result<Json<ApiResponse<Uuid>>, StatusCode> {
    let pool = &state.db;
    
    let encryption_service = match EncryptionService::new() {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            return Ok(Json(ApiResponse::error("Internal server error")));
        }
    };

    let password_manager = PasswordManagerService::new(pool.clone(), encryption_service);

    match password_manager.create_folder(request, claims.sub).await {
        Ok(folder_id) => {
            info!("Password folder created successfully: {}", folder_id);
            Ok(Json(ApiResponse::success(folder_id)))
        }
        Err(e) => {
            error!("Failed to create folder: {}", e);
            Ok(Json(ApiResponse::error("Failed to create folder")))
        }
    }
}

pub async fn delete_password(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(password_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let pool = &state.db;
    
    match sqlx::query!(
        "DELETE FROM passwords WHERE id = $1 AND created_by = $2",
        password_id,
        claims.sub
    )
    .execute(pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                info!("Password deleted successfully: {}", password_id);
                Ok(Json(ApiResponse::success(())))
            } else {
                Ok(Json(ApiResponse::error("Password not found or access denied")))
            }
        }
        Err(e) => {
            error!("Failed to delete password {}: {}", password_id, e);
            Ok(Json(ApiResponse::error("Failed to delete password")))
        }
    }
}

pub async fn update_password_favorite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(password_id): Path<Uuid>,
    Json(favorite): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let pool = &state.db;
    
    let favorite = favorite.get("favorite")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    match sqlx::query!(
        "UPDATE passwords SET favorite = $1, updated_at = NOW() WHERE id = $2 AND created_by = $3",
        favorite,
        password_id,
        claims.sub
    )
    .execute(pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(Json(ApiResponse::success(())))
            } else {
                Ok(Json(ApiResponse::error("Password not found or access denied")))
            }
        }
        Err(e) => {
            error!("Failed to update password favorite {}: {}", password_id, e);
            Ok(Json(ApiResponse::error("Failed to update password")))
        }
    }
}

pub async fn deactivate_password_share(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(share_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let pool = &state.db;
    
    match sqlx::query!(
        "UPDATE password_shares SET is_active = false WHERE id = $1 AND created_by = $2",
        share_id,
        claims.sub
    )
    .execute(pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                info!("Password share deactivated: {}", share_id);
                Ok(Json(ApiResponse::success(())))
            } else {
                Ok(Json(ApiResponse::error("Share not found or access denied")))
            }
        }
        Err(e) => {
            error!("Failed to deactivate password share {}: {}", share_id, e);
            Ok(Json(ApiResponse::error("Failed to deactivate share")))
        }
    }
}