// Expiration Monitor Job - Monitors domains, SSL certs, licenses, and warranties for expiration

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::services::EmailService;

#[derive(Debug)]
pub struct ExpirationMonitorJob {
    db_pool: PgPool,
    email_service: EmailService,
    domain_warning_days: Vec<i32>,
    ssl_warning_days: Vec<i32>,
    license_warning_days: Vec<i32>,
    warranty_warning_days: Vec<i32>,
}

#[derive(Debug, Default)]
pub struct ExpirationCheckResult {
    pub total_items_checked: i32,
    pub domains_expiring: i32,
    pub ssl_expiring: i32,
    pub licenses_expiring: i32,
    pub warranties_expiring: i32,
    pub alerts_sent: i32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpirationAlert {
    pub item_type: String,
    pub item_name: String,
    pub client_id: Uuid,
    pub client_name: String,
    pub expiration_date: NaiveDate,
    pub days_until_expiry: i32,
    pub details: serde_json::Value,
}

#[derive(Debug, FromRow)]
struct DomainExpiry {
    id: Uuid,
    domain_name: String,
    client_id: Uuid,
    client_name: String,
    client_email: Option<String>,
    expiration_date: NaiveDate,
    last_notification_date: Option<NaiveDate>,
    registrar: Option<String>,
    auto_renew: bool,
}

#[derive(Debug, FromRow)]
struct SslExpiry {
    id: Uuid,
    domain: String,
    client_id: Uuid,
    client_name: String,
    client_email: Option<String>,
    expiry_date: NaiveDate,
    last_notification_date: Option<NaiveDate>,
    issuer: Option<String>,
    cert_type: Option<String>,
}

#[derive(Debug, FromRow)]
struct LicenseExpiry {
    id: Uuid,
    software_name: String,
    client_id: Uuid,
    client_name: String,
    client_email: Option<String>,
    expiration_date: NaiveDate,
    last_notification_date: Option<NaiveDate>,
    license_count: Option<i32>,
    vendor: Option<String>,
    annual_cost: Option<rust_decimal::Decimal>,
}

#[derive(Debug, FromRow)]
struct WarrantyExpiry {
    id: Uuid,
    asset_name: String,
    asset_type: String,
    client_id: Uuid,
    client_name: String,
    client_email: Option<String>,
    warranty_expiry: NaiveDate,
    last_notification_date: Option<NaiveDate>,
    manufacturer: Option<String>,
    serial_number: Option<String>,
}

impl ExpirationMonitorJob {
    pub fn new(
        db_pool: PgPool,
        email_service: EmailService,
        domain_warning_days: Vec<i32>,
        ssl_warning_days: Vec<i32>,
        license_warning_days: Vec<i32>,
        warranty_warning_days: Vec<i32>,
    ) -> Self {
        Self {
            db_pool,
            email_service,
            domain_warning_days,
            ssl_warning_days,
            license_warning_days,
            warranty_warning_days,
        }
    }

    pub async fn run(&self) -> Result<ExpirationCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = ExpirationCheckResult::default();

        // Check domain expirations
        if let Err(e) = self.check_domain_expirations(&mut result).await {
            result.errors.push(format!("Domain check error: {}", e));
        }

        // Check SSL certificate expirations
        if let Err(e) = self.check_ssl_expirations(&mut result).await {
            result.errors.push(format!("SSL check error: {}", e));
        }

        // Check software license expirations
        if let Err(e) = self.check_license_expirations(&mut result).await {
            result.errors.push(format!("License check error: {}", e));
        }

        // Check warranty expirations
        if let Err(e) = self.check_warranty_expirations(&mut result).await {
            result.errors.push(format!("Warranty check error: {}", e));
        }

        Ok(result)
    }

    async fn check_domain_expirations(&self, result: &mut ExpirationCheckResult) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let today = Utc::now().date_naive();
        let max_days = *self.domain_warning_days.iter().max().unwrap_or(&90);
        let end_date = today + chrono::Duration::days(max_days as i64);

        let domains = sqlx::query_as::<_, DomainExpiry>(
            r#"
            SELECT
                d.id, d.domain_name, d.expiration_date, d.last_notification_date,
                d.registrar, d.auto_renew,
                c.id as client_id, c.name as client_name, c.email as client_email
            FROM domains d
            JOIN clients c ON d.client_id = c.id
            WHERE d.expiration_date BETWEEN $1 AND $2
                AND d.is_active = true
            ORDER BY d.expiration_date ASC
            "#
        )
        .bind(today)
        .bind(end_date)
        .fetch_all(&self.db_pool)
        .await?;

