// src/auth.rs

use crate::{response, AppState};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    // Get the API_TOKEN from the shared application state.
    let api_token = &state.api_token;

    // Extract the 'Authorization' header from the request.
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(header_value) => {
            if let Some(token) = header_value.strip_prefix("Bearer ") {
                if token == api_token.as_ref() {
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

