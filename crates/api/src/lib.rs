//! Axum application factory

use axum::Router;
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub mod error;
pub mod routes;

pub fn app(pool: SqlitePool) -> Router {
    Router::new()
        .merge(routes::accounts::router())
        .merge(routes::sources::router())
        .merge(routes::transactions::router())
        .merge(routes::payees::router())
        .merge(routes::budgets::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(pool)
}
