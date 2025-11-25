-- Network Documentation Features
-- Wi-Fi profile management, VLAN/subnet documentation, network topology mapping

-- Wi-Fi profiles and management
CREATE TABLE wifi_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    location_id UUID REFERENCES locations(id),
    profile_name VARCHAR(100) NOT NULL,
    ssid VARCHAR(100) NOT NULL,
    bssid MACADDR,
    security_type VARCHAR(50) NOT NULL, -- open, wep, wpa, wpa2, wpa3, enterprise
    authentication VARCHAR(50), -- psk, eap, certificate
    encryption VARCHAR(50), -- none, wep, tkip, aes, mixed
    passphrase_encrypted TEXT, -- encrypted Wi-Fi password
    eap_method VARCHAR(50), -- peap, ttls, tls, etc.
    eap_identity VARCHAR(255),
    eap_password_encrypted TEXT,
    certificate_id UUID, -- link to certificates table
    frequency_band VARCHAR(20), -- 2.4GHz, 5GHz, 6GHz, dual, tri
    channel INTEGER,
    channel_width INTEGER, -- 20, 40, 80, 160 MHz
    hidden BOOLEAN DEFAULT false,
    guest_network BOOLEAN DEFAULT false,
    captive_portal BOOLEAN DEFAULT false,
    bandwidth_limit_mbps INTEGER,
    device_limit INTEGER,
    vlan_id INTEGER,
    priority INTEGER DEFAULT 0, -- QoS priority
    auto_connect BOOLEAN DEFAULT true,
    proxy_config JSONB, -- proxy configuration
    dns_servers INET[],
    static_ip_config JSONB, -- {"ip": "192.168.1.100", "subnet": "255.255.255.0", "gateway": "192.168.1.1"}
    deployment_status VARCHAR(20) DEFAULT 'active', -- active, inactive, planned, deprecated
    access_points UUID[], -- array of asset IDs for APs broadcasting this profile
    connected_devices INTEGER DEFAULT 0,
    max_devices_seen INTEGER DEFAULT 0,
    last_seen_active TIMESTAMPTZ,
    signal_strength_dbm INTEGER,
    throughput_mbps DECIMAL(8,2),
    notes TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_wifi_profiles_client_id ON wifi_profiles(client_id);
CREATE INDEX idx_wifi_profiles_location_id ON wifi_profiles(location_id);
CREATE INDEX idx_wifi_profiles_ssid ON wifi_profiles(ssid);
CREATE INDEX idx_wifi_profiles_status ON wifi_profiles(deployment_status);

-- VLAN documentation
CREATE TABLE vlans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    location_id UUID REFERENCES locations(id),
    vlan_id INTEGER NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    subnet_id UUID REFERENCES network_subnets(id),
    purpose VARCHAR(50), -- management, users, guests, iot, security, voice, video
    security_level VARCHAR(20) DEFAULT 'standard', -- low, standard, high, critical
    inter_vlan_routing BOOLEAN DEFAULT true,
    firewall_rules JSONB, -- array of firewall rules
    qos_policy VARCHAR(50), -- voice, video, data, best_effort
    bandwidth_limit_mbps INTEGER,
    switch_ports JSONB, -- {"switch_id": "uuid", "ports": [1, 2, 3]}
    tagged_switches UUID[], -- asset IDs of switches with tagged ports
    untagged_switches UUID[], -- asset IDs of switches with untagged ports
    dhcp_enabled BOOLEAN DEFAULT false,
    dhcp_server_id UUID REFERENCES assets(id),
    dns_servers INET[],
    default_gateway INET,
    monitoring_enabled BOOLEAN DEFAULT true,
    stp_priority INTEGER,
    vtp_domain VARCHAR(100),
    is_native BOOLEAN DEFAULT false,
    trunk_ports JSONB, -- trunk port configurations
    access_control_list JSONB, -- ACL rules
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(client_id, location_id, vlan_id)
);

CREATE INDEX idx_vlans_client_id ON vlans(client_id);
CREATE INDEX idx_vlans_location_id ON vlans(location_id);
CREATE INDEX idx_vlans_vlan_id ON vlans(vlan_id);
CREATE INDEX idx_vlans_subnet_id ON vlans(subnet_id);

