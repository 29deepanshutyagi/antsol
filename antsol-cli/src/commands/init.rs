use crate::types::{AntSolManifest, PackageInfo, Result};
use crate::utils::*;
use colored::*;
use std::io::Write;

/// Initialize a new package for the decentralized registry
pub async fn handle_init() -> Result<()> {
    print_info("Initializing package for AntSol decentralized registry...");
    
    // Check if antsol.toml already exists
    if std::path::Path::new("antsol.toml").exists() {
        return Err("antsol.toml already exists!".into());
    }
    
    // Interactive prompts
    print!("\n📦 Package name: ");
    std::io::stdout().flush()?;
    let mut name = String::new();
    std::io::stdin().read_line(&mut name)?;
    let name = name.trim().to_string();
    
    if !validate_package_name(&name) {
        return Err("Invalid package name. Use only lowercase letters, numbers, and hyphens.".into());
    }
    
    print!("📌 Version (default: 0.1.0): ");
    std::io::stdout().flush()?;
    let mut version = String::new();
    std::io::stdin().read_line(&mut version)?;
    let version = version.trim();
    let version = if version.is_empty() {
        "0.1.0".to_string()
    } else {
        version.to_string()
    };
    
    if !validate_version(&version) {
        return Err("Invalid version. Use semantic versioning (e.g., 1.0.0)".into());
    }
    
    print!("📝 Description: ");
    std::io::stdout().flush()?;
    let mut description = String::new();
    std::io::stdin().read_line(&mut description)?;
    let description = description.trim().to_string();
    
    // Create manifest
    let manifest = AntSolManifest {
        package: PackageInfo {
            name,
            version,
            description,
            authors: None,
            license: None,
        },
        dependencies: None,
        external_dependencies: None,
    };
    
    // Write to file
    let toml_string = toml::to_string_pretty(&manifest)?;
    std::fs::write("antsol.toml", toml_string)?;
    
    print_success("Created antsol.toml manifest");
    
    println!("\n{}", "🚀 Next Steps:".cyan().bold());
    println!("  1. Add your package files to this directory");
    println!("  2. Connect your Solana wallet: {}", "antsol wallet connect <keypair.json>".cyan());
    println!("  3. Publish to blockchain: {}", "antsol publish".cyan());
    println!("\n{}", "💡 Your package will be:".yellow());
    println!("  • Stored immutably on IPFS");
    println!("  • Registered on-chain on Solana");
    println!("  • Verifiable and censorship-resistant");
    
    Ok(())
}
