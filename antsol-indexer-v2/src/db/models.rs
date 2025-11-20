use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: i32,
    pub name: String,
    pub author: String,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub total_downloads: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: i32,
    pub package_id: i32,
    pub version: String,
    pub ipfs_hash: String,
    pub downloads: i64,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageWithVersions {
    #[serde(flatten)]
    pub package: Package,
    pub versions: Vec<Version>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub event_type: String,
    pub package_name: String,
    pub version: Option<String>,
    pub transaction_signature: String,
    pub slot: i64,
    pub block_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_packages: i64,
    pub total_versions: i64,
    pub total_downloads: i64,
    pub total_events: i64,
}
