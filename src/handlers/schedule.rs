use crate::{
    models::{ApiError, ScheduleBatchRequest},
    scheduler::{CancelError, Job, JobSummary, Scheduler},
    validation::{is_valid_email, MAX_BATCH},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;

type ErrorResponse = (StatusCode, Json<ApiError>);

fn error(status: StatusCode, message: impl Into<String>) -> ErrorResponse {
    (status, Json(ApiError { success: false, message: message.into() }))
}

pub async fn schedule_batch_handler(
    State(scheduler): State<Scheduler>,
    Json(request): Json<ScheduleBatchRequest>,
) -> Result<(StatusCode, Json<JobSummary>), ErrorResponse> {
    if request.send_at <= Utc::now() {
        return Err(error(StatusCode::BAD_REQUEST, "send_at must be in the future"));
    }
    if request.messages.is_empty() || request.messages.len() > MAX_BATCH {
        return Err(error(StatusCode::BAD_REQUEST, format!("messages must contain 1 to {} entries", MAX_BATCH)));
    }
    // Fail now rather than at send time, when nobody is watching
    for m in &request.messages {
        if !is_valid_email(&m.to) {
            return Err(error(StatusCode::BAD_REQUEST, format!("Invalid email address format: {}", m.to)));
        }
        if m.subject.trim().is_empty() || m.body.trim().is_empty() {
            return Err(error(StatusCode::BAD_REQUEST, format!("Subject and body cannot be empty ({})", m.to)));
        }
    }

    let job = scheduler.add(request.send_at, request.messages);
    Ok((StatusCode::CREATED, Json(job.summary())))
}

pub async fn list_scheduled_handler(State(scheduler): State<Scheduler>) -> Json<Vec<JobSummary>> {
    Json(scheduler.list().iter().map(Job::summary).collect())
}

pub async fn get_scheduled_handler(
    State(scheduler): State<Scheduler>,
    Path(id): Path<String>,
) -> Result<Json<Job>, ErrorResponse> {
    scheduler
        .get(&id)
        .map(Json)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "Scheduled job not found"))
}

pub async fn cancel_scheduled_handler(
    State(scheduler): State<Scheduler>,
    Path(id): Path<String>,
) -> Result<Json<JobSummary>, ErrorResponse> {
    match scheduler.cancel(&id) {
        Ok(job) => Ok(Json(job.summary())),
        Err(CancelError::NotFound) => Err(error(StatusCode::NOT_FOUND, "Scheduled job not found")),
        Err(CancelError::NotPending(status)) => Err(error(
            StatusCode::CONFLICT,
            format!("Only pending jobs can be cancelled, this one is {:?}", status).to_lowercase(),
        )),
    }
}
