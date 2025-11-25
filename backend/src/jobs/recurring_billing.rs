// Recurring Billing Job - Handles automated invoice generation and payment reminders

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::services::EmailService;

#[derive(Debug)]
pub struct RecurringBillingJob {
    db_pool: PgPool,
    email_service: EmailService,
    payment_reminder_enabled: bool,
}

#[derive(Debug, Default)]
pub struct BillingJobResult {
    pub invoices_generated: i32,
    pub recurring_services_processed: i32,
    pub reminders_sent: i32,
    pub total_amount_invoiced: Decimal,
    pub errors: Vec<String>,
}

#[derive(Debug, FromRow)]
struct RecurringService {
    id: Uuid,
    client_id: Uuid,
    client_name: String,
    client_email: Option<String>,
    service_name: String,
    description: Option<String>,
    amount: Decimal,
    billing_frequency: String, // monthly, quarterly, annually
    next_billing_date: NaiveDate,
    last_invoice_date: Option<NaiveDate>,
    is_active: bool,
}

#[derive(Debug, FromRow)]
struct UnpaidInvoice {
    id: Uuid,
    invoice_number: String,
    client_id: Uuid,
    client_name: String,
    client_email: Option<String>,
    total_amount: Decimal,
    due_date: NaiveDate,
    days_overdue: i32,
    reminder_count: i32,
    last_reminder_date: Option<NaiveDate>,
}

impl RecurringBillingJob {
    pub fn new(
        db_pool: PgPool,
        email_service: EmailService,
        payment_reminder_enabled: bool,
    ) -> Self {
        Self {
            db_pool,
            email_service,
            payment_reminder_enabled,
        }
    }

    pub async fn run(&self) -> Result<BillingJobResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = BillingJobResult::default();

        // Process recurring services due for billing
        if let Err(e) = self.process_recurring_services(&mut result).await {
            result.errors.push(format!("Recurring services error: {}", e));
        }

        // Send payment reminders for unpaid invoices
        if self.payment_reminder_enabled {
            if let Err(e) = self.send_payment_reminders(&mut result).await {
                result.errors.push(format!("Payment reminders error: {}", e));
            }
        }

