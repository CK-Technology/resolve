-- Reporting & Analytics Module for Resolve
-- Executive dashboards, custom reports, scheduled delivery, data exports, client health scores

-- Report definitions and templates
CREATE TABLE reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL, -- financial, operational, client, technical, executive
    report_type VARCHAR(50) NOT NULL, -- dashboard, table, chart, kpi, custom
    
    -- Data source configuration
    data_sources JSONB NOT NULL, -- Tables/views to query
    base_query TEXT, -- Base SQL query template
    filters JSONB DEFAULT '[]', -- Available filters
    parameters JSONB DEFAULT '[]', -- Report parameters
    
    -- Visualization
    chart_type VARCHAR(50), -- bar, line, pie, scatter, table
    chart_config JSONB DEFAULT '{}', -- Chart-specific configuration
    layout_config JSONB DEFAULT '{}', -- Dashboard layout
    
    -- Access control
    visibility VARCHAR(50) DEFAULT 'private', -- private, team, company, client
    allowed_users UUID[] DEFAULT '{}',
    allowed_roles VARCHAR(100)[] DEFAULT '{}',
    client_accessible BOOLEAN DEFAULT false,
    
    -- Caching
    cache_duration_minutes INTEGER DEFAULT 60,
    last_cached TIMESTAMPTZ,
    cache_data JSONB,
    
    -- Usage tracking
    view_count INTEGER DEFAULT 0,
    last_viewed TIMESTAMPTZ,
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    is_template BOOLEAN DEFAULT false,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Scheduled report deliveries
CREATE TABLE scheduled_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id UUID NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    
    -- Schedule configuration
    schedule_type VARCHAR(50) NOT NULL, -- once, daily, weekly, monthly, quarterly
    schedule_cron VARCHAR(100), -- Cron expression for complex schedules
    timezone VARCHAR(50) DEFAULT 'UTC',
    
    -- Delivery settings
    delivery_method VARCHAR(50) DEFAULT 'email', -- email, slack, webhook, ftp
    recipients TEXT[] NOT NULL,
    subject_template VARCHAR(500),
    message_template TEXT,
    
    -- Format options
    export_format VARCHAR(20) DEFAULT 'pdf', -- pdf, excel, csv, html
    include_charts BOOLEAN DEFAULT true,
    include_raw_data BOOLEAN DEFAULT false,
    
    -- Filters and parameters
    report_filters JSONB DEFAULT '{}',
    report_parameters JSONB DEFAULT '{}',
    
    -- Execution tracking
    next_delivery TIMESTAMPTZ,
    last_delivery TIMESTAMPTZ,
    delivery_count INTEGER DEFAULT 0,
    last_status VARCHAR(50), -- success, failed, skipped
    last_error TEXT,
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Report execution history
CREATE TABLE report_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id UUID NOT NULL REFERENCES reports(id),
    scheduled_report_id UUID REFERENCES scheduled_reports(id),
    
    -- Execution details
    executed_by UUID REFERENCES users(id),
    execution_type VARCHAR(50), -- manual, scheduled, api
    parameters JSONB DEFAULT '{}',
    filters JSONB DEFAULT '{}',
    
    -- Results
    status VARCHAR(50) NOT NULL DEFAULT 'running', -- running, completed, failed
    row_count INTEGER,
    execution_time_ms INTEGER,
    file_size_bytes BIGINT,
    
    -- Output
    output_format VARCHAR(20),
    output_file_path VARCHAR(500),
    download_url VARCHAR(500),
    expires_at TIMESTAMPTZ,
    
    -- Error handling
    error_message TEXT,
    stack_trace TEXT,
    
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    
    -- Cleanup
    cleaned_up BOOLEAN DEFAULT false
);

-- Dashboard widgets
CREATE TABLE dashboard_widgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID REFERENCES reports(id), -- Parent dashboard report
    name VARCHAR(255) NOT NULL,
    widget_type VARCHAR(50) NOT NULL, -- metric, chart, table, alert, iframe
    
    -- Layout
    position_x INTEGER DEFAULT 0,
    position_y INTEGER DEFAULT 0,
    width INTEGER DEFAULT 4,
    height INTEGER DEFAULT 3,
    
    -- Data configuration
    data_source JSONB NOT NULL,
    refresh_interval_seconds INTEGER DEFAULT 300,
    
    -- Display options
    title VARCHAR(255),
    show_title BOOLEAN DEFAULT true,
    color_scheme VARCHAR(50) DEFAULT 'default',
    custom_styling JSONB DEFAULT '{}',
    
    -- Interactivity
    clickable BOOLEAN DEFAULT false,
    drill_down_report_id UUID REFERENCES reports(id),
    
    -- Status
    is_visible BOOLEAN DEFAULT true,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- KPI definitions and tracking
