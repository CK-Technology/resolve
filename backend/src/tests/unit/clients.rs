use crate::tests::{TestContext, fixtures::*};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_create_client() {
    let ctx = TestContext::new().await;
    let fixture = ClientFixture::default();
    
    // Test inserting a client
    let result = insert_client_fixture(&ctx.db_pool, &fixture).await;
    assert!(result.is_ok());
    
    // Verify the client was created
    let client = sqlx::query!(
        "SELECT id, name, identifier FROM clients WHERE id = $1",
        fixture.id
    )
    .fetch_one(&ctx.db_pool)
    .await
    .expect("Failed to fetch client");
    
    assert_eq!(client.id, fixture.id);
    assert_eq!(client.name, fixture.name);
    assert_eq!(client.identifier, fixture.identifier);
    
    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_client_identifier_uniqueness() {
    let ctx = TestContext::new().await;
    let mut fixture1 = ClientFixture::default();
    let mut fixture2 = ClientFixture::default();
    
    // Both clients have the same identifier
    fixture1.identifier = "TEST".to_string();
    fixture2.identifier = "TEST".to_string();
    
    // Insert first client - should succeed
    let result1 = insert_client_fixture(&ctx.db_pool, &fixture1).await;
    assert!(result1.is_ok());
    
    // Insert second client with same identifier - should fail
    let result2 = insert_client_fixture(&ctx.db_pool, &fixture2).await;
    assert!(result2.is_err());
    
    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_client_status_validation() {
    let ctx = TestContext::new().await;
    let mut fixture = ClientFixture::default();
    
    // Test valid status
    fixture.status = "active".to_string();
    let result = insert_client_fixture(&ctx.db_pool, &fixture).await;
    assert!(result.is_ok());
    
    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_list_clients_pagination() {
    let ctx = TestContext::new().await;
    
    // Create multiple clients
    for i in 0..15 {
        let mut fixture = ClientFixture::default();
        fixture.identifier = format!("CLI{:03}", i);
        fixture.name = format!("Test Client {}", i);
        insert_client_fixture(&ctx.db_pool, &fixture).await.unwrap();
    }
    
    // Test pagination
    let clients = sqlx::query!(
        "SELECT id, name FROM clients ORDER BY name LIMIT 10 OFFSET 0"
    )
    .fetch_all(&ctx.db_pool)
    .await
    .expect("Failed to fetch clients");
    
    assert_eq!(clients.len(), 10);
    
    // Test second page
    let clients_page2 = sqlx::query!(
        "SELECT id, name FROM clients ORDER BY name LIMIT 10 OFFSET 10"
    )
    .fetch_all(&ctx.db_pool)
    .await
    .expect("Failed to fetch clients page 2");
    
    assert_eq!(clients_page2.len(), 5);
    
    ctx.cleanup().await;
}