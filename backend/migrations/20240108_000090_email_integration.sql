-- Email Integration Tables
-- Provides email-to-ticket functionality, mailbox management, and email templates

-- Email Mailboxes for email-to-ticket
CREATE TABLE IF NOT EXISTS email_mailboxes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email_address VARCHAR(255) NOT NULL UNIQUE,
    mailbox_type VARCHAR(50) NOT NULL DEFAULT 'support', -- support, sales, billing, general

    -- IMAP Configuration
    imap_host VARCHAR(255) NOT NULL,
    imap_port INTEGER NOT NULL DEFAULT 993,
    imap_username VARCHAR(255) NOT NULL,
    imap_password TEXT NOT NULL, -- encrypted
    imap_folder VARCHAR(255) NOT NULL DEFAULT 'INBOX',
    use_tls BOOLEAN NOT NULL DEFAULT true,

    -- Processing settings
    is_active BOOLEAN NOT NULL DEFAULT false,
    poll_interval_secs INTEGER NOT NULL DEFAULT 60,

    -- Ticket defaults
    default_queue_id UUID REFERENCES ticket_queues(id) ON DELETE SET NULL,
    default_priority VARCHAR(50) NOT NULL DEFAULT 'medium',

    -- Status tracking
    last_checked_at TIMESTAMPTZ,
    last_error TEXT,
    processed_count INTEGER DEFAULT 0,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_email_mailboxes_active ON email_mailboxes(is_active) WHERE is_active = true;
CREATE INDEX idx_email_mailboxes_email ON email_mailboxes(email_address);

-- Email Templates
CREATE TABLE IF NOT EXISTS email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    subject VARCHAR(500) NOT NULL,
    html_body TEXT NOT NULL,
    text_body TEXT,
    description TEXT,

    -- Template category
    category VARCHAR(50) DEFAULT 'general', -- ticket, invoice, alert, notification

    -- Available variables for this template
    variables JSONB DEFAULT '[]'::jsonb,

    -- System templates cannot be deleted
    is_system BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_email_templates_slug ON email_templates(slug);
CREATE INDEX idx_email_templates_category ON email_templates(category);

-- Email Send Log
CREATE TABLE IF NOT EXISTS email_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Recipient
    to_email VARCHAR(255) NOT NULL,
    to_name VARCHAR(255),

    -- Email content
    subject VARCHAR(500) NOT NULL,
    template_id UUID REFERENCES email_templates(id) ON DELETE SET NULL,

    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, sent, failed
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,

    -- Context
    related_type VARCHAR(50), -- ticket, invoice, client, user
    related_id UUID,

    -- Timestamps
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_logs_status ON email_logs(status);
CREATE INDEX idx_email_logs_related ON email_logs(related_type, related_id);
CREATE INDEX idx_email_logs_created ON email_logs(created_at DESC);

-- Processed Emails (to prevent reprocessing)
CREATE TABLE IF NOT EXISTS processed_emails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mailbox_id UUID NOT NULL REFERENCES email_mailboxes(id) ON DELETE CASCADE,
    message_id VARCHAR(500) NOT NULL, -- Email Message-ID header

    -- Result
    result_type VARCHAR(50) NOT NULL, -- ticket_created, reply_added, ignored, failed
    ticket_id UUID REFERENCES tickets(id) ON DELETE SET NULL,

    -- Email metadata
    from_email VARCHAR(255) NOT NULL,
    from_name VARCHAR(255),
    subject VARCHAR(500),
    received_at TIMESTAMPTZ,

    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(mailbox_id, message_id)
);

CREATE INDEX idx_processed_emails_mailbox ON processed_emails(mailbox_id);
CREATE INDEX idx_processed_emails_ticket ON processed_emails(ticket_id);
CREATE INDEX idx_processed_emails_message_id ON processed_emails(message_id);

