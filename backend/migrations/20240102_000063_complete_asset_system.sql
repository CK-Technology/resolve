-- Complete Asset Management System with all requested features
-- This includes: Networks, Email, Licenses, Wireless, and all integrations

-- Asset Types (predefined categories)
CREATE TABLE IF NOT EXISTS asset_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    category VARCHAR(50) NOT NULL,
    icon VARCHAR(100),
    custom_fields_schema JSONB DEFAULT '{}',
    is_system_type BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert comprehensive asset types
INSERT INTO asset_types (name, category, icon, is_system_type) VALUES
-- Hardware
('Desktop Computer', 'hardware', 'desktop', true),
('Laptop Computer', 'hardware', 'laptop', true),
('Server', 'hardware', 'server', true),
('Printer', 'hardware', 'printer', true),
('Scanner', 'hardware', 'scanner', true),
('Monitor', 'hardware', 'monitor', true),
('Phone', 'hardware', 'phone', true),
('Tablet', 'hardware', 'tablet', true),
('Camera', 'hardware', 'camera', true),
('UPS', 'hardware', 'ups', true),
-- Network Equipment & Infrastructure
('Switch', 'network', 'switch', true),
('Router', 'network', 'router', true),
('Firewall', 'network', 'firewall', true),
('Wireless Access Point', 'network', 'wifi', true),
('Network Attached Storage', 'network', 'nas', true),
('Load Balancer', 'network', 'loadbalancer', true),
('Modem', 'network', 'modem', true),
('Network Cable', 'network', 'cable', true),
('Patch Panel', 'network', 'patch-panel', true),
('Network Rack', 'network', 'rack', true),
('KVM Switch', 'network', 'kvm', true),
-- Email Infrastructure
('Mail Server', 'email', 'mail-server', true),
('Exchange Server', 'email', 'exchange', true),
('SMTP Relay', 'email', 'smtp', true),
('Email Security Gateway', 'email', 'email-security', true),
('Email Archiving System', 'email', 'archive', true),
('Office 365 Tenant', 'email', 'office365', true),
('Google Workspace', 'email', 'google-workspace', true),
-- Software & Applications
('Operating System', 'software', 'os', true),
('Antivirus', 'software', 'antivirus', true),
('Office Suite', 'software', 'office', true),
('Database', 'software', 'database', true),
('Application', 'software', 'application', true),
('Web Browser', 'software', 'browser', true),
('Backup Software', 'software', 'backup', true),
('Monitoring Software', 'software', 'monitoring', true),
-- Licenses
('Software License', 'license', 'license', true),
('Windows License', 'license', 'windows-license', true),
('Office License', 'license', 'office-license', true),
('CAL License', 'license', 'cal-license', true),
('Per-User License', 'license', 'user-license', true),
('Per-Device License', 'license', 'device-license', true),
('Subscription License', 'license', 'subscription-license', true),
('Volume License', 'license', 'volume-license', true),
('OEM License', 'license', 'oem-license', true),
-- Cloud/Virtual
('Virtual Machine', 'virtual', 'vm', true),
('Cloud Instance', 'virtual', 'cloud', true),
('Container', 'virtual', 'container', true),
('Kubernetes Cluster', 'virtual', 'k8s', true),
('Hyper-V Host', 'virtual', 'hyperv', true),
('VMware vSphere', 'virtual', 'vmware', true),
-- Services & Subscriptions
('Cloud Service', 'service', 'cloud-service', true),
('SaaS Application', 'service', 'saas', true),
('Domain Registration', 'service', 'domain', true),
('SSL Certificate', 'service', 'ssl-cert', true),
('Hosting Service', 'service', 'hosting', true),
('Backup Service', 'service', 'backup-service', true),
('Security Service', 'service', 'security-service', true)
ON CONFLICT (name) DO NOTHING;

