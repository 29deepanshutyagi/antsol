use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::types::Result;

/// Global configuration for AntSol CLI
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub wallet_path: Option<PathBuf>,
    pub rpc_url: String,
    pub ipfs_url: String,
    pub program_id: String,
    pub pinata_jwt: Option<String>,
    #[serde(default = "Config::default_indexer_url")]
    pub indexer_url: String,
}

impl Config {
    pub fn default_indexer_url() -> String {
        "https://antsol-indexer-v2.onrender.com".to_string()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wallet_path: None,
            rpc_url: "https://api.devnet.solana.com".to_string(),
            ipfs_url: "https://api.pinata.cloud".to_string(),
            program_id: "A9igkBugcujD9Nw9d97FFN4aY3qHXnJxEqCChJt8C42S".to_string(),
            pinata_jwt: None,
            indexer_url: Self::default_indexer_url(),
        }
    }
}

impl Config {
    /// Load configuration from ~/.antsol/config.toml
    pub fn load() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        let config_file = config_dir.join("config.toml");
        
        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
    
    /// Save configuration to ~/.antsol/config.toml
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        std::fs::create_dir_all(&config_dir)?;
        
        let config_file = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_file, content)?;
        
        Ok(())
    }
    
    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        Ok(home.join(".antsol"))
    }
}