-- Insert default system templates
INSERT INTO email_templates (id, name, slug, subject, html_body, text_body, category, is_system, variables) VALUES
(
    gen_random_uuid(),
    'New Ticket Created',
    'ticket_created',
    '[Ticket #{{ticket_number}}] {{subject}}',
    '<html>
    <head><style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 600px; margin: 0 auto; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .header { background: #2563eb; color: white; padding: 20px; text-align: center; }
        .content { padding: 30px; }
        .ticket-info { background: #f8fafc; border-left: 4px solid #2563eb; padding: 15px; margin: 20px 0; }
        .footer { background: #f8fafc; padding: 20px; text-align: center; color: #666; }
        .btn { display: inline-block; background: #2563eb; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin: 10px 0; }
    </style></head>
    <body>
        <div class="container">
            <div class="header"><h1>Support Ticket Created</h1></div>
            <div class="content">
                <p>Hello {{client_name}},</p>
                <p>Your support ticket has been created.</p>
                <div class="ticket-info">
                    <p><strong>Ticket #:</strong> {{ticket_number}}</p>
                    <p><strong>Subject:</strong> {{subject}}</p>
                    <p><strong>Priority:</strong> {{priority}}</p>
                </div>
                <a href="{{portal_url}}" class="btn">View Ticket</a>
            </div>
            <div class="footer"><p>Thank you for contacting support.</p></div>
        </div>
    </body>
    </html>',
    'Your support ticket #{{ticket_number}} has been created.\n\nSubject: {{subject}}\nPriority: {{priority}}\n\nView ticket: {{portal_url}}',
    'ticket',
    true,
    '["ticket_number", "subject", "priority", "client_name", "portal_url"]'::jsonb
),
(
    gen_random_uuid(),
    'Ticket Updated',
    'ticket_updated',
    '[Ticket #{{ticket_number}}] Update: {{subject}}',
    '<html>
    <head><style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 600px; margin: 0 auto; background: white; border-radius: 8px; overflow: hidden; }
        .header { background: #059669; color: white; padding: 20px; text-align: center; }
        .content { padding: 30px; }
        .update { background: #f0fdf4; border-left: 4px solid #059669; padding: 15px; margin: 20px 0; }
        .btn { display: inline-block; background: #059669; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; }
    </style></head>
    <body>
        <div class="container">
            <div class="header"><h1>Ticket Updated</h1></div>
            <div class="content">
                <p>Hello {{client_name}},</p>
                <p>Your ticket #{{ticket_number}} has been updated.</p>
                <div class="update">
                    <p><strong>Status:</strong> {{status}}</p>
                    <p>{{update_message}}</p>
                </div>
                <a href="{{portal_url}}" class="btn">View Ticket</a>
            </div>
        </div>
    </body>
    </html>',
    'Your ticket #{{ticket_number}} has been updated.\n\nStatus: {{status}}\n\n{{update_message}}\n\nView ticket: {{portal_url}}',
    'ticket',
    true,
    '["ticket_number", "subject", "status", "update_message", "client_name", "portal_url"]'::jsonb
),
(
    gen_random_uuid(),
    'Ticket Resolved',
    'ticket_resolved',
    '[Ticket #{{ticket_number}}] Resolved: {{subject}}',
    '<html>
    <head><style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 600px; margin: 0 auto; background: white; border-radius: 8px; overflow: hidden; }
        .header { background: #16a34a; color: white; padding: 20px; text-align: center; }
        .content { padding: 30px; }
        .btn { display: inline-block; background: #16a34a; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; }
    </style></head>
    <body>
        <div class="container">
            <div class="header"><h1>Ticket Resolved</h1></div>
            <div class="content">
                <p>Hello {{client_name}},</p>
                <p>Your support ticket #{{ticket_number}} has been resolved.</p>
                <p><strong>Resolution:</strong></p>
                <p>{{resolution}}</p>
                <p>If you have any further questions, you can reopen this ticket by replying to this email.</p>
                <a href="{{portal_url}}" class="btn">View Ticket</a>
            </div>
        </div>
    </body>
    </html>',
    'Your ticket #{{ticket_number}} has been resolved.\n\nResolution: {{resolution}}\n\nView ticket: {{portal_url}}',
    'ticket',
    true,
    '["ticket_number", "subject", "resolution", "client_name", "portal_url"]'::jsonb
),
(
    gen_random_uuid(),
    'Invoice Sent',
    'invoice_sent',
    'Invoice #{{invoice_number}} from {{company_name}}',
    '<html>
    <head><style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 600px; margin: 0 auto; background: white; border-radius: 8px; overflow: hidden; }
        .header { background: #7c3aed; color: white; padding: 20px; text-align: center; }
        .content { padding: 30px; }
        .invoice-details { background: #f8fafc; padding: 15px; margin: 20px 0; }
        .amount { font-size: 24px; color: #7c3aed; font-weight: bold; }
        .btn { display: inline-block; background: #7c3aed; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; }
    </style></head>
    <body>
        <div class="container">
            <div class="header"><h1>Invoice</h1></div>
            <div class="content">
                <p>Hello {{client_name}},</p>
                <p>Please find your invoice attached.</p>
                <div class="invoice-details">
                    <p><strong>Invoice #:</strong> {{invoice_number}}</p>
                    <p><strong>Date:</strong> {{invoice_date}}</p>
                    <p><strong>Due Date:</strong> {{due_date}}</p>
                    <p class="amount">Amount Due: ${{amount}}</p>
                </div>
                <a href="{{portal_url}}" class="btn">View Invoice</a>
            </div>
        </div>
    </body>
    </html>',
    'Invoice #{{invoice_number}} - Amount Due: ${{amount}}\n\nDue Date: {{due_date}}\n\nView invoice: {{portal_url}}',
    'invoice',
    true,
    '["invoice_number", "invoice_date", "due_date", "amount", "client_name", "company_name", "portal_url"]'::jsonb
),
(
    gen_random_uuid(),
    'Password Reset',
    'password_reset',
    'Reset Your Password',
    '<html>
    <head><style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 600px; margin: 0 auto; background: white; border-radius: 8px; overflow: hidden; }
        .header { background: #dc2626; color: white; padding: 20px; text-align: center; }
        .content { padding: 30px; }
        .btn { display: inline-block; background: #dc2626; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; }
        .warning { color: #dc2626; font-size: 12px; margin-top: 20px; }
    </style></head>
    <body>
        <div class="container">
            <div class="header"><h1>Password Reset</h1></div>
            <div class="content">
                <p>Hello {{user_name}},</p>
                <p>We received a request to reset your password.</p>
                <p>Click the button below to reset your password:</p>
                <a href="{{reset_url}}" class="btn">Reset Password</a>
                <p class="warning">This link will expire in 1 hour. If you did not request this, please ignore this email.</p>
            </div>
        </div>
    </body>
    </html>',
    'Password Reset Request\n\nClick here to reset your password: {{reset_url}}\n\nThis link expires in 1 hour.',
    'notification',
    true,
    '["user_name", "reset_url"]'::jsonb
);

-- Comments
COMMENT ON TABLE email_mailboxes IS 'Email mailboxes for email-to-ticket integration';
COMMENT ON TABLE email_templates IS 'Reusable email templates with variable substitution';
COMMENT ON TABLE email_logs IS 'Log of all sent emails for auditing';
COMMENT ON TABLE processed_emails IS 'Track processed emails to prevent duplicates';