CREATE TABLE kpis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100), -- financial, operational, customer, technical
    
    -- Calculation
    calculation_query TEXT NOT NULL,
    calculation_frequency VARCHAR(50) DEFAULT 'daily', -- hourly, daily, weekly, monthly
    unit VARCHAR(50), -- dollars, percent, count, hours, etc.
    format_pattern VARCHAR(100), -- Number formatting pattern
    
    -- Targets and thresholds
    target_value DECIMAL(15,4),
    warning_threshold DECIMAL(15,4),
    critical_threshold DECIMAL(15,4),
    good_direction VARCHAR(10) DEFAULT 'up', -- up, down (whether higher is better)
    
    -- Display
    chart_type VARCHAR(50) DEFAULT 'line',
    color_good VARCHAR(7) DEFAULT '#10b981',
    color_warning VARCHAR(7) DEFAULT '#f59e0b',
    color_critical VARCHAR(7) DEFAULT '#ef4444',
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    last_calculated TIMESTAMPTZ,
    current_value DECIMAL(15,4),
    previous_value DECIMAL(15,4),
    trend VARCHAR(20), -- improving, stable, declining
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- KPI historical values
CREATE TABLE kpi_values (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kpi_id UUID NOT NULL REFERENCES kpis(id) ON DELETE CASCADE,
    value DECIMAL(15,4) NOT NULL,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(kpi_id, period_start)
);

-- Client health scores
CREATE TABLE client_health_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    
    -- Overall score (0-100)
    overall_score INTEGER NOT NULL,
    score_trend VARCHAR(20), -- improving, stable, declining
    
    -- Component scores
    asset_health_score INTEGER DEFAULT 50,
    ticket_satisfaction_score INTEGER DEFAULT 50,
    financial_health_score INTEGER DEFAULT 50,
    communication_score INTEGER DEFAULT 50,
    security_score INTEGER DEFAULT 50,
    
    -- Risk indicators
    risk_level VARCHAR(20) DEFAULT 'medium', -- low, medium, high, critical
    risk_factors TEXT[],
    recommendations TEXT[],
    
    -- Calculation metadata
    calculation_date DATE NOT NULL,
    data_completeness_percent INTEGER DEFAULT 100,
    calculation_version VARCHAR(20) DEFAULT '1.0',
    
    -- Alerts
    alert_sent BOOLEAN DEFAULT false,
    alert_sent_at TIMESTAMPTZ,
    
    UNIQUE(client_id, calculation_date)
);

-- Custom report builder field definitions
CREATE TABLE report_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    description TEXT,
    table_name VARCHAR(255) NOT NULL,
    column_name VARCHAR(255) NOT NULL,
    data_type VARCHAR(50) NOT NULL, -- string, integer, decimal, date, boolean
    
    -- Display options
    is_filterable BOOLEAN DEFAULT true,
    is_groupable BOOLEAN DEFAULT true,
    is_sortable BOOLEAN DEFAULT true,
    default_aggregation VARCHAR(50), -- sum, avg, count, min, max
    
    -- Formatting
    format_type VARCHAR(50), -- currency, percent, date, number
    format_pattern VARCHAR(100),
    
    -- Categories for organization
    category VARCHAR(100),
    subcategory VARCHAR(100),
    
    -- Access control
    requires_permission VARCHAR(100),
    
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Data export configurations
CREATE TABLE data_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Data selection
    base_query TEXT NOT NULL,
    parameters JSONB DEFAULT '{}',
    
    -- Export options
    export_format VARCHAR(20) DEFAULT 'csv', -- csv, excel, json, xml
    include_headers BOOLEAN DEFAULT true,
    date_format VARCHAR(50) DEFAULT 'yyyy-mm-dd',
    delimiter VARCHAR(5) DEFAULT ',',
    
    -- Scheduling
    is_scheduled BOOLEAN DEFAULT false,
    schedule_cron VARCHAR(100),
    timezone VARCHAR(50) DEFAULT 'UTC',
    
    -- Delivery
    delivery_method VARCHAR(50) DEFAULT 'download', -- download, email, ftp, s3
    delivery_config JSONB DEFAULT '{}',
    
    -- Access control
    created_by UUID REFERENCES users(id),
    allowed_users UUID[] DEFAULT '{}',
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    last_executed TIMESTAMPTZ,
    execution_count INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Business intelligence metrics aggregation
