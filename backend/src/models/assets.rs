use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Asset {
    pub id: Uuid,
    pub client_id: Uuid,
    pub asset_type: String, // computer, server, printer, firewall, switch, router, wireless_ap, phone, etc.
    pub category: String,   // hardware, software, network, security
    pub name: String,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub asset_tag: Option<String>,
    pub model: Option<String>,
    pub manufacturer: Option<String>,
    pub location: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub status: String, // active, inactive, maintenance, retired, broken
    pub warranty_expires: Option<NaiveDate>,
    pub purchase_date: Option<NaiveDate>,
    pub purchase_cost: Option<rust_decimal::Decimal>,
    pub assigned_to: Option<String>,
    pub operating_system: Option<String>,
    pub installed_software: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub notes: Option<String>,
    pub monitoring_enabled: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetType {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub icon: Option<String>,
    pub custom_fields_schema: serde_json::Value,
    pub is_system_type: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Network {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub network_type: String, // lan, wan, wireless, vlan
    pub subnet: String,       // CIDR notation
    pub vlan_id: Option<i32>,
    pub gateway: Option<String>,
    pub dns_servers: Vec<String>,
    pub dhcp_enabled: bool,
    pub dhcp_range_start: Option<String>,
    pub dhcp_range_end: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub monitoring_enabled: bool,
    pub status: String, // active, inactive, down
    pub last_monitored: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WirelessNetwork {
    pub id: Uuid,
    pub client_id: Uuid,
    pub network_id: Option<Uuid>, // Link to main network
    pub ssid: String,
    pub bssid: Option<String>,
    pub security_type: String, // open, wep, wpa, wpa2, wpa3, enterprise
    pub password: Option<String>, // Encrypted
    pub channel: Option<i32>,
    pub frequency: Option<String>, // 2.4GHz, 5GHz, 6GHz
    pub bandwidth: Option<String>, // 20MHz, 40MHz, 80MHz, 160MHz
    pub access_points: Vec<Uuid>, // Associated APs
    pub hidden: bool,
    pub guest_network: bool,
    pub max_clients: Option<i32>,
    pub vlan_id: Option<i32>,
    pub monitoring_enabled: bool,
    pub status: String, // active, inactive, down
    pub signal_strength: Option<i32>,
    pub client_count: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetFile {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_type: String, // document, image, configuration, manual
    pub mime_type: String,
    pub file_size: i64,
    pub file_path: String,
    pub description: Option<String>,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetRelationship {
    pub id: Uuid,
    pub parent_asset_id: Uuid,
    pub child_asset_id: Uuid,
    pub relationship_type: String, // depends_on, connects_to, hosts, manages, monitors
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetConfiguration {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub config_type: String, // firewall_rules, switch_config, router_config, backup_job
    pub config_name: String,
    pub config_data: serde_json::Value,
    pub version: i32,
    pub is_active: bool,
    pub backup_location: Option<String>,
    pub last_backup: Option<DateTime<Utc>>,
    pub change_description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecretKey {
    pub id: Uuid,
    pub client_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub name: String,
    pub key_type: String, // api_key, ssl_cert, ssh_key, license_key, service_account
    pub key_data_encrypted: String,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rotation_days: Option<i32>,
    pub last_rotated: Option<DateTime<Utc>>,
    pub usage_count: i64,
    pub status: String, // active, expired, revoked, compromised
    pub tags: Vec<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

// Integration Models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Integration {
    pub id: Uuid,
    pub client_id: Uuid,
    pub integration_type: String, // unifi, fortigate, azure, veeam, synology, bitwarden
    pub name: String,
    pub endpoint_url: String,
    pub credentials_encrypted: String,
    pub enabled: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_interval_minutes: i32,
    pub sync_status: String, // success, error, syncing
    pub last_error: Option<String>,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationLog {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub action: String, // sync, import, export, backup
    pub status: String, // started, completed, failed
    pub details: Option<String>,
    pub assets_affected: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct CreateAssetRequest {
    pub client_id: Uuid,
    pub asset_type: String,
    pub category: String,
    pub name: String,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub asset_tag: Option<String>,
    pub model: Option<String>,
    pub manufacturer: Option<String>,
    pub location: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub status: Option<String>,
    pub warranty_expires: Option<NaiveDate>,
    pub purchase_date: Option<NaiveDate>,
    pub purchase_cost: Option<rust_decimal::Decimal>,
    pub assigned_to: Option<String>,
    pub operating_system: Option<String>,
    pub installed_software: Option<Vec<String>>,
    pub custom_fields: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub monitoring_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNetworkRequest {
    pub client_id: Uuid,
    pub name: String,
    pub network_type: String,
    pub subnet: String,
    pub vlan_id: Option<i32>,
    pub gateway: Option<String>,
    pub dns_servers: Option<Vec<String>>,
    pub dhcp_enabled: Option<bool>,
    pub dhcp_range_start: Option<String>,
    pub dhcp_range_end: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub monitoring_enabled: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWirelessNetworkRequest {
    pub client_id: Uuid,
    pub network_id: Option<Uuid>,
    pub ssid: String,
    pub bssid: Option<String>,
    pub security_type: String,
    pub password: Option<String>,
    pub channel: Option<i32>,
    pub frequency: Option<String>,
    pub bandwidth: Option<String>,
    pub hidden: Option<bool>,
    pub guest_network: Option<bool>,
    pub max_clients: Option<i32>,
    pub vlan_id: Option<i32>,
    pub monitoring_enabled: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSecretKeyRequest {
    pub client_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub name: String,
    pub key_type: String,
    pub key_data: String,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rotation_days: Option<i32>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateIntegrationRequest {
    pub client_id: Uuid,
    pub integration_type: String,
    pub name: String,
    pub endpoint_url: String,
    pub credentials: serde_json::Value,
    pub sync_interval_minutes: Option<i32>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AssetWithDetails {
    #[serde(flatten)]
    pub asset: Asset,
    pub files: Vec<AssetFile>,
    pub relationships: Vec<AssetRelationshipDetail>,
    pub configurations: Vec<AssetConfiguration>,
    pub warranty_status: String, // active, expiring, expired
    pub days_until_warranty_expires: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct AssetRelationshipDetail {
    #[serde(flatten)]
    pub relationship: AssetRelationship,
    pub parent_asset_name: String,
    pub child_asset_name: String,
}

#[derive(Debug, Serialize)]
pub struct NetworkTopology {
    pub networks: Vec<NetworkWithAssets>,
    pub wireless_networks: Vec<WirelessNetworkWithDetails>,
    pub unmanaged_assets: Vec<Asset>,
}

#[derive(Debug, Serialize)]
pub struct NetworkWithAssets {
    #[serde(flatten)]
    pub network: Network,
    pub connected_assets: Vec<Asset>,
    pub ip_utilization: Option<f64>,
    pub active_connections: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct WirelessNetworkWithDetails {
    #[serde(flatten)]
    pub wireless: WirelessNetwork,
    pub access_points: Vec<Asset>,
    pub connected_clients: Option<i32>,
    pub coverage_map: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AssetDashboard {
    pub total_assets: i64,
    pub assets_by_type: std::collections::HashMap<String, i64>,
    pub assets_by_status: std::collections::HashMap<String, i64>,
    pub warranty_expiring_30_days: i64,
    pub warranty_expired: i64,
    pub assets_offline: i64,
    pub network_summary: NetworkSummary,
    pub recent_changes: Vec<AssetChange>,
}

#[derive(Debug, Serialize)]
pub struct NetworkSummary {
    pub total_networks: i64,
    pub total_wireless_networks: i64,
    pub total_vlans: i64,
    pub networks_down: i64,
    pub avg_network_utilization: Option<f64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AssetChange {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub asset_name: String,
    pub change_type: String, // created, updated, deleted, moved, assigned
    pub description: String,
    pub changed_by: Uuid,
    pub changed_by_name: String,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssetQuery {
    pub client_id: Option<Uuid>,
    pub asset_type: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub location: Option<String>,
    pub assigned_to: Option<String>,
    pub search: Option<String>,
    pub warranty_expiring: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BulkAssetOperation {
    pub asset_ids: Vec<Uuid>,
    pub operation: String, // update_status, assign, move_location, enable_monitoring
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct IntegrationStatus {
    #[serde(flatten)]
    pub integration: Integration,
    pub is_connected: bool,
    pub last_sync_duration: Option<i64>, // seconds
    pub assets_synced: Option<i64>,
    pub errors_count: Option<i64>,
}

// UniFi specific models
#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiDevice {
    pub mac: String,
    pub name: String,
    pub model: String,
    pub ip: Option<String>,
    pub status: String,
    pub uptime: Option<i64>,
    pub clients: Option<i32>,
    pub load_avg: Option<f64>,
    pub firmware_version: Option<String>,
}

// FortiGate specific models
#[derive(Debug, Serialize, Deserialize)]
pub struct FortigatePolicy {
    pub id: i32,
    pub name: String,
    pub source: String,
    pub destination: String,
    pub service: String,
    pub action: String,
    pub status: String,
}

// Azure specific models
#[derive(Debug, Serialize, Deserialize)]
pub struct AzureResource {
    pub id: String,
    pub name: String,
    pub resource_type: String,
    pub resource_group: String,
    pub location: String,
    pub status: String,
    pub tags: std::collections::HashMap<String, String>,
}

// Veeam specific models
#[derive(Debug, Serialize, Deserialize)]
pub struct VeeamBackupJob {
    pub id: String,
    pub name: String,
    pub job_type: String,
    pub status: String,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub success_rate: Option<f64>,
}

// Synology specific models
#[derive(Debug, Serialize, Deserialize)]
pub struct SynologyVolume {
    pub id: String,
    pub name: String,
    pub status: String,
    pub size_total: i64,
    pub size_used: i64,
    pub raid_type: String,
    pub filesystem: String,
}