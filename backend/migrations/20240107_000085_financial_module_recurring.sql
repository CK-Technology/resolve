-- Enhanced Financial Module with Recurring Billing for Resolve
-- Automated charges, expense tracking, profitability, budgets, payment portal

-- Payment methods for clients
CREATE TABLE payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL, -- credit_card, ach, wire, check, paypal, stripe
    is_default BOOLEAN DEFAULT false,
    
    -- Card details (encrypted)
    card_last_four VARCHAR(4),
    card_brand VARCHAR(50),
    card_exp_month INTEGER,
    card_exp_year INTEGER,
    card_holder_name VARCHAR(255),
    
    -- Bank details (encrypted)
    bank_name VARCHAR(255),
    account_last_four VARCHAR(4),
    routing_number_encrypted TEXT,
    account_type VARCHAR(50), -- checking, savings
    
    -- External payment
    stripe_payment_method_id VARCHAR(255),
    paypal_email VARCHAR(255),
    
    billing_address TEXT,
    is_verified BOOLEAN DEFAULT false,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Recurring billing profiles
CREATE TABLE recurring_billing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Billing details
    billing_type VARCHAR(50) NOT NULL, -- fixed, usage_based, tiered, per_user
    amount DECIMAL(10,2),
    quantity INTEGER DEFAULT 1,
    unit_price DECIMAL(10,2),
    
    -- Frequency
    frequency VARCHAR(50) NOT NULL, -- daily, weekly, monthly, quarterly, annually
    billing_day INTEGER, -- Day of month/week for billing
    
    -- Duration
    start_date DATE NOT NULL,
    end_date DATE,
    next_billing_date DATE NOT NULL,
    last_billed_date DATE,
    
    -- Payment
    payment_method_id UUID REFERENCES payment_methods(id),
    auto_charge BOOLEAN DEFAULT true,
    send_invoice BOOLEAN DEFAULT true,
    payment_terms_days INTEGER DEFAULT 30,
    
    -- Status
    status VARCHAR(50) DEFAULT 'active', -- active, paused, cancelled, expired
    pause_reason TEXT,
    cancel_reason TEXT,
    
    -- Metadata
    contract_id UUID REFERENCES contracts(id),
    service_id UUID,
    tags TEXT[] DEFAULT '{}',
    custom_fields JSONB DEFAULT '{}',
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Recurring billing line items
CREATE TABLE recurring_billing_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recurring_billing_id UUID NOT NULL REFERENCES recurring_billing(id) ON DELETE CASCADE,
    item_type VARCHAR(50) NOT NULL, -- service, product, license, support
    name VARCHAR(255) NOT NULL,
    description TEXT,
    quantity DECIMAL(10,2) DEFAULT 1,
    unit_price DECIMAL(10,2) NOT NULL,
    discount_percent DECIMAL(5,2) DEFAULT 0,
    tax_rate DECIMAL(5,2) DEFAULT 0,
    total DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Expense tracking
CREATE TABLE expenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id),
    project_id UUID REFERENCES projects(id),
    category VARCHAR(100) NOT NULL, -- hardware, software, travel, services, utilities
    vendor VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    amount DECIMAL(10,2) NOT NULL,
    tax_amount DECIMAL(10,2) DEFAULT 0,
    expense_date DATE NOT NULL,
    
    -- Reimbursement
    is_billable BOOLEAN DEFAULT false,
    is_reimbursable BOOLEAN DEFAULT false,
    markup_percent DECIMAL(5,2) DEFAULT 0,
    billed_amount DECIMAL(10,2),
    invoice_id UUID REFERENCES invoices(id),
    
    -- Payment info
    payment_method VARCHAR(50), -- cash, credit_card, check, transfer
    receipt_url VARCHAR(500),
    
    -- Allocation
    allocated_to UUID REFERENCES users(id),
    department VARCHAR(100),
    cost_center VARCHAR(100),
    
    -- Approval workflow
    requires_approval BOOLEAN DEFAULT false,
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    approval_notes TEXT,
    
    -- Status
    status VARCHAR(50) DEFAULT 'pending', -- pending, approved, rejected, reimbursed
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Client budgets
CREATE TABLE client_budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    budget_type VARCHAR(50) NOT NULL, -- monthly, quarterly, annual, project
    category VARCHAR(100), -- support, projects, hardware, software
    amount DECIMAL(10,2) NOT NULL,
    
    -- Period
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    
    -- Tracking
    spent_amount DECIMAL(10,2) DEFAULT 0,
    committed_amount DECIMAL(10,2) DEFAULT 0,
    remaining_amount DECIMAL(10,2),
    
    -- Alerts
    alert_threshold_percent INTEGER DEFAULT 80,
    alert_sent BOOLEAN DEFAULT false,
    alert_sent_at TIMESTAMPTZ,
    
    -- Status
    status VARCHAR(50) DEFAULT 'active', -- active, exceeded, closed
    notes TEXT,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Profitability tracking
