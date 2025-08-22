use crate::{config::Config, email};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Serialize)]
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

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "rust-mailer-api".to_string(),
        version: "0.1.0".to_string(),
    })
}

pub async fn send_email_handler(
    State(config): State<Arc<Config>>,
    Json(request): Json<EmailRequest>,
) -> Result<Json<EmailResponse>, (StatusCode, Json<EmailResponse>)> {
    // Validate email format
    if !is_valid_email(&request.to) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(EmailResponse {
                success: false,
                message: "Invalid email address format".to_string(),
                email_sent_to: None,
            }),
        ));
    }

    // Validate required fields
    if request.subject.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(EmailResponse {
                success: false,
                message: "Subject cannot be empty".to_string(),
                email_sent_to: None,
            }),
        ));
    }

    if request.body.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(EmailResponse {
                success: false,
                message: "Body cannot be empty".to_string(),
                email_sent_to: None,
            }),
        ));
    }

    // Send the email
    match email::send_email(
        &config,
        &request.to,
        &request.subject,
        &request.body,
        request.html.as_deref(),
    )
    .await
    {
        Ok(_) => Ok(Json(EmailResponse {
            success: true,
            message: "Email sent successfully".to_string(),
            email_sent_to: Some(request.to),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(EmailResponse {
                success: false,
                message: format!("Failed to send email: {}", e),
                email_sent_to: None,
            }),
        )),
    }
}

pub async fn send_bulk_email_handler(
    State(config): State<Arc<Config>>,
    Json(request): Json<BulkEmailRequest>,
) -> Result<Json<BulkEmailResponse>, (StatusCode, Json<BulkEmailResponse>)> {
    // Validate required fields
    if request.subject.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(BulkEmailResponse {
                total_emails: 0,
                successful: 0,
                failed: 0,
                results: vec![],
            }),
        ));
    }

    if request.body.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(BulkEmailResponse {
                total_emails: 0,
                successful: 0,
                failed: 0,
                results: vec![],
            }),
        ));
    }

    if request.recipients.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(BulkEmailResponse {
                total_emails: 0,
                successful: 0,
                failed: 0,
                results: vec![],
            }),
        ));
    }

    // Validate all email addresses first
    let mut invalid_emails = Vec::new();
    for email in &request.recipients {
        if !is_valid_email(email) {
            invalid_emails.push(email.clone());
        }
    }

    if !invalid_emails.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(BulkEmailResponse {
                total_emails: request.recipients.len(),
                successful: 0,
                failed: invalid_emails.len(),
                results: invalid_emails
                    .into_iter()
                    .map(|email| EmailResult {
                        email,
                        success: false,
                        message: "Invalid email address format".to_string(),
                    })
                    .collect(),
            }),
        ));
    }

    // Send emails concurrently
    let results = email::send_bulk_emails(
        &config,
        &request.recipients,
        &request.subject,
        &request.body,
        request.html.as_deref(),
    )
    .await;

    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    let status_code = if failed == 0 {
        StatusCode::OK
    } else if successful == 0 {
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::PARTIAL_CONTENT
    };

    let response = BulkEmailResponse {
        total_emails: request.recipients.len(),
        successful,
        failed,
        results,
    };

    if status_code == StatusCode::INTERNAL_SERVER_ERROR {
        Err((status_code, Json(response)))
    } else {
        Ok(Json(response))
    }
}

fn is_valid_email(email: &str) -> bool {
    // Basic email validation - you might want to use a more sophisticated library
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    email_regex.is_match(email)
} 