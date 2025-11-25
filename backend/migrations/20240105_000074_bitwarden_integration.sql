-- Bitwarden/Vaultwarden Integration for Password Synchronization
-- Enables bi-directional sync between Resolve password manager and external password managers

-- Bitwarden Server Configurations
CREATE TABLE bitwarden_servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    server_type VARCHAR(50) NOT NULL, -- bitwarden, vaultwarden
    name VARCHAR(255) NOT NULL,
    server_url TEXT NOT NULL,
    
    -- Authentication
    client_id_encrypted TEXT, -- For API access
    client_secret_encrypted TEXT,
    api_key_encrypted TEXT, -- Personal API key
    identity_url TEXT,
    api_url TEXT,
    
    -- Organization Settings
    organization_id VARCHAR(255), -- Bitwarden organization ID
    collection_sync_enabled BOOLEAN DEFAULT true,
    sync_all_collections BOOLEAN DEFAULT false,
    allowed_collections TEXT[] DEFAULT '{}',
    
    -- Sync Configuration
    sync_enabled BOOLEAN DEFAULT true,
    sync_direction VARCHAR(20) DEFAULT 'bidirectional', -- pull_only, push_only, bidirectional
    sync_interval_hours INTEGER DEFAULT 24,
    last_sync TIMESTAMPTZ,
    last_sync_status VARCHAR(50) DEFAULT 'pending',
    last_error TEXT,
    
    -- Conflict Resolution
    conflict_resolution VARCHAR(30) DEFAULT 'manual', -- manual, bitwarden_wins, resolve_wins, newer_wins
    auto_resolve_conflicts BOOLEAN DEFAULT false,
    
    -- Mapping Rules
    folder_mapping JSONB DEFAULT '{}', -- Maps Bitwarden folders to Resolve folders
    field_mapping JSONB DEFAULT '{}', -- Maps custom fields
    
    -- Security Settings
    require_master_password BOOLEAN DEFAULT true,
    enforce_2fa BOOLEAN DEFAULT false,
    vault_timeout_minutes INTEGER DEFAULT 15,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

-- Sync Mapping Table (tracks which items are synced between systems)
CREATE TABLE bitwarden_sync_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bitwarden_server_id UUID NOT NULL REFERENCES bitwarden_servers(id) ON DELETE CASCADE,
    resolve_password_id UUID NOT NULL REFERENCES passwords(id) ON DELETE CASCADE,
    bitwarden_item_id VARCHAR(255) NOT NULL,
    bitwarden_organization_id VARCHAR(255),
    bitwarden_collection_id VARCHAR(255),
    bitwarden_folder_id VARCHAR(255),
    
    -- Sync Status
    sync_status VARCHAR(50) DEFAULT 'pending', -- pending, synced, conflict, error, deleted
    last_synced TIMESTAMPTZ,
    
    -- Conflict Information
    conflict_reason TEXT,
    conflict_data JSONB,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id),
    
    -- Item Versions for conflict detection
    resolve_version INTEGER DEFAULT 1,
    bitwarden_revision_date TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(bitwarden_server_id, bitwarden_item_id)
);

-- Sync History/Audit Log
CREATE TABLE bitwarden_sync_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bitwarden_server_id UUID NOT NULL REFERENCES bitwarden_servers(id) ON DELETE CASCADE,
    sync_mapping_id UUID REFERENCES bitwarden_sync_mappings(id) ON DELETE SET NULL,
    
    -- Sync Event Details
    sync_type VARCHAR(50) NOT NULL, -- full_sync, incremental_sync, item_sync, manual_sync
    sync_direction VARCHAR(20) NOT NULL, -- pull, push, bidirectional
    action VARCHAR(50) NOT NULL, -- create, update, delete, conflict, skip
    
    -- Item Information
    item_type VARCHAR(50), -- login, secure_note, card, identity, file_attachment
    item_name VARCHAR(500),
    bitwarden_item_id VARCHAR(255),
    resolve_password_id UUID,
    
    -- Sync Results
    status VARCHAR(50) NOT NULL, -- success, failure, warning, conflict
    error_message TEXT,
    warning_message TEXT,
    
    -- Data Changes
    changes_summary JSONB, -- Summary of what changed
    before_data JSONB, -- State before sync (for rollback)
    after_data JSONB, -- State after sync
    
    -- Timing
    sync_started_at TIMESTAMPTZ NOT NULL,
    sync_completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    
    -- User Context
    initiated_by UUID REFERENCES users(id),
    user_agent TEXT,
    ip_address INET,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Bitwarden Collections (for organization-based sync)
