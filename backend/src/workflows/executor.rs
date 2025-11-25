// Workflow Executor - Executes workflow actions

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::{Action, ActionResult, ActionType};
use crate::services::EmailService;
use crate::websocket::WsManager;

/// Context for workflow execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub instance_id: Uuid,
    pub workflow_id: Uuid,
    pub event_payload: serde_json::Value,
    pub variables: HashMap<String, serde_json::Value>,
}

/// Result of workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub instance_id: Uuid,
    pub success: bool,
    pub actions_executed: i32,
    pub actions_failed: i32,
    pub total_duration_ms: i64,
    pub outputs: Vec<ActionResult>,
}

pub struct WorkflowExecutor {
    db_pool: PgPool,
    email_service: EmailService,
    ws_manager: WsManager,
}

impl WorkflowExecutor {
    pub fn new(db_pool: PgPool, email_service: EmailService, ws_manager: WsManager) -> Self {
        Self {
            db_pool,
            email_service,
            ws_manager,
        }
    }

    /// Execute a single action
    pub async fn execute_action(
        &self,
        action: &Action,
        context: &ExecutionContext,
    ) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let start = Instant::now();

        info!("Executing action: {} ({})", action.name, action.action_type.to_string());

        // Process template variables in config
        let config = self.process_templates(&action.config, context);

        let result = match action.action_type {
            // Ticket Actions
            ActionType::AssignTicket => self.execute_assign_ticket(&config, context).await,
            ActionType::UpdateTicketStatus => self.execute_update_status(&config, context).await,
            ActionType::UpdateTicketPriority => self.execute_update_priority(&config, context).await,
            ActionType::AddTicketComment => self.execute_add_comment(&config, context).await,
            ActionType::AddTicketTag => self.execute_add_tag(&config, context).await,
            ActionType::RemoveTicketTag => self.execute_remove_tag(&config, context).await,
            ActionType::EscalateTicket => self.execute_escalate(&config, context).await,

            // Assignment Actions
            ActionType::AssignToGroup => self.execute_assign_to_group(&config, context).await,
            ActionType::AssignRoundRobin => self.execute_assign_round_robin(&config, context).await,
            ActionType::AssignByWorkload => self.execute_assign_by_workload(&config, context).await,
            ActionType::AssignBySkill => self.execute_assign_by_skill(&config, context).await,

            // Notification Actions
            ActionType::SendEmail => self.execute_send_email(&config, context).await,
            ActionType::SendTeamsNotification => self.execute_send_teams(&config, context).await,
            ActionType::SendWebhook => self.execute_send_webhook(&config, context).await,
            ActionType::CreateNotification => self.execute_create_notification(&config, context).await,

            // SLA Actions
            ActionType::ApplySlaPolicy => self.execute_apply_sla(&config, context).await,
            ActionType::PauseSla => self.execute_pause_sla(&config, context).await,
            ActionType::ResumeSla => self.execute_resume_sla(&config, context).await,

            // Data Actions
            ActionType::SetField => self.execute_set_field(&config, context).await,
            ActionType::IncrementField => self.execute_increment_field(&config, context).await,
            ActionType::CopyField => self.execute_copy_field(&config, context).await,

            // Control Flow
            ActionType::Wait => self.execute_wait(&config).await,
            ActionType::CallWorkflow => self.execute_call_workflow(&config, context).await,
            ActionType::StopWorkflow => Ok(ActionResult::success(Some(serde_json::json!({"stopped": true})))),

            // Integration Actions
            ActionType::CallApi => self.execute_call_api(&config, context).await,

            // Default/unimplemented
            _ => Ok(ActionResult::success(None)),
        };

        let duration = start.elapsed().as_millis() as i64;

