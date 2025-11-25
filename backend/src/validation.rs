//! Request validation for Resolve API
//!
//! Provides type-safe validation with clear error messages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{AppError, ValidationBuilder};

/// Validation result type
pub type ValidationResult<T> = Result<T, AppError>;

/// Validated wrapper type - indicates the value has been validated
#[derive(Debug, Clone)]
pub struct Validated<T>(pub T);

impl<T> Validated<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Validated<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// String validation helpers
pub mod string {
    use super::*;

    /// Validate required non-empty string
    pub fn required(value: &Option<String>, field: &str) -> ValidationResult<String> {
        match value {
            Some(s) if !s.trim().is_empty() => Ok(s.trim().to_string()),
            Some(_) => Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(field.to_string(), vec![format!("{} cannot be empty", field)]);
                    d
                },
            }),
            None => Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(field.to_string(), vec![format!("{} is required", field)]);
                    d
                },
            }),
        }
    }

    /// Validate optional string with max length
    pub fn max_length(value: &Option<String>, field: &str, max: usize) -> ValidationResult<Option<String>> {
        match value {
            Some(s) if s.len() > max => Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(
                        field.to_string(),
                        vec![format!("{} must be {} characters or less", field, max)],
                    );
                    d
                },
            }),
            Some(s) => Ok(Some(s.trim().to_string())),
            None => Ok(None),
        }
    }

    /// Validate required string with length constraints
    pub fn required_length(
        value: &Option<String>,
        field: &str,
        min: usize,
        max: usize,
    ) -> ValidationResult<String> {
        let s = required(value, field)?;
        if s.len() < min {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(
                        field.to_string(),
                        vec![format!("{} must be at least {} characters", field, min)],
                    );
                    d
                },
            });
        }
        if s.len() > max {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(
                        field.to_string(),
                        vec![format!("{} must be {} characters or less", field, max)],
                    );
                    d
                },
            });
        }
        Ok(s)
    }
}

/// Email validation
pub mod email {
    use super::*;

    /// Validate email format
    pub fn validate(value: &str, field: &str) -> ValidationResult<String> {
        let email = value.trim().to_lowercase();

        // Basic email validation
        if email.is_empty() {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(field.to_string(), vec![format!("{} is required", field)]);
                    d
                },
            });
        }

        // Must contain @ and have parts before and after
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(field.to_string(), vec!["Invalid email format".to_string()]);
                    d
                },
            });
        }

        // Domain must contain at least one dot
        if !parts[1].contains('.') {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(field.to_string(), vec!["Invalid email domain".to_string()]);
                    d
                },
            });
        }

        Ok(email)
    }

    /// Validate optional email
    pub fn validate_optional(value: &Option<String>, field: &str) -> ValidationResult<Option<String>> {
        match value {
            Some(s) if !s.trim().is_empty() => Ok(Some(validate(s, field)?)),
            _ => Ok(None),
        }
    }
}

/// UUID validation
pub mod uuid {
    use super::*;

    /// Validate required UUID
    pub fn required(value: &Option<uuid::Uuid>, field: &str) -> ValidationResult<uuid::Uuid> {
        value.ok_or_else(|| AppError::ValidationError {
            details: {
                let mut d = HashMap::new();
                d.insert(field.to_string(), vec![format!("{} is required", field)]);
                d
            },
        })
    }

    /// Parse UUID from string
    pub fn parse(value: &str, field: &str) -> ValidationResult<uuid::Uuid> {
        uuid::Uuid::parse_str(value).map_err(|_| AppError::ValidationError {
            details: {
                let mut d = HashMap::new();
                d.insert(field.to_string(), vec![format!("Invalid {} format", field)]);
                d
            },
        })
    }
}

/// Numeric validation
pub mod number {
    use super::*;

    /// Validate required positive integer
    pub fn required_positive(value: &Option<i64>, field: &str) -> ValidationResult<i64> {
        match value {
            Some(n) if *n > 0 => Ok(*n),
            Some(_) => Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(field.to_string(), vec![format!("{} must be positive", field)]);
                    d
                },
            }),
            None => Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(field.to_string(), vec![format!("{} is required", field)]);
                    d
                },
            }),
        }
    }

    /// Validate number in range
    pub fn in_range(value: i64, field: &str, min: i64, max: i64) -> ValidationResult<i64> {
        if value < min || value > max {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(
                        field.to_string(),
                        vec![format!("{} must be between {} and {}", field, min, max)],
                    );
                    d
                },
            });
        }
        Ok(value)
    }

    /// Validate decimal/money amount
    pub fn valid_amount(value: &rust_decimal::Decimal, field: &str) -> ValidationResult<rust_decimal::Decimal> {
        if value.is_sign_negative() {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(field.to_string(), vec![format!("{} cannot be negative", field)]);
                    d
                },
            });
        }
        Ok(*value)
    }
}

/// Enum validation
pub mod enums {
    use super::*;

    /// Validate value is one of allowed options
    pub fn one_of(value: &str, field: &str, allowed: &[&str]) -> ValidationResult<String> {
        let lower = value.to_lowercase();
        if allowed.iter().any(|a| a.to_lowercase() == lower) {
            Ok(lower)
        } else {
            Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(
                        field.to_string(),
                        vec![format!(
                            "{} must be one of: {}",
                            field,
                            allowed.join(", ")
                        )],
                    );
                    d
                },
            })
        }
    }

    /// Validate optional enum value
    pub fn one_of_optional(
        value: &Option<String>,
        field: &str,
        allowed: &[&str],
    ) -> ValidationResult<Option<String>> {
        match value {
            Some(v) => Ok(Some(one_of(v, field, allowed)?)),
            None => Ok(None),
        }
    }
}

