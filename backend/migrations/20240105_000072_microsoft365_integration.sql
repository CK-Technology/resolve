-- Microsoft 365 Tenant Management Integration
-- Comprehensive M365 tenant, user, license, and service monitoring

-- M365 Tenants (main organization/client tenant)
CREATE TABLE m365_tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    tenant_id VARCHAR(255) NOT NULL,
    tenant_name VARCHAR(255) NOT NULL,
    domain_name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    default_domain VARCHAR(255),
    tenant_type VARCHAR(50) DEFAULT 'business', -- business, enterprise, education
    country VARCHAR(2),
    preferred_language VARCHAR(10) DEFAULT 'en-US',
    
    -- Authentication & API Access
    client_id_encrypted TEXT NOT NULL, -- Application (client) ID
    client_secret_encrypted TEXT NOT NULL, -- Client secret
    tenant_endpoint TEXT,
    graph_api_endpoint TEXT DEFAULT 'https://graph.microsoft.com/v1.0',
    
    -- Status & Sync
    status VARCHAR(50) DEFAULT 'active',
    last_sync TIMESTAMPTZ,
    sync_enabled BOOLEAN DEFAULT true,
    sync_interval_hours INTEGER DEFAULT 6,
    last_sync_status VARCHAR(50) DEFAULT 'pending',
    last_error TEXT,
    
    -- License Information
    total_licenses INTEGER DEFAULT 0,
    assigned_licenses INTEGER DEFAULT 0,
    available_licenses INTEGER DEFAULT 0,
    
    -- Tenant Settings
    security_defaults_enabled BOOLEAN,
    password_policy JSONB DEFAULT '{}',
    conditional_access_enabled BOOLEAN DEFAULT false,
    mfa_required BOOLEAN DEFAULT false,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

-- M365 Users
CREATE TABLE m365_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES m365_tenants(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL, -- Microsoft Graph user ID
    user_principal_name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    given_name VARCHAR(100),
    surname VARCHAR(100),
    mail VARCHAR(255),
    mobile_phone VARCHAR(50),
    office_location VARCHAR(255),
    job_title VARCHAR(255),
    department VARCHAR(255),
    manager_id VARCHAR(255),
    
    -- Account Status
    account_enabled BOOLEAN DEFAULT true,
    last_sign_in TIMESTAMPTZ,
    sign_in_activity JSONB DEFAULT '{}',
    creation_type VARCHAR(50),
    external_user_state VARCHAR(50),
    
    -- Authentication
    password_policies TEXT[] DEFAULT '{}',
    strong_authentication JSONB DEFAULT '{}',
    mfa_enabled BOOLEAN DEFAULT false,
    mfa_methods TEXT[] DEFAULT '{}',
    
    -- License Information  
    assigned_licenses JSONB DEFAULT '[]',
    license_assignment_states JSONB DEFAULT '[]',
    usage_location VARCHAR(2),
    
    -- Additional Properties
    on_premises_sync_enabled BOOLEAN DEFAULT false,
    on_premises_distinguished_name TEXT,
    on_premises_sam_account_name VARCHAR(255),
    proxy_addresses TEXT[] DEFAULT '{}',
    
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, user_id)
);

-- M365 Groups (Teams, Distribution Lists, Security Groups)
CREATE TABLE m365_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES m365_tenants(id) ON DELETE CASCADE,
    group_id VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    mail_nickname VARCHAR(255),
    mail VARCHAR(255),
    description TEXT,
    
    -- Group Type
    group_type VARCHAR(50), -- unified, security, distribution, dynamic
    security_enabled BOOLEAN DEFAULT false,
    mail_enabled BOOLEAN DEFAULT false,
    
    -- Teams Properties
    is_teams_enabled BOOLEAN DEFAULT false,
    teams_template VARCHAR(100),
    
    -- Membership
    member_count INTEGER DEFAULT 0,
    owner_count INTEGER DEFAULT 0,
    guest_count INTEGER DEFAULT 0,
    
    -- Settings
    visibility VARCHAR(20) DEFAULT 'private', -- public, private, hiddenmembership
    auto_subscribe_new_members BOOLEAN DEFAULT false,
    allow_external_senders BOOLEAN DEFAULT false,
    hide_from_address_lists BOOLEAN DEFAULT false,
    hide_from_outlook_clients BOOLEAN DEFAULT false,
    
    creation_datetime TIMESTAMPTZ,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, group_id)
);

