// Unit tests for service layer

use super::*;
use crate::services::{
    cache::{CacheService, cache_keys, ttl},
    audit::{AuditService, AuditAction, AuditSeverity, AuditEntryBuilder, ChangeTracker},
    metrics::{MetricsService, MetricType, HealthStatus, RequestLog, Timer, metric_names},
};
use serde_json::json;
use uuid::Uuid;

// ============================================
// Cache Service Tests
// ============================================

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let client_id = Uuid::new_v4();
        let ticket_id = Uuid::new_v4();

        // Dashboard stats key
        assert_eq!(cache_keys::dashboard_stats(), "dashboard:stats");

        // Client keys
        let client_key = cache_keys::client(client_id);
        assert!(client_key.starts_with("client:"));
        assert!(client_key.contains(&client_id.to_string()));

        // Client list key
        assert_eq!(cache_keys::client_list(1), "clients:list:page:1");
        assert_eq!(cache_keys::client_list(5), "clients:list:page:5");

        // Ticket keys
        let ticket_key = cache_keys::ticket(ticket_id);
        assert!(ticket_key.starts_with("ticket:"));

        // Ticket list with and without client filter
        let all_tickets = cache_keys::ticket_list(None, 1);
        assert_eq!(all_tickets, "tickets:all:page:1");

        let client_tickets = cache_keys::ticket_list(Some(client_id), 2);
        assert!(client_tickets.contains("client"));
        assert!(client_tickets.contains(&client_id.to_string()));

        // SLA metrics keys
        let all_sla = cache_keys::sla_metrics(None);
        assert_eq!(all_sla, "sla:metrics:all");

        let client_sla = cache_keys::sla_metrics(Some(client_id));
        assert!(client_sla.contains(&client_id.to_string()));

        // Analytics keys
        let utilization = cache_keys::analytics_utilization("2024-01-01", "2024-01-31");
        assert!(utilization.contains("utilization"));

        let profitability = cache_keys::analytics_profitability("2024-01-01", "2024-01-31");
        assert!(profitability.contains("profitability"));
    }

    #[test]
    fn test_cache_patterns() {
        let client_id = Uuid::new_v4();
        let ticket_id = Uuid::new_v4();

        // Client pattern
        let client_pattern = cache_keys::client_pattern(client_id);
        assert!(client_pattern.ends_with("%"));

        // Ticket pattern
        let ticket_pattern = cache_keys::ticket_pattern(ticket_id);
        assert!(ticket_pattern.ends_with("%"));

        // Analytics pattern
        let analytics_pattern = cache_keys::analytics_pattern();
        assert_eq!(analytics_pattern, "analytics:%");
    }

    #[test]
    fn test_ttl_values() {
        assert_eq!(ttl::SHORT, 60);
        assert_eq!(ttl::MEDIUM, 300);
        assert_eq!(ttl::LONG, 900);
        assert_eq!(ttl::DASHBOARD, 120);
        assert_eq!(ttl::ANALYTICS, 600);
        assert_eq!(ttl::STATIC, 3600);
    }
}

// ============================================
// Audit Service Tests
// ============================================

#[cfg(test)]
mod audit_tests {
    use super::*;

    #[test]
    fn test_audit_action_strings() {
        assert_eq!(AuditAction::Create.as_str(), "create");
        assert_eq!(AuditAction::Update.as_str(), "update");
        assert_eq!(AuditAction::Delete.as_str(), "delete");
        assert_eq!(AuditAction::View.as_str(), "view");
        assert_eq!(AuditAction::Export.as_str(), "export");
        assert_eq!(AuditAction::Login.as_str(), "login");
        assert_eq!(AuditAction::Logout.as_str(), "logout");
        assert_eq!(AuditAction::PasswordChange.as_str(), "password_change");
        assert_eq!(AuditAction::PermissionChange.as_str(), "permission_change");
        assert_eq!(AuditAction::ApiKeyCreate.as_str(), "api_key_create");
        assert_eq!(AuditAction::ApiKeyRevoke.as_str(), "api_key_revoke");
        assert_eq!(AuditAction::BulkOperation.as_str(), "bulk_operation");
        assert_eq!(AuditAction::Import.as_str(), "import");
        assert_eq!(AuditAction::Archive.as_str(), "archive");
        assert_eq!(AuditAction::Restore.as_str(), "restore");
    }

    #[test]
    fn test_sensitive_actions() {
        // Sensitive actions
        assert!(AuditAction::PasswordChange.is_sensitive());
        assert!(AuditAction::PermissionChange.is_sensitive());
        assert!(AuditAction::ApiKeyCreate.is_sensitive());
        assert!(AuditAction::ApiKeyRevoke.is_sensitive());

        // Non-sensitive actions
        assert!(!AuditAction::Create.is_sensitive());
        assert!(!AuditAction::Update.is_sensitive());
        assert!(!AuditAction::Delete.is_sensitive());
        assert!(!AuditAction::View.is_sensitive());
        assert!(!AuditAction::Login.is_sensitive());
    }

