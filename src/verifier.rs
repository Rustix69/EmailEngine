use crate::models::ValidateResponse;
use anyhow::{Context, Result};
use std::sync::OnceLock;
use std::time::Duration;

const VERIFIER_URL: &str = "https://rapid-email-verifier.fly.dev/api/validate";

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build HTTP client")
    })
}

/// Asks the external verifier whether an address looks deliverable (syntax, domain, MX, mailbox, disposable, role-based)
pub async fn verify(email: &str) -> Result<ValidateResponse> {
    client()
        .get(VERIFIER_URL)
        .query(&[("email", email)])
        .send()
        .await
        .context("could not reach the email verifier")?
        .error_for_status()
        .context("email verifier returned an error")?
        .json()
        .await
        .context("email verifier returned an unexpected response")
}
