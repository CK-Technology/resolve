use crate::models::domains_ssl::*;
use crate::services::EmailService;
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use rustls::pki_types::ServerName;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{interval, timeout};
use tracing::{error, info, warn, debug};
use uuid::Uuid;
use trust_dns_resolver::{Resolver, config::*};
use trust_dns_resolver::proto::rr::{RecordType, RData};

#[derive(Debug, Clone)]
pub struct DomainSslMonitorService {
    db_pool: PgPool,
    http_client: Client,
    email_service: EmailService,
    resolver: Resolver,
}

impl DomainSslMonitorService {
    pub fn new(db_pool: PgPool, email_service: EmailService) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .danger_accept_invalid_certs(false)
            .build()?;

        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;

        Ok(Self {
            db_pool,
            http_client,
            email_service,
            resolver,
        })
    }

    // Start the monitoring service
    pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting domain and SSL monitoring service");

        // Create monitoring tasks
        let domain_monitor = self.clone();
        let ssl_monitor = self.clone();
        let website_monitor = self.clone();
        let alert_processor = self.clone();

        tokio::spawn(async move {
            domain_monitor.run_domain_monitoring().await;
        });

        tokio::spawn(async move {
            ssl_monitor.run_ssl_monitoring().await;
        });

        tokio::spawn(async move {
            website_monitor.run_website_monitoring().await;
        });

        tokio::spawn(async move {
            alert_processor.run_alert_processing().await;
        });

        Ok(())
    }

    // Domain monitoring loop
    async fn run_domain_monitoring(&self) {
        let mut interval = interval(tokio::time::Duration::from_secs(3600)); // Check every hour
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_all_domains().await {
                error!("Error in domain monitoring: {}", e);
            }
        }
    }

    // SSL monitoring loop
    async fn run_ssl_monitoring(&self) {
        let mut interval = interval(tokio::time::Duration::from_secs(1800)); // Check every 30 minutes
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_all_ssl_certificates().await {
                error!("Error in SSL monitoring: {}", e);
            }
        }
    }

    // Website monitoring loop
    async fn run_website_monitoring(&self) {
        let mut interval = interval(tokio::time::Duration::from_secs(300)); // Check every 5 minutes
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_all_websites().await {
                error!("Error in website monitoring: {}", e);
            }
        }
    }

    // Alert processing loop
    async fn run_alert_processing(&self) {
        let mut interval = interval(tokio::time::Duration::from_secs(600)); // Check every 10 minutes
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.process_alerts().await {
                error!("Error processing alerts: {}", e);
            }
        }
    }

    // Check all domains for expiry and WHOIS updates
    pub async fn check_all_domains(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let domains = sqlx::query_as!(
            Domain,
            r#"
            SELECT id, client_id, name, registrar, nameservers, registration_date,
                   expiry_date, auto_renew, dns_records, notes, monitoring_enabled,
                   last_monitored, monitoring_status, whois_data, created_at, updated_at
            FROM domains 
            WHERE monitoring_enabled = true 
            AND (last_monitored IS NULL OR last_monitored < NOW() - INTERVAL '24 hours')
            LIMIT 100
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        info!("Checking {} domains for updates", domains.len());

        for domain in domains {
            if let Err(e) = self.check_domain_status(&domain).await {
                error!("Error checking domain {}: {}", domain.name, e);
                
                // Update monitoring status to error
                sqlx::query!(
                    "UPDATE domains SET monitoring_status = 'error', last_monitored = NOW() WHERE id = $1",
                    domain.id
                )
                .execute(&self.db_pool)
                .await?;
            }
        }

        Ok(())
    }

    // Check individual domain status
    async fn check_domain_status(&self, domain: &Domain) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Checking domain: {}", domain.name);

        // Perform WHOIS lookup
        let whois_data = self.whois_lookup(&domain.name).await?;
        
        // Update domain with WHOIS data
        let whois_json = serde_json::to_value(&whois_data)?;
        
        sqlx::query!(
            r#"
            UPDATE domains 
            SET whois_data = $2, last_monitored = NOW(), monitoring_status = 'active',
                registrar = COALESCE($3, registrar),
                expiry_date = COALESCE($4, expiry_date)
            WHERE id = $1
            "#,
            domain.id,
            whois_json,
            whois_data.registrar,
            whois_data.expiry_date.map(|d| d.date_naive())
        )
        .execute(&self.db_pool)
        .await?;

        // Check for expiry alerts
        if let Some(expiry_date) = whois_data.expiry_date {
            let days_until_expiry = (expiry_date - Utc::now()).num_days();
            
            // Create alerts for domains expiring soon
            if days_until_expiry <= 30 && days_until_expiry > 0 {
                self.create_alert(
                    domain.client_id,
                    "domain_expiry",
                    "domain",
                    domain.id,
                    &format!("Domain {} expires in {} days", domain.name, days_until_expiry),
                    &format!("Domain {} is set to expire on {}. Please renew to avoid service interruption.", 
                            domain.name, expiry_date.format("%Y-%m-%d")),
                    if days_until_expiry <= 7 { "critical" } else { "warning" },
                    json!({
                        "domain": domain.name,
                        "expiry_date": expiry_date,
                        "days_until_expiry": days_until_expiry
                    })
                ).await?;
            } else if days_until_expiry <= 0 {
                self.create_alert(
                    domain.client_id,
                    "domain_expired",
                    "domain",
                    domain.id,
                    &format!("Domain {} has expired", domain.name),
                    &format!("Domain {} expired on {}. Immediate action required.", 
                            domain.name, expiry_date.format("%Y-%m-%d")),
                    "critical",
                    json!({
                        "domain": domain.name,
                        "expiry_date": expiry_date,
                        "days_since_expiry": -days_until_expiry
                    })
                ).await?;
            }
        }

        // Update DNS records
        self.update_dns_records(domain.id, &domain.name).await?;

        Ok(())
    }

    // Check all SSL certificates
    pub async fn check_all_ssl_certificates(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let certificates = sqlx::query_as!(
            SslCertificate,
            r#"
            SELECT id, domain_id, client_id, domain_name, port, issuer, subject,
                   serial_number, signature_algorithm, valid_from, valid_until,
                   is_wildcard, san_domains, monitoring_enabled, last_checked,
                   status, certificate_chain, fingerprint_sha1, fingerprint_sha256,
                   notes, created_at, updated_at
            FROM ssl_certificates 
            WHERE monitoring_enabled = true 
            AND (last_checked IS NULL OR last_checked < NOW() - INTERVAL '1 hour')
            LIMIT 100
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        info!("Checking {} SSL certificates", certificates.len());

        for cert in certificates {
            if let Err(e) = self.check_ssl_certificate(&cert).await {
                error!("Error checking SSL certificate for {}: {}", cert.domain_name, e);
                
                // Update status to error
                sqlx::query!(
                    "UPDATE ssl_certificates SET status = 'error', last_checked = NOW() WHERE id = $1",
                    cert.id
                )
                .execute(&self.db_pool)
                .await?;
            }
        }

        Ok(())
    }

    // Check individual SSL certificate
    async fn check_ssl_certificate(&self, cert: &SslCertificate) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Checking SSL certificate for: {}:{}", cert.domain_name, cert.port);

        // Get SSL certificate details
        let ssl_info = self.get_ssl_certificate_info(&cert.domain_name, cert.port as u16).await?;

        // Update certificate information
        sqlx::query!(
            r#"
            UPDATE ssl_certificates 
            SET issuer = $2, subject = $3, serial_number = $4, signature_algorithm = $5,
                valid_from = $6, valid_until = $7, is_wildcard = $8, san_domains = $9,
                status = $10, fingerprint_sha1 = $11, fingerprint_sha256 = $12,
                last_checked = NOW()
            WHERE id = $1
            "#,
            cert.id,
            ssl_info.issuer,
            ssl_info.subject,
            ssl_info.serial_number,
            ssl_info.signature_algorithm,
            ssl_info.valid_from,
            ssl_info.valid_until,
            ssl_info.is_wildcard,
            &ssl_info.san_domains,
            ssl_info.status,
            ssl_info.fingerprint_sha1,
            ssl_info.fingerprint_sha256
        )
        .execute(&self.db_pool)
        .await?;

        // Check for expiry alerts
        if let Some(valid_until) = ssl_info.valid_until {
            let days_until_expiry = (valid_until - Utc::now()).num_days();
            
            if days_until_expiry <= 30 && days_until_expiry > 0 {
                self.create_alert(
                    cert.client_id,
                    "ssl_expiry",
                    "ssl_certificate",
                    cert.id,
                    &format!("SSL certificate for {} expires in {} days", cert.domain_name, days_until_expiry),
                    &format!("SSL certificate for {} is set to expire on {}. Please renew to avoid security warnings.", 
                            cert.domain_name, valid_until.format("%Y-%m-%d")),
                    if days_until_expiry <= 7 { "critical" } else { "warning" },
                    json!({
                        "domain": cert.domain_name,
                        "port": cert.port,
                        "expiry_date": valid_until,
                        "days_until_expiry": days_until_expiry,
                        "issuer": ssl_info.issuer
                    })
                ).await?;
            } else if days_until_expiry <= 0 {
                self.create_alert(
                    cert.client_id,
                    "ssl_expired",
                    "ssl_certificate",
                    cert.id,
                    &format!("SSL certificate for {} has expired", cert.domain_name),
                    &format!("SSL certificate for {} expired on {}. Users will see security warnings.", 
                            cert.domain_name, valid_until.format("%Y-%m-%d")),
                    "critical",
                    json!({
                        "domain": cert.domain_name,
                        "port": cert.port,
                        "expiry_date": valid_until,
                        "days_since_expiry": -days_until_expiry,
                        "issuer": ssl_info.issuer
                    })
                ).await?;
            }
        }

        Ok(())
    }

    // Check all websites
    pub async fn check_all_websites(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let websites = sqlx::query_as!(
            Website,
            r#"
            SELECT id, client_id, domain_id, name, url, expected_status_code,
                   monitoring_enabled, check_interval_minutes, timeout_seconds,
                   last_checked, status, response_time_ms, status_code,
                   response_headers, downtime_alerts_enabled, performance_alerts_enabled,
                   notes, created_at, updated_at
            FROM websites 
            WHERE monitoring_enabled = true 
            AND (last_checked IS NULL OR last_checked < NOW() - (check_interval_minutes || ' minutes')::INTERVAL)
            LIMIT 100
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        info!("Checking {} websites", websites.len());

        for website in websites {
            if let Err(e) = self.check_website(&website).await {
                error!("Error checking website {}: {}", website.url, e);
            }
        }

        Ok(())
    }

    // Check individual website
    async fn check_website(&self, website: &Website) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Checking website: {}", website.url);

        let start_time = std::time::Instant::now();
        
        let check_result = timeout(
            std::time::Duration::from_secs(website.timeout_seconds as u64),
            self.http_client.get(&website.url).send()
        ).await;

        let response_time_ms = start_time.elapsed().as_millis() as i32;
        let mut status = "down".to_string();
        let mut status_code = None;
        let mut error_message = None;
        let mut response_headers = None;

        match check_result {
            Ok(Ok(response)) => {
                status_code = Some(response.status().as_u16() as i32);
                
                if status_code == Some(website.expected_status_code) {
                    status = "up".to_string();
                } else {
                    status = "warning".to_string();
                    error_message = Some(format!("Expected status {}, got {}", 
                                               website.expected_status_code, 
                                               status_code.unwrap_or(0)));
                }

                // Capture response headers
                let headers: HashMap<String, String> = response.headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                response_headers = Some(serde_json::to_value(headers).unwrap_or_default());
            }
            Ok(Err(e)) => {
                status = "down".to_string();
                error_message = Some(e.to_string());
            }
            Err(_) => {
                status = "timeout".to_string();
                error_message = Some("Request timed out".to_string());
            }
        }

        // Update website status
        sqlx::query!(
            r#"
            UPDATE websites 
            SET status = $2, response_time_ms = $3, status_code = $4, 
                response_headers = $5, last_checked = NOW()
            WHERE id = $1
            "#,
            website.id,
            status,
            response_time_ms,
            status_code,
            response_headers
        )
        .execute(&self.db_pool)
        .await?;

        // Record check in history
        sqlx::query!(
            r#"
            INSERT INTO website_checks (website_id, status_code, response_time_ms, status, error_message, response_headers)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            website.id,
            status_code,
            response_time_ms,
            status,
            error_message,
            response_headers
        )
        .execute(&self.db_pool)
        .await?;

        // Create alerts if needed
        if status == "down" && website.downtime_alerts_enabled {
            self.create_alert(
                website.client_id,
                "website_down",
                "website",
                website.id,
                &format!("Website {} is down", website.name),
                &format!("Website {} ({}) is not responding. Error: {}", 
                        website.name, website.url, error_message.unwrap_or_else(|| "Unknown error".to_string())),
                "critical",
                json!({
                    "website": website.name,
                    "url": website.url,
                    "status_code": status_code,
                    "response_time_ms": response_time_ms,
                    "error": error_message
                })
            ).await?;
        } else if response_time_ms > 5000 && website.performance_alerts_enabled {
            self.create_alert(
                website.client_id,
                "website_slow",
                "website",
                website.id,
                &format!("Website {} is slow", website.name),
                &format!("Website {} ({}) is responding slowly ({}ms). This may indicate performance issues.", 
                        website.name, website.url, response_time_ms),
                "warning",
                json!({
                    "website": website.name,
                    "url": website.url,
                    "response_time_ms": response_time_ms,
                    "threshold_ms": 5000
                })
            ).await?;
        }

        Ok(())
    }

    // Create monitoring alert
    async fn create_alert(
        &self,
        client_id: Uuid,
        alert_type: &str,
        entity_type: &str,
        entity_id: Uuid,
        title: &str,
        message: &str,
        severity: &str,
        metadata: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if similar alert already exists
        let existing_alert = sqlx::query!(
            r#"
            SELECT id FROM monitoring_alerts 
            WHERE client_id = $1 AND alert_type = $2 AND entity_type = $3 AND entity_id = $4 
            AND status = 'active'
            "#,
            client_id,
            alert_type,
            entity_type,
            entity_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_alert.is_some() {
            // Update existing alert's last_detected timestamp
            sqlx::query!(
                r#"
                UPDATE monitoring_alerts 
                SET last_detected = NOW(), metadata = $2
                WHERE client_id = $1 AND alert_type = $3 AND entity_type = $4 AND entity_id = $5 
                AND status = 'active'
                "#,
                client_id,
                metadata,
                alert_type,
                entity_type,
                entity_id
            )
            .execute(&self.db_pool)
            .await?;
        } else {
            // Create new alert
            sqlx::query!(
                r#"
                INSERT INTO monitoring_alerts 
                (client_id, alert_type, entity_type, entity_id, title, message, severity, metadata)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                client_id,
                alert_type,
                entity_type,
                entity_id,
                title,
                message,
                severity,
                metadata
            )
            .execute(&self.db_pool)
            .await?;

            info!("Created alert: {} - {}", title, message);
        }

        Ok(())
    }

    // Process and send alerts
    async fn process_alerts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let alerts = sqlx::query_as!(
            MonitoringAlert,
            r#"
            SELECT ma.*, c.name as client_name, c.email as client_email
            FROM monitoring_alerts ma
            JOIN clients c ON ma.client_id = c.id
            WHERE ma.status = 'active' 
            AND ma.notification_sent = false
            AND ma.severity IN ('critical', 'warning')
            LIMIT 50
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        for alert in alerts {
            if let Err(e) = self.send_alert_notification(&alert).await {
                error!("Failed to send alert notification: {}", e);
            } else {
                // Mark as sent
                sqlx::query!(
                    "UPDATE monitoring_alerts SET notification_sent = true WHERE id = $1",
                    alert.id
                )
                .execute(&self.db_pool)
                .await?;
            }
        }

        Ok(())
    }

    // Send alert notification
    async fn send_alert_notification(&self, alert: &MonitoringAlert) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get client email from the alert query
        let client_info = sqlx::query!(
            "SELECT name, email FROM clients WHERE id = $1",
            alert.client_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        let subject = format!("[Resolve] {} Alert: {}", 
                             alert.severity.to_uppercase(), 
                             alert.title);

        let html_body = format!(
            r#"
            <html>
            <body>
                <h2 style="color: {};">Monitoring Alert</h2>
                <p><strong>Client:</strong> {}</p>
                <p><strong>Alert Type:</strong> {}</p>
                <p><strong>Severity:</strong> {}</p>
                <p><strong>Entity:</strong> {} ({})</p>
                <p><strong>Message:</strong></p>
                <p>{}</p>
                <p><strong>First Detected:</strong> {}</p>
                <p><strong>Last Detected:</strong> {}</p>
                <hr>
                <p><small>This is an automated alert from your Resolve monitoring system.</small></p>
            </body>
            </html>
            "#,
            match alert.severity.as_str() {
                "critical" => "#dc3545",
                "warning" => "#ffc107", 
                _ => "#6c757d"
            },
            client_info.name,
            alert.alert_type,
            alert.severity,
            alert.entity_type,
            alert.entity_id,
            alert.message,
            alert.first_detected.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.last_detected.format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.email_service
            .send_email(&client_info.email, Some(&client_info.name), &subject, &html_body, None)
            .await?;

        info!("Sent alert notification to {}: {}", client_info.email, alert.title);
        Ok(())
    }

    // WHOIS lookup implementation
    async fn whois_lookup(&self, domain: &str) -> Result<WhoisResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified WHOIS implementation - in production you'd want a proper WHOIS library
        // For now, return mock data
        Ok(WhoisResponse {
            domain: domain.to_string(),
            registrar: Some("Mock Registrar".to_string()),
            creation_date: Some(Utc::now() - Duration::days(365)),
            expiry_date: Some(Utc::now() + Duration::days(30)),
            updated_date: Some(Utc::now() - Duration::days(30)),
            nameservers: vec!["ns1.example.com".to_string(), "ns2.example.com".to_string()],
            status: vec!["clientTransferProhibited".to_string()],
            raw_data: "Mock WHOIS data".to_string(),
        })
    }

    // Get SSL certificate information
    async fn get_ssl_certificate_info(&self, domain: &str, port: u16) -> Result<SslCertInfo, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified SSL check - in production you'd want a proper TLS library
        // Return mock data for now
        Ok(SslCertInfo {
            issuer: Some("Mock Certificate Authority".to_string()),
            subject: Some(format!("CN={}", domain)),
            serial_number: Some("123456789".to_string()),
            signature_algorithm: Some("SHA256withRSA".to_string()),
            valid_from: Some(Utc::now() - Duration::days(30)),
            valid_until: Some(Utc::now() + Duration::days(60)),
            is_wildcard: domain.starts_with("*."),
            san_domains: vec![domain.to_string()],
            status: "valid".to_string(),
            fingerprint_sha1: Some("AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD".to_string()),
            fingerprint_sha256: Some("AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99".to_string()),
        })
    }

    // Update DNS records for a domain
    async fn update_dns_records(&self, domain_id: Uuid, domain: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Look up common DNS records
        let record_types = vec!["A", "AAAA", "CNAME", "MX", "TXT", "NS"];
        
        for record_type in record_types {
            if let Ok(records) = self.dns_lookup(domain, record_type).await {
                for record in records.records {
                    // Insert or update DNS record
                    sqlx::query!(
                        r#"
                        INSERT INTO dns_records (domain_id, record_type, name, value, ttl, last_checked, status)
                        VALUES ($1, $2, $3, $4, $5, NOW(), 'valid')
                        ON CONFLICT (domain_id, record_type, name, value) 
                        DO UPDATE SET last_checked = NOW(), status = 'valid'
                        "#,
                        domain_id,
                        record.record_type,
                        record.name,
                        record.value,
                        record.ttl
                    )
                    .execute(&self.db_pool)
                    .await?;
                }
            }
        }

        Ok(())
    }

    // DNS lookup
    async fn dns_lookup(&self, domain: &str, record_type: &str) -> Result<DnsLookupResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified DNS lookup - mock implementation
        Ok(DnsLookupResponse {
            domain: domain.to_string(),
            record_type: record_type.to_string(),
            records: vec![
                DnsLookupResult {
                    name: domain.to_string(),
                    record_type: record_type.to_string(),
                    value: "1.2.3.4".to_string(),
                    ttl: Some(300),
                    priority: None,
                }
            ],
            nameservers: vec!["ns1.example.com".to_string()],
        })
    }
}

#[derive(Debug)]
struct SslCertInfo {
    issuer: Option<String>,
    subject: Option<String>,
    serial_number: Option<String>,
    signature_algorithm: Option<String>,
    valid_from: Option<DateTime<Utc>>,
    valid_until: Option<DateTime<Utc>>,
    is_wildcard: bool,
    san_domains: Vec<String>,
    status: String,
    fingerprint_sha1: Option<String>,
    fingerprint_sha256: Option<String>,
}