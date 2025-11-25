-- Azure Resource Monitoring Integration
-- Comprehensive Azure subscription, resource group, and resource monitoring

-- Azure Subscriptions
CREATE TABLE azure_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    subscription_id VARCHAR(255) NOT NULL,
    subscription_name VARCHAR(255) NOT NULL,
    tenant_id VARCHAR(255) NOT NULL,
    
    -- Subscription Details
    state VARCHAR(50), -- enabled, disabled, warned, past_due, suspended
    subscription_policies JSONB DEFAULT '{}',
    spending_limit VARCHAR(20), -- on, off, current_period_off
    authorization_source VARCHAR(100),
    
    -- Authentication
    client_id_encrypted TEXT NOT NULL, -- Service Principal Application ID
    client_secret_encrypted TEXT NOT NULL, -- Service Principal Secret
    tenant_id_encrypted TEXT NOT NULL, -- Azure Tenant ID
    
    -- Cost Management
    current_spend_usd DECIMAL(15,2) DEFAULT 0,
    budget_limit_usd DECIMAL(15,2),
    budget_alerts_enabled BOOLEAN DEFAULT true,
    cost_alerts JSONB DEFAULT '[]',
    
    -- Quota Information
    quota_limits JSONB DEFAULT '{}',
    quota_usage JSONB DEFAULT '{}',
    
    -- Sync Settings
    sync_enabled BOOLEAN DEFAULT true,
    sync_interval_hours INTEGER DEFAULT 4,
    last_sync TIMESTAMPTZ,
    last_sync_status VARCHAR(50) DEFAULT 'pending',
    last_error TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    UNIQUE(subscription_id)
);

-- Azure Resource Groups
CREATE TABLE azure_resource_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES azure_subscriptions(id) ON DELETE CASCADE,
    resource_group_id VARCHAR(500) NOT NULL,
    name VARCHAR(255) NOT NULL,
    location VARCHAR(100) NOT NULL,
    
    -- Resource Group Properties
    provisioning_state VARCHAR(50),
    managed_by VARCHAR(500),
    
    -- Resource Counts
    total_resources INTEGER DEFAULT 0,
    compute_resources INTEGER DEFAULT 0,
    storage_resources INTEGER DEFAULT 0,
    network_resources INTEGER DEFAULT 0,
    database_resources INTEGER DEFAULT 0,
    
    -- Tags and Metadata
    tags JSONB DEFAULT '{}',
    
    -- Cost Information
    monthly_cost_usd DECIMAL(12,2) DEFAULT 0,
    daily_cost_usd DECIMAL(10,2) DEFAULT 0,
    
    created_time TIMESTAMPTZ,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(subscription_id, resource_group_id)
);

-- Azure Resources (VMs, Storage, Databases, etc.)
CREATE TABLE azure_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES azure_subscriptions(id) ON DELETE CASCADE,
    resource_group_id UUID NOT NULL REFERENCES azure_resource_groups(id) ON DELETE CASCADE,
    resource_id VARCHAR(1000) NOT NULL,
    name VARCHAR(255) NOT NULL,
    
    -- Resource Type and Details
    resource_type VARCHAR(200) NOT NULL,
    kind VARCHAR(100),
    location VARCHAR(100) NOT NULL,
    sku JSONB,
    
    -- Status
    provisioning_state VARCHAR(50),
    power_state VARCHAR(50), -- for VMs: running, stopped, deallocated
    
    -- Configuration
    properties JSONB DEFAULT '{}',
    tags JSONB DEFAULT '{}',
    
    -- Compute Specific (VMs)
    vm_size VARCHAR(50),
    os_type VARCHAR(20), -- windows, linux
    os_disk_size_gb INTEGER,
    data_disk_count INTEGER DEFAULT 0,
    network_interfaces TEXT[] DEFAULT '{}',
    
    -- Storage Specific
    storage_type VARCHAR(50), -- standard, premium, ultra
    storage_size_gb BIGINT,
    storage_tier VARCHAR(50), -- hot, cool, archive
    
    -- Database Specific
    database_edition VARCHAR(50),
    service_tier VARCHAR(50),
    compute_tier VARCHAR(50),
    max_size_gb INTEGER,
    
    -- Network Specific
    ip_address INET,
    subnet_id VARCHAR(500),
    security_groups TEXT[] DEFAULT '{}',
    
    -- Monitoring & Performance
    cpu_utilization_avg DECIMAL(5,2),
    memory_utilization_avg DECIMAL(5,2),
    disk_iops_avg INTEGER,
    network_in_mb DECIMAL(10,2),
    network_out_mb DECIMAL(10,2),
    
    -- Cost Information
    daily_cost_usd DECIMAL(10,2) DEFAULT 0,
    monthly_cost_usd DECIMAL(12,2) DEFAULT 0,
    
    -- Backup & Security
    backup_enabled BOOLEAN DEFAULT false,
    backup_policy VARCHAR(255),
    encryption_enabled BOOLEAN DEFAULT false,
    
    created_time TIMESTAMPTZ,
    changed_time TIMESTAMPTZ,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(resource_id)
);

