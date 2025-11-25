-- Ticket Queues, Canned Responses, and Enhanced Ticketing Features
-- Phase 3: Core Ticketing Enhancements

-- Ticket Queues for team-based routing
CREATE TABLE IF NOT EXISTS ticket_queues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    color VARCHAR(7) DEFAULT '#6b7280',
    icon VARCHAR(50) DEFAULT 'inbox',

    -- Queue settings
    email_address VARCHAR(255), -- incoming email creates tickets in this queue
    auto_assign BOOLEAN DEFAULT false,
    round_robin BOOLEAN DEFAULT false, -- round-robin assignment

    -- Default values for tickets in this queue
    default_priority VARCHAR(20) DEFAULT 'medium',
    default_sla_policy_id UUID REFERENCES sla_policies(id),
    default_category_id UUID REFERENCES ticket_categories(id),

    -- Access control
    is_private BOOLEAN DEFAULT false, -- only queue members can see tickets

    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_ticket_queues_active ON ticket_queues(is_active);
CREATE INDEX idx_ticket_queues_email ON ticket_queues(email_address);

-- Queue membership
CREATE TABLE IF NOT EXISTS ticket_queue_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    queue_id UUID NOT NULL REFERENCES ticket_queues(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member', -- member, manager, admin
    receive_notifications BOOLEAN DEFAULT true,
    can_assign BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(queue_id, user_id)
);

CREATE INDEX idx_queue_members_queue ON ticket_queue_members(queue_id);
CREATE INDEX idx_queue_members_user ON ticket_queue_members(user_id);

-- Add queue_id to tickets table
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tickets' AND column_name = 'queue_id') THEN
        ALTER TABLE tickets ADD COLUMN queue_id UUID REFERENCES ticket_queues(id);
        CREATE INDEX idx_tickets_queue_id ON tickets(queue_id);
    END IF;
END $$;

-- Ticket routing rules
CREATE TABLE IF NOT EXISTS ticket_routing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,

    -- Conditions (JSONB for flexibility)
    conditions JSONB NOT NULL, -- e.g., {"client_id": "...", "category_id": "...", "subject_contains": "..."}

    -- Actions
    assign_queue_id UUID REFERENCES ticket_queues(id),
    assign_user_id UUID REFERENCES users(id),
    set_priority VARCHAR(20),
    set_category_id UUID REFERENCES ticket_categories(id),
    add_tags TEXT[],

    -- Rule settings
    stop_processing BOOLEAN DEFAULT true, -- stop checking other rules if matched
    is_active BOOLEAN DEFAULT true,
    priority INTEGER DEFAULT 0, -- higher = checked first

    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_routing_rules_active ON ticket_routing_rules(is_active, priority DESC);

-- Canned responses / Quick replies
CREATE TABLE IF NOT EXISTS canned_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    shortcut VARCHAR(50), -- e.g., "/thanks" or "#thanks"

    -- Content
    subject VARCHAR(255), -- optional, for email subject line
    content TEXT NOT NULL,
    content_html TEXT, -- rich text version

    -- Categorization
    category VARCHAR(50), -- general, closing, greeting, technical, etc.
    tags TEXT[],

    -- Scope
    is_global BOOLEAN DEFAULT true, -- available to all users
    user_id UUID REFERENCES users(id), -- personal canned response
    queue_id UUID REFERENCES ticket_queues(id), -- queue-specific

    -- Variables available in this response
    variables JSONB DEFAULT '[]'::jsonb, -- ["client_name", "ticket_number", etc.]

    -- Usage tracking
    usage_count INTEGER DEFAULT 0,
    last_used_at TIMESTAMPTZ,

    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_canned_responses_shortcut ON canned_responses(shortcut) WHERE shortcut IS NOT NULL;
CREATE INDEX idx_canned_responses_category ON canned_responses(category);
CREATE INDEX idx_canned_responses_user ON canned_responses(user_id);
CREATE INDEX idx_canned_responses_queue ON canned_responses(queue_id);
CREATE INDEX idx_canned_responses_global ON canned_responses(is_global) WHERE is_global = true;

-- Ticket links (parent/child, related, duplicate, blocked by)
CREATE TABLE IF NOT EXISTS ticket_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    target_ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    link_type VARCHAR(50) NOT NULL, -- parent, child, related, duplicate, blocks, blocked_by
    notes TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(source_ticket_id, target_ticket_id, link_type),
    CHECK(source_ticket_id != target_ticket_id)
);

