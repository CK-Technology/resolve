//! Standardized error handling for Resolve API
//!
//! This module provides a consistent error response format across all endpoints.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code (e.g., "VALIDATION_ERROR", "NOT_FOUND", "UNAUTHORIZED")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional field-level errors for validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, Vec<String>>>,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Request path that caused the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            path: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, Vec<String>>) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    // Convenience constructors for common error types

    /// Create a 404 Not Found error
    pub fn not_found(message: impl Into<String>) -> AppError {
        AppError::NotFound(message.into())
    }

    /// Create a 500 Internal Server Error
    pub fn internal(message: impl Into<String>) -> AppError {
        AppError::InternalError(message.into())
    }

    /// Create a 403 Forbidden error
    pub fn forbidden(message: impl Into<String>) -> AppError {
        AppError::Forbidden(message.into())
    }

    /// Create a 401 Unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> AppError {
        AppError::Unauthorized(message.into())
    }

    /// Create a 400 Bad Request error
    pub fn bad_request(message: impl Into<String>) -> AppError {
        AppError::BadRequest(message.into())
    }

    /// Create a 409 Conflict error
    pub fn conflict(message: impl Into<String>) -> AppError {
        AppError::Conflict(message.into())
    }

    /// Create a validation error with a single field error
    pub fn validation_single(field: impl Into<String>, message: impl Into<String>) -> AppError {
        let mut details = HashMap::new();
        details.insert(field.into(), vec![message.into()]);
        AppError::ValidationError { details }
    }

    /// Create a validation error with multiple field errors
    pub fn validation(details: HashMap<String, Vec<String>>) -> AppError {
        AppError::ValidationError { details }
    }
}

/// Application error type that can be converted to HTTP responses
#[derive(Debug)]
pub enum AppError {
    // Authentication errors
    Unauthorized(String),
    InvalidCredentials,
    TokenExpired,
    MfaRequired,
    MfaInvalid,
    AccountLocked { until: chrono::DateTime<chrono::Utc> },

    // Authorization errors
    Forbidden(String),
    InsufficientPermissions { required: String },

    // Resource errors
    NotFound(String),
    Conflict(String),
    Gone(String),

    // Validation errors
    ValidationError { details: HashMap<String, Vec<String>> },
    BadRequest(String),

    // Rate limiting
    TooManyRequests { retry_after: u64 },

    // Server errors
    InternalError(String),
    DatabaseError(String),
    ExternalServiceError { service: String, message: String },