CREATE TABLE client_profitability (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    
    -- Revenue
    recurring_revenue DECIMAL(10,2) DEFAULT 0,
    project_revenue DECIMAL(10,2) DEFAULT 0,
    service_revenue DECIMAL(10,2) DEFAULT 0,
    product_revenue DECIMAL(10,2) DEFAULT 0,
    total_revenue DECIMAL(10,2) DEFAULT 0,
    
    -- Costs
    labor_cost DECIMAL(10,2) DEFAULT 0,
    expense_cost DECIMAL(10,2) DEFAULT 0,
    overhead_cost DECIMAL(10,2) DEFAULT 0,
    total_cost DECIMAL(10,2) DEFAULT 0,
    
    -- Metrics
    gross_profit DECIMAL(10,2) DEFAULT 0,
    gross_margin_percent DECIMAL(5,2) DEFAULT 0,
    net_profit DECIMAL(10,2) DEFAULT 0,
    net_margin_percent DECIMAL(5,2) DEFAULT 0,
    
    -- Time metrics
    hours_worked DECIMAL(10,2) DEFAULT 0,
    billable_hours DECIMAL(10,2) DEFAULT 0,
    utilization_rate DECIMAL(5,2) DEFAULT 0,
    effective_hourly_rate DECIMAL(10,2) DEFAULT 0,
    
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(client_id, period_start, period_end)
);

-- Payment transactions
CREATE TABLE payment_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id),
    invoice_id UUID REFERENCES invoices(id),
    payment_method_id UUID REFERENCES payment_methods(id),
    
    -- Transaction details
    transaction_type VARCHAR(50) NOT NULL, -- payment, refund, credit, charge
    amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    
    -- Processing
    processor VARCHAR(50), -- stripe, paypal, manual, check, wire
    processor_transaction_id VARCHAR(255),
    processor_fee DECIMAL(10,2) DEFAULT 0,
    
    -- Status
    status VARCHAR(50) NOT NULL, -- pending, processing, completed, failed, refunded
    status_message TEXT,
    
    -- Dates
    initiated_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    settled_at TIMESTAMPTZ,
    
    -- References
    reference_number VARCHAR(255),
    check_number VARCHAR(50),
    notes TEXT,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Payment portal sessions
CREATE TABLE payment_portal_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id),
    contact_id UUID NOT NULL REFERENCES contacts(id),
    session_token VARCHAR(255) UNIQUE NOT NULL,
    
    -- Access control
    ip_address INET,
    user_agent TEXT,
    
    -- Permissions
    can_view_invoices BOOLEAN DEFAULT true,
    can_make_payments BOOLEAN DEFAULT true,
    can_update_payment_methods BOOLEAN DEFAULT false,
    can_download_statements BOOLEAN DEFAULT true,
    
    -- Session management
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Credit notes / adjustments
CREATE TABLE credit_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id),
    invoice_id UUID REFERENCES invoices(id),
    credit_note_number VARCHAR(50) UNIQUE NOT NULL,
    
    -- Details
    reason VARCHAR(255) NOT NULL,
    description TEXT,
    amount DECIMAL(10,2) NOT NULL,
    
    -- Application
    applied_amount DECIMAL(10,2) DEFAULT 0,
    remaining_amount DECIMAL(10,2),
    
    -- Status
    status VARCHAR(50) DEFAULT 'draft', -- draft, issued, partially_applied, fully_applied, void
    issue_date DATE,
    void_reason TEXT,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Billing automation rules
