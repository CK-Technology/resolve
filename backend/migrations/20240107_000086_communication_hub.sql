-- Communication Hub for Resolve
-- Email integration, SMS, client portal, internal chat, and unified messaging

-- Email accounts configuration
CREATE TABLE email_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email_address VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255),
    
    -- Server configuration
    imap_server VARCHAR(255) NOT NULL,
    imap_port INTEGER DEFAULT 993,
    imap_security VARCHAR(10) DEFAULT 'ssl', -- ssl, tls, none
    smtp_server VARCHAR(255) NOT NULL,
    smtp_port INTEGER DEFAULT 587,
    smtp_security VARCHAR(10) DEFAULT 'tls',
    
    -- Authentication
    username VARCHAR(255),
    password TEXT, -- Encrypted
    oauth_provider VARCHAR(50), -- gmail, outlook, etc
    oauth_token TEXT, -- Encrypted
    oauth_refresh_token TEXT, -- Encrypted
    oauth_expires_at TIMESTAMPTZ,
    
    -- Settings
    is_default BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    auto_create_tickets BOOLEAN DEFAULT true,
    signature TEXT,
    
    -- Monitoring
    last_sync TIMESTAMPTZ,
    sync_status VARCHAR(50) DEFAULT 'pending', -- pending, active, error, disabled
    error_message TEXT,
    emails_processed INTEGER DEFAULT 0,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Email processing rules
CREATE TABLE email_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_account_id UUID REFERENCES email_accounts(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    priority INTEGER DEFAULT 100,
    is_active BOOLEAN DEFAULT true,
    
    -- Conditions (all must match)
    from_contains TEXT[],
    from_not_contains TEXT[],
    subject_contains TEXT[],
    subject_not_contains TEXT[],
    body_contains TEXT[],
    to_addresses TEXT[],
    cc_addresses TEXT[],
    
    -- Actions
    action_type VARCHAR(50) NOT NULL, -- create_ticket, update_ticket, forward, ignore, create_lead
    assign_to_user_id UUID REFERENCES users(id),
    client_id UUID REFERENCES clients(id),
    ticket_priority VARCHAR(50),
    ticket_category_id UUID REFERENCES ticket_categories(id),
    forward_to_email VARCHAR(255),
    
    -- Auto-response
    send_auto_response BOOLEAN DEFAULT false,
    auto_response_template_id UUID,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Email templates
CREATE TABLE email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    category VARCHAR(100), -- ticket_update, welcome, invoice, reminder, auto_response
    subject VARCHAR(500) NOT NULL,
    html_body TEXT NOT NULL,
    text_body TEXT,
    variables JSONB DEFAULT '{}', -- Available template variables
    
    -- Usage tracking
    usage_count INTEGER DEFAULT 0,
    last_used TIMESTAMPTZ,
    
    is_active BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Unified message queue
CREATE TABLE message_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_type VARCHAR(50) NOT NULL, -- email, sms, push, webhook
    priority INTEGER DEFAULT 100,
    
    -- Recipients
    to_addresses TEXT[] NOT NULL,
    cc_addresses TEXT[],
    bcc_addresses TEXT[],
    from_address VARCHAR(255),
    
    -- Content
    subject VARCHAR(500),
    html_body TEXT,
    text_body TEXT,
    attachments JSONB DEFAULT '[]',
    
    -- Context
    client_id UUID REFERENCES clients(id),
    ticket_id UUID REFERENCES tickets(id),
    user_id UUID REFERENCES users(id),
    template_id UUID REFERENCES email_templates(id),
    
    -- Scheduling
    scheduled_at TIMESTAMPTZ DEFAULT NOW(),
    send_after TIMESTAMPTZ DEFAULT NOW(),
    
    -- Status tracking
    status VARCHAR(50) DEFAULT 'pending', -- pending, sending, sent, failed, cancelled
    attempts INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3,
    error_message TEXT,
    sent_at TIMESTAMPTZ,
    
    -- External references
    external_id VARCHAR(255),
    provider_response JSONB,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Email threads/conversations
CREATE TABLE email_threads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    thread_id VARCHAR(255) NOT NULL, -- Gmail/Outlook thread ID
    subject VARCHAR(500) NOT NULL,
    client_id UUID REFERENCES clients(id),
    contact_id UUID REFERENCES contacts(id),
    ticket_id UUID REFERENCES tickets(id),
    assigned_to UUID REFERENCES users(id),
    
    -- Thread metadata
    message_count INTEGER DEFAULT 1,
    participant_emails TEXT[],
    last_message_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Classification
    category VARCHAR(100), -- support, sales, billing, general
    sentiment VARCHAR(50), -- positive, neutral, negative
    priority VARCHAR(50) DEFAULT 'normal',
    
    -- Status
    status VARCHAR(50) DEFAULT 'active', -- active, closed, archived
    is_spam BOOLEAN DEFAULT false,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(thread_id)
);