        result.total_items_checked += domains.len() as i32;

        for domain in domains {
            let days_until = (domain.expiration_date - today).num_days() as i32;

            // Check if we should send notification for this threshold
            if self.should_notify(&domain.last_notification_date, days_until, &self.domain_warning_days) {
                result.domains_expiring += 1;

                // Create alert record
                let alert = ExpirationAlert {
                    item_type: "domain".to_string(),
                    item_name: domain.domain_name.clone(),
                    client_id: domain.client_id,
                    client_name: domain.client_name.clone(),
                    expiration_date: domain.expiration_date,
                    days_until_expiry: days_until,
                    details: serde_json::json!({
                        "registrar": domain.registrar,
                        "auto_renew": domain.auto_renew
                    }),
                };

                // Send notification
                if let Some(email) = &domain.client_email {
                    if let Err(e) = self.send_domain_expiration_email(&domain, days_until).await {
                        result.errors.push(format!("Failed to send domain alert for {}: {}", domain.domain_name, e));
                    } else {
                        result.alerts_sent += 1;
                    }
                }

                // Update last notification date
                sqlx::query("UPDATE domains SET last_notification_date = $2 WHERE id = $1")
                    .bind(domain.id)
                    .bind(today)
                    .execute(&self.db_pool)
                    .await?;

                // Log the alert
                self.log_expiration_alert(&alert).await?;
            }
        }

