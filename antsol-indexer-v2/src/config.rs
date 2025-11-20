use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub solana_rpc_url: String,
    pub antsol_program_id: String,
    pub host: String,
    pub port: u16,
    pub start_slot: Option<u64>,
    pub poll_interval_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenv::dotenv().ok();

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL must be set")?,
            solana_rpc_url: env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
            antsol_program_id: env::var("ANTSOL_PROGRAM_ID")
                .map_err(|_| "ANTSOL_PROGRAM_ID must be set")?,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            start_slot: env::var("INDEXER_START_SLOT").ok().and_then(|s| s.parse().ok()),
            poll_interval_secs: env::var("INDEXER_POLL_INTERVAL_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(2),
        })
    }
}
