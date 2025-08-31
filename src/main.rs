// src/main.rs

mod auth;
mod bootstrap;
mod cors;
mod rate_limiting;
mod response;
mod router;

use crate::{
    auth::auth_middleware,
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

    let rate_limiter_state = RateLimiterState::new();

    // Create the main application router.
    let app_router = router::create_router();

    // Build the final app by applying middleware layers.
    // The auth middleware is applied directly to the router, protecting all its routes.
    // The request processing order is: cors -> rate_limit -> auth -> router handler.
    let app = app_router
        .layer(middleware::from_fn(auth_middleware))
        .layer(RateLimitLayer::new(rate_limiter_state))
        .layer(middleware::from_fn(cors_middleware));

    log(LogLevel::Info, "Starting server...");

    if let Err(e) = bootstrap::run(app).await {
        let error_message = format!("Server failed to start: {}", e);
        log(LogLevel::Error, &error_message);
    }
}
