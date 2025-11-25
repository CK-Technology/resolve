// Workflow Actions - Actions that can be executed by workflows

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of actions that workflows can execute
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    // Ticket actions
    AssignTicket,
    UpdateTicketStatus,
    UpdateTicketPriority,
    AddTicketComment,
    AddTicketTag,
    RemoveTicketTag,
    EscalateTicket,
    MergeTickets,
    LinkTickets,

    // Notification actions
    SendEmail,
    SendTeamsNotification,
    SendSlackNotification,
    SendWebhook,
    SendSms,
    CreateNotification,

    // Assignment actions
    AssignToUser,
    AssignToGroup,
    AssignRoundRobin,
    AssignBySkill,
    AssignByWorkload,

    // SLA actions
    ApplySlaPolicy,
    PauseSla,
    ResumeSla,
    ResetSlaTimer,

    // Time tracking actions
    StartTimer,
    StopTimer,
    AddTimeEntry,

    // Invoice actions
    CreateInvoice,
    SendInvoice,
    ApplyCredit,
    AddLineItem,

    // Asset actions
    CreateAsset,
    UpdateAsset,
    LinkAssetToTicket,

    // Client actions
    UpdateClientField,
    AddClientNote,
    ChangeClientTier,

    // Data actions
    SetField,
    IncrementField,
    CopyField,
    TransformField,

    // Control flow
    Wait,
    ConditionalBranch,
    CallWorkflow,
    StopWorkflow,

    // Integration actions
    CallApi,
    RunScript,
    ExecuteQuery,

    // Custom
    CustomAction,
}

/// An action to be executed in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: Uuid,
    pub name: String,
    pub action_type: ActionType,
    pub config: serde_json::Value,
    pub delay_seconds: i32,
    pub retry_count: i32,
    pub retry_delay_seconds: i32,
    pub stop_on_failure: bool,
    pub condition: Option<String>, // Optional inline condition
}

/// Result of executing an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub retry_attempted: i32,
    pub duration_ms: i64,
}