-- M365 Licenses (SKUs available to tenant)
CREATE TABLE m365_licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES m365_tenants(id) ON DELETE CASCADE,
    sku_id VARCHAR(255) NOT NULL,
    sku_part_number VARCHAR(100) NOT NULL,
    product_name VARCHAR(255) NOT NULL,
    
    -- License Quantities
    total_units INTEGER NOT NULL DEFAULT 0,
    consumed_units INTEGER DEFAULT 0,
    enabled_units INTEGER DEFAULT 0,
    suspended_units INTEGER DEFAULT 0,
    warning_units INTEGER DEFAULT 0,
    
    -- Service Plans included in this license
    service_plans JSONB DEFAULT '[]',
    
    -- Cost Information
    cost_per_license DECIMAL(10,2),
    annual_cost DECIMAL(15,2),
    
    -- Purchase Information
    purchase_date DATE,
    renewal_date DATE,
    subscription_id VARCHAR(255),
    
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, sku_id)
);

-- M365 Applications/Services
CREATE TABLE m365_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES m365_tenants(id) ON DELETE CASCADE,
    app_id VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    app_type VARCHAR(50), -- enterprise_app, registered_app, managed_identity
    
    -- Authentication
    sign_in_audience VARCHAR(50),
    required_resource_access JSONB DEFAULT '[]',
    oauth2_permissions JSONB DEFAULT '[]',
    
    -- Status
    enabled BOOLEAN DEFAULT true,
    last_sign_in TIMESTAMPTZ,
    sign_in_count INTEGER DEFAULT 0,
    
    -- Enterprise App Properties
    homepage_url TEXT,
    logout_url TEXT,
    service_principal_id VARCHAR(255),
    
    created_datetime TIMESTAMPTZ,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, app_id)
);

-- M365 SharePoint Sites
CREATE TABLE m365_sharepoint_sites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES m365_tenants(id) ON DELETE CASCADE,
    site_id VARCHAR(255) NOT NULL,
    web_url TEXT NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    
    -- Site Type & Template
    site_collection_type VARCHAR(50), -- team_site, communication_site, hub_site
    template VARCHAR(100),
    is_personal_site BOOLEAN DEFAULT false,
    
    -- Storage
    storage_quota_mb BIGINT DEFAULT 0,
    storage_used_mb BIGINT DEFAULT 0,
    storage_warning_level_mb BIGINT DEFAULT 0,
    
    -- Activity
    last_activity_date DATE,
    file_count INTEGER DEFAULT 0,
    active_file_count INTEGER DEFAULT 0,
    page_view_count INTEGER DEFAULT 0,
    visited_page_count INTEGER DEFAULT 0,
    
    -- Settings
    sharing_capability VARCHAR(50), -- disabled, external_user_sharing_only, external_user_and_guest_sharing, existing_external_user_sharing_only
    external_sharing BOOLEAN DEFAULT false,
    
    root_web_template VARCHAR(100),
    locale_id INTEGER,
    created_datetime TIMESTAMPTZ,
    last_modified_datetime TIMESTAMPTZ,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, site_id)
);

-- M365 Exchange Online Mailboxes
CREATE TABLE m365_exchange_mailboxes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES m365_tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES m365_users(id) ON DELETE CASCADE,
    mailbox_guid VARCHAR(255) NOT NULL,
    primary_smtp_address VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    
    -- Mailbox Type
    recipient_type VARCHAR(50), -- user_mailbox, shared_mailbox, room_mailbox, equipment_mailbox
    recipient_type_details VARCHAR(100),
    
    -- Storage
    mailbox_size_mb BIGINT DEFAULT 0,
    archive_size_mb BIGINT DEFAULT 0,
    prohibit_send_quota_mb INTEGER,
    prohibit_send_receive_quota_mb INTEGER,
    issue_warning_quota_mb INTEGER,
    
    -- Archive
    archive_status VARCHAR(50),
    archive_name VARCHAR(255),
    auto_expanding_archive BOOLEAN DEFAULT false,
    
    -- Activity
    last_logon_time TIMESTAMPTZ,
    last_user_action_time TIMESTAMPTZ,
    
    -- Settings
    litigation_hold_enabled BOOLEAN DEFAULT false,
    in_place_holds TEXT[] DEFAULT '{}',
    forwarding_smtp_address VARCHAR(255),
    deliver_to_mailbox_and_forward BOOLEAN DEFAULT false,
    
    -- Permissions
    send_as_permissions TEXT[] DEFAULT '{}',
    send_on_behalf_permissions TEXT[] DEFAULT '{}',
    full_access_permissions TEXT[] DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, mailbox_guid)
);