        Ok(())
    }

    async fn check_ssl_expirations(&self, result: &mut ExpirationCheckResult) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let today = Utc::now().date_naive();
        let max_days = *self.ssl_warning_days.iter().max().unwrap_or(&60);
        let end_date = today + chrono::Duration::days(max_days as i64);

        let certs = sqlx::query_as::<_, SslExpiry>(
            r#"
            SELECT
                s.id, s.domain, s.expiry_date, s.last_notification_date,
                s.issuer, s.cert_type,
                c.id as client_id, c.name as client_name, c.email as client_email
            FROM ssl_certificates s
            JOIN clients c ON s.client_id = c.id
            WHERE s.expiry_date BETWEEN $1 AND $2
                AND s.is_active = true
            ORDER BY s.expiry_date ASC
            "#
        )
        .bind(today)
        .bind(end_date)
        .fetch_all(&self.db_pool)
        .await?;

        result.total_items_checked += certs.len() as i32;

        for cert in certs {
            let days_until = (cert.expiry_date - today).num_days() as i32;

            if self.should_notify(&cert.last_notification_date, days_until, &self.ssl_warning_days) {
                result.ssl_expiring += 1;

                let alert = ExpirationAlert {
                    item_type: "ssl_certificate".to_string(),
                    item_name: cert.domain.clone(),
                    client_id: cert.client_id,
                    client_name: cert.client_name.clone(),
                    expiration_date: cert.expiry_date,
                    days_until_expiry: days_until,
                    details: serde_json::json!({
                        "issuer": cert.issuer,
                        "cert_type": cert.cert_type
                    }),
                };

                if let Some(email) = &cert.client_email {
                    if let Err(e) = self.send_ssl_expiration_email(&cert, days_until).await {
                        result.errors.push(format!("Failed to send SSL alert for {}: {}", cert.domain, e));
                    } else {
                        result.alerts_sent += 1;
                    }
                }

                sqlx::query("UPDATE ssl_certificates SET last_notification_date = $2 WHERE id = $1")
                    .bind(cert.id)
                    .bind(today)
                    .execute(&self.db_pool)
                    .await?;

                self.log_expiration_alert(&alert).await?;
            }
        }

        Ok(())
    }

    async fn check_license_expirations(&self, result: &mut ExpirationCheckResult) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let today = Utc::now().date_naive();
        let max_days = *self.license_warning_days.iter().max().unwrap_or(&90);
        let end_date = today + chrono::Duration::days(max_days as i64);

        let licenses = sqlx::query_as::<_, LicenseExpiry>(
            r#"
            SELECT
                l.id, l.software_name, l.expiration_date, l.last_notification_date,
                l.license_count, l.vendor, l.annual_cost,
                c.id as client_id, c.name as client_name, c.email as client_email
            FROM software_licenses l
            JOIN clients c ON l.client_id = c.id
            WHERE l.expiration_date BETWEEN $1 AND $2
                AND l.is_active = true
            ORDER BY l.expiration_date ASC
            "#
        )
        .bind(today)
        .bind(end_date)
        .fetch_all(&self.db_pool)
        .await?;

        result.total_items_checked += licenses.len() as i32;

        for license in licenses {
            let days_until = (license.expiration_date - today).num_days() as i32;

            if self.should_notify(&license.last_notification_date, days_until, &self.license_warning_days) {
                result.licenses_expiring += 1;

                let alert = ExpirationAlert {
                    item_type: "software_license".to_string(),
                    item_name: license.software_name.clone(),
                    client_id: license.client_id,
                    client_name: license.client_name.clone(),
                    expiration_date: license.expiration_date,
                    days_until_expiry: days_until,
                    details: serde_json::json!({
                        "vendor": license.vendor,
                        "license_count": license.license_count,
                        "annual_cost": license.annual_cost
                    }),
                };

                if let Some(email) = &license.client_email {
                    if let Err(e) = self.send_license_expiration_email(&license, days_until).await {
                        result.errors.push(format!("Failed to send license alert for {}: {}", license.software_name, e));
                    } else {
                        result.alerts_sent += 1;
                    }
                }

                sqlx::query("UPDATE software_licenses SET last_notification_date = $2 WHERE id = $1")
                    .bind(license.id)
                    .bind(today)
                    .execute(&self.db_pool)
                    .await?;

                self.log_expiration_alert(&alert).await?;
            }
        }

        Ok(())
    }

    async fn check_warranty_expirations(&self, result: &mut ExpirationCheckResult) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let today = Utc::now().date_naive();
        let max_days = *self.warranty_warning_days.iter().max().unwrap_or(&90);
        let end_date = today + chrono::Duration::days(max_days as i64);

        let warranties = sqlx::query_as::<_, WarrantyExpiry>(
            r#"
            SELECT
                a.id, a.name as asset_name, a.asset_type, a.warranty_expiry,
                a.last_notification_date, a.manufacturer, a.serial_number,
                c.id as client_id, c.name as client_name, c.email as client_email
            FROM assets a
            JOIN clients c ON a.client_id = c.id
            WHERE a.warranty_expiry BETWEEN $1 AND $2
                AND a.is_active = true
                AND a.warranty_expiry IS NOT NULL
            ORDER BY a.warranty_expiry ASC
            "#
        )
        .bind(today)
        .bind(end_date)
        .fetch_all(&self.db_pool)
        .await?;

        result.total_items_checked += warranties.len() as i32;

        for warranty in warranties {
            let days_until = (warranty.warranty_expiry - today).num_days() as i32;

            if self.should_notify(&warranty.last_notification_date, days_until, &self.warranty_warning_days) {
                result.warranties_expiring += 1;

                let alert = ExpirationAlert {
                    item_type: "warranty".to_string(),
                    item_name: warranty.asset_name.clone(),
                    client_id: warranty.client_id,
                    client_name: warranty.client_name.clone(),
                    expiration_date: warranty.warranty_expiry,
                    days_until_expiry: days_until,
                    details: serde_json::json!({
                        "asset_type": warranty.asset_type,
                        "manufacturer": warranty.manufacturer,
                        "serial_number": warranty.serial_number
                    }),
                };

                if let Some(email) = &warranty.client_email {
                    if let Err(e) = self.send_warranty_expiration_email(&warranty, days_until).await {
                        result.errors.push(format!("Failed to send warranty alert for {}: {}", warranty.asset_name, e));
                    } else {
                        result.alerts_sent += 1;
                    }
                }

                sqlx::query("UPDATE assets SET last_notification_date = $2 WHERE id = $1")
                    .bind(warranty.id)
                    .bind(today)
                    .execute(&self.db_pool)
                    .await?;

                self.log_expiration_alert(&alert).await?;
            }
        }

        Ok(())
    }

    fn should_notify(&self, last_notification: &Option<NaiveDate>, days_until: i32, thresholds: &[i32]) -> bool {
        // Check if current days_until matches any threshold
        if !thresholds.contains(&days_until) {
            return false;
        }

        // If never notified, send notification
        if last_notification.is_none() {
            return true;
        }

        // If last notification was more than 1 day ago, send again for this threshold
        let today = Utc::now().date_naive();
        if let Some(last) = last_notification {
            let days_since_notification = (today - *last).num_days();
            return days_since_notification >= 1;
        }

        false
    }

    async fn send_domain_expiration_email(&self, domain: &DomainExpiry, days_until: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let urgency_class = self.get_urgency_class(days_until);
        let subject = format!(
            "[{}] Domain {} expires in {} days",
            urgency_class.0, domain.domain_name, days_until
        );

        let html_body = self.build_expiration_email(
            "Domain Expiration",
            &domain.domain_name,
            &domain.client_name,
            domain.expiration_date,
            days_until,
            &urgency_class,
            vec![
                ("Registrar", domain.registrar.as_deref().unwrap_or("Unknown")),
                ("Auto-Renew", if domain.auto_renew { "Enabled" } else { "Disabled" }),
            ],
            "Ensure domain renewal is processed before expiration to avoid service disruption.",
        );

        self.email_service.send_email(
            domain.client_email.as_deref().unwrap_or(""),
            Some(&domain.client_name),
            &subject,
            &html_body,
            None
        ).await?;

        Ok(())
    }

    async fn send_ssl_expiration_email(&self, cert: &SslExpiry, days_until: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let urgency_class = self.get_urgency_class(days_until);
        let subject = format!(
            "[{}] SSL Certificate for {} expires in {} days",
            urgency_class.0, cert.domain, days_until
        );

        let html_body = self.build_expiration_email(
            "SSL Certificate Expiration",
            &cert.domain,
            &cert.client_name,
            cert.expiry_date,
            days_until,
            &urgency_class,
            vec![
                ("Issuer", cert.issuer.as_deref().unwrap_or("Unknown")),
                ("Certificate Type", cert.cert_type.as_deref().unwrap_or("Standard")),
            ],
            "Renew your SSL certificate before expiration to maintain secure connections.",
        );

        self.email_service.send_email(
            cert.client_email.as_deref().unwrap_or(""),
            Some(&cert.client_name),
            &subject,
            &html_body,
            None
        ).await?;

        Ok(())
    }

    async fn send_license_expiration_email(&self, license: &LicenseExpiry, days_until: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let urgency_class = self.get_urgency_class(days_until);
        let subject = format!(
            "[{}] {} license expires in {} days",
            urgency_class.0, license.software_name, days_until
        );

        let cost_str = license.annual_cost
            .map(|c| format!("${}", c))
            .unwrap_or_else(|| "N/A".to_string());

        let html_body = self.build_expiration_email(
            "Software License Expiration",
            &license.software_name,
            &license.client_name,
            license.expiration_date,
            days_until,
            &urgency_class,
            vec![
                ("Vendor", license.vendor.as_deref().unwrap_or("Unknown")),
                ("License Count", &license.license_count.map(|c| c.to_string()).unwrap_or("N/A".to_string())),
                ("Annual Cost", &cost_str),
            ],
            "Contact your vendor to renew the license before expiration.",
        );

        self.email_service.send_email(
            license.client_email.as_deref().unwrap_or(""),
            Some(&license.client_name),
            &subject,
            &html_body,
            None
        ).await?;

        Ok(())
    }

    async fn send_warranty_expiration_email(&self, warranty: &WarrantyExpiry, days_until: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let urgency_class = self.get_urgency_class(days_until);
        let subject = format!(
            "[{}] Warranty for {} expires in {} days",
            urgency_class.0, warranty.asset_name, days_until
        );

        let html_body = self.build_expiration_email(
            "Warranty Expiration",
            &warranty.asset_name,
            &warranty.client_name,
            warranty.warranty_expiry,
            days_until,
            &urgency_class,
            vec![
                ("Asset Type", &warranty.asset_type),
                ("Manufacturer", warranty.manufacturer.as_deref().unwrap_or("Unknown")),
                ("Serial Number", warranty.serial_number.as_deref().unwrap_or("N/A")),
            ],
            "Consider extended warranty options or plan for potential replacement.",
        );

        self.email_service.send_email(
            warranty.client_email.as_deref().unwrap_or(""),
            Some(&warranty.client_name),
            &subject,
            &html_body,
            None
        ).await?;

        Ok(())
    }

    fn get_urgency_class(&self, days: i32) -> (String, String, String) {
        if days <= 7 {
            ("CRITICAL".to_string(), "#dc2626".to_string(), "#fef2f2".to_string())
        } else if days <= 14 {
            ("URGENT".to_string(), "#f97316".to_string(), "#fff7ed".to_string())
        } else if days <= 30 {
            ("WARNING".to_string(), "#eab308".to_string(), "#fefce8".to_string())
        } else {
            ("NOTICE".to_string(), "#3b82f6".to_string(), "#eff6ff".to_string())
        }
    }

    fn build_expiration_email(
        &self,
        title: &str,
        item_name: &str,
        client_name: &str,
        expiration_date: NaiveDate,
        days_until: i32,
        urgency: &(String, String, String),
        details: Vec<(&str, &str)>,
        action_text: &str,
    ) -> String {
        let details_html: String = details.iter()
            .map(|(label, value)| {
                format!(r#"<tr><td style="padding: 8px 12px; color: #6b7280;">{}</td><td style="padding: 8px 12px; font-weight: 600; color: #111827;">{}</td></tr>"#, label, value)
            })
            .collect();

        format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
                    .container {{ max-width: 600px; margin: 0 auto; background: white; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 12px rgba(0,0,0,0.1); }}
                    .header {{ background: {}; color: white; padding: 24px; text-align: center; }}
                    .urgency-badge {{ display: inline-block; background: rgba(255,255,255,0.2); padding: 4px 12px; border-radius: 999px; font-size: 12px; font-weight: 600; margin-bottom: 8px; }}
                    .content {{ padding: 24px; }}
                    .alert-box {{ background: {}; border-left: 4px solid {}; padding: 16px; margin-bottom: 20px; border-radius: 0 8px 8px 0; }}
                    .countdown {{ font-size: 48px; font-weight: 700; color: {}; text-align: center; margin: 20px 0; }}
                    .countdown-label {{ text-align: center; color: #6b7280; margin-bottom: 20px; }}
                    table {{ width: 100%; border-collapse: collapse; }}
                    .footer {{ background: #f9fafb; padding: 16px 24px; text-align: center; color: #6b7280; font-size: 14px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <span class="urgency-badge">{}</span>
                        <h1 style="margin: 0; font-size: 24px;">{}</h1>
                    </div>
                    <div class="content">
                        <div class="alert-box">
                            <strong>{}</strong> for <strong>{}</strong> is expiring soon.
                        </div>

                        <div class="countdown">{}</div>
                        <div class="countdown-label">days until expiration</div>

                        <h3 style="color: #111827; margin-bottom: 12px;">Details</h3>
                        <table style="background: #f9fafb; border-radius: 8px;">
                            <tr><td style="padding: 8px 12px; color: #6b7280;">Item</td><td style="padding: 8px 12px; font-weight: 600; color: #111827;">{}</td></tr>
                            <tr><td style="padding: 8px 12px; color: #6b7280;">Client</td><td style="padding: 8px 12px; font-weight: 600; color: #111827;">{}</td></tr>
                            <tr><td style="padding: 8px 12px; color: #6b7280;">Expiration Date</td><td style="padding: 8px 12px; font-weight: 600; color: {};">{}</td></tr>
                            {}
                        </table>

                        <p style="margin-top: 20px; padding: 16px; background: #f0f9ff; border-radius: 8px; color: #0369a1;">
                            <strong>Action Required:</strong> {}
                        </p>
                    </div>
                    <div class="footer">
                        <p>Resolve MSP Platform - Expiration Monitoring</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            urgency.1, // header bg
            urgency.2, // alert box bg
            urgency.1, // alert box border
            urgency.1, // countdown color
            urgency.0, // urgency badge text
            title,
            title,
            client_name,
            days_until,
            item_name,
            client_name,
            urgency.1, // date color
            expiration_date.format("%B %d, %Y"),
            details_html,
            action_text
        )
    }

    async fn log_expiration_alert(&self, alert: &ExpirationAlert) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO expiration_alerts (id, item_type, item_name, client_id, expiration_date, days_until_expiry, details, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            ON CONFLICT (item_type, item_name, client_id, expiration_date) DO UPDATE
            SET days_until_expiry = $6, updated_at = NOW()
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&alert.item_type)
        .bind(&alert.item_name)
        .bind(alert.client_id)
        .bind(alert.expiration_date)
        .bind(alert.days_until_expiry)
        .bind(&alert.details)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}