CREATE TABLE bitwarden_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bitwarden_server_id UUID NOT NULL REFERENCES bitwarden_servers(id) ON DELETE CASCADE,
    collection_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    organization_id VARCHAR(255) NOT NULL,
    
    -- Collection Properties
    external_id VARCHAR(255),
    read_only BOOLEAN DEFAULT false,
    hide_passwords BOOLEAN DEFAULT false,
    
    -- Sync Settings
    sync_enabled BOOLEAN DEFAULT true,
    resolve_folder_id UUID REFERENCES password_folders(id) ON DELETE SET NULL,
    
    -- Statistics
    item_count INTEGER DEFAULT 0,
    last_item_sync TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    UNIQUE(bitwarden_server_id, collection_id)
);

-- Bitwarden Organizations
CREATE TABLE bitwarden_organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bitwarden_server_id UUID NOT NULL REFERENCES bitwarden_servers(id) ON DELETE CASCADE,
    organization_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    
    -- Organization Details
    business_name VARCHAR(255),
    business_address1 VARCHAR(255),
    business_address2 VARCHAR(255),
    business_address3 VARCHAR(255),
    business_country VARCHAR(2),
    business_tax_number VARCHAR(50),
    
    -- Plan Information
    plan_type VARCHAR(50), -- free, families, teams, enterprise
    seats INTEGER,
    max_collections INTEGER,
    max_storage_gb INTEGER,
    use_groups BOOLEAN DEFAULT false,
    use_directory BOOLEAN DEFAULT false,
    use_events BOOLEAN DEFAULT false,
    use_totp BOOLEAN DEFAULT false,
    use_2fa BOOLEAN DEFAULT false,
    use_api BOOLEAN DEFAULT false,
    use_reset_password BOOLEAN DEFAULT false,
    
    -- User's Role in Organization
    user_type INTEGER, -- 0=Owner, 1=Admin, 2=User, 3=Manager, 4=Custom
    user_status INTEGER, -- 0=Invited, 1=Accepted, 2=Confirmed
    permissions JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    UNIQUE(bitwarden_server_id, organization_id)
);

-- Bitwarden Sync Conflicts (for manual resolution)
CREATE TABLE bitwarden_sync_conflicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bitwarden_server_id UUID NOT NULL REFERENCES bitwarden_servers(id) ON DELETE CASCADE,
    sync_mapping_id UUID NOT NULL REFERENCES bitwarden_sync_mappings(id) ON DELETE CASCADE,
    
    -- Conflict Details
    conflict_type VARCHAR(50) NOT NULL, -- data_mismatch, both_modified, deleted_modified, field_conflict
    conflict_field VARCHAR(100), -- Which field has the conflict
    conflict_reason TEXT NOT NULL,
    
    -- Conflicting Data
    resolve_data JSONB NOT NULL,
    bitwarden_data JSONB NOT NULL,
    
    -- Timestamps
    resolve_modified_at TIMESTAMPTZ,
    bitwarden_modified_at TIMESTAMPTZ,
    conflict_detected_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Resolution
    resolution_status VARCHAR(30) DEFAULT 'pending', -- pending, resolved, ignored
    resolution_choice VARCHAR(30), -- use_resolve, use_bitwarden, use_merged, use_custom
    resolved_data JSONB,
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    
    -- Auto-resolution attempts
    auto_resolution_attempted BOOLEAN DEFAULT false,
    auto_resolution_result VARCHAR(50),
    auto_resolution_reason TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Bitwarden Export/Import Jobs
CREATE TABLE bitwarden_export_import_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bitwarden_server_id UUID NOT NULL REFERENCES bitwarden_servers(id) ON DELETE CASCADE,
    
    -- Job Details
    job_type VARCHAR(20) NOT NULL, -- export, import
    job_status VARCHAR(30) DEFAULT 'pending', -- pending, running, completed, failed, cancelled
    operation VARCHAR(50) NOT NULL, -- full_export, selective_export, full_import, selective_import, csv_import
    
    -- File Information
    file_format VARCHAR(20), -- json, csv, xml, encrypted_json
    file_path TEXT,
    file_size_bytes BIGINT,
    file_hash VARCHAR(128),
    
    -- Selection Criteria (for selective operations)
    include_folders TEXT[] DEFAULT '{}',
    exclude_folders TEXT[] DEFAULT '{}',
    include_collections TEXT[] DEFAULT '{}',
    date_range_start TIMESTAMPTZ,
    date_range_end TIMESTAMPTZ,
    
    -- Results
    items_processed INTEGER DEFAULT 0,
    items_succeeded INTEGER DEFAULT 0,
    items_failed INTEGER DEFAULT 0,
    items_skipped INTEGER DEFAULT 0,
    
    -- Error Information
    error_message TEXT,
    error_details JSONB,
    warnings TEXT[] DEFAULT '{}',
    
    -- Progress Tracking
    progress_percentage INTEGER DEFAULT 0,
    current_operation VARCHAR(255),
    estimated_completion TIMESTAMPTZ,
    
    -- User Context
    requested_by UUID NOT NULL REFERENCES users(id),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Comprehensive indexing