-- Individual email messages
CREATE TABLE email_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    thread_id UUID REFERENCES email_threads(id) ON DELETE CASCADE,
    message_id VARCHAR(255) NOT NULL UNIQUE, -- Gmail/Outlook message ID
    
    -- Headers
    from_email VARCHAR(255) NOT NULL,
    from_name VARCHAR(255),
    to_emails TEXT[] NOT NULL,
    cc_emails TEXT[],
    bcc_emails TEXT[],
    reply_to VARCHAR(255),
    subject VARCHAR(500),
    
    -- Content
    html_body TEXT,
    text_body TEXT,
    snippet TEXT, -- Preview text
    
    -- Metadata
    received_at TIMESTAMPTZ NOT NULL,
    size_bytes INTEGER,
    has_attachments BOOLEAN DEFAULT false,
    attachments JSONB DEFAULT '[]',
    
    -- Processing
    is_inbound BOOLEAN DEFAULT true,
    is_read BOOLEAN DEFAULT false,
    is_spam BOOLEAN DEFAULT false,
    processed BOOLEAN DEFAULT false,
    processing_result JSONB,
    
    -- Raw data
    raw_headers JSONB,
    raw_message TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- SMS configuration and messages
CREATE TABLE sms_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    provider_type VARCHAR(50) NOT NULL, -- twilio, aws_sns, etc
    api_endpoint VARCHAR(500),
    api_key TEXT, -- Encrypted
    api_secret TEXT, -- Encrypted
    sender_number VARCHAR(20),
    is_active BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE sms_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID REFERENCES sms_providers(id),
    
    -- Recipients
    to_number VARCHAR(20) NOT NULL,
    from_number VARCHAR(20),
    
    -- Content
    message TEXT NOT NULL,
    media_urls TEXT[],
    
    -- Context
    client_id UUID REFERENCES clients(id),
    contact_id UUID REFERENCES contacts(id),
    ticket_id UUID REFERENCES tickets(id),
    user_id UUID REFERENCES users(id),
    
    -- Status
    status VARCHAR(50) DEFAULT 'pending', -- pending, sent, delivered, failed
    provider_id_external VARCHAR(255),
    error_message TEXT,
    cost DECIMAL(8,4), -- Cost in dollars
    
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Internal team chat/communication
CREATE TABLE chat_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    channel_type VARCHAR(50) DEFAULT 'general', -- general, client, project, incident
    
    -- References
    client_id UUID REFERENCES clients(id),
    project_id UUID REFERENCES projects(id),
    ticket_id UUID REFERENCES tickets(id),
    
    -- Settings
    is_private BOOLEAN DEFAULT false,
    is_archived BOOLEAN DEFAULT false,
    auto_archive_days INTEGER,
    
    -- Members
    members UUID[], -- Array of user IDs
    admins UUID[], -- Array of admin user IDs
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- Content
    message TEXT NOT NULL,
    message_type VARCHAR(50) DEFAULT 'text', -- text, file, image, system
    
    -- Threading
    parent_message_id UUID REFERENCES chat_messages(id),
    thread_count INTEGER DEFAULT 0,
    
    -- Attachments
    attachments JSONB DEFAULT '[]',
    
    -- Reactions/engagement
    reactions JSONB DEFAULT '{}', -- {emoji: [user_ids]}
    
    -- Editing
    edited_at TIMESTAMPTZ,
    original_message TEXT,
    
    -- Status
    is_deleted BOOLEAN DEFAULT false,
    deleted_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Notifications system