-- Azure Virtual Networks
CREATE TABLE azure_virtual_networks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES azure_subscriptions(id) ON DELETE CASCADE,
    resource_group_id UUID NOT NULL REFERENCES azure_resource_groups(id) ON DELETE CASCADE,
    vnet_id VARCHAR(500) NOT NULL,
    name VARCHAR(255) NOT NULL,
    location VARCHAR(100) NOT NULL,
    
    -- Network Configuration
    address_space TEXT[] DEFAULT '{}',
    dns_servers TEXT[] DEFAULT '{}',
    
    -- Subnets
    subnet_count INTEGER DEFAULT 0,
    subnets JSONB DEFAULT '[]',
    
    -- Peering
    peering_connections JSONB DEFAULT '[]',
    
    -- Security
    network_security_groups TEXT[] DEFAULT '{}',
    route_tables TEXT[] DEFAULT '{}',
    
    -- DDoS Protection
    ddos_protection_enabled BOOLEAN DEFAULT false,
    ddos_protection_plan VARCHAR(500),
    
    provisioning_state VARCHAR(50),
    tags JSONB DEFAULT '{}',
    
    created_time TIMESTAMPTZ,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(vnet_id)
);

-- Azure Storage Accounts
CREATE TABLE azure_storage_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES azure_subscriptions(id) ON DELETE CASCADE,
    resource_group_id UUID NOT NULL REFERENCES azure_resource_groups(id) ON DELETE CASCADE,
    storage_account_id VARCHAR(500) NOT NULL,
    storage_account_name VARCHAR(255) NOT NULL,
    location VARCHAR(100) NOT NULL,
    
    -- Account Properties
    account_type VARCHAR(50), -- storage, storageV2, blobStorage
    kind VARCHAR(50),
    sku_name VARCHAR(50), -- standard_lrs, standard_grs, premium_lrs
    sku_tier VARCHAR(20), -- standard, premium
    
    -- Access Configuration
    access_tier VARCHAR(20), -- hot, cool
    allow_blob_public_access BOOLEAN DEFAULT false,
    allow_shared_key_access BOOLEAN DEFAULT true,
    minimum_tls_version VARCHAR(10) DEFAULT 'TLS1_2',
    
    -- Encryption
    encryption_key_source VARCHAR(50), -- microsoft.storage, microsoft.keyvault
    encryption_services JSONB DEFAULT '{}',
    
    -- Network Access
    network_rule_set JSONB DEFAULT '{}',
    public_network_access VARCHAR(20) DEFAULT 'enabled',
    
    -- Usage Statistics
    blob_capacity_gb DECIMAL(15,2) DEFAULT 0,
    blob_count INTEGER DEFAULT 0,
    file_capacity_gb DECIMAL(15,2) DEFAULT 0,
    file_count INTEGER DEFAULT 0,
    queue_count INTEGER DEFAULT 0,
    table_count INTEGER DEFAULT 0,
    
    -- Costs
    monthly_cost_usd DECIMAL(12,2) DEFAULT 0,
    storage_cost_usd DECIMAL(10,2) DEFAULT 0,
    transaction_cost_usd DECIMAL(8,2) DEFAULT 0,
    
    provisioning_state VARCHAR(50),
    status_of_primary VARCHAR(50),
    status_of_secondary VARCHAR(50),
    tags JSONB DEFAULT '{}',
    
    created_time TIMESTAMPTZ,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(storage_account_id)
);