CREATE INDEX idx_ticket_links_source ON ticket_links(source_ticket_id);
CREATE INDEX idx_ticket_links_target ON ticket_links(target_ticket_id);
CREATE INDEX idx_ticket_links_type ON ticket_links(link_type);

-- Ticket merge history
CREATE TABLE IF NOT EXISTS ticket_merges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    primary_ticket_id UUID NOT NULL REFERENCES tickets(id), -- the ticket that remains
    merged_ticket_id UUID NOT NULL, -- the ticket that was merged (soft deleted)
    merged_ticket_number INTEGER NOT NULL, -- preserve the original number
    merged_ticket_subject VARCHAR(500),
    merge_reason TEXT,
    merged_by UUID REFERENCES users(id),
    merged_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_ticket_merges_primary ON ticket_merges(primary_ticket_id);
CREATE INDEX idx_ticket_merges_merged ON ticket_merges(merged_ticket_id);

-- Add merged_into field to tickets
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tickets' AND column_name = 'merged_into_id') THEN
        ALTER TABLE tickets ADD COLUMN merged_into_id UUID REFERENCES tickets(id);
        ALTER TABLE tickets ADD COLUMN is_merged BOOLEAN DEFAULT false;
        CREATE INDEX idx_tickets_merged ON tickets(is_merged) WHERE is_merged = true;
    END IF;
END $$;

-- Ticket tags
CREATE TABLE IF NOT EXISTS ticket_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    color VARCHAR(7) DEFAULT '#6b7280',
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ticket_tag_assignments (
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES ticket_tags(id) ON DELETE CASCADE,
    added_by UUID REFERENCES users(id),
    added_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (ticket_id, tag_id)
);

CREATE INDEX idx_ticket_tag_assignments_ticket ON ticket_tag_assignments(ticket_id);
CREATE INDEX idx_ticket_tag_assignments_tag ON ticket_tag_assignments(tag_id);

-- Teams/Slack integration settings
CREATE TABLE IF NOT EXISTS notification_integrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    integration_type VARCHAR(50) NOT NULL, -- teams, slack, discord, webhook

    -- Connection settings
    webhook_url TEXT,
    api_token_encrypted TEXT,
    channel_id VARCHAR(255),

    -- Notification settings
    notify_on JSONB DEFAULT '{"ticket_created": true, "ticket_assigned": true, "ticket_resolved": true, "sla_breach": true}'::jsonb,

    -- Filtering
    queue_ids UUID[], -- only notify for these queues (empty = all)
    priority_filter TEXT[], -- only notify for these priorities

    -- Message templates
    message_templates JSONB,

    is_active BOOLEAN DEFAULT true,
    last_notification_at TIMESTAMPTZ,
    error_count INTEGER DEFAULT 0,
    last_error TEXT,

    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_notification_integrations_type ON notification_integrations(integration_type);
CREATE INDEX idx_notification_integrations_active ON notification_integrations(is_active);

-- Notification log
CREATE TABLE IF NOT EXISTS notification_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID REFERENCES notification_integrations(id) ON DELETE SET NULL,
    notification_type VARCHAR(50) NOT NULL,
    ticket_id UUID REFERENCES tickets(id) ON DELETE SET NULL,

    -- Payload and response
    payload JSONB,
    response_status INTEGER,
    response_body TEXT,

    success BOOLEAN NOT NULL,
    error_message TEXT,

    sent_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_notification_log_integration ON notification_log(integration_id);
CREATE INDEX idx_notification_log_ticket ON notification_log(ticket_id);
CREATE INDEX idx_notification_log_sent ON notification_log(sent_at DESC);

-- Insert default queues
INSERT INTO ticket_queues (name, description, color, icon, default_priority) VALUES
('Support', 'General support requests', '#3b82f6', 'headphones', 'medium'),
('Urgent', 'Critical and urgent issues', '#dc2626', 'alert-triangle', 'critical'),
('Projects', 'Project-related tickets', '#10b981', 'folder', 'medium'),
('Billing', 'Billing and invoice inquiries', '#8b5cf6', 'credit-card', 'low'),
('Onboarding', 'New client and user setup', '#f59e0b', 'user-plus', 'medium')
ON CONFLICT DO NOTHING;

