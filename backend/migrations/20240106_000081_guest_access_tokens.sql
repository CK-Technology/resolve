-- Guest Access Tokens for Client Portal Access
-- Allows clients to access their tickets and time entries without full portal login

CREATE TABLE guest_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_from_portal_token VARCHAR(255),
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_guest_access_tokens_client_id ON guest_access_tokens(client_id);
CREATE INDEX idx_guest_access_tokens_token ON guest_access_tokens(token);
CREATE INDEX idx_guest_access_tokens_expires_at ON guest_access_tokens(expires_at);

-- Clean up expired tokens periodically
CREATE OR REPLACE FUNCTION cleanup_expired_guest_tokens()
RETURNS void AS $$
BEGIN
    DELETE FROM guest_access_tokens WHERE expires_at < NOW() - INTERVAL '1 day';
END;
$$ LANGUAGE plpgsql;

-- Set up automatic cleanup (can be called by a scheduled job)
-- This function will be called by a cron job or similar scheduling system

-- Update timestamp trigger
CREATE OR REPLACE FUNCTION update_guest_access_tokens_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_guest_access_tokens_updated_at
    BEFORE UPDATE ON guest_access_tokens
    FOR EACH ROW
    EXECUTE FUNCTION update_guest_access_tokens_updated_at();

-- Grant appropriate permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON guest_access_tokens TO resolve_app;