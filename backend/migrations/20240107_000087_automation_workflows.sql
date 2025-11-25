-- Automation & Workflows System for Resolve
-- Scheduled tasks, webhooks, custom scripts, alert rules, and workflow automation

-- Workflow definitions
CREATE TABLE workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    workflow_type VARCHAR(50) NOT NULL, -- manual, scheduled, event_driven, webhook
    
    -- Trigger configuration
    trigger_type VARCHAR(100) NOT NULL, -- schedule, ticket_created, ticket_updated, invoice_sent, etc.
    trigger_config JSONB NOT NULL, -- Trigger-specific configuration
    
    -- Conditions
    conditions JSONB DEFAULT '[]', -- Array of condition objects
    
    -- Execution settings
    is_active BOOLEAN DEFAULT true,
    run_as_user_id UUID REFERENCES users(id),
    timeout_seconds INTEGER DEFAULT 300,
    max_retries INTEGER DEFAULT 3,
    retry_delay_seconds INTEGER DEFAULT 60,
    
    -- Scheduling (for scheduled workflows)
    schedule_cron VARCHAR(100), -- Cron expression
    schedule_timezone VARCHAR(50) DEFAULT 'UTC',
    next_run_at TIMESTAMPTZ,
    last_run_at TIMESTAMPTZ,
    
    -- Statistics
    execution_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0,
    avg_execution_time_ms INTEGER,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Workflow actions/steps
CREATE TABLE workflow_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    action_type VARCHAR(100) NOT NULL, -- email, sms, webhook, script, create_ticket, update_asset, etc.
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Execution order
    step_number INTEGER NOT NULL,
    depends_on_step INTEGER, -- Previous step that must succeed
    run_in_parallel BOOLEAN DEFAULT false,
    
    -- Action configuration
    action_config JSONB NOT NULL, -- Action-specific parameters
    
    -- Conditional execution
    conditions JSONB DEFAULT '[]',
    on_error_action VARCHAR(50) DEFAULT 'stop', -- stop, continue, retry, skip
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(workflow_id, step_number)
);