-- Azure SQL Databases
CREATE TABLE azure_sql_databases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES azure_subscriptions(id) ON DELETE CASCADE,
    resource_group_id UUID NOT NULL REFERENCES azure_resource_groups(id) ON DELETE CASCADE,
    database_id VARCHAR(500) NOT NULL,
    database_name VARCHAR(255) NOT NULL,
    server_name VARCHAR(255) NOT NULL,
    location VARCHAR(100) NOT NULL,
    
    -- Database Configuration
    edition VARCHAR(50), -- basic, standard, premium, general_purpose, business_critical
    service_level_objective VARCHAR(50),
    max_size_bytes BIGINT,
    current_size_bytes BIGINT,
    
    -- Performance Tier
    sku_name VARCHAR(50),
    sku_tier VARCHAR(50),
    sku_capacity INTEGER,
    
    -- Backup and Recovery
    backup_retention_days INTEGER DEFAULT 7,
    point_in_time_restore_enabled BOOLEAN DEFAULT true,
    long_term_retention_policy JSONB DEFAULT '{}',
    geo_backup_enabled BOOLEAN DEFAULT true,
    
    -- Security
    transparent_data_encryption_enabled BOOLEAN DEFAULT true,
    advanced_threat_protection_enabled BOOLEAN DEFAULT false,
    auditing_enabled BOOLEAN DEFAULT false,
    
    -- Performance Metrics
    cpu_utilization_avg DECIMAL(5,2),
    data_io_percentage_avg DECIMAL(5,2),
    log_io_percentage_avg DECIMAL(5,2),
    memory_usage_percentage_avg DECIMAL(5,2),
    
    -- DTU/vCore Information
    dtu_limit INTEGER,
    dtu_consumption_percent DECIMAL(5,2),
    
    -- Costs
    monthly_cost_usd DECIMAL(12,2) DEFAULT 0,
    
    status VARCHAR(50),
    collation VARCHAR(128),
    tags JSONB DEFAULT '{}',
    
    created_date TIMESTAMPTZ,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(database_id)
);

-- Azure Key Vaults
CREATE TABLE azure_key_vaults (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES azure_subscriptions(id) ON DELETE CASCADE,
    resource_group_id UUID NOT NULL REFERENCES azure_resource_groups(id) ON DELETE CASCADE,
    vault_id VARCHAR(500) NOT NULL,
    vault_name VARCHAR(255) NOT NULL,
    location VARCHAR(100) NOT NULL,
    
    -- Vault Properties
    vault_uri TEXT NOT NULL,
    sku_name VARCHAR(50), -- standard, premium
    tenant_id VARCHAR(255),
    
    -- Access Policies
    access_policies JSONB DEFAULT '[]',
    enabled_for_deployment BOOLEAN DEFAULT false,
    enabled_for_disk_encryption BOOLEAN DEFAULT false,
    enabled_for_template_deployment BOOLEAN DEFAULT false,
    
    -- Network Security
    network_acls JSONB DEFAULT '{}',
    public_network_access VARCHAR(20) DEFAULT 'enabled',
    
    -- Soft Delete and Purge Protection
    soft_delete_enabled BOOLEAN DEFAULT true,
    soft_delete_retention_days INTEGER DEFAULT 90,
    purge_protection_enabled BOOLEAN DEFAULT false,
    
    -- Content Counts
    key_count INTEGER DEFAULT 0,
    secret_count INTEGER DEFAULT 0,
    certificate_count INTEGER DEFAULT 0,
    
    -- HSM Information
    hsm_pool_resource_id VARCHAR(500),
    
    provisioning_state VARCHAR(50),
    tags JSONB DEFAULT '{}',
    
    created_time TIMESTAMPTZ,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(vault_id)
);

