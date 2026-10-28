use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EmailRequest {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub html: Option<String>,
}

#[derive(Deserialize)]
pub struct BulkEmailRequest {
    pub recipients: Vec<String>,
    pub subject: String,
    pub body: String,
    pub html: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutgoingMessage {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub html: Option<String>,
}

#[derive(Deserialize)]
pub struct BatchEmailRequest {
    pub messages: Vec<OutgoingMessage>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EmailResult {
    pub email: String,
    pub success: bool,
    pub message: String,
}

#[derive(Serialize)]
pub struct BulkEmailResponse {
    pub total_emails: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<EmailResult>,
}

#[derive(Serialize)]
pub struct EmailResponse {
    pub success: bool,
    pub message: String,
    pub email_sent_to: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct SentResponse {
    pub addresses: Vec<String>,
}

#[derive(Deserialize)]
pub struct ScheduleBatchRequest {
    /// RFC 3339 with an offset, e.g. `2026-11-02T10:30:00+05:30`
    pub send_at: chrono::DateTime<chrono::Utc>,
    pub messages: Vec<OutgoingMessage>,
}

#[derive(Serialize)]
pub struct ApiError {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct ValidateQuery {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct EmailChecks {
    pub syntax: bool,
    pub domain_exists: bool,
    pub mx_records: bool,
    pub mailbox_exists: bool,
    pub is_disposable: bool,
    pub is_role_based: bool,
}

/// Same shape as the upstream verifier, so callers see one stable format even if we change provider later
#[derive(Serialize, Deserialize)]
pub struct ValidateResponse {
    pub email: String,
    pub validations: EmailChecks,
    pub score: u32,
    pub status: String,
    /// Canonical address when this is an alias (e.g. `user.name+tag@gmail.com` -> `username@gmail.com`)
    #[serde(rename = "aliasOf", default, skip_serializing_if = "Option::is_none")]
    pub alias_of: Option<String>,
    /// Likely intended address when the domain looks mistyped (e.g. `gmial.com`)
    #[serde(rename = "typoSuggestion", default, skip_serializing_if = "Option::is_none")]
    pub typo_suggestion: Option<String>,
}

#[derive(Deserialize)]
pub struct ValidateBatchRequest {
    pub emails: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ValidateBatchResponse {
    pub results: Vec<ValidateResponse>,
}