    // OAuth/OIDC errors
    OAuthError(String),
    ProviderNotFound(String),
    ProviderDisabled(String),
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized(_) | Self::InvalidCredentials | Self::TokenExpired => {
                StatusCode::UNAUTHORIZED
            }
            Self::MfaRequired => StatusCode::UNAUTHORIZED, // 401 with special code
            Self::MfaInvalid => StatusCode::UNAUTHORIZED,
            Self::AccountLocked { .. } => StatusCode::LOCKED,
            Self::Forbidden(_) | Self::InsufficientPermissions { .. } => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Gone(_) => StatusCode::GONE,
            Self::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::InternalError(_) | Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ExternalServiceError { .. } => StatusCode::BAD_GATEWAY,
            Self::OAuthError(_) | Self::ProviderNotFound(_) | Self::ProviderDisabled(_) => {
                StatusCode::BAD_REQUEST
            }
        }
    }

    /// Get the error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::MfaRequired => "MFA_REQUIRED",
            Self::MfaInvalid => "MFA_INVALID",
            Self::AccountLocked { .. } => "ACCOUNT_LOCKED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::InsufficientPermissions { .. } => "INSUFFICIENT_PERMISSIONS",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Gone(_) => "GONE",
            Self::ValidationError { .. } => "VALIDATION_ERROR",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::TooManyRequests { .. } => "TOO_MANY_REQUESTS",
            Self::InternalError(_) => "INTERNAL_ERROR",
            Self::DatabaseError(_) => "DATABASE_ERROR",
            Self::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            Self::OAuthError(_) => "OAUTH_ERROR",
            Self::ProviderNotFound(_) => "PROVIDER_NOT_FOUND",
            Self::ProviderDisabled(_) => "PROVIDER_DISABLED",
        }
    }

    /// Get the error message
    pub fn message(&self) -> String {
        match self {
            Self::Unauthorized(msg) => msg.clone(),
            Self::InvalidCredentials => "Invalid email or password".to_string(),
            Self::TokenExpired => "Authentication token has expired".to_string(),
            Self::MfaRequired => "Multi-factor authentication code required".to_string(),
            Self::MfaInvalid => "Invalid multi-factor authentication code".to_string(),
            Self::AccountLocked { until } => {
                format!("Account is locked until {}", until.to_rfc3339())
            }
            Self::Forbidden(msg) => msg.clone(),
            Self::InsufficientPermissions { required } => {
                format!("Insufficient permissions. Required: {}", required)
            }
            Self::NotFound(resource) => format!("{} not found", resource),
            Self::Conflict(msg) => msg.clone(),
            Self::Gone(msg) => msg.clone(),
            Self::ValidationError { .. } => "Validation failed".to_string(),
            Self::BadRequest(msg) => msg.clone(),
            Self::TooManyRequests { retry_after } => {
                format!("Too many requests. Retry after {} seconds", retry_after)
            }
            Self::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                "An internal error occurred".to_string()
            }
            Self::DatabaseError(msg) => {
                tracing::error!("Database error: {}", msg);
                "A database error occurred".to_string()
            }
            Self::ExternalServiceError { service, message } => {
                tracing::error!("External service error ({}): {}", service, message);
                format!("External service '{}' is unavailable", service)
            }
            Self::OAuthError(msg) => format!("OAuth error: {}", msg),
            Self::ProviderNotFound(name) => format!("Auth provider '{}' not found", name),
            Self::ProviderDisabled(name) => format!("Auth provider '{}' is disabled", name),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let mut error = ApiError::new(self.error_code(), self.message());

        // Add details for validation errors
        if let Self::ValidationError { details } = &self {
            error.details = Some(details.clone());
        }

        // Add retry-after header for rate limiting
        if let Self::TooManyRequests { retry_after } = &self {
            return (
                status,
                [("Retry-After", retry_after.to_string())],
                Json(error),
            )
                .into_response();
        }

        // Add locked-until info
        if let Self::AccountLocked { until } = &self {
            let mut details = HashMap::new();
            details.insert("locked_until".to_string(), vec![until.to_rfc3339()]);
            error.details = Some(details);
        }

        (status, Json(error)).into_response()
    }
}

// Implement From for common error types
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound("Resource".to_string()),
            _ => Self::DatabaseError(err.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => Self::TokenExpired,
            _ => Self::Unauthorized(format!("Invalid token: {}", err)),
        }
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(_err: argon2::password_hash::Error) -> Self {
        Self::InternalError("Password hashing error".to_string())
    }
}

/// Result type alias for handlers
pub type ApiResult<T> = Result<T, AppError>;

/// Helper to create validation errors
pub fn validation_error(field: &str, message: &str) -> AppError {
    let mut details = HashMap::new();
    details.insert(field.to_string(), vec![message.to_string()]);
    AppError::ValidationError { details }
}

/// Helper to add multiple validation errors
pub struct ValidationBuilder {
    details: HashMap<String, Vec<String>>,
}

impl ValidationBuilder {
    pub fn new() -> Self {
        Self {
            details: HashMap::new(),
        }
    }

    pub fn error(mut self, field: &str, message: &str) -> Self {
        self.details
            .entry(field.to_string())
            .or_insert_with(Vec::new)
            .push(message.to_string());
        self
    }

    pub fn build(self) -> Option<AppError> {
        if self.details.is_empty() {
            None
        } else {
            Some(AppError::ValidationError {
                details: self.details,
            })
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.details.is_empty()
    }
}

impl Default for ValidationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_builder() {
        let error = ValidationBuilder::new()
            .error("email", "Email is required")
            .error("email", "Email must be valid")
            .error("password", "Password is too short")
            .build();

        assert!(error.is_some());
        if let Some(AppError::ValidationError { details }) = error {
            assert_eq!(details.get("email").unwrap().len(), 2);
            assert_eq!(details.get("password").unwrap().len(), 1);
        }
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(AppError::InvalidCredentials.error_code(), "INVALID_CREDENTIALS");
        assert_eq!(AppError::MfaRequired.error_code(), "MFA_REQUIRED");
        assert_eq!(AppError::NotFound("User".to_string()).status_code(), StatusCode::NOT_FOUND);
    }
}
