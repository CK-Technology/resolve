//! Pagination and query helpers for Resolve API
//!
//! Provides standardized pagination, sorting, and filtering across all endpoints.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Default page size if not specified
pub const DEFAULT_PAGE_SIZE: i64 = 25;
/// Maximum allowed page size
pub const MAX_PAGE_SIZE: i64 = 100;
/// Default page number (1-indexed for API consumers)
pub const DEFAULT_PAGE: i64 = 1;

/// Standard pagination query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    /// Sort field
    pub sort_by: Option<String>,
    /// Sort direction (asc/desc)
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

fn default_page() -> i64 {
    DEFAULT_PAGE
}

fn default_per_page() -> i64 {
    DEFAULT_PAGE_SIZE
}

fn default_sort_order() -> String {
    "desc".to_string()
}

impl PaginationParams {
    /// Get SQL OFFSET value
    pub fn offset(&self) -> i64 {
        let page = self.page.max(1);
        let per_page = self.per_page.clamp(1, MAX_PAGE_SIZE);
        (page - 1) * per_page
    }

    /// Get SQL LIMIT value
    pub fn limit(&self) -> i64 {
        self.per_page.clamp(1, MAX_PAGE_SIZE)
    }

    /// Get sort direction as SQL string
    pub fn sort_direction(&self) -> &str {
        if self.sort_order.to_lowercase() == "asc" {
            "ASC"
        } else {
            "DESC"
        }
    }

    /// Validate and sanitize sort field against allowed fields
    pub fn validated_sort_field(&self, allowed: &[&str], default: &str) -> String {
        self.sort_by
            .as_ref()
            .filter(|s| allowed.contains(&s.as_str()))
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: DEFAULT_PAGE,
            per_page: DEFAULT_PAGE_SIZE,
            sort_by: None,
            sort_order: "desc".to_string(),
        }
    }
}

/// Pagination metadata returned with list responses
#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    /// Current page (1-indexed)
    pub page: i64,
    /// Items per page
    pub per_page: i64,
    /// Total number of items
    pub total: i64,
    /// Total number of pages
    pub total_pages: i64,
    /// Whether there's a next page
    pub has_next: bool,
    /// Whether there's a previous page
    pub has_prev: bool,
}

impl PaginationMeta {
    pub fn new(page: i64, per_page: i64, total: i64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self {
            page,
            per_page,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// Standard paginated response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    /// The actual data items
    pub data: Vec<T>,
    /// Pagination metadata
    pub meta: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, params: &PaginationParams, total: i64) -> Self {
        Self {
            data,
            meta: PaginationMeta::new(params.page, params.limit(), total),
        }
    }
}

/// Search parameters common across entities
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SearchParams {
    /// Free-text search query
    pub q: Option<String>,
    /// Filter by date from (ISO 8601)
    pub from_date: Option<chrono::NaiveDate>,
    /// Filter by date to (ISO 8601)
    pub to_date: Option<chrono::NaiveDate>,
    /// Filter by created after (ISO 8601 datetime)
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    /// Filter by created before (ISO 8601 datetime)
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

impl SearchParams {
    /// Get search pattern for ILIKE queries
    pub fn search_pattern(&self) -> Option<String> {
        self.q.as_ref().map(|q| format!("%{}%", q.trim()))
    }
}

/// Helper trait for counting total records
pub trait Countable {
    fn count_query(&self) -> String;
}

/// SQL query builder helper for dynamic filtering
#[derive(Debug, Default)]
pub struct QueryBuilder {
    conditions: Vec<String>,
    param_count: usize,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start parameter numbering from a specific index
    pub fn with_start_param(start: usize) -> Self {
        Self {
            conditions: Vec::new(),
            param_count: start,
        }
    }

    /// Add a condition (returns the parameter placeholder number)
    pub fn add_condition(&mut self, condition: &str) -> usize {
        self.param_count += 1;
        let full_condition = condition.replace("{}", &format!("${}", self.param_count));
        self.conditions.push(full_condition);
        self.param_count
    }

    /// Add condition only if value is Some
    pub fn add_optional<T>(&mut self, condition: &str, value: &Option<T>) -> Option<usize> {
        if value.is_some() {
            Some(self.add_condition(condition))
        } else {
            None
        }
    }

    /// Get the WHERE clause (empty string if no conditions)
    pub fn where_clause(&self) -> String {
        if self.conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", self.conditions.join(" AND "))
        }
    }

    /// Get the AND clause for appending to existing WHERE
    pub fn and_clause(&self) -> String {
        if self.conditions.is_empty() {
            String::new()
        } else {
            format!("AND {}", self.conditions.join(" AND "))
        }
    }

    /// Get current parameter count
    pub fn param_count(&self) -> usize {
        self.param_count
    }
}

/// Standard list query parameters combining pagination, search, and common filters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    #[serde(flatten)]
    pub search: SearchParams,
    /// Include soft-deleted records
    #[serde(default)]
    pub include_deleted: bool,
    /// Include archived records
    #[serde(default)]
    pub include_archived: bool,
}

