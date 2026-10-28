use crate::{config::Config, mailer, models::{EmailRequest, EmailResponse}, validation::is_valid_email};
use axum::{extract::State, http::StatusCode, response::Json};
use std::sync::Arc;

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
    match mailer::send_email(
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