-- Azure Alerts and Monitoring
CREATE TABLE azure_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES azure_subscriptions(id) ON DELETE CASCADE,
    alert_id VARCHAR(255) NOT NULL,
    alert_name VARCHAR(255) NOT NULL,
    
    -- Alert Configuration
    alert_type VARCHAR(50), -- metric, log, activity_log
    severity INTEGER, -- 0=critical, 1=error, 2=warning, 3=informational, 4=verbose
    condition JSONB NOT NULL,
    target_resource_id VARCHAR(1000),
    target_resource_type VARCHAR(200),
    
    -- Status
    monitor_condition VARCHAR(50), -- fired, resolved
    signal_type VARCHAR(50),
    
    -- Timestamps
    fired_datetime TIMESTAMPTZ,
    resolved_datetime TIMESTAMPTZ,
    
    -- Details
    description TEXT,
    summary TEXT,
    context JSONB DEFAULT '{}',
    
    -- Action Groups
    action_groups TEXT[] DEFAULT '{}',
    
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(subscription_id, alert_id)
);

-- Azure Activity Log
CREATE TABLE azure_activity_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES azure_subscriptions(id) ON DELETE CASCADE,
    event_id VARCHAR(255) NOT NULL,
    
    -- Event Details
    event_name VARCHAR(255) NOT NULL,
    operation_name VARCHAR(255) NOT NULL,
    category VARCHAR(100), -- administrative, security, service_health, alert, recommendation, policy, autoscale
    level VARCHAR(20), -- critical, error, warning, informational
    status VARCHAR(50), -- started, succeeded, failed
    
    -- Resource Information
    resource_id VARCHAR(1000),
    resource_group_name VARCHAR(255),
    resource_provider_name VARCHAR(255),
    resource_type VARCHAR(200),
    
    -- Caller Information
    caller VARCHAR(255),
    caller_ip_address INET,
    
    -- Claims and Authorization
    claims JSONB DEFAULT '{}',
    authorization JSONB DEFAULT '{}',
    
    -- Properties
    properties JSONB DEFAULT '{}',
    
    -- Timestamps
    event_timestamp TIMESTAMPTZ NOT NULL,
    submission_timestamp TIMESTAMPTZ,
    
    -- Correlation
    correlation_id VARCHAR(255),
    
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(subscription_id, event_id)
);

-- Comprehensive indexing for performance
CREATE INDEX idx_azure_subscriptions_client ON azure_subscriptions(client_id);
CREATE INDEX idx_azure_subscriptions_state ON azure_subscriptions(state);
CREATE INDEX idx_azure_subscriptions_sync ON azure_subscriptions(last_sync, sync_enabled);

CREATE INDEX idx_azure_resource_groups_subscription ON azure_resource_groups(subscription_id);
CREATE INDEX idx_azure_resource_groups_location ON azure_resource_groups(location);

CREATE INDEX idx_azure_resources_subscription ON azure_resources(subscription_id);
CREATE INDEX idx_azure_resources_resource_group ON azure_resources(resource_group_id);
CREATE INDEX idx_azure_resources_type ON azure_resources(resource_type);
CREATE INDEX idx_azure_resources_location ON azure_resources(location);
CREATE INDEX idx_azure_resources_power_state ON azure_resources(power_state);
CREATE INDEX idx_azure_resources_cost ON azure_resources(monthly_cost_usd);

CREATE INDEX idx_azure_vnets_subscription ON azure_virtual_networks(subscription_id);
CREATE INDEX idx_azure_vnets_location ON azure_virtual_networks(location);

CREATE INDEX idx_azure_storage_subscription ON azure_storage_accounts(subscription_id);
CREATE INDEX idx_azure_storage_account_type ON azure_storage_accounts(account_type);
CREATE INDEX idx_azure_storage_tier ON azure_storage_accounts(sku_tier);

CREATE INDEX idx_azure_sql_subscription ON azure_sql_databases(subscription_id);
CREATE INDEX idx_azure_sql_edition ON azure_sql_databases(edition);
CREATE INDEX idx_azure_sql_server ON azure_sql_databases(server_name);

CREATE INDEX idx_azure_keyvaults_subscription ON azure_key_vaults(subscription_id);
CREATE INDEX idx_azure_keyvaults_location ON azure_key_vaults(location);

