-- Enhanced Password Management System for Resolve
-- Secure vault with sharing, rotation, complexity, and breach monitoring

-- Password categories for organization
CREATE TABLE password_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    icon VARCHAR(50),
    color VARCHAR(7),
    description TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enhanced password vault
CREATE TABLE password_vault (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    asset_id UUID REFERENCES assets(id),
    category_id UUID REFERENCES password_categories(id),
    
    -- Basic fields
    name VARCHAR(255) NOT NULL,
    username VARCHAR(255),
    password TEXT NOT NULL, -- Encrypted
    url VARCHAR(500),
    
    -- Enhanced security fields
    totp_secret TEXT, -- Encrypted TOTP for 2FA
    recovery_codes TEXT[], -- Encrypted backup codes
    security_questions JSONB DEFAULT '[]', -- Q&A pairs, encrypted
    private_key TEXT, -- Encrypted SSH/SSL keys
    public_key TEXT,
    certificate TEXT,
    
    -- Password metadata
    password_strength INTEGER, -- 0-100 score
    password_age_days INTEGER DEFAULT 0,
    last_rotated TIMESTAMPTZ DEFAULT NOW(),
    rotation_period_days INTEGER, -- How often to rotate
    next_rotation_date DATE,
    complexity_requirements JSONB DEFAULT '{}', -- min_length, uppercase, numbers, etc
    
    -- Breach monitoring
    breach_check_enabled BOOLEAN DEFAULT true,
    last_breach_check TIMESTAMPTZ,
    breach_detected BOOLEAN DEFAULT false,
    breach_details JSONB,
    
    -- Access control
    owner_id UUID REFERENCES users(id),
    shared_with_users UUID[] DEFAULT '{}',
    shared_with_teams UUID[] DEFAULT '{}',
    share_expiry_date TIMESTAMPTZ,
    require_mfa_to_view BOOLEAN DEFAULT false,
    
    -- Additional fields
    notes TEXT, -- Encrypted
    tags TEXT[] DEFAULT '{}',
    custom_fields JSONB DEFAULT '{}', -- Encrypted
    attachments JSONB DEFAULT '[]', -- File references
    
    -- Compliance
    compliance_standards TEXT[] DEFAULT '{}', -- PCI, HIPAA, etc
    expires_at TIMESTAMPTZ,
    auto_rotate BOOLEAN DEFAULT false,
    
    -- Audit
    created_by UUID REFERENCES users(id),
    last_accessed_by UUID REFERENCES users(id),
    last_accessed_at TIMESTAMPTZ,
    access_count INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Password sharing permissions
CREATE TABLE password_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID NOT NULL REFERENCES password_vault(id) ON DELETE CASCADE,
    shared_by UUID NOT NULL REFERENCES users(id),
    shared_with_type VARCHAR(50) NOT NULL, -- user, team, contact
    shared_with_id UUID NOT NULL,
    permission_level VARCHAR(50) DEFAULT 'view', -- view, edit, admin
    expires_at TIMESTAMPTZ,
    requires_approval BOOLEAN DEFAULT false,
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    share_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Password history tracking
CREATE TABLE password_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID NOT NULL REFERENCES password_vault(id) ON DELETE CASCADE,
    old_password TEXT NOT NULL, -- Encrypted
    changed_by UUID REFERENCES users(id),
    change_reason TEXT,
    password_strength INTEGER,
    changed_at TIMESTAMPTZ DEFAULT NOW()
);

-- Password access logs
CREATE TABLE password_access_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID NOT NULL REFERENCES password_vault(id) ON DELETE CASCADE,
    accessed_by UUID REFERENCES users(id),
    access_type VARCHAR(50) NOT NULL, -- view, copy, edit, share
    ip_address INET,
    user_agent TEXT,
    access_granted BOOLEAN DEFAULT true,
    denial_reason TEXT,
    mfa_verified BOOLEAN DEFAULT false,
    accessed_at TIMESTAMPTZ DEFAULT NOW()
);

