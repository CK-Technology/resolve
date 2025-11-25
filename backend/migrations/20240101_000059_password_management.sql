-- Password Management Tables
CREATE TABLE IF NOT EXISTS password_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES password_folders(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS passwords (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    username VARCHAR(255),
    password_encrypted TEXT NOT NULL,
    url TEXT,
    notes_encrypted TEXT,
    category VARCHAR(100),
    tags JSONB DEFAULT '[]'::jsonb,
    favorite BOOLEAN DEFAULT false,
    otp_secret_encrypted TEXT,
    phonetic_enabled BOOLEAN DEFAULT true,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_accessed TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    strength_score INTEGER DEFAULT 0,
    breach_detected BOOLEAN DEFAULT false,
    folder_id UUID REFERENCES password_folders(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS password_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    password_id UUID NOT NULL REFERENCES passwords(id) ON DELETE CASCADE,
    share_token VARCHAR(255) NOT NULL UNIQUE,
    created_by UUID NOT NULL REFERENCES users(id),
    recipient_email VARCHAR(255),
    recipient_name VARCHAR(255),
    expires_at TIMESTAMPTZ NOT NULL,
    max_views INTEGER,
    view_count INTEGER DEFAULT 0,
    require_email_verification BOOLEAN DEFAULT false,
    require_password BOOLEAN DEFAULT false,
    access_password VARCHAR(255),
    one_time_use BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_accessed TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true
);

-- Indexes for better performance
CREATE INDEX IF NOT EXISTS idx_passwords_client_id ON passwords(client_id);
CREATE INDEX IF NOT EXISTS idx_passwords_created_by ON passwords(created_by);
CREATE INDEX IF NOT EXISTS idx_passwords_folder_id ON passwords(folder_id);
CREATE INDEX IF NOT EXISTS idx_passwords_category ON passwords(category);
CREATE INDEX IF NOT EXISTS idx_passwords_tags ON passwords USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_passwords_expires_at ON passwords(expires_at);
CREATE INDEX IF NOT EXISTS idx_passwords_strength_score ON passwords(strength_score);
CREATE INDEX IF NOT EXISTS idx_passwords_favorite ON passwords(favorite) WHERE favorite = true;

CREATE INDEX IF NOT EXISTS idx_password_folders_client_id ON password_folders(client_id);
CREATE INDEX IF NOT EXISTS idx_password_folders_parent_id ON password_folders(parent_id);

CREATE INDEX IF NOT EXISTS idx_password_shares_token ON password_shares(share_token);
CREATE INDEX IF NOT EXISTS idx_password_shares_password_id ON password_shares(password_id);
CREATE INDEX IF NOT EXISTS idx_password_shares_created_by ON password_shares(created_by);
CREATE INDEX IF NOT EXISTS idx_password_shares_expires_at ON password_shares(expires_at);
CREATE INDEX IF NOT EXISTS idx_password_shares_active ON password_shares(is_active) WHERE is_active = true;

-- Update triggers for password management
CREATE OR REPLACE FUNCTION update_password_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_passwords_updated_at 
    BEFORE UPDATE ON passwords 
    FOR EACH ROW EXECUTE FUNCTION update_password_updated_at();

CREATE TRIGGER update_password_folders_updated_at 
    BEFORE UPDATE ON password_folders 
    FOR EACH ROW EXECUTE FUNCTION update_password_updated_at();