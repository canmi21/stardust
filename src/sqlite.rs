// src/sqlite.rs

use fancy_log::{log, LogLevel};
use std::{fs, path::Path};
use tokio_rusqlite::{Connection, Result};

/// Initializes all SQLite databases and creates necessary tables.
/// This function will create the databases and tables if they don't exist.
pub async fn initialize_databases() -> Result<()> {
    let root_dir = Path::new("/opt/stardust");
    let data_dir = root_dir.join("data");

    // Ensure data directory exists
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).expect("Failed to create /opt/stardust/data directory");
        log(LogLevel::Info, "Created directory: /opt/stardust/data");
    }

    // Initialize database
    initialize_account_db(&data_dir).await?;
    initialize_email_db(&data_dir).await?;
    initialize_handle_db(&data_dir).await?;

    log(LogLevel::Info, "All databases initialized successfully.");
    Ok(())
}

/// Initializes the account.sqlite database with user information table.
async fn initialize_account_db(data_dir: &Path) -> Result<()> {
    let db_path = data_dir.join("account.sqlite");
    let conn = Connection::open(&db_path).await?;
    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                user_id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_level INTEGER NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_modified DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create trigger to update last_modified automatically
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

/// Initializes the email.sqlite database with email-to-user mapping.
async fn initialize_email_db(data_dir: &Path) -> Result<()> {
    let db_path = data_dir.join("email.sqlite");
    let conn = Connection::open(&db_path).await?;

    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS email_users (
                email TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (user_id)
            )",
            [],
        )?;
        Ok(())
    }).await?;

    log(LogLevel::Debug, "Email database initialized.");
    Ok(())
}

/// Initializes the handle.sqlite database with handle-to-user mapping.
async fn initialize_handle_db(data_dir: &Path) -> Result<()> {
    let db_path = data_dir.join("handle.sqlite");
    let conn = Connection::open(&db_path).await?;

    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS handle_users (
                handle TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (user_id)
            )",
            [],
        )?;
        Ok(())
    }).await?;

    log(LogLevel::Debug, "Handle database initialized.");
    Ok(())
}

/// Gets a connection to the account database.
pub async fn get_account_connection() -> Result<Connection> {
    let db_path = Path::new("/opt/stardust/data/account.sqlite");
    Connection::open(db_path).await
}

/// Gets a connection to the email database.
pub async fn get_email_connection() -> Result<Connection> {
    let db_path = Path::new("/opt/stardust/data/email.sqlite");
    Connection::open(db_path).await
}

/// Gets a connection to the handle database.
pub async fn get_handle_connection() -> Result<Connection> {
    let db_path = Path::new("/opt/stardust/data/handle.sqlite");
    Connection::open(db_path).await
}