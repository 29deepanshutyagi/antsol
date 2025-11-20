use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

mod commands;
mod config;
mod ipfs;
mod solana_client;
mod types;
mod utils;

use commands::*;

#[derive(Parser)]
#[command(name = "antsol")]
#[command(about = "AntSol - Decentralized Package Registry on Solana Blockchain", long_about = "
A fully decentralized package registry powered by Solana blockchain and IPFS.
• Publish packages with on-chain verification
• Install packages with cryptographic integrity checks
• Immutable, censorship-resistant package storage
• Wallet-based ownership and version control
")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run initial setup wizard for configuration
    Setup,
    
    /// Initialize package manifest for decentralized registry
    Init,
    
    /// Publish a package to the on-chain registry
    Publish {
        /// Path to package directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        
        /// Specify version (overrides manifest)
        #[arg(short, long)]
        version: Option<String>,
    },
    
    /// Install a package from the decentralized registry
    Install {
        /// Package name with optional version (e.g., spl-token-utils@1.0.0)
        package: String,
    },
    
    /// Search for packages in the registry
    Search {
        /// Search query
        query: String,
    },
    
    /// Show package information from blockchain
    Info {
        /// Package name
        package: String,
    },
    
    /// Manage wallet for on-chain transactions
    Wallet {
        #[command(subcommand)]
        action: WalletAction,
    },
    
    /// Update a package to a new version on-chain
    Update {
        /// Path to package directory
        #[arg(default_value = ".")]
        path: PathBuf,
        
        /// New version
        #[arg(short, long)]
        version: String,
    },
}

#[derive(Subcommand)]
enum WalletAction {
    /// Connect a wallet for signing transactions
    Connect {
        /// Path to wallet keypair JSON file
        keypair: PathBuf,
    },
    
    /// Show current wallet and network info
    Show,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    // Print banner
    print_banner();
    
    let result = match cli.command {
        Commands::Setup => setup::handle_setup().await,
        Commands::Init => init::handle_init().await,
        Commands::Publish { path, version } => publish::handle_publish(path, version).await,
        Commands::Install { package } => install::handle_install(package).await,
        Commands::Search { query } => search::handle_search(query).await,
        Commands::Info { package } => info::handle_info(package).await,
        Commands::Wallet { action } => match action {
            WalletAction::Connect { keypair } => wallet::handle_connect(keypair).await,
            WalletAction::Show => wallet::handle_show().await,
        },
        Commands::Update { path, version } => update::handle_update(path, version).await,
    };
    
    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("\n{} {}", "✗".red().bold(), e.to_string().red());
            std::process::exit(1);
        }
    }
}

fn print_banner() {
    println!("{}", r#"
    ╔══════════════════════════════════════════════╗
    ║   🐜 AntSol - Decentralized Registry v1.0.0  ║
    ║   On-Chain Package Management on Solana      ║
    ║   Immutable Storage via IPFS                 ║
    ╚══════════════════════════════════════════════╝
    "#.cyan());
}