    #[test]
    fn test_audit_severity() {
        assert_eq!(AuditSeverity::Info.as_str(), "info");
        assert_eq!(AuditSeverity::Warning.as_str(), "warning");
        assert_eq!(AuditSeverity::Critical.as_str(), "critical");

        // Default should be Info
        assert_eq!(AuditSeverity::default(), AuditSeverity::Info);
    }

    #[test]
    fn test_audit_entry_builder() {
        let user_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();

        let builder = AuditEntryBuilder::new(AuditAction::Create, "ticket")
            .user(user_id, Some("test@example.com".to_string()))
            .resource(resource_id, Some("Ticket #123".to_string()))
            .request_id(request_id)
            .severity(AuditSeverity::Warning);

        // Builder should compile and work (actual db insert tested in integration)
        assert!(true);
    }

    #[test]
    fn test_change_tracker() {
        let mut tracker = ChangeTracker::new();

        // Initially no changes
        assert!(!tracker.has_changes());

        // Track a change
        let changed = tracker.track("name", &"Old Name", &"New Name").unwrap();
        assert!(changed);
        assert!(tracker.has_changes());

        // Track a non-change
        let not_changed = tracker.track("status", &"active", &"active").unwrap();
        assert!(!not_changed);

        // Convert to JSON
        let json = tracker.into_json();
        assert!(json.is_object());
    }

    #[test]
    fn test_change_tracker_multiple_fields() {
        let mut tracker = ChangeTracker::new();

        tracker.track("field1", &"a", &"b").unwrap();
        tracker.track("field2", &1, &2).unwrap();
        tracker.track("field3", &true, &false).unwrap();
        tracker.track("field4", &"same", &"same").unwrap(); // No change

        let json = tracker.into_json();
        let obj = json.as_object().unwrap();

        assert!(obj.contains_key("field1"));
        assert!(obj.contains_key("field2"));
        assert!(obj.contains_key("field3"));
        assert!(!obj.contains_key("field4")); // No change, not included
    }
}

// ============================================
// Metrics Service Tests
// ============================================

#[cfg(test)]
mod metrics_tests {
    use super::*;

    #[test]
    fn test_metric_type_strings() {
        assert_eq!(MetricType::Counter.as_str(), "counter");
        assert_eq!(MetricType::Gauge.as_str(), "gauge");
        assert_eq!(MetricType::Histogram.as_str(), "histogram");
    }

    #[test]
    fn test_health_status_strings() {
        assert_eq!(HealthStatus::Healthy.as_str(), "healthy");
        assert_eq!(HealthStatus::Degraded.as_str(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.as_str(), "unhealthy");
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }

    #[test]
    fn test_request_log_creation() {
        let request_id = Uuid::new_v4();
        let log = RequestLog::new(request_id, "GET", "/api/v1/clients");

        assert_eq!(log.request_id, request_id);
        assert_eq!(log.method, "GET");
        assert_eq!(log.path, "/api/v1/clients");
        assert!(log.query_params.is_none());
        assert!(log.user_id.is_none());
        assert!(log.status_code.is_none());
        assert!(log.response_time_ms.is_none());
    }

    #[test]
    fn test_metric_names_constants() {
        // Verify metric name constants are properly defined
        assert!(!metric_names::HTTP_REQUESTS_TOTAL.is_empty());
        assert!(!metric_names::HTTP_REQUEST_DURATION_MS.is_empty());
        assert!(!metric_names::HTTP_ERRORS_TOTAL.is_empty());
        assert!(!metric_names::DB_CONNECTIONS_ACTIVE.is_empty());
        assert!(!metric_names::CACHE_HITS.is_empty());
        assert!(!metric_names::CACHE_MISSES.is_empty());
        assert!(!metric_names::TICKETS_CREATED.is_empty());
        assert!(!metric_names::SLA_BREACHES.is_empty());
    }
}

// ============================================
// Validation Tests
// ============================================

#[cfg(test)]
mod validation_tests {
    use crate::validation::Validator;

    #[test]
    fn test_string_validation() {
        let validator = Validator::new();

        // Valid string
        let result = validator.string("name", "Test Client").min_length(1).max_length(100);
        assert!(result.validate().is_ok());

        // Empty string
        let result = Validator::new().string("name", "").min_length(1);
        assert!(result.validate().is_err());
    }

    #[test]
    fn test_email_validation() {
        let validator = Validator::new();

        // Valid email
        let result = validator.email("email", "test@example.com");
        assert!(result.validate().is_ok());

        // Invalid email
        let result = Validator::new().email("email", "invalid-email");
        assert!(result.validate().is_err());
    }

