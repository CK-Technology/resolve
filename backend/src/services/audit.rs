use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use std::net::IpAddr;

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type AuditResult<T> = Result<T, AuditError>;

/// Audit logging service for comprehensive activity tracking
pub struct AuditService {
    pool: PgPool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    Create,
    Update,
    Delete,
    View,
    Export,
    Login,
    Logout,
    PasswordChange,
    PermissionChange,
    ApiKeyCreate,
    ApiKeyRevoke,
    BulkOperation,
    Import,
    Archive,
    Restore,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::View => "view",
            Self::Export => "export",
            Self::Login => "login",
            Self::Logout => "logout",
            Self::PasswordChange => "password_change",
            Self::PermissionChange => "permission_change",
            Self::ApiKeyCreate => "api_key_create",
            Self::ApiKeyRevoke => "api_key_revoke",
            Self::BulkOperation => "bulk_operation",
            Self::Import => "import",
            Self::Archive => "archive",
            Self::Restore => "restore",
        }
    }

    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            Self::PasswordChange | Self::PermissionChange | Self::ApiKeyCreate | Self::ApiKeyRevoke
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditSeverity {
    Info,
    Warning,
    Critical,
}

impl AuditSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

impl Default for AuditSeverity {
    fn default() -> Self {
        Self::Info
    }
}

/// Builder for creating audit log entries
pub struct AuditEntryBuilder {
    user_id: Option<Uuid>,
    user_email: Option<String>,
    api_key_id: Option<Uuid>,
    ip_address: Option<IpAddr>,
    user_agent: Option<String>,
    action: AuditAction,
    resource_type: String,
    resource_id: Option<Uuid>,
    resource_name: Option<String>,
    changes: Option<JsonValue>,
    metadata: Option<JsonValue>,
    request_id: Option<Uuid>,
    severity: AuditSeverity,
}

impl AuditEntryBuilder {
    pub fn new(action: AuditAction, resource_type: impl Into<String>) -> Self {
        Self {
            user_id: None,
            user_email: None,
            api_key_id: None,
            ip_address: None,
            user_agent: None,
            action,
            resource_type: resource_type.into(),
            resource_id: None,
            resource_name: None,
            changes: None,
            metadata: None,
            request_id: None,
            severity: AuditSeverity::default(),
        }
    }

    pub fn user(mut self, id: Uuid, email: Option<String>) -> Self {
        self.user_id = Some(id);
        self.user_email = email;
        self
    }

    pub fn api_key(mut self, id: Uuid) -> Self {
        self.api_key_id = Some(id);
        self
    }

    pub fn ip_address(mut self, ip: IpAddr) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    pub fn resource(mut self, id: Uuid, name: Option<String>) -> Self {
        self.resource_id = Some(id);
        self.resource_name = name;
        self
    }

    pub fn changes<T: Serialize>(mut self, changes: &T) -> Result<Self, serde_json::Error> {
        self.changes = Some(serde_json::to_value(changes)?);
        Ok(self)
    }

    pub fn changes_json(mut self, changes: JsonValue) -> Self {
        self.changes = Some(changes);
        self
    }

    pub fn metadata<T: Serialize>(mut self, metadata: &T) -> Result<Self, serde_json::Error> {
        self.metadata = Some(serde_json::to_value(metadata)?);
        Ok(self)
    }

    pub fn metadata_json(mut self, metadata: JsonValue) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn request_id(mut self, id: Uuid) -> Self {
        self.request_id = Some(id);
        self
    }

    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn critical(mut self) -> Self {
        self.severity = AuditSeverity::Critical;
        self
    }

    pub fn warning(mut self) -> Self {
        self.severity = AuditSeverity::Warning;
        self
    }
}

/// Represents changes made during an update operation
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldChange {
    pub old: JsonValue,
    pub new: JsonValue,
}

impl FieldChange {
    pub fn new<T: Serialize>(old: &T, new: &T) -> Result<Self, serde_json::Error> {
        Ok(Self {
            old: serde_json::to_value(old)?,
            new: serde_json::to_value(new)?,
        })
    }
}

/// Helper to track changes between old and new values
#[derive(Debug, Default)]
pub struct ChangeTracker {
    changes: serde_json::Map<String, JsonValue>,
}

