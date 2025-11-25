use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
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
use crate::auth::middleware::AuthUser;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Documentation {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub template_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub content_type: String,
    pub summary: Option<String>,
    pub tags: Vec<String>,
    pub version: i32,
    pub status: String,
    pub visibility: String,
    pub featured_image: Option<String>,
    pub attachments: serde_json::Value,
    pub embedded_media: serde_json::Value,
    pub author_id: Option<Uuid>,
    pub last_editor_id: Option<Uuid>,
    pub published_at: Option<chrono::DateTime<Utc>>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub review_date: Option<chrono::NaiveDate>,
    pub view_count: i32,
    pub helpful_count: i32,
    pub not_helpful_count: i32,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<Vec<String>>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocTemplate {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub category: String,
    pub description: Option<String>,
    pub content: String,
    pub variables: serde_json::Value,
    pub icon: Option<String>,
    pub is_active: bool,
    pub usage_count: i32,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub client_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub template_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub content_type: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<String>,
    pub meta_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentQuery {
    pub client_id: Option<Uuid>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

pub fn documentation_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_documents).post(create_document))
        .route("/:id", get(get_document).put(update_document).delete(delete_document))
        .route("/:id/publish", post(publish_document))
        .route("/:id/versions", get(get_document_versions))
        .route("/templates", get(list_templates))
        .route("/templates/:id", get(get_template))
        .route("/search", get(search_documents))
}

async fn list_documents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DocumentQuery>,
    auth: AuthUser,
) -> Result<Json<Vec<Documentation>>, StatusCode> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let mut sql = "SELECT * FROM documentation WHERE 1=1".to_string();
    let mut params = Vec::new();

    if let Some(client_id) = query.client_id {
        sql.push_str(" AND client_id = $");
        sql.push_str(&(params.len() + 1).to_string());
        params.push(client_id.to_string());
    }

    if let Some(status) = &query.status {
        sql.push_str(" AND status = $");
        sql.push_str(&(params.len() + 1).to_string());
        params.push(status.clone());
    }

    if let Some(visibility) = &query.visibility {
        sql.push_str(" AND visibility = $");
        sql.push_str(&(params.len() + 1).to_string());
        params.push(visibility.clone());
    }

    if let Some(search) = &query.search {
        sql.push_str(" AND (title ILIKE $");
        sql.push_str(&(params.len() + 1).to_string());
        sql.push_str(" OR content ILIKE $");
        sql.push_str(&(params.len() + 1).to_string());
        sql.push_str(")");
        let search_term = format!("%{}%", search);
        params.push(search_term.clone());
        params.push(search_term);
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT $");
    sql.push_str(&(params.len() + 1).to_string());
    sql.push_str(" OFFSET $");
    sql.push_str(&(params.len() + 2).to_string());
    
    params.push(limit.to_string());
    params.push(offset.to_string());

    let documents = sqlx::query_as::<_, Documentation>(&sql)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(documents))
}

async fn get_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<Documentation>, StatusCode> {
    let document = sqlx::query_as!(
        Documentation,
        r#"
        SELECT id, client_id, category_id, template_id, parent_id, title, slug,
               content, content_type, summary, tags, version, status, visibility,
               featured_image, attachments, embedded_media, author_id, last_editor_id,
               published_at, expires_at, review_date, view_count, helpful_count,
               not_helpful_count, meta_description, meta_keywords,
               created_at, updated_at
        FROM documentation 
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Increment view count
    let _ = sqlx::query!(
        "UPDATE documentation SET view_count = view_count + 1 WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await;

    Ok(Json(document))
}

async fn create_document(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::new_v4();
    let slug = req.title.to_lowercase().replace(" ", "-").chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    sqlx::query!(
        r#"
        INSERT INTO documentation (
            id, client_id, category_id, template_id, title, slug, content,
            content_type, summary, tags, visibility, meta_description,
            author_id, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW())
        "#,
        id,
        req.client_id,
        req.category_id,
        req.template_id,
        req.title,
        slug,
        req.content,
        req.content_type.unwrap_or_else(|| "markdown".to_string()),
        req.summary,
        &req.tags.unwrap_or_default(),
        req.visibility.unwrap_or_else(|| "internal".to_string()),
        req.meta_description,
        auth.0.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "id": id,
        "message": "Document created successfully"
    })))
}

