use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
    pub monitoring_enabled: bool,
    pub last_monitored: Option<DateTime<Utc>>,
    pub monitoring_status: String, // "active", "expired", "error", "pending"
    pub whois_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SslCertificate {
    pub id: Uuid,
    pub domain_id: Option<Uuid>,
    pub client_id: Uuid,
    pub domain_name: String,
    pub port: i32,
    pub issuer: Option<String>,
    pub subject: Option<String>,
    pub serial_number: Option<String>,
    pub signature_algorithm: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub is_wildcard: bool,
    pub san_domains: Vec<String>, // Subject Alternative Names
    pub monitoring_enabled: bool,
    pub last_checked: Option<DateTime<Utc>>,
    pub status: String, // "valid", "expired", "expiring", "invalid", "error"
    pub certificate_chain: Option<String>,
    pub fingerprint_sha1: Option<String>,
    pub fingerprint_sha256: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Website {
    pub id: Uuid,
    pub client_id: Uuid,
    pub domain_id: Option<Uuid>,
    pub name: String,
    pub url: String,
    pub expected_status_code: i32,
    pub monitoring_enabled: bool,
    pub check_interval_minutes: i32,
    pub timeout_seconds: i32,
    pub last_checked: Option<DateTime<Utc>>,
    pub status: String, // "up", "down", "warning", "unknown"
    pub response_time_ms: Option<i32>,
    pub status_code: Option<i32>,
    pub response_headers: Option<serde_json::Value>,
    pub downtime_alerts_enabled: bool,
    pub performance_alerts_enabled: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DnsRecord {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub record_type: String, // A, AAAA, CNAME, MX, TXT, etc.
    pub name: String,
    pub value: String,
    pub ttl: Option<i32>,
    pub priority: Option<i32>, // For MX records
    pub monitoring_enabled: bool,
    pub last_checked: Option<DateTime<Utc>>,
    pub status: String, // "valid", "invalid", "error"
    pub expected_value: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MonitoringAlert {
    pub id: Uuid,
    pub client_id: Uuid,
    pub alert_type: String, // "domain_expiry", "ssl_expiry", "website_down", "dns_change"
    pub entity_type: String, // "domain", "ssl_certificate", "website", "dns_record"
    pub entity_id: Uuid,
    pub title: String,
    pub message: String,
    pub severity: String, // "critical", "warning", "info"
    pub status: String,   // "active", "acknowledged", "resolved"
    pub first_detected: DateTime<Utc>,
    pub last_detected: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub notification_sent: bool,
    pub notification_methods: Vec<String>, // "email", "slack", "webhook"
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct CreateDomainRequest {
    pub client_id: Uuid,
    pub name: String,
    pub registrar: Option<String>,
    pub nameservers: Option<Vec<String>>,
    pub registration_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub auto_renew: Option<bool>,
    pub monitoring_enabled: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSslCertificateRequest {
    pub client_id: Uuid,
    pub domain_id: Option<Uuid>,
    pub domain_name: String,
    pub port: Option<i32>,
    pub monitoring_enabled: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebsiteRequest {
    pub client_id: Uuid,
    pub domain_id: Option<Uuid>,
    pub name: String,
    pub url: String,
    pub expected_status_code: Option<i32>,
    pub monitoring_enabled: Option<bool>,
    pub check_interval_minutes: Option<i32>,
    pub timeout_seconds: Option<i32>,
    pub downtime_alerts_enabled: Option<bool>,
    pub performance_alerts_enabled: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DomainWithStatus {
    #[serde(flatten)]
    pub domain: Domain,
    pub days_until_expiry: Option<i32>,
    pub is_expired: bool,
    pub ssl_certificates: Vec<SslCertificate>,
    pub websites: Vec<Website>,
    pub active_alerts: Vec<MonitoringAlert>,
}

#[derive(Debug, Serialize)]
pub struct SslCertificateWithStatus {
    #[serde(flatten)]
    pub certificate: SslCertificate,
    pub days_until_expiry: Option<i32>,
    pub is_expired: bool,
    pub is_expiring_soon: bool, // Within 30 days
}

#[derive(Debug, Serialize)]
pub struct WebsiteStatus {
    #[serde(flatten)]
    pub website: Website,
    pub uptime_percentage: Option<f64>,
    pub avg_response_time: Option<f64>,
    pub recent_checks: Vec<WebsiteCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebsiteCheck {
    pub id: Uuid,
    pub website_id: Uuid,
    pub checked_at: DateTime<Utc>,
    pub status_code: Option<i32>,
    pub response_time_ms: i32,
    pub status: String, // "up", "down", "timeout", "error"
    pub error_message: Option<String>,
    pub response_headers: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct MonitoringDashboard {
    pub total_domains: i64,
    pub domains_expiring_30_days: i64,
    pub domains_expired: i64,
    pub total_ssl_certificates: i64,
    pub ssl_expiring_30_days: i64,
    pub ssl_expired: i64,
    pub total_websites: i64,
    pub websites_down: i64,
    pub active_alerts: i64,
    pub recent_alerts: Vec<MonitoringAlert>,
}

#[derive(Debug, Deserialize)]
pub struct WhoisLookupRequest {
    pub domain: String,
}

#[derive(Debug, Serialize)]
pub struct WhoisResponse {
    pub domain: String,
    pub registrar: Option<String>,
    pub creation_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub updated_date: Option<DateTime<Utc>>,
    pub nameservers: Vec<String>,
    pub status: Vec<String>,
    pub raw_data: String,
}

#[derive(Debug, Deserialize)]
pub struct DnsLookupRequest {
    pub domain: String,
    pub record_type: String, // A, AAAA, CNAME, MX, TXT, etc.
}

#[derive(Debug, Serialize)]
pub struct DnsLookupResponse {
    pub domain: String,
    pub record_type: String,
    pub records: Vec<DnsLookupResult>,
    pub nameservers: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DnsLookupResult {
    pub name: String,
    pub record_type: String,
    pub value: String,
    pub ttl: Option<i32>,
    pub priority: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AlertAcknowledgeRequest {
    pub acknowledged_by: Uuid,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkMonitoringRequest {
    pub domain_ids: Option<Vec<Uuid>>,
    pub ssl_ids: Option<Vec<Uuid>>,
    pub website_ids: Option<Vec<Uuid>>,
    pub action: String, // "enable", "disable", "check_now"
}

#[derive(Debug, Serialize)]
pub struct MonitoringStats {
    pub domains_monitored: i64,
    pub ssl_certificates_monitored: i64,
    pub websites_monitored: i64,
    pub total_checks_today: i64,
    pub avg_response_time_today: Option<f64>,
    pub uptime_percentage_today: Option<f64>,
}