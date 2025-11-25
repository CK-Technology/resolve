-- Modern Ticketing Features: SLA Management, Workflows, and Email Integration
-- Implements comprehensive SLA tracking, automated workflows, and email-to-ticket functionality

-- SLA Policies and Rules
CREATE TABLE sla_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    client_id UUID REFERENCES clients(id), -- NULL for global policies
    is_global BOOLEAN DEFAULT false,
    priority_levels JSONB NOT NULL, -- {"low": {...}, "medium": {...}, "high": {...}, "critical": {...}}
    business_hours JSONB NOT NULL, -- {"timezone": "UTC", "days": {"monday": {"start": "09:00", "end": "17:00"}}}
    holiday_calendar_id UUID, -- Reference to holiday calendar
    auto_escalation BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- SLA Rules for specific priorities
CREATE TABLE sla_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id UUID NOT NULL REFERENCES sla_policies(id) ON DELETE CASCADE,
    priority VARCHAR(20) NOT NULL,
    response_time_minutes INTEGER NOT NULL,
    resolution_time_hours INTEGER NOT NULL,
    escalation_time_minutes INTEGER, -- time before escalation
    escalation_user_id UUID REFERENCES users(id),
    escalation_group_id UUID, -- Reference to user groups
    breach_notification_emails TEXT[], -- emails to notify on breach
    auto_assign_user_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_sla_rules_policy_id ON sla_rules(policy_id);
CREATE INDEX idx_sla_rules_priority ON sla_rules(priority);

-- Ticket SLA tracking
CREATE TABLE ticket_sla_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    sla_policy_id UUID NOT NULL REFERENCES sla_policies(id),
    sla_rule_id UUID NOT NULL REFERENCES sla_rules(id),
    response_due_at TIMESTAMPTZ NOT NULL,
    resolution_due_at TIMESTAMPTZ NOT NULL,
    first_response_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    response_breached BOOLEAN DEFAULT false,
    resolution_breached BOOLEAN DEFAULT false,
    response_breach_minutes INTEGER,
    resolution_breach_minutes INTEGER,
    escalated_at TIMESTAMPTZ,
    escalated_to_user_id UUID REFERENCES users(id),
    pause_start TIMESTAMPTZ, -- for pausing SLA during customer wait
    pause_duration_minutes INTEGER DEFAULT 0,
    breach_notifications_sent INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(ticket_id)
);

CREATE INDEX idx_ticket_sla_tracking_ticket_id ON ticket_sla_tracking(ticket_id);
CREATE INDEX idx_ticket_sla_tracking_response_due ON ticket_sla_tracking(response_due_at);
CREATE INDEX idx_ticket_sla_tracking_resolution_due ON ticket_sla_tracking(resolution_due_at);
CREATE INDEX idx_ticket_sla_tracking_breached ON ticket_sla_tracking(response_breached, resolution_breached);

-- Workflow definitions
CREATE TABLE ticket_workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    trigger_type VARCHAR(50) NOT NULL, -- created, status_changed, priority_changed, time_elapsed, sla_breach
    trigger_conditions JSONB, -- conditions that must be met
    is_active BOOLEAN DEFAULT true,
    execution_order INTEGER DEFAULT 0,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Workflow actions
CREATE TABLE workflow_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES ticket_workflows(id) ON DELETE CASCADE,
    action_type VARCHAR(50) NOT NULL, 
    -- assign_user, change_status, change_priority, send_email, create_task, add_note, escalate
    action_parameters JSONB NOT NULL, -- parameters for the action
    execution_order INTEGER DEFAULT 0,
    delay_minutes INTEGER DEFAULT 0, -- delay before executing this action
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_workflow_actions_workflow_id ON workflow_actions(workflow_id);

-- Workflow execution log
CREATE TABLE workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES ticket_workflows(id),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    trigger_type VARCHAR(50) NOT NULL,
    trigger_data JSONB,
    execution_status VARCHAR(20) DEFAULT 'pending', -- pending, running, completed, failed
    actions_completed INTEGER DEFAULT 0,
    total_actions INTEGER DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    execution_duration_ms INTEGER
);

CREATE INDEX idx_workflow_executions_ticket_id ON workflow_executions(ticket_id);
CREATE INDEX idx_workflow_executions_status ON workflow_executions(execution_status);

