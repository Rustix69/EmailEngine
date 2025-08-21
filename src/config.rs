use anyhow::{Context, Result};
use std::env;

pub struct Config {
    pub email: String,
    pub app_password: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let email = env::var("EMAIL_ADDRESS").context("EMAIL_ADDRESS not set")?;
        let app_password = env::var("APP_PASSWORD").context("APP_PASSWORD not set")?;
        let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".into());
        let smtp_port = env::var("SMTP_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(587);
        let server_port = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(6969);

        Ok(Self { email, app_password, smtp_server, smtp_port, server_port })
    }
}