-- Network diagrams and topology maps
CREATE TABLE network_diagrams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    location_id UUID REFERENCES locations(id),
    name VARCHAR(100) NOT NULL,
    diagram_type VARCHAR(50) DEFAULT 'logical', -- logical, physical, rack, cable
    description TEXT,
    diagram_data JSONB NOT NULL, -- JSON representation of network diagram
    diagram_format VARCHAR(20) DEFAULT 'json', -- json, visio, drawio, lucidchart
    auto_generated BOOLEAN DEFAULT false,
    last_discovery_scan TIMESTAMPTZ,
    visibility VARCHAR(20) DEFAULT 'private', -- private, team, client
    version INTEGER DEFAULT 1,
    parent_diagram_id UUID REFERENCES network_diagrams(id),
    is_template BOOLEAN DEFAULT false,
    template_category VARCHAR(50), -- small_office, datacenter, campus, etc.
    zoom_level DECIMAL(4,2) DEFAULT 1.0,
    canvas_size JSONB, -- {"width": 1920, "height": 1080}
    grid_settings JSONB,
    layer_visibility JSONB, -- which diagram layers are visible
    export_formats TEXT[], -- pdf, png, svg, vsd
    shared_link_token VARCHAR(255) UNIQUE,
    shared_link_expires TIMESTAMPTZ,
    view_count INTEGER DEFAULT 0,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_network_diagrams_client_id ON network_diagrams(client_id);
CREATE INDEX idx_network_diagrams_location_id ON network_diagrams(location_id);
CREATE INDEX idx_network_diagrams_type ON network_diagrams(diagram_type);
CREATE INDEX idx_network_diagrams_template ON network_diagrams(is_template) WHERE is_template = true;

-- Network device configurations
CREATE TABLE device_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    config_type VARCHAR(50) NOT NULL, -- running, startup, backup, template
    config_name VARCHAR(100) NOT NULL,
    config_content TEXT NOT NULL,
    config_hash VARCHAR(64), -- SHA-256 hash for change detection
    config_format VARCHAR(20) DEFAULT 'text', -- text, json, xml, yaml
    vendor VARCHAR(50), -- cisco, juniper, hp, ubiquiti, etc.
    os_version VARCHAR(100),
    firmware_version VARCHAR(100),
    feature_set VARCHAR(100),
    backup_method VARCHAR(50), -- tftp, scp, snmp, api, ssh
    backup_location TEXT,
    is_encrypted BOOLEAN DEFAULT false,
    encryption_method VARCHAR(50),
    config_size_bytes INTEGER,
    change_count INTEGER DEFAULT 0,
    last_changed TIMESTAMPTZ,
    change_description TEXT,
    auto_backup_enabled BOOLEAN DEFAULT true,
    backup_schedule VARCHAR(50), -- daily, weekly, monthly, on_change
    next_backup TIMESTAMPTZ,
    retention_days INTEGER DEFAULT 90,
    compliance_status VARCHAR(20), -- compliant, non_compliant, unknown
    compliance_policies UUID[], -- array of policy IDs
    validation_errors JSONB,
    archived BOOLEAN DEFAULT false,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_device_configurations_asset_id ON device_configurations(asset_id);
CREATE INDEX idx_device_configurations_type ON device_configurations(config_type);
CREATE INDEX idx_device_configurations_hash ON device_configurations(config_hash);
CREATE INDEX idx_device_configurations_changed ON device_configurations(last_changed);

-- Network monitoring rules and thresholds
CREATE TABLE network_monitoring_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    client_id UUID REFERENCES clients(id),
    rule_type VARCHAR(50) NOT NULL, -- ping, snmp, bandwidth, device_count, port_status
    targets JSONB NOT NULL, -- array of target assets, subnets, or IPs
    check_interval_minutes INTEGER DEFAULT 5,
    timeout_seconds INTEGER DEFAULT 30,
    retries INTEGER DEFAULT 3,
    thresholds JSONB NOT NULL, -- {"warning": 80, "critical": 95} or specific values
    monitoring_enabled BOOLEAN DEFAULT true,
    alert_enabled BOOLEAN DEFAULT true,
    notification_channels JSONB, -- {"email": ["admin@company.com"], "slack": "#alerts"}
    escalation_policy_id UUID,
    business_hours_only BOOLEAN DEFAULT false,
    maintenance_windows JSONB, -- scheduled maintenance periods
    snmp_community VARCHAR(100),
    snmp_version VARCHAR(10) DEFAULT 'v2c',
    snmp_oids JSONB, -- array of OIDs to monitor
    custom_script_path TEXT,
    expected_response TEXT,
    response_regex VARCHAR(500),
    dependency_rules JSONB, -- dependencies that affect this rule
    suppress_alerts_until TIMESTAMPTZ,
    consecutive_failures_threshold INTEGER DEFAULT 3,
    is_flapping BOOLEAN DEFAULT false,
    flap_detection_window_minutes INTEGER DEFAULT 30,
    last_check TIMESTAMPTZ,
    last_status VARCHAR(20) DEFAULT 'unknown',
    last_alert TIMESTAMPTZ,
    alert_count INTEGER DEFAULT 0,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_network_monitoring_rules_client_id ON network_monitoring_rules(client_id);