CREATE TABLE bi_metrics_daily (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_date DATE NOT NULL,
    client_id UUID REFERENCES clients(id), -- NULL for company-wide metrics
    
    -- Financial metrics
    revenue_total DECIMAL(10,2) DEFAULT 0,
    revenue_recurring DECIMAL(10,2) DEFAULT 0,
    revenue_project DECIMAL(10,2) DEFAULT 0,
    expenses_total DECIMAL(10,2) DEFAULT 0,
    profit_gross DECIMAL(10,2) DEFAULT 0,
    profit_margin_percent DECIMAL(5,2) DEFAULT 0,
    
    -- Operational metrics
    tickets_created INTEGER DEFAULT 0,
    tickets_resolved INTEGER DEFAULT 0,
    tickets_escalated INTEGER DEFAULT 0,
    avg_resolution_time_hours DECIMAL(8,2) DEFAULT 0,
    sla_breaches INTEGER DEFAULT 0,
    
    -- Time metrics
    hours_billable DECIMAL(8,2) DEFAULT 0,
    hours_non_billable DECIMAL(8,2) DEFAULT 0,
    utilization_rate DECIMAL(5,2) DEFAULT 0,
    efficiency_rate DECIMAL(5,2) DEFAULT 0,
    
    -- Asset metrics
    assets_total INTEGER DEFAULT 0,
    assets_online INTEGER DEFAULT 0,
    assets_with_issues INTEGER DEFAULT 0,
    discovery_scans INTEGER DEFAULT 0,
    
    -- Communication metrics
    emails_sent INTEGER DEFAULT 0,
    emails_received INTEGER DEFAULT 0,
    portal_logins INTEGER DEFAULT 0,
    chat_messages INTEGER DEFAULT 0,
    
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(metric_date, client_id)
);

-- Report bookmarks for users
CREATE TABLE report_bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    report_id UUID NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    name VARCHAR(255),
    parameters JSONB DEFAULT '{}',
    filters JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, report_id)
);

-- Create indexes for performance
CREATE INDEX idx_reports_category ON reports(category);
CREATE INDEX idx_reports_visibility ON reports(visibility);
CREATE INDEX idx_reports_is_active ON reports(is_active);
CREATE INDEX idx_scheduled_reports_next_delivery ON scheduled_reports(next_delivery) WHERE is_active = true;
CREATE INDEX idx_report_executions_report_id ON report_executions(report_id);
CREATE INDEX idx_report_executions_status ON report_executions(status);
CREATE INDEX idx_dashboard_widgets_dashboard_id ON dashboard_widgets(dashboard_id);
CREATE INDEX idx_kpis_category ON kpis(category);
CREATE INDEX idx_kpi_values_kpi_id ON kpi_values(kpi_id);
CREATE INDEX idx_kpi_values_period ON kpi_values(period_start, period_end);
CREATE INDEX idx_client_health_scores_client_id ON client_health_scores(client_id);
CREATE INDEX idx_client_health_scores_date ON client_health_scores(calculation_date);
CREATE INDEX idx_client_health_scores_risk ON client_health_scores(risk_level);
CREATE INDEX idx_bi_metrics_daily_date ON bi_metrics_daily(metric_date);
CREATE INDEX idx_bi_metrics_daily_client ON bi_metrics_daily(client_id);

-- Function to calculate client health score
CREATE OR REPLACE FUNCTION calculate_client_health_score(p_client_id UUID, p_date DATE DEFAULT CURRENT_DATE)
RETURNS INTEGER AS $$
DECLARE
    v_asset_health INTEGER := 50;
    v_ticket_satisfaction INTEGER := 50;
    v_financial_health INTEGER := 50;
    v_communication INTEGER := 50;
    v_security INTEGER := 50;
    v_overall_score INTEGER;
