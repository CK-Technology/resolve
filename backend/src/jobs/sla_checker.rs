// SLA Checker Job - Monitors tickets for SLA breaches and escalations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::services::EmailService;
use crate::websocket::WsManager;

#[derive(Debug)]
pub struct SlaCheckerJob {
    db_pool: PgPool,
    email_service: EmailService,
    ws_manager: WsManager,
    auto_escalation_enabled: bool,
}

#[derive(Debug, Default)]
pub struct SlaCheckResult {
    pub tickets_checked: i32,
    pub breaches_detected: i32,
    pub escalations_triggered: i32,
    pub notifications_sent: i32,
    pub errors: Vec<String>,
}

#[derive(Debug, FromRow)]
struct TicketSlaInfo {
    ticket_id: Uuid,
    ticket_subject: String,
    client_id: Uuid,
    client_name: String,
    assigned_to: Option<Uuid>,
    assigned_user_name: Option<String>,
    priority: String,
    status: String,
    sla_tracking_id: Uuid,
    response_due_at: DateTime<Utc>,
    resolution_due_at: DateTime<Utc>,
    first_response_at: Option<DateTime<Utc>>,
    resolved_at: Option<DateTime<Utc>>,
    response_breached: bool,
    resolution_breached: bool,
    pause_start: Option<DateTime<Utc>>,
    escalation_time_minutes: Option<i32>,
    escalation_user_id: Option<Uuid>,
    breach_notifications_sent: i32,
    breach_notification_emails: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SlaBreachNotification {
    ticket_id: Uuid,
    ticket_subject: String,
    client_name: String,
    priority: String,
    breach_type: String,
    due_at: DateTime<Utc>,
    breach_minutes: i32,
    assigned_to: Option<String>,
}

impl SlaCheckerJob {
    pub fn new(
        db_pool: PgPool,
        email_service: EmailService,
        ws_manager: WsManager,
        auto_escalation_enabled: bool,
    ) -> Self {
        Self {
            db_pool,
            email_service,
            ws_manager,
            auto_escalation_enabled,
        }
    }

    pub async fn run(&self) -> Result<SlaCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = SlaCheckResult::default();

        // Get all active tickets with SLA tracking
        let tickets = self.get_tracked_tickets().await?;
        result.tickets_checked = tickets.len() as i32;

        let now = Utc::now();

        for ticket in tickets {
            // Skip paused tickets
            if ticket.pause_start.is_some() {
                continue;
            }

            // Check response SLA
            if ticket.first_response_at.is_none() && !ticket.response_breached {
                if now > ticket.response_due_at {
                    result.breaches_detected += 1;

                    let breach_minutes = (now - ticket.response_due_at).num_minutes() as i32;

                    if let Err(e) = self.mark_response_breach(&ticket, breach_minutes).await {
                        result.errors.push(format!("Failed to mark response breach for ticket {}: {}", ticket.ticket_id, e));
                        continue;
                    }

                    // Send notification
                    if let Err(e) = self.send_breach_notification(&ticket, "response", breach_minutes).await {
                        result.errors.push(format!("Failed to send breach notification for ticket {}: {}", ticket.ticket_id, e));
                    } else {
                        result.notifications_sent += 1;
                    }

                    // Broadcast via WebSocket
                    self.broadcast_breach_alert(&ticket, "response", breach_minutes).await;
                }
            }

            // Check resolution SLA
            if ticket.resolved_at.is_none() && !ticket.resolution_breached {
                if now > ticket.resolution_due_at {
                    result.breaches_detected += 1;

                    let breach_minutes = (now - ticket.resolution_due_at).num_minutes() as i32;

                    if let Err(e) = self.mark_resolution_breach(&ticket, breach_minutes).await {
                        result.errors.push(format!("Failed to mark resolution breach for ticket {}: {}", ticket.ticket_id, e));
                        continue;
                    }

                    // Send notification
                    if let Err(e) = self.send_breach_notification(&ticket, "resolution", breach_minutes).await {
                        result.errors.push(format!("Failed to send breach notification for ticket {}: {}", ticket.ticket_id, e));
                    } else {
                        result.notifications_sent += 1;
                    }

                    // Check for escalation
                    if self.auto_escalation_enabled {
                        if let Some(escalation_minutes) = ticket.escalation_time_minutes {
                            if breach_minutes >= escalation_minutes {
                                if let Err(e) = self.escalate_ticket(&ticket).await {
                                    result.errors.push(format!("Failed to escalate ticket {}: {}", ticket.ticket_id, e));
                                } else {
                                    result.escalations_triggered += 1;
                                }
                            }
                        }
                    }

                    // Broadcast via WebSocket
                    self.broadcast_breach_alert(&ticket, "resolution", breach_minutes).await;
                }
            }

            // Check for approaching breach (warning notifications)
            self.check_approaching_breach(&ticket, &now, &mut result).await;
        }

