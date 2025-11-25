-- Enhanced Password Management with Secure Sharing and MFA Token Storage
-- Implements comprehensive password management, secure sharing, and multi-factor authentication

-- Password vaults for organizing credentials
CREATE TABLE password_vaults (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    client_id UUID REFERENCES clients(id), -- NULL for global/shared vaults
    vault_type VARCHAR(50) DEFAULT 'standard', -- standard, shared, personal, emergency
    access_level VARCHAR(50) DEFAULT 'team', -- personal, team, client, global
    encryption_key_id UUID, -- Reference to encryption key
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_password_vaults_client_id ON password_vaults(client_id);
CREATE INDEX idx_password_vaults_type ON password_vaults(vault_type);

-- Enhanced password entries with secure sharing
CREATE TABLE password_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES password_vaults(id) ON DELETE CASCADE,
    client_id UUID REFERENCES clients(id), -- for client-specific passwords
    asset_id UUID REFERENCES assets(id), -- link to specific asset
    folder_path VARCHAR(255), -- folder organization within vault
    title VARCHAR(100) NOT NULL,
    username VARCHAR(255),
    password_encrypted TEXT, -- AES-256 encrypted password
    email VARCHAR(255),
    url VARCHAR(500),
    notes_encrypted TEXT, -- encrypted notes
    password_strength_score INTEGER, -- 0-100 password strength
    password_last_changed TIMESTAMPTZ,
    password_expires_at TIMESTAMPTZ,
    rotation_days INTEGER, -- auto-rotate every X days
    last_accessed TIMESTAMPTZ,
    access_count INTEGER DEFAULT 0,
    is_favorite BOOLEAN DEFAULT false,
    is_compromised BOOLEAN DEFAULT false,
    compromised_date TIMESTAMPTZ,
    tags TEXT[], -- searchable tags
    custom_fields JSONB, -- additional encrypted fields
    requires_approval BOOLEAN DEFAULT false,
    approval_required_for TEXT[], -- array of actions requiring approval
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ -- soft delete
);

CREATE INDEX idx_password_entries_vault_id ON password_entries(vault_id);
CREATE INDEX idx_password_entries_client_id ON password_entries(client_id);
CREATE INDEX idx_password_entries_asset_id ON password_entries(asset_id);
CREATE INDEX idx_password_entries_title ON password_entries(title);
CREATE INDEX idx_password_entries_url ON password_entries(url);
CREATE INDEX idx_password_entries_tags ON password_entries USING GIN(tags);
CREATE INDEX idx_password_entries_active ON password_entries(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_password_entries_expires ON password_entries(password_expires_at) WHERE password_expires_at IS NOT NULL;

-- Password sharing and access control
CREATE TABLE password_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID NOT NULL REFERENCES password_entries(id) ON DELETE CASCADE,
    shared_with_user_id UUID REFERENCES users(id),
    shared_with_group_id UUID, -- reference to user groups
    shared_with_client_id UUID REFERENCES clients(id), -- share with all client contacts
    share_type VARCHAR(50) NOT NULL, -- view, edit, admin
    permissions JSONB, -- detailed permissions
    expires_at TIMESTAMPTZ,
    max_access_count INTEGER,
    current_access_count INTEGER DEFAULT 0,
    require_reason BOOLEAN DEFAULT false,
    notify_on_access BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    shared_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_accessed TIMESTAMPTZ
);

CREATE INDEX idx_password_shares_password_id ON password_shares(password_id);
CREATE INDEX idx_password_shares_user_id ON password_shares(shared_with_user_id);
CREATE INDEX idx_password_shares_expires ON password_shares(expires_at);

