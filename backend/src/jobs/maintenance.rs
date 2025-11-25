// Maintenance Jobs - Database cleanup, metrics aggregation, and system maintenance tasks

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::{error, info, warn};

pub struct MaintenanceJobs;

impl MaintenanceJobs {
    /// Aggregate metrics data for reporting
    pub async fn aggregate_metrics(db_pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting metrics aggregation");

        // Aggregate ticket metrics
        sqlx::query(
            r#"
            INSERT INTO metrics_hourly (metric_type, metric_key, value, timestamp)
            SELECT
                'tickets_created',
                'count',
                COUNT(*)::decimal,
                date_trunc('hour', NOW())
            FROM tickets
            WHERE created_at >= date_trunc('hour', NOW()) - INTERVAL '1 hour'
                AND created_at < date_trunc('hour', NOW())
            ON CONFLICT (metric_type, metric_key, timestamp) DO UPDATE
            SET value = EXCLUDED.value
            "#
        )
        .execute(db_pool)
        .await?;

        // Aggregate ticket resolution time
        sqlx::query(
            r#"
            INSERT INTO metrics_hourly (metric_type, metric_key, value, timestamp)
            SELECT
                'avg_resolution_time',
                'hours',
                COALESCE(AVG(EXTRACT(EPOCH FROM (resolved_at - created_at)) / 3600), 0)::decimal,
                date_trunc('hour', NOW())
            FROM tickets
            WHERE resolved_at >= date_trunc('hour', NOW()) - INTERVAL '1 hour'
                AND resolved_at < date_trunc('hour', NOW())
            ON CONFLICT (metric_type, metric_key, timestamp) DO UPDATE
            SET value = EXCLUDED.value
            "#
        )
        .execute(db_pool)
        .await?;

        // Aggregate time entry hours
        sqlx::query(
            r#"
            INSERT INTO metrics_hourly (metric_type, metric_key, value, timestamp)
            SELECT
                'hours_logged',
                'total',
                COALESCE(SUM(duration_minutes) / 60.0, 0)::decimal,
                date_trunc('hour', NOW())
            FROM time_entries
            WHERE created_at >= date_trunc('hour', NOW()) - INTERVAL '1 hour'
                AND created_at < date_trunc('hour', NOW())
            ON CONFLICT (metric_type, metric_key, timestamp) DO UPDATE
            SET value = EXCLUDED.value
            "#
        )
        .execute(db_pool)
        .await?;

        // Aggregate billable vs non-billable
        sqlx::query(
            r#"
            INSERT INTO metrics_hourly (metric_type, metric_key, value, timestamp)
            SELECT
                'billable_ratio',
                'percentage',
                CASE
                    WHEN SUM(duration_minutes) > 0 THEN
                        (SUM(CASE WHEN billable THEN duration_minutes ELSE 0 END)::decimal /
                         SUM(duration_minutes)::decimal * 100)
                    ELSE 0
                END,
                date_trunc('hour', NOW())
            FROM time_entries
            WHERE created_at >= date_trunc('hour', NOW()) - INTERVAL '1 hour'
                AND created_at < date_trunc('hour', NOW())
            ON CONFLICT (metric_type, metric_key, timestamp) DO UPDATE
            SET value = EXCLUDED.value
            "#
        )
        .execute(db_pool)
        .await?;

        // Aggregate SLA compliance
        sqlx::query(
            r#"
            INSERT INTO metrics_hourly (metric_type, metric_key, value, timestamp)
            SELECT
                'sla_compliance',
                'percentage',
                CASE
                    WHEN COUNT(*) > 0 THEN
                        (COUNT(*) FILTER (WHERE NOT response_breached AND NOT resolution_breached)::decimal /
                         COUNT(*)::decimal * 100)
                    ELSE 100
                END,
                date_trunc('hour', NOW())
            FROM ticket_sla_tracking st
            JOIN tickets t ON st.ticket_id = t.id
            WHERE t.created_at >= date_trunc('hour', NOW()) - INTERVAL '1 hour'
                AND t.created_at < date_trunc('hour', NOW())
            ON CONFLICT (metric_type, metric_key, timestamp) DO UPDATE
            SET value = EXCLUDED.value
            "#
        )
        .execute(db_pool)
        .await?;

        // Roll up hourly to daily (at midnight)
        let hour = Utc::now().hour();
        if hour == 0 {
            Self::rollup_daily_metrics(db_pool).await?;
        }

        info!("Metrics aggregation completed");
        Ok(())
    }

