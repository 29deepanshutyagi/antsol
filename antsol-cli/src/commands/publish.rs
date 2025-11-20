use crate::config::Config;
use crate::ipfs::IpfsClient;
use crate::solana_client::AntSolClient;
use crate::types::{AntSolManifest, Result};
use crate::utils::*;
use colored::*;
use solana_sdk::signature::Keypair;
use std::path::PathBuf;

pub async fn handle_publish(path: PathBuf, version_override: Option<String>) -> Result<()> {
    let manifest_path = path.join("antsol.toml");
    if !manifest_path.exists() {
        return Err("No antsol.toml found. Run 'antsol init' first.".into());
    }
    
    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let mut manifest: AntSolManifest = toml::from_str(&manifest_content)?;
    
    if let Some(version) = version_override {
        if !validate_version(&version) {
            return Err("Invalid version format".into());
        }
        manifest.package.version = version;
    }
    
    print_info(&format!("Publishing {} v{}", manifest.package.name.cyan(), manifest.package.version.cyan()));
    
    // Load wallet and config
    let config = Config::load()?;
    let wallet_path = config.wallet_path.as_ref().ok_or("No wallet connected. Use 'antsol wallet connect'")?;
    let keypair_bytes = std::fs::read(&wallet_path)?;
    let keypair_vec: Vec<u8> = serde_json::from_slice(&keypair_bytes)?;
    let keypair = Keypair::from_bytes(&keypair_vec)?;
    
    let spinner = create_spinner("Uploading package to IPFS...");
    
    // Create IPFS client with JWT from config or environment
    let ipfs_client = if let Some(jwt) = config.pinata_jwt.clone() {
        IpfsClient::with_jwt(config.ipfs_url.clone(), jwt)
    } else {
        IpfsClient::new(config.ipfs_url.clone())
    };
    
    let cid = ipfs_client.upload_package(&path).await?;
    spinner.finish_and_clear();
    print_success(&format!("Uploaded to IPFS: {}", cid.green()));
    
    let spinner = create_spinner("Publishing to Solana...");
    let solana_client = AntSolClient::new(&config)?;
    
    let signature = solana_client.publish_package(
        &keypair,
        manifest.package.name.clone(),
        manifest.package.version.clone(),
        cid.clone(),
        manifest.package.description.clone(),
        manifest.dependencies.unwrap_or_default(),
        manifest.external_dependencies.unwrap_or_default(),
    ).await?;
    
    spinner.finish_and_clear();
    
    print_success(&format!("Published {}@{}", manifest.package.name.green().bold(), manifest.package.version.green()));
    
    println!("\n{}", "Package Details".cyan().bold());
    println!("  IPFS CID: {}", cid.cyan());
    println!("  Transaction: {}", signature.cyan());
    println!("  Explorer: {}", format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature).blue());
    
    Ok(())
}
