-- Asset Discovery and Lifecycle Management for Resolve
-- Network scanning, auto-discovery, warranty tracking, lifecycle management

-- Asset manufacturers database
CREATE TABLE asset_manufacturers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    website VARCHAR(500),
    support_url VARCHAR(500),
    support_phone VARCHAR(50),
    support_email VARCHAR(255),
    warranty_check_url VARCHAR(500),
    api_endpoint VARCHAR(500),
    api_key TEXT, -- Encrypted
    logo_url VARCHAR(500),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Asset models catalog
CREATE TABLE asset_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manufacturer_id UUID REFERENCES asset_manufacturers(id),
    model_number VARCHAR(255) NOT NULL,
    model_name VARCHAR(255),
    category VARCHAR(100), -- laptop, desktop, server, network, printer, etc
    specifications JSONB DEFAULT '{}',
    image_url VARCHAR(500),
    eol_date DATE, -- End of life
    end_of_support_date DATE,
    typical_lifespan_years INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(manufacturer_id, model_number)
);

-- Enhanced assets table additions
ALTER TABLE assets ADD COLUMN IF NOT EXISTS manufacturer_id UUID REFERENCES asset_manufacturers(id);
ALTER TABLE assets ADD COLUMN IF NOT EXISTS model_id UUID REFERENCES asset_models(id);
ALTER TABLE assets ADD COLUMN IF NOT EXISTS discovered_at TIMESTAMPTZ;
ALTER TABLE assets ADD COLUMN IF NOT EXISTS discovery_method VARCHAR(50); -- manual, scan, api, agent
ALTER TABLE assets ADD COLUMN IF NOT EXISTS mac_address VARCHAR(17);
ALTER TABLE assets ADD COLUMN IF NOT EXISTS last_seen TIMESTAMPTZ;
ALTER TABLE assets ADD COLUMN IF NOT EXISTS lifecycle_stage VARCHAR(50) DEFAULT 'active'; -- planned, deployed, active, retiring, retired
ALTER TABLE assets ADD COLUMN IF NOT EXISTS replacement_date DATE;
ALTER TABLE assets ADD COLUMN IF NOT EXISTS disposal_date DATE;
ALTER TABLE assets ADD COLUMN IF NOT EXISTS disposal_method VARCHAR(100);
ALTER TABLE assets ADD COLUMN IF NOT EXISTS disposal_certificate VARCHAR(500);
ALTER TABLE assets ADD COLUMN IF NOT EXISTS installed_software JSONB DEFAULT '[]';
ALTER TABLE assets ADD COLUMN IF NOT EXISTS performance_metrics JSONB DEFAULT '{}';
ALTER TABLE assets ADD COLUMN IF NOT EXISTS health_score INTEGER DEFAULT 100; -- 0-100
ALTER TABLE assets ADD COLUMN IF NOT EXISTS parent_asset_id UUID REFERENCES assets(id);
ALTER TABLE assets ADD COLUMN IF NOT EXISTS location_details JSONB DEFAULT '{}'; -- rack, shelf, room, etc

-- Asset discovery scans
CREATE TABLE discovery_scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id),
    scan_type VARCHAR(50) NOT NULL, -- network, agent, api, manual
    ip_range VARCHAR(255),
    subnet_mask VARCHAR(15),
    credentials_id UUID REFERENCES credentials(id),
    status VARCHAR(50) DEFAULT 'pending', -- pending, running, completed, failed
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    assets_discovered INTEGER DEFAULT 0,
    assets_updated INTEGER DEFAULT 0,
    new_assets INTEGER DEFAULT 0,
    scan_results JSONB DEFAULT '{}',
    error_message TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Discovered devices (staging before becoming assets)
CREATE TABLE discovered_devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID REFERENCES discovery_scans(id) ON DELETE CASCADE,
    client_id UUID REFERENCES clients(id),
    hostname VARCHAR(255),
    ip_address INET,
    mac_address VARCHAR(17),
    manufacturer VARCHAR(255),
    model VARCHAR(255),
    serial_number VARCHAR(255),
    device_type VARCHAR(100),
    operating_system VARCHAR(255),
    os_version VARCHAR(100),
    open_ports INTEGER[],
    services JSONB DEFAULT '[]',
    software JSONB DEFAULT '[]',
    hardware_info JSONB DEFAULT '{}',
    network_info JSONB DEFAULT '{}',
    discovered_at TIMESTAMPTZ DEFAULT NOW(),
    last_seen TIMESTAMPTZ DEFAULT NOW(),
    confidence_score INTEGER DEFAULT 50, -- 0-100 confidence in identification
    approved BOOLEAN DEFAULT false,
    approved_by UUID REFERENCES users(id),
    converted_to_asset_id UUID REFERENCES assets(id),
    ignore BOOLEAN DEFAULT false,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Asset warranty tracking
