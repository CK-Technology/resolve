-- Resolve Demo Data
-- Sample data for demonstration purposes

-- Insert demo users
INSERT INTO users (id, email, password_hash, first_name, last_name, role, is_active, hourly_rate, department, phone, timezone) VALUES
(gen_random_uuid(), 'admin@resolve.demo', '$2b$12$LQv3c1yqBwEHxv68UVgAiO1.Q0IKEWKhLzxg2.fGQK8BmL.3K9FX6', 'Admin', 'User', 'admin', true, 150.00, 'Management', '555-0100', 'UTC'),
(gen_random_uuid(), 'tech@resolve.demo', '$2b$12$LQv3c1yqBwEHxv68UVgAiO1.Q0IKEWKhLzxg2.fGQK8BmL.3K9FX6', 'John', 'Smith', 'technician', true, 125.00, 'Technical', '555-0101', 'UTC'),
(gen_random_uuid(), 'sarah@resolve.demo', '$2b$12$LQv3c1yqBwEHxv68UVgAiO1.Q0IKEWKhLzxg2.fGQK8BmL.3K9FX6', 'Sarah', 'Johnson', 'technician', true, 130.00, 'Technical', '555-0102', 'UTC'),
(gen_random_uuid(), 'mike@resolve.demo', '$2b$12$LQv3c1yqBwEHxv68UVgAiO1.Q0IKEWKhLzxg2.fGQK8BmL.3K9FX6', 'Mike', 'Davis', 'manager', true, 175.00, 'Technical', '555-0103', 'UTC'),
(gen_random_uuid(), 'billing@resolve.demo', '$2b$12$LQv3c1yqBwEHxv68UVgAiO1.Q0IKEWKhLzxg2.fGQK8BmL.3K9FX6', 'Lisa', 'Brown', 'billing', true, 95.00, 'Finance', '555-0104', 'UTC')
ON CONFLICT (email) DO NOTHING;

-- Insert demo clients
INSERT INTO clients (id, name, email, phone, website, address, city, state, zip, country, client_type, is_active, default_hourly_rate, payment_terms, tax_rate, billing_address, notes) VALUES
(gen_random_uuid(), 'Acme Corporation', 'admin@acmecorp.com', '555-1000', 'https://acmecorp.com', '123 Business Ave', 'New York', 'NY', '10001', 'USA', 'business', true, 150.00, 30, 8.25, '123 Business Ave, New York, NY 10001', 'Large enterprise client with 200+ employees'),
(gen_random_uuid(), 'TechStart Inc', 'hello@techstart.io', '555-2000', 'https://techstart.io', '456 Innovation Blvd', 'San Francisco', 'CA', '94105', 'USA', 'startup', true, 175.00, 15, 8.75, '456 Innovation Blvd, San Francisco, CA 94105', 'Growing startup, high-growth potential'),
(gen_random_uuid(), 'Local Law Firm', 'info@locallegal.com', '555-3000', 'https://locallegal.com', '789 Justice Street', 'Chicago', 'IL', '60601', 'USA', 'professional', true, 200.00, 30, 7.50, '789 Justice Street, Chicago, IL 60601', 'Professional services firm requiring high security')
ON CONFLICT (name) DO NOTHING;

-- Get client IDs for foreign key references
DO $$
DECLARE
    acme_id UUID;
    tech_id UUID; 
    law_id UUID;
    admin_user_id UUID;
    tech_user_id UUID;
    sarah_user_id UUID;
