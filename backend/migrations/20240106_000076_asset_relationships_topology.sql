-- Advanced Asset Relationships and Rack/Physical Topology Management
-- Implements physical topology mapping, rack management, and IP address management

-- Physical locations and racks
CREATE TABLE locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    location_type VARCHAR(50) DEFAULT 'office', -- office, datacenter, remote, cloud
    address TEXT,
    city VARCHAR(100),
    state VARCHAR(50),
    country VARCHAR(100),
    postal_code VARCHAR(20),
    timezone VARCHAR(50) DEFAULT 'UTC',
    coordinates POINT, -- lat,lng coordinates
    floor VARCHAR(50),
    room VARCHAR(50),
    notes TEXT,
    is_primary BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_locations_client_id ON locations(client_id);
CREATE INDEX idx_locations_coordinates ON locations USING GIST (coordinates) WHERE coordinates IS NOT NULL;

-- Equipment racks
CREATE TABLE equipment_racks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    location_id UUID NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    rack_units INTEGER NOT NULL DEFAULT 42, -- standard rack height
    width_inches INTEGER DEFAULT 19, -- standard rack width
    depth_inches INTEGER DEFAULT 36,
    power_capacity_watts INTEGER,
    power_used_watts INTEGER DEFAULT 0,
    cooling_capacity_btu INTEGER,
    weight_capacity_lbs INTEGER,
    weight_used_lbs INTEGER DEFAULT 0,
    rack_type VARCHAR(50) DEFAULT 'standard', -- standard, wall_mount, blade_enclosure
    manufacturer VARCHAR(100),
    model VARCHAR(100),
    serial_number VARCHAR(100),
    asset_tag VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_equipment_racks_location_id ON equipment_racks(location_id);

-- Asset positions in racks
CREATE TABLE asset_rack_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    rack_id UUID NOT NULL REFERENCES equipment_racks(id) ON DELETE CASCADE,
    start_unit INTEGER NOT NULL, -- starting rack unit (1-based)
    unit_height INTEGER NOT NULL DEFAULT 1, -- how many U the device occupies
    position VARCHAR(20) DEFAULT 'front', -- front, rear, both
    power_consumption_watts INTEGER,
    weight_lbs INTEGER,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(rack_id, start_unit, position),
    CHECK (start_unit > 0),
    CHECK (unit_height > 0)
);

CREATE INDEX idx_asset_rack_positions_asset_id ON asset_rack_positions(asset_id);
CREATE INDEX idx_asset_rack_positions_rack_id ON asset_rack_positions(rack_id);

-- Network subnets and IP address management
CREATE TABLE network_subnets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    location_id UUID REFERENCES locations(id),
    name VARCHAR(100) NOT NULL,
    subnet_cidr INET NOT NULL, -- PostgreSQL INET type for CIDR notation
    network_type VARCHAR(50) DEFAULT 'lan', -- lan, wan, dmz, management, guest
    vlan_id INTEGER,
    gateway_ip INET,
    dhcp_enabled BOOLEAN DEFAULT false,
    dhcp_start_ip INET,
    dhcp_end_ip INET,
    dns_servers INET[],
    description TEXT,
    monitoring_enabled BOOLEAN DEFAULT true,
    discovery_enabled BOOLEAN DEFAULT true,
    last_scanned TIMESTAMPTZ,
    utilization_percentage DECIMAL(5,2),
    status VARCHAR(20) DEFAULT 'active', -- active, inactive, planned, decommissioned
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_network_subnets_client_id ON network_subnets(client_id);
CREATE INDEX idx_network_subnets_subnet_cidr ON network_subnets USING GIST (subnet_cidr inet_ops);
CREATE INDEX idx_network_subnets_location_id ON network_subnets(location_id);