CREATE TABLE asset_warranties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    warranty_type VARCHAR(50) NOT NULL, -- manufacturer, extended, third-party
    provider VARCHAR(255) NOT NULL,
    contract_number VARCHAR(255),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    coverage_type VARCHAR(100), -- parts, labor, onsite, mail-in
    response_time VARCHAR(100), -- next-business-day, 4-hour, etc
    cost DECIMAL(10,2),
    renewable BOOLEAN DEFAULT false,
    auto_renew BOOLEAN DEFAULT false,
    contact_name VARCHAR(255),
    contact_phone VARCHAR(50),
    contact_email VARCHAR(255),
    claim_process TEXT,
    notes TEXT,
    document_urls JSONB DEFAULT '[]',
    alert_days_before INTEGER DEFAULT 30,
    last_alert_sent TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Asset lifecycle events
CREATE TABLE asset_lifecycle_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL, -- purchased, received, deployed, moved, upgraded, repaired, retired
    event_date TIMESTAMPTZ NOT NULL,
    description TEXT,
    performed_by UUID REFERENCES users(id),
    old_value JSONB,
    new_value JSONB,
    cost DECIMAL(10,2),
    ticket_id UUID REFERENCES tickets(id),
    attachments JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Asset maintenance schedules
CREATE TABLE asset_maintenance_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    maintenance_type VARCHAR(100) NOT NULL, -- preventive, predictive, corrective
    name VARCHAR(255) NOT NULL,
    description TEXT,
    frequency VARCHAR(50), -- daily, weekly, monthly, quarterly, annually
    last_performed TIMESTAMPTZ,
    next_due DATE NOT NULL,
    assigned_to UUID REFERENCES users(id),
    estimated_duration_hours DECIMAL(5,2),
    checklist JSONB DEFAULT '[]',
    auto_create_ticket BOOLEAN DEFAULT true,
    alert_days_before INTEGER DEFAULT 7,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Asset dependencies and relationships
CREATE TABLE asset_dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    child_asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    dependency_type VARCHAR(100) NOT NULL, -- requires, supports, connects-to, hosted-on
    criticality VARCHAR(50) DEFAULT 'medium', -- critical, high, medium, low
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(parent_asset_id, child_asset_id, dependency_type)
);

-- Asset QR/Barcode labels
CREATE TABLE asset_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    label_type VARCHAR(50) NOT NULL, -- qr, barcode, rfid, nfc
    label_code VARCHAR(255) NOT NULL UNIQUE,
    label_data JSONB DEFAULT '{}',
    print_template VARCHAR(100),
    printed_at TIMESTAMPTZ,
    printed_by UUID REFERENCES users(id),
    last_scanned TIMESTAMPTZ,
    scan_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Asset software inventory
CREATE TABLE asset_software (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    software_name VARCHAR(255) NOT NULL,
    vendor VARCHAR(255),
    version VARCHAR(100),
    installed_date DATE,
    last_used TIMESTAMPTZ,
    license_key VARCHAR(500), -- Encrypted
    license_type VARCHAR(50),
    license_id UUID REFERENCES software_licenses(id),
    install_location TEXT,
    size_mb INTEGER,
    auto_update BOOLEAN DEFAULT false,
    requires_license BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Asset performance monitoring
CREATE TABLE asset_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    metric_type VARCHAR(100) NOT NULL, -- cpu, memory, disk, network, uptime
    metric_value DECIMAL(10,2) NOT NULL,
    metric_unit VARCHAR(50), -- percent, GB, Mbps, etc
    threshold_warning DECIMAL(10,2),
    threshold_critical DECIMAL(10,2),
    is_healthy BOOLEAN DEFAULT true,
    recorded_at TIMESTAMPTZ DEFAULT NOW()
);

