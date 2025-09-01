// src/sqlite.rs

use fancy_log::{log, LogLevel};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::{fs, path::Path};

pub async fn initialize_databases() -> anyhow::Result<()> {
    let root_dir = Path::new("/opt/stardust");
    let data_dir = root_dir.join("data");

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
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

async fn get_pool(db_path: &Path) -> anyhow::Result<Pool<Sqlite>> {
    if !db_path.exists() {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::File::create(db_path)?;
        log(LogLevel::Info, &format!("Created database file: {}", db_path.display()));
    }

    let uri = format!("sqlite://{}", db_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&uri)
        .await?;
    Ok(pool)
}

/// CREATE TABLE
macro_rules! create_table {
    ($pool:expr, $table:expr, $schema:expr) => {{
        let sql = format!("CREATE TABLE IF NOT EXISTS {} ({})", $table, $schema);
        sqlx::query(&sql).execute($pool).await?;
    }};
}

/// Account
async fn initialize_account_db(data_dir: &Path) -> anyhow::Result<()> {
    let db_path = data_dir.join("account.sqlite");
    let pool = get_pool(&db_path).await?;

    create_table!(
        &pool,
        "users",
        "
            user_id TEXT PRIMARY KEY,
            user_level INTEGER NOT NULL DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_modified DATETIME DEFAULT CURRENT_TIMESTAMP
        "
    );

    sqlx::query(
        "
        CREATE TRIGGER IF NOT EXISTS update_user_timestamp
        AFTER UPDATE ON users
        BEGIN
            UPDATE users SET last_modified = CURRENT_TIMESTAMP WHERE user_id = NEW.user_id;
        END
        ",
    )
    .execute(&pool)
    .await?;

    log(LogLevel::Debug, "Account database initialized.");
    Ok(())
}

/// Email
async fn initialize_email_db(data_dir: &Path) -> anyhow::Result<()> {
    let db_path = data_dir.join("email.sqlite");
    let pool = get_pool(&db_path).await?;

    create_table!(
        &pool,
        "email_users",
        "
            email TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            last_modified DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (user_id)
        "
    );

    sqlx::query(
        "
        CREATE TRIGGER IF NOT EXISTS update_email_timestamp
        AFTER UPDATE ON email_users
        BEGIN
            UPDATE email_users SET last_modified = CURRENT_TIMESTAMP WHERE email = NEW.email;
        END
        ",
    )
    .execute(&pool)
    .await?;

    log(LogLevel::Debug, "Email database initialized.");
    Ok(())
}

/// Handle
async fn initialize_handle_db(data_dir: &Path) -> anyhow::Result<()> {
    let db_path = data_dir.join("handle.sqlite");
    let pool = get_pool(&db_path).await?;

    create_table!(
        &pool,
        "handle_users",
        "
            handle TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            last_modified DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (user_id)
        "
    );

    sqlx::query(
        "
        CREATE TRIGGER IF NOT EXISTS update_handle_timestamp
        AFTER UPDATE ON handle_users
        BEGIN
            UPDATE handle_users SET last_modified = CURRENT_TIMESTAMP WHERE handle = NEW.handle;
        END
        ",
    )
    .execute(&pool)
    .await?;

    log(LogLevel::Debug, "Handle database initialized.");
    Ok(())
}

/// Passwd
async fn initialize_passwd_db(data_dir: &Path) -> anyhow::Result<()> {
    let db_path = data_dir.join("passwd.sqlite");
    let pool = get_pool(&db_path).await?;

    create_table!(
        &pool,
        "passwd_users",
        "
            user_id TEXT PRIMARY KEY,
            password_hash TEXT,
            salt TEXT
        "
    );

    log(LogLevel::Debug, "Passwd database initialized.");
    Ok(())
}

/// TOTP
async fn initialize_totp_db(data_dir: &Path) -> anyhow::Result<()> {
    let db_path = data_dir.join("totp.sqlite");
    let pool = get_pool(&db_path).await?;

    create_table!(
        &pool,
        "totp_users",
        "
            user_id TEXT PRIMARY KEY,
            totp_secret TEXT
        "
    );

    log(LogLevel::Debug, "TOTP database initialized.");
    Ok(())
}

/// Passkey
async fn initialize_passkey_db(data_dir: &Path) -> anyhow::Result<()> {
    let db_path = data_dir.join("passkey.sqlite");
    let pool = get_pool(&db_path).await?;

    create_table!(
        &pool,
        "passkey_users",
        "
            user_id TEXT PRIMARY KEY,
            public_key TEXT
        "
    );

    log(LogLevel::Debug, "Passkey database initialized.");
    Ok(())
}

/// Recovery
async fn initialize_recovery_db(data_dir: &Path) -> anyhow::Result<()> {
    let db_path = data_dir.join("recovery.sqlite");
    let pool = get_pool(&db_path).await?;

    create_table!(
        &pool,
        "recovery_users",
        "
            user_id TEXT PRIMARY KEY,
            recovery_key TEXT
        "
    );

    log(LogLevel::Debug, "Recovery database initialized.");
    Ok(())
}
