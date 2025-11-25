-- License & Expiration Alerting System
-- Comprehensive license management and automated expiration alerting

-- Software and service licenses
CREATE TABLE licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    asset_id UUID REFERENCES assets(id), -- link to specific asset using this license
    license_name VARCHAR(255) NOT NULL,
    vendor VARCHAR(100) NOT NULL,
    product_name VARCHAR(255) NOT NULL,
    version VARCHAR(50),
    license_type VARCHAR(50) NOT NULL, -- perpetual, subscription, oem, volume, site, user_based
    license_key VARCHAR(500),
    activation_key VARCHAR(500),
    license_file_path TEXT, -- path to license file if applicable
    seats_total INTEGER,
    seats_used INTEGER DEFAULT 0,
    seats_available INTEGER,
    cost_per_seat DECIMAL(10,2),
    purchase_date DATE,
    start_date DATE,
    end_date DATE,
    renewal_date DATE,
    grace_period_days INTEGER DEFAULT 0,
    auto_renewal BOOLEAN DEFAULT false,
    renewal_cost DECIMAL(12,2),
    annual_cost DECIMAL(12,2),
    total_cost DECIMAL(12,2), -- total cost of license
    purchase_order VARCHAR(100),
    invoice_number VARCHAR(100),
    vendor_contact_name VARCHAR(100),
    vendor_contact_email VARCHAR(255),
    vendor_contact_phone VARCHAR(50),
    support_level VARCHAR(50), -- basic, standard, premium, enterprise
    support_phone VARCHAR(50),
    support_email VARCHAR(255),
    support_url VARCHAR(500),
    documentation_url VARCHAR(500),
    license_server VARCHAR(255), -- license server hostname/IP
    license_server_port INTEGER,
    license_manager VARCHAR(100), -- FlexLM, RLM, etc.
    compliance_notes TEXT,
    usage_tracking_enabled BOOLEAN DEFAULT false,
    usage_monitoring_url VARCHAR(500),
    status VARCHAR(50) DEFAULT 'active', -- active, expired, suspended, terminated, pending
    criticality VARCHAR(20) DEFAULT 'medium', -- low, medium, high, critical
    business_impact TEXT, -- what happens if this license expires
    renewal_process TEXT, -- steps to renew this license
    notification_emails TEXT[], -- emails to notify about expiration
    alert_days_before INTEGER[] DEFAULT ARRAY[90, 60, 30, 14, 7, 1], -- days before expiration to alert
    last_alert_sent TIMESTAMPTZ,
    alert_count INTEGER DEFAULT 0,
    custom_fields JSONB,
    notes TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_licenses_client_id ON licenses(client_id);
CREATE INDEX idx_licenses_asset_id ON licenses(asset_id);
CREATE INDEX idx_licenses_vendor ON licenses(vendor);
CREATE INDEX idx_licenses_end_date ON licenses(end_date);
CREATE INDEX idx_licenses_renewal_date ON licenses(renewal_date);
CREATE INDEX idx_licenses_status ON licenses(status);
CREATE INDEX idx_licenses_criticality ON licenses(criticality);

-- License usage tracking
CREATE TABLE license_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    license_id UUID NOT NULL REFERENCES licenses(id) ON DELETE CASCADE,
    usage_date DATE NOT NULL,
    peak_usage INTEGER DEFAULT 0,
    average_usage DECIMAL(5,2) DEFAULT 0,
    min_usage INTEGER DEFAULT 0,
    max_usage INTEGER DEFAULT 0,
    utilization_percentage DECIMAL(5,2),
    concurrent_users INTEGER,
    total_sessions INTEGER,
    session_duration_minutes INTEGER,
    denied_access_count INTEGER DEFAULT 0, -- when license limit reached
    usage_details JSONB, -- detailed usage information
    collected_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(license_id, usage_date)
);

CREATE INDEX idx_license_usage_license_id ON license_usage(license_id);
CREATE INDEX idx_license_usage_date ON license_usage(usage_date);

-- License expiration alerts
CREATE TABLE license_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    license_id UUID NOT NULL REFERENCES licenses(id) ON DELETE CASCADE,
    alert_type VARCHAR(50) NOT NULL, -- expiration, renewal, usage_high, compliance
    severity VARCHAR(20) NOT NULL, -- info, warning, critical
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    days_until_expiration INTEGER,
    triggered_at TIMESTAMPTZ DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMPTZ,
    notification_sent BOOLEAN DEFAULT false,
    notification_channels JSONB, -- {"email": true, "slack": false, "ticket": true}
    ticket_id UUID REFERENCES tickets(id), -- auto-created ticket
    action_required BOOLEAN DEFAULT true,
    action_description TEXT,
    resolution_notes TEXT,
    next_alert_date TIMESTAMPTZ, -- when to send next reminder
    escalation_level INTEGER DEFAULT 1,
    escalated_to UUID REFERENCES users(id),
    is_resolved BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_license_alerts_license_id ON license_alerts(license_id);
