pub mod unit;
pub mod integration;
pub mod fixtures;
pub mod helpers;

// Common test utilities and shared test setup
use sqlx::{PgPool, Pool, Postgres};
use std::sync::Arc;
use testcontainers::{clients::Cli, images::postgres::Postgres as PostgresImage, Container};
use uuid::Uuid;

pub struct TestContext {
    pub db_pool: PgPool,
    pub _container: Option<Container<'static, PostgresImage>>,
}

impl TestContext {
    pub async fn new() -> Self {
        // Check if we should use a real database (for CI) or testcontainers
        if let Ok(database_url) = std::env::var("TEST_DATABASE_URL") {
            let pool = PgPool::connect(&database_url)
                .await
                .expect("Failed to connect to test database");
                
            // Run migrations
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("Failed to run migrations");
                
            Self {
                db_pool: pool,
                _container: None,
            }
        } else {
            // Use testcontainers for local testing
            let docker = Cli::default();
            let postgres_image = PostgresImage::default()
                .with_db_name("resolve_test")
                .with_user("test")
                .with_password("test");
                
            let container = docker.run(postgres_image);
            let connection_string = format!(
                "postgresql://test:test@{}:{}/resolve_test",
                container.get_host_address(),
                container.get_host_port_ipv4(5432)
            );
            
            let pool = PgPool::connect(&connection_string)
                .await
                .expect("Failed to connect to test database");
                
            // Run migrations
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("Failed to run migrations");
                
            Self {
                db_pool: pool,
                _container: Some(container),
            }
        }
    }
    
    pub async fn cleanup(&self) {
        // Clean up test data between tests
        let tables = [
            "time_entries", "tickets", "assets", "contacts", "clients",
            "m365_users", "m365_tenants",
            "azure_resources", "azure_resource_groups", "azure_subscriptions",
            "bitwarden_items", "bitwarden_collections", "bitwarden_organizations", "bitwarden_servers",
            "network_devices", "network_controllers",
            "passwords", "domains", "ssl_certificates"
        ];
        
        for table in tables {
            sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
                .execute(&self.db_pool)
                .await
                .ok(); // Ignore errors for tables that might not exist
        }
    }
}