-- Password rotation schedules
CREATE TABLE password_rotation_schedule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID NOT NULL REFERENCES password_vault(id) ON DELETE CASCADE,
    schedule_type VARCHAR(50) NOT NULL, -- manual, automatic, policy-based
    rotation_frequency_days INTEGER NOT NULL,
    last_rotation TIMESTAMPTZ,
    next_rotation TIMESTAMPTZ NOT NULL,
    notification_days_before INTEGER DEFAULT 7,
    auto_generate_password BOOLEAN DEFAULT true,
    password_policy JSONB DEFAULT '{}', -- Generation rules
    rotation_status VARCHAR(50) DEFAULT 'pending', -- pending, in-progress, completed, failed
    assigned_to UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Password policies
CREATE TABLE password_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    client_id UUID REFERENCES clients(id), -- NULL for global policies
    
    -- Complexity requirements
    min_length INTEGER DEFAULT 12,
    max_length INTEGER DEFAULT 128,
    require_uppercase BOOLEAN DEFAULT true,
    require_lowercase BOOLEAN DEFAULT true,
    require_numbers BOOLEAN DEFAULT true,
    require_special_chars BOOLEAN DEFAULT true,
    special_chars_set VARCHAR(255) DEFAULT '!@#$%^&*()_+-=[]{}|;:,.<>?',
    prohibited_words TEXT[] DEFAULT '{}',
    
    -- Rotation requirements
    max_age_days INTEGER DEFAULT 90,
    min_age_days INTEGER DEFAULT 1,
    history_count INTEGER DEFAULT 5, -- Can't reuse last N passwords
    
    -- Security requirements
    require_mfa BOOLEAN DEFAULT false,
    require_approval_for_view BOOLEAN DEFAULT false,
    auto_expire_shared_access BOOLEAN DEFAULT true,
    shared_access_max_days INTEGER DEFAULT 30,
    
    -- Breach monitoring
    enable_breach_monitoring BOOLEAN DEFAULT true,
    block_breached_passwords BOOLEAN DEFAULT true,
    
    is_active BOOLEAN DEFAULT true,
    priority INTEGER DEFAULT 0, -- Higher priority policies override lower
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Password breach database (for checking compromised passwords)
CREATE TABLE password_breaches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_hash VARCHAR(64) NOT NULL, -- SHA-256 hash of compromised password
    breach_source VARCHAR(255),
    breach_date DATE,
    times_seen INTEGER DEFAULT 1,
    severity VARCHAR(50) DEFAULT 'high',
    added_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(password_hash)
);

-- Client password portal access
CREATE TABLE password_portal_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    allowed_passwords UUID[] DEFAULT '{}', -- Specific passwords they can access
    allowed_categories UUID[] DEFAULT '{}', -- Categories they can access
    require_mfa BOOLEAN DEFAULT true,
    access_level VARCHAR(50) DEFAULT 'view', -- view, request
    last_access_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(client_id, contact_id)
);

-- Password change requests from clients
CREATE TABLE password_change_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID NOT NULL REFERENCES password_vault(id),
    requested_by_contact UUID REFERENCES contacts(id),
    requested_by_user UUID REFERENCES users(id),
    request_type VARCHAR(50) NOT NULL, -- reset, rotation, access
    reason TEXT NOT NULL,
    urgency VARCHAR(50) DEFAULT 'normal', -- low, normal, high, critical
    status VARCHAR(50) DEFAULT 'pending', -- pending, approved, rejected, completed
    approved_by UUID REFERENCES users(id),
    completed_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_password_vault_client_id ON password_vault(client_id);
CREATE INDEX idx_password_vault_category_id ON password_vault(category_id);
CREATE INDEX idx_password_vault_next_rotation ON password_vault(next_rotation_date);
CREATE INDEX idx_password_vault_breach_detected ON password_vault(breach_detected);
CREATE INDEX idx_password_shares_password_id ON password_shares(password_id);
CREATE INDEX idx_password_shares_shared_with ON password_shares(shared_with_type, shared_with_id);
CREATE INDEX idx_password_history_password_id ON password_history(password_id);
CREATE INDEX idx_password_access_logs_password_id ON password_access_logs(password_id);
CREATE INDEX idx_password_breaches_hash ON password_breaches(password_hash);