impl Action {
    pub fn new(name: &str, action_type: ActionType, config: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            action_type,
            config,
            delay_seconds: 0,
            retry_count: 0,
            retry_delay_seconds: 30,
            stop_on_failure: false,
            condition: None,
        }
    }

    pub fn with_delay(mut self, seconds: i32) -> Self {
        self.delay_seconds = seconds;
        self
    }

    pub fn with_retry(mut self, count: i32, delay_seconds: i32) -> Self {
        self.retry_count = count;
        self.retry_delay_seconds = delay_seconds;
        self
    }

    pub fn stop_on_failure(mut self) -> Self {
        self.stop_on_failure = true;
        self
    }

    pub fn with_condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_string());
        self
    }

    // ===== Ticket Action Builders =====

    pub fn assign_ticket(user_id: Uuid) -> Self {
        Self::new(
            "Assign Ticket",
            ActionType::AssignTicket,
            serde_json::json!({ "user_id": user_id }),
        )
    }

    pub fn assign_to_group(group_id: Uuid) -> Self {
        Self::new(
            "Assign to Group",
            ActionType::AssignToGroup,
            serde_json::json!({ "group_id": group_id }),
        )
    }

    pub fn update_ticket_status(status: &str) -> Self {
        Self::new(
            "Update Status",
            ActionType::UpdateTicketStatus,
            serde_json::json!({ "status": status }),
        )
    }

    pub fn update_ticket_priority(priority: &str) -> Self {
        Self::new(
            "Update Priority",
            ActionType::UpdateTicketPriority,
            serde_json::json!({ "priority": priority }),
        )
    }

    pub fn add_ticket_comment(comment: &str, internal: bool) -> Self {
        Self::new(
            "Add Comment",
            ActionType::AddTicketComment,
            serde_json::json!({
                "comment": comment,
                "internal": internal
            }),
        )
    }

    pub fn add_ticket_tag(tag: &str) -> Self {
        Self::new(
            "Add Tag",
            ActionType::AddTicketTag,
            serde_json::json!({ "tag": tag }),
        )
    }

    pub fn escalate_ticket(to_user_id: Uuid, reason: &str) -> Self {
        Self::new(
            "Escalate Ticket",
            ActionType::EscalateTicket,
            serde_json::json!({
                "to_user_id": to_user_id,
                "reason": reason
            }),
        )
    }

    // ===== Notification Action Builders =====

    pub fn send_email(to: &str, subject: &str, body: &str) -> Self {
        Self::new(
            "Send Email",
            ActionType::SendEmail,
            serde_json::json!({
                "to": to,
                "subject": subject,
                "body": body
            }),
        )
    }

    pub fn send_email_template(to: &str, template_id: &str, variables: serde_json::Value) -> Self {
        Self::new(
            "Send Email Template",
            ActionType::SendEmail,
            serde_json::json!({
                "to": to,
                "template_id": template_id,
                "variables": variables
            }),
        )
    }

    pub fn send_teams_notification(channel: &str, message: &str) -> Self {
        Self::new(
            "Send Teams Notification",
            ActionType::SendTeamsNotification,
            serde_json::json!({
                "channel": channel,
                "message": message
            }),
        )
    }

    pub fn send_webhook(url: &str, payload: serde_json::Value) -> Self {
        Self::new(
            "Send Webhook",
            ActionType::SendWebhook,
            serde_json::json!({
                "url": url,
                "method": "POST",
                "payload": payload
            }),
        )
    }

    pub fn create_notification(user_id: Uuid, title: &str, message: &str, notification_type: &str) -> Self {
        Self::new(
            "Create Notification",
            ActionType::CreateNotification,
            serde_json::json!({
                "user_id": user_id,
                "title": title,
                "message": message,
                "type": notification_type
            }),
        )
    }

    // ===== SLA Action Builders =====

    pub fn apply_sla_policy(policy_id: Uuid) -> Self {
        Self::new(
            "Apply SLA Policy",
            ActionType::ApplySlaPolicy,
            serde_json::json!({ "policy_id": policy_id }),
        )
    }

    pub fn pause_sla(reason: &str) -> Self {
        Self::new(
            "Pause SLA",
            ActionType::PauseSla,
            serde_json::json!({ "reason": reason }),
        )
    }

    pub fn resume_sla() -> Self {
        Self::new(
            "Resume SLA",
            ActionType::ResumeSla,
            serde_json::json!({}),
        )
    }

    // ===== Data Action Builders =====

    pub fn set_field(field: &str, value: serde_json::Value) -> Self {
        Self::new(
            &format!("Set {}", field),
            ActionType::SetField,
            serde_json::json!({
                "field": field,
                "value": value
            }),
        )
    }

    pub fn increment_field(field: &str, amount: i32) -> Self {
        Self::new(
            &format!("Increment {}", field),
            ActionType::IncrementField,
            serde_json::json!({
                "field": field,
                "amount": amount
            }),
        )
    }

    pub fn copy_field(from: &str, to: &str) -> Self {
        Self::new(
            &format!("Copy {} to {}", from, to),
            ActionType::CopyField,
            serde_json::json!({
                "from": from,
                "to": to
            }),
        )
    }

    // ===== Control Flow Action Builders =====

    pub fn wait(seconds: i32) -> Self {
        Self::new(
            &format!("Wait {} seconds", seconds),
            ActionType::Wait,
            serde_json::json!({ "seconds": seconds }),
        )
    }

    pub fn call_workflow(workflow_id: Uuid) -> Self {
        Self::new(
            "Call Workflow",
            ActionType::CallWorkflow,
            serde_json::json!({ "workflow_id": workflow_id }),
        )
    }

    pub fn stop_workflow(reason: &str) -> Self {
        Self::new(
            "Stop Workflow",
            ActionType::StopWorkflow,
            serde_json::json!({ "reason": reason }),
        )
    }

    // ===== Integration Action Builders =====

    pub fn call_api(url: &str, method: &str, headers: Option<serde_json::Value>, body: Option<serde_json::Value>) -> Self {
        Self::new(
            "Call API",
            ActionType::CallApi,
            serde_json::json!({
                "url": url,
                "method": method,
                "headers": headers,
                "body": body
            }),
        )
    }

    pub fn run_script(script_id: &str, parameters: serde_json::Value) -> Self {
        Self::new(
            "Run Script",
            ActionType::RunScript,
            serde_json::json!({
                "script_id": script_id,
                "parameters": parameters
            }),
        )
    }

    // ===== Round Robin Assignment =====

    pub fn assign_round_robin(user_ids: Vec<Uuid>) -> Self {
        Self::new(
            "Assign Round Robin",
            ActionType::AssignRoundRobin,
            serde_json::json!({ "user_ids": user_ids }),
        )
    }

    pub fn assign_by_workload(user_ids: Vec<Uuid>) -> Self {
        Self::new(
            "Assign by Workload",
            ActionType::AssignByWorkload,
            serde_json::json!({ "user_ids": user_ids }),
        )
    }

    pub fn assign_by_skill(skill_tags: Vec<String>) -> Self {
        Self::new(
            "Assign by Skill",
            ActionType::AssignBySkill,
            serde_json::json!({ "skill_tags": skill_tags }),
        )
    }
}

