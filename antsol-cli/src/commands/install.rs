use crate::config::Config;
use crate::ipfs::IpfsClient;
use crate::solana_client::AntSolClient;
use crate::types::Result;
use crate::utils::*;
use colored::*;
use std::path::PathBuf;

/// Install a package from the decentralized registry
pub async fn handle_install(package_spec: String) -> Result<()> {
    let (name, version) = parse_package_spec(&package_spec);
    
    println!("\n{}", "📥 Installing from Decentralized Registry".cyan().bold());
    print_info(&format!("Package: {}...", name.cyan()));
    
    // Load config
    let config = Config::load()?;
    let solana_client = AntSolClient::new(&config)?;
    
    // Determine version
    let version = version.unwrap_or_else(|| {
        print_warning("No version specified, attempting to find latest");
        "latest".to_string()
    });
    
    // Step 1: Fetch package metadata from blockchain
    let spinner = create_spinner("🔍 Fetching package metadata from blockchain...");
    let package = match solana_client.get_package(&name, &version) {
        Ok(Some(pkg)) => pkg,
        Ok(None) => {
            spinner.finish_and_clear();
            return Err(format!("Package {}@{} not found on-chain", name, version).into());
        }
        Err(e) => {
            spinner.finish_and_clear();
            return Err(format!("Error fetching package from blockchain: {}", e).into());
        }
    };
    spinner.finish_and_clear();
    
    print_success(&format!("Found {}@{} on blockchain", name.green(), version.green()));
    
    // Step 2: Verify dependencies
    if !package.dependencies.is_empty() {
        let spinner = create_spinner("🔗 Verifying dependencies on-chain...");
        for dep in &package.dependencies {
            let dep_exists = solana_client.get_package(&dep.name, &dep.version)?.is_some();
            if !dep_exists {
                spinner.finish_and_clear();
                return Err(format!("❌ Dependency {}@{} not found on-chain", dep.name, dep.version).into());
            }
        }
        spinner.finish_and_clear();
        print_success(&format!("All {} dependencies verified on blockchain", package.dependencies.len()));
    }
    
    // Step 3: Download from IPFS
    let spinner = create_spinner("⬇️  Downloading package from IPFS (verifying integrity)...");
    let ipfs_client = IpfsClient::new(config.ipfs_url);
    
    let packages_dir = PathBuf::from("antsol_packages");
    std::fs::create_dir_all(&packages_dir)?;
    
    let package_dir = packages_dir.join(&name);
    std::fs::create_dir_all(&package_dir)?;
    
    ipfs_client.download_package(&package.ipfs_cid, &package_dir).await?;
    spinner.finish_and_clear();
    
    print_success(&format!("Installed {}@{} with cryptographic verification", name.green().bold(), version.green()));
    
    println!("\n{}", "✨ Package Installed Successfully!".green().bold());
    println!("{}", "═".repeat(80).cyan());
    println!("\n{}", "📦 Installation Details:".cyan().bold());
    println!("  Name: {}", package.name.green());
    println!("  Version: {}", package.version.cyan());
    println!("  Location: {}", package_dir.display().to_string().yellow());
    println!("  Description: {}", package.description);
    println!("  IPFS CID: {}", package.ipfs_cid.cyan());
    
    if !package.dependencies.is_empty() {
        println!("\n{}", "🔗 Dependencies:".blue().bold());
        for dep in &package.dependencies {
            println!("  • {}@{}", dep.name.green(), dep.version.yellow());
        }
    }
    
    if !package.external_dependencies.is_empty() {
        println!("\n{}", "📦 External Dependencies:".blue().bold());
        for dep in &package.external_dependencies {
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
    
    println!("\n{}", "💡 Import in your code:".yellow());
    println!("  {}", format!("use antsol_packages::{};", name.replace("-", "_")).cyan());
    
    println!("\n{}", "🔐 Security:".green().bold());
    println!("  ✓ On-chain verification passed");
    println!("  ✓ IPFS content integrity verified");
    println!("  ✓ Dependencies checked on blockchain");
    
    Ok(())
}
