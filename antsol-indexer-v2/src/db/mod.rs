pub mod models;
pub mod queries;

use deadpool_postgres::{Pool, Runtime};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;

pub async fn create_pool(database_url: &str) -> Result<Pool, Box<dyn std::error::Error>> {
    let mut cfg = deadpool_postgres::Config::new();
    cfg.dbname = None;
    cfg.user = None;
    cfg.password = None;
    cfg.host = None;
    cfg.port = None;
        cfg.url = Some(database_url.to_string());
    let tls_connector = TlsConnector::builder().build()?;
    let make_tls = MakeTlsConnector::new(tls_connector);
    let pool = cfg.create_pool(Some(Runtime::Tokio1), make_tls)?;
    Ok(pool)
}

pub async fn run_migrations(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    
    let migration_sql = include_str!("../../migrations/001_init.sql");
    
    client.batch_execute(migration_sql).await?;
    
    tracing::info!("Database migrations completed successfully");
    Ok(())
}
