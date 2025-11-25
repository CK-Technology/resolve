use crate::config::SmtpConfig;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::{authentication::Credentials, PoolConfig},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct EmailService {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
    from_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketNotificationTemplate {
    pub ticket_number: i32,
    pub subject: String,
    pub client_name: String,
    pub priority: String,
    pub status: String,
    pub created_by: String,
    pub portal_url: String,
}

impl EmailService {
    pub async fn new(smtp_config: &SmtpConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let creds = Credentials::new(
            smtp_config.username.clone(),
            smtp_config.password.clone(),
        );

        let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp_config.host)
            .port(smtp_config.port)
            .credentials(creds)
            .pool_config(PoolConfig::new().max_size(10))
            .timeout(Some(Duration::from_secs(10)))
            .build();

        Ok(EmailService {
            transport,
            from_email: smtp_config.from_email.clone(),
            from_name: smtp_config.from_name.clone(),
        })
    }

    pub async fn send_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        subject: &str,
        html_body: &str,
        text_body: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let from = format!("{} <{}>", self.from_name, self.from_email)
            .parse::<Mailbox>()?;

        let to = if let Some(name) = to_name {
            format!("{} <{}>", name, to_email).parse::<Mailbox>()?
        } else {
            to_email.parse::<Mailbox>()?
        };

        let mut message_builder = Message::builder()
            .from(from)
            .to(to)
            .subject(subject);

        if let Some(text) = text_body {
            message_builder = message_builder
                .multipart(
                    lettre::message::MultiPart::alternative()
                        .singlepart(
                            lettre::message::SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(text.to_string()),
                        )
                        .singlepart(
                            lettre::message::SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html_body.to_string()),
                        ),
                )?;
        } else {
            message_builder = message_builder.body(html_body.to_string())?;
        }

        let message = message_builder;

        match self.transport.send(message).await {
            Ok(_) => {
                info!("Email sent successfully to {}", to_email);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send email to {}: {}", to_email, e);
                Err(Box::new(e))
            }
        }
    }

    // Template for new ticket notifications
    pub fn ticket_created_template(&self, data: &TicketNotificationTemplate) -> EmailTemplate {
        let subject = format!("New Ticket #{} - {}", data.ticket_number, data.subject);
        
        let html_body = format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
                    .container {{ max-width: 600px; margin: 0 auto; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                    .header {{ background: #2563eb; color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 30px; }}
                    .ticket-info {{ background: #f8fafc; border-left: 4px solid #2563eb; padding: 15px; margin: 20px 0; }}
                    .footer {{ background: #f8fafc; padding: 20px; text-align: center; color: #666; }}
                    .btn {{ display: inline-block; background: #2563eb; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin: 10px 0; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>🎫 New Support Ticket Created</h1>
                    </div>
                    <div class="content">
                        <p>Hello {},</p>
                        <p>A new support ticket has been created for your account.</p>
                        
                        <div class="ticket-info">
                            <h3>Ticket Details</h3>
                            <p><strong>Ticket #:</strong> {}</p>
                            <p><strong>Subject:</strong> {}</p>
                            <p><strong>Priority:</strong> {}</p>
                            <p><strong>Status:</strong> {}</p>
                            <p><strong>Created by:</strong> {}</p>
                        </div>
                        
                        <p>You can view and manage this ticket through our client portal:</p>
                        
                        <a href="{}" class="btn">View Ticket in Portal</a>
                        
                        <p>Our support team will review your ticket and respond as soon as possible.</p>
                        
                        <p>Best regards,<br>The Resolve Support Team</p>
                    </div>
                    <div class="footer">
                        <p>This is an automated message. Please do not reply directly to this email.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            data.client_name,
            data.ticket_number,
            data.subject,
            data.priority,
            data.status,
            data.created_by,
            data.portal_url
        );

        let text_body = format!(
            "New Support Ticket Created\n\n\
            Hello {},\n\n\
            A new support ticket has been created for your account.\n\n\
            Ticket Details:\n\
            - Ticket #: {}\n\
            - Subject: {}\n\
            - Priority: {}\n\
            - Status: {}\n\
            - Created by: {}\n\n\
            You can view this ticket at: {}\n\n\
            Our support team will review your ticket and respond as soon as possible.\n\n\
            Best regards,\n\
            The Resolve Support Team",
            data.client_name,
            data.ticket_number,
            data.subject,
            data.priority,
            data.status,
            data.created_by,
            data.portal_url
        );

        EmailTemplate {
            subject,
            html_body,
            text_body: Some(text_body),
        }
    }

    // Template for ticket updates
    pub fn ticket_updated_template(&self, data: &TicketNotificationTemplate, update_message: &str) -> EmailTemplate {
        let subject = format!("Ticket #{} Updated - {}", data.ticket_number, data.subject);
        
        let html_body = format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
                    .container {{ max-width: 600px; margin: 0 auto; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                    .header {{ background: #059669; color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 30px; }}
                    .ticket-info {{ background: #f0fdf4; border-left: 4px solid #059669; padding: 15px; margin: 20px 0; }}
                    .update-message {{ background: #fef3c7; border: 1px solid #f59e0b; padding: 15px; border-radius: 6px; margin: 20px 0; }}
                    .footer {{ background: #f8fafc; padding: 20px; text-align: center; color: #666; }}
                    .btn {{ display: inline-block; background: #059669; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin: 10px 0; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>📝 Ticket Updated</h1>
                    </div>
                    <div class="content">
                        <p>Hello {},</p>
                        <p>Your support ticket has been updated.</p>
                        
                        <div class="ticket-info">
                            <h3>Ticket Details</h3>
                            <p><strong>Ticket #:</strong> {}</p>
                            <p><strong>Subject:</strong> {}</p>
                            <p><strong>Status:</strong> {}</p>
                        </div>
                        
                        <div class="update-message">
                            <h4>Update:</h4>
                            <p>{}</p>
                        </div>
                        
                        <a href="{}" class="btn">View Ticket in Portal</a>
                        
                        <p>Best regards,<br>The Resolve Support Team</p>
                    </div>
                    <div class="footer">
                        <p>This is an automated message. Please do not reply directly to this email.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            data.client_name,
            data.ticket_number,
            data.subject,
            data.status,
            update_message,
            data.portal_url
        );

        EmailTemplate {
            subject,
            html_body,
            text_body: None,
        }
    }
}