impl ChangeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track<T: Serialize + PartialEq>(
        &mut self,
        field: &str,
        old: &T,
        new: &T,
    ) -> Result<bool, serde_json::Error> {
        if old != new {
            let change = FieldChange::new(old, new)?;
            self.changes.insert(field.to_string(), serde_json::to_value(change)?);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    pub fn into_json(self) -> JsonValue {
        JsonValue::Object(self.changes)
    }
}

impl AuditService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Log an audit entry using the builder
    pub async fn log(&self, entry: AuditEntryBuilder) -> AuditResult<Uuid> {
        let is_sensitive = entry.action.is_sensitive();

        let id: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO audit_logs (
                user_id, user_email, api_key_id, ip_address, user_agent,
                action, resource_type, resource_id, resource_name,
                changes, metadata, request_id, is_sensitive, severity
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id
            "#,
        )
        .bind(entry.user_id)
        .bind(&entry.user_email)
        .bind(entry.api_key_id)
        .bind(entry.ip_address.map(|ip| ip.to_string()))
        .bind(&entry.user_agent)
        .bind(entry.action.as_str())
        .bind(&entry.resource_type)
        .bind(entry.resource_id)
        .bind(&entry.resource_name)
        .bind(&entry.changes)
        .bind(&entry.metadata)
        .bind(entry.request_id)
        .bind(is_sensitive)
        .bind(entry.severity.as_str())
        .fetch_one(&self.pool)
        .await?;

        Ok(id.0)
    }

    /// Quick log for simple actions
    pub async fn log_simple(
        &self,
        user_id: Uuid,
        action: AuditAction,
        resource_type: &str,
        resource_id: Uuid,
    ) -> AuditResult<Uuid> {
        self.log(
            AuditEntryBuilder::new(action, resource_type)
                .user(user_id, None)
                .resource(resource_id, None),
        )
        .await
    }

    /// Get audit logs for a specific resource
    pub async fn get_resource_history(
        &self,
        resource_type: &str,
        resource_id: Uuid,
        limit: i64,
    ) -> AuditResult<Vec<AuditLogEntry>> {
        let entries = sqlx::query_as(
            r#"
            SELECT
                id, user_id, user_email, api_key_id, ip_address, user_agent,
                action, resource_type, resource_id, resource_name,
                changes, metadata, request_id, is_sensitive, severity, created_at
            FROM audit_logs
            WHERE resource_type = $1 AND resource_id = $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(resource_type)
        .bind(resource_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Get audit logs for a specific user
    pub async fn get_user_activity(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> AuditResult<Vec<AuditLogEntry>> {
        let entries = sqlx::query_as(
            r#"
            SELECT
                id, user_id, user_email, api_key_id, ip_address, user_agent,
                action, resource_type, resource_id, resource_name,
                changes, metadata, request_id, is_sensitive, severity, created_at
            FROM audit_logs
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Get recent sensitive operations
    pub async fn get_sensitive_operations(&self, limit: i64) -> AuditResult<Vec<AuditLogEntry>> {
        let entries = sqlx::query_as(
            r#"
            SELECT
                id, user_id, user_email, api_key_id, ip_address, user_agent,
                action, resource_type, resource_id, resource_name,
                changes, metadata, request_id, is_sensitive, severity, created_at
            FROM audit_logs
            WHERE is_sensitive = true
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Search audit logs with filters
    pub async fn search(
        &self,
        filters: AuditSearchFilters,
        limit: i64,
        offset: i64,
    ) -> AuditResult<Vec<AuditLogEntry>> {
        let mut query = String::from(
            r#"
            SELECT
                id, user_id, user_email, api_key_id, ip_address, user_agent,
                action, resource_type, resource_id, resource_name,
                changes, metadata, request_id, is_sensitive, severity, created_at
            FROM audit_logs
            WHERE 1=1
            "#,
        );

        let mut params_count = 0;

        if filters.user_id.is_some() {
            params_count += 1;
            query.push_str(&format!(" AND user_id = ${}", params_count));
        }
        if filters.action.is_some() {
            params_count += 1;
            query.push_str(&format!(" AND action = ${}", params_count));
        }
        if filters.resource_type.is_some() {
            params_count += 1;
            query.push_str(&format!(" AND resource_type = ${}", params_count));
        }
        if filters.from_date.is_some() {
            params_count += 1;
            query.push_str(&format!(" AND created_at >= ${}", params_count));
        }
        if filters.to_date.is_some() {
            params_count += 1;
            query.push_str(&format!(" AND created_at <= ${}", params_count));
        }
        if filters.sensitive_only {
            query.push_str(" AND is_sensitive = true");
        }

        query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            params_count + 1,
            params_count + 2
        ));

        // Build the query dynamically
        let mut db_query = sqlx::query_as::<_, AuditLogEntry>(&query);

        if let Some(user_id) = filters.user_id {
            db_query = db_query.bind(user_id);
        }
        if let Some(action) = filters.action {
            db_query = db_query.bind(action);
        }
        if let Some(resource_type) = filters.resource_type {
            db_query = db_query.bind(resource_type);
        }
        if let Some(from_date) = filters.from_date {
            db_query = db_query.bind(from_date);
        }
        if let Some(to_date) = filters.to_date {
            db_query = db_query.bind(to_date);
        }

        db_query = db_query.bind(limit).bind(offset);

        let entries = db_query.fetch_all(&self.pool).await?;

        Ok(entries)
    }
}

#[derive(Debug, Default)]
pub struct AuditSearchFilters {
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
    pub sensitive_only: bool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_email: Option<String>,
    pub api_key_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub resource_name: Option<String>,
    pub changes: Option<JsonValue>,
    pub metadata: Option<JsonValue>,
    pub request_id: Option<Uuid>,
    pub is_sensitive: bool,
    pub severity: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
