use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// AntSol manifest file structure (antsol.toml)
#[derive(Debug, Serialize, Deserialize)]
pub struct AntSolManifest {
    pub package: PackageInfo,
    pub dependencies: Option<Vec<Dependency>>,
    pub external_dependencies: Option<Vec<ExternalDependency>>,
}

/// Package metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
}

/// Package dependency specification (other AntSol packages)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

/// External dependency specification (Rust crates, npm packages, etc.)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalDependency {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub dep_type: String, // "rust", "npm", "python", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>, // Optional: "crates.io", "npmjs.com", etc.
}

/// On-chain package account data
#[derive(Debug)]
pub struct PackageAccount {
    pub name: String,
    pub version: String,
    pub authority: Pubkey,
    pub ipfs_cid: String,
    pub published_at: i64,
    pub description: String,
    pub dependencies: Vec<Dependency>,
    pub external_dependencies: Vec<ExternalDependency>,
}

/// Result type for error handling
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