CREATE INDEX idx_network_monitoring_rules_enabled ON network_monitoring_rules(monitoring_enabled);
CREATE INDEX idx_network_monitoring_rules_next_check ON network_monitoring_rules(last_check);

-- Network documentation templates
CREATE TABLE network_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    template_type VARCHAR(50) NOT NULL, -- wifi_profile, vlan_config, firewall_rule, switch_config
    category VARCHAR(50), -- small_business, enterprise, datacenter, campus
    vendor VARCHAR(50), -- cisco, ubiquiti, meraki, etc.
    description TEXT,
    template_content JSONB NOT NULL,
    default_values JSONB,
    required_fields TEXT[],
    validation_schema JSONB,
    usage_count INTEGER DEFAULT 0,
    is_public BOOLEAN DEFAULT false,
    is_verified BOOLEAN DEFAULT false,
    verified_by UUID REFERENCES users(id),
    tags TEXT[],
    version VARCHAR(20) DEFAULT '1.0',
    changelog TEXT,
    documentation_url VARCHAR(500),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_network_templates_type ON network_templates(template_type);
CREATE INDEX idx_network_templates_category ON network_templates(category);
CREATE INDEX idx_network_templates_vendor ON network_templates(vendor);
CREATE INDEX idx_network_templates_public ON network_templates(is_public) WHERE is_public = true;

