// src/router.rs

use crate::response;
use axum::{response::Response, routing::get, Router};
use serde_json::json;

/// Creates the main application router.
/// All routes defined here will be protected by the auth middleware applied in main.rs.
pub fn create_router() -> Router {
    Router::new().route("/", get(root_get))
}

// This handler is now protected and will only be reached if the token is valid.
async fn root_get() -> Response {
    response::success(Some(json!({ "message": "Hello, authenticated World!" })))
}
