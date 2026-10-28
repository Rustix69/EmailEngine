use crate::{config::Config, mailer, models::{BulkEmailRequest, BulkEmailResponse, EmailResult}, validation::is_valid_email};
use axum::{extract::State, http::StatusCode, response::Json};
use std::sync::Arc;

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
    let results = mailer::send_bulk_emails(
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