/// Date/time validation
pub mod datetime {
    use super::*;
    use chrono::{DateTime, NaiveDate, Utc};

    /// Validate date is not in the past
    pub fn not_past(value: &DateTime<Utc>, field: &str) -> ValidationResult<DateTime<Utc>> {
        if *value < Utc::now() {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(field.to_string(), vec![format!("{} cannot be in the past", field)]);
                    d
                },
            });
        }
        Ok(*value)
    }

    /// Validate date range (from before to)
    pub fn valid_range(
        from: &DateTime<Utc>,
        to: &DateTime<Utc>,
        from_field: &str,
        to_field: &str,
    ) -> ValidationResult<()> {
        if from > to {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(
                        from_field.to_string(),
                        vec![format!("{} must be before {}", from_field, to_field)],
                    );
                    d
                },
            });
        }
        Ok(())
    }
}

/// Collection validation
pub mod collection {
    use super::*;

    /// Validate non-empty collection
    pub fn not_empty<T>(value: &[T], field: &str) -> ValidationResult<()> {
        if value.is_empty() {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(field.to_string(), vec![format!("{} cannot be empty", field)]);
                    d
                },
            });
        }
        Ok(())
    }

    /// Validate collection max size
    pub fn max_size<T>(value: &[T], field: &str, max: usize) -> ValidationResult<()> {
        if value.len() > max {
            return Err(AppError::ValidationError {
                details: {
                    let mut d = HashMap::new();
                    d.insert(
                        field.to_string(),
                        vec![format!("{} cannot have more than {} items", field, max)],
                    );
                    d
                },
            });
        }
        Ok(())
    }
}

/// Validator builder for complex validations
pub struct Validator {
    builder: ValidationBuilder,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            builder: ValidationBuilder::new(),
        }
    }

    /// Add error for a field
    pub fn error(mut self, field: &str, message: &str) -> Self {
        self.builder = self.builder.error(field, message);
        self
    }

    /// Add error if condition is true
    pub fn error_if(self, condition: bool, field: &str, message: &str) -> Self {
        if condition {
            self.error(field, message)
        } else {
            self
        }
    }

    /// Validate required string
    pub fn required_string(self, value: &Option<String>, field: &str) -> Self {
        match value {
            Some(s) if !s.trim().is_empty() => self,
            Some(_) => self.error(field, &format!("{} cannot be empty", field)),
            None => self.error(field, &format!("{} is required", field)),
        }
    }

    /// Validate email format
    pub fn email(self, value: &Option<String>, field: &str) -> Self {
        match value {
            Some(e) if !e.trim().is_empty() => {
                if email::validate(e, field).is_err() {
                    self.error(field, "Invalid email format")
                } else {
                    self
                }
            }
            _ => self,
        }
    }

    /// Validate max length
    pub fn max_length(self, value: &Option<String>, field: &str, max: usize) -> Self {
        match value {
            Some(s) if s.len() > max => {
                self.error(field, &format!("{} must be {} characters or less", field, max))
            }
            _ => self,
        }
    }

    /// Validate min length
    pub fn min_length(self, value: &Option<String>, field: &str, min: usize) -> Self {
        match value {
            Some(s) if s.trim().len() < min => {
                self.error(field, &format!("{} must be at least {} characters", field, min))
            }
            _ => self,
        }
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        !self.builder.has_errors()
    }

    /// Finish validation, returning error if any
    pub fn finish(self) -> ValidationResult<()> {
        match self.builder.build() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    /// Finish with a value if valid
    pub fn finish_with<T>(self, value: T) -> ValidationResult<Validated<T>> {
        self.finish()?;
        Ok(Validated(value))
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

/// Common ticket status values
pub const TICKET_STATUSES: &[&str] = &[
    "new", "open", "in_progress", "pending", "waiting_on_client",
    "scheduled", "resolved", "closed", "cancelled"
];

/// Common ticket priority values
pub const TICKET_PRIORITIES: &[&str] = &["low", "medium", "high", "critical", "urgent"];

/// Common invoice statuses
pub const INVOICE_STATUSES: &[&str] = &["draft", "sent", "viewed", "partial", "paid", "overdue", "cancelled"];

/// Common asset statuses
pub const ASSET_STATUSES: &[&str] = &["active", "inactive", "retired", "maintenance", "disposed"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_string() {
        assert!(string::required(&Some("hello".to_string()), "name").is_ok());
        assert!(string::required(&Some("  ".to_string()), "name").is_err());
        assert!(string::required(&None, "name").is_err());
    }

    #[test]
    fn test_email_validation() {
        assert!(email::validate("test@example.com", "email").is_ok());
        assert!(email::validate("invalid", "email").is_err());
        assert!(email::validate("no@domain", "email").is_err());
    }

    #[test]
    fn test_enum_validation() {
        assert!(enums::one_of("high", "priority", TICKET_PRIORITIES).is_ok());
        assert!(enums::one_of("invalid", "priority", TICKET_PRIORITIES).is_err());
    }

    #[test]
    fn test_validator_builder() {
        let result = Validator::new()
            .required_string(&Some("test".to_string()), "name")
            .email(&Some("test@example.com".to_string()), "email")
            .finish();
        assert!(result.is_ok());

        let result = Validator::new()
            .required_string(&None, "name")
            .email(&Some("invalid".to_string()), "email")
            .finish();
        assert!(result.is_err());
    }
}