/// Ticket-specific list parameters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TicketListParams {
    #[serde(flatten)]
    pub base: ListParams,
    /// Filter by status
    pub status: Option<String>,
    /// Filter by priority
    pub priority: Option<String>,
    /// Filter by assigned user
    pub assigned_to: Option<uuid::Uuid>,
    /// Filter by client
    pub client_id: Option<uuid::Uuid>,
    /// Filter by category
    pub category_id: Option<uuid::Uuid>,
    /// Filter by SLA breach status
    pub sla_breached: Option<bool>,
    /// Filter by queue
    pub queue_id: Option<uuid::Uuid>,
    /// Show only my tickets (requires auth context)
    #[serde(default)]
    pub my_tickets: bool,
    /// Show only unassigned tickets
    #[serde(default)]
    pub unassigned: bool,
}

/// Client-specific list parameters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClientListParams {
    #[serde(flatten)]
    pub base: ListParams,
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Filter by tag
    pub tag: Option<String>,
    /// Filter by category
    pub category: Option<String>,
}

/// Asset-specific list parameters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AssetListParams {
    #[serde(flatten)]
    pub base: ListParams,
    /// Filter by client
    pub client_id: Option<uuid::Uuid>,
    /// Filter by asset type
    pub asset_type_id: Option<uuid::Uuid>,
    /// Filter by status
    pub status: Option<String>,
    /// Filter by warranty status
    pub warranty_status: Option<String>,
}

/// Invoice-specific list parameters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct InvoiceListParams {
    #[serde(flatten)]
    pub base: ListParams,
    /// Filter by client
    pub client_id: Option<uuid::Uuid>,
    /// Filter by status
    pub status: Option<String>,
    /// Filter by overdue only
    #[serde(default)]
    pub overdue_only: bool,
}

/// Time entry list parameters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TimeEntryListParams {
    #[serde(flatten)]
    pub base: ListParams,
    /// Filter by user
    pub user_id: Option<uuid::Uuid>,
    /// Filter by client
    pub client_id: Option<uuid::Uuid>,
    /// Filter by ticket
    pub ticket_id: Option<uuid::Uuid>,
    /// Filter by billable status
    pub billable: Option<bool>,
    /// Filter by invoiced status
    pub invoiced: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_offset() {
        let params = PaginationParams {
            page: 3,
            per_page: 25,
            ..Default::default()
        };
        assert_eq!(params.offset(), 50);
        assert_eq!(params.limit(), 25);
    }

    #[test]
    fn test_pagination_clamps() {
        let params = PaginationParams {
            page: -1,
            per_page: 500,
            ..Default::default()
        };
        assert_eq!(params.offset(), 0); // page clamped to 1
        assert_eq!(params.limit(), MAX_PAGE_SIZE); // per_page clamped to max
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(2, 25, 100);
        assert_eq!(meta.total_pages, 4);
        assert!(meta.has_next);
        assert!(meta.has_prev);
    }

    #[test]
    fn test_query_builder() {
        let mut qb = QueryBuilder::new();
        qb.add_condition("status = {}");
        qb.add_condition("client_id = {}");

        assert_eq!(qb.where_clause(), "WHERE status = $1 AND client_id = $2");
    }

    #[test]
    fn test_search_pattern() {
        let params = SearchParams {
            q: Some("  test  ".to_string()),
            ..Default::default()
        };
        assert_eq!(params.search_pattern(), Some("%test%".to_string()));
    }
}