-- IP address assignments
CREATE TABLE ip_address_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subnet_id UUID NOT NULL REFERENCES network_subnets(id) ON DELETE CASCADE,
    asset_id UUID REFERENCES assets(id) ON DELETE SET NULL,
    ip_address INET NOT NULL,
    mac_address MACADDR,
    hostname VARCHAR(253),
    assignment_type VARCHAR(50) DEFAULT 'static', -- static, dhcp, reserved
    status VARCHAR(20) DEFAULT 'active', -- active, inactive, conflict, unknown
    first_seen TIMESTAMPTZ DEFAULT NOW(),
    last_seen TIMESTAMPTZ DEFAULT NOW(),
    lease_expiry TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(subnet_id, ip_address)
);

CREATE INDEX idx_ip_address_assignments_subnet_id ON ip_address_assignments(subnet_id);
CREATE INDEX idx_ip_address_assignments_asset_id ON ip_address_assignments(asset_id);
CREATE INDEX idx_ip_address_assignments_ip_address ON ip_address_assignments(ip_address);
CREATE INDEX idx_ip_address_assignments_mac_address ON ip_address_assignments(mac_address);

-- Asset relationships (depends_on, connects_to, hosts, manages, monitors)
CREATE TABLE asset_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    child_asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    relationship_type VARCHAR(50) NOT NULL,
    -- depends_on: child depends on parent (server depends on UPS)
    -- connects_to: physical or logical connection (switch connects to router)
    -- hosts: parent hosts child (hypervisor hosts VM)
    -- manages: parent manages child (management server manages switches)
    -- monitors: parent monitors child (monitoring system monitors servers)
    -- powers: parent powers child (UPS powers server)
    -- cools: parent cools child (AC unit cools rack)
    connection_details JSONB, -- port numbers, interface details, etc.
    bandwidth_mbps INTEGER, -- for network connections
    is_critical BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CHECK (parent_asset_id != child_asset_id),
    UNIQUE(parent_asset_id, child_asset_id, relationship_type)
);

CREATE INDEX idx_asset_relationships_parent ON asset_relationships(parent_asset_id);
CREATE INDEX idx_asset_relationships_child ON asset_relationships(child_asset_id);
CREATE INDEX idx_asset_relationships_type ON asset_relationships(relationship_type);

-- Network topology discovery and mapping
CREATE TABLE network_discovery_scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    scan_type VARCHAR(50) NOT NULL, -- ping_sweep, port_scan, snmp_walk, arp_scan
    target_subnets INET[],
    scan_status VARCHAR(20) DEFAULT 'queued', -- queued, running, completed, failed
    devices_discovered INTEGER DEFAULT 0,
    new_devices INTEGER DEFAULT 0,
    updated_devices INTEGER DEFAULT 0,
    scan_duration_seconds INTEGER,
    error_message TEXT,
    scan_results JSONB,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_network_discovery_scans_client_id ON network_discovery_scans(client_id);
CREATE INDEX idx_network_discovery_scans_status ON network_discovery_scans(scan_status);

-- Physical connections between assets (cables, fiber, etc.)
CREATE TABLE physical_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    to_asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    from_port VARCHAR(100), -- port/interface name on source asset
    to_port VARCHAR(100), -- port/interface name on destination asset
    connection_type VARCHAR(50) NOT NULL, -- ethernet, fiber, power, serial, usb
    cable_type VARCHAR(100), -- cat6, fiber_om3, power_c13, etc.
    cable_length_ft DECIMAL(6,2),
    cable_color VARCHAR(50),
    cable_label VARCHAR(100),
    speed_mbps INTEGER, -- for network connections
    duplex VARCHAR(20), -- full, half, auto
    status VARCHAR(20) DEFAULT 'active', -- active, inactive, disconnected, faulty
    last_verified TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CHECK (from_asset_id != to_asset_id)
);

CREATE INDEX idx_physical_connections_from_asset ON physical_connections(from_asset_id);
CREATE INDEX idx_physical_connections_to_asset ON physical_connections(to_asset_id);
CREATE INDEX idx_physical_connections_type ON physical_connections(connection_type);