CREATE TABLE notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Email notifications
    email_tickets BOOLEAN DEFAULT true,
    email_mentions BOOLEAN DEFAULT true,
    email_projects BOOLEAN DEFAULT true,
    email_billing BOOLEAN DEFAULT false,
    email_reports BOOLEAN DEFAULT true,
    
    -- SMS notifications
    sms_enabled BOOLEAN DEFAULT false,
    sms_number VARCHAR(20),
    sms_critical_only BOOLEAN DEFAULT true,
    
    -- Push notifications
    push_enabled BOOLEAN DEFAULT true,
    push_device_tokens TEXT[],
    
    -- Chat notifications
    chat_mentions BOOLEAN DEFAULT true,
    chat_direct_messages BOOLEAN DEFAULT true,
    
    -- Frequency settings
    digest_frequency VARCHAR(50) DEFAULT 'daily', -- immediate, hourly, daily, weekly
    quiet_hours_start TIME DEFAULT '22:00',
    quiet_hours_end TIME DEFAULT '08:00',
    weekend_notifications BOOLEAN DEFAULT false,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id)
);

-- Client portal messaging
CREATE TABLE portal_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES contacts(id),
    ticket_id UUID REFERENCES tickets(id),
    
    -- Message details
    subject VARCHAR(500),
    message TEXT NOT NULL,
    message_type VARCHAR(50) DEFAULT 'general', -- general, support, billing
    priority VARCHAR(50) DEFAULT 'normal',
    
    -- Threading
    parent_message_id UUID REFERENCES portal_messages(id),
    
    -- Attachments
    attachments JSONB DEFAULT '[]',
    
    -- Status
    status VARCHAR(50) DEFAULT 'unread', -- unread, read, responded, closed
    is_internal_note BOOLEAN DEFAULT false,
    
    -- Response tracking
    response_due_at TIMESTAMPTZ,
    responded_at TIMESTAMPTZ,
    responded_by UUID REFERENCES users(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Communication analytics
CREATE TABLE communication_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_date DATE NOT NULL,
    client_id UUID REFERENCES clients(id),
    
    -- Email metrics
    emails_sent INTEGER DEFAULT 0,
    emails_received INTEGER DEFAULT 0,
    emails_processed INTEGER DEFAULT 0,
    avg_response_time_hours DECIMAL(8,2),
    
    -- SMS metrics
    sms_sent INTEGER DEFAULT 0,
    sms_delivered INTEGER DEFAULT 0,
    sms_failed INTEGER DEFAULT 0,
    sms_cost DECIMAL(8,2),
    
    -- Chat metrics
    chat_messages INTEGER DEFAULT 0,
    active_channels INTEGER DEFAULT 0,
    
    -- Portal metrics
    portal_messages INTEGER DEFAULT 0,
    portal_logins INTEGER DEFAULT 0,
    
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(metric_date, client_id)
);

-- Create indexes
CREATE INDEX idx_email_accounts_email ON email_accounts(email_address);
CREATE INDEX idx_email_rules_account_id ON email_rules(email_account_id);
CREATE INDEX idx_message_queue_status ON message_queue(status);
CREATE INDEX idx_message_queue_scheduled ON message_queue(scheduled_at);
CREATE INDEX idx_email_threads_client_id ON email_threads(client_id);
CREATE INDEX idx_email_threads_ticket_id ON email_threads(ticket_id);
CREATE INDEX idx_email_messages_thread_id ON email_messages(thread_id);
CREATE INDEX idx_email_messages_received ON email_messages(received_at);
CREATE INDEX idx_chat_messages_channel_id ON chat_messages(channel_id);
CREATE INDEX idx_chat_messages_created ON chat_messages(created_at);
CREATE INDEX idx_portal_messages_client_id ON portal_messages(client_id);
CREATE INDEX idx_portal_messages_status ON portal_messages(status);

-- Function to process email queue
CREATE OR REPLACE FUNCTION process_email_queue()
RETURNS void AS $$
DECLARE
    message_record RECORD;
