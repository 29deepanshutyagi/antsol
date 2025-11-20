use crate::config::Config;
use crate::utils::*;
use colored::*;
use solana_sdk::signature::{Keypair, Signer};
use std::io::Write;
use std::path::PathBuf;

pub async fn handle_setup() -> Result<(), Box<dyn std::error::Error>> {
    print_info("🚀 Welcome to AntSol Setup - Let's configure your decentralized registry CLI!\n");
    
    // Load existing config or create default
    let mut config = Config::load().unwrap_or_else(|_| Config {
        rpc_url: "https://api.devnet.solana.com".to_string(),
        ipfs_url: "https://api.pinata.cloud".to_string(),
        program_id: "A9igkBugcujD9Nw9d97FFN4aY3qHXnJxEqCChJt8C42S".to_string(),
        wallet_path: None,
        pinata_jwt: None,
        indexer_url: Config::default_indexer_url(),
    });
    
    println!("{}", "═".repeat(50).cyan());
    println!("{}", "Step 1: Wallet Configuration".cyan().bold());
    println!("{}", "═".repeat(50).cyan());
    
    // Wallet setup
    print!("\nDo you want to connect a wallet now? (Y/n): ");
    std::io::stdout().flush()?;
    let mut wallet_choice = String::new();
    std::io::stdin().read_line(&mut wallet_choice)?;
    
    if wallet_choice.trim().to_lowercase() != "n" {
        print!("Enter path to your Solana keypair JSON file: ");
        print!("\n  (Default: ~/.config/solana/id.json): ");
        std::io::stdout().flush()?;
        
        let mut keypair_path = String::new();
        std::io::stdin().read_line(&mut keypair_path)?;
        let keypair_path = keypair_path.trim();
        
        let wallet_path = if keypair_path.is_empty() {
            let home = std::env::var("HOME")?;
            PathBuf::from(format!("{}/.config/solana/id.json", home))
        } else {
            PathBuf::from(keypair_path)
        };
        
        // Validate wallet
        if !wallet_path.exists() {
            print_warning(&format!("Wallet file not found at: {}", wallet_path.display()));
            print_info("You can connect a wallet later with: antsol wallet connect <path>");
        } else {
            // Try to load the keypair
            match std::fs::read(&wallet_path) {
                Ok(keypair_bytes) => {
                    match serde_json::from_slice::<Vec<u8>>(&keypair_bytes) {
                        Ok(keypair_vec) => {
                            match Keypair::from_bytes(&keypair_vec) {
                                Ok(keypair) => {
                                    config.wallet_path = Some(wallet_path.clone());
                                    print_success(&format!("✓ Wallet connected: {}", keypair.pubkey()));
                                    print_info(&format!("  Path: {}", wallet_path.display()));
                                }
                                Err(e) => {
                                    print_warning(&format!("Invalid keypair format: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            print_warning(&format!("Could not parse keypair: {}", e));
                        }
                    }
                }
                Err(e) => {
                    print_warning(&format!("Could not read wallet file: {}", e));
                }
            }
        }
    } else {
        print_info("Skipped wallet setup. You can connect later with: antsol wallet connect <path>");
    }
    
    println!("\n{}", "═".repeat(50).cyan());
    println!("{}", "Step 2: IPFS Configuration (Pinata)".cyan().bold());
    println!("{}", "═".repeat(50).cyan());
    
    println!("\n{}", "ℹ️  Pinata JWT is required for publishing packages to IPFS.".yellow());
    println!("   Get your free token at: {}", "https://app.pinata.cloud".blue().underline());
    
    print!("\nEnter your Pinata JWT token (or press Enter to skip): ");
    std::io::stdout().flush()?;
    let mut jwt = String::new();
    std::io::stdin().read_line(&mut jwt)?;
    let jwt = jwt.trim().to_string();
    
    if !jwt.is_empty() {
        config.pinata_jwt = Some(jwt);
        print_success("✓ Pinata JWT token saved");
    } else {
        print_info("Skipped Pinata JWT. You can add it later by editing: ~/.antsol/config.toml");
    }
    
    println!("\n{}", "═".repeat(50).cyan());
    println!("{}", "Step 3: Network Configuration".cyan().bold());
    println!("{}", "═".repeat(50).cyan());
    
    println!("\nCurrent RPC URL: {}", config.rpc_url.cyan());
    print!("Change RPC URL? (y/N): ");
    std::io::stdout().flush()?;
    let mut rpc_choice = String::new();
    std::io::stdin().read_line(&mut rpc_choice)?;
    
    if rpc_choice.trim().to_lowercase() == "y" {
        println!("\nSelect network:");
        println!("  1. Devnet (default) - {}", "https://api.devnet.solana.com".cyan());
        println!("  2. Mainnet - {}", "https://api.mainnet-beta.solana.com".cyan());
        println!("  3. Custom URL");
        
        print!("\nChoice (1-3): ");
        std::io::stdout().flush()?;
        let mut network_choice = String::new();
        std::io::stdin().read_line(&mut network_choice)?;
        
        match network_choice.trim() {
            "1" => config.rpc_url = "https://api.devnet.solana.com".to_string(),
            "2" => config.rpc_url = "https://api.mainnet-beta.solana.com".to_string(),
            "3" => {
                print!("Enter custom RPC URL: ");
                std::io::stdout().flush()?;
                let mut custom_url = String::new();
                std::io::stdin().read_line(&mut custom_url)?;
                config.rpc_url = custom_url.trim().to_string();
            }
            _ => print_info("Keeping current RPC URL"),
        }
        print_success(&format!("✓ RPC URL set to: {}", config.rpc_url));
    }
    
    // New: Indexer configuration
    println!("\n{}", "═".repeat(50).cyan());
    println!("{}", "Step 4: Indexer Configuration".cyan().bold());
    println!("{}", "═".repeat(50).cyan());

    println!("\nCurrent Indexer URL: {}", config.indexer_url.cyan());
    print!("Change Indexer URL? (y/N): ");
    std::io::stdout().flush()?;
    let mut idx_choice = String::new();
    std::io::stdin().read_line(&mut idx_choice)?;

    if idx_choice.trim().to_lowercase() == "y" {
        print!("Enter Indexer base URL (e.g., https://antsol-indexer-v2.onrender.com): ");
        std::io::stdout().flush()?;
        let mut new_idx = String::new();
        std::io::stdin().read_line(&mut new_idx)?;
        let url = new_idx.trim().to_string();
        if !url.is_empty() {
            // quick health check hint
            print_info("Tip: Indexer health endpoint should be /health returning { success: true }");
            config.indexer_url = url;
            print_success(&format!("✓ Indexer URL set to: {}", config.indexer_url));
        }
    }
    
    // Save configuration
    config.save()?;
    
    println!("\n{}", "═".repeat(50).green());
    println!("{}", "🎉 Setup Complete!".green().bold());
    println!("{}", "═".repeat(50).green());
    
    println!("\n{}", "Configuration Summary:".cyan().bold());
    println!("  RPC URL: {}", config.rpc_url.cyan());
    println!("  IPFS Gateway: {}", config.ipfs_url.cyan());
    println!("  Program ID: {}", config.program_id.cyan());
    println!("  Indexer URL: {}", config.indexer_url.cyan());
    
    if let Some(wallet) = &config.wallet_path {
        println!("  Wallet: {} {}", "✓".green(), wallet.display().to_string().cyan());
    } else {
        println!("  Wallet: {} {}", "✗".red(), "Not connected".yellow());
    }
    
    if config.pinata_jwt.is_some() {
        println!("  Pinata JWT: {} {}", "✓".green(), "Configured".cyan());
    } else {
        println!("  Pinata JWT: {} {}", "✗".red(), "Not configured".yellow());
    }
    
    println!("\n{}", "Next Steps:".cyan().bold());
    
    if config.wallet_path.is_none() {
        println!("  • Connect wallet: {}", "antsol wallet connect <keypair.json>".yellow());
    }
    
    if config.pinata_jwt.is_none() {
        println!("  • Add Pinata JWT to: {}", "~/.antsol/config.toml".yellow());
    }
    
    if config.wallet_path.is_some() {
        println!("  • Get devnet SOL: {}", "solana airdrop 2 --url devnet".cyan());
    }
    
    println!("  • Initialize a package: {}", "antsol init".cyan());
    println!("  • Publish a package: {}", "antsol publish".cyan());
    println!("  • Search registry: {}", "antsol search <query>".cyan());
    
    println!("\n{}", "Configuration saved to: ~/.antsol/config.toml".green());
    
    Ok(())
}
