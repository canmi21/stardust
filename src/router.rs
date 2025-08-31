// src/router.rs

use crate::{passwd, response, AppState};
use axum::{
    response::Response,
    routing::{get, post},
    Router,
};
use serde_json::json;

/// Creates the main application router.
/// All routes defined here will be protected by the auth middleware applied in main.rs.
pub fn create_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(root_get))
        .route("/v1/token/reload", post(passwd::token_reload))
}

// This handler is now protected and will only be reached if the token is valid.
async fn root_get() -> Response {
    response::success(Some(json!({ "service": "PONG" })))
}