-- M365 Teams
CREATE TABLE m365_teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES m365_tenants(id) ON DELETE CASCADE,
    group_id UUID REFERENCES m365_groups(id) ON DELETE CASCADE,
    team_id VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Team Settings
    internal_id VARCHAR(255),
    classification VARCHAR(100),
    specialization VARCHAR(50), -- none, education_standard, education_class, education_professional_learning_community, education_pln, healthcare_standard, healthcare_care_coordination
    visibility VARCHAR(20) DEFAULT 'private',
    
    -- Features
    is_archived BOOLEAN DEFAULT false,
    allow_add_remove_apps BOOLEAN DEFAULT true,
    allow_channel_mentions BOOLEAN DEFAULT true,
    allow_create_update_channels BOOLEAN DEFAULT true,
    allow_create_update_remove_connectors BOOLEAN DEFAULT true,
    allow_create_update_remove_tabs BOOLEAN DEFAULT true,
    allow_custom_memes BOOLEAN DEFAULT true,
    allow_delete_channels BOOLEAN DEFAULT true,
    allow_giphy BOOLEAN DEFAULT true,
    allow_guest_create_update_channels BOOLEAN DEFAULT false,
    allow_guest_delete_channels BOOLEAN DEFAULT false,
    allow_owner_delete_messages BOOLEAN DEFAULT true,
    allow_stickers_and_memes BOOLEAN DEFAULT true,
    allow_team_mentions BOOLEAN DEFAULT true,
    allow_user_delete_messages BOOLEAN DEFAULT true,
    allow_user_edit_messages BOOLEAN DEFAULT true,
    
    -- Activity Statistics
    active_users_count INTEGER DEFAULT 0,
    channels_count INTEGER DEFAULT 0,
    posts_count INTEGER DEFAULT 0,
    replies_count INTEGER DEFAULT 0,
    mentions_count INTEGER DEFAULT 0,
    
    created_datetime TIMESTAMPTZ,
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, team_id)
);

-- M365 Compliance & Security Events
CREATE TABLE m365_security_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES m365_tenants(id) ON DELETE CASCADE,
    event_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_category VARCHAR(50) NOT NULL, -- sign_in, audit, security_alert, compliance
    
    -- Event Details
    title VARCHAR(500),
    description TEXT,
    severity VARCHAR(20), -- low, medium, high, informational
    status VARCHAR(50), -- new, in_progress, resolved, dismissed
    
    -- User/Resource Information
    affected_user_id VARCHAR(255),
    affected_user_principal_name VARCHAR(255),
    source_ip INET,
    user_agent TEXT,
    location JSONB,
    
    -- Additional Context
    activity VARCHAR(255),
    application VARCHAR(255),
    device_detail JSONB,
    risk_detail JSONB,
    
    -- Timestamps
    created_datetime TIMESTAMPTZ NOT NULL,
    event_datetime TIMESTAMPTZ NOT NULL,
    last_modified_datetime TIMESTAMPTZ,
    
    -- Investigation
    assigned_to VARCHAR(255),
    comments TEXT,
    feedback VARCHAR(50),
    
    last_synced TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, event_id)
);

-- Comprehensive indexing
CREATE INDEX idx_m365_tenants_client ON m365_tenants(client_id);
CREATE INDEX idx_m365_tenants_domain ON m365_tenants(domain_name);
CREATE INDEX idx_m365_tenants_sync ON m365_tenants(last_sync, sync_enabled);