        match result {
            Ok(mut r) => {
                r.duration_ms = duration;
                Ok(r)
            }
            Err(e) => {
                // Retry logic
                if action.retry_count > 0 {
                    for attempt in 1..=action.retry_count {
                        warn!("Action {} failed, retrying ({}/{})", action.name, attempt, action.retry_count);
                        tokio::time::sleep(tokio::time::Duration::from_secs(action.retry_delay_seconds as u64)).await;

                        let retry_result = self.execute_action(action, context).await;
                        if let Ok(mut r) = retry_result {
                            r.retry_attempted = attempt;
                            return Ok(r);
                        }
                    }
                }

                error!("Action {} failed: {}", action.name, e);
                Ok(ActionResult::failure(&e.to_string()).with_duration(duration))
            }
        }
    }

    /// Process template variables in configuration
    fn process_templates(&self, config: &serde_json::Value, context: &ExecutionContext) -> serde_json::Value {
        match config {
            serde_json::Value::String(s) => {
                let processed = self.replace_template_vars(s, context);
                serde_json::Value::String(processed)
            }
            serde_json::Value::Object(map) => {
                let processed: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), self.process_templates(v, context)))
                    .collect();
                serde_json::Value::Object(processed)
            }
            serde_json::Value::Array(arr) => {
                let processed: Vec<serde_json::Value> = arr
                    .iter()
                    .map(|v| self.process_templates(v, context))
                    .collect();
                serde_json::Value::Array(processed)
            }
            _ => config.clone(),
        }
    }

    fn replace_template_vars(&self, template: &str, context: &ExecutionContext) -> String {
        let mut result = template.to_string();

        // Replace {{field}} patterns with values from payload
        let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();

        for cap in re.captures_iter(template) {
            let var_path = &cap[1];
            let value = self.get_nested_value(&context.event_payload, var_path)
                .or_else(|| context.variables.get(var_path).cloned());

            if let Some(val) = value {
                let replacement = match val {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => val.to_string(),
                };
                result = result.replace(&cap[0], &replacement);
            }
        }

        result
    }

    fn get_nested_value(&self, json: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;

        for part in parts {
            match current.get(part) {
                Some(v) => current = v,
                None => return None,
            }
        }

        Some(current.clone())
    }

    // ===== Action Implementations =====

    async fn execute_assign_ticket(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let user_id: Uuid = serde_json::from_value(config["user_id"].clone())?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        sqlx::query("UPDATE tickets SET assigned_to = $2, updated_at = NOW() WHERE id = $1")
            .bind(ticket_id)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "assigned_to": user_id
        }))))
    }

    async fn execute_update_status(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let status = config["status"].as_str().ok_or("Missing status")?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        sqlx::query("UPDATE tickets SET status = $2, updated_at = NOW() WHERE id = $1")
            .bind(ticket_id)
            .bind(status)
            .execute(&self.db_pool)
            .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "new_status": status
        }))))
    }

    async fn execute_update_priority(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let priority = config["priority"].as_str().ok_or("Missing priority")?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        sqlx::query("UPDATE tickets SET priority = $2, updated_at = NOW() WHERE id = $1")
            .bind(ticket_id)
            .bind(priority)
            .execute(&self.db_pool)
            .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "new_priority": priority
        }))))
    }

    async fn execute_add_comment(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let comment = config["comment"].as_str().ok_or("Missing comment")?;
        let internal = config["internal"].as_bool().unwrap_or(false);
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        let comment_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO ticket_comments (id, ticket_id, content, is_internal, created_by_system, created_at)
             VALUES ($1, $2, $3, $4, true, NOW())"
        )
        .bind(comment_id)
        .bind(ticket_id)
        .bind(comment)
        .bind(internal)
        .execute(&self.db_pool)
        .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "comment_id": comment_id,
            "ticket_id": ticket_id
        }))))
    }

    async fn execute_add_tag(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let tag = config["tag"].as_str().ok_or("Missing tag")?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        // Ensure tag exists
        let tag_id: Uuid = sqlx::query_scalar(
            "INSERT INTO ticket_tags (id, name, created_at)
             VALUES ($1, $2, NOW())
             ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
             RETURNING id"
        )
        .bind(Uuid::new_v4())
        .bind(tag)
        .fetch_one(&self.db_pool)
        .await?;

        // Link tag to ticket
        sqlx::query(
            "INSERT INTO ticket_tag_assignments (ticket_id, tag_id)
             VALUES ($1, $2)
             ON CONFLICT DO NOTHING"
        )
        .bind(ticket_id)
        .bind(tag_id)
        .execute(&self.db_pool)
        .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "tag": tag
        }))))
    }

    async fn execute_remove_tag(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let tag = config["tag"].as_str().ok_or("Missing tag")?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        sqlx::query(
            "DELETE FROM ticket_tag_assignments
             WHERE ticket_id = $1
             AND tag_id = (SELECT id FROM ticket_tags WHERE name = $2)"
        )
        .bind(ticket_id)
        .bind(tag)
        .execute(&self.db_pool)
        .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "removed_tag": tag
        }))))
    }

    async fn execute_escalate(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let to_user_id: Uuid = serde_json::from_value(config["to_user_id"].clone())?;
        let reason = config["reason"].as_str().unwrap_or("Escalated by workflow");
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        // Update ticket assignment
        sqlx::query("UPDATE tickets SET assigned_to = $2, escalated = true, updated_at = NOW() WHERE id = $1")
            .bind(ticket_id)
            .bind(to_user_id)
            .execute(&self.db_pool)
            .await?;

        // Add escalation comment
        sqlx::query(
            "INSERT INTO ticket_comments (id, ticket_id, content, is_internal, created_by_system, created_at)
             VALUES ($1, $2, $3, true, true, NOW())"
        )
        .bind(Uuid::new_v4())
        .bind(ticket_id)
        .bind(format!("🔺 Ticket escalated: {}", reason))
        .execute(&self.db_pool)
        .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "escalated_to": to_user_id,
            "reason": reason
        }))))
    }

    async fn execute_assign_to_group(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let group_id: Uuid = serde_json::from_value(config["group_id"].clone())?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        sqlx::query("UPDATE tickets SET assigned_group_id = $2, updated_at = NOW() WHERE id = $1")
            .bind(ticket_id)
            .bind(group_id)
            .execute(&self.db_pool)
            .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "assigned_group": group_id
        }))))
    }

    async fn execute_assign_round_robin(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let user_ids: Vec<Uuid> = serde_json::from_value(config["user_ids"].clone())?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        // Get last assigned user for round robin
        let last_assigned: Option<(Uuid,)> = sqlx::query_as(
            "SELECT assigned_to FROM tickets WHERE assigned_to = ANY($1) ORDER BY updated_at DESC LIMIT 1"
        )
        .bind(&user_ids)
        .fetch_optional(&self.db_pool)
        .await?;

        let next_user = match last_assigned {
            Some((last_id,)) => {
                let idx = user_ids.iter().position(|&id| id == last_id).unwrap_or(0);
                user_ids[(idx + 1) % user_ids.len()]
            }
            None => user_ids[0],
        };

        sqlx::query("UPDATE tickets SET assigned_to = $2, updated_at = NOW() WHERE id = $1")
            .bind(ticket_id)
            .bind(next_user)
            .execute(&self.db_pool)
            .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "assigned_to": next_user
        }))))
    }

    async fn execute_assign_by_workload(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let user_ids: Vec<Uuid> = serde_json::from_value(config["user_ids"].clone())?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        // Find user with lowest open ticket count
        let least_loaded: (Uuid,) = sqlx::query_as(
            r#"
            SELECT u.id
            FROM users u
            WHERE u.id = ANY($1)
            ORDER BY (
                SELECT COUNT(*) FROM tickets t
                WHERE t.assigned_to = u.id
                AND t.status NOT IN ('resolved', 'closed')
            ) ASC
            LIMIT 1
            "#
        )
        .bind(&user_ids)
        .fetch_one(&self.db_pool)
        .await?;

        sqlx::query("UPDATE tickets SET assigned_to = $2, updated_at = NOW() WHERE id = $1")
            .bind(ticket_id)
            .bind(least_loaded.0)
            .execute(&self.db_pool)
            .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "assigned_to": least_loaded.0
        }))))
    }

    async fn execute_assign_by_skill(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let skill_tags: Vec<String> = serde_json::from_value(config["skill_tags"].clone())?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        // Find user with matching skills
        let matched_user: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT u.id
            FROM users u
            JOIN user_skills us ON u.id = us.user_id
            WHERE us.skill_name = ANY($1)
            AND u.is_active = true
            GROUP BY u.id
            ORDER BY COUNT(*) DESC, (
                SELECT COUNT(*) FROM tickets t
                WHERE t.assigned_to = u.id AND t.status NOT IN ('resolved', 'closed')
            ) ASC
            LIMIT 1
            "#
        )
        .bind(&skill_tags)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some((user_id,)) = matched_user {
            sqlx::query("UPDATE tickets SET assigned_to = $2, updated_at = NOW() WHERE id = $1")
                .bind(ticket_id)
                .bind(user_id)
                .execute(&self.db_pool)
                .await?;

            Ok(ActionResult::success(Some(serde_json::json!({
                "ticket_id": ticket_id,
                "assigned_to": user_id,
                "matched_skills": skill_tags
            }))))
        } else {
            Ok(ActionResult::failure("No user found with matching skills"))
        }
    }

    async fn execute_send_email(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let to = config["to"].as_str().ok_or("Missing 'to' address")?;
        let subject = config["subject"].as_str().ok_or("Missing subject")?;
        let body = config["body"].as_str().ok_or("Missing body")?;

        self.email_service.send_email(to, None, subject, body, None).await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "sent_to": to,
            "subject": subject
        }))))
    }

    async fn execute_send_teams(&self, config: &serde_json::Value, _context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let channel = config["channel"].as_str().ok_or("Missing channel")?;
        let message = config["message"].as_str().ok_or("Missing message")?;

        // Teams notification would use the TeamsNotificationService
        // For now, we'll just log it
        info!("Teams notification to {}: {}", channel, message);

        Ok(ActionResult::success(Some(serde_json::json!({
            "channel": channel,
            "message": message
        }))))
    }

    async fn execute_send_webhook(&self, config: &serde_json::Value, _context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let url = config["url"].as_str().ok_or("Missing URL")?;
        let method = config["method"].as_str().unwrap_or("POST");
        let payload = &config["payload"];

        let client = reqwest::Client::new();
        let response = match method.to_uppercase().as_str() {
            "GET" => client.get(url).send().await?,
            "POST" => client.post(url).json(payload).send().await?,
            "PUT" => client.put(url).json(payload).send().await?,
            _ => return Err("Unsupported HTTP method".into()),
        };

        let status = response.status().as_u16();

        Ok(ActionResult::success(Some(serde_json::json!({
            "url": url,
            "status_code": status
        }))))
    }

    async fn execute_create_notification(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let user_id: Uuid = if config["user_id"].is_null() || config["user_id"] == serde_json::json!(Uuid::nil()) {
            // Use assigned_to from payload if user_id is nil
            context.event_payload["assigned_to"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(Uuid::new_v4)
        } else {
            serde_json::from_value(config["user_id"].clone())?
        };

        let title = config["title"].as_str().ok_or("Missing title")?;
        let message = config["message"].as_str().ok_or("Missing message")?;
        let notification_type = config["type"].as_str().unwrap_or("info");

        let notification_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO notifications (id, user_id, title, message, type, created_at)
             VALUES ($1, $2, $3, $4, $5, NOW())"
        )
        .bind(notification_id)
        .bind(user_id)
        .bind(title)
        .bind(message)
        .bind(notification_type)
        .execute(&self.db_pool)
        .await?;

        // Broadcast via WebSocket
        self.ws_manager.send_to_user(
            user_id,
            &format!(r#"{{"type": "notification", "data": {{"id": "{}", "title": "{}", "message": "{}"}}}}"#,
                notification_id, title, message)
        ).await;

        Ok(ActionResult::success(Some(serde_json::json!({
            "notification_id": notification_id,
            "user_id": user_id
        }))))
    }

    async fn execute_apply_sla(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let policy_id: Uuid = serde_json::from_value(config["policy_id"].clone())?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        // Get SLA rules for the policy
        let rule: (Uuid, i32, i32) = sqlx::query_as(
            "SELECT id, response_time_minutes, resolution_time_hours FROM sla_rules
             WHERE policy_id = $1 ORDER BY execution_order LIMIT 1"
        )
        .bind(policy_id)
        .fetch_one(&self.db_pool)
        .await?;

        let now = Utc::now();
        let response_due = now + chrono::Duration::minutes(rule.1 as i64);
        let resolution_due = now + chrono::Duration::hours(rule.2 as i64);

        sqlx::query(
            "INSERT INTO ticket_sla_tracking
             (id, ticket_id, sla_policy_id, sla_rule_id, response_due_at, resolution_due_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW())
             ON CONFLICT (ticket_id) DO UPDATE
             SET sla_policy_id = $3, sla_rule_id = $4, response_due_at = $5, resolution_due_at = $6, updated_at = NOW()"
        )
        .bind(Uuid::new_v4())
        .bind(ticket_id)
        .bind(policy_id)
        .bind(rule.0)
        .bind(response_due)
        .bind(resolution_due)
        .execute(&self.db_pool)
        .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "policy_id": policy_id,
            "response_due_at": response_due,
            "resolution_due_at": resolution_due
        }))))
    }

    async fn execute_pause_sla(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;
        let reason = config["reason"].as_str().unwrap_or("Paused by workflow");

        sqlx::query(
            "UPDATE ticket_sla_tracking SET pause_start = NOW(), updated_at = NOW() WHERE ticket_id = $1"
        )
        .bind(ticket_id)
        .execute(&self.db_pool)
        .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id,
            "reason": reason
        }))))
    }

    async fn execute_resume_sla(&self, _config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        sqlx::query(
            "UPDATE ticket_sla_tracking
             SET pause_duration_minutes = pause_duration_minutes + EXTRACT(EPOCH FROM (NOW() - pause_start)) / 60,
                 pause_start = NULL,
                 updated_at = NOW()
             WHERE ticket_id = $1"
        )
        .bind(ticket_id)
        .execute(&self.db_pool)
        .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "ticket_id": ticket_id
        }))))
    }

    async fn execute_set_field(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let field = config["field"].as_str().ok_or("Missing field")?;
        let value = &config["value"];
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        // Determine table and column from field path
        let (table, column) = if field.contains('.') {
            let parts: Vec<&str> = field.split('.').collect();
            (parts[0], parts[1])
        } else {
            ("tickets", field)
        };

        // Build and execute update query
        let query = format!("UPDATE {} SET {} = $2, updated_at = NOW() WHERE id = $1", table, column);
        sqlx::query(&query)
            .bind(ticket_id)
            .bind(value.as_str().unwrap_or(&value.to_string()))
            .execute(&self.db_pool)
            .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "field": field,
            "value": value
        }))))
    }

    async fn execute_increment_field(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let field = config["field"].as_str().ok_or("Missing field")?;
        let amount = config["amount"].as_i64().unwrap_or(1);
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        let query = format!("UPDATE tickets SET {} = {} + $2, updated_at = NOW() WHERE id = $1", field, field);
        sqlx::query(&query)
            .bind(ticket_id)
            .bind(amount as i32)
            .execute(&self.db_pool)
            .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "field": field,
            "incremented_by": amount
        }))))
    }

    async fn execute_copy_field(&self, config: &serde_json::Value, context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let from = config["from"].as_str().ok_or("Missing 'from' field")?;
        let to = config["to"].as_str().ok_or("Missing 'to' field")?;
        let ticket_id: Uuid = serde_json::from_value(context.event_payload["ticket_id"].clone())?;

        let query = format!("UPDATE tickets SET {} = {}, updated_at = NOW() WHERE id = $1", to, from);
        sqlx::query(&query)
            .bind(ticket_id)
            .execute(&self.db_pool)
            .await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "from": from,
            "to": to
        }))))
    }

    async fn execute_wait(&self, config: &serde_json::Value) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let seconds = config["seconds"].as_u64().unwrap_or(0);

        tokio::time::sleep(tokio::time::Duration::from_secs(seconds)).await;

        Ok(ActionResult::success(Some(serde_json::json!({
            "waited_seconds": seconds
        }))))
    }

    async fn execute_call_workflow(&self, config: &serde_json::Value, _context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let workflow_id: Uuid = serde_json::from_value(config["workflow_id"].clone())?;

        // This would trigger another workflow execution
        // For now, just return success
        Ok(ActionResult::success(Some(serde_json::json!({
            "called_workflow": workflow_id
        }))))
    }

    async fn execute_call_api(&self, config: &serde_json::Value, _context: &ExecutionContext) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let url = config["url"].as_str().ok_or("Missing URL")?;
        let method = config["method"].as_str().unwrap_or("GET");
        let headers = config["headers"].as_object();
        let body = &config["body"];

        let client = reqwest::Client::new();
        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            _ => return Err("Unsupported HTTP method".into()),
        };

        if let Some(hdrs) = headers {
            for (key, value) in hdrs {
                if let Some(v) = value.as_str() {
                    request = request.header(key, v);
                }
            }
        }

        if !body.is_null() {
            request = request.json(body);
        }

        let response = request.send().await?;
        let status = response.status().as_u16();
        let body = response.text().await?;

        Ok(ActionResult::success(Some(serde_json::json!({
            "status_code": status,
            "response_body": body
        }))))
    }
}

impl ActionType {
    fn to_string(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string()
    }
}
