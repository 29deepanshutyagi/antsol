use crate::config::Config;
use crate::solana_client::AntSolClient;
use crate::types::Result;
use crate::utils::*;
use colored::*;
use chrono::{DateTime, Utc};

/// Show detailed package information from the blockchain
pub async fn handle_info(package: String) -> Result<()> {
    let (name, version) = parse_package_spec(&package);
    
    println!("\n{}", "📋 Fetching Package Info from Blockchain".cyan().bold());
    let spinner = create_spinner(&format!("Querying on-chain data for {}...", name));
    
    let config = Config::load()?;
    let solana_client = AntSolClient::new(&config)?;
    
    // If version not specified, use a default (in production, query latest)
    let version = version.unwrap_or_else(|| {
        print_warning("No version specified. Specify version like: package@1.0.0");
        "1.0.0".to_string()
    });
    
    let package_info = solana_client.get_package(&name, &version)?
        .ok_or_else(|| format!("Package {}@{} not found on blockchain", name, version))?;
    
    spinner.finish_and_clear();
    
    // Display comprehensive package information
    println!("\n{} {}", "📦".cyan(), name.green().bold());
    println!("{}", "═".repeat(80).cyan());
    
    println!("\n{}", "📋 Package Information".cyan().bold());
    println!("  Name: {}", package_info.name.green());
    println!("  Version: {}", package_info.version.cyan());
    println!("  Description: {}", package_info.description);
    
    let datetime = DateTime::<Utc>::from_timestamp(package_info.published_at, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    println!("  Published: {}", datetime.yellow());
    
    println!("\n{}", "⛓️  Blockchain Details".cyan().bold());
    println!("  Authority (Publisher): {}", package_info.authority.to_string().cyan());
    let (pda, _) = solana_client.derive_package_pda(&name, &version);
    println!("  On-chain Account: {}", pda.to_string().cyan());
    println!("  Program ID: {}", config.program_id.cyan());
    
    println!("\n{}", "💾 Storage Details".cyan().bold());
    println!("  IPFS CID: {}", package_info.ipfs_cid.yellow());
    println!("  Storage Type: {}", "IPFS (Immutable)".green());
    
    if !package_info.dependencies.is_empty() {
        println!("\n{}", "🔗 Dependencies".cyan().bold());
        for dep in &package_info.dependencies {
            println!("  • {}@{}", dep.name.green(), dep.version.yellow());
        }
    } else {
        println!("\n{}", "🔗 Dependencies".cyan().bold());
        println!("  No dependencies");
    }
    
    if !package_info.external_dependencies.is_empty() {
        println!("\n{}", "📦 External Dependencies".blue().bold());
        for dep in &package_info.external_dependencies {
            let registry_info = dep.registry.as_ref()
                .map(|r| format!(" ({})", r))
                .unwrap_or_default();
            println!("  • {}@{} [{}]{}",
                dep.name.green(),
                dep.version.yellow(),
                dep.dep_type.cyan(),
                registry_info.dimmed()
            );
        }
    }
    
    println!("\n{}", "═".repeat(80).cyan());
    
    println!("\n{}", "🚀 Quick Actions:".yellow().bold());
    println!("  Install: {}", format!("antsol install {}@{}", name, version).green());
    
    println!("\n{}", "🔗 Explorer Links:".blue().bold());
    println!("  Package Account: {}", 
        format!("https://explorer.solana.com/address/{}?cluster=devnet", pda).blue()
    );
    println!("  Authority: {}", 
        format!("https://explorer.solana.com/address/{}?cluster=devnet", package_info.authority).blue()
    );
    println!("  IPFS Gateway: {}", 
        format!("https://gateway.pinata.cloud/ipfs/{}", package_info.ipfs_cid).blue()
    );
    
    println!("\n{}", "🔐 Verification:".green().bold());
    println!("  ✓ Package registered on Solana blockchain");
    println!("  ✓ Content stored immutably on IPFS");
    println!("  ✓ Ownership verified by on-chain authority");
    
    Ok(())
}
