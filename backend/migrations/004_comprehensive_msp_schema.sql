-- Comprehensive MSP Platform Schema Enhancement
-- OAuth/SSO, ITDoc, Enhanced Ticketing, Financial Management, and more

-- Authentication Providers for OAuth2/OIDC/SAML
CREATE TABLE auth_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    provider_type VARCHAR NOT NULL, -- oauth2, oidc, saml
    client_id VARCHAR,
    client_secret TEXT, -- Encrypted
    auth_url VARCHAR,
    token_url VARCHAR,
    userinfo_url VARCHAR,
    scopes TEXT[], -- Array of OAuth scopes
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Update users table with OAuth/MFA support
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash VARCHAR;
ALTER TABLE users ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS mfa_secret VARCHAR; -- Encrypted TOTP secret
ALTER TABLE users ADD COLUMN IF NOT EXISTS oauth_provider VARCHAR;
ALTER TABLE users ADD COLUMN IF NOT EXISTS oauth_id VARCHAR;
ALTER TABLE users ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER DEFAULT 0;
ALTER TABLE users ADD COLUMN IF NOT EXISTS locked_until TIMESTAMP WITH TIME ZONE;

-- ITDoc Module: Credentials Management
CREATE TABLE credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id),
    asset_id UUID REFERENCES assets(id),
    name VARCHAR NOT NULL,
    username VARCHAR,
    password TEXT, -- Encrypted
    private_key TEXT, -- Encrypted
    public_key TEXT,
    certificate TEXT,
    uri VARCHAR,
    notes TEXT,
    tags TEXT[] DEFAULT '{}',
    last_accessed TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- ITDoc Module: Domain Management
CREATE TABLE domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id),
    name VARCHAR NOT NULL,
    registrar VARCHAR,
    nameservers TEXT[] DEFAULT '{}',
    registration_date DATE,
    expiry_date DATE,
    auto_renew BOOLEAN DEFAULT false,
    dns_records JSONB DEFAULT '{}',
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- ITDoc Module: SSL Certificate Management
CREATE TABLE ssl_certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id UUID REFERENCES domains(id),
    client_id UUID NOT NULL REFERENCES clients(id),
    name VARCHAR NOT NULL,
    common_name VARCHAR NOT NULL,
    subject_alt_names TEXT[] DEFAULT '{}',
    issuer VARCHAR NOT NULL,
    issued_date DATE NOT NULL,
    expiry_date DATE NOT NULL,
    certificate_chain TEXT,
    private_key TEXT, -- Encrypted
    auto_renew BOOLEAN DEFAULT false,
    status VARCHAR DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- ITDoc Module: Network Documentation
CREATE TABLE networks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id),
    name VARCHAR NOT NULL,
    description TEXT,
    network_type VARCHAR NOT NULL, -- lan, wan, vpn, etc
    ip_range VARCHAR NOT NULL,
    subnet_mask VARCHAR NOT NULL,
    gateway VARCHAR,
    dns_servers TEXT[] DEFAULT '{}',
    vlan_id INTEGER,
    location_id UUID REFERENCES locations(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- Client Locations
CREATE TABLE IF NOT EXISTS locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id),
    name VARCHAR NOT NULL,
    address TEXT,
    city VARCHAR,
    state VARCHAR,
    country VARCHAR,
    zip VARCHAR,
    timezone VARCHAR DEFAULT 'UTC',
    primary_location BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- ITDoc Module: Software License Management
CREATE TABLE software_licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id),
    name VARCHAR NOT NULL,
    vendor VARCHAR NOT NULL,
    version VARCHAR,
    license_key TEXT, -- Encrypted
    license_type VARCHAR NOT NULL, -- perpetual, subscription, etc
    seats INTEGER,
    used_seats INTEGER DEFAULT 0,
    purchase_date DATE,
    expiry_date DATE,
    renewal_date DATE,
    cost DECIMAL(10,2),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- Enhanced Ticketing: Templates