-- Cable management and documentation
CREATE TABLE network_cables (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    location_id UUID REFERENCES locations(id),
    cable_label VARCHAR(100),
    cable_type VARCHAR(50) NOT NULL, -- cat5e, cat6, cat6a, fiber_sm, fiber_mm, coax, power
    cable_category VARCHAR(20), -- horizontal, backbone, patch, crossover
    length_feet DECIMAL(6,2),
    color VARCHAR(50),
    from_location TEXT, -- human readable location
    to_location TEXT,
    from_asset_id UUID REFERENCES assets(id),
    to_asset_id UUID REFERENCES assets(id),
    from_port VARCHAR(50),
    to_port VARCHAR(50),
    from_panel_position VARCHAR(50),
    to_panel_position VARCHAR(50),
    installation_date DATE,
    installer_name VARCHAR(100),
    test_results JSONB, -- cable testing results
    certification_level VARCHAR(50), -- cat5e, cat6, etc.
    bend_radius_compliance BOOLEAN,
    jacket_rating VARCHAR(20), -- plenum, riser, pvc
    fire_rating VARCHAR(20),
    bandwidth_rating_mhz INTEGER,
    attenuation_db DECIMAL(5,2),
    crosstalk_db DECIMAL(5,2),
    impedance_ohms INTEGER,
    status VARCHAR(20) DEFAULT 'active', -- active, inactive, damaged, testing
    maintenance_schedule VARCHAR(50),
    last_tested TIMESTAMPTZ,
    next_test_due TIMESTAMPTZ,
    warranty_expires TIMESTAMPTZ,
    purchase_info JSONB, -- vendor, cost, purchase_date
    notes TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_network_cables_client_id ON network_cables(client_id);
CREATE INDEX idx_network_cables_location_id ON network_cables(location_id);
CREATE INDEX idx_network_cables_from_asset ON network_cables(from_asset_id);
CREATE INDEX idx_network_cables_to_asset ON network_cables(to_asset_id);
CREATE INDEX idx_network_cables_type ON network_cables(cable_type);
CREATE INDEX idx_network_cables_status ON network_cables(status);

-- Network discovery and scanning results
CREATE TABLE network_scan_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES network_discovery_scans(id) ON DELETE CASCADE,
    ip_address INET NOT NULL,
    mac_address MACADDR,
    hostname VARCHAR(253),
    device_type VARCHAR(50), -- router, switch, server, workstation, printer, iot, unknown
    vendor VARCHAR(100),
    device_model VARCHAR(100),
    os_detection VARCHAR(200),
    open_ports INTEGER[],
    services_detected JSONB,
    snmp_details JSONB,
    response_time_ms INTEGER,
    is_new_device BOOLEAN DEFAULT false,
    confidence_score INTEGER, -- 0-100 confidence in device detection
    geolocation JSONB, -- if available from IP geolocation
    vulnerability_scan_id UUID,
    security_flags JSONB, -- security-related findings
    asset_matched UUID REFERENCES assets(id), -- if matched to existing asset
    requires_investigation BOOLEAN DEFAULT false,
    investigation_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_network_scan_results_scan_id ON network_scan_results(scan_id);
CREATE INDEX idx_network_scan_results_ip ON network_scan_results(ip_address);
CREATE INDEX idx_network_scan_results_mac ON network_scan_results(mac_address);
CREATE INDEX idx_network_scan_results_new_device ON network_scan_results(is_new_device) WHERE is_new_device = true;

-- Insert default Wi-Fi security templates
INSERT INTO network_templates (name, template_type, category, description, template_content, default_values)
VALUES
('WPA3 Personal', 'wifi_profile', 'small_business', 'Secure WPA3 Personal configuration', 
 '{"security_type": "wpa3", "authentication": "psk", "encryption": "aes"}',
 '{"frequency_band": "dual", "channel_width": 80, "hidden": false}'),
 
('WPA2/WPA3 Mixed Enterprise', 'wifi_profile', 'enterprise', 'Mixed mode enterprise Wi-Fi with RADIUS',
 '{"security_type": "wpa3", "authentication": "eap", "encryption": "aes", "eap_method": "peap"}',
 '{"frequency_band": "dual", "channel_width": 80, "guest_network": false}'),
 
('Guest Network Template', 'wifi_profile', 'small_business', 'Isolated guest network configuration',
 '{"security_type": "wpa2", "authentication": "psk", "encryption": "aes", "guest_network": true}',
 '{"bandwidth_limit_mbps": 50, "device_limit": 20, "captive_portal": true}');

-- Insert default VLAN templates
INSERT INTO network_templates (name, template_type, category, description, template_content, default_values)
VALUES
('Management VLAN', 'vlan_config', 'enterprise', 'Network management VLAN template',
 '{"purpose": "management", "security_level": "high", "inter_vlan_routing": false}',
 '{"vlan_id": 10, "dhcp_enabled": false, "monitoring_enabled": true}'),
 
('User Data VLAN', 'vlan_config', 'enterprise', 'Standard user data VLAN',
 '{"purpose": "users", "security_level": "standard", "inter_vlan_routing": true}',
 '{"vlan_id": 100, "dhcp_enabled": true, "qos_policy": "data"}'),
 
('Voice VLAN', 'vlan_config', 'enterprise', 'Voice/VoIP VLAN with QoS priority',
 '{"purpose": "voice", "security_level": "standard", "qos_policy": "voice"}',
 '{"vlan_id": 200, "dhcp_enabled": true, "bandwidth_limit_mbps": 1000}'),
 
('IoT Device VLAN', 'vlan_config', 'enterprise', 'Isolated IoT device VLAN',
 '{"purpose": "iot", "security_level": "high", "inter_vlan_routing": false}',
 '{"vlan_id": 300, "dhcp_enabled": true, "monitoring_enabled": true}');

-- Function to calculate network utilization across all subnets
CREATE OR REPLACE FUNCTION get_network_utilization_summary(client_id_param UUID)
RETURNS TABLE(
    total_subnets INTEGER,
    total_addresses INTEGER,
    used_addresses INTEGER,
    available_addresses INTEGER,
    avg_utilization DECIMAL(5,2),
    critical_subnets INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COUNT(*)::INTEGER as total_subnets,
        SUM((2^(32-masklen(subnet_cidr)))-2)::INTEGER as total_addresses,
        SUM(COALESCE(
            (SELECT COUNT(*) FROM ip_address_assignments WHERE subnet_id = ns.id AND status = 'active'), 
            0
        ))::INTEGER as used_addresses,
        (SUM((2^(32-masklen(subnet_cidr)))-2) - SUM(COALESCE(
            (SELECT COUNT(*) FROM ip_address_assignments WHERE subnet_id = ns.id AND status = 'active'), 
            0
        )))::INTEGER as available_addresses,
        AVG(COALESCE(utilization_percentage, 0)) as avg_utilization,
        COUNT(CASE WHEN COALESCE(utilization_percentage, 0) > 90 THEN 1 END)::INTEGER as critical_subnets
    FROM network_subnets ns
    WHERE ns.client_id = client_id_param AND ns.status = 'active';
END;
$$ LANGUAGE plpgsql;