// Workflow Conditions - Conditional logic for workflow execution

use serde::{Deserialize, Serialize};

/// A single condition to evaluate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Field name to evaluate (supports dot notation for nested fields)
    pub field: String,
    /// Operator for comparison
    pub operator: String,
    /// Value to compare against
    pub value: serde_json::Value,
}

/// Group of conditions with AND/OR logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionGroup {
    /// Logic operator: "AND" or "OR"
    pub logic: String,
    /// List of conditions in this group
    pub conditions: Vec<Condition>,
    /// Nested condition groups for complex logic
    #[serde(default)]
    pub groups: Vec<ConditionGroup>,
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    // Equality
    Equals,
    NotEquals,

    // String operations
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Regex,

    // Numeric comparisons
    GreaterThan,
    GreaterThanOrEquals,
    LessThan,
    LessThanOrEquals,

    // Array operations
    In,
    NotIn,
    ArrayContains,

    // Null/Empty checks
    IsNull,
    IsNotNull,
    IsEmpty,
    IsNotEmpty,

    // Boolean
    IsTrue,
    IsFalse,

    // Date comparisons
    DateBefore,
    DateAfter,
    DateBetween,
}

/// Field type for condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: Option<serde_json::Value>,
    pub case_sensitive: bool,
}

impl FieldCondition {
    pub fn new(field: &str, operator: ConditionOperator, value: Option<serde_json::Value>) -> Self {
        Self {
            field: field.to_string(),
            operator,
            value,
            case_sensitive: false,
        }
    }

    pub fn equals(field: &str, value: serde_json::Value) -> Self {
        Self::new(field, ConditionOperator::Equals, Some(value))
    }

    pub fn not_equals(field: &str, value: serde_json::Value) -> Self {
        Self::new(field, ConditionOperator::NotEquals, Some(value))
    }

    pub fn contains(field: &str, value: &str) -> Self {
        Self::new(field, ConditionOperator::Contains, Some(serde_json::Value::String(value.to_string())))
    }

    pub fn is_null(field: &str) -> Self {
        Self::new(field, ConditionOperator::IsNull, None)
    }

    pub fn is_not_null(field: &str) -> Self {
        Self::new(field, ConditionOperator::IsNotNull, None)
    }

    pub fn greater_than(field: &str, value: f64) -> Self {
        Self::new(field, ConditionOperator::GreaterThan, Some(serde_json::json!(value)))
    }

    pub fn less_than(field: &str, value: f64) -> Self {
        Self::new(field, ConditionOperator::LessThan, Some(serde_json::json!(value)))
    }

    pub fn in_list(field: &str, values: Vec<serde_json::Value>) -> Self {
        Self::new(field, ConditionOperator::In, Some(serde_json::Value::Array(values)))
    }

    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }
}

impl Condition {
    pub fn new(field: &str, operator: &str, value: serde_json::Value) -> Self {
        Self {
            field: field.to_string(),
            operator: operator.to_string(),
            value,
        }
    }

    pub fn equals(field: &str, value: serde_json::Value) -> Self {
        Self::new(field, "equals", value)
    }

    pub fn not_equals(field: &str, value: serde_json::Value) -> Self {
        Self::new(field, "not_equals", value)
    }

    pub fn contains(field: &str, value: &str) -> Self {
        Self::new(field, "contains", serde_json::Value::String(value.to_string()))
    }

    pub fn starts_with(field: &str, value: &str) -> Self {
        Self::new(field, "starts_with", serde_json::Value::String(value.to_string()))
    }

    pub fn ends_with(field: &str, value: &str) -> Self {
        Self::new(field, "ends_with", serde_json::Value::String(value.to_string()))
    }

    pub fn greater_than(field: &str, value: f64) -> Self {
        Self::new(field, "greater_than", serde_json::json!(value))
    }

    pub fn less_than(field: &str, value: f64) -> Self {
        Self::new(field, "less_than", serde_json::json!(value))
    }

    pub fn is_null(field: &str) -> Self {
        Self::new(field, "is_null", serde_json::Value::Null)
    }

    pub fn is_not_null(field: &str) -> Self {
        Self::new(field, "is_not_null", serde_json::Value::Null)
    }

