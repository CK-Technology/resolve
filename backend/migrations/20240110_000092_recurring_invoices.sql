-- Recurring Invoices and Enhanced Billing Features
-- Phase 6: Invoicing & Billing Enhancements

-- Recurring invoice templates
CREATE TABLE IF NOT EXISTS recurring_invoice_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    contract_id UUID REFERENCES contracts(id) ON DELETE SET NULL,

    -- Template name and settings
    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Recurrence settings
    frequency VARCHAR(20) NOT NULL DEFAULT 'monthly', -- weekly, biweekly, monthly, quarterly, yearly
    interval_count INTEGER NOT NULL DEFAULT 1, -- every N frequency periods
    day_of_month INTEGER, -- for monthly (1-28, NULL = same day as start)
    day_of_week INTEGER, -- for weekly (0=Sunday, 6=Saturday)

    -- Schedule
    start_date DATE NOT NULL,
    end_date DATE, -- NULL = no end date
    next_run_date DATE NOT NULL,
    last_run_date DATE,

    -- Invoice defaults
    payment_terms VARCHAR(50) NOT NULL DEFAULT 'net_30',
    due_days INTEGER NOT NULL DEFAULT 30, -- days after invoice date
    notes TEXT,
    terms TEXT,

    -- Amounts (for fixed recurring charges)
    subtotal DECIMAL(15,2),
    tax_rate DECIMAL(5,2),

    -- Include unbilled time/expenses
    include_unbilled_time BOOLEAN DEFAULT true,
    include_unbilled_expenses BOOLEAN DEFAULT true,

    -- Email settings
    auto_send BOOLEAN DEFAULT false,
    send_reminder_days INTEGER[], -- e.g., {7, 3, 1} days before due

    -- Status
    is_active BOOLEAN DEFAULT true,
    run_count INTEGER DEFAULT 0,

    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_recurring_templates_client ON recurring_invoice_templates(client_id);
CREATE INDEX idx_recurring_templates_next_run ON recurring_invoice_templates(next_run_date) WHERE is_active = true;
CREATE INDEX idx_recurring_templates_active ON recurring_invoice_templates(is_active);

-- Recurring invoice line items (fixed items that repeat)
CREATE TABLE IF NOT EXISTS recurring_invoice_line_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES recurring_invoice_templates(id) ON DELETE CASCADE,
    description VARCHAR(500) NOT NULL,
    quantity DECIMAL(10,2) NOT NULL DEFAULT 1,
    unit_price DECIMAL(15,2) NOT NULL,
    tax_rate DECIMAL(5,2),
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_recurring_line_items_template ON recurring_invoice_line_items(template_id);

-- Track recurring invoice executions
CREATE TABLE IF NOT EXISTS recurring_invoice_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES recurring_invoice_templates(id) ON DELETE CASCADE,
    invoice_id UUID REFERENCES invoices(id) ON DELETE SET NULL,
    run_date DATE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'success', -- success, failed, skipped
    error_message TEXT,

    -- Stats for this run
    time_entries_count INTEGER DEFAULT 0,
    time_entries_amount DECIMAL(15,2) DEFAULT 0,
    fixed_items_amount DECIMAL(15,2) DEFAULT 0,
    total_amount DECIMAL(15,2) DEFAULT 0,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_recurring_runs_template ON recurring_invoice_runs(template_id);
CREATE INDEX idx_recurring_runs_invoice ON recurring_invoice_runs(invoice_id);
CREATE INDEX idx_recurring_runs_date ON recurring_invoice_runs(run_date DESC);

-- Link time entries to invoices (for time-to-invoice workflow)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'time_entries' AND column_name = 'invoice_id') THEN
        ALTER TABLE time_entries ADD COLUMN invoice_id UUID REFERENCES invoices(id) ON DELETE SET NULL;
        CREATE INDEX idx_time_entries_invoice ON time_entries(invoice_id);
    END IF;

    -- Add invoice_line_item_id for detailed tracking
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'time_entries' AND column_name = 'invoice_line_item_id') THEN
        ALTER TABLE time_entries ADD COLUMN invoice_line_item_id UUID REFERENCES invoice_line_items(id) ON DELETE SET NULL;
    END IF;
END $$;

-- Payment methods configuration
CREATE TABLE IF NOT EXISTS payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    type VARCHAR(50) NOT NULL, -- credit_card, bank_transfer, check, cash, paypal, stripe, etc.

    -- Integration settings (for online payments)
    provider VARCHAR(50), -- stripe, square, paypal, etc.
    provider_config JSONB, -- encrypted API keys, account IDs, etc.

    -- Display settings
    instructions TEXT, -- payment instructions for clients
    is_online BOOLEAN DEFAULT false, -- supports online payment
    is_default BOOLEAN DEFAULT false,

    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enhanced payments table (add more tracking)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'payments' AND column_name = 'payment_method_id') THEN
        ALTER TABLE payments ADD COLUMN payment_method_id UUID REFERENCES payment_methods(id);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'payments' AND column_name = 'transaction_id') THEN
        ALTER TABLE payments ADD COLUMN transaction_id VARCHAR(255); -- external payment processor ID
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'payments' AND column_name = 'status') THEN
        ALTER TABLE payments ADD COLUMN status VARCHAR(50) DEFAULT 'completed'; -- pending, completed, failed, refunded
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'payments' AND column_name = 'processed_by') THEN
        ALTER TABLE payments ADD COLUMN processed_by UUID REFERENCES users(id);
    END IF;
END $$;