        Ok(result)
    }

    async fn get_tracked_tickets(&self) -> Result<Vec<TicketSlaInfo>, sqlx::Error> {
        sqlx::query_as::<_, TicketSlaInfo>(
            r#"
            SELECT
                t.id as ticket_id,
                t.subject as ticket_subject,
                t.client_id,
                c.name as client_name,
                t.assigned_to,
                u.first_name || ' ' || u.last_name as assigned_user_name,
                t.priority,
                t.status,
                st.id as sla_tracking_id,
                st.response_due_at,
                st.resolution_due_at,
                st.first_response_at,
                st.resolved_at,
                st.response_breached,
                st.resolution_breached,
                st.pause_start,
                sr.escalation_time_minutes,
                sr.escalation_user_id,
                st.breach_notifications_sent,
                sr.breach_notification_emails
            FROM tickets t
            JOIN clients c ON t.client_id = c.id
            JOIN ticket_sla_tracking st ON t.id = st.ticket_id
            JOIN sla_rules sr ON st.sla_rule_id = sr.id
            LEFT JOIN users u ON t.assigned_to = u.id
            WHERE t.status NOT IN ('resolved', 'closed', 'cancelled')
            ORDER BY
                CASE t.priority
                    WHEN 'critical' THEN 1
                    WHEN 'high' THEN 2
                    WHEN 'medium' THEN 3
                    WHEN 'low' THEN 4
                    ELSE 5
                END,
                st.resolution_due_at ASC
            "#
        )
        .fetch_all(&self.db_pool)
        .await
    }

    async fn mark_response_breach(&self, ticket: &TicketSlaInfo, breach_minutes: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE ticket_sla_tracking
             SET response_breached = true,
                 response_breach_minutes = $2,
                 updated_at = NOW()
             WHERE id = $1"
        )
        .bind(ticket.sla_tracking_id)
        .bind(breach_minutes)
        .execute(&self.db_pool)
        .await?;

        // Update ticket with breach indicator
        sqlx::query(
            "UPDATE tickets SET sla_breached = true, updated_at = NOW() WHERE id = $1"
        )
        .bind(ticket.ticket_id)
        .execute(&self.db_pool)
        .await?;

        info!("Marked response breach for ticket {}: {} minutes overdue", ticket.ticket_id, breach_minutes);

