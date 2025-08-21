use crate::config::Config;
use anyhow::Result;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};

pub async fn send_email(cfg: &Config, to: &str, subject: &str, body: &str, html: Option<&str>) -> Result<()> {
    let builder = Message::builder()
        .from(cfg.email.parse()?)
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
    println!("Email sent to {}", to);
    Ok(())
}