-- Insert default canned responses
INSERT INTO canned_responses (name, shortcut, content, content_html, category, is_global, variables) VALUES
(
    'Greeting',
    '/hi',
    'Hello {{client_name}},

Thank you for contacting support. I will be assisting you with this request.

',
    '<p>Hello {{client_name}},</p><p>Thank you for contacting support. I will be assisting you with this request.</p>',
    'greeting',
    true,
    '["client_name"]'::jsonb
),
(
    'Thank You - Resolved',
    '/thanks',
    'Thank you for your patience while we resolved this issue. If you have any further questions, please don''t hesitate to reach out.

Best regards,
{{technician_name}}',
    '<p>Thank you for your patience while we resolved this issue. If you have any further questions, please don''t hesitate to reach out.</p><p>Best regards,<br>{{technician_name}}</p>',
    'closing',
    true,
    '["technician_name"]'::jsonb
),
(
    'Awaiting Information',
    '/waiting',
    'Hi {{client_name}},

In order to proceed with your request, we need the following information:

[Please specify what information is needed]

Please reply to this ticket with the requested details.

Thank you,
{{technician_name}}',
    '<p>Hi {{client_name}},</p><p>In order to proceed with your request, we need the following information:</p><p>[Please specify what information is needed]</p><p>Please reply to this ticket with the requested details.</p><p>Thank you,<br>{{technician_name}}</p>',
    'general',
    true,
    '["client_name", "technician_name"]'::jsonb
),
(
    'Scheduled Maintenance',
    '/maintenance',
    'Hi {{client_name}},

We have scheduled maintenance for this issue. The work will be performed:

Date: [DATE]
Time: [TIME]
Expected Duration: [DURATION]

We will notify you once the work is complete.

Thank you,
{{technician_name}}',
    '<p>Hi {{client_name}},</p><p>We have scheduled maintenance for this issue. The work will be performed:</p><ul><li>Date: [DATE]</li><li>Time: [TIME]</li><li>Expected Duration: [DURATION]</li></ul><p>We will notify you once the work is complete.</p><p>Thank you,<br>{{technician_name}}</p>',
    'general',
    true,
    '["client_name", "technician_name"]'::jsonb
),
(
    'Password Reset Instructions',
    '/pwreset',
    'Hi {{client_name}},

To reset your password, please follow these steps:

1. Go to the login page
2. Click "Forgot Password"
3. Enter your email address
4. Check your email for the reset link
5. Follow the link to create a new password

If you don''t receive the email within 5 minutes, please check your spam folder.

Let me know if you need further assistance.

{{technician_name}}',
    NULL,
    'technical',
    true,
    '["client_name", "technician_name"]'::jsonb
),
(
    'Remote Session Required',
    '/remote',
    'Hi {{client_name}},

To resolve this issue, we''ll need to connect to your computer remotely.

Please click the following link when you''re ready:
[REMOTE SESSION LINK]

Make sure to save any open work before we connect.

Reply to this ticket when you''re ready, and we''ll initiate the session.

{{technician_name}}',
    NULL,
    'technical',
    true,
    '["client_name", "technician_name"]'::jsonb
)
ON CONFLICT DO NOTHING;

-- Insert default tags
INSERT INTO ticket_tags (name, color, description) VALUES
('urgent', '#dc2626', 'Requires immediate attention'),
('billing', '#8b5cf6', 'Related to billing or invoices'),
('security', '#ef4444', 'Security-related issue'),
('network', '#f59e0b', 'Network or connectivity issue'),
('hardware', '#6b7280', 'Hardware problem'),
('software', '#3b82f6', 'Software issue'),
('email', '#10b981', 'Email-related issue'),
('new-user', '#ec4899', 'New user setup'),
('vip', '#fbbf24', 'VIP client')
ON CONFLICT DO NOTHING;

-- Comments
COMMENT ON TABLE ticket_queues IS 'Queues for organizing and routing tickets to teams';
COMMENT ON TABLE canned_responses IS 'Pre-written responses for quick ticket replies';
COMMENT ON TABLE ticket_links IS 'Links between related tickets (parent/child, duplicate, etc.)';
COMMENT ON TABLE ticket_merges IS 'History of merged tickets';
COMMENT ON TABLE notification_integrations IS 'External notification integrations (Teams, Slack, etc.)';
