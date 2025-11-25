// Workflow Triggers - Event types that can trigger workflow execution

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of events that can trigger workflows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    // Ticket triggers
    TicketCreated,
    TicketUpdated,
    TicketStatusChanged,
    TicketAssigned,
    TicketPriorityChanged,
    TicketCommentAdded,
    TicketEscalated,

    // SLA triggers
    SlaBreach,
    SlaWarning,

    // Client triggers
    ClientCreated,
    ClientUpdated,

    // Invoice triggers
    InvoiceCreated,
    InvoiceSent,
    InvoiceOverdue,
    PaymentReceived,

    // Time tracking triggers
    TimeEntryCreated,
    TimeEntryUpdated,

    // Asset triggers
    AssetCreated,
    AssetUpdated,
    WarrantyExpiring,

    // Schedule triggers
    Scheduled,
    Recurring,

    // Integration triggers
    EmailReceived,
    WebhookReceived,

    // Custom triggers
    Manual,
    ApiCall,
}

/// Payload for trigger events
pub type EventPayload = serde_json::Value;

/// A trigger event that can initiate workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvent {
    pub event_id: Uuid,
    pub trigger_type: TriggerType,
    pub payload: EventPayload,
    pub source: EventSource,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
}

/// Source of the trigger event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSource {
    System,
    User(Uuid),
    Api,
    Email,
    Webhook,
    Scheduler,
    Integration(String),
}

impl TriggerEvent {
    /// Create a new trigger event
    pub fn new(trigger_type: TriggerType, payload: EventPayload, source: EventSource) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            trigger_type,
            payload,
            source,
            timestamp: Utc::now(),
            correlation_id: None,
        }
    }

    /// Create a ticket created event
    pub fn ticket_created(
        ticket_id: Uuid,
        client_id: Uuid,
        subject: &str,
        priority: &str,
        status: &str,
        category_id: Option<Uuid>,
        source: EventSource,
    ) -> Self {
        Self::new(
            TriggerType::TicketCreated,
            serde_json::json!({
                "ticket_id": ticket_id,
                "client_id": client_id,
                "subject": subject,
                "priority": priority,
                "status": status,
                "category_id": category_id
            }),
            source,
        )
    }

    /// Create a ticket status changed event
    pub fn ticket_status_changed(
        ticket_id: Uuid,
        old_status: &str,
        new_status: &str,
        changed_by: Uuid,
    ) -> Self {
        Self::new(
            TriggerType::TicketStatusChanged,
            serde_json::json!({
                "ticket_id": ticket_id,
                "old_status": old_status,
                "new_status": new_status,
                "changed_by": changed_by
            }),
            EventSource::User(changed_by),
        )
    }

    /// Create a ticket assigned event
    pub fn ticket_assigned(
        ticket_id: Uuid,
        old_assignee: Option<Uuid>,
        new_assignee: Uuid,
        assigned_by: Uuid,
    ) -> Self {
        Self::new(
            TriggerType::TicketAssigned,
            serde_json::json!({
                "ticket_id": ticket_id,
                "old_assignee": old_assignee,
                "new_assignee": new_assignee,
                "assigned_by": assigned_by
            }),
            EventSource::User(assigned_by),
        )
    }

    /// Create an SLA breach event
    pub fn sla_breach(
        ticket_id: Uuid,
        breach_type: &str, // "response" or "resolution"
        breach_minutes: i32,
        assigned_to: Option<Uuid>,
    ) -> Self {
        Self::new(
            TriggerType::SlaBreach,
            serde_json::json!({
                "ticket_id": ticket_id,
                "breach_type": breach_type,
                "breach_minutes": breach_minutes,
                "assigned_to": assigned_to
            }),
            EventSource::System,
        )
    }

    /// Create an SLA warning event
    pub fn sla_warning(
        ticket_id: Uuid,
        breach_type: &str,
        minutes_remaining: i32,
    ) -> Self {
        Self::new(
            TriggerType::SlaWarning,
            serde_json::json!({
                "ticket_id": ticket_id,
                "breach_type": breach_type,
                "minutes_remaining": minutes_remaining
            }),
            EventSource::System,
        )
    }

    /// Create an invoice overdue event
    pub fn invoice_overdue(
        invoice_id: Uuid,
        client_id: Uuid,
        amount: rust_decimal::Decimal,
        days_overdue: i32,
    ) -> Self {
        Self::new(
            TriggerType::InvoiceOverdue,
            serde_json::json!({
                "invoice_id": invoice_id,
                "client_id": client_id,
                "amount": amount.to_string(),
                "days_overdue": days_overdue
            }),
            EventSource::System,
        )
    }

    /// Create a client created event
    pub fn client_created(
        client_id: Uuid,
        client_name: &str,
        created_by: Uuid,
    ) -> Self {
        Self::new(
            TriggerType::ClientCreated,
            serde_json::json!({
                "client_id": client_id,
                "client_name": client_name,
                "created_by": created_by
            }),
            EventSource::User(created_by),
        )
    }

    /// Create an email received event
    pub fn email_received(
        from_address: &str,
        subject: &str,
        body_preview: &str,
        client_id: Option<Uuid>,
        ticket_id: Option<Uuid>,
    ) -> Self {
        Self::new(
            TriggerType::EmailReceived,
            serde_json::json!({
                "from_address": from_address,
                "subject": subject,
                "body_preview": body_preview,
                "client_id": client_id,
                "ticket_id": ticket_id
            }),
            EventSource::Email,
        )
    }

    /// Create a scheduled event
    pub fn scheduled(schedule_name: &str, schedule_config: serde_json::Value) -> Self {
        Self::new(
            TriggerType::Scheduled,
            serde_json::json!({
                "schedule_name": schedule_name,
                "config": schedule_config
            }),
            EventSource::Scheduler,
        )
    }

    /// Create a webhook received event
    pub fn webhook_received(
        webhook_id: &str,
        source: &str,
        payload: serde_json::Value,
    ) -> Self {
        Self::new(
            TriggerType::WebhookReceived,
            serde_json::json!({
                "webhook_id": webhook_id,
                "source": source,
                "payload": payload
            }),
            EventSource::Webhook,
        )
    }

    /// Create a warranty expiring event
    pub fn warranty_expiring(
        asset_id: Uuid,
        asset_name: &str,
        client_id: Uuid,
        days_until_expiry: i32,
    ) -> Self {
        Self::new(
            TriggerType::WarrantyExpiring,
            serde_json::json!({
                "asset_id": asset_id,
                "asset_name": asset_name,
                "client_id": client_id,
                "days_until_expiry": days_until_expiry
            }),
            EventSource::System,
        )
    }

    /// Add correlation ID for tracking related events
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_event_creation() {
        let event = TriggerEvent::ticket_created(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test Ticket",
            "high",
            "open",
            None,
            EventSource::System,
        );

        assert_eq!(event.trigger_type, TriggerType::TicketCreated);
        assert!(event.payload.get("subject").is_some());
    }

    #[test]
    fn test_sla_breach_event() {
        let ticket_id = Uuid::new_v4();
        let event = TriggerEvent::sla_breach(
            ticket_id,
            "response",
            30,
            Some(Uuid::new_v4()),
        );

        assert_eq!(event.trigger_type, TriggerType::SlaBreach);
        assert_eq!(event.payload.get("breach_type").unwrap(), "response");
        assert_eq!(event.payload.get("breach_minutes").unwrap(), 30);
    }
}