CREATE TABLE billing_automation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    rule_type VARCHAR(50) NOT NULL, -- time_based, ticket_closed, project_complete, usage_threshold
    
    -- Triggers
    trigger_conditions JSONB NOT NULL,
    
    -- Actions
    action_type VARCHAR(50) NOT NULL, -- create_invoice, add_line_item, send_reminder, apply_late_fee
    action_parameters JSONB NOT NULL,
    
    -- Scope
    apply_to_all_clients BOOLEAN DEFAULT false,
    client_ids UUID[] DEFAULT '{}',
    
    -- Execution
    is_active BOOLEAN DEFAULT true,
    last_executed TIMESTAMPTZ,
    execution_count INTEGER DEFAULT 0,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Statements for client portal
CREATE TABLE billing_statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id),
    statement_number VARCHAR(50) UNIQUE NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    
    -- Summary
    opening_balance DECIMAL(10,2) DEFAULT 0,
    total_charges DECIMAL(10,2) DEFAULT 0,
    total_payments DECIMAL(10,2) DEFAULT 0,
    total_credits DECIMAL(10,2) DEFAULT 0,
    closing_balance DECIMAL(10,2) DEFAULT 0,
    
    -- Details (JSONB for line items)
    line_items JSONB DEFAULT '[]',
    
    -- Status
    status VARCHAR(50) DEFAULT 'draft', -- draft, final, sent
    sent_at TIMESTAMPTZ,
    sent_to TEXT[],
    
    generated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- Create indexes
CREATE INDEX idx_payment_methods_client_id ON payment_methods(client_id);
CREATE INDEX idx_recurring_billing_client_id ON recurring_billing(client_id);
CREATE INDEX idx_recurring_billing_next_date ON recurring_billing(next_billing_date);
CREATE INDEX idx_recurring_billing_status ON recurring_billing(status);
CREATE INDEX idx_expenses_client_id ON expenses(client_id);
CREATE INDEX idx_expenses_status ON expenses(status);
CREATE INDEX idx_expenses_date ON expenses(expense_date);
CREATE INDEX idx_client_budgets_client_id ON client_budgets(client_id);
CREATE INDEX idx_client_profitability_client_id ON client_profitability(client_id);
CREATE INDEX idx_payment_transactions_client_id ON payment_transactions(client_id);
CREATE INDEX idx_payment_transactions_invoice_id ON payment_transactions(invoice_id);
CREATE INDEX idx_payment_transactions_status ON payment_transactions(status);
CREATE INDEX idx_credit_notes_client_id ON credit_notes(client_id);

-- Function to process recurring billing
CREATE OR REPLACE FUNCTION process_recurring_billing()
RETURNS void AS $$
DECLARE
    billing_record RECORD;
    new_invoice_id UUID;
BEGIN
    FOR billing_record IN 
        SELECT * FROM recurring_billing 
        WHERE status = 'active' 
        AND next_billing_date <= CURRENT_DATE
    LOOP
        -- Create invoice
        new_invoice_id := gen_random_uuid();
        
        INSERT INTO invoices (
            id, client_id, invoice_number, issue_date, due_date, 
            status, subtotal, total_amount, notes
        ) VALUES (
            new_invoice_id,
            billing_record.client_id,
            'INV-' || to_char(NOW(), 'YYYYMMDD') || '-' || substr(new_invoice_id::text, 1, 8),
            CURRENT_DATE,
            CURRENT_DATE + billing_record.payment_terms_days,
            CASE WHEN billing_record.auto_charge THEN 'processing' ELSE 'sent' END,
            billing_record.amount,
            billing_record.amount,
            'Recurring billing: ' || billing_record.name
        );
        
        -- Copy line items
        INSERT INTO invoice_line_items (invoice_id, line_number, description, quantity, unit_price, line_total)
        SELECT new_invoice_id, row_number() OVER (), name || ' - ' || description, 
               quantity, unit_price, total
        FROM recurring_billing_items
        WHERE recurring_billing_id = billing_record.id;
        
        -- Update next billing date
        UPDATE recurring_billing
        SET last_billed_date = CURRENT_DATE,
            next_billing_date = CASE frequency
                WHEN 'monthly' THEN CURRENT_DATE + INTERVAL '1 month'
                WHEN 'quarterly' THEN CURRENT_DATE + INTERVAL '3 months'
                WHEN 'annually' THEN CURRENT_DATE + INTERVAL '1 year'
                ELSE CURRENT_DATE + INTERVAL '1 month'
            END
        WHERE id = billing_record.id;
        
        -- Process auto-charge if enabled
        IF billing_record.auto_charge AND billing_record.payment_method_id IS NOT NULL THEN
            -- Queue payment processing
            INSERT INTO payment_transactions (
                client_id, invoice_id, payment_method_id,
                transaction_type, amount, status
            ) VALUES (
                billing_record.client_id, new_invoice_id, billing_record.payment_method_id,
                'payment', billing_record.amount, 'pending'
            );
        END IF;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate client profitability