async fn update_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Create version backup first
    sqlx::query!(
        r#"
        INSERT INTO doc_versions (document_id, version_number, title, content, author_id)
        SELECT id, version, title, content, $2
        FROM documentation WHERE id = $1
        "#,
        id,
        auth.0.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = sqlx::query!(
        r#"
        UPDATE documentation SET
            title = $2, content = $3, summary = $4, tags = $5,
            visibility = $6, meta_description = $7, last_editor_id = $8,
            version = version + 1, updated_at = NOW()
        WHERE id = $1
        "#,
        id,
        req.title,
        req.content,
        req.summary,
        &req.tags.unwrap_or_default(),
        req.visibility.unwrap_or_else(|| "internal".to_string()),
        req.meta_description,
        auth.0.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({
        "message": "Document updated successfully"
    })))
}

async fn delete_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query!(
        "DELETE FROM documentation WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({
        "message": "Document deleted successfully"
    })))
}

async fn publish_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query!(
        r#"
        UPDATE documentation SET
            status = 'published',
            published_at = NOW(),
            last_editor_id = $2
        WHERE id = $1 AND status = 'draft'
        "#,
        id,
        auth.0.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({
        "message": "Document published successfully"
    })))
}

async fn get_document_versions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let versions = sqlx::query!(
        r#"
        SELECT dv.*, u.first_name || ' ' || u.last_name as author_name
        FROM doc_versions dv
        LEFT JOIN users u ON dv.author_id = u.id
        WHERE document_id = $1
        ORDER BY version_number DESC
        "#,
        id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<serde_json::Value> = versions.iter().map(|v| {
        serde_json::json!({
            "id": v.id,
            "version_number": v.version_number,
            "title": v.title,
            "change_summary": v.change_summary,
            "author_name": v.author_name,
            "created_at": v.created_at
        })
    }).collect();

    Ok(Json(result))
}

async fn list_templates(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<DocTemplate>>, StatusCode> {
    let templates = sqlx::query_as!(
        DocTemplate,
        r#"
        SELECT id, name, slug, category, description, content, variables,
               icon, is_active, usage_count, created_by, created_at, updated_at
        FROM doc_templates
        WHERE is_active = true
        ORDER BY category, name
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(templates))
}

async fn get_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<DocTemplate>, StatusCode> {
    let template = sqlx::query_as!(
        DocTemplate,
        r#"
        SELECT id, name, slug, category, description, content, variables,
               icon, is_active, usage_count, created_by, created_at, updated_at
        FROM doc_templates
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Increment usage count
    let _ = sqlx::query!(
        "UPDATE doc_templates SET usage_count = usage_count + 1 WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await;

    Ok(Json(template))
}

async fn search_documents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DocumentQuery>,
    auth: AuthUser,
) -> Result<Json<Vec<Documentation>>, StatusCode> {
    let search_term = query.search.unwrap_or_default();
    if search_term.is_empty() {
        return Ok(Json(vec![]));
    }

    let documents = sqlx::query_as!(
        Documentation,
        r#"
        SELECT id, client_id, category_id, template_id, parent_id, title, slug,
               content, content_type, summary, tags, version, status, visibility,
               featured_image, attachments, embedded_media, author_id, last_editor_id,
               published_at, expires_at, review_date, view_count, helpful_count,
               not_helpful_count, meta_description, meta_keywords,
               created_at, updated_at,
               ts_rank(search_vector, plainto_tsquery('english', $1)) as rank
        FROM documentation
        WHERE search_vector @@ plainto_tsquery('english', $1)
        AND status = 'published'
        ORDER BY rank DESC, created_at DESC
        LIMIT 50
        "#,
        search_term
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(documents))
}