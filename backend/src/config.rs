use std::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub jwt_secret: String,
    pub smtp: SmtpConfig,
    pub imap: Option<ImapConfig>,
}

/// SMTP configuration for sending emails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: bool,
}

/// IMAP configuration for receiving emails (email-to-ticket)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub mailbox: String,
    pub use_tls: bool,
    /// How often to check for new emails (seconds)
    pub poll_interval_secs: u64,
    /// Support email address (for matching incoming emails)
    pub support_email: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Parse IMAP config only if IMAP_HOST is set
        let imap = if env::var("IMAP_HOST").is_ok() {
            Some(ImapConfig {
                host: env::var("IMAP_HOST").unwrap_or_default(),
                port: env::var("IMAP_PORT")
                    .unwrap_or_else(|_| "993".to_string())
                    .parse()
                    .unwrap_or(993),
                username: env::var("IMAP_USERNAME").unwrap_or_default(),
                password: env::var("IMAP_PASSWORD").unwrap_or_default(),
                mailbox: env::var("IMAP_MAILBOX").unwrap_or_else(|_| "INBOX".to_string()),
                use_tls: env::var("IMAP_USE_TLS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                poll_interval_secs: env::var("IMAP_POLL_INTERVAL")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
                support_email: env::var("SUPPORT_EMAIL")
                    .unwrap_or_else(|_| "support@example.com".to_string()),
            })
        } else {
            None
        };

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://resolve:resolve@localhost/resolve".to_string()),
            server_addr: env::var("SERVER_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            smtp: SmtpConfig {
                // SMTP2GO configuration
                host: env::var("SMTP_HOST").unwrap_or_else(|_| "mail.smtp2go.com".to_string()),
                port: env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "2525".to_string())
                    .parse()
                    .unwrap_or(2525),
                username: env::var("SMTP_USERNAME").unwrap_or_default(),
                password: env::var("SMTP_PASSWORD").unwrap_or_default(),
                from_email: env::var("SMTP_FROM_EMAIL")
                    .unwrap_or_else(|_| "support@cktechx.com".to_string()),
                from_name: env::var("SMTP_FROM_NAME")
                    .unwrap_or_else(|_| "Resolve Support".to_string()),
                use_tls: env::var("SMTP_USE_TLS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            imap,
        })
    }
}

impl SmtpConfig {
    /// Check if SMTP is properly configured
    pub fn is_configured(&self) -> bool {
        !self.host.is_empty() && !self.username.is_empty() && !self.password.is_empty()
    }
}

impl ImapConfig {
    /// Check if IMAP is properly configured
    pub fn is_configured(&self) -> bool {
        !self.host.is_empty() && !self.username.is_empty() && !self.password.is_empty()
    }
}