// src/main.rs

mod bootstrap;
mod cors;
mod rate_limiting;
mod response;
mod router;

use crate::{
    cors::cors_middleware,
    rate_limiting::{RateLimitLayer, RateLimiterState},
};
use axum::middleware;
use dotenvy::dotenv;
use fancy_log::{log, set_log_level, LogLevel};

#[tokio::main]
async fn main() {
    dotenv().ok();
    set_log_level(LogLevel::Debug);

    // Create state for middleware layers
    let rate_limiter_state = RateLimiterState::new();

    // Create the router
    let app_router = router::create_router();

    // Build the final application by applying middleware layers
    let app = app_router
        .layer(RateLimitLayer::new(rate_limiter_state))
        .layer(middleware::from_fn(cors_middleware));

    log(LogLevel::Info, "Starting server...");

    if let Err(e) = bootstrap::run(app).await {
        let error_message = format!("Server failed to start: {}", e);
        log(LogLevel::Error, &error_message);
    }
}