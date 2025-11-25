//! Microsoft Teams Integration Service
//!
//! Provides webhook-based notifications to Microsoft Teams channels.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Teams Adaptive Card for rich notifications
#[derive(Debug, Clone, Serialize)]
pub struct TeamsAdaptiveCard {
    #[serde(rename = "type")]
    pub card_type: String,
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub body: Vec<TeamsCardElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<TeamsCardAction>>,
}

impl Default for TeamsAdaptiveCard {
    fn default() -> Self {
        Self {
            card_type: "AdaptiveCard".to_string(),
            schema: "http://adaptivecards.io/schemas/adaptive-card.json".to_string(),
            version: "1.4".to_string(),
            body: vec![],
            actions: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum TeamsCardElement {
    TextBlock {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        weight: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        color: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        wrap: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        spacing: Option<String>,
    },
    ColumnSet {
        columns: Vec<TeamsColumn>,
        #[serde(skip_serializing_if = "Option::is_none")]
        spacing: Option<String>,
    },
    FactSet {
        facts: Vec<TeamsFact>,
        #[serde(skip_serializing_if = "Option::is_none")]
        spacing: Option<String>,
    },
    Container {
        items: Vec<TeamsCardElement>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        spacing: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamsColumn {
    pub width: String,
    pub items: Vec<TeamsCardElement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamsFact {
    pub title: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum TeamsCardAction {
    #[serde(rename = "Action.OpenUrl")]
    OpenUrl {
        title: String,
        url: String,
    },
    #[serde(rename = "Action.Submit")]
    Submit {
        title: String,
        data: serde_json::Value,
    },
}

/// Wrapper for Teams webhook payload
#[derive(Debug, Clone, Serialize)]
pub struct TeamsWebhookPayload {
    #[serde(rename = "type")]
    pub payload_type: String,
    pub attachments: Vec<TeamsAttachment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamsAttachment {
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub content: TeamsAdaptiveCard,
}

impl TeamsWebhookPayload {
    pub fn from_card(card: TeamsAdaptiveCard) -> Self {
        Self {
            payload_type: "message".to_string(),
            attachments: vec![TeamsAttachment {
                content_type: "application/vnd.microsoft.card.adaptive".to_string(),
                content: card,
            }],
        }
    }
}

/// Teams notification service
pub struct TeamsNotificationService {
    client: reqwest::Client,
}

impl TeamsNotificationService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Send a notification to a Teams webhook
    pub async fn send_webhook(
        &self,
        webhook_url: &str,
        payload: &TeamsWebhookPayload,
    ) -> Result<(), TeamsError> {
        let response = self
            .client
            .post(webhook_url)
            .json(payload)
            .send()
            .await
            .map_err(|e| TeamsError::RequestFailed(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(TeamsError::WebhookFailed(format!(
                "Status: {}, Body: {}",
                status, body
            )))
        }
    }

    /// Build and send a ticket created notification
    pub async fn notify_ticket_created(
        &self,
        webhook_url: &str,
        ticket: &TicketNotification,
        portal_base_url: &str,
    ) -> Result<(), TeamsError> {
        let priority_color = match ticket.priority.as_str() {
            "critical" => "attention",
            "high" => "warning",
            "medium" => "accent",
            _ => "good",
        };

        let card = TeamsAdaptiveCard {
            body: vec![
                TeamsCardElement::Container {
                    items: vec![
                        TeamsCardElement::TextBlock {
                            text: format!("🎫 New Ticket #{}", ticket.number),
                            size: Some("large".to_string()),
                            weight: Some("bolder".to_string()),
                            color: None,
                            wrap: None,
                            spacing: None,
                        },
                        TeamsCardElement::TextBlock {
                            text: ticket.subject.clone(),
                            size: Some("medium".to_string()),
                            weight: Some("bolder".to_string()),
                            color: None,
                            wrap: Some(true),
                            spacing: Some("small".to_string()),
                        },
                    ],
                    style: None,
                    spacing: None,
                },
                TeamsCardElement::FactSet {
                    facts: vec![
                        TeamsFact {
                            title: "Client".to_string(),
                            value: ticket.client_name.clone(),
                        },
                        TeamsFact {
                            title: "Priority".to_string(),
                            value: ticket.priority.to_uppercase(),
                        },
                        TeamsFact {
                            title: "Category".to_string(),
                            value: ticket.category.clone().unwrap_or_else(|| "General".to_string()),
                        },
                        TeamsFact {
                            title: "Created".to_string(),
                            value: ticket.created_at.format("%Y-%m-%d %H:%M").to_string(),
                        },
                    ],
                    spacing: Some("medium".to_string()),
                },
                TeamsCardElement::TextBlock {
                    text: ticket.description.chars().take(200).collect::<String>()
                        + if ticket.description.len() > 200 { "..." } else { "" },
                    size: None,
                    weight: None,
                    color: None,
                    wrap: Some(true),
                    spacing: Some("medium".to_string()),
                },
            ],
            actions: Some(vec![
                TeamsCardAction::OpenUrl {
                    title: "View Ticket".to_string(),
                    url: format!("{}/tickets/{}", portal_base_url, ticket.id),
                },
            ]),
            ..Default::default()
        };

        let payload = TeamsWebhookPayload::from_card(card);
        self.send_webhook(webhook_url, &payload).await
    }

    /// Build and send a ticket assigned notification
    pub async fn notify_ticket_assigned(
        &self,
        webhook_url: &str,
        ticket: &TicketNotification,
        assigned_to: &str,
        portal_base_url: &str,
    ) -> Result<(), TeamsError> {
        let card = TeamsAdaptiveCard {
            body: vec![
                TeamsCardElement::TextBlock {
                    text: format!("📋 Ticket #{} Assigned", ticket.number),
                    size: Some("large".to_string()),
                    weight: Some("bolder".to_string()),
                    color: None,
                    wrap: None,
                    spacing: None,
                },
                TeamsCardElement::TextBlock {
                    text: ticket.subject.clone(),
                    size: None,
                    weight: Some("bolder".to_string()),
                    color: None,
                    wrap: Some(true),
                    spacing: Some("small".to_string()),
                },
                TeamsCardElement::FactSet {
                    facts: vec![
                        TeamsFact {
                            title: "Assigned To".to_string(),
                            value: assigned_to.to_string(),
                        },
                        TeamsFact {
                            title: "Client".to_string(),
                            value: ticket.client_name.clone(),
                        },
                        TeamsFact {
                            title: "Priority".to_string(),
                            value: ticket.priority.to_uppercase(),
                        },
                    ],
                    spacing: Some("medium".to_string()),
                },
            ],
            actions: Some(vec![
                TeamsCardAction::OpenUrl {
                    title: "View Ticket".to_string(),
                    url: format!("{}/tickets/{}", portal_base_url, ticket.id),
                },
            ]),
            ..Default::default()
        };

        let payload = TeamsWebhookPayload::from_card(card);
        self.send_webhook(webhook_url, &payload).await
    }

    /// Build and send an SLA breach notification
    pub async fn notify_sla_breach(
        &self,
        webhook_url: &str,
        ticket: &TicketNotification,
        breach_type: &str,
        breach_duration_minutes: i64,
        portal_base_url: &str,
    ) -> Result<(), TeamsError> {
        let hours = breach_duration_minutes / 60;
        let minutes = breach_duration_minutes % 60;
        let breach_duration = if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        };

        let card = TeamsAdaptiveCard {
            body: vec![
                TeamsCardElement::Container {
                    items: vec![
                        TeamsCardElement::TextBlock {
                            text: "⚠️ SLA Breach Alert".to_string(),
                            size: Some("large".to_string()),
                            weight: Some("bolder".to_string()),
                            color: Some("attention".to_string()),
                            wrap: None,
                            spacing: None,
                        },
                    ],
                    style: Some("attention".to_string()),
                    spacing: None,
                },
                TeamsCardElement::TextBlock {
                    text: format!("Ticket #{} - {}", ticket.number, ticket.subject),
                    size: None,
                    weight: Some("bolder".to_string()),
                    color: None,
                    wrap: Some(true),
                    spacing: Some("medium".to_string()),
                },
                TeamsCardElement::FactSet {
                    facts: vec![
                        TeamsFact {
                            title: "Breach Type".to_string(),
                            value: breach_type.to_string(),
                        },
                        TeamsFact {
                            title: "Overdue By".to_string(),
                            value: breach_duration,
                        },
                        TeamsFact {
                            title: "Client".to_string(),
                            value: ticket.client_name.clone(),
                        },
                        TeamsFact {
                            title: "Priority".to_string(),
                            value: ticket.priority.to_uppercase(),
                        },
                        TeamsFact {
                            title: "Assigned To".to_string(),
                            value: ticket.assigned_to.clone().unwrap_or_else(|| "Unassigned".to_string()),
                        },
                    ],
                    spacing: Some("medium".to_string()),
                },
            ],
            actions: Some(vec![
                TeamsCardAction::OpenUrl {
                    title: "View Ticket".to_string(),
                    url: format!("{}/tickets/{}", portal_base_url, ticket.id),
                },
            ]),
            ..Default::default()
        };

        let payload = TeamsWebhookPayload::from_card(card);
        self.send_webhook(webhook_url, &payload).await
    }

    /// Build and send a ticket resolved notification
    pub async fn notify_ticket_resolved(
        &self,
        webhook_url: &str,
        ticket: &TicketNotification,
        resolution: &str,
        resolved_by: &str,
        portal_base_url: &str,
    ) -> Result<(), TeamsError> {
        let card = TeamsAdaptiveCard {
            body: vec![
                TeamsCardElement::TextBlock {
                    text: format!("✅ Ticket #{} Resolved", ticket.number),
                    size: Some("large".to_string()),
                    weight: Some("bolder".to_string()),
                    color: Some("good".to_string()),
                    wrap: None,
                    spacing: None,
                },
                TeamsCardElement::TextBlock {
                    text: ticket.subject.clone(),
                    size: None,
                    weight: Some("bolder".to_string()),
                    color: None,
                    wrap: Some(true),
                    spacing: Some("small".to_string()),
                },
                TeamsCardElement::FactSet {
                    facts: vec![
                        TeamsFact {
                            title: "Client".to_string(),
                            value: ticket.client_name.clone(),
                        },
                        TeamsFact {
                            title: "Resolved By".to_string(),
                            value: resolved_by.to_string(),
                        },
                    ],
                    spacing: Some("medium".to_string()),
                },
                TeamsCardElement::TextBlock {
                    text: format!("**Resolution:** {}", resolution),
                    size: None,
                    weight: None,
                    color: None,
                    wrap: Some(true),
                    spacing: Some("medium".to_string()),
                },
            ],
            actions: Some(vec![
                TeamsCardAction::OpenUrl {
                    title: "View Ticket".to_string(),
                    url: format!("{}/tickets/{}", portal_base_url, ticket.id),
                },
            ]),
            ..Default::default()
        };

        let payload = TeamsWebhookPayload::from_card(card);
        self.send_webhook(webhook_url, &payload).await
    }

    /// Build and send a daily summary notification
    pub async fn notify_daily_summary(
        &self,
        webhook_url: &str,
        summary: &DailySummary,
        portal_base_url: &str,
    ) -> Result<(), TeamsError> {
        let card = TeamsAdaptiveCard {
            body: vec![
                TeamsCardElement::TextBlock {
                    text: format!("📊 Daily Summary - {}", summary.date.format("%Y-%m-%d")),
                    size: Some("large".to_string()),
                    weight: Some("bolder".to_string()),
                    color: None,
                    wrap: None,
                    spacing: None,
                },
                TeamsCardElement::ColumnSet {
                    columns: vec![
                        TeamsColumn {
                            width: "stretch".to_string(),
                            items: vec![
                                TeamsCardElement::TextBlock {
                                    text: format!("{}", summary.tickets_created),
                                    size: Some("extraLarge".to_string()),
                                    weight: Some("bolder".to_string()),
                                    color: None,
                                    wrap: None,
                                    spacing: None,
                                },
                                TeamsCardElement::TextBlock {
                                    text: "New Tickets".to_string(),
                                    size: Some("small".to_string()),
                                    weight: None,
                                    color: None,
                                    wrap: None,
                                    spacing: None,
                                },
                            ],
                        },
                        TeamsColumn {
                            width: "stretch".to_string(),
                            items: vec![
                                TeamsCardElement::TextBlock {
                                    text: format!("{}", summary.tickets_resolved),
                                    size: Some("extraLarge".to_string()),
                                    weight: Some("bolder".to_string()),
                                    color: Some("good".to_string()),
                                    wrap: None,
                                    spacing: None,
                                },
                                TeamsCardElement::TextBlock {
                                    text: "Resolved".to_string(),
                                    size: Some("small".to_string()),
                                    weight: None,
                                    color: None,
                                    wrap: None,
                                    spacing: None,
                                },
                            ],
                        },
                        TeamsColumn {
                            width: "stretch".to_string(),
                            items: vec![
                                TeamsCardElement::TextBlock {
                                    text: format!("{}", summary.sla_breaches),
                                    size: Some("extraLarge".to_string()),
                                    weight: Some("bolder".to_string()),
                                    color: if summary.sla_breaches > 0 {
                                        Some("attention".to_string())
                                    } else {
                                        Some("good".to_string())
                                    },
                                    wrap: None,
                                    spacing: None,
                                },
                                TeamsCardElement::TextBlock {
                                    text: "SLA Breaches".to_string(),
                                    size: Some("small".to_string()),
                                    weight: None,
                                    color: None,
                                    wrap: None,
                                    spacing: None,
                                },
                            ],
                        },
                    ],
                    spacing: Some("medium".to_string()),
                },
                TeamsCardElement::FactSet {
                    facts: vec![
                        TeamsFact {
                            title: "Open Tickets".to_string(),
                            value: summary.open_tickets.to_string(),
                        },
                        TeamsFact {
                            title: "Billable Hours".to_string(),
                            value: format!("{:.1}h", summary.billable_hours),
                        },
                        TeamsFact {
                            title: "SLA Compliance".to_string(),
                            value: format!("{:.1}%", summary.sla_compliance),
                        },
                    ],
                    spacing: Some("medium".to_string()),
                },
            ],
            actions: Some(vec![
                TeamsCardAction::OpenUrl {
                    title: "View Dashboard".to_string(),
                    url: format!("{}/dashboard", portal_base_url),
                },
            ]),
            ..Default::default()
        };

        let payload = TeamsWebhookPayload::from_card(card);
        self.send_webhook(webhook_url, &payload).await
    }

    /// Send a simple text message
    pub async fn send_simple_message(
        &self,
        webhook_url: &str,
        message: &str,
    ) -> Result<(), TeamsError> {
        let card = TeamsAdaptiveCard {
            body: vec![
                TeamsCardElement::TextBlock {
                    text: message.to_string(),
                    size: None,
                    weight: None,
                    color: None,
                    wrap: Some(true),
                    spacing: None,
                },
            ],
            actions: None,
            ..Default::default()
        };

        let payload = TeamsWebhookPayload::from_card(card);
        self.send_webhook(webhook_url, &payload).await
    }
}

/// Ticket data for notifications
#[derive(Debug, Clone)]
pub struct TicketNotification {
    pub id: Uuid,
    pub number: i32,
    pub subject: String,
    pub description: String,
    pub priority: String,
    pub status: String,
    pub client_id: Uuid,
    pub client_name: String,
    pub category: Option<String>,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Daily summary data
#[derive(Debug, Clone)]
pub struct DailySummary {
    pub date: chrono::NaiveDate,
    pub tickets_created: i64,
    pub tickets_resolved: i64,
    pub open_tickets: i64,
    pub sla_breaches: i64,
    pub sla_compliance: f64,
    pub billable_hours: f64,
}

#[derive(Debug)]
pub enum TeamsError {
    RequestFailed(String),
    WebhookFailed(String),
    InvalidPayload(String),
}

impl std::fmt::Display for TeamsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamsError::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
            TeamsError::WebhookFailed(msg) => write!(f, "Webhook failed: {}", msg),
            TeamsError::InvalidPayload(msg) => write!(f, "Invalid payload: {}", msg),
        }
    }
}

impl std::error::Error for TeamsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_card_serialization() {
        let card = TeamsAdaptiveCard {
            body: vec![
                TeamsCardElement::TextBlock {
                    text: "Hello Teams!".to_string(),
                    size: Some("large".to_string()),
                    weight: Some("bolder".to_string()),
                    color: None,
                    wrap: Some(true),
                    spacing: None,
                },
            ],
            actions: Some(vec![
                TeamsCardAction::OpenUrl {
                    title: "Open Link".to_string(),
                    url: "https://example.com".to_string(),
                },
            ]),
            ..Default::default()
        };

        let payload = TeamsWebhookPayload::from_card(card);
        let json = serde_json::to_string_pretty(&payload).unwrap();
        assert!(json.contains("AdaptiveCard"));
        assert!(json.contains("Hello Teams!"));
    }
}
