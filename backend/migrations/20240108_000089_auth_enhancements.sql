-- Authentication Enhancements Migration
-- Adds OIDC providers, API keys, SAML providers, and RBAC tables

-- Auth providers table (OIDC/OAuth2)
CREATE TABLE IF NOT EXISTS auth_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    provider_type VARCHAR(50) NOT NULL, -- 'oidc', 'oauth2', 'saml'
    display_name VARCHAR(255) NOT NULL,
    client_id VARCHAR(500) NOT NULL,
    client_secret TEXT, -- Encrypted
    tenant_id VARCHAR(255), -- For Azure AD
    auth_url TEXT,
    token_url TEXT,
    userinfo_url TEXT,
    issuer_url TEXT,
    jwks_url TEXT,
    scopes TEXT[] DEFAULT ARRAY['openid', 'profile', 'email'],
    allowed_domains TEXT[] DEFAULT ARRAY[]::TEXT[],
    auto_create_users BOOLEAN DEFAULT true,
    default_role_id UUID REFERENCES roles(id),
    role_mapping JSONB, -- Maps IdP groups to Resolve roles
    attribute_mapping JSONB, -- Maps IdP claims to user fields
    enabled BOOLEAN DEFAULT true,
    allow_registration BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- SAML providers table
CREATE TABLE IF NOT EXISTS saml_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    entity_id TEXT NOT NULL, -- IdP Entity ID
    sso_url TEXT NOT NULL, -- IdP SSO URL
    sso_binding VARCHAR(50) DEFAULT 'HTTP-POST', -- HTTP-POST or HTTP-Redirect
    slo_url TEXT, -- IdP Single Logout URL (optional)
    slo_binding VARCHAR(50), -- HTTP-POST or HTTP-Redirect
    signing_cert TEXT NOT NULL, -- IdP signing certificate (PEM)
    encrypt_assertions BOOLEAN DEFAULT false,
    encryption_cert TEXT, -- SP encryption certificate
    sign_authn_requests BOOLEAN DEFAULT false,
    sp_signing_key TEXT, -- SP signing private key (encrypted)
    sp_signing_cert TEXT, -- SP signing certificate
    name_id_format VARCHAR(255) DEFAULT 'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress',
    attribute_mapping JSONB NOT NULL DEFAULT '{
        "email": ["email", "emailaddress", "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress"],
        "first_name": ["firstName", "givenname", "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname"],
        "last_name": ["lastName", "surname", "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname"],
        "groups": ["groups", "memberOf", "http://schemas.microsoft.com/ws/2008/06/identity/claims/groups"]
    }'::jsonb,
    role_mapping JSONB, -- Maps SAML groups to Resolve roles
    allowed_domains TEXT[] DEFAULT ARRAY[]::TEXT[],
    auto_create_users BOOLEAN DEFAULT true,
    default_role_id UUID REFERENCES roles(id),
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- API Keys table
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    key_hash VARCHAR(64) NOT NULL, -- SHA-256 hash
    key_prefix VARCHAR(8) NOT NULL, -- First 8 chars for identification
    scopes JSONB NOT NULL DEFAULT '[]'::jsonb,
    expires_at TIMESTAMPTZ,
    allowed_ips TEXT[] DEFAULT ARRAY[]::TEXT[],
    rate_limit INTEGER DEFAULT 0, -- Requests per minute, 0 = unlimited
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    usage_count BIGINT DEFAULT 0,
    UNIQUE(key_prefix)
);

-- OAuth state table (for PKCE and state validation)
CREATE TABLE IF NOT EXISTS oauth_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state VARCHAR(128) NOT NULL UNIQUE,
    provider_id UUID NOT NULL,
    provider_type VARCHAR(50) NOT NULL, -- 'oidc', 'saml'
    code_verifier VARCHAR(128), -- For PKCE
    nonce VARCHAR(128), -- For OIDC
    redirect_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

-- SAML assertion tracking (for replay prevention)
CREATE TABLE IF NOT EXISTS saml_assertions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assertion_id VARCHAR(255) NOT NULL UNIQUE,
    provider_id UUID NOT NULL REFERENCES saml_providers(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id),
    issued_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- User OAuth connections (links users to identity providers)
CREATE TABLE IF NOT EXISTS user_oauth_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_type VARCHAR(50) NOT NULL, -- 'oidc', 'saml'
    provider_id UUID NOT NULL, -- FK to auth_providers or saml_providers
    external_id VARCHAR(500) NOT NULL, -- User's ID at the IdP
    external_email VARCHAR(500),
    access_token TEXT, -- Encrypted
    refresh_token TEXT, -- Encrypted
    token_expires_at TIMESTAMPTZ,
    raw_profile JSONB, -- Full profile data from IdP
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    UNIQUE(provider_type, provider_id, external_id)
);