CREATE INDEX idx_m365_users_tenant ON m365_users(tenant_id);
CREATE INDEX idx_m365_users_upn ON m365_users(user_principal_name);
CREATE INDEX idx_m365_users_mail ON m365_users(mail);
CREATE INDEX idx_m365_users_enabled ON m365_users(account_enabled);
CREATE INDEX idx_m365_users_last_signin ON m365_users(last_sign_in);

CREATE INDEX idx_m365_groups_tenant ON m365_groups(tenant_id);
CREATE INDEX idx_m365_groups_type ON m365_groups(group_type);
CREATE INDEX idx_m365_groups_teams ON m365_groups(is_teams_enabled);

CREATE INDEX idx_m365_licenses_tenant ON m365_licenses(tenant_id);
CREATE INDEX idx_m365_licenses_sku ON m365_licenses(sku_part_number);

CREATE INDEX idx_m365_applications_tenant ON m365_applications(tenant_id);
CREATE INDEX idx_m365_applications_type ON m365_applications(app_type);

CREATE INDEX idx_m365_sharepoint_tenant ON m365_sharepoint_sites(tenant_id);
CREATE INDEX idx_m365_sharepoint_activity ON m365_sharepoint_sites(last_activity_date);

CREATE INDEX idx_m365_mailboxes_tenant ON m365_exchange_mailboxes(tenant_id);
CREATE INDEX idx_m365_mailboxes_smtp ON m365_exchange_mailboxes(primary_smtp_address);
CREATE INDEX idx_m365_mailboxes_size ON m365_exchange_mailboxes(mailbox_size_mb);

CREATE INDEX idx_m365_teams_tenant ON m365_teams(tenant_id);
CREATE INDEX idx_m365_teams_archived ON m365_teams(is_archived);

CREATE INDEX idx_m365_security_events_tenant ON m365_security_events(tenant_id);
CREATE INDEX idx_m365_security_events_type ON m365_security_events(event_type);
CREATE INDEX idx_m365_security_events_datetime ON m365_security_events(event_datetime);
CREATE INDEX idx_m365_security_events_severity ON m365_security_events(severity);
CREATE INDEX idx_m365_security_events_status ON m365_security_events(status);

-- Update triggers
CREATE TRIGGER update_m365_tenants_updated_at 
    BEFORE UPDATE ON m365_tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_generic();

-- License utilization maintenance
CREATE OR REPLACE FUNCTION update_m365_license_utilization()
RETURNS TRIGGER AS $$
BEGIN
    -- Update tenant license counts when user licenses change
    IF TG_OP = 'UPDATE' AND (OLD.assigned_licenses IS DISTINCT FROM NEW.assigned_licenses) THEN
        UPDATE m365_tenants 
        SET 
            assigned_licenses = (
                SELECT COUNT(DISTINCT mu.user_id) 
                FROM m365_users mu 
                WHERE mu.tenant_id = NEW.tenant_id 
                AND jsonb_array_length(mu.assigned_licenses) > 0
            ),
            updated_at = NOW()
        WHERE id = NEW.tenant_id;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_m365_license_utilization_trigger
    AFTER UPDATE ON m365_users
    FOR EACH ROW EXECUTE FUNCTION update_m365_license_utilization();

-- Cleanup old security events
CREATE OR REPLACE FUNCTION cleanup_m365_security_events()
RETURNS void AS $$
BEGIN
    DELETE FROM m365_security_events 
    WHERE event_datetime < NOW() - INTERVAL '1 year'
    AND severity IN ('low', 'informational');
    
    DELETE FROM m365_security_events 
    WHERE event_datetime < NOW() - INTERVAL '2 years'
    AND severity = 'medium';
END;
$$ LANGUAGE plpgsql;

COMMENT ON TABLE m365_tenants IS 'Microsoft 365 tenant configurations and sync settings';
COMMENT ON TABLE m365_users IS 'Microsoft 365 user accounts with licensing and authentication details';
COMMENT ON TABLE m365_groups IS 'Microsoft 365 groups including Teams, Distribution Lists, and Security Groups';
COMMENT ON TABLE m365_licenses IS 'Microsoft 365 license SKUs and utilization tracking';
COMMENT ON TABLE m365_security_events IS 'Microsoft 365 security and compliance event monitoring';