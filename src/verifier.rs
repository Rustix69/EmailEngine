use crate::models::{ValidateBatchResponse, ValidateResponse};
use anyhow::{Context, Result};
use std::sync::OnceLock;
use std::time::Duration;

const VERIFIER_URL: &str = "https://rapid-email-verifier.fly.dev/api/validate";
const VERIFIER_BATCH_URL: &str = "https://rapid-email-verifier.fly.dev/api/validate/batch";

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

/// Same checks for many addresses in one request; results come back in the order given
pub async fn verify_batch(emails: &[String]) -> Result<Vec<ValidateResponse>> {
    let batch: ValidateBatchResponse = client()
        .post(VERIFIER_BATCH_URL)
        .json(&serde_json::json!({ "emails": emails }))
        .send()
        .await
        .context("could not reach the email verifier")?
        .error_for_status()
        .context("email verifier returned an error")?
        .json()
        .await
        .context("email verifier returned an unexpected response")?;
    Ok(batch.results)
}