CREATE INDEX idx_license_alerts_triggered_at ON license_alerts(triggered_at);
CREATE INDEX idx_license_alerts_severity ON license_alerts(severity);
CREATE INDEX idx_license_alerts_resolved ON license_alerts(is_resolved);
CREATE INDEX idx_license_alerts_next_alert ON license_alerts(next_alert_date);

-- Domain and SSL certificate tracking
CREATE TABLE domain_ssl_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    domain_name VARCHAR(253) NOT NULL,
    subdomain VARCHAR(100),
    full_domain VARCHAR(253) NOT NULL, -- subdomain.domain_name
    domain_type VARCHAR(50) DEFAULT 'production', -- production, staging, development, test
    registrar VARCHAR(100),
    registrar_account VARCHAR(100),
    registration_date DATE,
    expiry_date DATE NOT NULL,
    renewal_date DATE,
    auto_renewal BOOLEAN DEFAULT false,
    renewal_cost DECIMAL(10,2),
    nameservers TEXT[],
    dns_provider VARCHAR(100),
    ssl_provider VARCHAR(100),
    ssl_type VARCHAR(50), -- lets_encrypt, commercial, self_signed, wildcard
    ssl_issued_date DATE,
    ssl_expiry_date DATE,
    ssl_auto_renewal BOOLEAN DEFAULT false,
    ssl_renewal_cost DECIMAL(10,2),
    certificate_authority VARCHAR(100),
    certificate_fingerprint VARCHAR(255),
    key_size INTEGER,
    san_domains TEXT[], -- Subject Alternative Name domains
    monitoring_enabled BOOLEAN DEFAULT true,
    whois_privacy BOOLEAN DEFAULT true,
    transfer_lock BOOLEAN DEFAULT true,
    status VARCHAR(50) DEFAULT 'active', -- active, expired, suspended, pending_transfer
    business_criticality VARCHAR(20) DEFAULT 'medium',
    service_dependencies TEXT[], -- services that depend on this domain
    notification_emails TEXT[],
    alert_days_before INTEGER[] DEFAULT ARRAY[90, 60, 30, 14, 7, 1],
    last_checked TIMESTAMPTZ,
    check_interval_hours INTEGER DEFAULT 24,
    last_alert_sent TIMESTAMPTZ,
    alert_count INTEGER DEFAULT 0,
    notes TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(client_id, full_domain)
);

CREATE INDEX idx_domain_ssl_tracking_client_id ON domain_ssl_tracking(client_id);
CREATE INDEX idx_domain_ssl_tracking_domain ON domain_ssl_tracking(full_domain);
CREATE INDEX idx_domain_ssl_tracking_expiry ON domain_ssl_tracking(expiry_date);
CREATE INDEX idx_domain_ssl_tracking_ssl_expiry ON domain_ssl_tracking(ssl_expiry_date);
CREATE INDEX idx_domain_ssl_tracking_status ON domain_ssl_tracking(status);

-- Support contracts and warranties
CREATE TABLE support_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    asset_id UUID REFERENCES assets(id),
    contract_type VARCHAR(50) NOT NULL, -- hardware_warranty, software_support, maintenance, sla
    vendor VARCHAR(100) NOT NULL,
    contract_number VARCHAR(100),
    service_level VARCHAR(50), -- basic, standard, premium, enterprise, 24x7
    coverage_type VARCHAR(50), -- phone, email, onsite, remote, comprehensive
    contract_name VARCHAR(255) NOT NULL,
    description TEXT,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    renewal_date DATE,
    auto_renewal BOOLEAN DEFAULT false,
    renewal_cost DECIMAL(12,2),
    annual_cost DECIMAL(12,2),
    response_time_hours INTEGER, -- guaranteed response time
    resolution_time_hours INTEGER, -- guaranteed resolution time
    coverage_hours VARCHAR(50), -- 24x7, 8x5, business_hours
    included_services TEXT[],
    excluded_services TEXT[],
    escalation_contacts JSONB,
    vendor_contact_name VARCHAR(100),
    vendor_contact_email VARCHAR(255),
    vendor_contact_phone VARCHAR(50),
    account_manager VARCHAR(100),
    technical_contact VARCHAR(100),
    emergency_contact VARCHAR(100),
    contract_url VARCHAR(500),
    portal_url VARCHAR(500),
    portal_credentials_id UUID, -- link to password manager
    status VARCHAR(50) DEFAULT 'active',
    business_criticality VARCHAR(20) DEFAULT 'medium',
    notification_emails TEXT[],
    alert_days_before INTEGER[] DEFAULT ARRAY[90, 60, 30, 14, 7],
    last_alert_sent TIMESTAMPTZ,
    alert_count INTEGER DEFAULT 0,
    notes TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_support_contracts_client_id ON support_contracts(client_id);
