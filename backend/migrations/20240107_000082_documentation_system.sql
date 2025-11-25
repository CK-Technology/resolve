-- Enhanced Documentation System for Resolve
-- Rich documentation, runbooks, templates, and client portal access

-- Documentation categories
CREATE TABLE doc_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES doc_categories(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    description TEXT,
    icon VARCHAR(50),
    sort_order INTEGER DEFAULT 0,
    is_public BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Documentation templates
CREATE TABLE doc_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    category VARCHAR(100) NOT NULL, -- sop, network, disaster-recovery, onboarding, etc
    description TEXT,
    content TEXT NOT NULL, -- Markdown/HTML template with variables
    variables JSONB DEFAULT '{}', -- Template variables definition
    icon VARCHAR(50),
    is_active BOOLEAN DEFAULT true,
    usage_count INTEGER DEFAULT 0,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enhanced documentation articles
CREATE TABLE documentation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    category_id UUID REFERENCES doc_categories(id),
    template_id UUID REFERENCES doc_templates(id),
    parent_id UUID REFERENCES documentation(id), -- For nested docs
    title VARCHAR(500) NOT NULL,
    slug VARCHAR(500) NOT NULL,
    content TEXT NOT NULL, -- Markdown/HTML content
    content_type VARCHAR(50) DEFAULT 'markdown', -- markdown, html, plaintext
    summary TEXT,
    tags TEXT[] DEFAULT '{}',
    version INTEGER DEFAULT 1,
    status VARCHAR(50) DEFAULT 'draft', -- draft, published, archived
    visibility VARCHAR(50) DEFAULT 'internal', -- internal, client, public
    
    -- Rich media support
    featured_image VARCHAR(500),
    attachments JSONB DEFAULT '[]', -- Array of file attachments
    embedded_media JSONB DEFAULT '[]', -- Videos, diagrams, etc
    
    -- Metadata
    author_id UUID REFERENCES users(id),
    last_editor_id UUID REFERENCES users(id),
    published_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    review_date DATE,
    
    -- Engagement tracking
    view_count INTEGER DEFAULT 0,
    helpful_count INTEGER DEFAULT 0,
    not_helpful_count INTEGER DEFAULT 0,
    
    -- Search optimization
    search_vector tsvector,
    meta_description TEXT,
    meta_keywords TEXT[],
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(client_id, slug)
);

-- Documentation version history
CREATE TABLE doc_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documentation(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    change_summary TEXT,
    author_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(document_id, version_number)
);

-- Runbooks/Procedures with step tracking
CREATE TABLE runbooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id),
    template_id UUID REFERENCES doc_templates(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL, -- maintenance, incident, deployment, backup
    severity VARCHAR(50) DEFAULT 'medium', -- critical, high, medium, low
    estimated_duration_minutes INTEGER,
    requires_approval BOOLEAN DEFAULT false,
    approval_roles TEXT[] DEFAULT '{}',
    
    -- Scheduling
    schedule_type VARCHAR(50), -- one-time, recurring, on-demand
    schedule_cron VARCHAR(100), -- Cron expression for recurring
    next_run_date TIMESTAMPTZ,
    last_run_date TIMESTAMPTZ,
    
    -- Notifications
    notify_on_start BOOLEAN DEFAULT true,
    notify_on_complete BOOLEAN DEFAULT true,
    notify_on_failure BOOLEAN DEFAULT true,
    notification_emails TEXT[] DEFAULT '{}',
    
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Runbook steps
CREATE TABLE runbook_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    runbook_id UUID NOT NULL REFERENCES runbooks(id) ON DELETE CASCADE,
    step_number INTEGER NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    instructions TEXT NOT NULL,
    script_type VARCHAR(50), -- powershell, bash, python, manual
    script_content TEXT,
    expected_result TEXT,
    on_failure_action VARCHAR(50) DEFAULT 'stop', -- stop, continue, skip
    requires_confirmation BOOLEAN DEFAULT false,
    estimated_duration_minutes INTEGER,
    attachments JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(runbook_id, step_number)
);

-- Runbook execution history
CREATE TABLE runbook_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    runbook_id UUID NOT NULL REFERENCES runbooks(id),
    executed_by UUID REFERENCES users(id),
    ticket_id UUID REFERENCES tickets(id),
    status VARCHAR(50) NOT NULL, -- pending, running, completed, failed, cancelled
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_minutes INTEGER,
    notes TEXT,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Runbook step execution tracking
CREATE TABLE runbook_step_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES runbook_executions(id) ON DELETE CASCADE,
    step_id UUID NOT NULL REFERENCES runbook_steps(id),
    status VARCHAR(50) NOT NULL, -- pending, running, completed, failed, skipped
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    output TEXT,
    error_message TEXT,
    confirmed_by UUID REFERENCES users(id),
    confirmed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Client documentation portal access
CREATE TABLE client_portal_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    access_level VARCHAR(50) DEFAULT 'read', -- read, comment, edit
    allowed_categories UUID[] DEFAULT '{}', -- Specific category access
    last_access_at TIMESTAMPTZ,
    access_token VARCHAR(255) UNIQUE,
    token_expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(client_id, contact_id)
);