-- Email integration for ticket creation and updates
CREATE TABLE email_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    email_address VARCHAR(255) NOT NULL UNIQUE,
    smtp_host VARCHAR(255) NOT NULL,
    smtp_port INTEGER DEFAULT 587,
    smtp_encryption VARCHAR(20) DEFAULT 'tls', -- none, tls, ssl
    smtp_username VARCHAR(255) NOT NULL,
    smtp_password_encrypted TEXT NOT NULL,
    imap_host VARCHAR(255) NOT NULL,
    imap_port INTEGER DEFAULT 993,
    imap_encryption VARCHAR(20) DEFAULT 'ssl',
    imap_username VARCHAR(255) NOT NULL,
    imap_password_encrypted TEXT NOT NULL,
    auto_create_tickets BOOLEAN DEFAULT true,
    default_client_id UUID REFERENCES clients(id),
    default_category_id UUID,
    default_priority VARCHAR(20) DEFAULT 'medium',
    signature TEXT,
    is_active BOOLEAN DEFAULT true,
    last_check TIMESTAMPTZ,
    check_interval_minutes INTEGER DEFAULT 5,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Email messages linked to tickets
CREATE TABLE ticket_email_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    email_account_id UUID REFERENCES email_accounts(id),
    message_id VARCHAR(255), -- email message ID
    thread_id VARCHAR(255), -- email thread ID
    direction VARCHAR(20) NOT NULL, -- inbound, outbound
    from_email VARCHAR(255) NOT NULL,
    to_emails TEXT[] NOT NULL,
    cc_emails TEXT[],
    bcc_emails TEXT[],
    subject VARCHAR(500) NOT NULL,
    body_text TEXT,
    body_html TEXT,
    attachments JSONB, -- array of attachment info
    is_processed BOOLEAN DEFAULT false,
    processing_error TEXT,
    received_at TIMESTAMPTZ NOT NULL,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_ticket_email_messages_ticket_id ON ticket_email_messages(ticket_id);
CREATE INDEX idx_ticket_email_messages_message_id ON ticket_email_messages(message_id);
CREATE INDEX idx_ticket_email_messages_direction ON ticket_email_messages(direction);

-- Ticket categories for better organization
CREATE TABLE ticket_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    color VARCHAR(7), -- hex color
    icon VARCHAR(50),
    parent_category_id UUID REFERENCES ticket_categories(id),
    default_priority VARCHAR(20) DEFAULT 'medium',
    default_sla_policy_id UUID REFERENCES sla_policies(id),
    auto_assign_user_id UUID REFERENCES users(id),
    billing_rate DECIMAL(10,2), -- override hourly rate for this category
    is_billable BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_ticket_categories_parent ON ticket_categories(parent_category_id);

-- Ticket templates for common issues
CREATE TABLE ticket_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    category_id UUID REFERENCES ticket_categories(id),
    priority VARCHAR(20) DEFAULT 'medium',
    estimated_hours DECIMAL(5,2),
    is_billable BOOLEAN DEFAULT true,
    auto_assign_user_id UUID REFERENCES users(id),
    template_fields JSONB, -- custom fields for this template
    is_active BOOLEAN DEFAULT true,
    usage_count INTEGER DEFAULT 0,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Ticket escalations
CREATE TABLE ticket_escalations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    escalation_type VARCHAR(50) NOT NULL, -- sla_breach, manual, auto_timeout
    escalated_from_user_id UUID REFERENCES users(id),
    escalated_to_user_id UUID REFERENCES users(id),
    escalation_reason TEXT NOT NULL,
    escalation_level INTEGER DEFAULT 1,
    is_resolved BOOLEAN DEFAULT false,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_ticket_escalations_ticket_id ON ticket_escalations(ticket_id);
CREATE INDEX idx_ticket_escalations_to_user ON ticket_escalations(escalated_to_user_id);

-- Client portal access tokens for guest viewing
CREATE TABLE client_portal_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES contacts(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    access_level VARCHAR(20) DEFAULT 'read_only', -- read_only, create_tickets, full_access
    allowed_features JSONB, -- {"tickets": true, "invoices": false, "time_entries": true}
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    ip_restrictions INET[],
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_client_portal_tokens_client_id ON client_portal_tokens(client_id);
CREATE INDEX idx_client_portal_tokens_hash ON client_portal_tokens(token_hash);
CREATE INDEX idx_client_portal_tokens_expires ON client_portal_tokens(expires_at);

-- Ticket watchers (users who get notifications)
CREATE TABLE ticket_watchers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_preferences JSONB DEFAULT '{"status_changes": true, "new_replies": true, "assignments": true}',
    added_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(ticket_id, user_id)
);

CREATE INDEX idx_ticket_watchers_ticket_id ON ticket_watchers(ticket_id);
CREATE INDEX idx_ticket_watchers_user_id ON ticket_watchers(user_id);

