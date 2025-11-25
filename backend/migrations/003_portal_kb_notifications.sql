-- =====================================================
-- CLIENT PORTAL ENHANCEMENTS
-- =====================================================

-- Portal access tokens for clients
CREATE TABLE IF NOT EXISTS portal_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_portal_contact FOREIGN KEY (contact_id) REFERENCES contacts(id)
);

-- Portal settings per client
CREATE TABLE IF NOT EXISTS portal_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    is_enabled BOOLEAN DEFAULT true,
    logo_url TEXT,
    primary_color VARCHAR(7) DEFAULT '#3B82F6',
    secondary_color VARCHAR(7) DEFAULT '#1E40AF',
    custom_domain TEXT,
    welcome_message TEXT,
    show_tickets BOOLEAN DEFAULT true,
    show_invoices BOOLEAN DEFAULT true,
    show_assets BOOLEAN DEFAULT true,
    show_knowledge_base BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    CONSTRAINT fk_portal_client FOREIGN KEY (client_id) REFERENCES clients(id)
);

-- =====================================================
-- KNOWLEDGE BASE SYSTEM
-- =====================================================

-- Knowledge base categories
CREATE TABLE IF NOT EXISTS kb_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES kb_categories(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    icon VARCHAR(50),
    display_order INTEGER DEFAULT 0,
    is_public BOOLEAN DEFAULT false,
    is_client_visible BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP
);

-- Enhance existing kb_articles table with additional columns
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS slug VARCHAR(500);
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS excerpt TEXT;
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS is_featured BOOLEAN DEFAULT false;
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS is_public BOOLEAN DEFAULT false;
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS is_client_visible BOOLEAN DEFAULT true;
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS view_count INTEGER DEFAULT 0;
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS not_helpful_count INTEGER DEFAULT 0;
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS tags TEXT[];
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS meta_keywords TEXT;
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS meta_description TEXT;
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS published_at TIMESTAMP;
ALTER TABLE kb_articles ADD COLUMN IF NOT EXISTS archived_at TIMESTAMP;

-- Update column types and constraints
ALTER TABLE kb_articles ALTER COLUMN title TYPE VARCHAR(500);
ALTER TABLE kb_articles ADD CONSTRAINT kb_articles_status_check CHECK (status IN ('draft', 'published', 'archived'));

-- Create unique index for slug
CREATE UNIQUE INDEX IF NOT EXISTS idx_kb_articles_slug ON kb_articles(slug) WHERE slug IS NOT NULL;

-- Article attachments
CREATE TABLE IF NOT EXISTS kb_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES kb_articles(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT,
    mime_type VARCHAR(100),
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Article feedback
CREATE TABLE IF NOT EXISTS kb_feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES kb_articles(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES contacts(id) ON DELETE SET NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    is_helpful BOOLEAN NOT NULL,
    feedback_text TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Article access restrictions (for client-specific articles)
CREATE TABLE IF NOT EXISTS kb_article_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES kb_articles(id) ON DELETE CASCADE,
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(article_id, client_id)
);

-- =====================================================
-- NOTIFICATION SYSTEM
-- =====================================================

-- Notification templates
CREATE TABLE IF NOT EXISTS notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    subject VARCHAR(500),
    email_template TEXT,
    sms_template TEXT,
    push_template TEXT,
    variables JSONB, -- Available variables for this template
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP
);

-- Notification rules
CREATE TABLE IF NOT EXISTS notification_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    trigger_type VARCHAR(100) NOT NULL, -- ticket_created, sla_breach, invoice_overdue, etc.
    trigger_conditions JSONB, -- Conditions that must be met
    template_id UUID REFERENCES notification_templates(id) ON DELETE SET NULL,
    channels TEXT[] DEFAULT ARRAY['email'], -- email, sms, push, in_app
    recipient_type VARCHAR(50), -- user, contact, role, specific
    recipient_ids UUID[],
    delay_minutes INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP
);

-- Notification queue
CREATE TABLE IF NOT EXISTS notification_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID REFERENCES notification_rules(id) ON DELETE SET NULL,
    recipient_type VARCHAR(50) NOT NULL, -- user or contact
    recipient_id UUID NOT NULL,
    channel VARCHAR(50) NOT NULL,
    subject VARCHAR(500),
    content TEXT NOT NULL,
    metadata JSONB,
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'failed', 'cancelled')),
    scheduled_for TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    sent_at TIMESTAMP,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Notification preferences