-- Credit notes / refunds
CREATE TABLE IF NOT EXISTS credit_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    number VARCHAR(50) NOT NULL UNIQUE,
    client_id UUID NOT NULL REFERENCES clients(id),
    invoice_id UUID REFERENCES invoices(id), -- if credit is for specific invoice

    amount DECIMAL(15,2) NOT NULL,
    reason TEXT,

    -- Status tracking
    status VARCHAR(50) DEFAULT 'draft', -- draft, issued, applied, voided
    applied_amount DECIMAL(15,2) DEFAULT 0,
    remaining_amount DECIMAL(15,2), -- computed: amount - applied_amount

    issued_date DATE,
    issued_by UUID REFERENCES users(id),

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_credit_notes_client ON credit_notes(client_id);
CREATE INDEX idx_credit_notes_invoice ON credit_notes(invoice_id);
CREATE INDEX idx_credit_notes_status ON credit_notes(status);

-- Credit note applications (tracking how credits are used)
CREATE TABLE IF NOT EXISTS credit_note_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    credit_note_id UUID NOT NULL REFERENCES credit_notes(id) ON DELETE CASCADE,
    invoice_id UUID NOT NULL REFERENCES invoices(id),
    amount DECIMAL(15,2) NOT NULL,
    applied_at TIMESTAMPTZ DEFAULT NOW(),
    applied_by UUID REFERENCES users(id)
);

CREATE INDEX idx_credit_applications_credit ON credit_note_applications(credit_note_id);
CREATE INDEX idx_credit_applications_invoice ON credit_note_applications(invoice_id);

-- Invoice reminders (scheduled and sent)
CREATE TABLE IF NOT EXISTS invoice_reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    reminder_type VARCHAR(50) NOT NULL, -- upcoming_due, overdue, follow_up
    scheduled_date DATE NOT NULL,
    sent_at TIMESTAMPTZ,

    -- Email details
    email_template_id UUID REFERENCES email_templates(id),
    recipient_email VARCHAR(255),
    email_subject VARCHAR(500),
    email_body TEXT,

    status VARCHAR(50) DEFAULT 'scheduled', -- scheduled, sent, failed, cancelled
    error_message TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_invoice_reminders_invoice ON invoice_reminders(invoice_id);
CREATE INDEX idx_invoice_reminders_scheduled ON invoice_reminders(scheduled_date) WHERE status = 'scheduled';

-- Insert default payment methods
INSERT INTO payment_methods (name, type, instructions, is_default, display_order) VALUES
('Bank Transfer', 'bank_transfer', 'Please transfer to:\nBank: Example Bank\nAccount: XXXX-XXXX\nRouting: XXXXX\nReference: Invoice #', true, 1),
('Check', 'check', 'Please make checks payable to: [Company Name]', false, 2),
('Credit Card', 'credit_card', 'We accept Visa, Mastercard, and American Express', false, 3),
('Cash', 'cash', 'Cash payments accepted in person only', false, 4)
ON CONFLICT DO NOTHING;

-- Function to calculate next run date for recurring invoices
CREATE OR REPLACE FUNCTION calculate_next_run_date(
    p_frequency VARCHAR,
    p_interval_count INTEGER,
    p_current_date DATE,
    p_day_of_month INTEGER DEFAULT NULL,
    p_day_of_week INTEGER DEFAULT NULL
) RETURNS DATE AS $$
DECLARE
    next_date DATE;
BEGIN
    CASE p_frequency
        WHEN 'weekly' THEN
            next_date := p_current_date + (p_interval_count * INTERVAL '1 week');
            IF p_day_of_week IS NOT NULL THEN
                next_date := next_date + ((p_day_of_week - EXTRACT(DOW FROM next_date)::int + 7) % 7) * INTERVAL '1 day';
            END IF;

        WHEN 'biweekly' THEN
            next_date := p_current_date + (p_interval_count * INTERVAL '2 weeks');

        WHEN 'monthly' THEN
            next_date := p_current_date + (p_interval_count * INTERVAL '1 month');
            IF p_day_of_month IS NOT NULL THEN
                -- Clamp to valid day for the month
                next_date := make_date(
                    EXTRACT(YEAR FROM next_date)::int,
                    EXTRACT(MONTH FROM next_date)::int,
                    LEAST(p_day_of_month,
                          EXTRACT(DAY FROM (date_trunc('month', next_date) + INTERVAL '1 month - 1 day'))::int)
                );
            END IF;

        WHEN 'quarterly' THEN
            next_date := p_current_date + (p_interval_count * INTERVAL '3 months');

        WHEN 'yearly' THEN
            next_date := p_current_date + (p_interval_count * INTERVAL '1 year');

        ELSE
            next_date := p_current_date + INTERVAL '1 month'; -- default to monthly
    END CASE;

    RETURN next_date;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update remaining_amount on credit notes
CREATE OR REPLACE FUNCTION update_credit_note_remaining()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE credit_notes
    SET remaining_amount = amount - COALESCE((
        SELECT SUM(amount) FROM credit_note_applications WHERE credit_note_id = credit_notes.id
    ), 0),
    updated_at = NOW()
    WHERE id = NEW.credit_note_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS tr_update_credit_remaining ON credit_note_applications;
CREATE TRIGGER tr_update_credit_remaining
AFTER INSERT OR UPDATE OR DELETE ON credit_note_applications
FOR EACH ROW EXECUTE FUNCTION update_credit_note_remaining();

-- Comments
COMMENT ON TABLE recurring_invoice_templates IS 'Templates for generating recurring invoices automatically';
COMMENT ON TABLE recurring_invoice_runs IS 'History of recurring invoice generation runs';
COMMENT ON TABLE payment_methods IS 'Configured payment methods for the organization';
COMMENT ON TABLE credit_notes IS 'Credit notes and refunds for clients';
COMMENT ON TABLE invoice_reminders IS 'Scheduled and sent invoice payment reminders';