BEGIN
    SELECT id INTO acme_id FROM clients WHERE name = 'Acme Corporation';
    SELECT id INTO tech_id FROM clients WHERE name = 'TechStart Inc';
    SELECT id INTO law_id FROM clients WHERE name = 'Local Law Firm';
    SELECT id INTO admin_user_id FROM users WHERE email = 'admin@resolve.demo';
    SELECT id INTO tech_user_id FROM users WHERE email = 'tech@resolve.demo';
    SELECT id INTO sarah_user_id FROM users WHERE email = 'sarah@resolve.demo';

    -- Insert demo contacts
    INSERT INTO contacts (client_id, first_name, last_name, email, phone, title, department, is_primary, is_billing, is_technical) VALUES
    (acme_id, 'Robert', 'Wilson', 'rwilson@acmecorp.com', '555-1001', 'IT Director', 'Information Technology', true, false, true),
    (acme_id, 'Jennifer', 'Lee', 'jlee@acmecorp.com', '555-1002', 'CFO', 'Finance', false, true, false),
    (tech_id, 'David', 'Chen', 'david@techstart.io', '555-2001', 'CTO', 'Engineering', true, true, true),
    (tech_id, 'Emma', 'Rodriguez', 'emma@techstart.io', '555-2002', 'Office Manager', 'Operations', false, false, false),
    (law_id, 'James', 'Taylor', 'jtaylor@locallegal.com', '555-3001', 'Managing Partner', 'Legal', true, true, false),
    (law_id, 'Michelle', 'White', 'mwhite@locallegal.com', '555-3002', 'IT Coordinator', 'Administration', false, false, true)
    ON CONFLICT DO NOTHING;

    -- Insert demo assets
    INSERT INTO assets (client_id, name, asset_type, manufacturer, model, serial_number, ip_address, mac_address, location, status, purchase_date, warranty_expiry, health_score, notes) VALUES
    (acme_id, 'DC-SRV-01', 'server', 'Dell', 'PowerEdge R750', 'SN001234567', '10.0.1.100', '00:1A:2B:3C:4D:5E', 'Data Center Rack A1', 'active', '2023-01-15', '2026-01-15', 92, 'Primary domain controller'),
    (acme_id, 'DC-SRV-02', 'server', 'Dell', 'PowerEdge R750', 'SN001234568', '10.0.1.101', '00:1A:2B:3C:4D:5F', 'Data Center Rack A1', 'active', '2023-01-15', '2026-01-15', 88, 'Secondary domain controller'),
    (acme_id, 'FW-01', 'firewall', 'Fortinet', 'FortiGate 100F', 'FG001234', '192.168.1.1', '00:09:0F:AA:BB:CC', 'Network Closet', 'active', '2022-06-01', '2025-06-01', 95, 'Main firewall'),
    (acme_id, 'SW-CORE-01', 'switch', 'Cisco', 'Catalyst 2960X', 'CS001234', '10.0.1.10', '00:1B:2C:3D:4E:5F', 'Network Closet', 'active', '2022-08-15', '2025-08-15', 90, 'Core network switch'),
    (tech_id, 'CLOUD-SRV-01', 'server', 'AWS', 'EC2 t3.large', 'i-1234567890abcdef0', '10.0.0.100', 'N/A', 'us-west-2a', 'active', '2023-03-01', '2024-03-01', 85, 'Main application server'),
    (tech_id, 'CLOUD-DB-01', 'database', 'AWS', 'RDS PostgreSQL', 'db-ABCDEF123456', '10.0.0.200', 'N/A', 'us-west-2b', 'active', '2023-03-01', '2024-03-01', 93, 'Production database'),
    (law_id, 'FILE-SRV-01', 'server', 'HP', 'ProLiant ML350', 'HP123456789', '192.168.10.100', '00:25:B3:AA:BB:CC', 'Server Room', 'active', '2021-09-01', '2024-09-01', 75, 'File server - needs replacement soon'),
    (law_id, 'BACKUP-NAS-01', 'nas', 'Synology', 'DS918+', 'SYN123456', '192.168.10.200', '00:11:32:AA:BB:CC', 'Server Room', 'active', '2022-11-01', '2025-11-01', 88, 'Backup storage')
    ON CONFLICT DO NOTHING;

    -- Insert demo tickets
    INSERT INTO tickets (client_id, subject, description, priority, status, category, assigned_to, created_by, sla_breached) VALUES
    (acme_id, 'Email server running slow', 'Users reporting slow email performance during peak hours. Need to investigate Exchange server performance.', 'high', 'in_progress', 'email', tech_user_id, admin_user_id, false),
    (acme_id, 'New employee onboarding', 'Setup accounts and equipment for 3 new hires starting Monday. Need AD accounts, email, laptop configuration.', 'medium', 'open', 'onboarding', sarah_user_id, admin_user_id, false),
    (acme_id, 'Printer network connectivity', 'HP LaserJet in accounting department cannot connect to network. Users cannot print invoices.', 'medium', 'pending', 'hardware', tech_user_id, admin_user_id, false),
    (tech_id, 'SSL certificate renewal', 'Website SSL certificate expires in 2 weeks. Need to renew and install new certificate.', 'high', 'open', 'security', sarah_user_id, admin_user_id, false),
    (tech_id, 'Database performance optimization', 'Application queries running slow. Need to analyze and optimize database performance.', 'high', 'in_progress', 'database', tech_user_id, admin_user_id, false),
    (tech_id, 'Backup verification', 'Monthly backup verification and testing. Ensure all critical data is being backed up properly.', 'low', 'completed', 'backup', sarah_user_id, admin_user_id, false),
    (law_id, 'File server storage full', 'Main file server at 95% capacity. Need to add storage or archive old files.', 'critical', 'open', 'storage', tech_user_id, admin_user_id, true),
    (law_id, 'Security assessment', 'Annual security assessment and penetration testing for compliance requirements.', 'medium', 'scheduled', 'security', admin_user_id, admin_user_id, false),
    (law_id, 'Software license audit', 'Quarterly audit of software licenses to ensure compliance and optimize costs.', 'low', 'in_progress', 'licensing', sarah_user_id, admin_user_id, false)
    ON CONFLICT DO NOTHING;

    -- Insert demo invoices
    INSERT INTO invoices (client_id, invoice_number, issue_date, due_date, status, subtotal, tax_amount, total_amount, notes) VALUES
    (acme_id, 'INV-2024-001', '2024-01-01', '2024-01-31', 'paid', 4500.00, 371.25, 4871.25, 'Monthly managed services - January 2024'),
    (acme_id, 'INV-2024-007', '2024-01-07', '2024-02-06', 'sent', 4500.00, 371.25, 4871.25, 'Monthly managed services - February 2024'),
    (tech_id, 'INV-2024-002', '2024-01-01', '2024-01-16', 'paid', 3200.00, 280.00, 3480.00, 'Cloud infrastructure management - January 2024'),
    (tech_id, 'INV-2024-008', '2024-01-07', '2024-01-22', 'sent', 3200.00, 280.00, 3480.00, 'Cloud infrastructure management - February 2024'),
    (law_id, 'INV-2024-003', '2024-01-01', '2024-01-31', 'paid', 2800.00, 210.00, 3010.00, 'IT support and maintenance - January 2024'),
    (law_id, 'INV-2024-009', '2024-01-07', '2024-02-06', 'overdue', 2800.00, 210.00, 3010.00, 'IT support and maintenance - February 2024')
    ON CONFLICT DO NOTHING;

    -- Insert demo time entries
    INSERT INTO time_entries (user_id, ticket_id, start_time, end_time, duration_minutes, description, billable, hourly_rate, billed) VALUES
    (tech_user_id, (SELECT id FROM tickets WHERE subject = 'Email server running slow' LIMIT 1), NOW() - INTERVAL '2 days', NOW() - INTERVAL '2 days' + INTERVAL '3 hours', 180, 'Investigating Exchange server performance issues', true, 125.00, false),
    (sarah_user_id, (SELECT id FROM tickets WHERE subject = 'New employee onboarding' LIMIT 1), NOW() - INTERVAL '1 day', NOW() - INTERVAL '1 day' + INTERVAL '2 hours', 120, 'Creating user accounts and setting up equipment', true, 130.00, false),
    (tech_user_id, (SELECT id FROM tickets WHERE subject = 'Database performance optimization' LIMIT 1), NOW() - INTERVAL '3 hours', NOW() - INTERVAL '1 hour', 120, 'Analyzing slow queries and optimizing indexes', true, 125.00, false)
    ON CONFLICT DO NOTHING;

    -- Insert demo documentation
    INSERT INTO documentation (client_id, title, slug, content, content_type, status, visibility, author_id) VALUES
    (NULL, 'Password Policy', 'password-policy', '# Company Password Policy

## Requirements
- Minimum 12 characters
- Must include uppercase, lowercase, numbers, and symbols
- Cannot reuse last 5 passwords
- Must be changed every 90 days

## Multi-Factor Authentication
- Required for all administrative accounts
- Required for remote access
- TOTP apps recommended (Google Authenticator, Authy)', 'markdown', 'published', 'internal', admin_user_id),
    (acme_id, 'Network Documentation', 'acme-network', '# Acme Corporation Network Documentation

## Network Overview
- Main office: 192.168.1.0/24
- VLAN 10: Workstations (192.168.10.0/24)
- VLAN 20: Servers (192.168.20.0/24)
- VLAN 30: Guest WiFi (192.168.30.0/24)

## Critical Servers
- DC-SRV-01: Primary Domain Controller (192.168.20.10)
- DC-SRV-02: Secondary Domain Controller (192.168.20.11)
- FILE-SRV-01: File Server (192.168.20.20)', 'markdown', 'published', 'client', tech_user_id),
    (tech_id, 'AWS Infrastructure', 'techstart-aws', '# TechStart AWS Infrastructure

## Production Environment
- Region: us-west-2
- VPC: 10.0.0.0/16
- Application Servers: 10.0.1.0/24
- Database Subnet: 10.0.2.0/24

## Services Used
- EC2: Application hosting
- RDS: PostgreSQL database
- S3: File storage and backups
- CloudFront: CDN
- Route 53: DNS management', 'markdown', 'published', 'client', sarah_user_id)
    ON CONFLICT DO NOTHING;

    -- Insert demo password entries
    INSERT INTO password_vault (client_id, name, username, password, url, notes, owner_id, tags) VALUES
    (acme_id, 'Domain Administrator', 'administrator', 'encrypted_password_1', NULL, 'Primary domain admin account', admin_user_id, ARRAY['admin', 'windows', 'critical']),
    (acme_id, 'Firewall Admin', 'admin', 'encrypted_password_2', 'https://192.168.1.1', 'FortiGate firewall admin access', tech_user_id, ARRAY['network', 'firewall', 'admin']),
    (tech_id, 'AWS Root Account', 'root@techstart.io', 'encrypted_password_3', 'https://console.aws.amazon.com', 'AWS root account - use sparingly', admin_user_id, ARRAY['cloud', 'aws', 'critical']),
    (tech_id, 'Database Admin', 'dbadmin', 'encrypted_password_4', NULL, 'PostgreSQL admin user', sarah_user_id, ARRAY['database', 'postgresql']),
    (law_id, 'File Server Admin', 'administrator', 'encrypted_password_5', NULL, 'Windows file server admin', tech_user_id, ARRAY['windows', 'fileserver'])
    ON CONFLICT DO NOTHING;

    -- Insert demo recurring billing
    INSERT INTO recurring_billing (client_id, name, description, billing_type, amount, frequency, start_date, next_billing_date, payment_terms_days, status) VALUES
    (acme_id, 'Managed IT Services', 'Comprehensive IT management and support', 'fixed', 4500.00, 'monthly', '2024-01-01', '2024-02-01', 30, 'active'),
    (tech_id, 'Cloud Management', 'AWS infrastructure management and monitoring', 'fixed', 3200.00, 'monthly', '2024-01-01', '2024-02-01', 15, 'active'),
    (law_id, 'IT Support Package', 'IT support and maintenance services', 'fixed', 2800.00, 'monthly', '2024-01-01', '2024-02-01', 30, 'active')
    ON CONFLICT DO NOTHING;

    -- Insert client health scores
    INSERT INTO client_health_scores (client_id, overall_score, asset_health_score, ticket_satisfaction_score, financial_health_score, communication_score, security_score, risk_level, calculation_date) VALUES
    (acme_id, 82, 90, 85, 95, 75, 80, 'low', CURRENT_DATE),
    (tech_id, 88, 85, 90, 90, 85, 90, 'low', CURRENT_DATE),
    (law_id, 65, 75, 60, 80, 70, 45, 'medium', CURRENT_DATE)
    ON CONFLICT DO NOTHING;

    -- Insert demo KPI values
    INSERT INTO bi_metrics_daily (metric_date, client_id, revenue_total, tickets_created, tickets_resolved, hours_billable, assets_total) VALUES
    (CURRENT_DATE - 6, NULL, 10500.00, 8, 6, 42.5, 8),
    (CURRENT_DATE - 5, NULL, 11200.00, 12, 10, 38.0, 8),
    (CURRENT_DATE - 4, NULL, 9800.00, 6, 8, 45.5, 8),
    (CURRENT_DATE - 3, NULL, 12400.00, 15, 12, 52.0, 8),
    (CURRENT_DATE - 2, NULL, 10800.00, 9, 11, 41.5, 8),
    (CURRENT_DATE - 1, NULL, 11600.00, 11, 9, 47.0, 8),
    (CURRENT_DATE, NULL, 10500.00, 9, 7, 39.5, 8)
    ON CONFLICT DO NOTHING;

    RAISE NOTICE 'Demo data inserted successfully for clients: %, %, %', acme_id, tech_id, law_id;
END $$;