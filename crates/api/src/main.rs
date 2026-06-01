//! API server entrypoint. Reads config, connects to DB, and starts Axum.

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Only necessary in dev
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "api=debug,core=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data.db".to_string()); 
    tracing::info!("Connecting to DB: {}...", db_url);   
    let pool = smok_core::db::connect(&db_url).await?;
    smok_core::db::migrate(&pool).await?;
    tracing::info!("Migrations applied");

    let app = api::app(pool);
    let bind_addr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Now listening on {}", bind_addr);

    axum::serve(listener, app).await?;
    Ok(())
}
