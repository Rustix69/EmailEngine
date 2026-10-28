mod config;
mod handlers;
mod mailer;
mod models;
mod sent_log;
mod validation;

use anyhow::Result;
use dotenvy::dotenv;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let cfg = Arc::new(config::Config::from_env()?);
    let server_port = cfg.server_port;

    let app = Router::new()
        .route("/", get(handlers::health_check))
        .route("/send-email", post(handlers::send_email_handler))
        .route("/send-bulk-email", post(handlers::send_bulk_email_handler))
        .route("/send-batch", post(handlers::send_batch_handler))
        .route("/sent", get(handlers::sent_handler))
        .layer(CorsLayer::permissive())
        .with_state(cfg);

    let addr = format!("localhost:{}", server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("🚀 Email API Server running on http://{}", addr);
    println!("📧 Send POST requests to /send-email for single emails");
    println!("📬 Send POST requests to /send-bulk-email for multiple emails");
    println!("📨 Send POST requests to /send-batch for personalised emails");
    println!("📜 GET /sent lists everyone emailed so far");
    println!("💊 Health check available at /");

    println!("\x1b[32m==================================  LOGS  =========================================\x1b[0m");
    
    axum::serve(listener, app).await?;
    Ok(())
}
