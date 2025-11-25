// Workflow Engine - Core workflow processing and management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::{
    Action, ActionResult, ActionType, Condition, ConditionGroup,
    ExecutionContext, TriggerEvent, TriggerType, WorkflowExecutor,
};
use crate::services::EmailService;
use crate::websocket::WsManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
    pub trigger_config: serde_json::Value,
    pub conditions: Option<ConditionGroup>,
    pub actions: Vec<Action>,
    pub is_active: bool,
    pub execution_order: i32,
    pub stop_on_first_match: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowInstance {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub trigger_event_id: Uuid,
    pub status: WorkflowStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub actions_completed: i32,
    pub total_actions: i32,
    pub error_message: Option<String>,
    pub execution_log: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "workflow_status", rename_all = "snake_case")]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub struct WorkflowEngine {
    db_pool: PgPool,
    email_service: EmailService,
    ws_manager: WsManager,
    workflows: Arc<RwLock<Vec<WorkflowDefinition>>>,
    executor: WorkflowExecutor,
}

impl WorkflowEngine {
    pub async fn new(
        db_pool: PgPool,
        email_service: EmailService,
        ws_manager: WsManager,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let executor = WorkflowExecutor::new(db_pool.clone(), email_service.clone(), ws_manager.clone());

        let engine = Self {
            db_pool,
            email_service,
            ws_manager,
            workflows: Arc::new(RwLock::new(Vec::new())),
            executor,
        };

        // Load workflows from database
        engine.reload_workflows().await?;

        Ok(engine)
    }

