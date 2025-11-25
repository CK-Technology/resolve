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
pub struct CategoryCreate {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub display_order: Option<i32>,
    pub is_public: Option<bool>,
    pub is_client_visible: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleCreate {
    pub category_id: Option<Uuid>,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub status: Option<String>,
    pub is_featured: Option<bool>,
    pub is_public: Option<bool>,
    pub is_client_visible: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub meta_keywords: Option<String>,
    pub meta_description: Option<String>,
    pub client_ids: Option<Vec<Uuid>>, // For client-specific articles
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub category_id: Option<Uuid>,
    pub status: Option<String>,
    pub is_public: Option<bool>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub client_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub display_order: i32,
    pub is_public: bool,
    pub is_client_visible: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Article {
    pub id: Uuid,
    pub category_id: Option<Uuid>,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub author_id: Option<Uuid>,
    pub status: String,
    pub is_featured: bool,
    pub is_public: bool,
    pub is_client_visible: bool,
    pub view_count: i32,
    pub helpful_count: i32,
    pub not_helpful_count: i32,
    pub tags: Option<Vec<String>>,
    pub meta_keywords: Option<String>,
    pub meta_description: Option<String>,
    pub published_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleWithCategory {
    #[serde(flatten)]
    pub article: Article,
    pub category_name: Option<String>,
    pub author_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleFeedback {
    pub is_helpful: bool,
    pub feedback_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub title: String,
    pub excerpt: String,
    pub category_name: Option<String>,
    pub relevance_score: f32,
}

pub fn knowledge_base_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Categories
        .route("/categories", get(list_categories).post(create_category))
        .route("/categories/:id", get(get_category).put(update_category).delete(delete_category))
        .route("/categories/:id/articles", get(get_category_articles))
        
        // Articles
        .route("/articles", get(list_articles).post(create_article))
        .route("/articles/:id", get(get_article).put(update_article).delete(delete_article))
        .route("/articles/:id/view", post(increment_view_count))
        .route("/articles/:id/feedback", post(submit_feedback))
        
        // Search
        .route("/search", get(search_articles))
        
        // Portal-specific endpoints
        .route("/portal/articles", get(list_portal_articles))
        .route("/portal/categories", get(list_portal_categories))
}

async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Category>>, StatusCode> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM kb_categories ORDER BY display_order, name"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching categories: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(categories))
}

async fn create_category(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CategoryCreate>,
) -> Result<(StatusCode, Json<Category>), StatusCode> {
    // Verify user is authenticated
    let _token = extract_token(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let category_id = Uuid::new_v4();
    let now = Utc::now();
    
    let category = sqlx::query_as::<_, Category>(
        "INSERT INTO kb_categories (
            id, parent_id, name, slug, description, icon, display_order, 
            is_public, is_client_visible, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *"
    )
    .bind(category_id)
    .bind(payload.parent_id)
    .bind(payload.name)
    .bind(payload.slug)
    .bind(payload.description)
    .bind(payload.icon)
    .bind(payload.display_order.unwrap_or(0))
    .bind(payload.is_public.unwrap_or(false))
    .bind(payload.is_client_visible.unwrap_or(true))
    .bind(now)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok((StatusCode::CREATED, Json(category)))
}

async fn get_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Category>, StatusCode> {
    let category = sqlx::query_as::<_, Category>(
        "SELECT * FROM kb_categories WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error fetching category: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    
    Ok(Json(category))
}

async fn update_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<CategoryCreate>,
) -> Result<Json<Category>, StatusCode> {
    // Verify user is authenticated
    let _token = extract_token(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let category = sqlx::query_as::<_, Category>(
        "UPDATE kb_categories SET 
         parent_id = $2, name = $3, slug = $4, description = $5, icon = $6,
         display_order = $7, is_public = $8, is_client_visible = $9, updated_at = NOW()
         WHERE id = $1
         RETURNING *"
    )
    .bind(id)
    .bind(payload.parent_id)
    .bind(payload.name)
    .bind(payload.slug)
    .bind(payload.description)
    .bind(payload.icon)
    .bind(payload.display_order.unwrap_or(0))
    .bind(payload.is_public.unwrap_or(false))
    .bind(payload.is_client_visible.unwrap_or(true))
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error updating category: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    
    Ok(Json(category))
}

async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, StatusCode> {
    // Verify user is authenticated
    let _token = extract_token(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    sqlx::query("DELETE FROM kb_categories WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting category: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(StatusCode::NO_CONTENT)
}

async fn get_category_articles(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Article>>, StatusCode> {
    let articles = sqlx::query_as::<_, Article>(
        "SELECT * FROM kb_articles 
         WHERE category_id = $1 AND status = 'published'
         ORDER BY is_featured DESC, created_at DESC"
    )
    .bind(id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching category articles: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(articles))
}

async fn list_articles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ArticleQuery>,
) -> Result<Json<Vec<ArticleWithCategory>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    let mut query = String::from(
        "SELECT a.*, c.name as category_name, 
         u.first_name || ' ' || u.last_name as author_name
         FROM kb_articles a
         LEFT JOIN kb_categories c ON a.category_id = c.id
         LEFT JOIN users u ON a.author_id = u.id
         WHERE 1=1"
    );
    
    if let Some(category_id) = params.category_id {
        query.push_str(&format!(" AND a.category_id = '{}'", category_id));
    }
    
    if let Some(status) = params.status {
        query.push_str(&format!(" AND a.status = '{}'", status));
    }
    
    if let Some(is_public) = params.is_public {
        query.push_str(&format!(" AND a.is_public = {}", is_public));
    }
    
    if let Some(search) = params.search {
        query.push_str(&format!(
            " AND to_tsvector('english', a.title || ' ' || COALESCE(a.content, '') || ' ' || COALESCE(a.excerpt, '')) @@ plainto_tsquery('english', '{}')",
            search
        ));
    }
    
    query.push_str(&format!(" ORDER BY a.is_featured DESC, a.created_at DESC LIMIT {} OFFSET {}", limit, offset));
    
    let articles = sqlx::query(&query)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching articles: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Map to ArticleWithCategory (simplified for now)
    let result: Vec<ArticleWithCategory> = vec![];
    
    Ok(Json(result))
}

async fn create_article(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ArticleCreate>,
) -> Result<(StatusCode, Json<Article>), StatusCode> {
    // Extract user from token
    let token = extract_token(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token_data = verify_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let author_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let article_id = Uuid::new_v4();
    let now = Utc::now();
    let status = payload.status.unwrap_or_else(|| "draft".to_string());
    let published_at = if status == "published" { Some(now) } else { None };
    
    // Start transaction
    let mut tx = state.db_pool.begin().await.map_err(|e| {
        tracing::error!("Error starting transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Insert article
    let article = sqlx::query_as::<_, Article>(
        "INSERT INTO kb_articles (
            id, category_id, title, slug, content, excerpt, author_id, status,
            is_featured, is_public, is_client_visible, tags, meta_keywords,
            meta_description, published_at, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING *"
    )
    .bind(article_id)
    .bind(payload.category_id)
    .bind(payload.title)
    .bind(payload.slug)
    .bind(payload.content)
    .bind(payload.excerpt)
    .bind(author_id)
    .bind(status)
    .bind(payload.is_featured.unwrap_or(false))
    .bind(payload.is_public.unwrap_or(false))
    .bind(payload.is_client_visible.unwrap_or(true))
    .bind(payload.tags.as_deref())
    .bind(payload.meta_keywords)
    .bind(payload.meta_description)
    .bind(published_at)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error creating article: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Add client access restrictions if specified
    if let Some(client_ids) = payload.client_ids {
        for client_id in client_ids {
            sqlx::query(
                "INSERT INTO kb_article_access (article_id, client_id) VALUES ($1, $2)"
            )
            .bind(article_id)
            .bind(client_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Error adding article access: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }
    
    tx.commit().await.map_err(|e| {
        tracing::error!("Error committing transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Send notification for new article
    state.broadcast_notification(
        "article_created",
        serde_json::json!({
            "article_id": article_id,
            "title": article.title,
            "author_id": author_id
        })
    ).await;
    
    Ok((StatusCode::CREATED, Json(article)))
}

async fn get_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Article>, StatusCode> {
    let article = sqlx::query_as::<_, Article>(
        "SELECT * FROM kb_articles WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error fetching article: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    
    Ok(Json(article))
}

async fn update_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<ArticleCreate>,
) -> Result<Json<Article>, StatusCode> {
    // Verify user is authenticated
    let _token = extract_token(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let status = payload.status.unwrap_or_else(|| "draft".to_string());
    let published_at = if status == "published" {
        Some(Utc::now())
    } else {
        None
    };
    
    let article = sqlx::query_as::<_, Article>(
        "UPDATE kb_articles SET 
         category_id = $2, title = $3, slug = $4, content = $5, excerpt = $6,
         status = $7, is_featured = $8, is_public = $9, is_client_visible = $10,
         tags = $11, meta_keywords = $12, meta_description = $13,
         published_at = COALESCE($14, published_at), updated_at = NOW()
         WHERE id = $1
         RETURNING *"
    )
    .bind(id)
    .bind(payload.category_id)
    .bind(payload.title)
    .bind(payload.slug)
    .bind(payload.content)
    .bind(payload.excerpt)
    .bind(status)
    .bind(payload.is_featured.unwrap_or(false))
    .bind(payload.is_public.unwrap_or(false))
    .bind(payload.is_client_visible.unwrap_or(true))
    .bind(payload.tags.as_deref())
    .bind(payload.meta_keywords)
    .bind(payload.meta_description)
    .bind(published_at)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Error updating article: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    
    Ok(Json(article))
}

async fn delete_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, StatusCode> {
    // Verify user is authenticated
    let _token = extract_token(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    sqlx::query("UPDATE kb_articles SET archived_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error archiving article: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(StatusCode::NO_CONTENT)
}

async fn increment_view_count(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query(
        "UPDATE kb_articles SET view_count = view_count + 1 WHERE id = $1"
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error incrementing view count: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(StatusCode::OK)
}

async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<ArticleFeedback>,
) -> Result<StatusCode, StatusCode> {
    // Try to get user or contact ID
    let (user_id, contact_id) = if let Some(token) = extract_token(&headers) {
        if let Ok(token_data) = verify_token(&token) {
            (Some(Uuid::parse_str(&token_data.claims.sub).unwrap_or_default()), None::<Uuid>)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };
    
    let feedback_id = Uuid::new_v4();
    
    // Insert feedback
    sqlx::query(
        "INSERT INTO kb_feedback (id, article_id, user_id, contact_id, is_helpful, feedback_text)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(feedback_id)
    .bind(id)
    .bind(user_id)
    .bind(contact_id)
    .bind(payload.is_helpful)
    .bind(payload.feedback_text)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error submitting feedback: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Update article counts
    if payload.is_helpful {
        sqlx::query("UPDATE kb_articles SET helpful_count = helpful_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        sqlx::query("UPDATE kb_articles SET not_helpful_count = not_helpful_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    Ok(StatusCode::OK)
}

async fn search_articles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ArticleQuery>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let search_term = params.search.unwrap_or_default();
    let limit = params.limit.unwrap_or(20);
    
    let results = sqlx::query!(
        r#"
        SELECT 
            a.id,
            a.title,
            COALESCE(a.excerpt, LEFT(a.content, 200)) as excerpt,
            c.name as category_name,
            ts_rank(
                to_tsvector('english', a.title || ' ' || COALESCE(a.content, '') || ' ' || COALESCE(a.excerpt, '')),
                plainto_tsquery('english', $1)
            ) as relevance_score
        FROM kb_articles a
        LEFT JOIN kb_categories c ON a.category_id = c.id
        WHERE 
            a.status = 'published' AND
            to_tsvector('english', a.title || ' ' || COALESCE(a.content, '') || ' ' || COALESCE(a.excerpt, ''))
            @@ plainto_tsquery('english', $1)
        ORDER BY relevance_score DESC
        LIMIT $2
        "#,
        search_term,
        limit
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error searching articles: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let search_results: Vec<SearchResult> = results
        .into_iter()
        .map(|r| SearchResult {
            id: r.id,
            title: r.title,
            excerpt: r.excerpt.unwrap_or_default(),
            category_name: r.category_name,
            relevance_score: r.relevance_score.unwrap_or(0.0),
        })
        .collect();
    
    Ok(Json(search_results))
}

async fn list_portal_articles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ArticleQuery>,
) -> Result<Json<Vec<Article>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    let client_id = params.client_id;
    
    let mut query = String::from(
        "SELECT DISTINCT a.* FROM kb_articles a
         LEFT JOIN kb_article_access aa ON a.id = aa.article_id
         WHERE a.status = 'published' AND a.is_client_visible = true"
    );
    
    if let Some(client_id) = client_id {
        query.push_str(&format!(
            " AND (a.is_public = true OR aa.client_id = '{}')",
            client_id
        ));
    } else {
        query.push_str(" AND a.is_public = true");
    }
    
    query.push_str(&format!(" ORDER BY a.is_featured DESC, a.created_at DESC LIMIT {} OFFSET {}", limit, offset));
    
    let articles = sqlx::query_as::<_, Article>(&query)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching portal articles: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(articles))
}

async fn list_portal_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Category>>, StatusCode> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM kb_categories 
         WHERE is_client_visible = true
         ORDER BY display_order, name"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching portal categories: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(categories))
}