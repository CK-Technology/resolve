// Workflow Automation Engine
//
// Event-driven automation system for the Resolve MSP platform.
// Supports triggers, conditions, and actions for automated workflows.

pub mod engine;
pub mod triggers;
pub mod conditions;
pub mod actions;
pub mod executor;

pub use engine::{WorkflowEngine, WorkflowDefinition, WorkflowInstance};
pub use triggers::{TriggerType, TriggerEvent, EventPayload};
pub use conditions::{Condition, ConditionGroup, ConditionOperator, FieldCondition};
pub use actions::{Action, ActionType, ActionResult};
pub use executor::{WorkflowExecutor, ExecutionContext, ExecutionResult};