impl ActionResult {
    pub fn success(output: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            output,
            error: None,
            retry_attempted: 0,
            duration_ms: 0,
        }
    }

    pub fn failure(error: &str) -> Self {
        Self {
            success: false,
            output: None,
            error: Some(error.to_string()),
            retry_attempted: 0,
            duration_ms: 0,
        }
    }

    pub fn with_duration(mut self, duration_ms: i64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn with_retry_count(mut self, count: i32) -> Self {
        self.retry_attempted = count;
        self
    }
}

/// Pre-built action sequences for common MSP workflows
pub mod presets {
    use super::*;

    /// Actions for auto-acknowledging a ticket
    pub fn auto_acknowledge() -> Vec<Action> {
        vec![
            Action::update_ticket_status("acknowledged"),
            Action::add_ticket_comment(
                "Thank you for contacting support. Your ticket has been received and will be reviewed shortly.",
                false,
            ),
        ]
    }

    /// Actions for escalating a critical ticket
    pub fn escalate_critical(manager_id: Uuid) -> Vec<Action> {
        vec![
            Action::escalate_ticket(manager_id, "Auto-escalated due to critical priority"),
            Action::add_ticket_tag("escalated"),
            Action::send_teams_notification(
                "critical-alerts",
                "🚨 Critical ticket escalated: {{ticket.subject}}",
            ),
        ]
    }

    /// Actions for notifying about SLA breach
    pub fn sla_breach_notification() -> Vec<Action> {
        vec![
            Action::add_ticket_tag("sla-breach"),
            Action::send_email(
                "{{assigned_to.email}}",
                "SLA Breach Alert: {{ticket.subject}}",
                "The ticket '{{ticket.subject}}' has breached its {{breach_type}} SLA.",
            ),
            Action::create_notification(
                Uuid::nil(), // Will be replaced with actual user
                "SLA Breach",
                "Ticket #{{ticket.id}} has breached SLA",
                "warning",
            ),
        ]
    }

    /// Actions for auto-closing resolved tickets after N days
    pub fn auto_close_resolved(days: i32) -> Vec<Action> {
        vec![
            Action::wait(days * 24 * 60 * 60),
            Action::update_ticket_status("closed"),
            Action::add_ticket_comment(
                &format!("This ticket has been automatically closed after {} days without activity.", days),
                true,
            ),
        ]
    }

    /// Actions for VIP client ticket handling
    pub fn vip_client_handling(vip_team_id: Uuid) -> Vec<Action> {
        vec![
            Action::add_ticket_tag("vip"),
            Action::update_ticket_priority("high"),
            Action::assign_to_group(vip_team_id),
            Action::send_email(
                "vip-support@company.com",
                "VIP Ticket: {{ticket.subject}}",
                "A new ticket has been created by VIP client {{client.name}}.",
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_builder() {
        let action = Action::assign_ticket(Uuid::new_v4())
            .with_delay(60)
            .stop_on_failure();

        assert_eq!(action.action_type, ActionType::AssignTicket);
        assert_eq!(action.delay_seconds, 60);
        assert!(action.stop_on_failure);
    }

    #[test]
    fn test_action_result() {
        let success = ActionResult::success(Some(serde_json::json!({"ticket_id": "123"})));
        assert!(success.success);

        let failure = ActionResult::failure("Database connection error");
        assert!(!failure.success);
        assert!(failure.error.is_some());
    }

    #[test]
    fn test_preset_actions() {
        let ack = presets::auto_acknowledge();
        assert_eq!(ack.len(), 2);
    }
}