-- Asset monitoring and status
CREATE TABLE asset_monitoring_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    monitor_type VARCHAR(50) NOT NULL, -- ping, snmp, http, ssh, custom
    status VARCHAR(20) NOT NULL, -- up, down, warning, critical, unknown
    response_time_ms INTEGER,
    last_check TIMESTAMPTZ DEFAULT NOW(),
    next_check TIMESTAMPTZ,
    check_interval_minutes INTEGER DEFAULT 5,
    consecutive_failures INTEGER DEFAULT 0,
    uptime_percentage DECIMAL(5,2),
    monitoring_enabled BOOLEAN DEFAULT true,
    alert_threshold INTEGER DEFAULT 3, -- failures before alert
    status_details JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(asset_id, monitor_type)
);

CREATE INDEX idx_asset_monitoring_status_asset_id ON asset_monitoring_status(asset_id);
CREATE INDEX idx_asset_monitoring_status_status ON asset_monitoring_status(status);
CREATE INDEX idx_asset_monitoring_status_next_check ON asset_monitoring_status(next_check);

-- Add location reference to existing assets table (if not already present)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'assets' AND column_name = 'location_id') THEN
        ALTER TABLE assets ADD COLUMN location_id UUID REFERENCES locations(id);
        CREATE INDEX idx_assets_location_id ON assets(location_id);
    END IF;
END $$;

-- Update existing assets table with rack position reference
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'assets' AND column_name = 'rack_position_id') THEN
        ALTER TABLE assets ADD COLUMN rack_position_id UUID REFERENCES asset_rack_positions(id);
        CREATE INDEX idx_assets_rack_position_id ON assets(rack_position_id);
    END IF;
END $$;

-- Function to calculate subnet utilization
CREATE OR REPLACE FUNCTION calculate_subnet_utilization(subnet_id_param UUID)
RETURNS DECIMAL(5,2) AS $$
DECLARE
    total_addresses INTEGER;
    used_addresses INTEGER;
    subnet_cidr INET;
    utilization DECIMAL(5,2);
BEGIN
    -- Get subnet CIDR
    SELECT ns.subnet_cidr INTO subnet_cidr 
    FROM network_subnets ns 
    WHERE ns.id = subnet_id_param;
    
    IF subnet_cidr IS NULL THEN
        RETURN NULL;
    END IF;
    
    -- Calculate total addresses in subnet (excluding network and broadcast)
    total_addresses := (2 ^ (32 - masklen(subnet_cidr))) - 2;
    
    -- Count used addresses
    SELECT COUNT(*) INTO used_addresses
    FROM ip_address_assignments
    WHERE subnet_id = subnet_id_param AND status = 'active';
    
    -- Calculate utilization percentage
    IF total_addresses > 0 THEN
        utilization := (used_addresses::DECIMAL / total_addresses::DECIMAL) * 100;
    ELSE
        utilization := 0;
    END IF;
    
    -- Update the subnet utilization
    UPDATE network_subnets 
    SET utilization_percentage = utilization, updated_at = NOW()
    WHERE id = subnet_id_param;
    
    RETURN utilization;
END;
$$ LANGUAGE plpgsql;

-- Function to get available IP addresses in a subnet
CREATE OR REPLACE FUNCTION get_available_ips(subnet_id_param UUID, limit_count INTEGER DEFAULT 10)
RETURNS TABLE(ip_address INET) AS $$
BEGIN
    RETURN QUERY
    WITH subnet_info AS (
        SELECT subnet_cidr FROM network_subnets WHERE id = subnet_id_param
    ),
    all_ips AS (
        SELECT host(network(si.subnet_cidr) + generate_series(1, (2^(32-masklen(si.subnet_cidr)))-2))::INET as ip
        FROM subnet_info si
    ),
    used_ips AS (
        SELECT ip.ip_address 
        FROM ip_address_assignments ip 
        WHERE ip.subnet_id = subnet_id_param
    )
    SELECT ai.ip
    FROM all_ips ai
    LEFT JOIN used_ips ui ON ai.ip = ui.ip_address
    WHERE ui.ip_address IS NULL
    ORDER BY ai.ip
    LIMIT limit_count;
END;
$$ LANGUAGE plpgsql;