-- License Management
CREATE TABLE IF NOT EXISTS licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    asset_id UUID REFERENCES assets(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    product_name VARCHAR(255) NOT NULL,
    vendor VARCHAR(255),
    license_type VARCHAR(50) NOT NULL,
    license_key TEXT,
    license_key_encrypted TEXT,
    seats_total INTEGER NOT NULL DEFAULT 1,
    seats_used INTEGER DEFAULT 0,
    seats_available INTEGER GENERATED ALWAYS AS (seats_total - seats_used) STORED,
    cost_per_license DECIMAL(10,2),
    total_cost DECIMAL(15,2),
    purchase_date DATE,
    renewal_date DATE,
    expiry_date DATE,
    auto_renewal BOOLEAN DEFAULT false,
    renewal_cost DECIMAL(15,2),
    maintenance_cost DECIMAL(15,2),
    support_expires DATE,
    vendor_contact_email VARCHAR(255),
    vendor_contact_phone VARCHAR(50),
    license_file_path TEXT,
    compliance_notes TEXT,
    status VARCHAR(50) DEFAULT 'active',
    alerts_enabled BOOLEAN DEFAULT true,
    alert_days_before_expiry INTEGER DEFAULT 30,
    notes TEXT,
    tags TEXT[] DEFAULT '{}',
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

-- License Assignments
CREATE TABLE IF NOT EXISTS license_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    license_id UUID NOT NULL REFERENCES licenses(id) ON DELETE CASCADE,
    assignment_type VARCHAR(50) NOT NULL,
    assigned_to_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    assigned_to_asset_id UUID REFERENCES assets(id) ON DELETE CASCADE,
    assigned_to_identifier VARCHAR(255),
    assignment_date DATE DEFAULT CURRENT_DATE,
    unassignment_date DATE,
    is_active BOOLEAN DEFAULT true,
    notes TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Email Infrastructure
CREATE TABLE IF NOT EXISTS email_systems (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    asset_id UUID REFERENCES assets(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    system_type VARCHAR(50) NOT NULL,
    server_hostname VARCHAR(255),
    ip_address INET,
    version VARCHAR(100),
    mailboxes_total INTEGER DEFAULT 0,
    mailboxes_used INTEGER DEFAULT 0,
    storage_quota_gb INTEGER,
    storage_used_gb INTEGER,
    status VARCHAR(50) DEFAULT 'active',
    ssl_certificate_id UUID,
    backup_enabled BOOLEAN DEFAULT true,
    backup_location TEXT,
    last_backup TIMESTAMPTZ,
    monitoring_enabled BOOLEAN DEFAULT true,
    last_monitored TIMESTAMPTZ,
    configuration JSONB DEFAULT '{}',
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

-- Email Domains
CREATE TABLE IF NOT EXISTS email_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_system_id UUID NOT NULL REFERENCES email_systems(id) ON DELETE CASCADE,
    domain_id UUID,
    domain_name VARCHAR(255) NOT NULL,
    is_primary BOOLEAN DEFAULT false,
    mx_records TEXT[] DEFAULT '{}',
    spf_record TEXT,
    dkim_enabled BOOLEAN DEFAULT false,
    dkim_selector VARCHAR(100),
    dkim_public_key TEXT,
    dmarc_policy VARCHAR(20),
    dmarc_record TEXT,
    status VARCHAR(50) DEFAULT 'active',
    verification_status VARCHAR(50) DEFAULT 'pending',
    last_verified TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

-- Email Accounts
CREATE TABLE IF NOT EXISTS email_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_system_id UUID NOT NULL REFERENCES email_systems(id) ON DELETE CASCADE,
    email_domain_id UUID REFERENCES email_domains(id) ON DELETE SET NULL,
    email_address VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    account_type VARCHAR(50) DEFAULT 'user',
    status VARCHAR(50) DEFAULT 'active',
    mailbox_size_mb INTEGER DEFAULT 0,
    mailbox_quota_mb INTEGER,
    last_login TIMESTAMPTZ,
    forwarding_enabled BOOLEAN DEFAULT false,
    forwarding_address VARCHAR(255),
    auto_reply_enabled BOOLEAN DEFAULT false,
    auto_reply_message TEXT,
    license_id UUID REFERENCES licenses(id),
    assigned_to_user VARCHAR(255),
    location VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

-- Add comprehensive indexes
CREATE INDEX IF NOT EXISTS idx_licenses_client_id ON licenses(client_id);
CREATE INDEX IF NOT EXISTS idx_licenses_expiry ON licenses(expiry_date) WHERE expiry_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_licenses_renewal ON licenses(renewal_date) WHERE renewal_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_licenses_status ON licenses(status);
CREATE INDEX IF NOT EXISTS idx_licenses_seats ON licenses(seats_total, seats_used);

CREATE INDEX IF NOT EXISTS idx_email_systems_client_id ON email_systems(client_id);
CREATE INDEX IF NOT EXISTS idx_email_systems_type ON email_systems(system_type);

CREATE INDEX IF NOT EXISTS idx_email_accounts_email ON email_accounts(email_address);
CREATE INDEX IF NOT EXISTS idx_email_accounts_status ON email_accounts(status);

-- Update triggers
CREATE OR REPLACE FUNCTION update_updated_at_generic()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_licenses_updated_at') THEN
        CREATE TRIGGER update_licenses_updated_at BEFORE UPDATE ON licenses
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_generic();
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_email_systems_updated_at') THEN
        CREATE TRIGGER update_email_systems_updated_at BEFORE UPDATE ON email_systems
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_generic();
    END IF;
END $$;