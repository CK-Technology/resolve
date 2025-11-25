-- Network Integrations for UniFi, FortiGate, PowerDNS, Cloudflare

CREATE TABLE network_integrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    integration_type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    endpoint_url TEXT NOT NULL,
    api_version VARCHAR(20),
    credentials_encrypted TEXT NOT NULL,
    site_id VARCHAR(100),
    organization_id VARCHAR(100),
    enabled BOOLEAN DEFAULT true,
    auto_sync BOOLEAN DEFAULT true,
    sync_interval_minutes INTEGER DEFAULT 60,
    last_sync TIMESTAMPTZ,
    last_sync_status VARCHAR(50) DEFAULT 'pending',
    last_error TEXT,
    sync_statistics JSONB DEFAULT '{}',
    configuration JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

-- UniFi devices
CREATE TABLE unifi_devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES network_integrations(id) ON DELETE CASCADE,
    asset_id UUID REFERENCES assets(id) ON DELETE SET NULL,
    device_id VARCHAR(100) NOT NULL,
    mac_address MACADDR NOT NULL,
    name VARCHAR(255),
    model VARCHAR(100),
    device_type VARCHAR(50),
    version VARCHAR(50),
    ip_address INET,
    status VARCHAR(50),
    uptime_seconds BIGINT DEFAULT 0,
    cpu_utilization DECIMAL(5,2),
    memory_utilization DECIMAL(5,2),
    clients_connected INTEGER DEFAULT 0,
    device_configuration JSONB DEFAULT '{}',
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(integration_id, device_id)
);

-- FortiGate policies
CREATE TABLE fortigate_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES network_integrations(id) ON DELETE CASCADE,
    policy_id INTEGER NOT NULL,
    name VARCHAR(255),
    source_zones TEXT[] DEFAULT '{}',
    destination_zones TEXT[] DEFAULT '{}',
    action VARCHAR(20),
    status VARCHAR(20),
    hit_count BIGINT DEFAULT 0,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(integration_id, policy_id)
);

-- PowerDNS zones
CREATE TABLE powerdns_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES network_integrations(id) ON DELETE CASCADE,
    zone_id VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    zone_type VARCHAR(20),
    serial_number BIGINT,
    dnssec_enabled BOOLEAN DEFAULT false,
    records_count INTEGER DEFAULT 0,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(integration_id, zone_id)
);

-- Cloudflare zones
CREATE TABLE cloudflare_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES network_integrations(id) ON DELETE CASCADE,
    zone_id VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50),
    plan_type VARCHAR(50),
    name_servers TEXT[] DEFAULT '{}',
    dns_records_count INTEGER DEFAULT 0,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(integration_id, zone_id)
);

-- Indexes
CREATE INDEX idx_network_integrations_client_type ON network_integrations(client_id, integration_type);
CREATE INDEX idx_unifi_devices_integration ON unifi_devices(integration_id);
CREATE INDEX idx_fortigate_policies_integration ON fortigate_policies(integration_id);
CREATE INDEX idx_powerdns_zones_integration ON powerdns_zones(integration_id);
CREATE INDEX idx_cloudflare_zones_integration ON cloudflare_zones(integration_id);