-- Add category and SLA fields to existing tickets table (if not already present)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tickets' AND column_name = 'category_id') THEN
        ALTER TABLE tickets ADD COLUMN category_id UUID REFERENCES ticket_categories(id);
        CREATE INDEX idx_tickets_category_id ON tickets(category_id);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tickets' AND column_name = 'sla_policy_id') THEN
        ALTER TABLE tickets ADD COLUMN sla_policy_id UUID REFERENCES sla_policies(id);
        CREATE INDEX idx_tickets_sla_policy_id ON tickets(sla_policy_id);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tickets' AND column_name = 'source') THEN
        ALTER TABLE tickets ADD COLUMN source VARCHAR(50) DEFAULT 'manual';
        CREATE INDEX idx_tickets_source ON tickets(source);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tickets' AND column_name = 'estimated_hours') THEN
        ALTER TABLE tickets ADD COLUMN estimated_hours DECIMAL(5,2);
    END IF;
END $$;

-- Insert default SLA policy
INSERT INTO sla_policies (name, description, is_global, priority_levels, business_hours, created_by)
SELECT 
    'Default SLA Policy',
    'Standard SLA policy for all tickets',
    true,
    '{"low": {"name": "Low Priority", "color": "#10b981"}, "medium": {"name": "Medium Priority", "color": "#f59e0b"}, "high": {"name": "High Priority", "color": "#ef4444"}, "critical": {"name": "Critical Priority", "color": "#dc2626"}}'::jsonb,
    '{"timezone": "UTC", "days": {"monday": {"start": "09:00", "end": "17:00"}, "tuesday": {"start": "09:00", "end": "17:00"}, "wednesday": {"start": "09:00", "end": "17:00"}, "thursday": {"start": "09:00", "end": "17:00"}, "friday": {"start": "09:00", "end": "17:00"}}}'::jsonb,
    (SELECT id FROM users LIMIT 1)
WHERE EXISTS (SELECT 1 FROM users LIMIT 1);

-- Insert default SLA rules
INSERT INTO sla_rules (policy_id, priority, response_time_minutes, resolution_time_hours, escalation_time_minutes)
SELECT 
    p.id,
    'low',
    240,  -- 4 hours response
    24,   -- 24 hours resolution
    480   -- 8 hours escalation
FROM sla_policies p WHERE p.name = 'Default SLA Policy';

INSERT INTO sla_rules (policy_id, priority, response_time_minutes, resolution_time_hours, escalation_time_minutes)
SELECT 
    p.id,
    'medium',
    120,  -- 2 hours response
    8,    -- 8 hours resolution
    240   -- 4 hours escalation
FROM sla_policies p WHERE p.name = 'Default SLA Policy';

INSERT INTO sla_rules (policy_id, priority, response_time_minutes, resolution_time_hours, escalation_time_minutes)
SELECT 
    p.id,
    'high',
    60,   -- 1 hour response
    4,    -- 4 hours resolution
    120   -- 2 hours escalation
FROM sla_policies p WHERE p.name = 'Default SLA Policy';

INSERT INTO sla_rules (policy_id, priority, response_time_minutes, resolution_time_hours, escalation_time_minutes)
SELECT 
    p.id,
    'critical',
    15,   -- 15 minutes response
    2,    -- 2 hours resolution
    30    -- 30 minutes escalation
FROM sla_policies p WHERE p.name = 'Default SLA Policy';

-- Insert default ticket categories
INSERT INTO ticket_categories (name, description, color, icon, default_priority) VALUES
('General Support', 'General IT support requests', '#6b7280', 'help-circle', 'medium'),
('Hardware Issues', 'Hardware problems and failures', '#dc2626', 'server', 'high'),
('Software Issues', 'Software bugs and problems', '#f59e0b', 'code', 'medium'),
('Network Issues', 'Network connectivity problems', '#ef4444', 'network-wired', 'high'),
('Security Incidents', 'Security breaches and incidents', '#dc2626', 'shield-alert', 'critical'),
('Access Requests', 'User access and permission requests', '#10b981', 'key', 'low'),
('New User Setup', 'New user account and equipment setup', '#3b82f6', 'user-plus', 'medium'),
('Password Reset', 'Password reset requests', '#6b7280', 'lock', 'low'),
('Email Issues', 'Email problems and setup', '#f59e0b', 'mail', 'medium'),
('Backup Issues', 'Backup and recovery problems', '#ef4444', 'database', 'high');

-- Function to calculate SLA due dates based on business hours
CREATE OR REPLACE FUNCTION calculate_sla_due_date(
    start_time TIMESTAMPTZ,
    duration_minutes INTEGER,
    business_hours JSONB,
    timezone_name TEXT DEFAULT 'UTC'
) RETURNS TIMESTAMPTZ AS $$
DECLARE
    current_time TIMESTAMPTZ := start_time AT TIME ZONE timezone_name;
    remaining_minutes INTEGER := duration_minutes;
    current_day TEXT;
    day_schedule JSONB;
    day_start TIME;
    day_end TIME;
    minutes_today INTEGER;
    result_time TIMESTAMPTZ;