CREATE INDEX idx_bitwarden_servers_client ON bitwarden_servers(client_id);
CREATE INDEX idx_bitwarden_servers_sync ON bitwarden_servers(sync_enabled, last_sync);
CREATE INDEX idx_bitwarden_servers_type ON bitwarden_servers(server_type);

CREATE INDEX idx_bitwarden_sync_mappings_server ON bitwarden_sync_mappings(bitwarden_server_id);
CREATE INDEX idx_bitwarden_sync_mappings_password ON bitwarden_sync_mappings(resolve_password_id);
CREATE INDEX idx_bitwarden_sync_mappings_item ON bitwarden_sync_mappings(bitwarden_item_id);
CREATE INDEX idx_bitwarden_sync_mappings_status ON bitwarden_sync_mappings(sync_status);
CREATE INDEX idx_bitwarden_sync_mappings_synced ON bitwarden_sync_mappings(last_synced);

CREATE INDEX idx_bitwarden_sync_history_server ON bitwarden_sync_history(bitwarden_server_id);
CREATE INDEX idx_bitwarden_sync_history_type ON bitwarden_sync_history(sync_type);
CREATE INDEX idx_bitwarden_sync_history_status ON bitwarden_sync_history(status);
CREATE INDEX idx_bitwarden_sync_history_started ON bitwarden_sync_history(sync_started_at);

CREATE INDEX idx_bitwarden_collections_server ON bitwarden_collections(bitwarden_server_id);
CREATE INDEX idx_bitwarden_collections_org ON bitwarden_collections(organization_id);
CREATE INDEX idx_bitwarden_collections_sync ON bitwarden_collections(sync_enabled);

CREATE INDEX idx_bitwarden_organizations_server ON bitwarden_organizations(bitwarden_server_id);
CREATE INDEX idx_bitwarden_organizations_plan ON bitwarden_organizations(plan_type);

CREATE INDEX idx_bitwarden_conflicts_server ON bitwarden_sync_conflicts(bitwarden_server_id);
CREATE INDEX idx_bitwarden_conflicts_mapping ON bitwarden_sync_conflicts(sync_mapping_id);
CREATE INDEX idx_bitwarden_conflicts_status ON bitwarden_sync_conflicts(resolution_status);
CREATE INDEX idx_bitwarden_conflicts_detected ON bitwarden_sync_conflicts(conflict_detected_at);

CREATE INDEX idx_bitwarden_jobs_server ON bitwarden_export_import_jobs(bitwarden_server_id);
CREATE INDEX idx_bitwarden_jobs_status ON bitwarden_export_import_jobs(job_status);
CREATE INDEX idx_bitwarden_jobs_type ON bitwarden_export_import_jobs(job_type);
CREATE INDEX idx_bitwarden_jobs_requested ON bitwarden_export_import_jobs(requested_by);

-- Update triggers
CREATE TRIGGER update_bitwarden_servers_updated_at 
    BEFORE UPDATE ON bitwarden_servers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_generic();

CREATE TRIGGER update_bitwarden_collections_updated_at 
    BEFORE UPDATE ON bitwarden_collections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_generic();

CREATE TRIGGER update_bitwarden_organizations_updated_at 
    BEFORE UPDATE ON bitwarden_organizations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_generic();

-- Sync conflict detection trigger
CREATE OR REPLACE FUNCTION detect_bitwarden_sync_conflicts()
RETURNS TRIGGER AS $$
DECLARE
    conflict_exists BOOLEAN := false;
    existing_mapping RECORD;
BEGIN
    -- Check if this password is mapped to a Bitwarden item
    SELECT * INTO existing_mapping 
    FROM bitwarden_sync_mappings bsm
    WHERE bsm.resolve_password_id = NEW.id
    AND bsm.sync_status = 'synced';
    
    IF FOUND THEN
        -- Check if there might be a conflict (simplified check)
        IF OLD.updated_at IS DISTINCT FROM NEW.updated_at THEN
            -- Update the sync mapping status to indicate potential conflict
            UPDATE bitwarden_sync_mappings 
            SET sync_status = 'conflict',
                resolve_version = resolve_version + 1
            WHERE resolve_password_id = NEW.id;
            
            -- Log the potential conflict for sync process to handle
            INSERT INTO bitwarden_sync_history (
                bitwarden_server_id,
                sync_mapping_id,
                sync_type,
                sync_direction,
                action,
                item_type,
                item_name,
                resolve_password_id,
                status,
                warning_message,
                sync_started_at,
                sync_completed_at
            ) VALUES (
                existing_mapping.bitwarden_server_id,
                existing_mapping.id,
                'item_sync',
                'pull',
                'conflict',
                'login',
                NEW.name,
                NEW.id,
                'warning',
                'Resolve password modified - potential sync conflict',
                NOW(),
                NOW()
            );
        END IF;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Only create trigger if passwords table exists (it should from earlier migrations)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'passwords') THEN
        CREATE TRIGGER detect_bitwarden_sync_conflicts_trigger
            AFTER UPDATE ON passwords
            FOR EACH ROW EXECUTE FUNCTION detect_bitwarden_sync_conflicts();
    END IF;