    async fn rollup_daily_metrics(db_pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Rolling up hourly metrics to daily");

        sqlx::query(
            r#"
            INSERT INTO metrics_daily (metric_type, metric_key, avg_value, min_value, max_value, sum_value, count, date)
            SELECT
                metric_type,
                metric_key,
                AVG(value),
                MIN(value),
                MAX(value),
                SUM(value),
                COUNT(*),
                (NOW() - INTERVAL '1 day')::date
            FROM metrics_hourly
            WHERE timestamp >= (NOW() - INTERVAL '1 day')::date
                AND timestamp < NOW()::date
            GROUP BY metric_type, metric_key
            ON CONFLICT (metric_type, metric_key, date) DO UPDATE
            SET avg_value = EXCLUDED.avg_value,
                min_value = EXCLUDED.min_value,
                max_value = EXCLUDED.max_value,
                sum_value = EXCLUDED.sum_value,
                count = EXCLUDED.count
            "#
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(db_pool: &PgPool) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        info!("Cleaning up expired sessions");

        let result = sqlx::query(
            "DELETE FROM user_sessions WHERE expires_at < NOW()"
        )
        .execute(db_pool)
        .await?;

        let deleted = result.rows_affected() as i64;

        if deleted > 0 {
            info!("Deleted {} expired sessions", deleted);
        }

        // Also clean up expired refresh tokens
        let refresh_result = sqlx::query(
            "DELETE FROM refresh_tokens WHERE expires_at < NOW()"
        )
        .execute(db_pool)
        .await?;

        let refresh_deleted = refresh_result.rows_affected() as i64;

        if refresh_deleted > 0 {
            info!("Deleted {} expired refresh tokens", refresh_deleted);
        }

        // Clean up expired API keys
        let api_key_result = sqlx::query(
            "DELETE FROM api_keys WHERE expires_at IS NOT NULL AND expires_at < NOW()"
        )
        .execute(db_pool)
        .await?;

        let api_keys_deleted = api_key_result.rows_affected() as i64;

        if api_keys_deleted > 0 {
            info!("Deleted {} expired API keys", api_keys_deleted);
        }

        Ok(deleted + refresh_deleted + api_keys_deleted)
    }

    /// Clean up old audit logs beyond retention period
    pub async fn cleanup_old_audit_logs(db_pool: &PgPool, retention_days: i32) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        info!("Cleaning up audit logs older than {} days", retention_days);

        // First, archive important audit entries before deletion
        sqlx::query(
            r#"
            INSERT INTO audit_log_archive (id, user_id, action, resource_type, resource_id, details, ip_address, created_at)
            SELECT id, user_id, action, resource_type, resource_id, details, ip_address, created_at
            FROM audit_log
            WHERE created_at < NOW() - ($1 || ' days')::interval
                AND severity IN ('critical', 'high')
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(retention_days)
        .execute(db_pool)
        .await?;

        // Delete old audit logs
        let result = sqlx::query(
            "DELETE FROM audit_log WHERE created_at < NOW() - ($1 || ' days')::interval"
        )
        .bind(retention_days)
        .execute(db_pool)
        .await?;

        let deleted = result.rows_affected() as i64;

        if deleted > 0 {
            info!("Deleted {} old audit log entries", deleted);
        }

        Ok(deleted)
    }

    /// Clean up orphaned files not referenced by any record
    pub async fn cleanup_orphaned_files(db_pool: &PgPool) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        info!("Cleaning up orphaned files");

        // Mark files for deletion that have no references
        let result = sqlx::query(
            r#"
            UPDATE files
            SET deleted_at = NOW()
            WHERE id IN (
                SELECT f.id FROM files f
                LEFT JOIN ticket_attachments ta ON f.id = ta.file_id
                LEFT JOIN kb_article_attachments ka ON f.id = ka.file_id
                LEFT JOIN asset_documents ad ON f.id = ad.file_id
                WHERE ta.id IS NULL
                    AND ka.id IS NULL
                    AND ad.id IS NULL
                    AND f.created_at < NOW() - INTERVAL '24 hours'
                    AND f.deleted_at IS NULL
            )
            "#
        )
        .execute(db_pool)
        .await?;

        let marked = result.rows_affected() as i64;

        if marked > 0 {
            info!("Marked {} orphaned files for deletion", marked);
        }

        // Actually delete files marked more than 7 days ago
        let delete_result = sqlx::query(
            "DELETE FROM files WHERE deleted_at < NOW() - INTERVAL '7 days'"
        )
        .execute(db_pool)
        .await?;

        let deleted = delete_result.rows_affected() as i64;

        if deleted > 0 {
            info!("Permanently deleted {} orphaned files", deleted);
        }

        Ok(marked + deleted)
    }

