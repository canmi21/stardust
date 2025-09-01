// src/public.rs

use fancy_log::{log, LogLevel};
use rust_embed::RustEmbed;
use std::{fs, path::Path};

#[derive(RustEmbed)]
#[folder = "public/"]
pub struct PublicAssets;

pub fn setup_public_directory() -> Result<(), Box<dyn std::error::Error>> {
    let public_dir = Path::new("/opt/stardust/public");
    log(LogLevel::Debug, "Setting up public directory...");

    if !public_dir.exists() {
        fs::create_dir_all(public_dir)?;
        log(LogLevel::Info, "Created directory: /opt/stardust/public");
    }

    for file_path in PublicAssets::iter() {
        let target_path = public_dir.join(file_path.as_ref());
        let should_extract = if target_path.exists() {
            false
        } else {
            true
        };

        if should_extract {
            if let Some(file_data) = PublicAssets::get(&file_path) {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::write(&target_path, file_data.data.as_ref())?;
                log(
                    LogLevel::Debug,
                    &format!("Extracted: {}", target_path.display()),
                );
            }
        }
    }

    let index_path = public_dir.join("index.html");
    if !index_path.exists() {
        let default_index = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Stardust API</title>
    <!-- Favicon declarations -->
    <link rel="icon" type="image/x-icon" href="/favicon.ico">
    <link rel="icon" type="image/svg+xml" href="/favicon.svg">
    <link rel="icon" type="image/png" sizes="96x96" href="/favicon-96x96.png">
    <link rel="apple-touch-icon" href="/apple-touch-icon.png">
    <link rel="manifest" href="/site.webmanifest">
</head>
<body>
    <h1>Stardust API Server</h1>
    <p>API server is running. Use the API endpoints to interact with the service.</p>
</body>
</html>"#;
        fs::write(&index_path, default_index)?;
        log(LogLevel::Info, "Created default index.html");
    }

    log(LogLevel::Info, "Public directory setup completed.");
    Ok(())
}