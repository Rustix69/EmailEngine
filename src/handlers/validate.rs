use crate::{
    models::{ApiError, ValidateQuery, ValidateResponse},
    verifier,
};
use axum::{extract::Query, http::StatusCode, response::Json};

pub async fn validate_email_handler(
    Query(query): Query<ValidateQuery>,
) -> Result<Json<ValidateResponse>, (StatusCode, Json<ApiError>)> {
    let email = query.email.trim();
    if email.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError { success: false, message: "email cannot be empty".to_string() }),
        ));
    }

    verifier::verify(email).await.map(Json).map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(ApiError { success: false, message: format!("{:#}", e) }),
        )
    })
}
