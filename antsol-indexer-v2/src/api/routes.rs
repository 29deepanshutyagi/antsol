use axum::{routing::{get, post}, Router};
use deadpool_postgres::Pool;

use super::handlers::*;

pub fn create_router(pool: Pool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/search", get(search_packages_handler))
        .route("/api/packages/:name", get(get_package_handler))
        .route("/api/packages", get(list_packages_handler))
        .route("/api/stats", get(get_stats_handler))
        .route("/api/events/recent", get(get_recent_events_handler))
        .route("/api/events/:package", get(get_package_events_handler))
    .route("/api/ingest", post(ingest_log_handler))
        .with_state(pool)
}
