use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProvider {
    pub id: Uuid,
    pub name: String,
    pub provider_type: String, // oauth2, oidc, saml
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub auth_url: Option<String>,
    pub token_url: Option<String>,
    pub userinfo_url: Option<String>,
    pub scopes: Vec<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>, // Encrypted
    pub private_key: Option<String>, // Encrypted
    pub public_key: Option<String>,
    pub certificate: Option<String>,
    pub uri: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub registrar: Option<String>,
    pub nameservers: Vec<String>,
    pub registration_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub auto_renew: bool,
    pub dns_records: serde_json::Value,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslCertificate {
    pub id: Uuid,
    pub domain_id: Option<Uuid>,
    pub client_id: Uuid,
    pub name: String,
    pub common_name: String,
    pub subject_alt_names: Vec<String>,
    pub issuer: String,
    pub issued_date: NaiveDate,
    pub expiry_date: NaiveDate,
    pub certificate_chain: Option<String>,
    pub private_key: Option<String>, // Encrypted
    pub auto_renew: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub network_type: String, // lan, wan, vpn, etc
    pub ip_range: String,
    pub subnet_mask: String,
    pub gateway: Option<String>,
    pub dns_servers: Vec<String>,
    pub vlan_id: Option<i32>,
    pub location_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub zip: Option<String>,
    pub timezone: String,
    pub primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareLicense {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub vendor: String,
    pub version: Option<String>,
    pub license_key: Option<String>, // Encrypted
    pub license_type: String, // perpetual, subscription, etc
    pub seats: Option<i32>,
    pub used_seats: Option<i32>,
    pub purchase_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub renewal_date: Option<NaiveDate>,
    pub cost: Option<Decimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: String, // info, warning, error, success
    pub entity_type: Option<String>, // ticket, invoice, asset, etc
    pub entity_id: Option<Uuid>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketTemplate {
    pub id: Uuid,
    pub name: String,
    pub subject: String,
    pub details: String,
    pub priority: String,
    pub category_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub billable: bool,
    pub estimated_hours: Option<Decimal>,
    pub tags: Vec<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringTicket {
    pub id: Uuid,
    pub template_id: Uuid,
    pub client_id: Uuid,
    pub frequency: String, // daily, weekly, monthly, etc
    pub interval_value: i32,
    pub next_run: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub ticket_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub kb_article_id: Option<Uuid>,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub file_path: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub id: Uuid,
    pub name: String,
    pub integration_type: String, // github, azure, google, stripe, etc
    pub config: serde_json::Value,
    pub credentials: serde_json::Value, // Encrypted
    pub enabled: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub vendor_id: Option<Uuid>,
    pub category_id: Uuid,
    pub amount: Decimal,
    pub tax_amount: Option<Decimal>,
    pub description: String,
    pub expense_date: NaiveDate,
    pub receipt_file_id: Option<Uuid>,
    pub billable: bool,
    pub billed: bool,
    pub invoice_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub contact_name: Option<String>,
    pub account_number: Option<String>,
    pub payment_terms: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseCategory {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tax_deductible: bool,
    pub created_at: DateTime<Utc>,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub billing_address: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub extension: Option<String>,
    pub mobile: Option<String>,
    pub department: Option<String>,
    pub notes: Option<String>,
    pub primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub asset_type: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub os: Option<String>,
    pub ip: Option<String>,
    pub mac: Option<String>,
    pub uri: Option<String>,
    pub status: String,
    pub location_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub warranty_expire: Option<DateTime<Utc>>,
    pub install_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Uuid,
    pub client_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub number: i32,
    pub subject: String,
    pub details: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: Option<Uuid>,
    pub billable: bool,
    pub opened_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    pub client_id: Uuid,
    pub contract_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub number: String,
    pub date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
    pub balance: Decimal,
    pub status: String,
    pub payment_terms: String,
    pub late_fee_percentage: Option<Decimal>,
    pub discount_percentage: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

// Enhanced BMS types

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password_hash: Option<String>, // For local auth
    pub role_id: Option<Uuid>,
    pub hourly_rate: Option<Decimal>,
    pub timezone: String,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub is_active: bool,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>, // TOTP secret, encrypted
    pub oauth_provider: Option<String>,
    pub oauth_id: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub contract_type: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub monthly_value: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    pub included_hours: Option<i32>,
    pub overage_rate: Option<Decimal>,
    pub status: String,
    pub terms: Option<String>,
    pub auto_renew: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sla {
    pub id: Uuid,
    pub contract_id: Option<Uuid>,
    pub name: String,
    pub priority: String,
    pub response_time_minutes: i32,
    pub resolution_time_hours: i32,
    pub business_hours_only: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: Uuid,
    pub ticket_id: Option<Uuid>,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub description: Option<String>,
    pub billable: bool,
    pub billed: bool,
    pub hourly_rate: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub budget: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    pub project_manager_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub project_id: Uuid,
    pub ticket_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub status: String,
    pub priority: String,
    pub estimated_hours: Option<Decimal>,
    pub actual_hours: Option<Decimal>,
    pub due_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub amount: Decimal,
    pub payment_date: NaiveDate,
    pub payment_method: Option<String>,
    pub reference_number: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketCategory {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub default_priority: String,
    pub default_sla_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub ticket_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub message: Option<String>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbArticle {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub category_id: Option<Uuid>,
    pub author_id: Uuid,
    pub status: String,
    pub public: bool,
    pub views: i32,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}