-- Workflow executions history
CREATE TABLE workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id),
    
    -- Trigger info
    triggered_by VARCHAR(100), -- schedule, user, webhook, event
    trigger_user_id UUID REFERENCES users(id),
    trigger_data JSONB DEFAULT '{}',
    
    -- Execution details
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, running, completed, failed, cancelled
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    
    -- Results
    steps_total INTEGER,
    steps_completed INTEGER,
    steps_failed INTEGER,
    steps_skipped INTEGER,
    
    -- Output
    output_data JSONB DEFAULT '{}',
    error_message TEXT,
    logs TEXT,
    
    -- Context
    client_id UUID REFERENCES clients(id),
    ticket_id UUID REFERENCES tickets(id),
    asset_id UUID REFERENCES assets(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Individual action executions
CREATE TABLE workflow_action_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    action_id UUID NOT NULL REFERENCES workflow_actions(id),
    
    -- Execution details
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, running, completed, failed, skipped
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    retry_count INTEGER DEFAULT 0,
    
    -- Input/Output
    input_data JSONB DEFAULT '{}',
    output_data JSONB DEFAULT '{}',
    error_message TEXT,
    logs TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Webhook endpoints
CREATE TABLE webhook_endpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    endpoint_url VARCHAR(500) NOT NULL UNIQUE,
    secret_key VARCHAR(255), -- For webhook signature verification
    
    -- Configuration
    http_method VARCHAR(10) DEFAULT 'POST',
    content_type VARCHAR(100) DEFAULT 'application/json',
    
    -- Security
    require_auth BOOLEAN DEFAULT true,
    allowed_ips INET[], -- IP whitelist
    rate_limit_requests INTEGER DEFAULT 100, -- Requests per minute
    
    -- Processing
    workflow_id UUID REFERENCES workflows(id),
    auto_create_tickets BOOLEAN DEFAULT false,
    client_mapping JSONB DEFAULT '{}', -- How to map webhook data to clients
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    last_triggered TIMESTAMPTZ,
    trigger_count INTEGER DEFAULT 0,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Webhook execution logs
CREATE TABLE webhook_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_id UUID NOT NULL REFERENCES webhook_endpoints(id),
    
    -- Request details
    source_ip INET,
    user_agent TEXT,
    http_method VARCHAR(10),
    headers JSONB,
    body TEXT,
    signature VARCHAR(255),
    
    -- Processing
    status VARCHAR(50) NOT NULL, -- received, processing, completed, failed
    response_code INTEGER,
    response_body TEXT,
    processing_time_ms INTEGER,
    
    -- Results
    workflow_execution_id UUID REFERENCES workflow_executions(id),
    error_message TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Custom scripts repository
CREATE TABLE custom_scripts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    script_type VARCHAR(50) NOT NULL, -- powershell, bash, python, sql
    
    -- Script content
    script_content TEXT NOT NULL,
    script_hash VARCHAR(64), -- SHA256 for change detection
    version INTEGER DEFAULT 1,
    
    -- Configuration
    parameters JSONB DEFAULT '[]', -- Parameter definitions
    timeout_seconds INTEGER DEFAULT 300,
    run_as_admin BOOLEAN DEFAULT false,
    
    -- Usage tracking
    execution_count INTEGER DEFAULT 0,
    last_executed TIMESTAMPTZ,
    
    -- Security
    requires_approval BOOLEAN DEFAULT true,
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Script executions
CREATE TABLE script_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    script_id UUID NOT NULL REFERENCES custom_scripts(id),
    workflow_execution_id UUID REFERENCES workflow_executions(id),
    
    -- Execution context
    executed_by UUID REFERENCES users(id),
    client_id UUID REFERENCES clients(id),
    asset_id UUID REFERENCES assets(id),
    
    -- Parameters
    parameters JSONB DEFAULT '{}',
    
    -- Results
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, running, completed, failed
    exit_code INTEGER,
    stdout TEXT,
    stderr TEXT,
    
    -- Timing
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Alert rules for monitoring and notifications
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Rule definition
    rule_type VARCHAR(100) NOT NULL, -- threshold, anomaly, pattern, heartbeat
    metric_type VARCHAR(100) NOT NULL, -- asset_offline, disk_space, response_time, license_expiry, etc.
    
    -- Conditions
    conditions JSONB NOT NULL, -- Rule-specific conditions
    severity VARCHAR(50) DEFAULT 'medium', -- critical, high, medium, low, info
    
    -- Scope
    applies_to_all_clients BOOLEAN DEFAULT true,
    client_ids UUID[] DEFAULT '{}',
    asset_ids UUID[] DEFAULT '{}',
    
    -- Evaluation
    check_frequency_minutes INTEGER DEFAULT 5,
    evaluation_window_minutes INTEGER DEFAULT 60,
    require_consecutive_failures INTEGER DEFAULT 1,
    
    -- Actions
    create_ticket BOOLEAN DEFAULT false,
    ticket_priority VARCHAR(50),
    ticket_category_id UUID REFERENCES ticket_categories(id),
    
    send_email BOOLEAN DEFAULT true,
    email_recipients TEXT[],
    
    send_sms BOOLEAN DEFAULT false,
    sms_recipients TEXT[],
    
    webhook_url VARCHAR(500),
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    is_paused BOOLEAN DEFAULT false,
    pause_until TIMESTAMPTZ,
    
    -- Statistics
    trigger_count INTEGER DEFAULT 0,
    last_triggered TIMESTAMPTZ,
    last_evaluated TIMESTAMPTZ,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Alert instances
CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES alert_rules(id),
    
    -- Alert details
    title VARCHAR(500) NOT NULL,
    description TEXT,
    severity VARCHAR(50) NOT NULL,
    
    -- Context
    client_id UUID REFERENCES clients(id),
    asset_id UUID REFERENCES assets(id),
    ticket_id UUID REFERENCES tickets(id),
    
    -- Metrics
    current_value DECIMAL(15,4),
    threshold_value DECIMAL(15,4),
    unit VARCHAR(50),
    
    -- Status
    status VARCHAR(50) DEFAULT 'active', -- active, acknowledged, resolved, suppressed
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    
    -- Escalation
    escalation_level INTEGER DEFAULT 0,
    escalated_at TIMESTAMPTZ,
    escalated_to UUID REFERENCES users(id),
    
    -- Notifications sent
    notifications_sent JSONB DEFAULT '{}', -- Track which notifications were sent
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Scheduled maintenance windows
CREATE TABLE maintenance_windows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Timing
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    timezone VARCHAR(50) DEFAULT 'UTC',
    
    -- Recurrence
    is_recurring BOOLEAN DEFAULT false,
    recurrence_rule VARCHAR(255), -- RFC 5545 RRULE
    
    -- Scope
    affects_all_clients BOOLEAN DEFAULT false,
    client_ids UUID[] DEFAULT '{}',
    asset_ids UUID[] DEFAULT '{}',
    service_names TEXT[] DEFAULT '{}',
    
    -- Actions during maintenance
    suppress_alerts BOOLEAN DEFAULT true,
    suppress_monitoring BOOLEAN DEFAULT false,
    auto_create_ticket BOOLEAN DEFAULT true,
    
    -- Communication
    notify_clients BOOLEAN DEFAULT true,
    notification_template_id UUID REFERENCES email_templates(id),
    notify_hours_before INTEGER DEFAULT 24,
    
    -- Status
    status VARCHAR(50) DEFAULT 'scheduled', -- scheduled, active, completed, cancelled
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Task scheduler for one-time and recurring tasks
CREATE TABLE scheduled_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    task_type VARCHAR(100) NOT NULL, -- script, workflow, backup, report, cleanup
    
    -- Scheduling
    schedule_type VARCHAR(50) NOT NULL, -- once, recurring, cron
    schedule_expression VARCHAR(255), -- Cron expression or interval
    timezone VARCHAR(50) DEFAULT 'UTC',
    
    -- Execution
    next_run_at TIMESTAMPTZ,
    last_run_at TIMESTAMPTZ,
    run_count INTEGER DEFAULT 0,
    max_runs INTEGER, -- NULL for unlimited
    
    -- Task configuration
    task_config JSONB DEFAULT '{}',
    timeout_seconds INTEGER DEFAULT 1800, -- 30 minutes default
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    last_status VARCHAR(50), -- success, failed, timeout, skipped
    last_error TEXT,
    last_duration_ms INTEGER,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_workflows_type ON workflows(workflow_type);
CREATE INDEX idx_workflows_next_run ON workflows(next_run_at) WHERE is_active = true;
CREATE INDEX idx_workflow_actions_workflow_id ON workflow_actions(workflow_id);
CREATE INDEX idx_workflow_executions_workflow_id ON workflow_executions(workflow_id);
CREATE INDEX idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX idx_workflow_executions_started ON workflow_executions(started_at);
CREATE INDEX idx_webhook_endpoints_url ON webhook_endpoints(endpoint_url);
CREATE INDEX idx_webhook_logs_endpoint_id ON webhook_logs(endpoint_id);
CREATE INDEX idx_custom_scripts_type ON custom_scripts(script_type);
CREATE INDEX idx_script_executions_script_id ON script_executions(script_id);
CREATE INDEX idx_alert_rules_is_active ON alert_rules(is_active);
CREATE INDEX idx_alerts_rule_id ON alerts(rule_id);
CREATE INDEX idx_alerts_status ON alerts(status);
CREATE INDEX idx_alerts_client_id ON alerts(client_id);
CREATE INDEX idx_maintenance_windows_time ON maintenance_windows(start_time, end_time);
CREATE INDEX idx_scheduled_tasks_next_run ON scheduled_tasks(next_run_at) WHERE is_active = true;

-- Function to evaluate alert rules
CREATE OR REPLACE FUNCTION evaluate_alert_rules()
RETURNS void AS $$
DECLARE
    rule_record RECORD;
    alert_triggered BOOLEAN;
    current_metric_value DECIMAL;
BEGIN
    FOR rule_record IN 
        SELECT * FROM alert_rules 
        WHERE is_active = true AND is_paused = false
        AND (last_evaluated IS NULL OR last_evaluated < NOW() - (check_frequency_minutes || ' minutes')::INTERVAL)
    LOOP
        alert_triggered := false;
        
        -- Example: Check disk space threshold
        IF rule_record.metric_type = 'disk_space_low' THEN
            -- This would be implemented based on actual monitoring data
            -- For now, just a placeholder
            alert_triggered := false;
        END IF;
        
        -- Create alert if triggered
        IF alert_triggered THEN
            INSERT INTO alerts (
                rule_id, title, description, severity, 
                current_value, threshold_value,
                client_id, asset_id
            ) VALUES (
                rule_record.id,
                'Alert: ' || rule_record.name,
                rule_record.description,
                rule_record.severity,
                current_metric_value,
                (rule_record.conditions->>'threshold')::DECIMAL,
                NULL, -- Would be determined by rule scope
                NULL
            );
            
            -- Update rule statistics
            UPDATE alert_rules 
            SET trigger_count = trigger_count + 1, 
                last_triggered = NOW(),
                last_evaluated = NOW()
            WHERE id = rule_record.id;
        ELSE
            -- Just update evaluation time
            UPDATE alert_rules 
            SET last_evaluated = NOW()
            WHERE id = rule_record.id;
        END IF;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Function to execute scheduled workflows
CREATE OR REPLACE FUNCTION execute_scheduled_workflows()
RETURNS void AS $$
DECLARE
    workflow_record RECORD;
    execution_id UUID;
BEGIN
    FOR workflow_record IN 
        SELECT * FROM workflows 
        WHERE is_active = true 
        AND workflow_type = 'scheduled'
        AND next_run_at <= NOW()
    LOOP
        -- Create execution record
        execution_id := gen_random_uuid();
        
        INSERT INTO workflow_executions (
            id, workflow_id, triggered_by, status
        ) VALUES (
            execution_id, workflow_record.id, 'schedule', 'running'
        );
        
        -- Update workflow next run time
        UPDATE workflows 
        SET last_run_at = NOW(),
            next_run_at = NOW() + INTERVAL '1 hour', -- Simplified, should parse cron
            execution_count = execution_count + 1
        WHERE id = workflow_record.id;
        
        -- TODO: Implement actual workflow execution logic
        -- For now, just mark as completed
        UPDATE workflow_executions 
        SET status = 'completed', 
            completed_at = NOW(),
            steps_total = 1,
            steps_completed = 1
        WHERE id = execution_id;
        
        UPDATE workflows 
        SET success_count = success_count + 1
        WHERE id = workflow_record.id;
        
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Insert common workflow templates
INSERT INTO workflows (name, description, workflow_type, trigger_type, trigger_config, conditions) VALUES
('Daily Client Backup Check', 'Check if all clients have recent backups', 'scheduled', 'schedule', 
 '{"cron": "0 9 * * *", "timezone": "UTC"}'::jsonb,
 '[{"type": "time_window", "hours": 24}]'::jsonb),
 
('New Ticket Auto-Assignment', 'Automatically assign new tickets based on client and category', 'event_driven', 'ticket_created',
 '{"immediate": true}'::jsonb,
 '[{"type": "client_has_assigned_tech"}, {"type": "normal_business_hours"}]'::jsonb),
 
('Invoice Overdue Reminder', 'Send reminders for overdue invoices', 'scheduled', 'schedule',
 '{"cron": "0 10 * * *", "timezone": "UTC"}'::jsonb,
 '[{"type": "invoice_overdue", "days": 7}]'::jsonb);

-- Insert common alert rules
INSERT INTO alert_rules (name, description, rule_type, metric_type, conditions, severity) VALUES
('Disk Space Critical', 'Alert when disk space is below 10%', 'threshold', 'disk_space_low',
 '{"threshold": 10, "unit": "percent", "operator": "less_than"}'::jsonb, 'critical'),
 
('Asset Offline', 'Alert when asset has not reported in 30 minutes', 'heartbeat', 'asset_heartbeat',
 '{"timeout_minutes": 30}'::jsonb, 'high'),
 
('License Expiring Soon', 'Alert when software license expires in 30 days', 'threshold', 'license_expiry',
 '{"threshold": 30, "unit": "days", "operator": "less_than"}'::jsonb, 'medium');

-- Insert common custom scripts
INSERT INTO custom_scripts (name, description, script_type, script_content, parameters) VALUES
('Windows Disk Cleanup', 'Perform disk cleanup on Windows systems', 'powershell',
'# Windows Disk Cleanup Script
param(
    [string]$ComputerName = $env:COMPUTERNAME,
    [int]$MinFreeMB = 1024
)

Write-Host "Starting disk cleanup on $ComputerName..."

# Run Disk Cleanup
cleanmgr /sagerun:1

# Get disk space info
$disk = Get-WmiObject -Class Win32_LogicalDisk -Filter "DriveType=3" | Select-Object Size,FreeSpace
$freeSpaceMB = [math]::Round($disk.FreeSpace / 1MB, 2)

Write-Host "Free space after cleanup: $freeSpaceMB MB"

if ($freeSpaceMB -lt $MinFreeMB) {
    Write-Warning "Disk space still low after cleanup!"
    exit 1
} else {
    Write-Host "Disk cleanup completed successfully"
    exit 0
}',
'[{"name": "ComputerName", "type": "string", "required": false, "description": "Target computer name"}, {"name": "MinFreeMB", "type": "integer", "default": 1024, "description": "Minimum free space required in MB"}]'::jsonb),

('Linux System Update', 'Update packages on Linux systems', 'bash',
'#!/bin/bash
# Linux System Update Script

set -e

echo "Starting system update..."

# Update package list
if command -v apt-get &> /dev/null; then
    echo "Using apt package manager..."
    apt-get update
    apt-get upgrade -y
    apt-get autoremove -y
    apt-get autoclean
elif command -v yum &> /dev/null; then
    echo "Using yum package manager..."
    yum update -y
    yum autoremove -y
elif command -v dnf &> /dev/null; then
    echo "Using dnf package manager..."
    dnf upgrade -y
    dnf autoremove -y
else
    echo "No supported package manager found"
    exit 1
fi

echo "System update completed successfully"
echo "Reboot recommended if kernel was updated"',
'[]'::jsonb);