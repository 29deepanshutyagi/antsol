use crate::config::Config;
use crate::types::Result;
use crate::utils::*;
use colored::*;
use solana_sdk::signature::{Keypair, Signer};
use std::path::PathBuf;

pub async fn handle_connect(keypair_path: PathBuf) -> Result<()> {
    let spinner = create_spinner("Connecting wallet to decentralized registry...");
    
    let keypair_bytes = std::fs::read(&keypair_path)?;
    let keypair_vec: Vec<u8> = serde_json::from_slice(&keypair_bytes)?;
    let keypair = Keypair::from_bytes(&keypair_vec)?;
    
    let mut config = Config::load()?;
    config.wallet_path = Some(keypair_path.clone());
    config.save()?;
    
    spinner.finish_and_clear();
    
    print_success(&format!("Wallet connected: {}", keypair.pubkey().to_string().cyan()));
    println!("\n{}", "🔐 Wallet Details:".cyan().bold());
    println!("  Public Key: {}", keypair.pubkey().to_string().green());
    println!("  Keypair Path: {}", keypair_path.display());
    println!("\n{}", "⚡ This wallet will be used for:".yellow());
    println!("  • Signing package publish transactions");
    println!("  • Proving package ownership on-chain");
    println!("  • Updating package versions");
    
    Ok(())
}

pub async fn handle_show() -> Result<()> {
    let config = Config::load()?;
    
    if let Some(wallet_path) = config.wallet_path {
        let keypair_bytes = std::fs::read(&wallet_path)?;
        let keypair_vec: Vec<u8> = serde_json::from_slice(&keypair_bytes)?;
        let keypair = Keypair::from_bytes(&keypair_vec)?;
        
        println!("\n{}", "🔐 Current Wallet".cyan().bold());
        println!("  Address: {}", keypair.pubkey().to_string().green());
        println!("  Path: {}", wallet_path.display());
        
        println!("\n{}", "🌐 Network Configuration".cyan().bold());
        println!("  RPC Endpoint: {}", config.rpc_url.yellow());
        println!("  Program ID: {}", config.program_id.yellow());
        
        println!("\n{}", "💾 IPFS Storage".cyan().bold());
        println!("  API URL: {}", config.ipfs_url.yellow());
        
        println!("\n{}", "🔗 Explorer Links:".blue().bold());
        println!("  Wallet: {}", 
            format!("https://explorer.solana.com/address/{}?cluster=devnet", keypair.pubkey()).blue()
        );
        println!("  Program: {}", 
            format!("https://explorer.solana.com/address/{}?cluster=devnet", config.program_id).blue()
        );
    } else {
        print_warning("No wallet connected to the decentralized registry.");
        println!("\n{}", "📌 To get started:".yellow());
        println!("  {}", "antsol wallet connect <path-to-keypair.json>".cyan());
        println!("\n{}", "💡 Don't have a wallet?".blue());
        println!("  Generate one with: {}", "solana-keygen new".cyan());
    }
    
    Ok(())
}
