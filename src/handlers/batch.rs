use crate::{config::Config, mailer, models::{BatchEmailRequest, BulkEmailResponse}};
use axum::{extract::State, http::StatusCode, response::Json};
use std::sync::Arc;

/// Most messages one /send-batch call accepts
const MAX_BATCH: usize = 50;

pub async fn send_batch_handler(
    State(config): State<Arc<Config>>,
    Json(request): Json<BatchEmailRequest>,
) -> Result<Json<BulkEmailResponse>, (StatusCode, Json<BulkEmailResponse>)> {
    if request.messages.is_empty() || request.messages.len() > MAX_BATCH {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(BulkEmailResponse {
                total_emails: request.messages.len(),
                successful: 0,
                failed: 0,
                results: vec![],
            }),
        ));
    }

    let results = mailer::send_batch(&config, &request.messages).await;
    let successful = results.iter().filter(|r| r.success).count();

    Ok(Json(BulkEmailResponse {
        total_emails: results.len(),
        successful,
        failed: results.len() - successful,
        results,
    }))
}
