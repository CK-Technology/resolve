use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Password {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub username: Option<String>,
    pub password_encrypted: String,
    pub url: Option<String>,
    pub notes_encrypted: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub otp_secret_encrypted: Option<String>,
    pub phonetic_enabled: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub strength_score: i32,
    pub breach_detected: bool,
    pub folder_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasswordFolder {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePasswordRequest {
    pub client_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub otp_secret: Option<String>,
    pub phonetic_enabled: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub folder_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub otp_secret: Option<String>,
    pub phonetic_enabled: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub folder_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResponse {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub client_name: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub phonetic_password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub has_otp: bool,
    pub otp_code: Option<String>,
    pub phonetic_enabled: bool,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub expires_soon: bool,
    pub strength_score: i32,
    pub strength_label: String,
    pub breach_detected: bool,
    pub folder_id: Option<Uuid>,
    pub folder_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordListResponse {
    pub passwords: Vec<PasswordListItem>,
    pub folders: Vec<PasswordFolderResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordListItem {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub client_name: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub username: Option<String>,
    pub url: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub has_otp: bool,
    pub phonetic_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub expires_soon: bool,
    pub strength_score: i32,
    pub strength_label: String,
    pub breach_detected: bool,
    pub folder_id: Option<Uuid>,
    pub folder_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordFolderResponse {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub client_name: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub parent_name: Option<String>,
    pub password_count: i64,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderRequest {
    pub client_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePasswordRequest {
    pub length: u8,
    pub include_uppercase: bool,
    pub include_lowercase: bool,
    pub include_numbers: bool,
    pub include_symbols: bool,
    pub exclude_ambiguous: bool,
    pub phonetic_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePasswordResponse {
    pub password: String,
    pub phonetic_password: Option<String>,
    pub strength_score: i32,
    pub strength_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneticPassword {
    pub original: String,
    pub phonetic: String,
    pub segments: Vec<PhoneticSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneticSegment {
    pub character: String,
    pub phonetic: String,
    pub segment_type: PhoneticType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhoneticType {
    Letter,
    Number,
    Symbol,
    Space,
}

impl From<&str> for PhoneticType {
    fn from(s: &str) -> Self {
        match s {
            "letter" => PhoneticType::Letter,
            "number" => PhoneticType::Number,
            "symbol" => PhoneticType::Symbol,
            "space" => PhoneticType::Space,
            _ => PhoneticType::Letter,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasswordShare {
    pub id: Uuid,
    pub password_id: Uuid,
    pub share_token: String,
    pub created_by: Uuid,
    pub recipient_email: Option<String>,
    pub recipient_name: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub max_views: Option<i32>,
    pub view_count: i32,
    pub require_email_verification: bool,
    pub require_password: bool,
    pub access_password: Option<String>,
    pub one_time_use: bool,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePasswordShareRequest {
    pub password_id: Uuid,
    pub recipient_email: Option<String>,
    pub recipient_name: Option<String>,
    pub expires_in_hours: u32,
    pub max_views: Option<i32>,
    pub require_email_verification: bool,
    pub require_password: bool,
    pub access_password: Option<String>,
    pub one_time_use: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordShareResponse {
    pub id: Uuid,
    pub password_id: Uuid,
    pub password_name: String,
    pub share_token: String,
    pub share_url: String,
    pub recipient_email: Option<String>,
    pub recipient_name: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub max_views: Option<i32>,
    pub view_count: i32,
    pub require_email_verification: bool,
    pub require_password: bool,
    pub one_time_use: bool,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub is_expired: bool,
    pub created_by: Uuid,
    pub created_by_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPasswordShareRequest {
    pub share_token: String,
    pub email_verification_code: Option<String>,
    pub access_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordShareAccessResponse {
    pub password_name: String,
    pub password: String,
    pub phonetic_password: Option<String>,
    pub username: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub otp_code: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub remaining_views: Option<i32>,
}