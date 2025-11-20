use deadpool_postgres::Pool;
use tokio_postgres::Row;
use chrono::DateTime;

use super::models::*;

pub async fn insert_package(
    pool: &Pool,
    name: &str,
    author: &str,
    description: Option<&str>,
    repository: Option<&str>,
    homepage: Option<&str>,
) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let row = client.query_one(
        "INSERT INTO packages (name, author, description, repository, homepage)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (name) DO UPDATE SET
            author = EXCLUDED.author,
            description = EXCLUDED.description,
            repository = EXCLUDED.repository,
            homepage = EXCLUDED.homepage,
            updated_at = NOW()
         RETURNING id",
        &[&name, &author, &description, &repository, &homepage],
    ).await?;
    
    Ok(row.get(0))
}

pub async fn insert_version(
    pool: &Pool,
    package_id: i32,
    version: &str,
    ipfs_hash: &str,
) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let row = client.query_one(
        "INSERT INTO versions (package_id, version, ipfs_hash)
         VALUES ($1, $2, $3)
         ON CONFLICT (package_id, version) DO UPDATE SET
            ipfs_hash = EXCLUDED.ipfs_hash
         RETURNING id",
        &[&package_id, &version, &ipfs_hash],
    ).await?;
    
    Ok(row.get(0))
}

pub async fn insert_event(
    pool: &Pool,
    event_type: &str,
    package_name: &str,
    version: Option<&str>,
    transaction_signature: &str,
    slot: i64,
    block_time: Option<i64>,
) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let block_time_dt = block_time.map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_default());
    
    let row_opt = client.query_opt(
        "INSERT INTO events (event_type, package_name, version, transaction_signature, slot, block_time)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (transaction_signature) DO NOTHING
         RETURNING id",
        &[&event_type, &package_name, &version, &transaction_signature, &slot, &block_time_dt],
    ).await?;
    // If conflict occurred, RETURNING yields no row; treat as existing (id unknown -> 0)
    Ok(row_opt.map(|r| r.get(0)).unwrap_or(0))
}

pub async fn search_packages(
    pool: &Pool,
    query: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Package>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let rows = client.query(
        "SELECT id, name, author, description, repository, homepage, total_downloads, created_at, updated_at
         FROM packages
         WHERE name ILIKE $1 OR description ILIKE $1
         ORDER BY total_downloads DESC, name ASC
         LIMIT $2 OFFSET $3",
        &[&format!("%{}%", query), &limit, &offset],
    ).await?;
    
    Ok(rows.iter().map(row_to_package).collect())
}

pub async fn get_package_with_versions(
    pool: &Pool,
    name: &str,
) -> Result<Option<PackageWithVersions>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let package_row = client.query_opt(
        "SELECT id, name, author, description, repository, homepage, total_downloads, created_at, updated_at
         FROM packages
         WHERE name = $1",
        &[&name],
    ).await?;
    
    let package = match package_row {
        Some(row) => row_to_package(&row),
        None => return Ok(None),
    };
    
    let version_rows = client.query(
        "SELECT id, package_id, version, ipfs_hash, downloads, published_at
         FROM versions
         WHERE package_id = $1
         ORDER BY published_at DESC",
        &[&package.id],
    ).await?;
    
    let versions = version_rows.iter().map(row_to_version).collect();
    
    Ok(Some(PackageWithVersions { package, versions }))
}

pub async fn list_packages(
    pool: &Pool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Package>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let rows = client.query(
        "SELECT id, name, author, description, repository, homepage, total_downloads, created_at, updated_at
         FROM packages
         ORDER BY created_at DESC
         LIMIT $1 OFFSET $2",
        &[&limit, &offset],
    ).await?;
    
    Ok(rows.iter().map(row_to_package).collect())
}

pub async fn get_stats(pool: &Pool) -> Result<Stats, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let row = client.query_one(
        "SELECT 
            (SELECT COUNT(*) FROM packages) as total_packages,
            (SELECT COUNT(*) FROM versions) as total_versions,
            (SELECT COALESCE(SUM(total_downloads), 0) FROM packages) as total_downloads,
            (SELECT COUNT(*) FROM events) as total_events",
        &[],
    ).await?;
    
    Ok(Stats {
        total_packages: row.get(0),
        total_versions: row.get(1),
        total_downloads: row.get(2),
        total_events: row.get(3),
    })
}

