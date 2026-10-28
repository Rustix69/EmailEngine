use crate::{models::SentResponse, sent_log};
use axum::response::Json;

/// Every address in the sent log, so the frontend can show what is already done (read-only, sends nothing)
pub async fn sent_handler() -> Json<SentResponse> {
    let mut addresses: Vec<String> = sent_log::load_sent().into_iter().collect();
    addresses.sort();
    Json(SentResponse { addresses })
}