        Ok(())
    }

    async fn mark_resolution_breach(&self, ticket: &TicketSlaInfo, breach_minutes: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE ticket_sla_tracking
             SET resolution_breached = true,
                 resolution_breach_minutes = $2,
                 updated_at = NOW()
             WHERE id = $1"
        )
        .bind(ticket.sla_tracking_id)
        .bind(breach_minutes)
        .execute(&self.db_pool)
        .await?;

        sqlx::query(
            "UPDATE tickets SET sla_breached = true, updated_at = NOW() WHERE id = $1"
        )
        .bind(ticket.ticket_id)
        .execute(&self.db_pool)
        .await?;

        info!("Marked resolution breach for ticket {}: {} minutes overdue", ticket.ticket_id, breach_minutes);

        Ok(())
    }

    async fn send_breach_notification(
        &self,
        ticket: &TicketSlaInfo,
        breach_type: &str,
        breach_minutes: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let breach_hours = breach_minutes / 60;
        let breach_remaining_mins = breach_minutes % 60;

        let subject = format!(
            "[SLA BREACH] {} - {} is {} {}",
            ticket.client_name,
            ticket.ticket_subject,
            if breach_type == "response" { "Response" } else { "Resolution" },
            format_duration(breach_minutes)
        );

        let priority_color = match ticket.priority.as_str() {
            "critical" => "#dc2626",
            "high" => "#f97316",
            "medium" => "#eab308",
            _ => "#22c55e",
        };

        let html_body = format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
                    .container {{ max-width: 600px; margin: 0 auto; background: white; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 12px rgba(0,0,0,0.1); }}
                    .header {{ background: linear-gradient(135deg, #dc2626 0%, #b91c1c 100%); color: white; padding: 24px; text-align: center; }}
                    .header h1 {{ margin: 0; font-size: 24px; }}
                    .content {{ padding: 24px; }}
                    .alert-box {{ background: #fef2f2; border: 2px solid #dc2626; border-radius: 8px; padding: 16px; margin-bottom: 20px; }}
                    .detail-row {{ display: flex; justify-content: space-between; padding: 12px 0; border-bottom: 1px solid #e5e7eb; }}
                    .detail-row:last-child {{ border-bottom: none; }}
                    .label {{ color: #6b7280; font-weight: 500; }}
                    .value {{ color: #111827; font-weight: 600; }}
                    .priority-badge {{ display: inline-block; padding: 4px 12px; border-radius: 9999px; font-size: 12px; font-weight: 600; color: white; background: {}; }}
                    .cta-button {{ display: inline-block; background: #1f2937; color: white; padding: 12px 24px; text-decoration: none; border-radius: 8px; margin-top: 16px; }}
                    .footer {{ background: #f9fafb; padding: 16px 24px; text-align: center; color: #6b7280; font-size: 14px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>⚠️ SLA Breach Alert</h1>
                    </div>
                    <div class="content">
                        <div class="alert-box">
                            <strong>{} SLA has been breached!</strong>
                            <p>This ticket is <strong>{}</strong> past the {} deadline.</p>
                        </div>

                        <h3>Ticket Details</h3>
                        <div class="detail-row">
                            <span class="label">Ticket</span>
                            <span class="value">{}</span>
                        </div>
                        <div class="detail-row">
                            <span class="label">Client</span>
                            <span class="value">{}</span>
                        </div>
                        <div class="detail-row">
                            <span class="label">Priority</span>
                            <span class="priority-badge">{}</span>
                        </div>
                        <div class="detail-row">
                            <span class="label">Assigned To</span>
                            <span class="value">{}</span>
                        </div>
                        <div class="detail-row">
                            <span class="label">Breach Duration</span>
                            <span class="value" style="color: #dc2626;">{}</span>
                        </div>

                        <p style="margin-top: 20px;">Please take immediate action to address this ticket.</p>
                    </div>
                    <div class="footer">
                        <p>Resolve MSP Platform - SLA Monitoring System</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            priority_color,
            if breach_type == "response" { "Response" } else { "Resolution" },
            format_duration(breach_minutes),
            breach_type,
            ticket.ticket_subject,
            ticket.client_name,
            ticket.priority.to_uppercase(),
            ticket.assigned_user_name.as_deref().unwrap_or("Unassigned"),
            format_duration(breach_minutes)
        );

        // Send to breach notification emails
        for email in &ticket.breach_notification_emails {
            if let Err(e) = self.email_service.send_email(email, None, &subject, &html_body, None).await {
                error!("Failed to send breach notification to {}: {}", email, e);
            }
        }

        // Update notification count
        sqlx::query(
            "UPDATE ticket_sla_tracking
             SET breach_notifications_sent = breach_notifications_sent + 1
             WHERE id = $1"
        )
        .bind(ticket.sla_tracking_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn escalate_ticket(&self, ticket: &TicketSlaInfo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(escalation_user_id) = ticket.escalation_user_id {
            // Update ticket assignment
            sqlx::query(
                "UPDATE tickets
                 SET assigned_to = $2,
                     updated_at = NOW()
                 WHERE id = $1"
            )
            .bind(ticket.ticket_id)
            .bind(escalation_user_id)
            .execute(&self.db_pool)
            .await?;

            // Mark escalation in SLA tracking
            sqlx::query(
                "UPDATE ticket_sla_tracking
                 SET escalated_at = NOW(),
                     escalated_to_user_id = $2,
                     updated_at = NOW()
                 WHERE id = $1"
            )
            .bind(ticket.sla_tracking_id)
            .bind(escalation_user_id)
            .execute(&self.db_pool)
            .await?;

            // Get escalation user details
            let escalation_user: Option<(String,)> = sqlx::query_as(
                "SELECT email FROM users WHERE id = $1"
            )
            .bind(escalation_user_id)
            .fetch_optional(&self.db_pool)
            .await?;

            if let Some((email,)) = escalation_user {
                let subject = format!("[ESCALATION] {} - {} requires your attention",
                    ticket.client_name, ticket.ticket_subject);

                let html_body = format!(
                    r#"
                    <html>
                    <body style="font-family: Arial, sans-serif; padding: 20px;">
                        <h2>🚨 Ticket Escalated to You</h2>
                        <p>A ticket has been automatically escalated to you due to SLA breach:</p>
                        <div style="background: #fef2f2; border-left: 4px solid #dc2626; padding: 15px; margin: 15px 0;">
                            <p><strong>Client:</strong> {}</p>
                            <p><strong>Subject:</strong> {}</p>
                            <p><strong>Priority:</strong> {}</p>
                            <p><strong>Previous Assignee:</strong> {}</p>
                        </div>
                        <p>Please review and take action immediately.</p>
                    </body>
                    </html>
                    "#,
                    ticket.client_name,
                    ticket.ticket_subject,
                    ticket.priority,
                    ticket.assigned_user_name.as_deref().unwrap_or("Unassigned")
                );

                self.email_service.send_email(&email, None, &subject, &html_body, None).await?;
            }

            info!("Escalated ticket {} to user {}", ticket.ticket_id, escalation_user_id);
        }

        Ok(())
    }

    async fn broadcast_breach_alert(&self, ticket: &TicketSlaInfo, breach_type: &str, breach_minutes: i32) {
        let notification = SlaBreachNotification {
            ticket_id: ticket.ticket_id,
            ticket_subject: ticket.ticket_subject.clone(),
            client_name: ticket.client_name.clone(),
            priority: ticket.priority.clone(),
            breach_type: breach_type.to_string(),
            due_at: if breach_type == "response" { ticket.response_due_at } else { ticket.resolution_due_at },
            breach_minutes,
            assigned_to: ticket.assigned_user_name.clone(),
        };

        if let Ok(json) = serde_json::to_string(&notification) {
            self.ws_manager.broadcast(&format!(
                r#"{{"type": "sla_breach", "data": {}}}"#,
                json
            )).await;
        }
    }

    async fn check_approaching_breach(&self, ticket: &TicketSlaInfo, now: &DateTime<Utc>, result: &mut SlaCheckResult) {
        // Warning thresholds (in minutes)
        let warning_thresholds = vec![60, 30, 15, 5]; // 1 hour, 30 min, 15 min, 5 min

        // Check response approaching breach
        if ticket.first_response_at.is_none() && !ticket.response_breached {
            let minutes_until_breach = (ticket.response_due_at - *now).num_minutes();

            for threshold in &warning_thresholds {
                if minutes_until_breach <= *threshold as i64 && minutes_until_breach > (*threshold - 5) as i64 {
                    // Send warning notification
                    self.ws_manager.broadcast(&format!(
                        r#"{{"type": "sla_warning", "data": {{"ticket_id": "{}", "breach_type": "response", "minutes_remaining": {}}}}}"#,
                        ticket.ticket_id, minutes_until_breach
                    )).await;
                    break;
                }
            }
        }

        // Check resolution approaching breach
        if ticket.resolved_at.is_none() && !ticket.resolution_breached {
            let minutes_until_breach = (ticket.resolution_due_at - *now).num_minutes();

            for threshold in &warning_thresholds {
                if minutes_until_breach <= *threshold as i64 && minutes_until_breach > (*threshold - 5) as i64 {
                    self.ws_manager.broadcast(&format!(
                        r#"{{"type": "sla_warning", "data": {{"ticket_id": "{}", "breach_type": "resolution", "minutes_remaining": {}}}}}"#,
                        ticket.ticket_id, minutes_until_breach
                    )).await;
                    break;
                }
            }
        }
    }
}

fn format_duration(minutes: i32) -> String {
    if minutes < 60 {
        format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" })
    } else if minutes < 1440 {
        let hours = minutes / 60;
        let mins = minutes % 60;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{} hour{}", hours, if hours == 1 { "" } else { "s" })
        }
    } else {
        let days = minutes / 1440;
        let hours = (minutes % 1440) / 60;
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{} day{}", days, if days == 1 { "" } else { "s" })
        }
    }
}
