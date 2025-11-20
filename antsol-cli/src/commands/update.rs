use crate::config::Config;
use crate::ipfs::IpfsClient;
use crate::solana_client::AntSolClient;
use crate::types::{AntSolManifest, Result};
use crate::utils::*;
use colored::*;
use solana_sdk::signature::Keypair;
use std::path::PathBuf;

pub async fn handle_update(path: PathBuf, new_version: String) -> Result<()> {
    if !validate_version(&new_version) {
        return Err("Invalid version format. Use semantic versioning (e.g., 1.0.1)".into());
    }
    
    let manifest_path = path.join("antsol.toml");
    if !manifest_path.exists() {
        return Err("No antsol.toml found.".into());
    }
    
    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let manifest: AntSolManifest = toml::from_str(&manifest_content)?;
    let old_version = manifest.package.version.clone();
    
    print_info(&format!("Updating {} from {} to {}", manifest.package.name.cyan(), old_version.yellow(), new_version.green()));
    
    // Load wallet and config
    let config = Config::load()?;
    let wallet_path = config.wallet_path.as_ref().ok_or("No wallet connected")?;
    let keypair_bytes = std::fs::read(&wallet_path)?;
    let keypair_vec: Vec<u8> = serde_json::from_slice(&keypair_bytes)?;
    let keypair = Keypair::from_bytes(&keypair_vec)?;
    
    let spinner = create_spinner("Uploading updated package to IPFS...");
    
    // Create IPFS client with JWT from config or environment
    let ipfs_client = if let Some(jwt) = config.pinata_jwt.clone() {
        IpfsClient::with_jwt(config.ipfs_url.clone(), jwt)
    } else {
        IpfsClient::new(config.ipfs_url.clone())
    };
    
    let new_cid = ipfs_client.upload_package(&path).await?;
    spinner.finish_and_clear();
    print_success(&format!("New IPFS CID: {}", new_cid.green()));
    
    let spinner = create_spinner("Updating package on Solana...");
    let solana_client = AntSolClient::new(&config)?;
    
    let signature = solana_client.update_package(
        &keypair,
        manifest.package.name.clone(),
        old_version.clone(),
        new_version.clone(),
        new_cid.clone(),
        manifest.package.description.clone(),
        manifest.dependencies.unwrap_or_default(),
        manifest.external_dependencies.unwrap_or_default(),
    ).await?;
    
    spinner.finish_and_clear();
    
    print_success(&format!("Updated {}@{}", manifest.package.name.green().bold(), new_version.green()));
    
    println!("\n{}", "Update Details".cyan().bold());
    println!("  Previous: {}", old_version.yellow());
    println!("  Current: {}", new_version.green());
    println!("  New IPFS CID: {}", new_cid.cyan());
    println!("  Transaction: {}", signature.cyan());
    println!("  Explorer: {}", format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature).blue());
    
    print_info("\nDon't forget to update antsol.toml with the new version!");
    
    Ok(())
}