-- Password access audit log
CREATE TABLE password_access_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID NOT NULL REFERENCES password_entries(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    action VARCHAR(50) NOT NULL, -- view, copy, edit, share, delete, export
    access_method VARCHAR(50), -- web, api, mobile, browser_extension
    ip_address INET,
    user_agent TEXT,
    access_reason TEXT, -- reason provided by user
    session_id VARCHAR(255),
    geolocation JSONB, -- {"country": "US", "city": "New York"}
    is_authorized BOOLEAN DEFAULT true,
    risk_score INTEGER, -- 0-100 risk assessment
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_password_access_log_password_id ON password_access_log(password_id);
CREATE INDEX idx_password_access_log_user_id ON password_access_log(user_id);
CREATE INDEX idx_password_access_log_created_at ON password_access_log(created_at);
CREATE INDEX idx_password_access_log_action ON password_access_log(action);

-- MFA (TOTP) token storage
CREATE TABLE mfa_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID REFERENCES password_entries(id), -- optional link to password entry
    client_id UUID REFERENCES clients(id),
    asset_id UUID REFERENCES assets(id),
    service_name VARCHAR(100) NOT NULL,
    account_name VARCHAR(255) NOT NULL,
    issuer VARCHAR(100),
    secret_encrypted TEXT NOT NULL, -- AES-256 encrypted TOTP secret
    algorithm VARCHAR(20) DEFAULT 'SHA1', -- SHA1, SHA256, SHA512
    digits INTEGER DEFAULT 6, -- 6 or 8 digit codes
    period INTEGER DEFAULT 30, -- time period in seconds
    backup_codes_encrypted TEXT[], -- encrypted backup codes
    last_used_at TIMESTAMPTZ,
    usage_count INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    notes_encrypted TEXT,
    tags TEXT[],
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_mfa_tokens_password_id ON mfa_tokens(password_id);
CREATE INDEX idx_mfa_tokens_client_id ON mfa_tokens(client_id);
CREATE INDEX idx_mfa_tokens_asset_id ON mfa_tokens(asset_id);
CREATE INDEX idx_mfa_tokens_service ON mfa_tokens(service_name);
CREATE INDEX idx_mfa_tokens_active ON mfa_tokens(deleted_at) WHERE deleted_at IS NULL;

-- Secure notes for sensitive information
CREATE TABLE secure_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES password_vaults(id) ON DELETE CASCADE,
    client_id UUID REFERENCES clients(id),
    asset_id UUID REFERENCES assets(id),
    title VARCHAR(100) NOT NULL,
    content_encrypted TEXT NOT NULL, -- AES-256 encrypted content
    content_type VARCHAR(50) DEFAULT 'text', -- text, code, json, xml
    tags TEXT[],
    expires_at TIMESTAMPTZ,
    is_favorite BOOLEAN DEFAULT false,
    access_count INTEGER DEFAULT 0,
    last_accessed TIMESTAMPTZ,
    requires_approval BOOLEAN DEFAULT false,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_secure_notes_vault_id ON secure_notes(vault_id);
CREATE INDEX idx_secure_notes_client_id ON secure_notes(client_id);
CREATE INDEX idx_secure_notes_asset_id ON secure_notes(asset_id);
CREATE INDEX idx_secure_notes_tags ON secure_notes USING GIN(tags);
CREATE INDEX idx_secure_notes_active ON secure_notes(deleted_at) WHERE deleted_at IS NULL;

-- API keys and certificates storage
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES password_vaults(id) ON DELETE CASCADE,
    client_id UUID REFERENCES clients(id),
    asset_id UUID REFERENCES assets(id),
    service_name VARCHAR(100) NOT NULL,
    key_name VARCHAR(100) NOT NULL,
    key_type VARCHAR(50) NOT NULL, -- api_key, access_token, certificate, private_key, public_key
    key_value_encrypted TEXT NOT NULL,
    key_id VARCHAR(255), -- external key ID/name
    permissions_scope TEXT, -- what the key can access
    rate_limits JSONB, -- API rate limiting info
    expires_at TIMESTAMPTZ,
    last_rotated TIMESTAMPTZ,
    rotation_days INTEGER,
    auto_rotate BOOLEAN DEFAULT false,
    environment VARCHAR(50), -- production, staging, development
    endpoint_url VARCHAR(500),
    documentation_url VARCHAR(500),
    tags TEXT[],
    usage_count INTEGER DEFAULT 0,
    last_used_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_api_keys_vault_id ON api_keys(vault_id);
CREATE INDEX idx_api_keys_client_id ON api_keys(client_id);
CREATE INDEX idx_api_keys_service ON api_keys(service_name);
CREATE INDEX idx_api_keys_type ON api_keys(key_type);
CREATE INDEX idx_api_keys_expires ON api_keys(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_api_keys_active ON api_keys(deleted_at) WHERE deleted_at IS NULL;

-- Password sharing requests and approvals
CREATE TABLE password_share_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID NOT NULL REFERENCES password_entries(id) ON DELETE CASCADE,
    requested_by UUID NOT NULL REFERENCES users(id),
    requested_from UUID NOT NULL REFERENCES users(id),
    request_reason TEXT NOT NULL,
    access_level VARCHAR(50) NOT NULL, -- view, edit
    duration_hours INTEGER, -- how long access is needed
    approval_status VARCHAR(20) DEFAULT 'pending', -- pending, approved, denied
    approved_by UUID REFERENCES users(id),
    approval_reason TEXT,
    expires_at TIMESTAMPTZ,
    granted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_password_share_requests_password_id ON password_share_requests(password_id);
CREATE INDEX idx_password_share_requests_requested_by ON password_share_requests(requested_by);
CREATE INDEX idx_password_share_requests_status ON password_share_requests(approval_status);

-- Password security policies
CREATE TABLE password_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    client_id UUID REFERENCES clients(id), -- NULL for global policies
    is_global BOOLEAN DEFAULT false,
    min_length INTEGER DEFAULT 12,
    require_uppercase BOOLEAN DEFAULT true,
    require_lowercase BOOLEAN DEFAULT true,
    require_numbers BOOLEAN DEFAULT true,
    require_symbols BOOLEAN DEFAULT true,
    forbidden_patterns TEXT[], -- patterns not allowed in passwords
    max_age_days INTEGER DEFAULT 90,
    password_history_count INTEGER DEFAULT 5, -- can't reuse last N passwords
    lockout_after_failures INTEGER DEFAULT 5,
    require_2fa BOOLEAN DEFAULT false,
    allow_password_sharing BOOLEAN DEFAULT true,
    require_approval_for_sharing BOOLEAN DEFAULT false,
    audit_all_access BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_password_policies_client_id ON password_policies(client_id);
CREATE INDEX idx_password_policies_global ON password_policies(is_global);

-- Password breach monitoring
CREATE TABLE password_breaches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID NOT NULL REFERENCES password_entries(id) ON DELETE CASCADE,
    breach_source VARCHAR(100), -- haveibeenpwned, custom, etc.
    breach_date DATE,
    severity VARCHAR(20), -- low, medium, high, critical
    description TEXT,
    is_resolved BOOLEAN DEFAULT false,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id),
    resolution_notes TEXT,
    discovered_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_password_breaches_password_id ON password_breaches(password_id);
CREATE INDEX idx_password_breaches_severity ON password_breaches(severity);
CREATE INDEX idx_password_breaches_resolved ON password_breaches(is_resolved);

-- Password generation templates
CREATE TABLE password_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    length INTEGER DEFAULT 16,
    include_uppercase BOOLEAN DEFAULT true,
    include_lowercase BOOLEAN DEFAULT true,
    include_numbers BOOLEAN DEFAULT true,
    include_symbols BOOLEAN DEFAULT true,
    exclude_ambiguous BOOLEAN DEFAULT true, -- exclude 0,O,l,1,etc.
    custom_symbols VARCHAR(100), -- custom symbol set
    word_separator VARCHAR(10), -- for passphrase style
    word_count INTEGER, -- for passphrase style
    is_system_template BOOLEAN DEFAULT false,
    usage_count INTEGER DEFAULT 0,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default password vault
INSERT INTO password_vaults (name, description, vault_type, access_level, created_by)
SELECT 
    'Default Vault',
    'Default password vault for all teams',
    'standard',
    'team',
    (SELECT id FROM users LIMIT 1)
WHERE EXISTS (SELECT 1 FROM users LIMIT 1);

-- Insert default password policy
INSERT INTO password_policies (name, is_global, min_length, max_age_days, created_by)
SELECT 
    'Default Password Policy',
    true,
    12,
    90,
    (SELECT id FROM users LIMIT 1)
WHERE EXISTS (SELECT 1 FROM users LIMIT 1);

-- Insert default password templates
INSERT INTO password_templates (name, description, length, is_system_template) VALUES
('Strong Password', 'Secure 16-character password', 16, true),
('Complex Password', 'Very secure 24-character password', 24, true),
('Simple PIN', 'Numeric PIN code', 8, true),
('Passphrase', 'Easy to remember passphrase', 20, true);

-- Function to calculate password strength
CREATE OR REPLACE FUNCTION calculate_password_strength(password_text TEXT)
RETURNS INTEGER AS $$
DECLARE
    score INTEGER := 0;
    length_bonus INTEGER;
    char_variety INTEGER := 0;
BEGIN
    -- Length scoring
    length_bonus := LEAST(LENGTH(password_text) * 4, 25);
    score := score + length_bonus;
    
    -- Character variety scoring
    IF password_text ~ '[a-z]' THEN char_variety := char_variety + 1; END IF;
    IF password_text ~ '[A-Z]' THEN char_variety := char_variety + 1; END IF;
    IF password_text ~ '[0-9]' THEN char_variety := char_variety + 1; END IF;
    IF password_text ~ '[^a-zA-Z0-9]' THEN char_variety := char_variety + 1; END IF;
    
    score := score + (char_variety * 10);
    
    -- Deductions for patterns
    IF password_text ~ '(.)\1{2,}' THEN score := score - 10; END IF; -- repeated characters
    IF password_text ~ '(012|123|234|345|456|567|678|789|890|abc|bcd|cde)' THEN score := score - 10; END IF; -- sequences
    
    -- Cap the score
    score := GREATEST(0, LEAST(100, score));
    
    RETURN score;
END;
$$ LANGUAGE plpgsql;

-- Function to generate secure sharing tokens
CREATE OR REPLACE FUNCTION generate_share_token(password_id_param UUID, user_id_param UUID)
RETURNS TEXT AS $$
DECLARE
    token_data TEXT;
    token_hash TEXT;
BEGIN
    token_data := password_id_param::TEXT || '|' || user_id_param::TEXT || '|' || EXTRACT(EPOCH FROM NOW())::TEXT;
    
    -- Simple hash function - in production, use a proper HMAC
    SELECT encode(sha256(token_data::bytea), 'hex') INTO token_hash;
    
    RETURN token_hash;
END;
$$ LANGUAGE plpgsql;