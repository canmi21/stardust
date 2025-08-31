// src/main.rs

mod auth;
mod bootstrap;
mod cors;
mod rate_limiting;
mod response;
mod router;
mod sqlite;

use crate::{
    auth::auth_middleware,
    cors::cors_middleware,
    rate_limiting::{RateLimitLayer, RateLimiterState},
    sqlite::initialize_databases,
};
use axum::middleware;
use base64::{engine::general_purpose, Engine as _};
use dotenvy::dotenv;
use fancy_log::{log, set_log_level, LogLevel};
use rand::RngCore;
use std::{fs, io::Write, path::Path, sync::{Arc, RwLock}};

// The application's shared state, now holding the API token.
#[derive(Clone)]
pub struct AppState {
    api_token: Arc<RwLock<String>>,
}

/// Sets up the required directory structure and loads or creates the API token.
///
/// This function ensures that `/opt/stardust/data` and `/opt/stardust/etc` exist.
/// It then loads the token from `/opt/stardust/etc/passwd`. If the file doesn't exist,
/// it generates a new 64-byte, base64-encoded token and saves it.
///
/// # Panics
/// The function will panic if it encounters any I/O errors during setup,
/// as the application cannot run without a valid configuration.
fn setup_and_load_token() -> String {
    let root_dir = Path::new("/opt/stardust");
    let data_dir = root_dir.join("data");
    let etc_dir = root_dir.join("etc");
    let passwd_file = etc_dir.join("passwd");

    log(LogLevel::Debug, "Checking /opt/stardust file structure...");

    // Create directories only if they don't exist and log the creation.
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).expect("Failed to create /opt/stardust/data directory");
        log(LogLevel::Info, "Created directory: /opt/stardust/data");
    }
    if !etc_dir.exists() {
        fs::create_dir_all(&etc_dir).expect("Failed to create /opt/stardust/etc directory");
        log(LogLevel::Info, "Created directory: /opt/stardust/etc");
    }

    log(LogLevel::Debug, "File structure check is OK.");

    // Check for the passwd file and load or generate the token.
    if passwd_file.exists() {
        log(
            LogLevel::Debug,
            "Loading token from existing passwd file...",
        );
        let token = fs::read_to_string(&passwd_file)
            .expect("Failed to read token from passwd file")
            .trim()
            .to_string();
        if token.is_empty() {
            panic!("The passwd file is empty. Please delete it to generate a new token.");
        }
        log(LogLevel::Debug, "Token loaded successfully.");
        token
    } else {
        log(
            LogLevel::Warn,
            "Passwd file not found. Generating a new API token...",
        );
        let mut key = [0u8; 64];
        rand::rng().fill_bytes(&mut key);
        let token = general_purpose::STANDARD.encode(&key);

        let mut file = fs::File::create(&passwd_file).expect("Failed to create passwd file");
        file.write_all(token.as_bytes())
            .expect("Failed to write token to passwd file");
        log(
            LogLevel::Info,
            "New token successfully generated and saved to /opt/stardust/etc/passwd.",
        );
        token
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    set_log_level(LogLevel::Debug);

    // Setup file structure and load the token at startup.
    let api_token = setup_and_load_token();

    // Initialize databases
    if let Err(e) = initialize_databases().await {
        log(LogLevel::Error, &format!("Failed to initialize databases: {}", e));
        return;
    }

    // Create shared state.
    let app_state = AppState {
        api_token: Arc::new(RwLock::new(api_token)),
    };
    let rate_limiter_state = RateLimiterState::new();

    // Build the final app by applying middleware layers and providing state.
    let app = router::create_router()
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        .layer(RateLimitLayer::new(rate_limiter_state))
        .layer(middleware::from_fn(cors_middleware))
        .with_state(app_state);

    log(LogLevel::Info, "Starting server...");

    if let Err(e) = bootstrap::run(app).await {
        let error_message = format!("Server failed to start: {}", e);
        log(LogLevel::Error, &error_message);
    }
}