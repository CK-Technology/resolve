// Background Jobs Service
//
// This module provides scheduled background jobs for the Resolve MSP platform.
// Jobs are scheduled using tokio-cron-scheduler and run automatically at specified intervals.

pub mod scheduler;
pub mod sla_checker;
pub mod expiration_monitor;
pub mod recurring_billing;
pub mod maintenance;

pub use scheduler::{JobScheduler, JobConfig, JobResult, JobError};
pub use sla_checker::SlaCheckerJob;
pub use expiration_monitor::ExpirationMonitorJob;
pub use recurring_billing::RecurringBillingJob;
pub use maintenance::MaintenanceJobs;
