//! Sync server entrypoint.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    tracing::info!("Sync server not yet implemented");
    Ok(())
}