    /// Reload all active workflows from database
    pub async fn reload_workflows(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let workflows = sqlx::query_as::<_, (
            Uuid, String, Option<String>, String, serde_json::Value,
            Option<serde_json::Value>, serde_json::Value, bool, i32, bool,
            Option<Uuid>, DateTime<Utc>, Option<DateTime<Utc>>
        )>(
            r#"
            SELECT
                id, name, description, trigger_type, trigger_config,
                conditions, actions, is_active, execution_order, stop_on_first_match,
                created_by, created_at, updated_at
            FROM workflows
            WHERE is_active = true
            ORDER BY execution_order ASC
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        let definitions: Vec<WorkflowDefinition> = workflows
            .into_iter()
            .filter_map(|row| {
                let trigger_type: TriggerType = serde_json::from_str(&format!("\"{}\"", row.3)).ok()?;
                let conditions: Option<ConditionGroup> = row.5.and_then(|c| serde_json::from_value(c).ok());
                let actions: Vec<Action> = serde_json::from_value(row.6).ok()?;

                Some(WorkflowDefinition {
                    id: row.0,
                    name: row.1,
                    description: row.2,
                    trigger_type,
                    trigger_config: row.4,
                    conditions,
                    actions,
                    is_active: row.7,
                    execution_order: row.8,
                    stop_on_first_match: row.9,
                    created_by: row.10,
                    created_at: row.11,
                    updated_at: row.12,
                })
            })
            .collect();

        let mut workflows = self.workflows.write().await;
        *workflows = definitions;

        info!("Loaded {} active workflows", workflows.len());
        Ok(())
    }

    /// Process a trigger event and execute matching workflows
    pub async fn process_event(&self, event: TriggerEvent) -> Result<Vec<Uuid>, Box<dyn std::error::Error + Send + Sync>> {
        let workflows = self.workflows.read().await;
        let mut executed_instances = Vec::new();

        info!("Processing event: {:?}", event.trigger_type);

        for workflow in workflows.iter() {
            // Check if workflow matches the trigger type
            if workflow.trigger_type != event.trigger_type {
                continue;
            }

            // Check trigger-specific conditions
            if !self.matches_trigger_config(&event, &workflow.trigger_config) {
                continue;
            }

            // Evaluate workflow conditions
            if let Some(conditions) = &workflow.conditions {
                if !self.evaluate_conditions(conditions, &event.payload).await {
                    continue;
                }
            }

            // Create workflow instance
            let instance_id = self.create_instance(workflow.id, &event).await?;

            // Execute workflow
            match self.execute_workflow(workflow, &event, instance_id).await {
                Ok(_) => {
                    executed_instances.push(instance_id);
                    info!("Workflow '{}' executed successfully", workflow.name);

                    if workflow.stop_on_first_match {
                        break;
                    }
                }
                Err(e) => {
                    error!("Workflow '{}' failed: {}", workflow.name, e);
                    self.mark_instance_failed(instance_id, &e.to_string()).await?;
                }
            }
        }

        Ok(executed_instances)
    }

    fn matches_trigger_config(&self, event: &TriggerEvent, config: &serde_json::Value) -> bool {
        // Check specific trigger configuration
        match event.trigger_type {
            TriggerType::TicketCreated => {
                // Check for specific client, category, priority filters
                if let Some(priority_filter) = config.get("priority") {
                    if let Some(event_priority) = event.payload.get("priority") {
                        if priority_filter != event_priority {
                            return false;
                        }
                    }
                }
                if let Some(client_filter) = config.get("client_id") {
                    if let Some(event_client) = event.payload.get("client_id") {
                        if client_filter != event_client {
                            return false;
                        }
                    }
                }
            }
            TriggerType::TicketStatusChanged => {
                if let Some(to_status) = config.get("to_status") {
                    if let Some(event_status) = event.payload.get("new_status") {
                        if to_status != event_status {
                            return false;
                        }
                    }
                }
            }
            TriggerType::SlaBreach => {
                if let Some(breach_type) = config.get("breach_type") {
                    if let Some(event_breach) = event.payload.get("breach_type") {
                        if breach_type != event_breach {
                            return false;
                        }
                    }
                }
            }
            _ => {}
        }

        true
    }

    async fn evaluate_conditions(&self, group: &ConditionGroup, payload: &serde_json::Value) -> bool {
        let results: Vec<bool> = group
            .conditions
            .iter()
            .map(|c| self.evaluate_condition(c, payload))
            .collect();

        match group.logic.as_str() {
            "AND" | "and" => results.iter().all(|&r| r),
            "OR" | "or" => results.iter().any(|&r| r),
            _ => results.iter().all(|&r| r),
        }
    }

    fn evaluate_condition(&self, condition: &Condition, payload: &serde_json::Value) -> bool {
        let field_value = payload.get(&condition.field);

        match condition.operator.as_str() {
            "equals" | "eq" | "==" => {
                field_value.map(|v| v == &condition.value).unwrap_or(false)
            }
            "not_equals" | "ne" | "!=" => {
                field_value.map(|v| v != &condition.value).unwrap_or(true)
            }
            "contains" => {
                if let Some(val) = field_value {
                    if let (Some(s), Some(pattern)) = (val.as_str(), condition.value.as_str()) {
                        return s.to_lowercase().contains(&pattern.to_lowercase());
                    }
                }
                false
            }
            "not_contains" => {
                if let Some(val) = field_value {
                    if let (Some(s), Some(pattern)) = (val.as_str(), condition.value.as_str()) {
                        return !s.to_lowercase().contains(&pattern.to_lowercase());
                    }
                }
                true
            }
            "starts_with" => {
                if let Some(val) = field_value {
                    if let (Some(s), Some(pattern)) = (val.as_str(), condition.value.as_str()) {
                        return s.to_lowercase().starts_with(&pattern.to_lowercase());
                    }
                }
                false
            }
            "ends_with" => {
                if let Some(val) = field_value {
                    if let (Some(s), Some(pattern)) = (val.as_str(), condition.value.as_str()) {
                        return s.to_lowercase().ends_with(&pattern.to_lowercase());
                    }
                }
                false
            }
            "greater_than" | "gt" | ">" => {
                if let Some(val) = field_value {
                    if let (Some(v), Some(c)) = (val.as_f64(), condition.value.as_f64()) {
                        return v > c;
                    }
                }
                false
            }
            "less_than" | "lt" | "<" => {
                if let Some(val) = field_value {
                    if let (Some(v), Some(c)) = (val.as_f64(), condition.value.as_f64()) {
                        return v < c;
                    }
                }
                false
            }
            "in" => {
                if let Some(val) = field_value {
                    if let Some(arr) = condition.value.as_array() {
                        return arr.contains(val);
                    }
                }
                false
            }
            "not_in" => {
                if let Some(val) = field_value {
                    if let Some(arr) = condition.value.as_array() {
                        return !arr.contains(val);
                    }
                }
                true
            }
            "is_null" | "is_empty" => {
                field_value.is_none() || field_value == Some(&serde_json::Value::Null)
            }
            "is_not_null" | "is_not_empty" => {
                field_value.is_some() && field_value != Some(&serde_json::Value::Null)
            }
            "regex" => {
                if let Some(val) = field_value {
                    if let (Some(s), Some(pattern)) = (val.as_str(), condition.value.as_str()) {
                        if let Ok(re) = regex::Regex::new(pattern) {
                            return re.is_match(s);
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    async fn create_instance(&self, workflow_id: Uuid, event: &TriggerEvent) -> Result<Uuid, sqlx::Error> {
        let instance_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO workflow_instances
            (id, workflow_id, trigger_event_id, status, started_at, actions_completed, total_actions, execution_log)
            VALUES ($1, $2, $3, 'running', NOW(), 0, 0, '[]'::jsonb)
            "#
        )
        .bind(instance_id)
        .bind(workflow_id)
        .bind(event.event_id)
        .execute(&self.db_pool)
        .await?;

        Ok(instance_id)
    }

    async fn execute_workflow(
        &self,
        workflow: &WorkflowDefinition,
        event: &TriggerEvent,
        instance_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let total_actions = workflow.actions.len() as i32;

        // Update total actions count
        sqlx::query("UPDATE workflow_instances SET total_actions = $2 WHERE id = $1")
            .bind(instance_id)
            .bind(total_actions)
            .execute(&self.db_pool)
            .await?;

        let context = ExecutionContext {
            instance_id,
            workflow_id: workflow.id,
            event_payload: event.payload.clone(),
            variables: HashMap::new(),
        };

        // Execute actions sequentially
        for (index, action) in workflow.actions.iter().enumerate() {
            // Check for delay
            if action.delay_seconds > 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(action.delay_seconds as u64)).await;
            }

            // Execute action
            let result = self.executor.execute_action(action, &context).await?;

            // Log action result
            self.log_action_execution(instance_id, index as i32, action, &result).await?;

            // Update progress
            sqlx::query("UPDATE workflow_instances SET actions_completed = $2 WHERE id = $1")
                .bind(instance_id)
                .bind((index + 1) as i32)
                .execute(&self.db_pool)
                .await?;

            // Check for action failure with stop flag
            if !result.success && action.stop_on_failure {
                return Err(format!("Action '{}' failed: {:?}", action.name, result.error).into());
            }
        }

        // Mark workflow as completed
        sqlx::query(
            "UPDATE workflow_instances SET status = 'completed', completed_at = NOW() WHERE id = $1"
        )
        .bind(instance_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn log_action_execution(
        &self,
        instance_id: Uuid,
        action_index: i32,
        action: &Action,
        result: &ActionResult,
    ) -> Result<(), sqlx::Error> {
        let log_entry = serde_json::json!({
            "action_index": action_index,
            "action_name": action.name,
            "action_type": action.action_type,
            "success": result.success,
            "error": result.error,
            "output": result.output,
            "executed_at": Utc::now().to_rfc3339()
        });

        sqlx::query(
            "UPDATE workflow_instances SET execution_log = execution_log || $2 WHERE id = $1"
        )
        .bind(instance_id)
        .bind(log_entry)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn mark_instance_failed(&self, instance_id: Uuid, error: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE workflow_instances SET status = 'failed', error_message = $2, completed_at = NOW() WHERE id = $1"
        )
        .bind(instance_id)
        .bind(error)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Create a new workflow
    pub async fn create_workflow(&self, definition: WorkflowDefinition) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            INSERT INTO workflows
            (id, name, description, trigger_type, trigger_config, conditions, actions, is_active, execution_order, stop_on_first_match, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
            "#
        )
        .bind(definition.id)
        .bind(&definition.name)
        .bind(&definition.description)
        .bind(serde_json::to_string(&definition.trigger_type)?.trim_matches('"'))
        .bind(&definition.trigger_config)
        .bind(definition.conditions.as_ref().map(|c| serde_json::to_value(c).ok()).flatten())
        .bind(serde_json::to_value(&definition.actions)?)
        .bind(definition.is_active)
        .bind(definition.execution_order)
        .bind(definition.stop_on_first_match)
        .bind(definition.created_by)
        .execute(&self.db_pool)
        .await?;

        // Reload workflows
        self.reload_workflows().await?;

        Ok(definition.id)
    }

    /// Update an existing workflow
    pub async fn update_workflow(&self, definition: WorkflowDefinition) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            UPDATE workflows
            SET name = $2, description = $3, trigger_type = $4, trigger_config = $5,
                conditions = $6, actions = $7, is_active = $8, execution_order = $9,
                stop_on_first_match = $10, updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(definition.id)
        .bind(&definition.name)
        .bind(&definition.description)
        .bind(serde_json::to_string(&definition.trigger_type)?.trim_matches('"'))
        .bind(&definition.trigger_config)
        .bind(definition.conditions.as_ref().map(|c| serde_json::to_value(c).ok()).flatten())
        .bind(serde_json::to_value(&definition.actions)?)
        .bind(definition.is_active)
        .bind(definition.execution_order)
        .bind(definition.stop_on_first_match)
        .execute(&self.db_pool)
        .await?;

        // Reload workflows
        self.reload_workflows().await?;

        Ok(())
    }

    /// Delete a workflow
    pub async fn delete_workflow(&self, workflow_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM workflows WHERE id = $1")
            .bind(workflow_id)
            .execute(&self.db_pool)
            .await?;

        // Reload workflows
        self.reload_workflows().await.ok();

        Ok(())
    }

    /// Get workflow execution history
    pub async fn get_execution_history(
        &self,
        workflow_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<WorkflowInstance>, sqlx::Error> {
        let mut query = "SELECT * FROM workflow_instances".to_string();

        if workflow_id.is_some() {
            query.push_str(" WHERE workflow_id = $1");
        }

        query.push_str(" ORDER BY started_at DESC LIMIT ");
        query.push_str(&limit.to_string());

        if let Some(wf_id) = workflow_id {
            sqlx::query_as::<_, WorkflowInstance>(&query)
                .bind(wf_id)
                .fetch_all(&self.db_pool)
                .await
        } else {
            sqlx::query_as::<_, WorkflowInstance>(&query)
                .fetch_all(&self.db_pool)
                .await
        }
    }
}
