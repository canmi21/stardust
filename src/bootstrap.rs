// src/bootstrap.rs

use axum::Router;
use fancy_log::{log, LogLevel};
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub async fn run(app: Router) -> Result<(), Box<dyn std::error::Error>>
{
    let addr = SocketAddr::from(([0, 0, 0, 0], 33302));
    let listener = TcpListener::bind(&addr).await?;
    let info_message = "Listening on http://localhost:33302";
    log(LogLevel::Info, info_message);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}