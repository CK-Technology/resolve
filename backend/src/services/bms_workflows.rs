use crate::services::EmailService;
use chrono::{DateTime, Utc, NaiveDate, Datelike};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BmsWorkflowConfig {
    pub auto_invoice_day: u32,        // Day of month to generate invoices (1-28)
    pub payment_terms_days: i32,      // Payment terms in days
    pub overdue_reminder_days: Vec<i32>, // Days after due date to send reminders
    pub auto_collections_enabled: bool,
    pub minimum_billable_hours: Decimal,
}

#[derive(Debug)]
pub struct BmsWorkflowService {
    config: BmsWorkflowConfig,
    db_pool: PgPool,
    email_service: EmailService,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
    pub billable_hours: Option<Decimal>,
    pub project_name: Option<String>,
    pub task_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientBillingData {
    pub client_id: Uuid,
    pub client_name: String,
    pub contact_email: String,
    pub billing_address: Option<String>,
    pub hourly_rate: Decimal,
    pub time_entries: Vec<BillableTimeEntry>,
    pub fixed_charges: Vec<FixedCharge>,
    pub total_hours: Decimal,
    pub total_amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillableTimeEntry {
    pub id: Uuid,
    pub project_name: Option<String>,
    pub task_description: Option<String>,
    pub ticket_subject: Option<String>,
    pub hours: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
    pub date: NaiveDate,
    pub user_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FixedCharge {
    pub description: String,
    pub amount: Decimal,
    pub quantity: Decimal,
}

impl BmsWorkflowService {
    pub fn new(
        config: BmsWorkflowConfig,
        db_pool: PgPool,
        email_service: EmailService,
    ) -> Self {
        Self {
            config,
            db_pool,
            email_service,
        }
    }

    pub async fn start_billing_workflows(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting BMS billing workflows");
        
        // Start daily workflow checks
        let mut daily_interval = interval(Duration::from_secs(24 * 60 * 60)); // Daily
        
        tokio::spawn({
            let service = self.clone();
            async move {
                loop {
                    daily_interval.tick().await;
                    
                    if let Err(e) = service.run_daily_workflows().await {
                        error!("Error in daily billing workflows: {}", e);
                    }
                }
            }
        });

        // Start hourly workflow checks for reminders
        let mut hourly_interval = interval(Duration::from_secs(60 * 60)); // Hourly
        
        tokio::spawn({
            let service = self.clone();
            async move {
                loop {
                    hourly_interval.tick().await;
                    
                    if let Err(e) = service.run_hourly_workflows().await {
                        error!("Error in hourly billing workflows: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn run_daily_workflows(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let today = Utc::now().date_naive();
        
        // Check if today is the auto-invoice day
        if today.day() == self.config.auto_invoice_day {
            info!("Running monthly auto-invoicing on day {}", today.day());
            self.generate_monthly_invoices().await?;
        }

        // Run daily collections check
        self.process_overdue_invoices().await?;
        
        // Send time tracking reminders
        self.send_time_tracking_reminders().await?;

        Ok(())
    }

    async fn run_hourly_workflows(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Process any pending billing workflows
        self.process_pending_billing_items().await?;
        
        Ok(())
    }

    /// Generate monthly invoices for all clients with billable time
    pub async fn generate_monthly_invoices(&self) -> Result<Vec<Uuid>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Generating monthly invoices for all clients");
        
        let billing_data = self.collect_monthly_billing_data().await?;
        let mut created_invoices = Vec::new();

        for client_data in billing_data {
            if client_data.total_amount > Decimal::ZERO {
                match self.create_invoice_for_client(&client_data).await {
                    Ok(invoice_id) => {
                        created_invoices.push(invoice_id);
                        info!("Created invoice for client: {}", client_data.client_name);
                        
                        // Send invoice email
                        if let Err(e) = self.send_invoice_email(invoice_id, &client_data).await {
                            error!("Failed to send invoice email to {}: {}", client_data.contact_email, e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to create invoice for client {}: {}", client_data.client_name, e);
                    }
                }
            }
        }

        info!("Generated {} invoices", created_invoices.len());
        Ok(created_invoices)
    }

    async fn collect_monthly_billing_data(&self) -> Result<Vec<ClientBillingData>, Box<dyn std::error::Error + Send + Sync>> {
        let today = Utc::now().date_naive();
        let start_of_month = today.with_day(1).unwrap();
        let end_of_month = if today.month() == 12 {
            NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1).unwrap() - chrono::Duration::days(1)
        };

        // Get all clients with billable time entries
        let time_entries = sqlx::query!(
            r#"
            SELECT 
                te.id, te.user_id, te.project_id, te.ticket_id,
                te.start_time::date as work_date,
                te.duration_minutes,
                te.billable, te.hourly_rate,
                c.id as client_id, c.name as client_name,
                c.email as client_email, c.billing_address,
                u.first_name || ' ' || u.last_name as user_name,
                p.name as project_name,
                t.subject as ticket_subject,
                te.description as task_description,
                COALESCE(te.hourly_rate, c.default_hourly_rate, 150.0) as effective_rate
            FROM time_entries te
            JOIN users u ON te.user_id = u.id
            JOIN clients c ON (
                (te.project_id IS NOT NULL AND EXISTS(SELECT 1 FROM projects pr WHERE pr.id = te.project_id AND pr.client_id = c.id))
                OR (te.ticket_id IS NOT NULL AND EXISTS(SELECT 1 FROM tickets tk WHERE tk.id = te.ticket_id AND tk.client_id = c.id))
            )
            LEFT JOIN projects p ON te.project_id = p.id
            LEFT JOIN tickets t ON te.ticket_id = t.id
            WHERE te.start_time::date BETWEEN $1 AND $2
                AND te.billable = true
                AND te.billed = false
                AND te.duration_minutes > 0
            ORDER BY c.name, te.start_time
            "#,
            start_of_month,
            end_of_month
        )
        .fetch_all(&self.db_pool)
        .await?;

        // Group by client
        let mut client_map: HashMap<Uuid, ClientBillingData> = HashMap::new();

        for entry in time_entries {
            let client_id = entry.client_id;
            let hours = Decimal::from(entry.duration_minutes.unwrap_or(0)) / Decimal::from(60);
            let rate = entry.effective_rate.unwrap_or(rust_decimal::Decimal::from(150));
            let amount = hours * rate;

            let billing_entry = BillableTimeEntry {
                id: entry.id,
                project_name: entry.project_name,
                task_description: entry.task_description,
                ticket_subject: entry.ticket_subject,
                hours,
                rate,
                amount,
                date: entry.work_date.unwrap_or(start_of_month),
                user_name: entry.user_name,
            };

            let client_data = client_map.entry(client_id).or_insert_with(|| ClientBillingData {
                client_id,
                client_name: entry.client_name,
                contact_email: entry.client_email.unwrap_or_default(),
                billing_address: entry.billing_address,
                hourly_rate: rate,
                time_entries: Vec::new(),
                fixed_charges: Vec::new(),
                total_hours: Decimal::ZERO,
                total_amount: Decimal::ZERO,
            });

            client_data.time_entries.push(billing_entry);
            client_data.total_hours += hours;
            client_data.total_amount += amount;
        }

        // Filter clients with minimum billable hours
        let billing_data: Vec<ClientBillingData> = client_map
            .into_values()
            .filter(|data| data.total_hours >= self.config.minimum_billable_hours)
            .collect();

        Ok(billing_data)
    }

    async fn create_invoice_for_client(
        &self,
        client_data: &ClientBillingData,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let invoice_id = Uuid::new_v4();
        let invoice_number = self.generate_invoice_number().await?;
        let due_date = Utc::now().date_naive() + chrono::Duration::days(self.config.payment_terms_days as i64);

        // Create invoice
        sqlx::query!(
            r#"
            INSERT INTO invoices 
            (id, client_id, invoice_number, issue_date, due_date, status, 
             subtotal, tax_amount, total_amount, currency, notes, created_at, updated_at)
            VALUES ($1, $2, $3, CURRENT_DATE, $4, 'draft', 
                   $5, $6, $7, 'USD', $8, NOW(), NOW())
            "#,
            invoice_id,
            client_data.client_id,
            invoice_number,
            due_date,
            client_data.total_amount,
            Decimal::ZERO, // tax_amount - calculate if needed
            client_data.total_amount,
            format!("Invoice for services rendered - {} hours", client_data.total_hours)
        )
        .execute(&self.db_pool)
        .await?;

        // Create invoice line items
        for (index, time_entry) in client_data.time_entries.iter().enumerate() {
            let description = format!(
                "{} - {} ({} hrs @ ${}/hr)",
                time_entry.date.format("%Y-%m-%d"),
                time_entry.task_description.as_ref()
                    .or(time_entry.project_name.as_ref())
                    .or(time_entry.ticket_subject.as_ref())
                    .unwrap_or(&format!("Work by {}", time_entry.user_name)),
                time_entry.hours,
                time_entry.rate
            );

            sqlx::query!(
                r#"
                INSERT INTO invoice_line_items
                (id, invoice_id, line_number, description, quantity, 
                 unit_price, line_total, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                "#,
                Uuid::new_v4(),
                invoice_id,
                (index + 1) as i32,
                description,
                time_entry.hours,
                time_entry.rate,
                time_entry.amount
            )
            .execute(&self.db_pool)
            .await?;
        }

        // Mark time entries as billed
        let time_entry_ids: Vec<Uuid> = client_data.time_entries.iter().map(|te| te.id).collect();
        sqlx::query!(
            "UPDATE time_entries SET billed = true WHERE id = ANY($1)",
            &time_entry_ids
        )
        .execute(&self.db_pool)
        .await?;

        // Set invoice to 'sent' status
        sqlx::query!(
            "UPDATE invoices SET status = 'sent' WHERE id = $1",
            invoice_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(invoice_id)
    }

    async fn generate_invoice_number(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query!(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(invoice_number FROM '^INV-(\\d+)$') AS INTEGER)), 0) + 1 as next_number FROM invoices WHERE invoice_number ~ '^INV-\\d+$'"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let next_number = result.next_number.unwrap_or(1);
        Ok(format!("INV-{:06}", next_number))
    }

    async fn send_invoice_email(
        &self,
        invoice_id: Uuid,
        client_data: &ClientBillingData,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let invoice = sqlx::query!(
            "SELECT invoice_number, total_amount, due_date FROM invoices WHERE id = $1",
            invoice_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        let subject = format!("Invoice {} - ${}", invoice.invoice_number, invoice.total_amount);
        
        let html_body = format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
                    .container {{ max-width: 600px; margin: 0 auto; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                    .header {{ background: #1f2937; color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 30px; }}
                    .invoice-details {{ background: #f8fafc; border-left: 4px solid #1f2937; padding: 15px; margin: 20px 0; }}
                    .footer {{ background: #f8fafc; padding: 20px; text-align: center; color: #666; }}
                    .btn {{ display: inline-block; background: #1f2937; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin: 10px 0; }}
                    .amount {{ font-size: 24px; font-weight: bold; color: #1f2937; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>💼 Invoice from Resolve</h1>
                    </div>
                    <div class="content">
                        <p>Dear {},</p>
                        <p>Thank you for your business! Please find your invoice details below:</p>
                        
                        <div class="invoice-details">
                            <h3>Invoice Details</h3>
                            <p><strong>Invoice Number:</strong> {}</p>
                            <p><strong>Total Hours:</strong> {}</p>
                            <p><strong>Due Date:</strong> {}</p>
                            <p class="amount">Amount Due: ${}</p>
                        </div>
                        
                        <p>This invoice covers professional services rendered during the past billing period. 
                        All work has been completed to our high standards.</p>
                        
                        <p>Payment is due by the date specified above. If you have any questions about 
                        this invoice, please don't hesitate to contact our billing department.</p>
                        
                        <p>Thank you for choosing Resolve for your business needs.</p>
                        
                        <p>Best regards,<br>The Resolve Team</p>
                    </div>
                    <div class="footer">
                        <p>Resolve MSP Services | Questions? Reply to this email</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            client_data.client_name,
            invoice.invoice_number,
            client_data.total_hours,
            invoice.due_date,
            invoice.total_amount
        );

        self.email_service
            .send_email(&client_data.contact_email, Some(&client_data.client_name), &subject, &html_body, None)
            .await?;

        info!("Sent invoice email to {}", client_data.contact_email);
        Ok(())
    }

    async fn process_overdue_invoices(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let today = Utc::now().date_naive();
        
        for days_overdue in &self.config.overdue_reminder_days {
            let target_due_date = today - chrono::Duration::days(*days_overdue as i64);
            
            let overdue_invoices = sqlx::query!(
                r#"
                SELECT i.id, i.invoice_number, i.total_amount, i.due_date,
                       c.name as client_name, c.email as client_email
                FROM invoices i
                JOIN clients c ON i.client_id = c.id
                WHERE i.status IN ('sent', 'overdue')
                    AND i.due_date = $1
                    AND NOT EXISTS (
                        SELECT 1 FROM invoice_line_items ili 
                        WHERE ili.invoice_id = i.id 
                        AND ili.description LIKE '%Overdue Reminder%'
                    )
                "#,
                target_due_date
            )
            .fetch_all(&self.db_pool)
            .await?;

            for invoice in overdue_invoices {
                if let Err(e) = self.send_overdue_reminder(
                    invoice.id,
                    &invoice.invoice_number,
                    invoice.total_amount,
                    *days_overdue,
                    &invoice.client_name,
                    &invoice.client_email.unwrap_or_default(),
                ).await {
                    error!("Failed to send overdue reminder for invoice {}: {}", invoice.invoice_number, e);
                }

                // Update invoice status
                sqlx::query!(
                    "UPDATE invoices SET status = 'overdue' WHERE id = $1",
                    invoice.id
                )
                .execute(&self.db_pool)
                .await?;
            }
        }

        Ok(())
    }

    async fn send_overdue_reminder(
        &self,
        invoice_id: Uuid,
        invoice_number: &str,
        amount: rust_decimal::Decimal,
        days_overdue: i32,
        client_name: &str,
        client_email: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let subject = format!("OVERDUE NOTICE - Invoice {} ({} days past due)", invoice_number, days_overdue);
        
        let html_body = format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
                    .container {{ max-width: 600px; margin: 0 auto; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                    .header {{ background: #dc2626; color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 30px; }}
                    .overdue-notice {{ background: #fee2e2; border: 2px solid #dc2626; padding: 15px; margin: 20px 0; border-radius: 6px; }}
                    .footer {{ background: #f8fafc; padding: 20px; text-align: center; color: #666; }}
                    .amount {{ font-size: 24px; font-weight: bold; color: #dc2626; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>⚠️ Payment Overdue Notice</h1>
                    </div>
                    <div class="content">
                        <p>Dear {},</p>
                        
                        <div class="overdue-notice">
                            <h3>URGENT: Payment Required</h3>
                            <p><strong>Invoice:</strong> {}</p>
                            <p><strong>Days Overdue:</strong> {}</p>
                            <p class="amount">Amount Due: ${}</p>
                        </div>
                        
                        <p>This invoice is now <strong>{} days past due</strong>. To avoid any interruption 
                        in service, please arrange payment immediately.</p>
                        
                        <p>If you have already made this payment, please disregard this notice. 
                        If you have any questions or need to arrange a payment plan, 
                        please contact us immediately.</p>
                        
                        <p><strong>Payment must be received within 5 business days to avoid 
                        account suspension.</strong></p>
                        
                        <p>Thank you for your immediate attention to this matter.</p>
                        
                        <p>Sincerely,<br>Resolve Billing Department</p>
                    </div>
                    <div class="footer">
                        <p>For payment questions, contact billing@cktechx.com</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            client_name, invoice_number, days_overdue, amount, days_overdue
        );

        self.email_service
            .send_email(client_email, Some(client_name), &subject, &html_body, None)
            .await?;

        info!("Sent overdue reminder for invoice {} to {}", invoice_number, client_email);
        Ok(())
    }

    async fn send_time_tracking_reminders(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Send reminders to users who haven't logged time today
        let today = Utc::now().date_naive();
        
        let users_without_time = sqlx::query!(
            r#"
            SELECT u.id, u.email, u.first_name, u.last_name
            FROM users u
            WHERE u.is_active = true
                AND NOT EXISTS (
                    SELECT 1 FROM time_entries te 
                    WHERE te.user_id = u.id 
                    AND te.start_time::date = $1
                )
            "#,
            today
        )
        .fetch_all(&self.db_pool)
        .await?;

        for user in users_without_time {
            let user_name = format!("{} {}", user.first_name, user.last_name);
            if let Err(e) = self.send_time_tracking_reminder(&user.email, &user_name).await {
                error!("Failed to send time tracking reminder to {}: {}", user.email, e);
            }
        }

        Ok(())
    }

    async fn send_time_tracking_reminder(&self, email: &str, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let subject = "Time Tracking Reminder - Don't Forget to Log Your Hours!";
        
        let html_body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; padding: 20px;">
                <h2>⏰ Time Tracking Reminder</h2>
                <p>Hi {},</p>
                <p>This is a friendly reminder to log your work hours for today. Accurate time tracking helps ensure:</p>
                <ul>
                    <li>Proper client billing</li>
                    <li>Project cost tracking</li>
                    <li>Resource planning</li>
                </ul>
                <p>Please log into Resolve to record your time entries for today.</p>
                <p>Thank you!</p>
            </body>
            </html>
            "#,
            name
        );

        self.email_service
            .send_email(email, Some(name), subject, &html_body, None)
            .await?;

        Ok(())
    }

    async fn process_pending_billing_items(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Process any pending billing automations
        // This could include recurring charges, usage-based billing, etc.
        Ok(())
    }
}

impl Clone for BmsWorkflowService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            db_pool: self.db_pool.clone(),
            email_service: self.email_service.clone(),
        }
    }
}