END $$;

-- Collection item count maintenance
CREATE OR REPLACE FUNCTION update_bitwarden_collection_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE bitwarden_collections 
        SET item_count = item_count + 1,
            last_item_sync = NOW()
        WHERE collection_id = NEW.bitwarden_collection_id
        AND bitwarden_server_id = NEW.bitwarden_server_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE bitwarden_collections 
        SET item_count = GREATEST(item_count - 1, 0),
            last_item_sync = NOW()
        WHERE collection_id = OLD.bitwarden_collection_id
        AND bitwarden_server_id = OLD.bitwarden_server_id;
    END IF;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_bitwarden_collection_counts_trigger
    AFTER INSERT OR DELETE ON bitwarden_sync_mappings
    FOR EACH ROW EXECUTE FUNCTION update_bitwarden_collection_counts();

-- Cleanup old sync history
CREATE OR REPLACE FUNCTION cleanup_bitwarden_sync_history()
RETURNS void AS $$
BEGIN
    -- Keep only last 3 months of successful syncs
    DELETE FROM bitwarden_sync_history 
    WHERE sync_started_at < NOW() - INTERVAL '3 months'
    AND status = 'success'
    AND sync_type IN ('full_sync', 'incremental_sync');
    
    -- Keep errors and conflicts for 1 year
    DELETE FROM bitwarden_sync_history 
    WHERE sync_started_at < NOW() - INTERVAL '1 year'
    AND status IN ('failure', 'conflict');
END;
$$ LANGUAGE plpgsql;

-- Auto-resolve simple conflicts function
CREATE OR REPLACE FUNCTION auto_resolve_bitwarden_conflicts()
RETURNS INTEGER AS $$
DECLARE
    conflict_record RECORD;
    resolved_count INTEGER := 0;
BEGIN
    FOR conflict_record IN 
        SELECT * FROM bitwarden_sync_conflicts 
        WHERE resolution_status = 'pending' 
        AND auto_resolution_attempted = false
        AND conflict_type IN ('field_conflict')
        ORDER BY conflict_detected_at
        LIMIT 100
    LOOP
        -- Simple auto-resolution: use newer timestamp
        IF conflict_record.resolve_modified_at > conflict_record.bitwarden_modified_at THEN
            UPDATE bitwarden_sync_conflicts
            SET resolution_status = 'resolved',
                resolution_choice = 'use_resolve',
                resolved_data = conflict_record.resolve_data,
                resolved_at = NOW(),
                auto_resolution_attempted = true,
                auto_resolution_result = 'success',
                auto_resolution_reason = 'Resolve data is newer'
            WHERE id = conflict_record.id;
            
            resolved_count := resolved_count + 1;
        ELSIF conflict_record.bitwarden_modified_at > conflict_record.resolve_modified_at THEN
            UPDATE bitwarden_sync_conflicts
            SET resolution_status = 'resolved',
                resolution_choice = 'use_bitwarden',
                resolved_data = conflict_record.bitwarden_data,
                resolved_at = NOW(),
                auto_resolution_attempted = true,
                auto_resolution_result = 'success',
                auto_resolution_reason = 'Bitwarden data is newer'
            WHERE id = conflict_record.id;
            
            resolved_count := resolved_count + 1;
        ELSE
            -- Mark as attempted but not resolved
            UPDATE bitwarden_sync_conflicts
            SET auto_resolution_attempted = true,
                auto_resolution_result = 'skipped',
                auto_resolution_reason = 'Cannot determine which is newer'
            WHERE id = conflict_record.id;
        END IF;
    END LOOP;
    
    RETURN resolved_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON TABLE bitwarden_servers IS 'Configuration for Bitwarden/Vaultwarden server connections';
COMMENT ON TABLE bitwarden_sync_mappings IS 'Maps Resolve passwords to Bitwarden items for synchronization';
COMMENT ON TABLE bitwarden_sync_history IS 'Audit log of all password synchronization activities';
COMMENT ON TABLE bitwarden_sync_conflicts IS 'Tracks synchronization conflicts requiring manual resolution';