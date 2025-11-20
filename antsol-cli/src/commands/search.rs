use crate::types::Result;
use crate::utils::*;
use crate::config::Config;
use colored::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PackageRow {
    id: i64,
    name: String,
    author: Option<String>,
    description: Option<String>,
    repository: Option<String>,
    homepage: Option<String>,
    total_downloads: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct VersionRow {
    id: i64,
    package_id: i64,
    version: String,
    ipfs_hash: Option<String>,
    downloads: Option<u64>,
    published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PackageDetails {
    id: i64,
    name: String,
    author: Option<String>,
    description: Option<String>,
    repository: Option<String>,
    homepage: Option<String>,
    total_downloads: Option<u64>,
    versions: Vec<VersionRow>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
    error: Option<String>,
}

/// Search for packages in the decentralized registry
pub async fn handle_search(query: String) -> Result<()> {
    println!("\n{}", "🔍 Searching Decentralized Registry".cyan().bold());
    let spinner = create_spinner(&format!("Searching for '{}'...", query));
    
    let config = Config::load()?;
    let base = config.indexer_url.trim_end_matches('/');
    // Use the dedicated search endpoint: /api/search?q=<query>
    let list_url = format!("{}/api/search", base);
    
    let client = reqwest::Client::new();
    let list_resp = client
        .get(&list_url)
        .query(&[("q", query.clone())])
        .send()
        .await;
    
    match list_resp {
        Ok(resp) if resp.status().is_success() => {
            let api: ApiResponse<Vec<PackageRow>> = resp.json().await?;
            let rows = api.data;
            if rows.is_empty() {
                spinner.finish_and_clear();
                print_warning(&format!("No packages found matching '{}'", query));
                println!("\n{}", "💡 Tips:".yellow());
                println!("  • Check your spelling");
                println!("  • Try broader search terms");
                println!("  • Use 'antsol info <package>' if you know the exact name");
                return Ok(());
            }
            
            // For each package, fetch details to get versions
            let mut results: Vec<(PackageRow, Option<PackageDetails>)> = Vec::with_capacity(rows.len());
            for row in rows {
                let details_url = format!("{}/api/packages/{}", base, row.name);
                let details_resp = client.get(&details_url).send().await;
                let details: Option<PackageDetails> = match details_resp {
                    Ok(r) if r.status().is_success() => {
                        match r.json::<ApiResponse<PackageDetails>>().await {
                            Ok(api_ok) => Some(api_ok.data),
                            Err(_) => None,
                        }
                    }
                    _ => None,
                };
                results.push((row, details));
            }
            
            spinner.finish_and_clear();
            println!("\n{} {}", "📦 Found".cyan().bold(), format!("{} packages", results.len()).green());
            println!("{}", "─".repeat(80));
            
            for (row, details_opt) in results {
                let latest = details_opt
                    .as_ref()
                    .and_then(|d| d.versions.iter().map(|v| v.version.clone()).max())
                    .unwrap_or_else(|| "unknown".to_string());
                println!("\n{} {}", "📦".cyan(), row.name.green().bold());
                println!("  Version: {}", latest.cyan());
                let desc = row.description.unwrap_or_else(|| "No description".into());
                println!("  Description: {}", desc);
                if let Some(dls) = row.total_downloads { println!("  Downloads: {}", dls.to_string().yellow()); }
            }
            
            println!("\n{}", "─".repeat(80));
            println!("\n{}", "💡 To install:".yellow());
            println!("  {}", "antsol install <package-name>@<version>".cyan());
            println!("\n{}", "💡 To view details:".yellow());
            println!("  {}", "antsol info <package-name>".cyan());
        }
        _ => {
            spinner.finish_and_clear();
            print_warning("⚠️  Indexer service not available");
            println!("\n{}", "ℹ️  About the Indexer:".blue().bold());
            println!("  The indexer reads package data from the Solana blockchain");
            println!("  and provides fast search functionality.");
            println!("\n{}", "💡 Alternative:".yellow());
            println!("  • If you know the exact package name, use:");
            println!("    {}", "antsol info <package-name>@<version>".cyan());
            println!("  • This queries the blockchain directly");
        }
    }
    
    Ok(())
}