BEGIN
    FOR message_record IN 
        SELECT * FROM message_queue 
        WHERE status = 'pending' 
        AND send_after <= NOW()
        AND attempts < max_attempts
        ORDER BY priority DESC, scheduled_at ASC
        LIMIT 100
    LOOP
        -- Update status to sending
        UPDATE message_queue 
        SET status = 'sending', attempts = attempts + 1
        WHERE id = message_record.id;
        
        -- Here would be the actual email sending logic
        -- For now, just mark as sent
        UPDATE message_queue 
        SET status = 'sent', sent_at = NOW()
        WHERE id = message_record.id;
        
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Function to create email thread from message
CREATE OR REPLACE FUNCTION create_email_thread(
    p_subject VARCHAR(500),
    p_client_id UUID,
    p_contact_id UUID DEFAULT NULL
) RETURNS UUID AS $$
DECLARE
    thread_uuid UUID;
BEGIN
    thread_uuid := gen_random_uuid();
    
    INSERT INTO email_threads (
        id, thread_id, subject, client_id, contact_id
    ) VALUES (
        thread_uuid, thread_uuid::text, p_subject, p_client_id, p_contact_id
    );
    
    RETURN thread_uuid;
END;
$$ LANGUAGE plpgsql;

-- Insert default email templates
INSERT INTO email_templates (name, category, subject, html_body, text_body, variables) VALUES
('Ticket Created', 'ticket_update', 'New Support Ticket #{{ticket_number}} - {{ticket_subject}}', 
'<h2>New Support Ticket Created</h2>
<p>Hello {{contact_name}},</p>
<p>We have received your support request and created ticket <strong>#{{ticket_number}}</strong>.</p>
<p><strong>Subject:</strong> {{ticket_subject}}</p>
<p><strong>Priority:</strong> {{priority}}</p>
<p><strong>Assigned to:</strong> {{assigned_to}}</p>
<p>We will respond within {{sla_response_time}} hours.</p>
<p>You can track the progress of your ticket at: <a href="{{portal_url}}">Client Portal</a></p>
<br>
<p>Best regards,<br>{{company_name}} Support Team</p>',
'New Support Ticket Created

Hello {{contact_name}},

We have received your support request and created ticket #{{ticket_number}}.

Subject: {{ticket_subject}}
Priority: {{priority}}
Assigned to: {{assigned_to}}

We will respond within {{sla_response_time}} hours.

You can track the progress at: {{portal_url}}

Best regards,
{{company_name}} Support Team',
'{"ticket_number": "string", "ticket_subject": "string", "contact_name": "string", "priority": "string", "assigned_to": "string", "sla_response_time": "number", "portal_url": "string", "company_name": "string"}'::jsonb),

('Ticket Resolved', 'ticket_update', 'Ticket #{{ticket_number}} Resolved - {{ticket_subject}}',
'<h2>Support Ticket Resolved</h2>
<p>Hello {{contact_name}},</p>
<p>We are pleased to inform you that ticket <strong>#{{ticket_number}}</strong> has been resolved.</p>
<p><strong>Subject:</strong> {{ticket_subject}}</p>
<p><strong>Resolution:</strong> {{resolution_notes}}</p>
<p><strong>Time to Resolution:</strong> {{resolution_time}}</p>
<p>If you have any questions or if this issue reoccurs, please don''t hesitate to contact us.</p>
<p>Rate your experience: <a href="{{feedback_url}}">Click here</a></p>
<br>
<p>Best regards,<br>{{company_name}} Support Team</p>',
'Support Ticket Resolved

Hello {{contact_name}},

Ticket #{{ticket_number}} has been resolved.

Subject: {{ticket_subject}}
Resolution: {{resolution_notes}}
Time to Resolution: {{resolution_time}}

If you have any questions, please contact us.
Rate your experience: {{feedback_url}}

Best regards,
{{company_name}} Support Team',
'{"ticket_number": "string", "ticket_subject": "string", "contact_name": "string", "resolution_notes": "text", "resolution_time": "string", "feedback_url": "string", "company_name": "string"}'::jsonb);

-- Insert default SMS provider placeholder
INSERT INTO sms_providers (name, provider_type, is_active) VALUES
('Twilio', 'twilio', false);