-- Function to check password complexity
CREATE OR REPLACE FUNCTION check_password_complexity(
    password TEXT,
    policy_id UUID DEFAULT NULL
) RETURNS JSONB AS $$
DECLARE
    policy RECORD;
    result JSONB;
    score INTEGER := 0;
    issues TEXT[] := '{}';
BEGIN
    -- Get policy or use defaults
    IF policy_id IS NOT NULL THEN
        SELECT * INTO policy FROM password_policies WHERE id = policy_id;
    ELSE
        SELECT * INTO policy FROM password_policies WHERE client_id IS NULL AND is_active = true ORDER BY priority DESC LIMIT 1;
    END IF;
    
    -- Check length
    IF LENGTH(password) < COALESCE(policy.min_length, 12) THEN
        issues := array_append(issues, 'Password too short');
    ELSE
        score := score + 20;
    END IF;
    
    -- Check uppercase
    IF COALESCE(policy.require_uppercase, true) AND password !~ '[A-Z]' THEN
        issues := array_append(issues, 'Missing uppercase letter');
    ELSE
        score := score + 20;
    END IF;
    
    -- Check lowercase
    IF COALESCE(policy.require_lowercase, true) AND password !~ '[a-z]' THEN
        issues := array_append(issues, 'Missing lowercase letter');
    ELSE
        score := score + 20;
    END IF;
    
    -- Check numbers
    IF COALESCE(policy.require_numbers, true) AND password !~ '[0-9]' THEN
        issues := array_append(issues, 'Missing number');
    ELSE
        score := score + 20;
    END IF;
    
    -- Check special characters
    IF COALESCE(policy.require_special_chars, true) AND password !~ '[^A-Za-z0-9]' THEN
        issues := array_append(issues, 'Missing special character');
    ELSE
        score := score + 20;
    END IF;
    
    result := jsonb_build_object(
        'score', score,
        'valid', array_length(issues, 1) = 0,
        'issues', issues
    );
    
    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Function to generate secure password
CREATE OR REPLACE FUNCTION generate_secure_password(
    length INTEGER DEFAULT 16,
    include_uppercase BOOLEAN DEFAULT true,
    include_lowercase BOOLEAN DEFAULT true,
    include_numbers BOOLEAN DEFAULT true,
    include_special BOOLEAN DEFAULT true
) RETURNS TEXT AS $$
DECLARE
    chars TEXT := '';
    result TEXT := '';
    i INTEGER;
BEGIN
    IF include_lowercase THEN chars := chars || 'abcdefghijklmnopqrstuvwxyz'; END IF;
    IF include_uppercase THEN chars := chars || 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'; END IF;
    IF include_numbers THEN chars := chars || '0123456789'; END IF;
    IF include_special THEN chars := chars || '!@#$%^&*()_+-=[]{}|;:,.<>?'; END IF;
    
    FOR i IN 1..length LOOP
        result := result || substr(chars, floor(random() * length(chars) + 1)::integer, 1);
    END LOOP;
    
    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Insert default password categories
INSERT INTO password_categories (name, icon, color, sort_order) VALUES
('Administrative', 'shield', '#dc2626', 1),
('Application', 'apps', '#2563eb', 2),
('Database', 'database', '#7c3aed', 3),
('Email', 'mail', '#0891b2', 4),
('Network Device', 'router', '#059669', 5),
('Server', 'server', '#ea580c', 6),
('Service Account', 'robot', '#64748b', 7),
('Website', 'globe', '#0ea5e9', 8),
('WiFi', 'wifi', '#8b5cf6', 9),
('API Key', 'key', '#f59e0b', 10),
('Certificate', 'badge-check', '#10b981', 11),
('Other', 'dots-horizontal', '#6b7280', 99);

-- Insert default global password policy
INSERT INTO password_policies (name, description, client_id) VALUES
('Default Global Policy', 'Standard password policy for all clients', NULL);