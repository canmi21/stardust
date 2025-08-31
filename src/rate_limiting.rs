// src/rate_limiting.rs

use crate::response;
use axum::{
    body::{Body, Bytes},
    extract::ConnectInfo,
    http::{Request, Response as AxumResponse, StatusCode},
};
use futures::future::BoxFuture;
use http_body::Body as HttpBody;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct RateLimiterState {
    store: Arc<Mutex<HashMap<IpAddr, (Instant, u8)>>>,
}

impl RateLimiterState {
    pub fn new() -> Self {
        let store: Arc<Mutex<HashMap<IpAddr, (Instant, u8)>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let store_clone = store.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let mut guard = store_clone.lock().unwrap();
                let now = Instant::now();
                guard.retain(|_, (timestamp, _)| now.duration_since(*timestamp).as_secs() < 300);
            }
        });
        Self { store }
    }
}

#[derive(Clone)]
pub struct RateLimitLayer {
    state: RateLimiterState,
}

impl RateLimitLayer {
    pub fn new(state: RateLimiterState) -> Self {
        Self { state }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    state: RateLimiterState,
}

impl<S, B> Service<Request<B>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = AxumResponse<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        let ip = extract_client_ip(&request).unwrap_or_else(|| "127.0.0.1".parse().unwrap());
        let mut store = self.state.store.lock().unwrap();
        let now = Instant::now();
        let limit: u8 = 2;
        let window = Duration::from_secs(1);
        let record = store.entry(ip).or_insert_with(|| (now, 0));

        if now.duration_since(record.0) > window {
            record.0 = now;
            record.1 = 1;
        } else {
            record.1 += 1;
        }

        if record.1 > limit {
            // Use the standard error response format.
            let resp = response::error(StatusCode::TOO_MANY_REQUESTS, "Too many requests");
            return Box::pin(async { Ok(resp) });
        }

        drop(store);
        let request = request.map(Body::new);
        let future = self.inner.call(request);
        Box::pin(async move {
            let response = future.await?;
            Ok(response)
        })
    }
}

fn extract_client_ip<B>(req: &Request<B>) -> Option<IpAddr> {
    let headers = req.headers();
    if let Some(header_value) = headers.get("x-forwarded-for") {
        if let Ok(as_str) = header_value.to_str() {
            if let Some(client_ip_str) = as_str.split(',').next() {
                if let Ok(ip) = client_ip_str.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }
    if let Some(header_value) = headers.get("x-real-ip") {
        if let Ok(as_str) = header_value.to_str() {
            if let Ok(ip) = as_str.trim().parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip())
}