    pub fn in_list(field: &str, values: Vec<serde_json::Value>) -> Self {
        Self::new(field, "in", serde_json::Value::Array(values))
    }

    pub fn regex(field: &str, pattern: &str) -> Self {
        Self::new(field, "regex", serde_json::Value::String(pattern.to_string()))
    }
}

impl ConditionGroup {
    pub fn and(conditions: Vec<Condition>) -> Self {
        Self {
            logic: "AND".to_string(),
            conditions,
            groups: Vec::new(),
        }
    }

    pub fn or(conditions: Vec<Condition>) -> Self {
        Self {
            logic: "OR".to_string(),
            conditions,
            groups: Vec::new(),
        }
    }

    pub fn with_nested_group(mut self, group: ConditionGroup) -> Self {
        self.groups.push(group);
        self
    }

    pub fn add_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }
}

/// Common condition presets for MSP workflows
pub mod presets {
    use super::*;

    /// Condition for critical priority tickets
    pub fn critical_priority() -> Condition {
        Condition::equals("priority", serde_json::json!("critical"))
    }

    /// Condition for high priority tickets
    pub fn high_priority() -> Condition {
        Condition::in_list("priority", vec![
            serde_json::json!("critical"),
            serde_json::json!("high"),
        ])
    }

    /// Condition for unassigned tickets
    pub fn unassigned() -> Condition {
        Condition::is_null("assigned_to")
    }

    /// Condition for VIP clients
    pub fn vip_client() -> Condition {
        Condition::equals("client.is_vip", serde_json::json!(true))
    }

    /// Condition for tickets from a specific category
    pub fn category(category_id: uuid::Uuid) -> Condition {
        Condition::equals("category_id", serde_json::json!(category_id.to_string()))
    }

    /// Condition for response SLA breach
    pub fn response_breach() -> Condition {
        Condition::equals("breach_type", serde_json::json!("response"))
    }

    /// Condition for resolution SLA breach
    pub fn resolution_breach() -> Condition {
        Condition::equals("breach_type", serde_json::json!("resolution"))
    }

    /// Condition for tickets open more than N hours
    pub fn open_longer_than_hours(hours: i32) -> Condition {
        Condition::greater_than("hours_open", hours as f64)
    }

    /// Condition for invoice amount above threshold
    pub fn invoice_above(amount: f64) -> Condition {
        Condition::greater_than("amount", amount)
    }

    /// Condition for overdue invoices
    pub fn invoice_overdue_days(days: i32) -> Condition {
        Condition::greater_than("days_overdue", days as f64)
    }

    /// Condition for specific ticket status
    pub fn status(status: &str) -> Condition {
        Condition::equals("status", serde_json::json!(status))
    }

    /// Condition for ticket subject containing keyword
    pub fn subject_contains(keyword: &str) -> Condition {
        Condition::contains("subject", keyword)
    }

    /// Condition for email from specific domain
    pub fn email_from_domain(domain: &str) -> Condition {
        Condition::ends_with("from_address", domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_builder() {
        let condition = Condition::equals("priority", serde_json::json!("high"));
        assert_eq!(condition.field, "priority");
        assert_eq!(condition.operator, "equals");
    }

    #[test]
    fn test_condition_group() {
        let group = ConditionGroup::and(vec![
            Condition::equals("priority", serde_json::json!("high")),
            Condition::is_null("assigned_to"),
        ]);

        assert_eq!(group.logic, "AND");
        assert_eq!(group.conditions.len(), 2);
    }

    #[test]
    fn test_nested_groups() {
        let inner = ConditionGroup::or(vec![
            Condition::equals("priority", serde_json::json!("critical")),
            Condition::equals("priority", serde_json::json!("high")),
        ]);

        let outer = ConditionGroup::and(vec![
            Condition::is_null("assigned_to"),
        ]).with_nested_group(inner);

        assert_eq!(outer.groups.len(), 1);
    }

    #[test]
    fn test_presets() {
        let crit = presets::critical_priority();
        assert_eq!(crit.field, "priority");

        let unassigned = presets::unassigned();
        assert_eq!(unassigned.operator, "is_null");
    }
}
