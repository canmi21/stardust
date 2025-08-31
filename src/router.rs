// src/router.rs

use crate::response;
use axum::{response::Response, routing::get, Router};
use serde_json::json;

async fn root_get() -> Response {
    response::success(Some(json!({ "message": "Hello, World" })))
}

pub fn create_router() -> Router {
    Router::new().route("/", get(root_get))
}