    /// Run VACUUM ANALYZE to optimize database performance
    pub async fn vacuum_analyze(db_pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Running VACUUM ANALYZE on key tables");

        // Note: VACUUM ANALYZE cannot run in a transaction, so we use ANALYZE instead
        // which can run within a transaction and still updates statistics
        let tables = vec![
            "tickets",
            "time_entries",
            "clients",
            "invoices",
            "audit_log",
            "ticket_sla_tracking",
            "assets",
        ];

        for table in tables {
            if let Err(e) = sqlx::query(&format!("ANALYZE {}", table))
                .execute(db_pool)
                .await
            {
                warn!("Failed to ANALYZE {}: {}", table, e);
            }
        }

        info!("ANALYZE completed for key tables");

        // Update table statistics
        sqlx::query(
            r#"
            INSERT INTO system_stats (stat_key, stat_value, updated_at)
            SELECT 'table_' || relname, pg_size_pretty(pg_total_relation_size(relid)), NOW()
            FROM pg_stat_user_tables
            WHERE schemaname = 'public'
            ON CONFLICT (stat_key) DO UPDATE
            SET stat_value = EXCLUDED.stat_value, updated_at = NOW()
            "#
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }

    /// Clean up old notification records
    pub async fn cleanup_old_notifications(db_pool: &PgPool, retention_days: i32) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        info!("Cleaning up notifications older than {} days", retention_days);

        let result = sqlx::query(
            r#"
            DELETE FROM notifications
            WHERE created_at < NOW() - ($1 || ' days')::interval
                AND read_at IS NOT NULL
            "#
        )
        .bind(retention_days)
        .execute(db_pool)
        .await?;

        let deleted = result.rows_affected() as i64;

        if deleted > 0 {
            info!("Deleted {} old read notifications", deleted);
        }

        Ok(deleted)
    }

