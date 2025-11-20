use deadpool_postgres::Pool;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

use super::parser::parse_transaction;
/// Attempt to extract a probable IPFS hash / CID from a log line.
/// Heuristics:
///  - Look for "ipfs" followed by common separators and take next token
///  - Look for "cid" key styles
///  - Fallback: first token starting with "Qm" and length ~46 (CIDv0)
// Make IPFS hash extraction public for reuse in API ingestion endpoint
pub fn extract_ipfs_hash(log: &str) -> Option<String> {
    let lower = log.to_lowercase();
    // Patterns like ipfs=xxx ipfs: xxx ipfs_hash=xxx cid=xxx
    for key in ["ipfs_hash", "ipfs", "cid"] {
        if let Some(pos) = lower.find(&format!("{}=", key)) {
            let after = &log[pos + key.len() + 1..];
            let end = after.find(|c: char| c.is_whitespace() || c == ',' || c == '"' || c == '\n').unwrap_or(after.len());
            let candidate = after[..end].trim().trim_matches(['"', '\'', ',', ';', ']','}'].as_ref());
            if candidate.len() >= 46 { return Some(candidate.to_string()); }
        }
        if let Some(pos) = lower.find(&format!("{}: ", key)) {
            let after = &log[pos + key.len() + 2..];
            let end = after.find(|c: char| c.is_whitespace() || c == ',' || c == '"' || c == '\n').unwrap_or(after.len());
            let candidate = after[..end].trim().trim_matches(['"', '\'', ',', ';', ']','}'].as_ref());
            if candidate.len() >= 46 { return Some(candidate.to_string()); }
        }
    }
    // Fallback scan for Qm... pattern (CIDv0)
    for part in log.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        let trimmed = part.trim_matches(['"', '\'', ',', ';', ']','}'].as_ref());
        if trimmed.starts_with("Qm") && trimmed.len() >= 46 && trimmed.len() <= 60 { // crude length gate
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Shared ingestion logic used by the blockchain listener and the manual API ingestion endpoint.
pub async fn ingest_event(
    pool: &Pool,
    event: &crate::db::models::Event,
    log: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event.event_type.as_str() {
        "PackagePublished" => {
            let ipfs = extract_ipfs_hash(log).unwrap_or_else(|| "unknown".to_string());
            if let Some(ver) = &event.version {
                match crate::db::queries::insert_package(
                    pool,
                    &event.package_name,
                    "unknown",
                    None,
                    None,
                    None,
                ).await {
                    Ok(pkg_id) => {
                        if ipfs != "unknown" {
                            if let Err(e) = crate::db::queries::insert_version(pool, pkg_id, ver, &ipfs).await {
                                tracing::warn!("Failed to insert version {} for {}: {}", ver, event.package_name, e);
                            } else {
                                tracing::info!("Stored published version {}@{} (ipfs={})", event.package_name, ver, &ipfs[..8.min(ipfs.len())]);
                            }
                        } else {
                            tracing::debug!("No IPFS hash detected for published package {}@{}", event.package_name, ver);
                        }
                    }
                    Err(e) => tracing::warn!("Failed upsert package {}: {}", event.package_name, e),
                }
            } else {
                tracing::warn!("Publish event missing version for package {}", event.package_name);
            }
        }
        "PackageUpdated" => {
            let ipfs = extract_ipfs_hash(log).unwrap_or_else(|| "unknown".to_string());
            if let Some(ver) = &event.version {
                let pkg_id = match crate::db::queries::get_package_id(pool, &event.package_name).await {
                    Ok(Some(id)) => id,
                    _ => match crate::db::queries::insert_package(pool, &event.package_name, "unknown", None, None, None).await {
                        Ok(id) => id,
                        Err(e) => { tracing::warn!("Failed create package on update {}: {}", event.package_name, e); return Ok(()); }
                    },
                };
                if ipfs != "unknown" {
                    if let Err(e) = crate::db::queries::insert_version(pool, pkg_id, ver, &ipfs).await {
                        tracing::warn!("Failed upsert updated version {} for {}: {}", ver, event.package_name, e);
                    } else {
                        tracing::info!("Updated version {}@{} (ipfs={})", event.package_name, ver, &ipfs[..8.min(ipfs.len())]);
                    }
                } else {
                    tracing::debug!("Update event without IPFS for {}@{}", event.package_name, ver);
                }
            }
        }
        "PackageDownloaded" => {
            if let Some(ver) = &event.version {
                if let Ok(Some(pkg_id)) = crate::db::queries::get_package_id(pool, &event.package_name).await {
                    if let Ok(Some(ver_id)) = crate::db::queries::get_version_id(pool, pkg_id, ver).await {
                        if let Err(e) = crate::db::queries::increment_download(pool, pkg_id, ver_id).await {
                            tracing::warn!("Failed increment download for {}@{}: {}", event.package_name, ver, e);
                        } else {
                            tracing::info!("Incremented downloads for {}@{}", event.package_name, ver);
                        }
                    } else {
                        tracing::debug!("Download event version not found {}@{} (maybe publish not processed yet)", event.package_name, ver);
                    }
                } else {
                    tracing::debug!("Download event package not found {} (maybe publish not processed yet)", event.package_name);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::extract_ipfs_hash;

    #[test]
    fn test_extract_ipfs_basic_patterns() {
        assert_eq!(extract_ipfs_hash("ipfs=Qmabcdef123456789012345678901234567890123456789"), Some("Qmabcdef123456789012345678901234567890123456789".to_string()));
        assert_eq!(extract_ipfs_hash("cid=QmABCDEF123456789012345678901234567890123456789"), Some("QmABCDEF123456789012345678901234567890123456789".to_string()));
        assert!(extract_ipfs_hash("no cid here").is_none());
    // The fallback detection requires CID length >= 46, ensure test string meets that
    let cid = "QmZXYW9876543210987654321098765432109876543210123"; // length  Fifty? adjust
    assert!(cid.len() >= 46);
    assert_eq!(extract_ipfs_hash(&format!("Random text {} more", cid)), Some(cid.to_string()));
    }
}

pub async fn start_indexer(
    pool: Pool,
    rpc_url: String,
    program_id_str: String,
    start_slot_override: Option<u64>,
    poll_interval_secs: u64,
) {
    tracing::info!("Starting indexer for program: {}", program_id_str);
    
    let program_id = match Pubkey::from_str(&program_id_str) {
        Ok(pk) => pk,
        Err(e) => {
            tracing::error!("Invalid program ID: {}", e);
            return;
        }
    };
    
    let rpc_client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());
    
    // Determine starting slot: existing state > override > current slot
    let mut last_slot = match crate::db::queries::get_last_processed_slot(&pool).await {
        Ok(slot) if slot > 0 => {
            tracing::info!("Resuming from last processed slot: {}", slot);
            slot
        }
        _ => {
            if let Some(override_slot) = start_slot_override {
                tracing::info!("Indexer initial state empty; backfilling from override start slot {}", override_slot);
                override_slot
            } else {
                match rpc_client.get_slot() {
                    Ok(slot) => {
                        tracing::info!("Indexer initial state empty; starting from current slot {} (no historical backfill override provided)", slot);
                        slot
                    }
                    Err(e) => {
                        tracing::error!("Failed to get initial slot: {}", e);
                        return;
                    }
                }
            }
        }
    };
    
    let mut retry_count = 0;
    let max_retries = 5;
    let mut error_backoff = Duration::from_secs(2);
    
    loop {
        match rpc_client.get_slot() {
            Ok(current_slot) => {
                retry_count = 0;
                error_backoff = Duration::from_secs(2); // Reset backoff
                
                if current_slot > last_slot {
                    let slots_to_process = current_slot - last_slot;
                    
                    if slots_to_process > 100 {
                        tracing::info!("Processing {} slots ({} to {}), this may take a while...", 
                            slots_to_process, last_slot + 1, current_slot);
                    } else {
                        tracing::debug!("Processing slots {} to {}", last_slot + 1, current_slot);
                    }
                    
                    for slot in last_slot + 1..=current_slot {
                        match process_slot(&rpc_client, &pool, slot, &program_id).await {
                            Ok(_) => {
                                // Update database state every 10 slots or on last slot
                                if slot % 10 == 0 || slot == current_slot {
                                    if let Err(e) = crate::db::queries::update_last_processed_slot(
                                        &pool, 
                                        slot, 
                                        None
                                    ).await {
                                        tracing::warn!("Failed to update last processed slot: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Error processing slot {}: {}", slot, e);
                                if let Err(db_err) = crate::db::queries::update_indexer_error(
                                    &pool,
                                    &format!("Slot {}: {}", slot, e)
                                ).await {
                                    tracing::error!("Failed to log error to database: {}", db_err);
                                }
                            }
                        }
                    }
                    
                    last_slot = current_slot;
                }
            }
            Err(e) => {
                retry_count += 1;
                tracing::error!("Failed to get current slot (attempt {}/{}): {}", 
                    retry_count, max_retries, e);
                
                if retry_count >= max_retries {
                    tracing::error!("Max retries reached, using exponential backoff...");
                    error_backoff = std::cmp::min(error_backoff * 2, Duration::from_secs(300));
                    tracing::info!("Waiting {:?} before retry", error_backoff);
                    sleep(error_backoff).await;
                    retry_count = 0;
                } else {
                    sleep(Duration::from_secs(5)).await;
                }
                
                if let Err(db_err) = crate::db::queries::update_indexer_error(
                    &pool,
                    &format!("RPC error: {}", e)
                ).await {
                    tracing::error!("Failed to log error to database: {}", db_err);
                }
                
                continue;
            }
        }
        
        // Normal polling interval (configurable)
        sleep(Duration::from_secs(poll_interval_secs)).await;
    }
}

async fn process_slot(
    rpc_client: &RpcClient,
    pool: &Pool,
    slot: u64,
    program_id: &Pubkey,
) -> Result<(), anyhow::Error> {
    let block = match rpc_client.get_block_with_config(
        slot,
        solana_client::rpc_config::RpcBlockConfig {
            encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
            transaction_details: Some(solana_transaction_status::TransactionDetails::Full),
            rewards: Some(false),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        },
    ) {
        Ok(block) => block,
        Err(e) => {
            // Slot might not be available yet or skipped
            let err_str = e.to_string();
            if err_str.contains("skipped") || err_str.contains("not available") {
                tracing::debug!("Slot {} skipped or not available: {}", slot, e);
                return Ok(());
            }
            return Err(anyhow::Error::from(e));
        }
    };
    
    let transactions = match block.transactions {
        Some(txs) => txs,
        None => return Ok(()),
    };
    
    let mut events_found = 0;
    
    for tx_with_meta in transactions {
        // Extract signature from transaction
        let signature = match &tx_with_meta.transaction {
            solana_transaction_status::EncodedTransaction::Json(ui_tx) => {
                ui_tx.signatures.get(0).cloned().unwrap_or_default()
            }
            _ => String::new(),
        };

        if signature.is_empty() {
            continue;
        }

        if let Some(meta) = tx_with_meta.meta {
            // Check if transaction was successful
            if meta.err.is_some() {
                tracing::trace!("Skipping failed transaction: {}", signature);
                continue;
            }
            
            if let solana_transaction_status::option_serializer::OptionSerializer::Some(logs) = &meta.log_messages {
                let mut has_program_invocation = false;
                
                // Check if our program was invoked
                for log in logs {
tracing::debug!("Indexer saw log: {}", log);
                    if log.contains(&program_id.to_string()) {
                        has_program_invocation = true;
                        break;
                    }
                }
                
                if !has_program_invocation {
                    continue;
                }
                
                // Parse all logs for this transaction
                for log in logs {
tracing::debug!("Indexer saw log: {}", log);
                    if let Some(event) = parse_transaction(&log, &signature, slot as i64, block.block_time) {
                        match crate::db::queries::insert_event(
                            pool,
                            &event.event_type,
                            &event.package_name,
                            event.version.as_deref(),
                            &event.transaction_signature,
                            event.slot,
                            block.block_time,
                        ).await {
                            Ok(_) => {
                                events_found += 1;
                                tracing::info!(
                                    "Indexed event: {} for package {} (slot: {}, tx: {})", 
                                    event.event_type, 
                                    event.package_name,
                                    slot,
                                    &signature[..8]
                                );
                                // If this is a publish event, attempt to upsert package + version metadata
                                // Ingestion logic based on event type
                                // Delegate ingestion work to helper
                                if let Err(e) = ingest_event(pool, &event, &log).await {
                                    tracing::warn!("Ingestion helper failed for {}: {}", event.event_type, e);
                                }
                            }
                            Err(e) => {
                                // Ignore duplicate key errors (transaction signature already exists)
                                if !e.to_string().contains("duplicate") {
                                    tracing::warn!("Failed to insert event: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    if events_found > 0 {
        tracing::info!("Found {} events in slot {}", events_found, slot);
    }
    
    Ok(())
}
