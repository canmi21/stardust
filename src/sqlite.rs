// src/sqlite.rs

use fancy_log::{log, LogLevel};
use std::{fs, path::Path};
use tokio_rusqlite::{Connection, Result};

pub async fn initialize_databases() -> Result<()> {
    let root_dir = Path::new("/opt/stardust");
    let data_dir = root_dir.join("data");

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).expect("Failed to create /opt/stardust/data directory");
        log(LogLevel::Info, "Created directory: /opt/stardust/data");
    }

    initialize_account_db(&data_dir).await?;
    initialize_email_db(&data_dir).await?;
    initialize_handle_db(&data_dir).await?;
    initialize_passwd_db(&data_dir).await?;
    initialize_totp_db(&data_dir).await?;
    initialize_passkey_db(&data_dir).await?;
    initialize_recovery_db(&data_dir).await?;

    log(LogLevel::Info, "All databases initialized successfully.");
    Ok(())
}

/// Users
async fn initialize_account_db(data_dir: &Path) -> Result<()> {
    let db_path = data_dir.join("account.sqlite");
    let conn = Connection::open(&db_path).await?;
    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                user_id TEXT PRIMARY KEY,
                user_level INTEGER NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_modified DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS update_user_timestamp
            AFTER UPDATE ON users
            BEGIN
                UPDATE users SET last_modified = CURRENT_TIMESTAMP WHERE user_id = NEW.user_id;
            END",
            [],
        )?;
        Ok(())
    }).await?;
    log(LogLevel::Debug, "Account database initialized.");
    Ok(())
}

/// Email
async fn initialize_email_db(data_dir: &Path) -> Result<()> {
    let db_path = data_dir.join("email.sqlite");
    let conn = Connection::open(&db_path).await?;
    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS email_users (
                email TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                last_modified DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users (user_id)
            )",
            [],
        )?;
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS update_email_timestamp
            AFTER UPDATE ON email_users
            BEGIN
                UPDATE email_users SET last_modified = CURRENT_TIMESTAMP WHERE email = NEW.email;
            END",
            [],
        )?;
        Ok(())
    }).await?;
    log(LogLevel::Debug, "Email database initialized.");
    Ok(())
}

/// Handle
async fn initialize_handle_db(data_dir: &Path) -> Result<()> {
    let db_path = data_dir.join("handle.sqlite");
    let conn = Connection::open(&db_path).await?;
    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS handle_users (
                handle TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                last_modified DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users (user_id)
            )",
            [],
        )?;
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS update_handle_timestamp
            AFTER UPDATE ON handle_users
            BEGIN
                UPDATE handle_users SET last_modified = CURRENT_TIMESTAMP WHERE handle = NEW.handle;
            END",
            [],
        )?;
        Ok(())
    }).await?;
    log(LogLevel::Debug, "Handle database initialized.");
    Ok(())
}

/// Passwd
async fn initialize_passwd_db(data_dir: &Path) -> Result<()> {
    let db_path = data_dir.join("passwd.sqlite");
    let conn = Connection::open(&db_path).await?;
    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS passwd_users (
                user_id TEXT PRIMARY KEY,
                password_hash TEXT,
                salt TEXT
            )",
            [],
        )?;
        Ok(())
    }).await?;
    log(LogLevel::Debug, "Passwd database initialized.");
    Ok(())
}

/// TOTP
async fn initialize_totp_db(data_dir: &Path) -> Result<()> {
    let db_path = data_dir.join("totp.sqlite");
    let conn = Connection::open(&db_path).await?;
    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS totp_users (
                user_id TEXT PRIMARY KEY,
                totp_secret TEXT
            )",
            [],
        )?;
        Ok(())
    }).await?;
    log(LogLevel::Debug, "TOTP database initialized.");
    Ok(())
}

/// Passkey
async fn initialize_passkey_db(data_dir: &Path) -> Result<()> {
    let db_path = data_dir.join("passkey.sqlite");
    let conn = Connection::open(&db_path).await?;
    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS passkey_users (
                user_id TEXT PRIMARY KEY,
                public_key TEXT
            )",
            [],
        )?;
        Ok(())
    }).await?;
    log(LogLevel::Debug, "Passkey database initialized.");
    Ok(())
}

/// Recovery
async fn initialize_recovery_db(data_dir: &Path) -> Result<()> {
    let db_path = data_dir.join("recovery.sqlite");
    let conn = Connection::open(&db_path).await?;
    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS recovery_users (
                user_id TEXT PRIMARY KEY,
                recovery_key TEXT
            )",
            [],
        )?;
        Ok(())
    }).await?;
    log(LogLevel::Debug, "Recovery database initialized.");
    Ok(())
}