    /// Update calculated fields and denormalized data
    pub async fn update_calculated_fields(db_pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Updating calculated fields");

        // Update client ticket counts
        sqlx::query(
            r#"
            UPDATE clients c
            SET
                open_ticket_count = (SELECT COUNT(*) FROM tickets t WHERE t.client_id = c.id AND t.status NOT IN ('resolved', 'closed')),
                total_ticket_count = (SELECT COUNT(*) FROM tickets t WHERE t.client_id = c.id),
                updated_at = NOW()
            WHERE EXISTS (
                SELECT 1 FROM tickets t
                WHERE t.client_id = c.id
                AND t.updated_at > COALESCE(c.stats_updated_at, '1970-01-01')
            )
            "#
        )
        .execute(db_pool)
        .await?;

        // Update project progress
        sqlx::query(
            r#"
            UPDATE projects p
            SET
                completed_tasks = (SELECT COUNT(*) FROM project_tasks pt WHERE pt.project_id = p.id AND pt.status = 'completed'),
                total_tasks = (SELECT COUNT(*) FROM project_tasks pt WHERE pt.project_id = p.id),
                progress_percentage = CASE
                    WHEN (SELECT COUNT(*) FROM project_tasks pt WHERE pt.project_id = p.id) > 0 THEN
                        (SELECT COUNT(*) FROM project_tasks pt WHERE pt.project_id = p.id AND pt.status = 'completed')::decimal /
                        (SELECT COUNT(*) FROM project_tasks pt WHERE pt.project_id = p.id)::decimal * 100
                    ELSE 0
                END,
                updated_at = NOW()
            "#
        )
        .execute(db_pool)
        .await?;

        // Update invoice aging
        sqlx::query(
            r#"
            UPDATE invoices
            SET
                days_overdue = GREATEST(0, EXTRACT(DAY FROM (NOW() - due_date))::integer),
                status = CASE
                    WHEN status IN ('sent', 'viewed') AND due_date < CURRENT_DATE THEN 'overdue'
                    ELSE status
                END,
                updated_at = NOW()
            WHERE status NOT IN ('paid', 'cancelled', 'void')
            "#
        )
        .execute(db_pool)
        .await?;

        info!("Calculated fields updated");
        Ok(())
    }

    /// Generate daily summary report data
    pub async fn generate_daily_summary(db_pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Generating daily summary");

        let yesterday = Utc::now().date_naive() - chrono::Duration::days(1);

        sqlx::query(
            r#"
            INSERT INTO daily_summaries (
                date,
                tickets_created,
                tickets_resolved,
                tickets_escalated,
                total_hours_logged,
                billable_hours,
                revenue_generated,
                new_clients,
                sla_compliance_rate,
                created_at
            )
            SELECT
                $1 as date,
                (SELECT COUNT(*) FROM tickets WHERE created_at::date = $1),
                (SELECT COUNT(*) FROM tickets WHERE resolved_at::date = $1),
                (SELECT COUNT(*) FROM ticket_sla_tracking WHERE escalated_at::date = $1),
                COALESCE((SELECT SUM(duration_minutes) / 60.0 FROM time_entries WHERE start_time::date = $1), 0),
                COALESCE((SELECT SUM(duration_minutes) / 60.0 FROM time_entries WHERE start_time::date = $1 AND billable = true), 0),
                COALESCE((SELECT SUM(total_amount) FROM invoices WHERE issue_date = $1), 0),
                (SELECT COUNT(*) FROM clients WHERE created_at::date = $1),
                COALESCE((
                    SELECT
                        COUNT(*) FILTER (WHERE NOT st.response_breached AND NOT st.resolution_breached)::decimal /
                        NULLIF(COUNT(*)::decimal, 0) * 100
                    FROM ticket_sla_tracking st
                    JOIN tickets t ON st.ticket_id = t.id
                    WHERE t.created_at::date = $1
                ), 100),
                NOW()
            ON CONFLICT (date) DO UPDATE
            SET
                tickets_created = EXCLUDED.tickets_created,
                tickets_resolved = EXCLUDED.tickets_resolved,
                tickets_escalated = EXCLUDED.tickets_escalated,
                total_hours_logged = EXCLUDED.total_hours_logged,
                billable_hours = EXCLUDED.billable_hours,
                revenue_generated = EXCLUDED.revenue_generated,
                new_clients = EXCLUDED.new_clients,
                sla_compliance_rate = EXCLUDED.sla_compliance_rate,
                updated_at = NOW()
            "#
        )
        .bind(yesterday)
        .execute(db_pool)
        .await?;

        info!("Daily summary generated for {}", yesterday);
        Ok(())
    }
}
