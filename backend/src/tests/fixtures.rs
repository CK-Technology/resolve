use fake::{Fake, Faker};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::json;

// Test fixtures for creating sample data

#[derive(Debug, Clone)]
pub struct ClientFixture {
    pub id: Uuid,
    pub name: String,
    pub identifier: String,
    pub primary_contact_email: String,
    pub primary_contact_phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Default for ClientFixture {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: Faker.fake(),
            identifier: format!("{:04}", (1000..9999).fake::<u32>()),
            primary_contact_email: Faker.fake(),
            primary_contact_phone: Some(Faker.fake()),
            address: Some(Faker.fake()),
            city: Some(Faker.fake()),
            state: Some("CA".to_string()),
            zip_code: Some(Faker.fake()),
            country: Some("US".to_string()),
            website: Some(format!("https://{}", Faker.fake::<String>())),
            notes: Some(Faker.fake()),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TicketFixture {
    pub id: Uuid,
    pub client_id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub status: String,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
}

impl TicketFixture {
    pub fn new_with_client(client_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            client_id,
            title: Faker.fake(),
            description: Faker.fake(),
            priority: "medium".to_string(),
            status: "open".to_string(),
            assigned_to: Some(Faker.fake()),
            created_at: Utc::now(),
            updated_at: None,
            due_date: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssetFixture {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub purchase_date: Option<chrono::NaiveDate>,
    pub warranty_expiry: Option<chrono::NaiveDate>,
    pub status: String,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl AssetFixture {
    pub fn new_with_client(client_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            client_id,
            name: Faker.fake(),
            asset_type: "server".to_string(),
            manufacturer: Some("Dell".to_string()),
            model: Some(Faker.fake()),
            serial_number: Some(Faker.fake()),
            purchase_date: Some(Faker.fake()),
            warranty_expiry: Some(Faker.fake()),
            status: "active".to_string(),
            location: Some(Faker.fake()),
            notes: Some(Faker.fake()),
            created_at: Utc::now(),
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct M365TenantFixture {
    pub id: Uuid,
    pub client_id: Uuid,
    pub tenant_id: String,
    pub tenant_name: String,
    pub domain_name: String,
    pub display_name: Option<String>,
    pub tenant_type: String,
    pub status: String,
    pub sync_enabled: bool,
    pub total_licenses: i32,
    pub assigned_licenses: i32,
    pub available_licenses: i32,
    pub mfa_required: bool,
    pub created_at: DateTime<Utc>,
}

impl M365TenantFixture {
    pub fn new_with_client(client_id: Uuid) -> Self {
        let total = (50..200).fake::<i32>();
        let assigned = (total as f32 * 0.8) as i32;
        
        Self {
            id: Uuid::new_v4(),
            client_id,
            tenant_id: Uuid::new_v4().to_string(),
            tenant_name: format!("{} Organization", Faker.fake::<String>()),
            domain_name: format!("{}.onmicrosoft.com", Faker.fake::<String>().to_lowercase()),
            display_name: Some(Faker.fake()),
            tenant_type: "business".to_string(),
            status: "active".to_string(),
            sync_enabled: true,
            total_licenses: total,
            assigned_licenses: assigned,
            available_licenses: total - assigned,
            mfa_required: true,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AzureSubscriptionFixture {
    pub id: Uuid,
    pub client_id: Uuid,
    pub subscription_id: String,
    pub subscription_name: String,
    pub tenant_id: String,
    pub state: Option<String>,
    pub current_spend_usd: Option<f64>,
    pub budget_limit_usd: Option<f64>,
    pub budget_alerts_enabled: bool,
    pub sync_enabled: bool,
    pub last_sync_status: String,
    pub created_at: DateTime<Utc>,
}

impl AzureSubscriptionFixture {
    pub fn new_with_client(client_id: Uuid) -> Self {
        let budget = (1000.0..10000.0).fake::<f64>();
        let spend = budget * (0.3..0.9).fake::<f64>();
        
        Self {
            id: Uuid::new_v4(),
            client_id,
            subscription_id: Uuid::new_v4().to_string(),
            subscription_name: format!("Azure Subscription - {}", Faker.fake::<String>()),
            tenant_id: Uuid::new_v4().to_string(),
            state: Some("Enabled".to_string()),
            current_spend_usd: Some(spend),
            budget_limit_usd: Some(budget),
            budget_alerts_enabled: true,
            sync_enabled: true,
            last_sync_status: "success".to_string(),
            created_at: Utc::now(),
        }
    }
}

// Helper function to insert test fixtures into database
pub async fn insert_client_fixture(pool: &sqlx::PgPool, fixture: &ClientFixture) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO clients (
            id, name, identifier, primary_contact_email, primary_contact_phone,
            address, city, state, zip_code, country, website, notes, status, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        fixture.id,
        fixture.name,
        fixture.identifier,
        fixture.primary_contact_email,
        fixture.primary_contact_phone,
        fixture.address,
        fixture.city,
        fixture.state,
        fixture.zip_code,
        fixture.country,
        fixture.website,
        fixture.notes,
        fixture.status,
        fixture.created_at
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn insert_ticket_fixture(pool: &sqlx::PgPool, fixture: &TicketFixture) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO tickets (
            id, client_id, title, description, priority, status, assigned_to, created_at, due_date
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        fixture.id,
        fixture.client_id,
        fixture.title,
        fixture.description,
        fixture.priority,
        fixture.status,
        fixture.assigned_to,
        fixture.created_at,
        fixture.due_date
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn insert_m365_tenant_fixture(pool: &sqlx::PgPool, fixture: &M365TenantFixture) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO m365_tenants (
            id, client_id, tenant_id, tenant_name, domain_name, display_name,
            tenant_type, status, sync_enabled, total_licenses, assigned_licenses,
            available_licenses, mfa_required, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        fixture.id,
        fixture.client_id,
        fixture.tenant_id,
        fixture.tenant_name,
        fixture.domain_name,
        fixture.display_name,
        fixture.tenant_type,
        fixture.status,
        fixture.sync_enabled,
        fixture.total_licenses,
        fixture.assigned_licenses,
        fixture.available_licenses,
        fixture.mfa_required,
        fixture.created_at
    )
    .execute(pool)
    .await?;
    
    Ok(())
}