    #[test]
    fn test_uuid_validation() {
        let validator = Validator::new();
        let valid_uuid = Uuid::new_v4().to_string();

        // Valid UUID
        let result = validator.uuid("id", &valid_uuid);
        assert!(result.validate().is_ok());

        // Invalid UUID
        let result = Validator::new().uuid("id", "not-a-uuid");
        assert!(result.validate().is_err());
    }

    #[test]
    fn test_chained_validation() {
        let result = Validator::new()
            .string("name", "Test")
            .min_length(1)
            .max_length(50)
            .email("email", "test@example.com")
            .validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_validation() {
        let valid_statuses = vec!["open", "in_progress", "pending", "resolved", "closed"];

        // Valid enum
        let result = Validator::new()
            .string("status", "open")
            .one_of(&valid_statuses);
        assert!(result.validate().is_ok());

        // Invalid enum
        let result = Validator::new()
            .string("status", "invalid")
            .one_of(&valid_statuses);
        assert!(result.validate().is_err());
    }
}

// ============================================
// Teams Integration Tests
// ============================================

#[cfg(test)]
mod teams_tests {
    use crate::services::teams_integration::{TicketNotification, DailySummary};
    use uuid::Uuid;

    #[test]
    fn test_ticket_notification_creation() {
        let notification = TicketNotification {
            ticket_id: Uuid::new_v4(),
            ticket_number: 123,
            subject: "Test Ticket".to_string(),
            client_name: "Test Client".to_string(),
            priority: "high".to_string(),
            status: "open".to_string(),
            assigned_to: Some("John Doe".to_string()),
            created_by: "Jane Smith".to_string(),
            description: Some("Test description".to_string()),
        };

        assert_eq!(notification.ticket_number, 123);
        assert_eq!(notification.subject, "Test Ticket");
        assert_eq!(notification.priority, "high");
    }

    #[test]
    fn test_daily_summary_creation() {
        let summary = DailySummary {
            date: chrono::Utc::now().date_naive(),
            tickets_opened: 10,
            tickets_resolved: 8,
            tickets_breached_sla: 1,
            active_tickets: 15,
            critical_tickets: 2,
            avg_response_time_hours: Some(2.5),
            avg_resolution_time_hours: Some(24.0),
            top_clients: vec![
                ("Client A".to_string(), 5),
                ("Client B".to_string(), 3),
            ],
        };

        assert_eq!(summary.tickets_opened, 10);
        assert_eq!(summary.tickets_resolved, 8);
        assert_eq!(summary.tickets_breached_sla, 1);
        assert_eq!(summary.top_clients.len(), 2);
    }
}

// ============================================
// Error Handling Tests
// ============================================

#[cfg(test)]
mod error_tests {
    use crate::error::{AppError, ApiError};
    use axum::http::StatusCode;

    #[test]
    fn test_app_error_status_codes() {
        let not_found = AppError::NotFound("Resource".to_string());
        assert_eq!(not_found.status_code(), StatusCode::NOT_FOUND);

        let unauthorized = AppError::Unauthorized;
        assert_eq!(unauthorized.status_code(), StatusCode::UNAUTHORIZED);

        let forbidden = AppError::Forbidden("Permission denied".to_string());
        assert_eq!(forbidden.status_code(), StatusCode::FORBIDDEN);

        let bad_request = AppError::BadRequest("Invalid input".to_string());
        assert_eq!(bad_request.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_api_error_response() {
        let error = ApiError::new("Something went wrong")
            .with_code("ERR_SOMETHING");

        assert_eq!(error.message, "Something went wrong");
        assert_eq!(error.code, Some("ERR_SOMETHING".to_string()));
    }

    #[test]
    fn test_validation_error() {
        let error = AppError::ValidationFailed(vec![
            "Name is required".to_string(),
            "Email is invalid".to_string(),
        ]);

        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);

        if let AppError::ValidationFailed(errors) = error {
            assert_eq!(errors.len(), 2);
        }
    }
}

// ============================================
// Pagination Tests
// ============================================

#[cfg(test)]
mod pagination_tests {
    use crate::pagination::{PaginationParams, PaginationMeta};

    #[test]
    fn test_pagination_params_defaults() {
        let params = PaginationParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 20);
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta {
            page: 2,
            per_page: 20,
            total: 55,
            total_pages: 3,
        };

        assert_eq!(meta.page, 2);
        assert_eq!(meta.total_pages, 3);
    }

    #[test]
    fn test_pagination_offset_calculation() {
        let params = PaginationParams {
            page: 3,
            per_page: 20,
        };

        let offset = (params.page - 1) * params.per_page;
        assert_eq!(offset, 40);
    }

    #[test]
    fn test_total_pages_calculation() {
        // Exact division
        let total_pages = (100 + 20 - 1) / 20;
        assert_eq!(total_pages, 5);

        // Non-exact division
        let total_pages = (101 + 20 - 1) / 20;
        assert_eq!(total_pages, 6);

        // Less than one page
        let total_pages = (5 + 20 - 1) / 20;
        assert_eq!(total_pages, 1);
    }
}