-- Network discovery configurations
CREATE TABLE network_discovery_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id),
    name VARCHAR(255) NOT NULL,
    enabled BOOLEAN DEFAULT true,
    scan_frequency VARCHAR(50), -- hourly, daily, weekly, monthly
    ip_ranges TEXT[], -- Array of IP ranges to scan
    excluded_ips TEXT[], -- IPs to exclude
    discovery_methods TEXT[], -- ping, snmp, wmi, ssh, agent
    snmp_community VARCHAR(255), -- Encrypted
    snmp_version VARCHAR(10) DEFAULT 'v2c',
    ssh_credentials_id UUID REFERENCES credentials(id),
    wmi_credentials_id UUID REFERENCES credentials(id),
    auto_approve_devices BOOLEAN DEFAULT false,
    auto_create_assets BOOLEAN DEFAULT false,
    notification_email VARCHAR(255),
    last_scan TIMESTAMPTZ,
    next_scan TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Asset depreciation tracking
CREATE TABLE asset_depreciation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    depreciation_method VARCHAR(50) NOT NULL, -- straight-line, declining-balance, sum-of-years
    initial_value DECIMAL(10,2) NOT NULL,
    salvage_value DECIMAL(10,2) DEFAULT 0,
    useful_life_years INTEGER NOT NULL,
    depreciation_start_date DATE NOT NULL,
    current_value DECIMAL(10,2),
    accumulated_depreciation DECIMAL(10,2) DEFAULT 0,
    last_calculated TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_discovered_devices_scan_id ON discovered_devices(scan_id);
CREATE INDEX idx_discovered_devices_ip ON discovered_devices(ip_address);
CREATE INDEX idx_discovered_devices_mac ON discovered_devices(mac_address);
CREATE INDEX idx_asset_warranties_end_date ON asset_warranties(end_date);
CREATE INDEX idx_asset_lifecycle_events_asset_id ON asset_lifecycle_events(asset_id);
CREATE INDEX idx_asset_lifecycle_events_event_type ON asset_lifecycle_events(event_type);
CREATE INDEX idx_asset_dependencies_parent ON asset_dependencies(parent_asset_id);
CREATE INDEX idx_asset_dependencies_child ON asset_dependencies(child_asset_id);
CREATE INDEX idx_asset_labels_code ON asset_labels(label_code);
CREATE INDEX idx_asset_software_asset_id ON asset_software(asset_id);
CREATE INDEX idx_asset_performance_metrics_asset_id ON asset_performance_metrics(asset_id);
CREATE INDEX idx_asset_performance_metrics_recorded ON asset_performance_metrics(recorded_at);

-- Function to calculate asset health score
CREATE OR REPLACE FUNCTION calculate_asset_health_score(asset_uuid UUID)
RETURNS INTEGER AS $$
DECLARE
    health_score INTEGER := 100;
    warranty_status RECORD;
    recent_issues INTEGER;
    performance_issues INTEGER;
    age_factor DECIMAL;
BEGIN
    -- Check warranty status (-20 if expired)
    SELECT * INTO warranty_status FROM asset_warranties 
    WHERE asset_id = asset_uuid AND end_date > NOW() 
    ORDER BY end_date DESC LIMIT 1;
    
    IF warranty_status IS NULL THEN
        health_score := health_score - 20;
    END IF;
    
    -- Check recent tickets (-5 per ticket in last 30 days, max -25)
    SELECT COUNT(*) INTO recent_issues FROM tickets 
    WHERE asset_id = asset_uuid 
    AND created_at > NOW() - INTERVAL '30 days';
    
    health_score := health_score - LEAST(recent_issues * 5, 25);
    
    -- Check performance metrics (-10 per critical metric)
    SELECT COUNT(*) INTO performance_issues FROM asset_performance_metrics
    WHERE asset_id = asset_uuid 
    AND is_healthy = false
    AND recorded_at > NOW() - INTERVAL '24 hours';
    
    health_score := health_score - (performance_issues * 10);
    
    -- Age factor (assuming 5 year typical lifespan)
    SELECT EXTRACT(YEAR FROM AGE(NOW(), purchase_date)) INTO age_factor 
    FROM assets WHERE id = asset_uuid;
    
    IF age_factor > 5 THEN
        health_score := health_score - ((age_factor - 5) * 10);
    END IF;
    
    RETURN GREATEST(health_score, 0);
END;
$$ LANGUAGE plpgsql;

-- Insert common manufacturers
INSERT INTO asset_manufacturers (name, website) VALUES
('Dell', 'https://www.dell.com'),
('HP', 'https://www.hp.com'),
('Lenovo', 'https://www.lenovo.com'),
('Microsoft', 'https://www.microsoft.com'),
('Apple', 'https://www.apple.com'),
('Cisco', 'https://www.cisco.com'),
('Fortinet', 'https://www.fortinet.com'),
('Ubiquiti', 'https://www.ui.com'),
('Synology', 'https://www.synology.com'),
('APC', 'https://www.apc.com');