CREATE TABLE ticket_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    subject VARCHAR NOT NULL,
    details TEXT NOT NULL,
    priority VARCHAR DEFAULT 'medium',
    category_id UUID REFERENCES ticket_categories(id),
    assigned_to UUID REFERENCES users(id),
    billable BOOLEAN DEFAULT true,
    estimated_hours DECIMAL(8,2),
    tags TEXT[] DEFAULT '{}',
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- Enhanced Ticketing: Recurring Tickets
CREATE TABLE recurring_tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES ticket_templates(id),
    client_id UUID NOT NULL REFERENCES clients(id),
    frequency VARCHAR NOT NULL, -- daily, weekly, monthly, quarterly, yearly
    interval_value INTEGER DEFAULT 1,
    next_run TIMESTAMP WITH TIME ZONE NOT NULL,
    last_run TIMESTAMP WITH TIME ZONE,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- File Management System
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id),
    ticket_id UUID REFERENCES tickets(id),
    asset_id UUID REFERENCES assets(id),
    project_id UUID REFERENCES projects(id),
    kb_article_id UUID REFERENCES kb_articles(id),
    filename VARCHAR NOT NULL,
    original_filename VARCHAR NOT NULL,
    mime_type VARCHAR NOT NULL,
    file_size BIGINT NOT NULL,
    file_path VARCHAR NOT NULL,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Notification System
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    title VARCHAR NOT NULL,
    message TEXT NOT NULL,
    notification_type VARCHAR DEFAULT 'info', -- info, warning, error, success
    entity_type VARCHAR, -- ticket, invoice, asset, etc
    entity_id UUID,
    read BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Audit Logging (enhance existing table if it exists)
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    action VARCHAR NOT NULL,
    entity_type VARCHAR NOT NULL,
    entity_id UUID NOT NULL,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- External Integrations
CREATE TABLE integrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    integration_type VARCHAR NOT NULL, -- github, azure, google, stripe, etc
    config JSONB DEFAULT '{}',
    credentials JSONB DEFAULT '{}', -- Encrypted
    enabled BOOLEAN DEFAULT true,
    last_sync TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- Vendor Management
CREATE TABLE vendors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    email VARCHAR,
    phone VARCHAR,
    website VARCHAR,
    address TEXT,
    city VARCHAR,
    state VARCHAR,
    zip VARCHAR,
    contact_name VARCHAR,
    account_number VARCHAR,
    payment_terms VARCHAR,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- Expense Categories
CREATE TABLE expense_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    tax_deductible BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Expense Tracking
CREATE TABLE expenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id),
    vendor_id UUID REFERENCES vendors(id),
    category_id UUID NOT NULL REFERENCES expense_categories(id),
    amount DECIMAL(10,2) NOT NULL,
    tax_amount DECIMAL(10,2),
    description TEXT NOT NULL,
    expense_date DATE NOT NULL,
    receipt_file_id UUID REFERENCES files(id),
    billable BOOLEAN DEFAULT false,
    billed BOOLEAN DEFAULT false,
    invoice_id UUID REFERENCES invoices(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- Enhanced Time Tracking (Kaseya-style)
ALTER TABLE time_entries ADD COLUMN IF NOT EXISTS task_description TEXT;
ALTER TABLE time_entries ADD COLUMN IF NOT EXISTS break_time_minutes INTEGER DEFAULT 0;
ALTER TABLE time_entries ADD COLUMN IF NOT EXISTS mileage DECIMAL(8,2);
ALTER TABLE time_entries ADD COLUMN IF NOT EXISTS location TEXT;
ALTER TABLE time_entries ADD COLUMN IF NOT EXISTS tags TEXT[] DEFAULT '{}';
ALTER TABLE time_entries ADD COLUMN IF NOT EXISTS timer_started_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE time_entries ADD COLUMN IF NOT EXISTS timer_paused_duration INTEGER DEFAULT 0;

-- Ticket Watchers
CREATE TABLE ticket_watchers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID NOT NULL REFERENCES tickets(id),
    user_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(ticket_id, user_id)
);

-- Ticket Comments/Updates
CREATE TABLE ticket_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID NOT NULL REFERENCES tickets(id),
    user_id UUID NOT NULL REFERENCES users(id),
    comment TEXT NOT NULL,
    internal BOOLEAN DEFAULT false, -- Internal comments not visible to clients
    time_spent_minutes INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Enhanced Projects with Templates
CREATE TABLE project_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    default_tasks JSONB DEFAULT '[]', -- Array of task templates
    estimated_hours DECIMAL(8,2),
    default_hourly_rate DECIMAL(8,2),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add project template reference to projects
ALTER TABLE projects ADD COLUMN IF NOT EXISTS template_id UUID REFERENCES project_templates(id);

-- Enhanced Invoicing
ALTER TABLE invoices ADD COLUMN IF NOT EXISTS recurring BOOLEAN DEFAULT false;
ALTER TABLE invoices ADD COLUMN IF NOT EXISTS recurring_frequency VARCHAR; -- monthly, quarterly, yearly
ALTER TABLE invoices ADD COLUMN IF NOT EXISTS next_invoice_date DATE;
ALTER TABLE invoices ADD COLUMN IF NOT EXISTS auto_send BOOLEAN DEFAULT false;
ALTER TABLE invoices ADD COLUMN IF NOT EXISTS pdf_generated BOOLEAN DEFAULT false;
ALTER TABLE invoices ADD COLUMN IF NOT EXISTS pdf_path VARCHAR;