BEGIN
    -- Asset Health (30% weight)
    SELECT AVG(health_score)::INTEGER INTO v_asset_health
    FROM assets WHERE client_id = p_client_id;
    v_asset_health := COALESCE(v_asset_health, 50);
    
    -- Ticket Satisfaction (25% weight) 
    -- Based on resolution time, escalations, and recent ticket volume
    WITH ticket_metrics AS (
        SELECT 
            COUNT(*) as total_tickets,
            AVG(CASE WHEN sla_breached THEN 0 ELSE 100 END) as sla_compliance,
            COUNT(CASE WHEN created_at > p_date - INTERVAL '30 days' THEN 1 END) as recent_tickets
        FROM tickets 
        WHERE client_id = p_client_id 
        AND created_at > p_date - INTERVAL '90 days'
    )
    SELECT LEAST(100, sla_compliance - (recent_tickets * 2))::INTEGER INTO v_ticket_satisfaction
    FROM ticket_metrics;
    
    -- Financial Health (20% weight)
    -- Based on payment history, overdue amounts
    WITH financial_metrics AS (
        SELECT 
            COUNT(CASE WHEN status = 'overdue' THEN 1 END) as overdue_invoices,
            COUNT(CASE WHEN status = 'paid' THEN 1 END) as paid_invoices,
            SUM(CASE WHEN status = 'overdue' THEN total_amount ELSE 0 END) as overdue_amount
        FROM invoices 
        WHERE client_id = p_client_id 
        AND issue_date > p_date - INTERVAL '12 months'
    )
    SELECT 
        CASE 
            WHEN paid_invoices = 0 THEN 50
            WHEN overdue_amount > 10000 THEN 20
            WHEN overdue_invoices > 0 THEN 70 - (overdue_invoices * 10)
            ELSE 90
        END INTO v_financial_health
    FROM financial_metrics;
    
    -- Communication Score (15% weight)
    -- Based on portal usage, email responsiveness
    WITH comm_metrics AS (
        SELECT 
            COUNT(*) as portal_messages,
            AVG(EXTRACT(EPOCH FROM (responded_at - created_at))/3600) as avg_response_hours
        FROM portal_messages 
        WHERE client_id = p_client_id 
        AND created_at > p_date - INTERVAL '90 days'
        AND responded_at IS NOT NULL
    )
    SELECT 
        CASE 
            WHEN avg_response_hours IS NULL THEN 50
            WHEN avg_response_hours <= 4 THEN 90
            WHEN avg_response_hours <= 24 THEN 70
            ELSE 40
        END INTO v_communication
    FROM comm_metrics;
    
    -- Security Score (10% weight)
    -- Based on password policies, outdated software, security incidents
    WITH security_metrics AS (
        SELECT 
            COUNT(CASE WHEN breach_detected THEN 1 END) as breached_passwords,
            COUNT(CASE WHEN next_rotation_date < CURRENT_DATE THEN 1 END) as overdue_rotations
        FROM password_vault 
        WHERE client_id = p_client_id
    )
    SELECT 
        CASE 
            WHEN breached_passwords > 0 THEN 30
            WHEN overdue_rotations > 5 THEN 60
            WHEN overdue_rotations > 0 THEN 80
            ELSE 90
        END INTO v_security
    FROM security_metrics;
    
    -- Calculate weighted overall score
    v_overall_score := (
        (v_asset_health * 30 + 
         v_ticket_satisfaction * 25 + 
         v_financial_health * 20 + 
         v_communication * 15 + 
         v_security * 10) / 100
    );
    
    -- Insert or update health score record
    INSERT INTO client_health_scores (
        client_id, overall_score, asset_health_score, ticket_satisfaction_score,
        financial_health_score, communication_score, security_score,
        calculation_date, risk_level
    ) VALUES (
        p_client_id, v_overall_score, v_asset_health, v_ticket_satisfaction,
        v_financial_health, v_communication, v_security,
        p_date,
        CASE 
            WHEN v_overall_score >= 80 THEN 'low'
            WHEN v_overall_score >= 60 THEN 'medium'
            WHEN v_overall_score >= 40 THEN 'high'
            ELSE 'critical'
        END
    ) ON CONFLICT (client_id, calculation_date) 
    DO UPDATE SET
        overall_score = EXCLUDED.overall_score,
        asset_health_score = EXCLUDED.asset_health_score,
        ticket_satisfaction_score = EXCLUDED.ticket_satisfaction_score,
        financial_health_score = EXCLUDED.financial_health_score,
        communication_score = EXCLUDED.communication_score,
        security_score = EXCLUDED.security_score,
        risk_level = EXCLUDED.risk_level;
    
    RETURN v_overall_score;
