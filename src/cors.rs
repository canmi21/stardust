/* src/cors.rs */

use axum::{
    extract::Request,
    http::{header, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use fancy_log::{log, LogLevel};
use std::env;

/// This function intercepts requests to add CORS headers.
pub async fn cors_middleware(req: Request, next: Next) -> Response {
    let origin_header = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    // --- Handle OPTIONS preflight requests ---
    if req.method() == Method::OPTIONS {
        let mut response = (StatusCode::OK, ()).into_response();
        add_cors_headers(response.headers_mut(), origin_header.as_deref());
        return response;
    }

    // --- Handle actual requests ---
    let mut response = next.run(req).await;
    add_cors_headers(response.headers_mut(), origin_header.as_deref());
    response
}

/// Helper function to add CORS headers to any response
fn add_cors_headers(headers: &mut axum::http::HeaderMap, request_origin: Option<&str>) {
    let vite_gateway = match env::var("VITE_GATEWAY") {
        Ok(val) => val,
        Err(_) => {
            log(LogLevel::Error, "VITE_GATEWAY environment variable not set. CORS will not work.");
            return;
        }
    };

    let is_development = vite_gateway.contains("localhost");

    if let Some(origin_str) = request_origin {
        let origin_approved = if is_development {
            // DEV MODE: Approve any request coming from a localhost origin.
            if origin_str.starts_with("http://localhost:") {
                true
            } else {
                false
            }
        } else {
            origin_str == vite_gateway
        };

        if origin_approved {
            if let Ok(value) = HeaderValue::from_str(origin_str) {
                headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
            }
        } else {
            log(LogLevel::Warn, &format!("CORS request from untrusted origin blocked: {}", origin_str));
        }
    }

    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Origin, X-Requested-With, Content-Type, Accept, Authorization"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );
}