-- Invoice Line Items
CREATE TABLE invoice_line_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id),
    description TEXT NOT NULL,
    quantity DECIMAL(8,2) DEFAULT 1,
    unit_price DECIMAL(10,2) NOT NULL,
    total_price DECIMAL(10,2) NOT NULL,
    tax_rate DECIMAL(5,2) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Quote System
CREATE TABLE quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id),
    project_id UUID REFERENCES projects(id),
    number VARCHAR NOT NULL UNIQUE,
    title VARCHAR NOT NULL,
    date DATE NOT NULL,
    expiry_date DATE,
    subtotal DECIMAL(10,2) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(10,2) NOT NULL DEFAULT 0,
    total DECIMAL(10,2) NOT NULL DEFAULT 0,
    discount_percentage DECIMAL(5,2),
    discount_amount DECIMAL(10,2),
    status VARCHAR DEFAULT 'draft', -- draft, sent, accepted, rejected, expired
    notes TEXT,
    terms TEXT,
    accepted_at TIMESTAMP WITH TIME ZONE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- Quote Line Items
CREATE TABLE quote_line_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quote_id UUID NOT NULL REFERENCES quotes(id),
    description TEXT NOT NULL,
    quantity DECIMAL(8,2) DEFAULT 1,
    unit_price DECIMAL(10,2) NOT NULL,
    total_price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_credentials_client_id ON credentials(client_id);
CREATE INDEX IF NOT EXISTS idx_credentials_expires_at ON credentials(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_domains_client_id ON domains(client_id);
CREATE INDEX IF NOT EXISTS idx_domains_expiry_date ON domains(expiry_date) WHERE expiry_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ssl_certificates_client_id ON ssl_certificates(client_id);
CREATE INDEX IF NOT EXISTS idx_ssl_certificates_expiry_date ON ssl_certificates(expiry_date);
CREATE INDEX IF NOT EXISTS idx_networks_client_id ON networks(client_id);
CREATE INDEX IF NOT EXISTS idx_software_licenses_client_id ON software_licenses(client_id);
CREATE INDEX IF NOT EXISTS idx_software_licenses_expiry_date ON software_licenses(expiry_date) WHERE expiry_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_files_client_id ON files(client_id);
CREATE INDEX IF NOT EXISTS idx_files_ticket_id ON files(ticket_id);
CREATE INDEX IF NOT EXISTS idx_expenses_client_id ON expenses(client_id);
CREATE INDEX IF NOT EXISTS idx_expenses_date ON expenses(expense_date);
CREATE INDEX IF NOT EXISTS idx_ticket_comments_ticket_id ON ticket_comments(ticket_id);
CREATE INDEX IF NOT EXISTS idx_recurring_tickets_next_run ON recurring_tickets(next_run) WHERE enabled = true;

-- Insert default expense categories
INSERT INTO expense_categories (name, description, tax_deductible) VALUES
('Office Supplies', 'General office supplies and materials', true),
('Software Licenses', 'Software licensing and subscriptions', true),
('Hardware', 'Computer equipment and hardware', true),
('Training', 'Employee training and certification', true),
('Travel', 'Business travel expenses', true),
('Marketing', 'Marketing and advertising expenses', true),
('Professional Services', 'Legal, accounting, and consulting services', true),
('Utilities', 'Internet, phone, and utility bills', true),
('Insurance', 'Business insurance premiums', true),
('Meals', 'Business meals and entertainment', false)
ON CONFLICT DO NOTHING;

-- Insert default auth provider configurations (disabled by default)
INSERT INTO auth_providers (name, provider_type, enabled, auth_url, token_url, userinfo_url, scopes) VALUES
('Google', 'oauth2', false, 'https://accounts.google.com/o/oauth2/auth', 'https://oauth2.googleapis.com/token', 'https://www.googleapis.com/oauth2/v2/userinfo', ARRAY['openid', 'profile', 'email']),
('Microsoft Azure', 'oauth2', false, 'https://login.microsoftonline.com/common/oauth2/v2.0/authorize', 'https://login.microsoftonline.com/common/oauth2/v2.0/token', 'https://graph.microsoft.com/v1.0/me', ARRAY['openid', 'profile', 'email']),
('GitHub', 'oauth2', false, 'https://github.com/login/oauth/authorize', 'https://github.com/login/oauth/access_token', 'https://api.github.com/user', ARRAY['user:email'])
ON CONFLICT DO NOTHING;