-- Custom Asset Fields System
-- Implements Hudu-style flexible asset layouts and custom fields

-- Asset field types - defines available field types
CREATE TABLE asset_field_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) UNIQUE NOT NULL, -- text, number, date, boolean, select, multiselect, file, url, etc.
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    validation_rules JSONB, -- JSON schema for field validation
    ui_component VARCHAR(50), -- frontend component to use
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default field types
INSERT INTO asset_field_types (name, display_name, description, ui_component) VALUES
('text', 'Text', 'Single line text input', 'TextInput'),
('textarea', 'Text Area', 'Multi-line text input', 'TextArea'),
('number', 'Number', 'Numeric input', 'NumberInput'),
('decimal', 'Decimal', 'Decimal number input', 'DecimalInput'),
('date', 'Date', 'Date picker', 'DatePicker'),
('datetime', 'Date & Time', 'Date and time picker', 'DateTimePicker'),
('boolean', 'Yes/No', 'Boolean checkbox', 'Checkbox'),
('select', 'Dropdown', 'Single select dropdown', 'Select'),
('multiselect', 'Multi-Select', 'Multiple selection', 'MultiSelect'),
('file', 'File Upload', 'File attachment', 'FileUpload'),
('url', 'URL', 'Web address input', 'URLInput'),
('email', 'Email', 'Email address input', 'EmailInput'),
('phone', 'Phone', 'Phone number input', 'PhoneInput'),
('ip_address', 'IP Address', 'IP address input with validation', 'IPInput'),
('mac_address', 'MAC Address', 'MAC address input', 'MACInput'),
('password', 'Password', 'Encrypted password field', 'PasswordInput'),
('json', 'JSON Data', 'Structured JSON data', 'JSONEditor'),
('color', 'Color', 'Color picker', 'ColorPicker'),
('rating', 'Rating', 'Star rating field', 'RatingInput');

-- Asset layouts - defines custom layouts for different asset types
CREATE TABLE asset_layouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    asset_type VARCHAR(50) NOT NULL, -- matches asset.asset_type
    description TEXT,
    icon VARCHAR(50), -- icon class name
    color VARCHAR(7), -- hex color code
    is_system_layout BOOLEAN DEFAULT false, -- true for built-in layouts
    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(asset_type)
);

-- Asset layout fields - defines fields for each layout
CREATE TABLE asset_layout_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    layout_id UUID NOT NULL REFERENCES asset_layouts(id) ON DELETE CASCADE,
    field_type_id UUID NOT NULL REFERENCES asset_field_types(id),
    field_name VARCHAR(100) NOT NULL, -- database column name (snake_case)
    display_name VARCHAR(100) NOT NULL, -- human readable name
    description TEXT,
    is_required BOOLEAN DEFAULT false,
    is_searchable BOOLEAN DEFAULT true,
    is_shown_in_list BOOLEAN DEFAULT false, -- show in asset list view
    display_order INTEGER DEFAULT 0,
    default_value TEXT,
    placeholder TEXT,
    validation_rules JSONB, -- field-specific validation
    field_options JSONB, -- for select/multiselect options
    help_text TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(layout_id, field_name)
);

-- Asset field values - stores actual custom field data
CREATE TABLE asset_field_values (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    field_id UUID NOT NULL REFERENCES asset_layout_fields(id) ON DELETE CASCADE,
    field_value TEXT, -- stored as text, parsed based on field type
    field_value_encrypted TEXT, -- for password/sensitive fields
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(asset_id, field_id)
);

-- Create indexes for performance
CREATE INDEX idx_asset_field_values_asset_id ON asset_field_values(asset_id);
CREATE INDEX idx_asset_field_values_field_id ON asset_field_values(field_id);
CREATE INDEX idx_asset_layout_fields_layout_id ON asset_layout_fields(layout_id);
CREATE INDEX idx_asset_layouts_asset_type ON asset_layouts(asset_type);
CREATE INDEX idx_asset_layouts_active ON asset_layouts(is_active) WHERE is_active = true;

-- Insert some default asset layouts
INSERT INTO asset_layouts (name, asset_type, description, icon, color, is_system_layout, created_by) 
SELECT 
    'Server Layout', 'server', 'Standard server documentation layout', 'server', '#2563eb', true,
    (SELECT id FROM users LIMIT 1)
