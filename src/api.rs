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

fn is_valid_email(email: &str) -> bool {
    // Basic email validation - you might want to use a more sophisticated library
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    email_regex.is_match(email)
} 