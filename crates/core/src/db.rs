//! DB connection and migration management

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use anyhow::Result;

/// connect creates a connection pool
///
/// # Params
/// db_url: a SQLite path to a file or to memory
pub async fn connect(db_url: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    Ok(pool)
}

/// migrate runs all pending SQLx migrations from ./migrations
///
/// # Params
/// - pool: a SQlite connection pool
pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await?;
    Ok(())
}


