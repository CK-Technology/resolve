-- Polish Phase: Performance Indexes, Audit Logging, and Query Optimization
-- This migration adds missing indexes and audit infrastructure

-- ============================================
-- PERFORMANCE INDEXES
-- ============================================

-- Tickets - common query patterns
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tickets_client_status ON tickets(client_id, status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tickets_assigned_status ON tickets(assigned_to, status) WHERE assigned_to IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tickets_priority_status ON tickets(priority, status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tickets_created_desc ON tickets(created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tickets_queue_status ON tickets(queue_id, status) WHERE queue_id IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tickets_sla_due ON tickets(sla_resolution_due) WHERE status NOT IN ('resolved', 'closed');

-- Time entries - billing and reporting queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_time_entries_user_date ON time_entries(user_id, start_time DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_time_entries_billable_unbilled ON time_entries(billable, billed) WHERE billable = true AND billed = false;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_time_entries_ticket ON time_entries(ticket_id) WHERE ticket_id IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_time_entries_project ON time_entries(project_id) WHERE project_id IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_time_entries_invoice ON time_entries(invoice_id) WHERE invoice_id IS NOT NULL;

-- Invoices - billing queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_invoices_client_status ON invoices(client_id, status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_invoices_due_date ON invoices(due_date) WHERE status NOT IN ('paid', 'cancelled');
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_invoices_overdue ON invoices(due_date, status) WHERE due_date < CURRENT_DATE AND status NOT IN ('paid', 'cancelled');
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_invoices_date_desc ON invoices(date DESC);

-- Clients - active and search
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_clients_active ON clients(is_active) WHERE is_active = true;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_clients_name_trgm ON clients USING gin(name gin_trgm_ops);

-- Assets - client and status
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_assets_client_status ON assets(client_id, status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_assets_warranty ON assets(warranty_expiry) WHERE warranty_expiry IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_assets_serial ON assets(serial_number) WHERE serial_number IS NOT NULL;

-- Users - active and role
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_active_role ON users(is_active, role_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_email ON users(email) WHERE is_active = true;

-- Notification integrations - active webhook lookup
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_notification_integrations_active_type ON notification_integrations(integration_type, is_active) WHERE is_active = true;

-- Recurring invoices - due for processing
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recurring_templates_due ON recurring_invoice_templates(next_run_date, is_active) WHERE is_active = true;

-- SLA tracking indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ticket_sla_tracking_breach ON ticket_sla_tracking(is_breached, ticket_id) WHERE is_breached = true;

-- ============================================
-- AUDIT LOGGING INFRASTRUCTURE
-- ============================================

-- Comprehensive audit log table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Who
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    user_email VARCHAR(255), -- Denormalized for historical reference
    api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    ip_address INET,
    user_agent TEXT,

    -- What
    action VARCHAR(50) NOT NULL, -- create, update, delete, login, logout, export, etc.
    resource_type VARCHAR(100) NOT NULL, -- ticket, client, invoice, etc.
    resource_id UUID,
    resource_name VARCHAR(255), -- Human-readable identifier

    -- Details
    changes JSONB, -- For updates: {field: {old: x, new: y}}
    metadata JSONB, -- Additional context
    request_id UUID, -- For correlating related actions

    -- Security flags
    is_sensitive BOOLEAN DEFAULT false, -- Marks sensitive operations (password changes, etc.)
    severity VARCHAR(20) DEFAULT 'info', -- info, warning, critical

    -- Timing
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for audit log queries
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id, created_at DESC);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action, created_at DESC);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_sensitive ON audit_logs(is_sensitive, created_at DESC) WHERE is_sensitive = true;
CREATE INDEX idx_audit_logs_severity ON audit_logs(severity, created_at DESC) WHERE severity != 'info';

-- Partition audit logs by month for better performance (optional, for high-volume systems)
-- This would require additional setup in production

-- ============================================
-- CACHE TRACKING TABLE
-- ============================================

CREATE TABLE IF NOT EXISTS cache_entries (
    key VARCHAR(255) PRIMARY KEY,
    value JSONB NOT NULL,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    hit_count INTEGER DEFAULT 0
);

CREATE INDEX idx_cache_entries_expires ON cache_entries(expires_at) WHERE expires_at IS NOT NULL;

-- Function to get or set cache
CREATE OR REPLACE FUNCTION cache_get_or_set(
    p_key VARCHAR(255),
    p_default_value JSONB,
    p_ttl_seconds INTEGER DEFAULT 300
) RETURNS JSONB AS $$
DECLARE
    v_result JSONB;
    v_expires_at TIMESTAMPTZ;
BEGIN
    -- Try to get existing non-expired entry
    SELECT value INTO v_result
    FROM cache_entries
    WHERE key = p_key AND (expires_at IS NULL OR expires_at > NOW());

    IF FOUND THEN
        -- Update hit count
        UPDATE cache_entries SET hit_count = hit_count + 1 WHERE key = p_key;
        RETURN v_result;
    END IF;

    -- Calculate expiration
    v_expires_at := NOW() + (p_ttl_seconds || ' seconds')::interval;

    -- Insert or update with default value
    INSERT INTO cache_entries (key, value, expires_at)
    VALUES (p_key, p_default_value, v_expires_at)
    ON CONFLICT (key) DO UPDATE
    SET value = p_default_value, expires_at = v_expires_at, updated_at = NOW();

    RETURN p_default_value;
END;
$$ LANGUAGE plpgsql;

-- Function to invalidate cache entries by pattern
CREATE OR REPLACE FUNCTION cache_invalidate(p_pattern VARCHAR(255)) RETURNS INTEGER AS $$
DECLARE
    v_count INTEGER;
BEGIN
    DELETE FROM cache_entries WHERE key LIKE p_pattern;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    RETURN v_count;
END;
$$ LANGUAGE plpgsql;

-- Clean up expired cache entries (run periodically)
CREATE OR REPLACE FUNCTION cache_cleanup() RETURNS INTEGER AS $$
DECLARE
    v_count INTEGER;
BEGIN
    DELETE FROM cache_entries WHERE expires_at < NOW();
    GET DIAGNOSTICS v_count = ROW_COUNT;
    RETURN v_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- MATERIALIZED VIEWS FOR DASHBOARD
-- ============================================

-- Dashboard stats materialized view
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_dashboard_stats AS
SELECT
    -- Overview
    (SELECT COUNT(*) FROM clients WHERE is_active = true) as total_clients,
    (SELECT COUNT(*) FROM tickets WHERE status NOT IN ('resolved', 'closed')) as active_tickets,
    (SELECT COALESCE(SUM(total), 0) FROM invoices WHERE date >= date_trunc('month', CURRENT_DATE)) as monthly_revenue,
    (SELECT COALESCE(SUM(total_amount), 0) FROM time_entries WHERE billable = true AND billed = false) as unbilled_time,
    (SELECT COUNT(*) FROM invoices WHERE due_date < CURRENT_DATE AND status NOT IN ('paid', 'cancelled')) as overdue_invoices,

    -- Tickets by status
    (SELECT COUNT(*) FROM tickets WHERE status = 'open') as tickets_open,
    (SELECT COUNT(*) FROM tickets WHERE status = 'in_progress') as tickets_in_progress,
    (SELECT COUNT(*) FROM tickets WHERE status = 'pending') as tickets_pending,
    (SELECT COUNT(*) FROM tickets WHERE resolved_at::date = CURRENT_DATE) as tickets_resolved_today,
    (SELECT COUNT(*) FROM ticket_sla_tracking WHERE is_breached = true) as sla_breaches,

    -- Time stats
    (SELECT COALESCE(SUM(duration_minutes), 0)::decimal / 60 FROM time_entries WHERE start_time::date = CURRENT_DATE) as hours_today,
    (SELECT COALESCE(SUM(duration_minutes), 0)::decimal / 60 FROM time_entries WHERE billable = true AND start_time::date = CURRENT_DATE) as billable_hours_today,
    (SELECT COUNT(*) FROM time_entries WHERE end_time IS NULL) as active_timers,

    -- Timestamp
    NOW() as refreshed_at;

CREATE UNIQUE INDEX ON mv_dashboard_stats (refreshed_at);

-- Function to refresh dashboard stats
CREATE OR REPLACE FUNCTION refresh_dashboard_stats() RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_dashboard_stats;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- REQUEST TRACKING FOR OBSERVABILITY
-- ============================================

CREATE TABLE IF NOT EXISTS request_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL,

    -- Request info
    method VARCHAR(10) NOT NULL,
    path VARCHAR(500) NOT NULL,
    query_params JSONB,

    -- User info
    user_id UUID,
    api_key_id UUID,
    ip_address INET,

    -- Response info
    status_code INTEGER,
    response_time_ms INTEGER,
    error_code VARCHAR(50),
    error_message TEXT,

    -- Timestamps
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ
);

-- Indexes for request log queries
CREATE INDEX idx_request_logs_started ON request_logs(started_at DESC);
CREATE INDEX idx_request_logs_user ON request_logs(user_id, started_at DESC) WHERE user_id IS NOT NULL;
CREATE INDEX idx_request_logs_path ON request_logs(path, started_at DESC);
CREATE INDEX idx_request_logs_errors ON request_logs(status_code, started_at DESC) WHERE status_code >= 400;

-- Partition request logs by day (optional)
-- In production, you'd set up partitioning for this high-volume table

-- ============================================
-- HEALTH CHECK TABLE
-- ============================================

CREATE TABLE IF NOT EXISTS health_check_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL, -- healthy, degraded, unhealthy
    response_time_ms INTEGER,
    details JSONB,
    checked_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_health_history_service ON health_check_history(service, checked_at DESC);

-- Clean up old health checks (keep last 24 hours)
CREATE OR REPLACE FUNCTION cleanup_health_history() RETURNS void AS $$
BEGIN
    DELETE FROM health_check_history WHERE checked_at < NOW() - INTERVAL '24 hours';
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- METRICS AGGREGATION
-- ============================================

CREATE TABLE IF NOT EXISTS metrics_hourly (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name VARCHAR(100) NOT NULL,
    metric_type VARCHAR(20) NOT NULL, -- counter, gauge, histogram
    value DECIMAL(20,6) NOT NULL,
    labels JSONB DEFAULT '{}',
    hour TIMESTAMPTZ NOT NULL,
    UNIQUE(metric_name, labels, hour)
);

CREATE INDEX idx_metrics_hourly_name ON metrics_hourly(metric_name, hour DESC);
CREATE INDEX idx_metrics_hourly_hour ON metrics_hourly(hour DESC);

-- Function to record metric
CREATE OR REPLACE FUNCTION record_metric(
    p_name VARCHAR(100),
    p_type VARCHAR(20),
    p_value DECIMAL(20,6),
    p_labels JSONB DEFAULT '{}'
) RETURNS void AS $$
DECLARE
    v_hour TIMESTAMPTZ;
BEGIN
    v_hour := date_trunc('hour', NOW());

    INSERT INTO metrics_hourly (metric_name, metric_type, value, labels, hour)
    VALUES (p_name, p_type, p_value, p_labels, v_hour)
    ON CONFLICT (metric_name, labels, hour)
    DO UPDATE SET value = CASE
        WHEN p_type = 'counter' THEN metrics_hourly.value + p_value
        ELSE p_value
    END;
END;
$$ LANGUAGE plpgsql;

-- Comments
COMMENT ON TABLE audit_logs IS 'Comprehensive audit trail for all system changes';
COMMENT ON TABLE cache_entries IS 'Database-backed cache for expensive queries';
COMMENT ON MATERIALIZED VIEW mv_dashboard_stats IS 'Pre-computed dashboard statistics, refresh every 5 minutes';
COMMENT ON TABLE request_logs IS 'API request/response logging for observability';
COMMENT ON TABLE metrics_hourly IS 'Hourly aggregated metrics for monitoring';