END;
$$ LANGUAGE plpgsql;

-- Function to aggregate daily BI metrics
CREATE OR REPLACE FUNCTION aggregate_bi_metrics_daily(p_date DATE DEFAULT CURRENT_DATE)
RETURNS void AS $$
DECLARE
    client_record RECORD;
BEGIN
    -- Company-wide metrics
    INSERT INTO bi_metrics_daily (
        metric_date, client_id,
        revenue_total, tickets_created, tickets_resolved,
        hours_billable, assets_total
    )
    SELECT 
        p_date, NULL,
        COALESCE(SUM(i.total_amount), 0) as revenue_total,
        COALESCE(COUNT(t.id), 0) as tickets_created,
        COALESCE(COUNT(CASE WHEN t.status = 'resolved' THEN 1 END), 0) as tickets_resolved,
        COALESCE(SUM(te.duration_minutes) / 60.0, 0) as hours_billable,
        COALESCE(COUNT(DISTINCT a.id), 0) as assets_total
    FROM clients c
    LEFT JOIN invoices i ON c.id = i.client_id AND i.issue_date = p_date
    LEFT JOIN tickets t ON c.id = t.client_id AND DATE(t.created_at) = p_date
    LEFT JOIN time_entries te ON te.start_time::date = p_date AND te.billable = true
    LEFT JOIN assets a ON c.id = a.client_id
    ON CONFLICT (metric_date, client_id) DO UPDATE SET
        revenue_total = EXCLUDED.revenue_total,
        tickets_created = EXCLUDED.tickets_created,
        tickets_resolved = EXCLUDED.tickets_resolved,
        hours_billable = EXCLUDED.hours_billable,
        assets_total = EXCLUDED.assets_total;
    
    -- Per-client metrics
    FOR client_record IN SELECT id FROM clients WHERE is_active = true LOOP
        INSERT INTO bi_metrics_daily (
            metric_date, client_id,
            revenue_total, tickets_created, tickets_resolved, hours_billable
        )
        SELECT 
            p_date, client_record.id,
            COALESCE(SUM(i.total_amount), 0),
            COALESCE(COUNT(t.id), 0),
            COALESCE(COUNT(CASE WHEN t.status = 'resolved' THEN 1 END), 0),
            COALESCE(SUM(te.duration_minutes) / 60.0, 0)
        FROM invoices i
        FULL OUTER JOIN tickets t ON t.client_id = client_record.id AND DATE(t.created_at) = p_date
        FULL OUTER JOIN time_entries te ON EXISTS(
            SELECT 1 FROM tickets t2 WHERE t2.id = te.ticket_id AND t2.client_id = client_record.id
        ) AND te.start_time::date = p_date
        WHERE i.client_id = client_record.id AND i.issue_date = p_date
        ON CONFLICT (metric_date, client_id) DO UPDATE SET
            revenue_total = EXCLUDED.revenue_total,
            tickets_created = EXCLUDED.tickets_created,
            tickets_resolved = EXCLUDED.tickets_resolved,
            hours_billable = EXCLUDED.hours_billable;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Insert default reports
INSERT INTO reports (name, description, category, report_type, data_sources, base_query, chart_type) VALUES
('Executive Dashboard', 'High-level business metrics and KPIs', 'executive', 'dashboard', 
 '["bi_metrics_daily", "clients", "invoices"]'::jsonb,
 'SELECT * FROM bi_metrics_daily WHERE client_id IS NULL ORDER BY metric_date DESC LIMIT 30',
 'mixed'),
 
('Client Profitability Report', 'Revenue, costs, and profit margins by client', 'financial', 'table',
 '["client_profitability", "clients"]'::jsonb,
 'SELECT c.name, cp.* FROM client_profitability cp JOIN clients c ON c.id = cp.client_id WHERE cp.period_start >= $1 AND cp.period_end <= $2',
 'table'),
 