-- Documentation feedback from clients
CREATE TABLE doc_feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documentation(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES contacts(id),
    user_id UUID REFERENCES users(id),
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    helpful BOOLEAN,
    comment TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Documentation related items linking
CREATE TABLE doc_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documentation(id) ON DELETE CASCADE,
    related_type VARCHAR(50) NOT NULL, -- asset, ticket, project, password, network
    related_id UUID NOT NULL,
    relationship_type VARCHAR(50) DEFAULT 'reference', -- reference, dependency, parent, child
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_documentation_client_id ON documentation(client_id);
CREATE INDEX idx_documentation_category_id ON documentation(category_id);
CREATE INDEX idx_documentation_status ON documentation(status);
CREATE INDEX idx_documentation_visibility ON documentation(visibility);
CREATE INDEX idx_documentation_search_vector ON documentation USING gin(search_vector);
CREATE INDEX idx_documentation_tags ON documentation USING gin(tags);
CREATE INDEX idx_runbooks_client_id ON runbooks(client_id);
CREATE INDEX idx_runbooks_schedule_type ON runbooks(schedule_type);
CREATE INDEX idx_runbook_executions_runbook_id ON runbook_executions(runbook_id);
CREATE INDEX idx_doc_relationships_document_id ON doc_relationships(document_id);
CREATE INDEX idx_doc_relationships_related ON doc_relationships(related_type, related_id);

-- Trigger to update search vector
CREATE OR REPLACE FUNCTION update_documentation_search_vector()
RETURNS trigger AS $$
BEGIN
    NEW.search_vector := 
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.summary, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(NEW.content, '')), 'C') ||
        setweight(to_tsvector('english', COALESCE(array_to_string(NEW.tags, ' '), '')), 'D');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER documentation_search_vector_update
    BEFORE INSERT OR UPDATE ON documentation
    FOR EACH ROW
    EXECUTE FUNCTION update_documentation_search_vector();

-- Insert default documentation templates
INSERT INTO doc_templates (name, slug, category, description, content, variables) VALUES
('Network Documentation', 'network-doc', 'network', 'Standard network documentation template', 
'# Network Documentation - {{client_name}}

## Network Overview
{{network_overview}}

## IP Addressing Scheme
| Network | Subnet | VLAN | Description |
|---------|--------|------|-------------|
{{ip_table}}

## Network Devices
{{devices_list}}

## Firewall Rules
{{firewall_rules}}

## WiFi Networks
{{wifi_config}}

## VPN Configuration
{{vpn_config}}

## DNS Configuration
{{dns_config}}

## Network Diagram
{{network_diagram}}

## Change Log
{{change_log}}', 
'{"client_name": "string", "network_overview": "text", "ip_table": "table", "devices_list": "list", "firewall_rules": "text", "wifi_config": "text", "vpn_config": "text", "dns_config": "text", "network_diagram": "image", "change_log": "text"}'::jsonb),

('Disaster Recovery Plan', 'disaster-recovery', 'disaster-recovery', 'DR plan template', 
'# Disaster Recovery Plan - {{client_name}}

## Executive Summary
{{executive_summary}}

## Critical Systems
{{critical_systems}}

## Recovery Time Objectives (RTO)
{{rto_table}}

## Recovery Point Objectives (RPO)
{{rpo_table}}

## Backup Procedures
{{backup_procedures}}

## Recovery Procedures
{{recovery_procedures}}

## Emergency Contacts
{{emergency_contacts}}

## Testing Schedule
{{testing_schedule}}',
'{"client_name": "string", "executive_summary": "text", "critical_systems": "list", "rto_table": "table", "rpo_table": "table", "backup_procedures": "text", "recovery_procedures": "text", "emergency_contacts": "table", "testing_schedule": "text"}'::jsonb),

('Standard Operating Procedure', 'sop', 'sop', 'General SOP template',
'# {{procedure_name}}

## Purpose
{{purpose}}

## Scope
{{scope}}

## Responsibilities
{{responsibilities}}

## Procedure Steps
{{procedure_steps}}

## Related Documents
{{related_docs}}

## Revision History
{{revision_history}}',
'{"procedure_name": "string", "purpose": "text", "scope": "text", "responsibilities": "text", "procedure_steps": "list", "related_docs": "list", "revision_history": "table"}'::jsonb);

-- Insert default documentation categories
INSERT INTO doc_categories (name, slug, description, sort_order) VALUES
('Network', 'network', 'Network documentation and diagrams', 1),
('Security', 'security', 'Security policies and procedures', 2),
('Backup & Recovery', 'backup-recovery', 'Backup and disaster recovery documentation', 3),
('Procedures', 'procedures', 'Standard operating procedures', 4),
('Policies', 'policies', 'Company and IT policies', 5),
('Passwords', 'passwords', 'Password and credential documentation', 6),
('Licenses', 'licenses', 'Software licensing documentation', 7),
('Vendors', 'vendors', 'Vendor contact and support information', 8),
('How-To Guides', 'how-to', 'Step-by-step guides and tutorials', 9),
('FAQs', 'faqs', 'Frequently asked questions', 10);