        Ok(result)
    }

    async fn process_recurring_services(&self, result: &mut BillingJobResult) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let today = Utc::now().date_naive();

        // Get all recurring services due for billing
        let services = sqlx::query_as::<_, RecurringService>(
            r#"
            SELECT
                rs.id, rs.service_name, rs.description, rs.amount,
                rs.billing_frequency, rs.next_billing_date, rs.last_invoice_date, rs.is_active,
                c.id as client_id, c.name as client_name, c.email as client_email
            FROM recurring_services rs
            JOIN clients c ON rs.client_id = c.id
            WHERE rs.is_active = true
                AND rs.next_billing_date <= $1
            ORDER BY rs.next_billing_date ASC
            "#
        )
        .bind(today)
        .fetch_all(&self.db_pool)
        .await?;

        result.recurring_services_processed = services.len() as i32;

        for service in services {
            match self.create_invoice_for_service(&service).await {
                Ok(invoice_id) => {
                    result.invoices_generated += 1;
                    result.total_amount_invoiced += service.amount;

                    // Update next billing date
                    let next_date = self.calculate_next_billing_date(
                        service.next_billing_date,
                        &service.billing_frequency
                    );

                    sqlx::query(
                        "UPDATE recurring_services
                         SET next_billing_date = $2, last_invoice_date = $3, updated_at = NOW()
                         WHERE id = $1"
                    )
                    .bind(service.id)
                    .bind(next_date)
                    .bind(today)
                    .execute(&self.db_pool)
                    .await?;

                    // Send invoice email
                    if let Some(email) = &service.client_email {
                        if let Err(e) = self.send_invoice_email(&service, invoice_id).await {
                            result.errors.push(format!("Failed to send invoice email for {}: {}", service.service_name, e));
                        }
                    }

                    info!("Created invoice for recurring service: {} ({})", service.service_name, service.client_name);
                }
                Err(e) => {
                    result.errors.push(format!("Failed to create invoice for {}: {}", service.service_name, e));
                }
            }
        }

        Ok(())
    }

    async fn create_invoice_for_service(&self, service: &RecurringService) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let invoice_id = Uuid::new_v4();
        let invoice_number = self.generate_invoice_number().await?;
        let today = Utc::now().date_naive();
        let due_date = today + chrono::Duration::days(30); // Net 30

        // Create invoice
        sqlx::query(
            r#"
            INSERT INTO invoices
            (id, client_id, invoice_number, issue_date, due_date, status,
             subtotal, tax_amount, total_amount, currency, notes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 'sent', $6, $7, $8, 'USD', $9, NOW(), NOW())
            "#
        )
        .bind(invoice_id)
        .bind(service.client_id)
        .bind(&invoice_number)
        .bind(today)
        .bind(due_date)
        .bind(service.amount)
        .bind(Decimal::ZERO)
        .bind(service.amount)
        .bind(format!("Recurring service: {}", service.service_name))
        .execute(&self.db_pool)
        .await?;

        // Create line item
        let billing_period = self.get_billing_period_description(service.next_billing_date, &service.billing_frequency);

        sqlx::query(
            r#"
            INSERT INTO invoice_line_items
            (id, invoice_id, line_number, description, quantity, unit_price, line_total, created_at)
            VALUES ($1, $2, 1, $3, 1, $4, $4, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(invoice_id)
        .bind(format!("{} - {}", service.service_name, billing_period))
        .bind(service.amount)
        .execute(&self.db_pool)
        .await?;

        Ok(invoice_id)
    }

    fn calculate_next_billing_date(&self, current_date: NaiveDate, frequency: &str) -> NaiveDate {
        match frequency {
            "weekly" => current_date + chrono::Duration::weeks(1),
            "biweekly" => current_date + chrono::Duration::weeks(2),
            "monthly" => {
                let next_month = if current_date.month() == 12 {
                    NaiveDate::from_ymd_opt(current_date.year() + 1, 1, current_date.day().min(28))
                } else {
                    NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, current_date.day().min(28))
                };
                next_month.unwrap_or(current_date + chrono::Duration::days(30))
            }
            "quarterly" => {
                let new_month = current_date.month() + 3;
                if new_month > 12 {
                    NaiveDate::from_ymd_opt(current_date.year() + 1, new_month - 12, current_date.day().min(28))
                } else {
                    NaiveDate::from_ymd_opt(current_date.year(), new_month, current_date.day().min(28))
                }.unwrap_or(current_date + chrono::Duration::days(90))
            }
            "annually" => {
                NaiveDate::from_ymd_opt(current_date.year() + 1, current_date.month(), current_date.day())
                    .unwrap_or(current_date + chrono::Duration::days(365))
            }
            _ => current_date + chrono::Duration::days(30)
        }
    }

    fn get_billing_period_description(&self, billing_date: NaiveDate, frequency: &str) -> String {
        let end_date = self.calculate_next_billing_date(billing_date, frequency) - chrono::Duration::days(1);
        format!("{} - {}", billing_date.format("%b %d, %Y"), end_date.format("%b %d, %Y"))
    }

    async fn generate_invoice_number(&self) -> Result<String, sqlx::Error> {
        let result = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(invoice_number FROM '^INV-(\\d+)$') AS INTEGER)), 0) + 1 FROM invoices WHERE invoice_number ~ '^INV-\\d+$'"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let next_number = result.unwrap_or(1);
        Ok(format!("INV-{:06}", next_number))
    }

    async fn send_invoice_email(&self, service: &RecurringService, invoice_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let invoice = sqlx::query_as::<_, (String, Decimal, NaiveDate)>(
            "SELECT invoice_number, total_amount, due_date FROM invoices WHERE id = $1"
        )
        .bind(invoice_id)
        .fetch_one(&self.db_pool)
        .await?;

        let subject = format!("Invoice {} - {} - ${}", invoice.0, service.service_name, invoice.1);

        let html_body = format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
                    .container {{ max-width: 600px; margin: 0 auto; background: white; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 12px rgba(0,0,0,0.1); }}
                    .header {{ background: linear-gradient(135deg, #1f2937 0%, #374151 100%); color: white; padding: 24px; text-align: center; }}
                    .content {{ padding: 24px; }}
                    .invoice-details {{ background: #f9fafb; border-radius: 8px; padding: 20px; margin: 20px 0; }}
                    .amount {{ font-size: 32px; font-weight: 700; color: #1f2937; text-align: center; margin: 20px 0; }}
                    .detail-row {{ display: flex; justify-content: space-between; padding: 10px 0; border-bottom: 1px solid #e5e7eb; }}
                    .detail-row:last-child {{ border-bottom: none; }}
                    .footer {{ background: #f9fafb; padding: 16px 24px; text-align: center; color: #6b7280; font-size: 14px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1 style="margin: 0;">Invoice</h1>
                        <p style="margin: 8px 0 0; opacity: 0.9;">{}</p>
                    </div>
                    <div class="content">
                        <p>Dear {},</p>
                        <p>Please find attached your invoice for recurring services.</p>

                        <div class="invoice-details">
                            <div class="amount">${}</div>
                            <div class="detail-row">
                                <span>Invoice Number</span>
                                <strong>{}</strong>
                            </div>
                            <div class="detail-row">
                                <span>Service</span>
                                <strong>{}</strong>
                            </div>
                            <div class="detail-row">
                                <span>Due Date</span>
                                <strong>{}</strong>
                            </div>
                        </div>

                        <p style="background: #f0f9ff; padding: 16px; border-radius: 8px; color: #0369a1;">
                            Payment is due by <strong>{}</strong>. Please ensure timely payment to avoid any service interruptions.
                        </p>

                        <p>If you have any questions about this invoice, please don't hesitate to contact us.</p>

                        <p>Thank you for your business!</p>
                    </div>
                    <div class="footer">
                        <p>Resolve MSP Platform - Billing Department</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            invoice.0, // invoice number in header
            service.client_name,
            invoice.1, // amount
            invoice.0, // invoice number
            service.service_name,
            invoice.2.format("%B %d, %Y"),
            invoice.2.format("%B %d, %Y")
        );

        self.email_service.send_email(
            service.client_email.as_deref().unwrap_or(""),
            Some(&service.client_name),
            &subject,
            &html_body,
            None
        ).await?;

        Ok(())
    }

    async fn send_payment_reminders(&self, result: &mut BillingJobResult) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let today = Utc::now().date_naive();

        // Reminder schedule: 7 days before due, on due date, 7 days overdue, 14 days overdue, 30 days overdue
        let reminder_thresholds = vec![-7, 0, 7, 14, 30];

        let unpaid_invoices = sqlx::query_as::<_, UnpaidInvoice>(
            r#"
            SELECT
                i.id, i.invoice_number, i.total_amount, i.due_date,
                EXTRACT(DAY FROM ($1::date - i.due_date))::integer as days_overdue,
                i.reminder_count, i.last_reminder_date,
                c.id as client_id, c.name as client_name, c.email as client_email
            FROM invoices i
            JOIN clients c ON i.client_id = c.id
            WHERE i.status IN ('sent', 'overdue', 'viewed')
                AND i.due_date <= $1 + INTERVAL '7 days'
            ORDER BY i.due_date ASC
            "#
        )
        .bind(today)
        .fetch_all(&self.db_pool)
        .await?;

        for invoice in unpaid_invoices {
            // Check if we should send a reminder based on threshold
            if self.should_send_reminder(&invoice, &reminder_thresholds) {
                if let Some(email) = &invoice.client_email {
                    match self.send_payment_reminder_email(&invoice).await {
                        Ok(_) => {
                            result.reminders_sent += 1;

                            // Update reminder count and date
                            sqlx::query(
                                "UPDATE invoices
                                 SET reminder_count = reminder_count + 1,
                                     last_reminder_date = $2,
                                     status = CASE WHEN due_date < $2 THEN 'overdue' ELSE status END,
                                     updated_at = NOW()
                                 WHERE id = $1"
                            )
                            .bind(invoice.id)
                            .bind(today)
                            .execute(&self.db_pool)
                            .await?;

                            info!("Sent payment reminder for invoice {}", invoice.invoice_number);
                        }
                        Err(e) => {
                            result.errors.push(format!("Failed to send reminder for {}: {}", invoice.invoice_number, e));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn should_send_reminder(&self, invoice: &UnpaidInvoice, thresholds: &[i32]) -> bool {
        let today = Utc::now().date_naive();

        // Check if days_overdue matches any threshold
        let matches_threshold = thresholds.iter().any(|t| {
            let diff = (invoice.days_overdue - *t).abs();
            diff <= 1 // Allow 1 day tolerance
        });

        if !matches_threshold {
            return false;
        }

        // Check if we already sent a reminder today
        if let Some(last_reminder) = invoice.last_reminder_date {
            if last_reminder >= today - chrono::Duration::days(1) {
                return false;
            }
        }

        true
    }

    async fn send_payment_reminder_email(&self, invoice: &UnpaidInvoice) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let is_overdue = invoice.days_overdue > 0;
        let urgency = if invoice.days_overdue > 14 {
            ("FINAL NOTICE", "#dc2626", "#fef2f2")
        } else if invoice.days_overdue > 0 {
            ("OVERDUE", "#f97316", "#fff7ed")
        } else if invoice.days_overdue == 0 {
            ("DUE TODAY", "#eab308", "#fefce8")
        } else {
            ("REMINDER", "#3b82f6", "#eff6ff")
        };

        let subject = if is_overdue {
            format!("[{}] Invoice {} is {} days overdue - ${}", urgency.0, invoice.invoice_number, invoice.days_overdue, invoice.total_amount)
        } else {
            format!("[{}] Invoice {} - Payment Due {} - ${}", urgency.0, invoice.invoice_number,
                    if invoice.days_overdue == 0 { "Today".to_string() } else { format!("in {} days", -invoice.days_overdue) },
                    invoice.total_amount)
        };

        let html_body = format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
                    .container {{ max-width: 600px; margin: 0 auto; background: white; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 12px rgba(0,0,0,0.1); }}
                    .header {{ background: {}; color: white; padding: 24px; text-align: center; }}
                    .badge {{ display: inline-block; background: rgba(255,255,255,0.2); padding: 4px 12px; border-radius: 999px; font-size: 12px; font-weight: 600; margin-bottom: 8px; }}
                    .content {{ padding: 24px; }}
                    .alert-box {{ background: {}; border-left: 4px solid {}; padding: 16px; margin-bottom: 20px; border-radius: 0 8px 8px 0; }}
                    .amount {{ font-size: 40px; font-weight: 700; color: {}; text-align: center; margin: 20px 0; }}
                    .detail-row {{ display: flex; justify-content: space-between; padding: 10px 0; border-bottom: 1px solid #e5e7eb; }}
                    .footer {{ background: #f9fafb; padding: 16px 24px; text-align: center; color: #6b7280; font-size: 14px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <span class="badge">{}</span>
                        <h1 style="margin: 0;">Payment {}</h1>
                    </div>
                    <div class="content">
                        <div class="alert-box">
                            <strong>Invoice {}</strong> {} requires your attention.
                        </div>

                        <div class="amount">${}</div>

                        <div style="background: #f9fafb; border-radius: 8px; padding: 16px; margin: 20px 0;">
                            <div class="detail-row">
                                <span>Invoice Number</span>
                                <strong>{}</strong>
                            </div>
                            <div class="detail-row">
                                <span>Due Date</span>
                                <strong>{}</strong>
                            </div>
                            <div class="detail-row">
                                <span>Status</span>
                                <strong style="color: {};">{}</strong>
                            </div>
                        </div>

                        <p>{}</p>

                        <p>If you have already made this payment, please disregard this reminder. If you have any questions or need to arrange a payment plan, please contact us immediately.</p>

                        <p>Thank you for your prompt attention to this matter.</p>
                    </div>
                    <div class="footer">
                        <p>Resolve MSP Platform - Billing Department</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            urgency.1, // header bg
            urgency.2, // alert box bg
            urgency.1, // alert box border
            urgency.1, // amount color
            urgency.0, // badge text
            if is_overdue { "Overdue" } else { "Reminder" },
            invoice.invoice_number,
            if is_overdue { format!("is {} days overdue and", invoice.days_overdue) } else { "".to_string() },
            invoice.total_amount,
            invoice.invoice_number,
            invoice.due_date.format("%B %d, %Y"),
            urgency.1, // status color
            if is_overdue { format!("{} Days Overdue", invoice.days_overdue) } else { "Pending".to_string() },
            if is_overdue {
                "Please arrange payment immediately to avoid service interruption and potential late fees."
            } else {
                "This is a friendly reminder that your payment is due soon. Please arrange payment at your earliest convenience."
            }
        );

        self.email_service.send_email(
            invoice.client_email.as_deref().unwrap_or(""),
            Some(&invoice.client_name),
            &subject,
            &html_body,
            None
        ).await?;

        Ok(())
    }
}
