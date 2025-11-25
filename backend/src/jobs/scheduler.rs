// Job Scheduler - Central scheduler for all background jobs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler as TokioScheduler, JobSchedulerError};
use tracing::{error, info, warn};
use uuid::Uuid;

use super::{SlaCheckerJob, ExpirationMonitorJob, RecurringBillingJob, MaintenanceJobs};
use crate::services::EmailService;
use crate::websocket::WsManager;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Scheduler error: {0}")]
    SchedulerError(#[from] JobSchedulerError),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Job execution error: {0}")]
    ExecutionError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type JobResult<T> = Result<T, JobError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    // SLA Checking
    pub sla_check_interval_minutes: u32,
    pub sla_breach_notification_interval_minutes: u32,
    pub sla_auto_escalation_enabled: bool,

    // Expiration Monitoring
    pub expiration_check_interval_hours: u32,
    pub domain_expiry_warning_days: Vec<i32>,
    pub ssl_expiry_warning_days: Vec<i32>,
    pub license_expiry_warning_days: Vec<i32>,
    pub warranty_expiry_warning_days: Vec<i32>,

    // Recurring Billing
    pub billing_check_interval_hours: u32,
    pub auto_invoice_enabled: bool,
    pub payment_reminder_enabled: bool,

    // Maintenance
    pub cleanup_interval_hours: u32,
    pub metrics_aggregation_interval_minutes: u32,
    pub audit_log_retention_days: i32,
    pub session_cleanup_interval_hours: u32,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            // SLA - Check every 5 minutes
            sla_check_interval_minutes: 5,
            sla_breach_notification_interval_minutes: 30,
            sla_auto_escalation_enabled: true,

            // Expirations - Check every 6 hours
            expiration_check_interval_hours: 6,
            domain_expiry_warning_days: vec![90, 60, 30, 14, 7, 3, 1],
            ssl_expiry_warning_days: vec![60, 30, 14, 7, 3, 1],
            license_expiry_warning_days: vec![90, 60, 30, 14, 7],
            warranty_expiry_warning_days: vec![90, 60, 30],

            // Billing - Check every 4 hours
            billing_check_interval_hours: 4,
            auto_invoice_enabled: true,
            payment_reminder_enabled: true,

            // Maintenance
            cleanup_interval_hours: 24,
            metrics_aggregation_interval_minutes: 15,
            audit_log_retention_days: 365,
            session_cleanup_interval_hours: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecutionLog {
    pub id: Uuid,
    pub job_name: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub items_processed: i32,
    pub errors: Vec<String>,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Running,
    Completed,
    Failed,
    PartialFailure,
}

pub struct JobScheduler {
    scheduler: TokioScheduler,
    db_pool: PgPool,
    email_service: EmailService,
    ws_manager: WsManager,
    config: JobConfig,
    execution_logs: Arc<RwLock<Vec<JobExecutionLog>>>,
}

impl JobScheduler {
    pub async fn new(
        db_pool: PgPool,
        email_service: EmailService,
        ws_manager: WsManager,
        config: JobConfig,
    ) -> JobResult<Self> {
        let scheduler = TokioScheduler::new().await?;

        Ok(Self {
            scheduler,
            db_pool,
            email_service,
            ws_manager,
            config,
            execution_logs: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn start(&self) -> JobResult<()> {
        info!("Starting background job scheduler");

        // Schedule SLA Checker
        self.schedule_sla_checker().await?;

        // Schedule Expiration Monitor
        self.schedule_expiration_monitor().await?;

        // Schedule Recurring Billing
        self.schedule_recurring_billing().await?;

        // Schedule Maintenance Jobs
        self.schedule_maintenance_jobs().await?;

        // Start the scheduler
        self.scheduler.start().await?;

        info!("Background job scheduler started successfully");
        Ok(())
    }

    pub async fn shutdown(&self) -> JobResult<()> {
        info!("Shutting down background job scheduler");
        self.scheduler.shutdown().await?;
        Ok(())
    }

    async fn schedule_sla_checker(&self) -> JobResult<()> {
        let interval = self.config.sla_check_interval_minutes;
        let cron_expr = format!("0 */{} * * * *", interval); // Every N minutes

        let db_pool = self.db_pool.clone();
        let email_service = self.email_service.clone();
        let ws_manager = self.ws_manager.clone();
        let config = self.config.clone();
        let logs = self.execution_logs.clone();

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let db_pool = db_pool.clone();
            let email_service = email_service.clone();
            let ws_manager = ws_manager.clone();
            let config = config.clone();
            let logs = logs.clone();

            Box::pin(async move {
                let log_id = Uuid::new_v4();
                let started_at = Utc::now();

                info!("Running SLA checker job");

                let checker = SlaCheckerJob::new(
                    db_pool.clone(),
                    email_service.clone(),
                    ws_manager.clone(),
                    config.sla_auto_escalation_enabled,
                );

                match checker.run().await {
                    Ok(result) => {
                        let completed_at = Utc::now();
                        let duration = (completed_at - started_at).num_milliseconds();

                        let log = JobExecutionLog {
                            id: log_id,
                            job_name: "SLA Checker".to_string(),
                            started_at,
                            completed_at: Some(completed_at),
                            status: if result.errors.is_empty() { JobStatus::Completed } else { JobStatus::PartialFailure },
                            items_processed: result.tickets_checked,
                            errors: result.errors,
                            duration_ms: Some(duration),
                        };

                        if let Ok(mut logs) = logs.write().await {
                            logs.push(log);
                            // Keep only last 100 logs
                            if logs.len() > 100 {
                                logs.remove(0);
                            }
                        }

                        info!("SLA checker completed: {} tickets checked, {} breaches found",
                              result.tickets_checked, result.breaches_detected);
                    }
                    Err(e) => {
                        error!("SLA checker failed: {}", e);
                    }
                }
            })
        })?;

        self.scheduler.add(job).await?;
        info!("Scheduled SLA checker to run every {} minutes", interval);

        Ok(())
    }

    async fn schedule_expiration_monitor(&self) -> JobResult<()> {
        let interval = self.config.expiration_check_interval_hours;
        let cron_expr = format!("0 0 */{} * * *", interval); // Every N hours

        let db_pool = self.db_pool.clone();
        let email_service = self.email_service.clone();
        let config = self.config.clone();
        let logs = self.execution_logs.clone();

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let db_pool = db_pool.clone();
            let email_service = email_service.clone();
            let config = config.clone();
            let logs = logs.clone();

            Box::pin(async move {
                let log_id = Uuid::new_v4();
                let started_at = Utc::now();

                info!("Running expiration monitor job");

                let monitor = ExpirationMonitorJob::new(
                    db_pool.clone(),
                    email_service.clone(),
                    config.domain_expiry_warning_days.clone(),
                    config.ssl_expiry_warning_days.clone(),
                    config.license_expiry_warning_days.clone(),
                    config.warranty_expiry_warning_days.clone(),
                );

                match monitor.run().await {
                    Ok(result) => {
                        let completed_at = Utc::now();
                        let duration = (completed_at - started_at).num_milliseconds();

                        let log = JobExecutionLog {
                            id: log_id,
                            job_name: "Expiration Monitor".to_string(),
                            started_at,
                            completed_at: Some(completed_at),
                            status: if result.errors.is_empty() { JobStatus::Completed } else { JobStatus::PartialFailure },
                            items_processed: result.total_items_checked,
                            errors: result.errors,
                            duration_ms: Some(duration),
                        };

                        if let Ok(mut logs) = logs.write().await {
                            logs.push(log);
                            if logs.len() > 100 {
                                logs.remove(0);
                            }
                        }

                        info!("Expiration monitor completed: {} items checked, {} alerts sent",
                              result.total_items_checked, result.alerts_sent);
                    }
                    Err(e) => {
                        error!("Expiration monitor failed: {}", e);
                    }
                }
            })
        })?;

        self.scheduler.add(job).await?;
        info!("Scheduled expiration monitor to run every {} hours", interval);

        Ok(())
    }

    async fn schedule_recurring_billing(&self) -> JobResult<()> {
        if !self.config.auto_invoice_enabled {
            info!("Auto-invoicing is disabled, skipping recurring billing job");
            return Ok(());
        }

        let interval = self.config.billing_check_interval_hours;
        let cron_expr = format!("0 0 */{} * * *", interval);

        let db_pool = self.db_pool.clone();
        let email_service = self.email_service.clone();
        let payment_reminder_enabled = self.config.payment_reminder_enabled;
        let logs = self.execution_logs.clone();

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let db_pool = db_pool.clone();
            let email_service = email_service.clone();
            let logs = logs.clone();

            Box::pin(async move {
                let log_id = Uuid::new_v4();
                let started_at = Utc::now();

                info!("Running recurring billing job");

                let billing = RecurringBillingJob::new(
                    db_pool.clone(),
                    email_service.clone(),
                    payment_reminder_enabled,
                );

                match billing.run().await {
                    Ok(result) => {
                        let completed_at = Utc::now();
                        let duration = (completed_at - started_at).num_milliseconds();

                        let log = JobExecutionLog {
                            id: log_id,
                            job_name: "Recurring Billing".to_string(),
                            started_at,
                            completed_at: Some(completed_at),
                            status: if result.errors.is_empty() { JobStatus::Completed } else { JobStatus::PartialFailure },
                            items_processed: result.invoices_generated + result.reminders_sent,
                            errors: result.errors,
                            duration_ms: Some(duration),
                        };

                        if let Ok(mut logs) = logs.write().await {
                            logs.push(log);
                            if logs.len() > 100 {
                                logs.remove(0);
                            }
                        }

                        info!("Recurring billing completed: {} invoices generated, {} reminders sent",
                              result.invoices_generated, result.reminders_sent);
                    }
                    Err(e) => {
                        error!("Recurring billing failed: {}", e);
                    }
                }
            })
        })?;

        self.scheduler.add(job).await?;
        info!("Scheduled recurring billing to run every {} hours", interval);

        Ok(())
    }

    async fn schedule_maintenance_jobs(&self) -> JobResult<()> {
        // Metrics aggregation - every 15 minutes
        self.schedule_metrics_aggregation().await?;

        // Session cleanup - every hour
        self.schedule_session_cleanup().await?;

        // Daily cleanup - once per day at 3 AM
        self.schedule_daily_cleanup().await?;

        Ok(())
    }

    async fn schedule_metrics_aggregation(&self) -> JobResult<()> {
        let interval = self.config.metrics_aggregation_interval_minutes;
        let cron_expr = format!("0 */{} * * * *", interval);

        let db_pool = self.db_pool.clone();

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let db_pool = db_pool.clone();

            Box::pin(async move {
                if let Err(e) = MaintenanceJobs::aggregate_metrics(&db_pool).await {
                    warn!("Metrics aggregation failed: {}", e);
                }
            })
        })?;

        self.scheduler.add(job).await?;
        info!("Scheduled metrics aggregation every {} minutes", interval);

        Ok(())
    }

    async fn schedule_session_cleanup(&self) -> JobResult<()> {
        let interval = self.config.session_cleanup_interval_hours;
        let cron_expr = format!("0 0 */{} * * *", interval);

        let db_pool = self.db_pool.clone();

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let db_pool = db_pool.clone();

            Box::pin(async move {
                if let Err(e) = MaintenanceJobs::cleanup_expired_sessions(&db_pool).await {
                    warn!("Session cleanup failed: {}", e);
                }
            })
        })?;

        self.scheduler.add(job).await?;
        info!("Scheduled session cleanup every {} hours", interval);

        Ok(())
    }

    async fn schedule_daily_cleanup(&self) -> JobResult<()> {
        let retention_days = self.config.audit_log_retention_days;
        let db_pool = self.db_pool.clone();

        // Run at 3 AM every day
        let job = Job::new_async("0 0 3 * * *", move |_uuid, _lock| {
            let db_pool = db_pool.clone();

            Box::pin(async move {
                info!("Running daily cleanup tasks");

                if let Err(e) = MaintenanceJobs::cleanup_old_audit_logs(&db_pool, retention_days).await {
                    warn!("Audit log cleanup failed: {}", e);
                }

                if let Err(e) = MaintenanceJobs::cleanup_orphaned_files(&db_pool).await {
                    warn!("Orphaned file cleanup failed: {}", e);
                }

                if let Err(e) = MaintenanceJobs::vacuum_analyze(&db_pool).await {
                    warn!("Vacuum analyze failed: {}", e);
                }

                info!("Daily cleanup completed");
            })
        })?;

        self.scheduler.add(job).await?;
        info!("Scheduled daily cleanup at 3 AM");

        Ok(())
    }

    pub async fn get_execution_logs(&self) -> Vec<JobExecutionLog> {
        self.execution_logs.read().await.clone()
    }

    pub async fn run_job_now(&self, job_name: &str) -> JobResult<()> {
        match job_name {
            "sla_checker" => {
                let checker = SlaCheckerJob::new(
                    self.db_pool.clone(),
                    self.email_service.clone(),
                    self.ws_manager.clone(),
                    self.config.sla_auto_escalation_enabled,
                );
                checker.run().await.map_err(|e| JobError::ExecutionError(e.to_string()))?;
            }
            "expiration_monitor" => {
                let monitor = ExpirationMonitorJob::new(
                    self.db_pool.clone(),
                    self.email_service.clone(),
                    self.config.domain_expiry_warning_days.clone(),
                    self.config.ssl_expiry_warning_days.clone(),
                    self.config.license_expiry_warning_days.clone(),
                    self.config.warranty_expiry_warning_days.clone(),
                );
                monitor.run().await.map_err(|e| JobError::ExecutionError(e.to_string()))?;
            }
            "recurring_billing" => {
                let billing = RecurringBillingJob::new(
                    self.db_pool.clone(),
                    self.email_service.clone(),
                    self.config.payment_reminder_enabled,
                );
                billing.run().await.map_err(|e| JobError::ExecutionError(e.to_string()))?;
            }
            _ => return Err(JobError::ConfigError(format!("Unknown job: {}", job_name))),
        }

        Ok(())
    }
}