('Ticket Volume Trends', 'Ticket creation and resolution trends over time', 'operational', 'chart',
 '["bi_metrics_daily"]'::jsonb,
 'SELECT metric_date, tickets_created, tickets_resolved FROM bi_metrics_daily WHERE metric_date >= $1 ORDER BY metric_date',
 'line'),
 
('Asset Health Overview', 'Health scores and status of all client assets', 'technical', 'table',
 '["assets", "clients", "asset_warranties"]'::jsonb,
 'SELECT c.name as client, a.name as asset, a.health_score, aw.end_date as warranty_expires FROM assets a JOIN clients c ON c.id = a.client_id LEFT JOIN asset_warranties aw ON aw.asset_id = a.id',
 'table'),
 
('Client Health Scorecard', 'Overall health scores and risk assessment for clients', 'client', 'dashboard',
 '["client_health_scores", "clients"]'::jsonb,
 'SELECT c.name, chs.overall_score, chs.risk_level, chs.calculation_date FROM client_health_scores chs JOIN clients c ON c.id = chs.client_id WHERE chs.calculation_date = (SELECT MAX(calculation_date) FROM client_health_scores WHERE client_id = chs.client_id)',
 'mixed');

-- Insert default KPIs
INSERT INTO kpis (name, description, category, calculation_query, unit, target_value) VALUES
('Monthly Recurring Revenue', 'Total MRR from all clients', 'financial',
 'SELECT COALESCE(SUM(amount), 0) FROM recurring_billing WHERE status = ''active'' AND frequency = ''monthly''',
 'dollars', 50000),
 
('Average Ticket Resolution Time', 'Mean time to resolve tickets in hours', 'operational',
 'SELECT COALESCE(AVG(EXTRACT(EPOCH FROM (resolved_at - created_at))/3600), 0) FROM tickets WHERE resolved_at IS NOT NULL AND created_at > NOW() - INTERVAL ''30 days''',
 'hours', 8),
 
('Client Satisfaction Score', 'Average client health score', 'client',
 'SELECT COALESCE(AVG(overall_score), 0) FROM client_health_scores WHERE calculation_date = CURRENT_DATE',
 'score', 80),
 
('SLA Compliance Rate', 'Percentage of tickets meeting SLA', 'operational',
 'SELECT COALESCE(AVG(CASE WHEN sla_breached THEN 0 ELSE 100 END), 100) FROM tickets WHERE created_at > NOW() - INTERVAL ''30 days''',
 'percent', 95),
 
('Team Utilization Rate', 'Percentage of time spent on billable work', 'operational',
 'SELECT COALESCE(SUM(CASE WHEN billable THEN duration_minutes ELSE 0 END) * 100.0 / SUM(duration_minutes), 0) FROM time_entries WHERE start_time > NOW() - INTERVAL ''7 days''',
 'percent', 75);

-- Insert report fields for custom report builder
INSERT INTO report_fields (name, display_name, table_name, column_name, data_type, category) VALUES
('client_name', 'Client Name', 'clients', 'name', 'string', 'Client Information'),
('client_type', 'Client Type', 'clients', 'client_type', 'string', 'Client Information'),
('ticket_subject', 'Ticket Subject', 'tickets', 'subject', 'string', 'Tickets'),
('ticket_priority', 'Ticket Priority', 'tickets', 'priority', 'string', 'Tickets'),
('ticket_status', 'Ticket Status', 'tickets', 'status', 'string', 'Tickets'),
('ticket_created', 'Ticket Created', 'tickets', 'created_at', 'date', 'Tickets'),
('asset_name', 'Asset Name', 'assets', 'name', 'string', 'Assets'),
('asset_type', 'Asset Type', 'assets', 'asset_type', 'string', 'Assets'),
('invoice_total', 'Invoice Total', 'invoices', 'total_amount', 'decimal', 'Financial'),
('invoice_status', 'Invoice Status', 'invoices', 'status', 'string', 'Financial'),
('time_duration', 'Time Duration (Hours)', 'time_entries', 'duration_minutes', 'decimal', 'Time Tracking'),
('time_billable', 'Is Billable', 'time_entries', 'billable', 'boolean', 'Time Tracking');