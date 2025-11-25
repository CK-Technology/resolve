use axum::{
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Json},
    routing::{get, post, delete},
    Router,
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::auth::middleware::AuthUser;
use crate::AppState;
use resolve_shared::File;

pub fn file_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_files))
        .route("/upload", post(upload_file))
        .route("/:id", get(get_file).delete(delete_file))
        .route("/:id/download", get(download_file))
}

#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    pub client_id: Option<Uuid>,
    pub ticket_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub kb_article_id: Option<Uuid>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct FileResponse {
    #[serde(flatten)]
    pub file: File,
    pub download_url: String,
    pub file_size_formatted: String,
}

async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListFilesQuery>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    // Build dynamic WHERE clause based on query parameters
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 0;

    if let Some(client_id) = query.client_id {
        param_count += 1;
        conditions.push(format!("client_id = ${}", param_count));
        params.push(Box::new(client_id));
    }

    if let Some(ticket_id) = query.ticket_id {
        param_count += 1;
        conditions.push(format!("ticket_id = ${}", param_count));
        params.push(Box::new(ticket_id));
    }

    if let Some(asset_id) = query.asset_id {
        param_count += 1;
        conditions.push(format!("asset_id = ${}", param_count));
        params.push(Box::new(asset_id));
    }

    if let Some(project_id) = query.project_id {
        param_count += 1;
        conditions.push(format!("project_id = ${}", param_count));
        params.push(Box::new(project_id));
    }

    if let Some(kb_article_id) = query.kb_article_id {
        param_count += 1;
        conditions.push(format!("kb_article_id = ${}", param_count));
        params.push(Box::new(kb_article_id));
    }

    // For simplicity, using basic queries. In production, use a proper query builder
    let files = if conditions.is_empty() {
        sqlx::query_as!(
            File,
            r#"
            SELECT id, client_id, ticket_id, asset_id, project_id, kb_article_id,
                   filename, original_filename, mime_type, file_size, file_path,
                   uploaded_by, created_at
            FROM files
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else if let Some(client_id) = query.client_id {
        sqlx::query_as!(
            File,
            r#"
            SELECT id, client_id, ticket_id, asset_id, project_id, kb_article_id,
                   filename, original_filename, mime_type, file_size, file_path,
                   uploaded_by, created_at
            FROM files
            WHERE client_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            client_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        // Handle other single-parameter queries
        sqlx::query_as!(
            File,
            r#"
            SELECT id, client_id, ticket_id, asset_id, project_id, kb_article_id,
                   filename, original_filename, mime_type, file_size, file_path,
                   uploaded_by, created_at
            FROM files
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    // Add download URLs and format file sizes
    let file_responses: Vec<FileResponse> = files.into_iter().map(|file| {
        FileResponse {
            download_url: format!("/api/v1/files/{}/download", file.id),
            file_size_formatted: format_file_size(file.file_size),
            file: file,
        }
    }).collect();

    Ok(Json(file_responses))
}

async fn get_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let file = sqlx::query_as!(
        File,
        r#"
        SELECT id, client_id, ticket_id, asset_id, project_id, kb_article_id,
               filename, original_filename, mime_type, file_size, file_path,
               uploaded_by, created_at
        FROM files
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let file_response = FileResponse {
        download_url: format!("/api/v1/files/{}/download", file.id),
        file_size_formatted: format_file_size(file.file_size),
        file,
    };

    Ok(Json(file_response))
}

async fn upload_file(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let mut file_data = Vec::new();
    let mut original_filename = String::new();
    let mut mime_type = "application/octet-stream".to_string();
    let mut client_id: Option<Uuid> = None;
    let mut ticket_id: Option<Uuid> = None;
    let mut asset_id: Option<Uuid> = None;
    let mut project_id: Option<Uuid> = None;
    let mut kb_article_id: Option<Uuid> = None;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "file" => {
                original_filename = field.file_name()
                    .unwrap_or("unknown")
                    .to_string();
                
                if let Some(content_type) = field.content_type() {
                    mime_type = content_type.to_string();
                }
                
                file_data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec();
            },
            "client_id" => {
                let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                client_id = Uuid::parse_str(&value).ok();
            },
            "ticket_id" => {
                let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                ticket_id = Uuid::parse_str(&value).ok();
            },
            "asset_id" => {
                let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                asset_id = Uuid::parse_str(&value).ok();
            },
            "project_id" => {
                let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                project_id = Uuid::parse_str(&value).ok();
            },
            "kb_article_id" => {
                let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                kb_article_id = Uuid::parse_str(&value).ok();
            },
            _ => {}
        }
    }

    if file_data.is_empty() || original_filename.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Generate unique filename and file path
    let file_id = Uuid::new_v4();
    let file_extension = std::path::Path::new(&original_filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    let filename = if file_extension.is_empty() {
        file_id.to_string()
    } else {
        format!("{}.{}", file_id, file_extension)
    };

    // Create upload directory if it doesn't exist
    let upload_dir = get_upload_directory();
    fs::create_dir_all(&upload_dir).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Write file to disk
    let file_path = format!("{}/{}", upload_dir, filename);
    let mut file = fs::File::create(&file_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.write_all(&file_data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save file metadata to database
    sqlx::query!(
        r#"
        INSERT INTO files (
            id, client_id, ticket_id, asset_id, project_id, kb_article_id,
            filename, original_filename, mime_type, file_size, file_path,
            uploaded_by, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
        "#,
        file_id,
        client_id,
        ticket_id,
        asset_id,
        project_id,
        kb_article_id,
        filename,
        original_filename,
        mime_type,
        file_data.len() as i64,
        file_path,
        auth.0.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log the upload
    log_audit_action(&state.db_pool, auth.0.id, "UPLOAD", "file", file_id).await;

    Ok(Json(serde_json::json!({
        "id": file_id,
        "filename": filename,
        "original_filename": original_filename,
        "file_size": file_data.len(),
        "message": "File uploaded successfully"
    })))
}

async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    let file = sqlx::query_as!(
        File,
        r#"
        SELECT id, client_id, ticket_id, asset_id, project_id, kb_article_id,
               filename, original_filename, mime_type, file_size, file_path,
               uploaded_by, created_at
        FROM files
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Check if file exists on disk
    if !tokio::fs::metadata(&file.file_path).await.is_ok() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Read file content
    let file_content = fs::read(&file.file_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create response with appropriate headers
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        file.mime_type.parse().unwrap_or_else(|_| "application/octet-stream".parse().unwrap())
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file.original_filename)
            .parse()
            .unwrap()
    );
    headers.insert(
        header::CONTENT_LENGTH,
        file.file_size.to_string().parse().unwrap()
    );

    Ok((headers, file_content))
}

async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    // Get file info before deletion
    let file = sqlx::query!(
        "SELECT file_path FROM files WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Delete from database
    let result = sqlx::query!("DELETE FROM files WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Delete file from disk (ignore errors if file doesn't exist)
    let _ = fs::remove_file(&file.file_path).await;

    // Log the deletion
    log_audit_action(&state.db_pool, auth.0.id, "DELETE", "file", id).await;

    Ok(Json(serde_json::json!({ "message": "File deleted successfully" })))
}

fn get_upload_directory() -> String {
    std::env::var("UPLOAD_DIRECTORY").unwrap_or_else(|_| "./uploads".to_string())
}

fn format_file_size(size: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as i64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
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