WHERE EXISTS (SELECT 1 FROM users LIMIT 1);

INSERT INTO asset_layouts (name, asset_type, description, icon, color, is_system_layout, created_by)
SELECT 
    'Network Device Layout', 'network_device', 'Switches, routers, firewalls', 'network-wired', '#059669', true,
    (SELECT id FROM users LIMIT 1)
WHERE EXISTS (SELECT 1 FROM users LIMIT 1);

INSERT INTO asset_layouts (name, asset_type, description, icon, color, is_system_layout, created_by)
SELECT 
    'Workstation Layout', 'workstation', 'Desktop and laptop computers', 'monitor', '#7c3aed', true,
    (SELECT id FROM users LIMIT 1)
WHERE EXISTS (SELECT 1 FROM users LIMIT 1);

INSERT INTO asset_layouts (name, asset_type, description, icon, color, is_system_layout, created_by)
SELECT 
    'Mobile Device Layout', 'mobile', 'Phones and tablets', 'smartphone', '#dc2626', true,
    (SELECT id FROM users LIMIT 1)
WHERE EXISTS (SELECT 1 FROM users LIMIT 1);

-- Add some default fields for server layout (if layout was created)
INSERT INTO asset_layout_fields (layout_id, field_type_id, field_name, display_name, description, is_required, is_shown_in_list, display_order, validation_rules)
SELECT 
    al.id,
    aft.id,
    'cpu_cores',
    'CPU Cores',
    'Number of CPU cores',
    false,
    true,
    1,
    '{"min": 1, "max": 256}'::jsonb
FROM asset_layouts al, asset_field_types aft 
WHERE al.asset_type = 'server' AND aft.name = 'number'
AND EXISTS (SELECT 1 FROM users LIMIT 1);

INSERT INTO asset_layout_fields (layout_id, field_type_id, field_name, display_name, description, is_required, is_shown_in_list, display_order, validation_rules)
SELECT 
    al.id,
    aft.id,
    'memory_gb',
    'Memory (GB)',
    'Total system memory in GB',
    false,
    true,
    2,
    '{"min": 1, "max": 8192}'::jsonb
FROM asset_layouts al, asset_field_types aft 
WHERE al.asset_type = 'server' AND aft.name = 'number'
AND EXISTS (SELECT 1 FROM users LIMIT 1);

INSERT INTO asset_layout_fields (layout_id, field_type_id, field_name, display_name, description, is_required, is_shown_in_list, display_order, field_options)
SELECT 
    al.id,
    aft.id,
    'virtualization_platform',
    'Virtualization Platform',
    'Hypervisor or container platform',
    false,
    true,
    3,
    '{"options": ["VMware vSphere", "Microsoft Hyper-V", "Proxmox", "KVM", "Docker", "Kubernetes", "None"]}'::jsonb
FROM asset_layouts al, asset_field_types aft 
WHERE al.asset_type = 'server' AND aft.name = 'select'
AND EXISTS (SELECT 1 FROM users LIMIT 1);

INSERT INTO asset_layout_fields (layout_id, field_type_id, field_name, display_name, description, is_required, display_order)
SELECT 
    al.id,
    aft.id,
    'management_url',
    'Management URL',
    'Web management interface URL',
    false,
    4
FROM asset_layouts al, asset_field_types aft 
WHERE al.asset_type = 'server' AND aft.name = 'url'
AND EXISTS (SELECT 1 FROM users LIMIT 1);

-- Asset templates - predefined asset configurations
CREATE TABLE asset_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    layout_id UUID NOT NULL REFERENCES asset_layouts(id) ON DELETE CASCADE,
    description TEXT,
    default_values JSONB, -- JSON object with field_name -> default_value
    is_public BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Asset field history - track changes to custom fields
CREATE TABLE asset_field_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    field_id UUID NOT NULL REFERENCES asset_layout_fields(id),
    old_value TEXT,
    new_value TEXT,
    changed_by UUID REFERENCES users(id),
    change_reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_asset_field_history_asset_id ON asset_field_history(asset_id);
CREATE INDEX idx_asset_field_history_created_at ON asset_field_history(created_at);