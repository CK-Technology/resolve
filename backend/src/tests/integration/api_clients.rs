use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use crate::tests::{TestContext, fixtures::*};
use serial_test::serial;

// Helper function to create test app
async fn create_test_app(pool: sqlx::PgPool) -> axum::Router {
    // This would import your actual app creation function
    // For now, we'll create a minimal router for testing
    use axum::{routing::get, Json, Router};
    use serde_json::Value;
    
    Router::new()
        .route("/api/v1/clients", get(|| async { Json(json!([])) }))
        .with_state(pool)
}

#[tokio::test]
#[serial]
async fn test_get_clients_endpoint() {
    let ctx = TestContext::new().await;
    let app = create_test_app(ctx.db_pool.clone()).await;
    
    // Create test client
    let client_fixture = ClientFixture::default();
    insert_client_fixture(&ctx.db_pool, &client_fixture).await.unwrap();
    
    // Test GET /api/v1/clients
    let request = Request::builder()
        .uri("/api/v1/clients")
        .body(Body::empty())
        .unwrap();
        
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_create_client_endpoint() {
    let ctx = TestContext::new().await;
    let app = create_test_app(ctx.db_pool.clone()).await;
    
    let client_data = json!({
        "name": "Test Client",
        "identifier": "TEST",
        "primary_contact_email": "test@example.com",
        "status": "active"
    });
    
    let request = Request::builder()
        .uri("/api/v1/clients")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&client_data).unwrap()))
        .unwrap();
        
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    
    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_client_validation_errors() {
    let ctx = TestContext::new().await;
    let app = create_test_app(ctx.db_pool.clone()).await;
    
    // Test missing required fields
    let invalid_client = json!({
        "name": "Test Client"
        // Missing identifier and email
    });
    
    let request = Request::builder()
        .uri("/api/v1/clients")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_client).unwrap()))
        .unwrap();
        
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_client_not_found() {
    let ctx = TestContext::new().await;
    let app = create_test_app(ctx.db_pool.clone()).await;
    
    let request = Request::builder()
        .uri("/api/v1/clients/00000000-0000-0000-0000-000000000000")
        .body(Body::empty())
        .unwrap();
        
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    ctx.cleanup().await;
}