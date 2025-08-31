// src/router.rs

use crate::{response, AppState};
use axum::{
    extract::State,
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Router,
};
use fancy_log::{log, LogLevel};
use serde_json::json;
use std::{fs, path::Path};

/// Creates the main application router.
/// All routes defined here will be protected by the auth middleware applied in main.rs.
pub fn create_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(root_get))
        .route("/v1/token/reload", post(token_reload))
}

// This handler is now protected and will only be reached if the token is valid.
async fn root_get() -> Response {
    response::success(Some(json!({ "service": "PONG" })))
}

/// Handles token reload requests.
/// This endpoint allows clients to request the server to reload its API token from the passwd file.
async fn token_reload(State(state): State<AppState>) -> Response {
    let passwd_file = Path::new("/opt/stardust/etc/passwd");

    // Check if passwd file exists
    if !passwd_file.exists() {
        log(LogLevel::Error, "Passwd file not found during token reload request.");
        return response::error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Token file not found.",
        );
    }

    // Read the current token from file
    let file_token = match fs::read_to_string(&passwd_file) {
        Ok(content) => content.trim().to_string(),
        Err(e) => {
            log(LogLevel::Error, &format!("Failed to read passwd file: {}", e));
            return response::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read token file.",
            );
        }
    };

    if file_token.is_empty() {
        log(LogLevel::Error, "Passwd file is empty during token reload.");
        return response::error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Token file is empty.",
        );
    }

    // Compare with current token in memory
    let current_token = {
        let token_guard = state.api_token.read().unwrap();
        token_guard.clone()
    };

    if file_token == current_token {
        // Token hasn't changed
        log(LogLevel::Debug, "Token reload requested, but token hasn't changed.");
        response::success(Some(json!({
            "message": "Token unchanged",
            "reloaded": false
        })))
    } else {
        // Token has changed, update the application state
        {
            let mut token_guard = state.api_token.write().unwrap();
            *token_guard = file_token;
        }

        log(LogLevel::Info, "Token reload requested and token has been updated successfully.");

        response::success(Some(json!({
            "message": "Token reloaded successfully",
            "reloaded": true
        })))
    }
}