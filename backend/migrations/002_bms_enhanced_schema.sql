-- Enhanced BMS schema with advanced features
-- SLA Management, Time Tracking, Project Management, Automation

-- User roles and permissions
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    permissions JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enhanced users table with more BMS features
ALTER TABLE users ADD COLUMN role_id UUID REFERENCES roles(id);
ALTER TABLE users ADD COLUMN hourly_rate DECIMAL(10,2);
ALTER TABLE users ADD COLUMN timezone VARCHAR(50) DEFAULT 'UTC';
ALTER TABLE users ADD COLUMN avatar_url VARCHAR(500);
ALTER TABLE users ADD COLUMN phone VARCHAR(50);
ALTER TABLE users ADD COLUMN department VARCHAR(100);

-- Client contracts and SLAs
CREATE TABLE contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    contract_type VARCHAR(50) DEFAULT 'monthly', -- monthly, yearly, project, hourly
    start_date DATE NOT NULL,
    end_date DATE,
    monthly_value DECIMAL(15,2),
    hourly_rate DECIMAL(10,2),
    included_hours INTEGER,
    overage_rate DECIMAL(10,2),
    status VARCHAR(50) DEFAULT 'active',
    terms TEXT,
    auto_renew BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- SLA definitions
CREATE TABLE slas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id UUID REFERENCES contracts(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    priority VARCHAR(50) NOT NULL, -- critical, high, medium, low
    response_time_minutes INTEGER NOT NULL,
    resolution_time_hours INTEGER NOT NULL,
    business_hours_only BOOLEAN DEFAULT true,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Business hours configuration
CREATE TABLE business_hours (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    day_of_week INTEGER NOT NULL, -- 0=Sunday, 6=Saturday
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    timezone VARCHAR(50) DEFAULT 'UTC',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enhanced tickets with SLA tracking
ALTER TABLE tickets ADD COLUMN sla_id UUID REFERENCES slas(id);
ALTER TABLE tickets ADD COLUMN response_due_at TIMESTAMPTZ;
ALTER TABLE tickets ADD COLUMN resolution_due_at TIMESTAMPTZ;
ALTER TABLE tickets ADD COLUMN first_response_at TIMESTAMPTZ;
ALTER TABLE tickets ADD COLUMN resolved_at TIMESTAMPTZ;
ALTER TABLE tickets ADD COLUMN sla_breached BOOLEAN DEFAULT false;
ALTER TABLE tickets ADD COLUMN escalated_at TIMESTAMPTZ;
ALTER TABLE tickets ADD COLUMN escalated_to UUID REFERENCES users(id);
ALTER TABLE tickets ADD COLUMN source VARCHAR(50) DEFAULT 'manual'; -- email, portal, api, phone
ALTER TABLE tickets ADD COLUMN category_id UUID;
ALTER TABLE tickets ADD COLUMN estimated_hours DECIMAL(5,2);
ALTER TABLE tickets ADD COLUMN actual_hours DECIMAL(5,2);

-- Ticket categories for organization
CREATE TABLE ticket_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    color VARCHAR(7), -- hex color
    default_priority VARCHAR(50) DEFAULT 'medium',
    default_sla_id UUID REFERENCES slas(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Add foreign key for ticket categories
ALTER TABLE tickets ADD CONSTRAINT fk_tickets_category FOREIGN KEY (category_id) REFERENCES ticket_categories(id);

-- Time tracking entries
CREATE TABLE time_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID REFERENCES tickets(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    project_id UUID,
    task_id UUID,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    duration_minutes INTEGER,
    description TEXT,
    billable BOOLEAN DEFAULT true,
    billed BOOLEAN DEFAULT false,
    hourly_rate DECIMAL(10,2),
    total_amount DECIMAL(15,2),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Projects for organizing work
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'active', -- active, completed, on_hold, cancelled
    start_date DATE,
    end_date DATE,
    budget DECIMAL(15,2),
    hourly_rate DECIMAL(10,2),
    project_manager_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Project tasks
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    ticket_id UUID REFERENCES tickets(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    assigned_to UUID REFERENCES users(id),
    status VARCHAR(50) DEFAULT 'todo', -- todo, in_progress, completed, blocked
    priority VARCHAR(50) DEFAULT 'medium',
    estimated_hours DECIMAL(5,2),
    actual_hours DECIMAL(5,2),
    due_date DATE,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Add foreign key for time entries
ALTER TABLE time_entries ADD CONSTRAINT fk_time_entries_project FOREIGN KEY (project_id) REFERENCES projects(id);
ALTER TABLE time_entries ADD CONSTRAINT fk_time_entries_task FOREIGN KEY (task_id) REFERENCES tasks(id);

-- Enhanced invoicing with more detail
ALTER TABLE invoices ADD COLUMN contract_id UUID REFERENCES contracts(id);
ALTER TABLE invoices ADD COLUMN project_id UUID REFERENCES projects(id);
ALTER TABLE invoices ADD COLUMN payment_terms VARCHAR(50) DEFAULT 'net_30';
ALTER TABLE invoices ADD COLUMN late_fee_percentage DECIMAL(5,2);
ALTER TABLE invoices ADD COLUMN discount_percentage DECIMAL(5,2);
ALTER TABLE invoices ADD COLUMN discount_amount DECIMAL(15,2);

-- Invoice payments tracking
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    amount DECIMAL(15,2) NOT NULL,
    payment_date DATE NOT NULL,
    payment_method VARCHAR(50), -- check, credit_card, bank_transfer, cash
    reference_number VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Recurring invoice templates
CREATE TABLE recurring_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    contract_id UUID REFERENCES contracts(id),
    template_name VARCHAR(255) NOT NULL,
    frequency VARCHAR(50) NOT NULL, -- monthly, quarterly, yearly
    next_invoice_date DATE NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    auto_send BOOLEAN DEFAULT false,
    active BOOLEAN DEFAULT true,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Asset monitoring and alerts
CREATE TABLE asset_monitors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    monitor_type VARCHAR(50) NOT NULL, -- ping, http, disk_space, memory, cpu
    check_interval_minutes INTEGER DEFAULT 5,
    enabled BOOLEAN DEFAULT true,
    configuration JSONB DEFAULT '{}'::jsonb,
    last_check_at TIMESTAMPTZ,
    status VARCHAR(50) DEFAULT 'unknown', -- up, down, warning, unknown
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Alerts and notifications
CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID REFERENCES assets(id),
    ticket_id UUID REFERENCES tickets(id),
    monitor_id UUID REFERENCES asset_monitors(id),
    alert_type VARCHAR(50) NOT NULL, -- sla_breach, asset_down, invoice_overdue
    severity VARCHAR(50) DEFAULT 'medium', -- critical, high, medium, low, info
    title VARCHAR(255) NOT NULL,
    message TEXT,
    acknowledged BOOLEAN DEFAULT false,
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMPTZ,
    resolved BOOLEAN DEFAULT false,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Knowledge base articles
CREATE TABLE kb_articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    summary TEXT,
    category_id UUID,
    author_id UUID NOT NULL REFERENCES users(id),
    status VARCHAR(50) DEFAULT 'draft', -- draft, published, archived
    public BOOLEAN DEFAULT false,
    views INTEGER DEFAULT 0,
    helpful_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- KB categories
CREATE TABLE kb_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES kb_categories(id),
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

ALTER TABLE kb_articles ADD CONSTRAINT fk_kb_articles_category FOREIGN KEY (category_id) REFERENCES kb_categories(id);

-- Automation rules
CREATE TABLE automation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    trigger_type VARCHAR(50) NOT NULL, -- ticket_created, ticket_updated, sla_breach, asset_down
    conditions JSONB DEFAULT '{}'::jsonb,
    actions JSONB DEFAULT '{}'::jsonb,
    enabled BOOLEAN DEFAULT true,
    last_run_at TIMESTAMPTZ,
    run_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit log for tracking changes
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    entity_type VARCHAR(100) NOT NULL, -- ticket, client, invoice, etc.
    entity_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL, -- create, update, delete
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_contracts_client_id ON contracts(client_id);
CREATE INDEX idx_contracts_status ON contracts(status);
CREATE INDEX idx_slas_contract_id ON slas(contract_id);
CREATE INDEX idx_time_entries_ticket_id ON time_entries(ticket_id);
CREATE INDEX idx_time_entries_user_id ON time_entries(user_id);
CREATE INDEX idx_time_entries_billable ON time_entries(billable);
CREATE INDEX idx_time_entries_start_time ON time_entries(start_time);
CREATE INDEX idx_projects_client_id ON projects(client_id);
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_tasks_project_id ON tasks(project_id);
CREATE INDEX idx_tasks_assigned_to ON tasks(assigned_to);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_payments_invoice_id ON payments(invoice_id);
CREATE INDEX idx_alerts_asset_id ON alerts(asset_id);
CREATE INDEX idx_alerts_acknowledged ON alerts(acknowledged);
CREATE INDEX idx_alerts_resolved ON alerts(resolved);
CREATE INDEX idx_kb_articles_category_id ON kb_articles(category_id);
CREATE INDEX idx_kb_articles_status ON kb_articles(status);
CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- Insert default roles
INSERT INTO roles (name, description, permissions) VALUES 
('Admin', 'Full system access', '["*"]'::jsonb),
('Manager', 'Management access to all client data', '["clients.*", "tickets.*", "projects.*", "reports.*"]'::jsonb),
('Technician', 'Basic ticket and client access', '["tickets.view", "tickets.create", "tickets.update", "clients.view", "time_entries.*"]'::jsonb),
('Client', 'Client portal access', '["tickets.view_own", "invoices.view_own"]'::jsonb);

-- Insert default ticket categories
INSERT INTO ticket_categories (name, color, default_priority) VALUES 
('Hardware Issue', '#ef4444', 'high'),
('Software Issue', '#f59e0b', 'medium'),
('Network Issue', '#dc2626', 'high'),
('Security', '#7c2d12', 'critical'),
('General Support', '#6b7280', 'medium'),
('Project Work', '#059669', 'medium');