use crate::{
    models::{ApiError, ValidateBatchRequest, ValidateBatchResponse, ValidateQuery, ValidateResponse},
    validation::MAX_VALIDATE_BATCH,
    verifier,
};
use axum::{extract::Query, http::StatusCode, response::Json};

type ErrorResponse = (StatusCode, Json<ApiError>);

fn error(status: StatusCode, message: impl Into<String>) -> ErrorResponse {
    (status, Json(ApiError { success: false, message: message.into() }))
}

pub async fn validate_email_handler(
    Query(query): Query<ValidateQuery>,
) -> Result<Json<ValidateResponse>, ErrorResponse> {
    let email = query.email.trim();
    if email.is_empty() {
        return Err(error(StatusCode::BAD_REQUEST, "email cannot be empty"));
    }

    verifier::verify(email)
        .await
        .map(Json)
        .map_err(|e| error(StatusCode::BAD_GATEWAY, format!("{:#}", e)))
}

pub async fn validate_emails_handler(
    Json(request): Json<ValidateBatchRequest>,
) -> Result<Json<ValidateBatchResponse>, ErrorResponse> {
    let emails: Vec<String> = request.emails.iter().map(|e| e.trim().to_string()).collect();
    if emails.is_empty() || emails.len() > MAX_VALIDATE_BATCH {
        return Err(error(
            StatusCode::BAD_REQUEST,
            format!("emails must contain 1 to {} addresses", MAX_VALIDATE_BATCH),
        ));
    }
    if emails.iter().any(|e| e.is_empty()) {
        return Err(error(StatusCode::BAD_REQUEST, "emails cannot contain blank entries"));
    }

    verifier::verify_batch(&emails)
        .await
        .map(|results| Json(ValidateBatchResponse { results }))
        .map_err(|e| error(StatusCode::BAD_GATEWAY, format!("{:#}", e)))
}
