// src/auth.rs

use crate::response;
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::env;

pub async fn auth_middleware(req: Request, next: Next) -> Response {
    // Attempt to get the API_TOKEN from environment variables.
    let api_token = match env::var("API_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            // This is a server configuration error, so we return a 500.
            return response::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server configuration error: API_TOKEN not set.",
            );
        }
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