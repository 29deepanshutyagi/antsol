mod config;
mod db;
mod api;
mod indexer;

use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    tracing::info!("Starting AntSol Indexer v2");

    // Load configuration
    let config = config::Config::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // Create database pool
    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created");

    // Run migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Database migrations completed");

    // Start blockchain indexer in background
    let indexer_pool = pool.clone();
    let rpc_url = config.solana_rpc_url.clone();
    let program_id = config.antsol_program_id.clone();
    
    let start_slot_override = config.start_slot;
    let poll_interval = config.poll_interval_secs;
    tokio::spawn(async move {
        indexer::start_indexer(indexer_pool, rpc_url, program_id, start_slot_override, poll_interval).await;
    });
    tracing::info!("Blockchain indexer started");

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create router with CORS
    let app = api::routes::create_router(pool)
        .layer(cors)
        .layer(tower_http::compression::CompressionLayer::new());

    // Create server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting HTTP server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