BEGIN
    -- Simple implementation: add business hours only
    -- This is a simplified version - full implementation would handle holidays, weekends, etc.
    
    WHILE remaining_minutes > 0 LOOP
        current_day := CASE EXTRACT(dow FROM current_time)
            WHEN 0 THEN 'sunday'
            WHEN 1 THEN 'monday'
            WHEN 2 THEN 'tuesday'
            WHEN 3 THEN 'wednesday'
            WHEN 4 THEN 'thursday'
            WHEN 5 THEN 'friday'
            WHEN 6 THEN 'saturday'
        END;
        
        day_schedule := business_hours->'days'->current_day;
        
        IF day_schedule IS NULL THEN
            -- Non-business day, move to next day
            current_time := date_trunc('day', current_time) + INTERVAL '1 day';
            CONTINUE;
        END IF;
        
        day_start := (day_schedule->>'start')::TIME;
        day_end := (day_schedule->>'end')::TIME;
        
        -- Calculate available minutes today
        IF current_time::TIME < day_start THEN
            current_time := date_trunc('day', current_time) + day_start::INTERVAL;
        END IF;
        
        IF current_time::TIME >= day_end THEN
            -- After business hours, move to next day
            current_time := date_trunc('day', current_time) + INTERVAL '1 day';
            CONTINUE;
        END IF;
        
        minutes_today := EXTRACT(EPOCH FROM (date_trunc('day', current_time) + day_end::INTERVAL - current_time)) / 60;
        
        IF remaining_minutes <= minutes_today THEN
            -- Can finish today
            result_time := current_time + (remaining_minutes || ' minutes')::INTERVAL;
            remaining_minutes := 0;
        ELSE
            -- Need more days
            remaining_minutes := remaining_minutes - minutes_today;
            current_time := date_trunc('day', current_time) + INTERVAL '1 day';
        END IF;
    END LOOP;
    
    RETURN result_time AT TIME ZONE timezone_name;
END;
$$ LANGUAGE plpgsql;

-- Function to update SLA tracking when ticket status changes
CREATE OR REPLACE FUNCTION update_ticket_sla_tracking() RETURNS TRIGGER AS $$
DECLARE
    sla_record RECORD;
    policy_record RECORD;
    rule_record RECORD;
BEGIN
    -- Only for existing tickets being updated
    IF TG_OP = 'UPDATE' THEN
        -- Get SLA tracking record
        SELECT * INTO sla_record 
        FROM ticket_sla_tracking 
        WHERE ticket_id = NEW.id;
        
        -- If no SLA tracking exists, create it
        IF NOT FOUND THEN
            -- Get default SLA policy and rules
            SELECT * INTO policy_record 
            FROM sla_policies 
            WHERE is_global = true AND is_active = true 
            LIMIT 1;
            
            IF FOUND THEN
                SELECT * INTO rule_record
                FROM sla_rules
                WHERE policy_id = policy_record.id AND priority = COALESCE(NEW.priority, 'medium');
                
                IF FOUND THEN
                    INSERT INTO ticket_sla_tracking (
                        ticket_id, sla_policy_id, sla_rule_id,
                        response_due_at, resolution_due_at
                    ) VALUES (
                        NEW.id, policy_record.id, rule_record.id,
                        calculate_sla_due_date(NEW.created_at, rule_record.response_time_minutes, policy_record.business_hours),
                        calculate_sla_due_date(NEW.created_at, rule_record.resolution_time_hours * 60, policy_record.business_hours)
                    );
                END IF;
            END IF;
        ELSE
            -- Update existing SLA tracking
            IF OLD.status != NEW.status THEN
                IF NEW.status IN ('resolved', 'closed') AND sla_record.resolved_at IS NULL THEN
                    -- Mark as resolved
                    UPDATE ticket_sla_tracking
                    SET resolved_at = NOW(),
                        resolution_breached = (NOW() > sla_record.resolution_due_at),
                        resolution_breach_minutes = CASE 
                            WHEN NOW() > sla_record.resolution_due_at 
                            THEN EXTRACT(EPOCH FROM (NOW() - sla_record.resolution_due_at)) / 60
                            ELSE NULL
                        END,
                        updated_at = NOW()
                    WHERE ticket_id = NEW.id;
                END IF;
            END IF;
        END IF;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for SLA tracking
CREATE TRIGGER trigger_update_ticket_sla_tracking
    AFTER INSERT OR UPDATE ON tickets
    FOR EACH ROW EXECUTE FUNCTION update_ticket_sla_tracking();