CREATE OR REPLACE FUNCTION calculate_client_profitability(
    p_client_id UUID,
    p_start_date DATE,
    p_end_date DATE
) RETURNS void AS $$
DECLARE
    v_revenue RECORD;
    v_costs RECORD;
    v_time RECORD;
BEGIN
    -- Calculate revenue
    SELECT 
        COALESCE(SUM(CASE WHEN i.notes LIKE '%Recurring%' THEN i.total_amount ELSE 0 END), 0) as recurring,
        COALESCE(SUM(CASE WHEN i.notes LIKE '%Project%' THEN i.total_amount ELSE 0 END), 0) as project,
        COALESCE(SUM(i.total_amount), 0) as total
    INTO v_revenue
    FROM invoices i
    WHERE i.client_id = p_client_id 
    AND i.issue_date BETWEEN p_start_date AND p_end_date
    AND i.status IN ('paid', 'sent');
    
    -- Calculate costs
    SELECT 
        COALESCE(SUM(e.amount), 0) as expenses,
        COALESCE(SUM(te.duration_minutes * COALESCE(u.hourly_rate, 150) / 60), 0) as labor
    INTO v_costs
    FROM expenses e
    FULL OUTER JOIN (
        SELECT te.duration_minutes, te.user_id 
        FROM time_entries te
        WHERE te.start_time BETWEEN p_start_date AND p_end_date
    ) te ON false
    LEFT JOIN users u ON te.user_id = u.id
    WHERE e.client_id = p_client_id 
    AND e.expense_date BETWEEN p_start_date AND p_end_date;
    
    -- Calculate time metrics
    SELECT 
        COALESCE(SUM(duration_minutes) / 60.0, 0) as total_hours,
        COALESCE(SUM(CASE WHEN billable THEN duration_minutes ELSE 0 END) / 60.0, 0) as billable_hours
    INTO v_time
    FROM time_entries te
    WHERE EXISTS (
        SELECT 1 FROM tickets t WHERE t.id = te.ticket_id AND t.client_id = p_client_id
    ) AND te.start_time BETWEEN p_start_date AND p_end_date;
    
    -- Insert or update profitability record
    INSERT INTO client_profitability (
        client_id, period_start, period_end,
        recurring_revenue, total_revenue,
        expense_cost, labor_cost, total_cost,
        gross_profit, gross_margin_percent,
        hours_worked, billable_hours, utilization_rate
    ) VALUES (
        p_client_id, p_start_date, p_end_date,
        v_revenue.recurring, v_revenue.total,
        v_costs.expenses, v_costs.labor, v_costs.expenses + v_costs.labor,
        v_revenue.total - (v_costs.expenses + v_costs.labor),
        CASE WHEN v_revenue.total > 0 
            THEN ((v_revenue.total - (v_costs.expenses + v_costs.labor)) / v_revenue.total * 100)
            ELSE 0 
        END,
        v_time.total_hours, v_time.billable_hours,
        CASE WHEN v_time.total_hours > 0 
            THEN (v_time.billable_hours / v_time.total_hours * 100)
            ELSE 0 
        END
    )
    ON CONFLICT (client_id, period_start, period_end) 
    DO UPDATE SET
        recurring_revenue = EXCLUDED.recurring_revenue,
        total_revenue = EXCLUDED.total_revenue,
        expense_cost = EXCLUDED.expense_cost,
        labor_cost = EXCLUDED.labor_cost,
        total_cost = EXCLUDED.total_cost,
        gross_profit = EXCLUDED.gross_profit,
        gross_margin_percent = EXCLUDED.gross_margin_percent,
        hours_worked = EXCLUDED.hours_worked,
        billable_hours = EXCLUDED.billable_hours,
        utilization_rate = EXCLUDED.utilization_rate,
        calculated_at = NOW();
END;
$$ LANGUAGE plpgsql;