CREATE INDEX idx_azure_alerts_subscription ON azure_alerts(subscription_id);
CREATE INDEX idx_azure_alerts_severity ON azure_alerts(severity);
CREATE INDEX idx_azure_alerts_condition ON azure_alerts(monitor_condition);
CREATE INDEX idx_azure_alerts_fired ON azure_alerts(fired_datetime);

CREATE INDEX idx_azure_activity_subscription ON azure_activity_log(subscription_id);
CREATE INDEX idx_azure_activity_category ON azure_activity_log(category);
CREATE INDEX idx_azure_activity_timestamp ON azure_activity_log(event_timestamp);
CREATE INDEX idx_azure_activity_operation ON azure_activity_log(operation_name);

-- Update triggers
CREATE TRIGGER update_azure_subscriptions_updated_at 
    BEFORE UPDATE ON azure_subscriptions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_generic();

-- Resource count maintenance for resource groups
CREATE OR REPLACE FUNCTION update_azure_resource_group_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE azure_resource_groups 
        SET 
            total_resources = total_resources + 1,
            compute_resources = CASE WHEN NEW.resource_type LIKE '%/virtualMachines%' OR NEW.resource_type LIKE '%/compute%' THEN compute_resources + 1 ELSE compute_resources END,
            storage_resources = CASE WHEN NEW.resource_type LIKE '%/storageAccounts%' OR NEW.resource_type LIKE '%/storage%' THEN storage_resources + 1 ELSE storage_resources END,
            network_resources = CASE WHEN NEW.resource_type LIKE '%/network%' OR NEW.resource_type LIKE '%/virtualNetworks%' THEN network_resources + 1 ELSE network_resources END,
            database_resources = CASE WHEN NEW.resource_type LIKE '%/sql%' OR NEW.resource_type LIKE '%/databases%' THEN database_resources + 1 ELSE database_resources END
        WHERE id = NEW.resource_group_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE azure_resource_groups 
        SET 
            total_resources = GREATEST(total_resources - 1, 0),
            compute_resources = CASE WHEN OLD.resource_type LIKE '%/virtualMachines%' OR OLD.resource_type LIKE '%/compute%' THEN GREATEST(compute_resources - 1, 0) ELSE compute_resources END,
            storage_resources = CASE WHEN OLD.resource_type LIKE '%/storageAccounts%' OR OLD.resource_type LIKE '%/storage%' THEN GREATEST(storage_resources - 1, 0) ELSE storage_resources END,
            network_resources = CASE WHEN OLD.resource_type LIKE '%/network%' OR OLD.resource_type LIKE '%/virtualNetworks%' THEN GREATEST(network_resources - 1, 0) ELSE network_resources END,
            database_resources = CASE WHEN OLD.resource_type LIKE '%/sql%' OR OLD.resource_type LIKE '%/databases%' THEN GREATEST(database_resources - 1, 0) ELSE database_resources END
        WHERE id = OLD.resource_group_id;
    END IF;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_azure_resource_group_counts_trigger
    AFTER INSERT OR DELETE ON azure_resources
    FOR EACH ROW EXECUTE FUNCTION update_azure_resource_group_counts();

-- Cleanup old activity log entries
CREATE OR REPLACE FUNCTION cleanup_azure_activity_log()
RETURNS void AS $$
BEGIN
    DELETE FROM azure_activity_log 
    WHERE event_timestamp < NOW() - INTERVAL '3 months'
    AND level IN ('informational');
    
    DELETE FROM azure_activity_log 
    WHERE event_timestamp < NOW() - INTERVAL '1 year'
    AND level = 'warning';
END;
$$ LANGUAGE plpgsql;

COMMENT ON TABLE azure_subscriptions IS 'Azure subscription configurations with cost tracking and sync settings';
COMMENT ON TABLE azure_resources IS 'Comprehensive Azure resource inventory with performance metrics';
COMMENT ON TABLE azure_alerts IS 'Azure Monitor alerts and notifications tracking';
COMMENT ON TABLE azure_activity_log IS 'Azure Activity Log for audit and monitoring purposes';