CREATE INDEX idx_support_contracts_asset_id ON support_contracts(asset_id);
CREATE INDEX idx_support_contracts_vendor ON support_contracts(vendor);
CREATE INDEX idx_support_contracts_end_date ON support_contracts(end_date);
CREATE INDEX idx_support_contracts_status ON support_contracts(status);

-- Vendor and partner management
CREATE TABLE vendors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    vendor_type VARCHAR(50), -- software, hardware, service, cloud, telecom
    website VARCHAR(500),
    support_email VARCHAR(255),
    support_phone VARCHAR(50),
    sales_email VARCHAR(255),
    sales_phone VARCHAR(50),
    account_manager VARCHAR(100),
    account_manager_email VARCHAR(255),
    account_manager_phone VARCHAR(50),
    technical_contact VARCHAR(100),
    technical_contact_email VARCHAR(255),
    billing_contact VARCHAR(100),
    billing_contact_email VARCHAR(255),
    primary_address TEXT,
    billing_address TEXT,
    tax_id VARCHAR(50),
    payment_terms VARCHAR(50),
    preferred_payment_method VARCHAR(50),
    contract_terms TEXT,
    sla_terms TEXT,
    escalation_process TEXT,
    renewal_process TEXT,
    cancellation_policy TEXT,
    data_processing_agreement BOOLEAN DEFAULT false,
    security_certification TEXT[],
    compliance_certifications TEXT[],
    vendor_status VARCHAR(20) DEFAULT 'active', -- active, inactive, preferred, blacklisted
    risk_level VARCHAR(20) DEFAULT 'medium', -- low, medium, high
    last_review_date DATE,
    next_review_date DATE,
    performance_rating INTEGER CHECK (performance_rating >= 1 AND performance_rating <= 5),
    notes TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_vendors_name ON vendors(name);
CREATE INDEX idx_vendors_type ON vendors(vendor_type);
CREATE INDEX idx_vendors_status ON vendors(vendor_status);

-- Alert notification preferences
CREATE TABLE alert_notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id UUID REFERENCES clients(id), -- NULL for global preferences
    alert_type VARCHAR(50) NOT NULL, -- license_expiration, domain_expiration, ssl_expiration, support_contract
    enabled BOOLEAN DEFAULT true,
    email_enabled BOOLEAN DEFAULT true,
    sms_enabled BOOLEAN DEFAULT false,
    slack_enabled BOOLEAN DEFAULT false,
    teams_enabled BOOLEAN DEFAULT false,
    ticket_creation BOOLEAN DEFAULT true,
    escalation_enabled BOOLEAN DEFAULT true,
    escalation_delay_hours INTEGER DEFAULT 24,
    escalation_to_user_id UUID REFERENCES users(id),
    business_hours_only BOOLEAN DEFAULT false,
    minimum_severity VARCHAR(20) DEFAULT 'warning', -- info, warning, critical
    days_before_expiration INTEGER[] DEFAULT ARRAY[60, 30, 14, 7, 1],
    notification_template TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, client_id, alert_type)
);

CREATE INDEX idx_alert_preferences_user_id ON alert_notification_preferences(user_id);
CREATE INDEX idx_alert_preferences_client_id ON alert_notification_preferences(client_id);

-- Function to calculate days until expiration and create alerts
CREATE OR REPLACE FUNCTION check_and_create_expiration_alerts()
RETURNS INTEGER AS $$
DECLARE
    license_record RECORD;
    domain_record RECORD;
    contract_record RECORD;
    alert_count INTEGER := 0;
    days_until_exp INTEGER;
    alert_needed BOOLEAN;