pub async fn increment_download(
    pool: &Pool,
    package_id: i32,
    version_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    client.execute(
        "UPDATE packages SET total_downloads = total_downloads + 1 WHERE id = $1",
        &[&package_id],
    ).await?;
    
    client.execute(
        "UPDATE versions SET downloads = downloads + 1 WHERE id = $1",
        &[&version_id],
    ).await?;
    
    Ok(())
}

fn row_to_package(row: &Row) -> Package {
    Package {
        id: row.get(0),
        name: row.get(1),
        author: row.get(2),
        description: row.get(3),
        repository: row.get(4),
        homepage: row.get(5),
        total_downloads: row.get(6),
        created_at: row.get(7),
        updated_at: row.get(8),
    }
}

fn row_to_version(row: &Row) -> Version {
    Version {
        id: row.get(0),
        package_id: row.get(1),
        version: row.get(2),
        ipfs_hash: row.get(3),
        downloads: row.get(4),
        published_at: row.get(5),
    }
}

// Indexer state management
pub async fn get_last_processed_slot(pool: &Pool) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let row = client.query_one(
        "SELECT last_processed_slot FROM indexer_state WHERE id = 1",
        &[],
    ).await?;
    
    let slot: i64 = row.get(0);
    Ok(slot.max(0) as u64)
}

pub async fn update_last_processed_slot(
    pool: &Pool,
    slot: u64,
    block_time: Option<i64>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let block_time_dt = block_time.map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_default());
    
    client.execute(
        "UPDATE indexer_state 
         SET last_processed_slot = $1, 
             last_processed_block_time = $2,
             updated_at = NOW(),
             status = 'running'
         WHERE id = 1",
        &[&(slot as i64), &block_time_dt],
    ).await?;
    
    Ok(())
}

pub async fn update_indexer_error(
    pool: &Pool,
    error_msg: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    client.execute(
        "UPDATE indexer_state 
         SET error_count = error_count + 1,
             last_error = $1,
             updated_at = NOW()
         WHERE id = 1",
        &[&error_msg],
    ).await?;
    
    Ok(())
}

pub async fn get_recent_events(
    pool: &Pool,
    limit: i64,
) -> Result<Vec<Event>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let rows = client.query(
        "SELECT id, event_type, package_name, version, transaction_signature, slot, block_time
         FROM events
         ORDER BY slot DESC, id DESC
         LIMIT $1",
        &[&limit],
    ).await?;
    
    Ok(rows.iter().map(|row| Event {
        id: row.get(0),
        event_type: row.get(1),
        package_name: row.get(2),
        version: row.get(3),
        transaction_signature: row.get(4),
        slot: row.get(5),
        block_time: row.get(6),
    }).collect())
}

pub async fn get_package_events(
    pool: &Pool,
    package_name: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Event>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let rows = client.query(
        "SELECT id, event_type, package_name, version, transaction_signature, slot, block_time
         FROM events
         WHERE package_name = $1
         ORDER BY slot DESC, id DESC
         LIMIT $2 OFFSET $3",
        &[&package_name, &limit, &offset],
    ).await?;
    
    Ok(rows.iter().map(|row| Event {
        id: row.get(0),
        event_type: row.get(1),
        package_name: row.get(2),
        version: row.get(3),
        transaction_signature: row.get(4),
        slot: row.get(5),
        block_time: row.get(6),
    }).collect())
}

// --- New helper query functions for indexer ingestion logic ---

/// Return the package id if a package with the given name exists.
pub async fn get_package_id(
    pool: &Pool,
    name: &str,
) -> Result<Option<i32>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    let row = client.query_opt(
        "SELECT id FROM packages WHERE name = $1",
        &[&name],
    ).await?;
    Ok(row.map(|r| r.get(0)))
}

/// Return the version id for a given (package_id, version) pair.
pub async fn get_version_id(
    pool: &Pool,
    package_id: i32,
    version: &str,
) -> Result<Option<i32>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    let row = client.query_opt(
        "SELECT id FROM versions WHERE package_id = $1 AND version = $2",
        &[&package_id, &version],
    ).await?;
    Ok(row.map(|r| r.get(0)))
}