-- Add indexes
CREATE INDEX IF NOT EXISTS idx_auth_providers_enabled ON auth_providers(enabled) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_saml_providers_enabled ON saml_providers(enabled) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys(key_prefix) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_api_keys_expires ON api_keys(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_oauth_states_state ON oauth_states(state);
CREATE INDEX IF NOT EXISTS idx_oauth_states_expires ON oauth_states(expires_at);
CREATE INDEX IF NOT EXISTS idx_saml_assertions_assertion_id ON saml_assertions(assertion_id);
CREATE INDEX IF NOT EXISTS idx_saml_assertions_expires ON saml_assertions(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_oauth_connections_user ON user_oauth_connections(user_id);
CREATE INDEX IF NOT EXISTS idx_user_oauth_connections_provider ON user_oauth_connections(provider_type, provider_id);

-- Add MFA columns to users if not exists
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'mfa_enabled') THEN
        ALTER TABLE users ADD COLUMN mfa_enabled BOOLEAN DEFAULT false;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'mfa_secret') THEN
        ALTER TABLE users ADD COLUMN mfa_secret TEXT; -- Encrypted TOTP secret
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'mfa_backup_codes') THEN
        ALTER TABLE users ADD COLUMN mfa_backup_codes TEXT[]; -- Encrypted backup codes
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'failed_login_attempts') THEN
        ALTER TABLE users ADD COLUMN failed_login_attempts INTEGER DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'locked_until') THEN
        ALTER TABLE users ADD COLUMN locked_until TIMESTAMPTZ;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'last_login_at') THEN
        ALTER TABLE users ADD COLUMN last_login_at TIMESTAMPTZ;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'password_changed_at') THEN
        ALTER TABLE users ADD COLUMN password_changed_at TIMESTAMPTZ;
    END IF;
END $$;

-- Permissions table (for RBAC)
CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    resource VARCHAR(100) NOT NULL, -- 'clients', 'tickets', 'assets', etc.
    action VARCHAR(50) NOT NULL, -- 'read', 'create', 'update', 'delete', 'all'
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Role permissions junction table
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- Insert default permissions
INSERT INTO permissions (name, description, resource, action) VALUES
    ('clients.read', 'View clients', 'clients', 'read'),
    ('clients.create', 'Create clients', 'clients', 'create'),
    ('clients.update', 'Update clients', 'clients', 'update'),
    ('clients.delete', 'Delete clients', 'clients', 'delete'),
    ('tickets.read', 'View tickets', 'tickets', 'read'),
    ('tickets.create', 'Create tickets', 'tickets', 'create'),
    ('tickets.update', 'Update tickets', 'tickets', 'update'),
    ('tickets.delete', 'Delete tickets', 'tickets', 'delete'),
    ('tickets.assign', 'Assign tickets', 'tickets', 'assign'),
    ('assets.read', 'View assets', 'assets', 'read'),
    ('assets.create', 'Create assets', 'assets', 'create'),
    ('assets.update', 'Update assets', 'assets', 'update'),
    ('assets.delete', 'Delete assets', 'assets', 'delete'),
    ('passwords.read', 'View passwords', 'passwords', 'read'),
    ('passwords.create', 'Create passwords', 'passwords', 'create'),
    ('passwords.update', 'Update passwords', 'passwords', 'update'),
    ('passwords.delete', 'Delete passwords', 'passwords', 'delete'),
    ('passwords.reveal', 'Reveal password values', 'passwords', 'reveal'),
    ('documentation.read', 'View documentation', 'documentation', 'read'),
    ('documentation.create', 'Create documentation', 'documentation', 'create'),
    ('documentation.update', 'Update documentation', 'documentation', 'update'),
    ('documentation.delete', 'Delete documentation', 'documentation', 'delete'),
    ('invoices.read', 'View invoices', 'invoices', 'read'),
    ('invoices.create', 'Create invoices', 'invoices', 'create'),
    ('invoices.update', 'Update invoices', 'invoices', 'update'),
    ('invoices.delete', 'Delete invoices', 'invoices', 'delete'),
    ('invoices.approve', 'Approve invoices', 'invoices', 'approve'),
    ('users.read', 'View users', 'users', 'read'),
    ('users.create', 'Create users', 'users', 'create'),
    ('users.update', 'Update users', 'users', 'update'),
    ('users.delete', 'Delete users', 'users', 'delete'),
    ('settings.read', 'View settings', 'settings', 'read'),
    ('settings.update', 'Update settings', 'settings', 'update'),
    ('reports.read', 'View reports', 'reports', 'read'),
    ('reports.export', 'Export reports', 'reports', 'export'),
    ('admin.all', 'Full admin access', 'all', 'all')
ON CONFLICT (name) DO NOTHING;

-- Cleanup function for expired oauth states
CREATE OR REPLACE FUNCTION cleanup_expired_oauth_states()
RETURNS void AS $$
BEGIN
    DELETE FROM oauth_states WHERE expires_at < NOW();
    DELETE FROM saml_assertions WHERE expires_at < NOW();
END;
$$ LANGUAGE plpgsql;

-- Trigger function to update updated_at
CREATE OR REPLACE FUNCTION update_auth_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers
DROP TRIGGER IF EXISTS auth_providers_updated_at ON auth_providers;
CREATE TRIGGER auth_providers_updated_at
    BEFORE UPDATE ON auth_providers
    FOR EACH ROW EXECUTE FUNCTION update_auth_updated_at();

DROP TRIGGER IF EXISTS saml_providers_updated_at ON saml_providers;
CREATE TRIGGER saml_providers_updated_at
    BEFORE UPDATE ON saml_providers
    FOR EACH ROW EXECUTE FUNCTION update_auth_updated_at();

DROP TRIGGER IF EXISTS user_oauth_connections_updated_at ON user_oauth_connections;
CREATE TRIGGER user_oauth_connections_updated_at
    BEFORE UPDATE ON user_oauth_connections
    FOR EACH ROW EXECUTE FUNCTION update_auth_updated_at();
