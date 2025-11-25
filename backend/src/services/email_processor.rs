use crate::services::EmailService;
use imap::types::{Fetch, Flag};
use lettre::message::Mailbox;
use mail_parser::{Message, MessageParser};
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EmailProcessorConfig {
    pub imap_host: String,
    pub imap_port: u16,
    pub username: String,
    pub password: String,
    pub mailbox: String,
    pub check_interval_seconds: u64,
    pub support_email: String,
    pub portal_base_url: String,
}

#[derive(Debug)]
pub struct EmailProcessor {
    config: EmailProcessorConfig,
    db_pool: PgPool,
    email_service: EmailService,
}

#[derive(Debug)]
struct ParsedEmail {
    from: String,
    from_name: Option<String>,
    subject: String,
    body_text: String,
    body_html: Option<String>,
    message_id: Option<String>,
    in_reply_to: Option<String>,
    references: Vec<String>,
}

impl EmailProcessor {
    pub fn new(
        config: EmailProcessorConfig,
        db_pool: PgPool,
        email_service: EmailService,
    ) -> Self {
        Self {
            config,
            db_pool,
            email_service,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting email processor for {}", self.config.support_email);
        
        let mut interval = interval(Duration::from_secs(self.config.check_interval_seconds));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.process_emails().await {
                error!("Error processing emails: {}", e);
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        }
    }

    async fn process_emails(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Checking for new emails...");

        // Connect to IMAP server
        let tls = native_tls::TlsConnector::builder().build()?;
        let client = imap::connect(
            (&*self.config.imap_host, self.config.imap_port),
            &self.config.imap_host,
            &tls,
        )?;

        // Login
        let mut imap_session = client
            .login(&self.config.username, &self.config.password)
            .map_err(|e| e.0)?;

        // Select mailbox
        imap_session.select(&self.config.mailbox)?;

        // Search for unread emails
        let messages = imap_session.search("UNSEEN")?;
        
        info!("Found {} unread messages", messages.len());

        for &message_id in &messages {
            match self.process_single_message(&mut imap_session, message_id).await {
                Ok(processed) => {
                    if processed {
                        // Mark as read
                        imap_session.store(format!("{}", message_id), "+FLAGS (\\Seen)")?;
                        info!("Processed and marked message {} as read", message_id);
                    }
                }
                Err(e) => {
                    error!("Failed to process message {}: {}", message_id, e);
                }
            }
        }

        // Logout
        imap_session.logout()?;
        Ok(())
    }

    async fn process_single_message(
        &self,
        imap_session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
        message_id: u32,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Fetch message
        let messages = imap_session.fetch(format!("{}", message_id), "RFC822")?;
        
        if let Some(message) = messages.iter().next() {
            if let Some(body) = message.body() {
                let parsed_email = self.parse_email(body)?;
                
                // Check if this is a reply to an existing ticket
                if let Some(ticket_id) = self.extract_ticket_id_from_subject(&parsed_email.subject).await? {
                    self.add_reply_to_ticket(ticket_id, &parsed_email).await?;
                } else {
                    // Create new ticket
                    self.create_ticket_from_email(&parsed_email).await?;
                }
                
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    fn parse_email(&self, raw_email: &[u8]) -> Result<ParsedEmail, Box<dyn std::error::Error + Send + Sync>> {
        let message = MessageParser::default()
            .parse(raw_email)
            .ok_or("Failed to parse email")?;

        // Extract sender information
        let from = message
            .from()
            .and_then(|f| f.first())
            .map(|addr| addr.address().unwrap_or("unknown@unknown.com").to_string())
            .unwrap_or_else(|| "unknown@unknown.com".to_string());

        let from_name = message
            .from()
            .and_then(|f| f.first())
            .and_then(|addr| addr.name())
            .map(|name| name.to_string());

        // Extract subject
        let subject = message
            .subject()
            .unwrap_or("No Subject")
            .to_string();

        // Extract body
        let body_text = message
            .body_text(0)
            .unwrap_or("No text body")
            .to_string();

        let body_html = message
            .body_html(0)
            .map(|html| html.to_string());

        // Extract message threading headers
        let message_id = message
            .message_id()
            .map(|id| id.to_string());

        let in_reply_to = message
            .in_reply_to()
            .map(|id| id.to_string());

        let references = message
            .references()
            .map(|refs| refs.iter().map(|r| r.to_string()).collect())
            .unwrap_or_default();

        Ok(ParsedEmail {
            from,
            from_name,
            subject,
            body_text,
            body_html,
            message_id,
            in_reply_to,
            references,
        })
    }

    async fn extract_ticket_id_from_subject(
        &self,
        subject: &str,
    ) -> Result<Option<Uuid>, Box<dyn std::error::Error + Send + Sync>> {
        // Look for ticket patterns like "Re: [Ticket #123]" or "[#123]"
        let ticket_regex = regex::Regex::new(r"#(\d+)")?;
        
        if let Some(captures) = ticket_regex.captures(subject) {
            if let Some(ticket_number) = captures.get(1) {
                let ticket_num: i32 = ticket_number.as_str().parse()?;
                
                // Look up ticket by number
                let ticket = sqlx::query!(
                    "SELECT id FROM tickets WHERE ticket_number = $1",
                    ticket_num
                )
                .fetch_optional(&self.db_pool)
                .await?;
                
                return Ok(ticket.map(|t| t.id));
            }
        }
        
        Ok(None)
    }

    async fn create_ticket_from_email(
        &self,
        email: &ParsedEmail,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Creating new ticket from email: {}", email.subject);

        // Find or create contact based on email address
        let contact_id = self.find_or_create_contact(&email.from, email.from_name.as_deref()).await?;
        
        // Get client for this contact
        let client_info = sqlx::query!(
            "SELECT client_id, name FROM contacts WHERE id = $1",
            contact_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Generate ticket number
        let ticket_number = self.generate_ticket_number().await?;

        // Create ticket
        let ticket_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO tickets (id, client_id, contact_id, ticket_number, subject, details, 
                               priority, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'medium', 'open', NOW(), NOW())
            "#,
            ticket_id,
            client_info.client_id,
            contact_id,
            ticket_number,
            email.subject,
            email.body_text
        )
        .execute(&self.db_pool)
        .await?;

        info!("Created ticket #{} from email", ticket_number);

        // Send confirmation email
        self.send_ticket_confirmation_email(ticket_number, &email.from, email.from_name.as_deref(), &email.subject).await?;

        Ok(())
    }

    async fn add_reply_to_ticket(
        &self,
        ticket_id: Uuid,
        email: &ParsedEmail,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Adding reply to existing ticket: {}", ticket_id);

        // Find contact
        let contact_id = self.find_or_create_contact(&email.from, email.from_name.as_deref()).await?;

        // Add reply
        sqlx::query!(
            r#"
            INSERT INTO ticket_replies (id, ticket_id, contact_id, type, details, created_at)
            VALUES ($1, $2, $3, 'reply', $4, NOW())
            "#,
            Uuid::new_v4(),
            ticket_id,
            contact_id,
            email.body_text
        )
        .execute(&self.db_pool)
        .await?;

        // Update ticket timestamp
        sqlx::query!(
            "UPDATE tickets SET updated_at = NOW() WHERE id = $1",
            ticket_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn find_or_create_contact(
        &self,
        email: &str,
        name: Option<&str>,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        // Try to find existing contact
        if let Some(contact) = sqlx::query!(
            "SELECT id FROM contacts WHERE email = $1",
            email
        )
        .fetch_optional(&self.db_pool)
        .await?
        {
            return Ok(contact.id);
        }

        // Create new contact and client if needed
        let contact_name = name.unwrap_or(email);
        let client_name = format!("{} (Auto-created)", contact_name);

        // Create client first
        let client_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO clients (id, name, email, status, created_at, updated_at)
            VALUES ($1, $2, $3, 'active', NOW(), NOW())
            "#,
            client_id,
            client_name,
            email
        )
        .execute(&self.db_pool)
        .await?;

        // Create contact
        let contact_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO contacts (id, client_id, name, email, is_primary, created_at, updated_at)
            VALUES ($1, $2, $3, $4, true, NOW(), NOW())
            "#,
            contact_id,
            client_id,
            contact_name,
            email
        )
        .execute(&self.db_pool)
        .await?;

        info!("Created new client '{}' and contact for {}", client_name, email);
        Ok(contact_id)
    }

    async fn generate_ticket_number(&self) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query!(
            "SELECT COALESCE(MAX(ticket_number), 0) + 1 as next_number FROM tickets"
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(result.next_number.unwrap_or(1))
    }

    async fn send_ticket_confirmation_email(
        &self,
        ticket_number: i32,
        to_email: &str,
        to_name: Option<&str>,
        subject: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let email_subject = format!("[Ticket #{}] Ticket Created - {}", ticket_number, subject);
        
        let html_body = format!(
            r#"
            <h2>Support Ticket Created</h2>
            <p>Thank you for contacting our support team.</p>
            <p><strong>Ticket Number:</strong> #{}</p>
            <p><strong>Subject:</strong> {}</p>
            <p>Your ticket has been created and assigned to our support team. 
            We will review your request and respond as soon as possible.</p>
            <p>You can reference this ticket by including the ticket number #{} in any replies.</p>
            <p>Best regards,<br>Resolve Support Team</p>
            "#,
            ticket_number, subject, ticket_number
        );

        self.email_service
            .send_email(to_email, to_name, &email_subject, &html_body, None)
            .await?;

        Ok(())
    }
}