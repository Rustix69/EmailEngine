mod config;
mod handlers;
mod mailer;
mod models;
mod scheduler;
mod sent_log;
mod validation;
mod verifier;

use anyhow::Result;
use dotenvy::dotenv;
use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;

/// Shared server state; handlers extract just the part they need
#[derive(Clone, FromRef)]
struct AppState {
    config: Arc<config::Config>,
    scheduler: scheduler::Scheduler,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let cfg = Arc::new(config::Config::from_env()?);
    let server_port = cfg.server_port;

    let scheduler = scheduler::Scheduler::load();
    scheduler::spawn(scheduler.clone(), Arc::clone(&cfg));
    let state = AppState { config: cfg, scheduler };

    let app = Router::new()
        .route("/", get(handlers::health_check))
        .route("/send-email", post(handlers::send_email_handler))
        .route("/send-bulk-email", post(handlers::send_bulk_email_handler))
        .route("/send-batch", post(handlers::send_batch_handler))
        .route("/sent", get(handlers::sent_handler))
        .route("/validate-email", get(handlers::validate_email_handler))
        .route("/validate-emails", post(handlers::validate_emails_handler))
        .route("/schedule-batch", post(handlers::schedule_batch_handler))
        .route("/scheduled", get(handlers::list_scheduled_handler))
        .route("/scheduled/:id", get(handlers::get_scheduled_handler).delete(handlers::cancel_scheduled_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("localhost:{}", server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("🚀 Email API Server running on http://{}", addr);
    println!("📧 Send POST requests to /send-email for single emails");
    println!("📬 Send POST requests to /send-bulk-email for multiple emails");
    println!("📨 Send POST requests to /send-batch for personalised emails");
    println!("⏰ POST /schedule-batch sends a batch at a set time; see GET /scheduled");
    println!("🔎 GET /validate-email?email=... checks whether an address is deliverable");
    println!("🔎 POST /validate-emails checks up to 100 addresses at once");
    println!("📜 GET /sent lists everyone emailed so far");
    println!("💊 Health check available at /");

    println!("\x1b[32m==================================  LOGS  =========================================\x1b[0m");
    
    axum::serve(listener, app).await?;
    Ok(())
}
