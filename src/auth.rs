// src/auth.rs

use crate::{response, AppState};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::env;

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    // Check if we're in dev mode
    if let Ok(mode) = env::var("MODE") {
        if mode == "dev" {
            // Skip authentication in dev mode
            return next.run(req).await;
        }
    }

    // Get the API_TOKEN from the shared application state.
    let api_token = {
        let token_guard = state.api_token.read().unwrap();
        token_guard.clone()
    };

    // Extract the 'Authorization' header from the request.
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(header_value) => {
            if let Some(token) = header_value.strip_prefix("Bearer ") {
                if token == api_token {
                    // Token is valid, proceed with the request.
                    next.run(req).await
                } else {
                    // Token is invalid.
                    response::error(StatusCode::UNAUTHORIZED, "Invalid authentication token.")
                }
            } else {
                // Header format is incorrect.
                response::error(
                    StatusCode::UNAUTHORIZED,
                    "Invalid authorization header format.",
                )
            }
        }
        None => {
            // 'Authorization' header is missing.
            response::error(StatusCode::UNAUTHORIZED, "Authorization header is missing.")
        }
    }
}