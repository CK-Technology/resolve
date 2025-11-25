-- File Attachments and Customizable Asset Fields System

-- File storage and management
CREATE TABLE file_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    file_path TEXT NOT NULL,
    file_type VARCHAR(50) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    file_hash VARCHAR(128),
    is_public BOOLEAN DEFAULT false,
    description TEXT,
    tags TEXT[] DEFAULT '{}',
    uploaded_by UUID NOT NULL REFERENCES users(id),
    uploaded_at TIMESTAMPTZ DEFAULT NOW(),
    last_accessed TIMESTAMPTZ,
    access_count INTEGER DEFAULT 0,
    thumbnail_path TEXT,
    ocr_text TEXT,
    metadata JSONB DEFAULT '{}'
);

-- Custom field definitions
CREATE TABLE custom_field_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,
    asset_type VARCHAR(100),
    field_name VARCHAR(100) NOT NULL,
    field_label VARCHAR(255) NOT NULL,
    field_type VARCHAR(50) NOT NULL,
    field_options JSONB DEFAULT '{}',
    validation_rules JSONB DEFAULT '{}',
    default_value TEXT,
    help_text TEXT,
    field_group VARCHAR(100),
    sort_order INTEGER DEFAULT 0,
    is_required BOOLEAN DEFAULT false,
    is_searchable BOOLEAN DEFAULT false,
    is_visible BOOLEAN DEFAULT true,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    UNIQUE(client_id, entity_type, asset_type, field_name)
);

-- Custom field values storage
CREATE TABLE custom_field_values (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    field_definition_id UUID NOT NULL REFERENCES custom_field_definitions(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    field_value TEXT,
    field_value_json JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    UNIQUE(field_definition_id, entity_id)
);

-- Asset-specific file associations
CREATE TABLE asset_file_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    file_attachment_id UUID NOT NULL REFERENCES file_attachments(id) ON DELETE CASCADE,
    is_primary_image BOOLEAN DEFAULT false,
    display_order INTEGER DEFAULT 0,
    visibility VARCHAR(20) DEFAULT 'internal',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(asset_id, file_attachment_id)
);

-- Indexes
CREATE INDEX idx_file_attachments_entity ON file_attachments(client_id, entity_type, entity_id);
CREATE INDEX idx_custom_field_definitions_entity ON custom_field_definitions(client_id, entity_type, asset_type);
CREATE INDEX idx_custom_field_values_entity ON custom_field_values(entity_type, entity_id);
CREATE INDEX idx_asset_file_attachments_asset ON asset_file_attachments(asset_id);