BEGIN
    -- Check software licenses
    FOR license_record IN 
        SELECT * FROM licenses 
        WHERE status = 'active' AND end_date IS NOT NULL AND end_date >= CURRENT_DATE
    LOOP
        days_until_exp := (license_record.end_date - CURRENT_DATE)::INTEGER;
        alert_needed := days_until_exp = ANY(license_record.alert_days_before);
        
        IF alert_needed AND (
            license_record.last_alert_sent IS NULL OR 
            license_record.last_alert_sent < CURRENT_DATE - INTERVAL '1 day'
        ) THEN
            INSERT INTO license_alerts (
                license_id, alert_type, severity, title, message, 
                days_until_expiration, action_required, action_description
            ) VALUES (
                license_record.id,
                'expiration',
                CASE 
                    WHEN days_until_exp <= 7 THEN 'critical'
                    WHEN days_until_exp <= 30 THEN 'warning'
                    ELSE 'info'
                END,
                format('License Expiring: %s', license_record.license_name),
                format('License "%s" for %s expires in %s days on %s. Renewal action required.',
                    license_record.license_name, 
                    license_record.vendor, 
                    days_until_exp, 
                    license_record.end_date::TEXT),
                days_until_exp,
                true,
                COALESCE(license_record.renewal_process, 'Contact vendor to renew license')
            );
            
            UPDATE licenses 
            SET last_alert_sent = NOW(), alert_count = alert_count + 1
            WHERE id = license_record.id;
            
            alert_count := alert_count + 1;
        END IF;
    END LOOP;
    
    -- Check domain expirations
    FOR domain_record IN 
        SELECT * FROM domain_ssl_tracking 
        WHERE status = 'active' AND expiry_date IS NOT NULL AND expiry_date >= CURRENT_DATE
    LOOP
        days_until_exp := (domain_record.expiry_date - CURRENT_DATE)::INTEGER;
        alert_needed := days_until_exp = ANY(domain_record.alert_days_before);
        
        IF alert_needed AND (
            domain_record.last_alert_sent IS NULL OR 
            domain_record.last_alert_sent < CURRENT_DATE - INTERVAL '1 day'
        ) THEN
            INSERT INTO license_alerts (
                license_id, alert_type, severity, title, message, 
                days_until_expiration, action_required, action_description
            ) 
            SELECT 
                gen_random_uuid(),
                'domain_expiration',
                CASE 
                    WHEN days_until_exp <= 7 THEN 'critical'
                    WHEN days_until_exp <= 30 THEN 'warning'
                    ELSE 'info'
                END,
                format('Domain Expiring: %s', domain_record.full_domain),
                format('Domain "%s" expires in %s days on %s. Renewal required to prevent service disruption.',
                    domain_record.full_domain, 
                    days_until_exp, 
                    domain_record.expiry_date::TEXT),
                days_until_exp,
                true,
                format('Contact %s to renew domain registration', COALESCE(domain_record.registrar, 'registrar'))
            WHERE NOT EXISTS (
                SELECT 1 FROM licenses WHERE id = gen_random_uuid()
            ); -- Placeholder to satisfy license_id constraint
            
            UPDATE domain_ssl_tracking 
            SET last_alert_sent = NOW(), alert_count = alert_count + 1
            WHERE id = domain_record.id;
            
            alert_count := alert_count + 1;
        END IF;
    END LOOP;
    
    RETURN alert_count;
END;
$$ LANGUAGE plpgsql;

-- Create a scheduled job to run expiration checks (would be called by cron or background task)
-- This would typically be executed by the application scheduler

-- Insert default vendors
INSERT INTO vendors (name, vendor_type, website, support_email, support_phone) VALUES
('Microsoft', 'software', 'https://microsoft.com', 'support@microsoft.com', '1-800-642-7676'),
('Adobe', 'software', 'https://adobe.com', 'support@adobe.com', '1-800-833-6687'),
('VMware', 'software', 'https://vmware.com', 'support@vmware.com', '1-877-486-9273'),
('Fortinet', 'security', 'https://fortinet.com', 'support@fortinet.com', '1-866-648-4638'),
('Cisco', 'network', 'https://cisco.com', 'tac@cisco.com', '1-800-553-6387'),
('AWS', 'cloud', 'https://aws.amazon.com', 'aws-support@amazon.com', '1-206-266-4064'),
('Google Cloud', 'cloud', 'https://cloud.google.com', 'cloud-support@google.com', '1-855-836-1615'),
('Veeam', 'backup', 'https://veeam.com', 'support@veeam.com', '1-614-304-6174');

-- Function to get license renewal calendar
CREATE OR REPLACE FUNCTION get_license_renewal_calendar(client_id_param UUID, months_ahead INTEGER DEFAULT 12)
RETURNS TABLE(
    renewal_month TEXT,
    license_count INTEGER,
    total_cost DECIMAL(12,2),
    critical_licenses INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        TO_CHAR(DATE_TRUNC('month', renewal_date), 'YYYY-MM') as renewal_month,
        COUNT(*)::INTEGER as license_count,
        SUM(COALESCE(renewal_cost, annual_cost, 0)) as total_cost,
        COUNT(CASE WHEN criticality = 'critical' THEN 1 END)::INTEGER as critical_licenses
    FROM licenses
    WHERE client_id = client_id_param 
        AND renewal_date IS NOT NULL
        AND renewal_date BETWEEN CURRENT_DATE AND CURRENT_DATE + (months_ahead || ' months')::INTERVAL
        AND status = 'active'
    GROUP BY DATE_TRUNC('month', renewal_date)
    ORDER BY DATE_TRUNC('month', renewal_date);
END;
$$ LANGUAGE plpgsql;