CREATE TABLE IF NOT EXISTS notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES contacts(id) ON DELETE CASCADE,
    channel VARCHAR(50) NOT NULL,
    notification_type VARCHAR(100) NOT NULL,
    is_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    CONSTRAINT notification_owner CHECK (
        (user_id IS NOT NULL AND contact_id IS NULL) OR 
        (user_id IS NULL AND contact_id IS NOT NULL)
    ),
    UNIQUE(user_id, channel, notification_type),
    UNIQUE(contact_id, channel, notification_type)
);

-- In-app notifications
CREATE TABLE IF NOT EXISTS in_app_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES contacts(id) ON DELETE CASCADE,
    type VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    message TEXT NOT NULL,
    action_url TEXT,
    icon VARCHAR(50),
    is_read BOOLEAN DEFAULT false,
    read_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT notification_recipient CHECK (
        (user_id IS NOT NULL AND contact_id IS NULL) OR 
        (user_id IS NULL AND contact_id IS NOT NULL)
    )
);

-- =====================================================
-- WEBSOCKET CONNECTIONS
-- =====================================================

CREATE TABLE IF NOT EXISTS websocket_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    connection_id VARCHAR(255) NOT NULL UNIQUE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES contacts(id) ON DELETE CASCADE,
    ip_address INET,
    user_agent TEXT,
    connected_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_ping_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    disconnected_at TIMESTAMP,
    CONSTRAINT ws_owner CHECK (
        (user_id IS NOT NULL AND contact_id IS NULL) OR 
        (user_id IS NULL AND contact_id IS NOT NULL)
    )
);

-- =====================================================
-- SEARCH INDEXES
-- =====================================================

-- Full text search indexes for knowledge base
CREATE INDEX IF NOT EXISTS idx_kb_articles_search ON kb_articles USING gin(
    to_tsvector('english', title || ' ' || COALESCE(content, '') || ' ' || COALESCE(summary, '') || ' ' || COALESCE(excerpt, ''))
);

CREATE INDEX IF NOT EXISTS idx_kb_articles_tags ON kb_articles USING gin(tags);
CREATE INDEX IF NOT EXISTS idx_kb_articles_status_published ON kb_articles(status) WHERE status = 'published';
CREATE INDEX IF NOT EXISTS idx_kb_articles_public ON kb_articles(is_public) WHERE is_public = true;

-- Notification indexes
CREATE INDEX idx_notification_queue_status ON notification_queue(status, scheduled_for) 
    WHERE status = 'pending';
CREATE INDEX idx_in_app_notifications_unread ON in_app_notifications(user_id, is_read) 
    WHERE is_read = false;
CREATE INDEX idx_in_app_notifications_contact ON in_app_notifications(contact_id, is_read) 
    WHERE is_read = false;

-- Portal indexes
CREATE INDEX idx_portal_tokens_contact ON portal_access_tokens(contact_id);
CREATE INDEX idx_portal_tokens_token ON portal_access_tokens(token);

-- =====================================================
-- DEFAULT NOTIFICATION TEMPLATES
-- =====================================================

INSERT INTO notification_templates (name, subject, email_template, variables) VALUES
('ticket_created', 'New Ticket Created: {{ticket_subject}}', 
 'A new ticket has been created:\n\nTicket #{{ticket_number}}\nSubject: {{ticket_subject}}\nPriority: {{ticket_priority}}\n\nView ticket: {{ticket_url}}',
 '{"ticket_number": "string", "ticket_subject": "string", "ticket_priority": "string", "ticket_url": "string"}'::jsonb),

('sla_breach_warning', 'SLA Warning: Ticket #{{ticket_number}}',
 'Ticket #{{ticket_number}} is approaching SLA breach:\n\nTime remaining: {{time_remaining}}\nClient: {{client_name}}\n\nView ticket: {{ticket_url}}',
 '{"ticket_number": "string", "time_remaining": "string", "client_name": "string", "ticket_url": "string"}'::jsonb),

('invoice_sent', 'Invoice #{{invoice_number}} from {{company_name}}',
 'Dear {{client_name}},\n\nInvoice #{{invoice_number}} for {{invoice_amount}} is now available.\nDue date: {{due_date}}\n\nView invoice: {{invoice_url}}',
 '{"invoice_number": "string", "invoice_amount": "string", "due_date": "string", "client_name": "string", "invoice_url": "string"}'::jsonb);