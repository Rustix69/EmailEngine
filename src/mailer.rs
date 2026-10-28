use crate::config::Config;
use crate::models::{EmailResult, OutgoingMessage};
use crate::sent_log::log_sent;
use crate::validation::is_valid_email;
use anyhow::Result;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use tokio::time::{timeout, Duration};
use std::sync::Arc;

/// Gap between messages so Gmail doesn't flag a burst of connections
const SEND_GAP: Duration = Duration::from_millis(1500);

/// Only one batch runs at a time, so overlapping requests don't send in parallel
static BATCH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn send_email(cfg: &Config, to: &str, subject: &str, body: &str, html: Option<&str>) -> Result<()> {
    let sender = Mailbox::new(Some(cfg.sender_name.clone()), cfg.email.parse()?);
    let builder = Message::builder()
        .from(sender)
        .to(to.parse()?)
        .subject(subject);

    let email = if let Some(html_content) = html {
        builder.multipart(
            lettre::message::MultiPart::alternative_plain_html(
                body.to_string(),
                html_content.to_string(),
            )
        )?
    } else {
        builder.body(body.to_string())?
    };

    let creds = Credentials::new(cfg.email.clone(), cfg.app_password.clone());

    let tls_parameters = TlsParameters::new(cfg.smtp_server.clone())?;
    
    let mailer: AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_server)?
        .port(cfg.smtp_port)
        .tls(Tls::Required(tls_parameters))
        .credentials(creds)
        .build();

    mailer.send(email).await?;
    // Every endpoint sends through here, so the log lists every mail sent, however it was triggered
    log_sent(to);
    println!("Email sent to {}", to);
    Ok(())
}

pub async fn send_bulk_emails(
    cfg: &Arc<Config>,
    recipients: &[String],
    subject: &str,
    body: &str,
    html: Option<&str>,
) -> Vec<EmailResult> {
    let mut tasks = Vec::new();
    
    // Create concurrent tasks for sending emails
    for recipient in recipients {
        let cfg = Arc::clone(cfg);
        let recipient = recipient.clone();
        let subject = subject.to_string();
        let body = body.to_string();
        let html = html.map(|h| h.to_string());
        
        let task = tokio::spawn(async move {
            let result = timeout(
                Duration::from_secs(30), // 30 second timeout per email
                send_email(&cfg, &recipient, &subject, &body, html.as_deref())
            ).await;
            
            match result {
                Ok(Ok(_)) => EmailResult {
                    email: recipient,
                    success: true,
                    message: "Email sent successfully".to_string(),
                },
                Ok(Err(e)) => EmailResult {
                    email: recipient,
                    success: false,
                    message: format!("Failed to send email: {}", e),
                },
                Err(_) => EmailResult {
                    email: recipient,
                    success: false,
                    message: "Email sending timed out".to_string(),
                },
            }
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(result) => results.push(result),
            Err(e) => {
                // This shouldn't happen unless the task panics
                results.push(EmailResult {
                    email: "unknown".to_string(),
                    success: false,
                    message: format!("Task error: {}", e),
                });
            }
        }
    }
    
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;
    
    println!("Bulk email sending completed: {} successful, {} failed", successful, failed);
    
    results
}

/// Sends different messages one after another (each person gets their own subject and body)
pub async fn send_batch(cfg: &Arc<Config>, messages: &[OutgoingMessage]) -> Vec<EmailResult> {
    let _guard = BATCH_LOCK.lock().await;
    let mut results = Vec::with_capacity(messages.len());
    let mut slept = false;

    for m in messages {
        let failure = |message: &str| EmailResult {
            email: m.to.clone(),
            success: false,
            message: message.to_string(),
        };

        if !is_valid_email(&m.to) {
            results.push(failure("Invalid email address format"));
            continue;
        }
        if m.subject.trim().is_empty() || m.body.trim().is_empty() {
            results.push(failure("Subject and body cannot be empty"));
            continue;
        }

        if slept {
            tokio::time::sleep(SEND_GAP).await;
        }
        slept = true;

        let sent = timeout(
            Duration::from_secs(30),
            send_email(cfg, &m.to, &m.subject, &m.body, m.html.as_deref()),
        )
        .await;

        results.push(match sent {
            Ok(Ok(_)) => EmailResult {
                email: m.to.clone(),
                success: true,
                message: "Email sent successfully".to_string(),
            },
            Ok(Err(e)) => failure(&format!("Failed to send email: {}", e)),
            Err(_) => failure("Email sending timed out"),
        });
    }

    let successful = results.iter().filter(|r| r.success).count();
    println!("Batch completed: {} sent, {} failed", successful, results.len() - successful);

    results
}
