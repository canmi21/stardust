// src/router.rs

use crate::{auth::auth_middleware, passwd, AppState};
use axum::{
    middleware,
    routing::post,
    Router,
};
use tower_http::services::ServeDir;

/// Creates the main application router.
/// API routes are protected by auth middleware.
/// Static files are served without authentication, including index.html at "/".
pub fn create_router() -> Router<AppState> {
    // Create API routes with auth middleware
    let api_routes = Router::<AppState>::new()
        .route("/v1/token/reload", post(passwd::token_reload))
        .layer(middleware::from_fn_with_state(
            // We need a dummy state here, will be replaced when with_state is called
            AppState {
                api_token: std::sync::Arc::new(std::sync::RwLock::new(String::new())),
            },
            auth_middleware,
        ));

    // Combine API routes with static file service (including index.html at "/")
    Router::<AppState>::new()
        .merge(api_routes)
        .fallback_service(
            ServeDir::new("/opt/stardust/public")
                .append_index_html_on_directories(true)
                .precompressed_gzip()
                .precompressed_br()
                .precompressed_deflate(),
        )
}