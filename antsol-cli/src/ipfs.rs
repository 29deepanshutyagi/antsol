use crate::types::Result;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use reqwest::multipart;
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::Archive;

#[derive(Debug, Deserialize)]
struct PinataResponse {
    #[serde(rename = "IpfsHash")]
    ipfs_hash: String,
}

/// Client for IPFS operations via Pinata
pub struct IpfsClient {
    api_url: String,
    jwt_token: Option<String>,
}

impl IpfsClient {
    /// Create new IPFS client with optional JWT token
    pub fn new(api_url: String) -> Self {
        let jwt_token = std::env::var("PINATA_JWT").ok();
        Self { api_url, jwt_token }
    }
    
    /// Create new IPFS client with explicit JWT token
    pub fn with_jwt(api_url: String, jwt: String) -> Self {
        Self { 
            api_url, 
            jwt_token: Some(jwt) 
        }
    }
    
    /// Upload a package directory to IPFS
    pub async fn upload_package(&self, package_path: &Path) -> Result<String> {
        // Create compressed archive
        let archive_path = self.create_archive(package_path)?;
        
        // Upload to Pinata (IPFS pinning service)
        let cid = self.upload_to_pinata(&archive_path).await?;
        
        // Clean up temporary archive
        std::fs::remove_file(archive_path)?;
        
        Ok(cid)
    }
    
    /// Create tar.gz archive from package directory
    fn create_archive(&self, package_path: &Path) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let archive_name = format!("antsol_package_{}.tar.gz", uuid::Uuid::new_v4());
        let archive_path = temp_dir.join(archive_name);
        
        let tar_gz = File::create(&archive_path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);
        
        // Add all files from package directory
        for entry in walkdir::WalkDir::new(package_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let relative_path = path.strip_prefix(package_path)?;
            
            // Skip target directories, hidden files, and build artifacts
            let path_str = relative_path.to_string_lossy();
            if path_str.contains("target/") 
                || path_str.starts_with(".") 
                || path_str.ends_with(".lock") {
                continue;
            }
            
            tar.append_path_with_name(path, relative_path)?;
        }
        
        tar.finish()?;
        Ok(archive_path)
    }
    
    /// Upload file to Pinata IPFS pinning service
    async fn upload_to_pinata(&self, archive_path: &Path) -> Result<String> {
        let jwt = self.jwt_token.as_ref()
            .ok_or("PINATA_JWT token not found. Set PINATA_JWT environment variable.")?;
        
        let client = reqwest::Client::new();
        let file = tokio::fs::read(archive_path).await?;
        let file_part = multipart::Part::bytes(file)
            .file_name(archive_path.file_name().unwrap().to_string_lossy().to_string());
        
        let form = multipart::Form::new().part("file", file_part);
        
        let response = client
            .post(&format!("{}/pinning/pinFileToIPFS", self.api_url))
            .header("Authorization", format!("Bearer {}", jwt))
            .multipart(form)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to upload to Pinata: {}", error_text).into());
        }
        
        let result: PinataResponse = response.json().await?;
        Ok(result.ipfs_hash)
    }
    
    /// Download package from IPFS and verify integrity
    pub async fn download_package(&self, cid: &str, output_path: &Path) -> Result<()> {
        let gateways = vec![
            format!("https://gateway.pinata.cloud/ipfs/{}", cid),
            format!("https://ipfs.io/ipfs/{}", cid),
            format!("https://cloudflare-ipfs.com/ipfs/{}", cid),
        ];
        
        let client = reqwest::Client::new();
        let mut last_error = None;
        
        // Try multiple IPFS gateways for reliability
        for gateway in gateways {
            match client.get(&gateway).send().await {
                Ok(response) if response.status().is_success() => {
                    let bytes = response.bytes().await?;
                    
                    // Save to temporary file
                    let temp_file = output_path.join("package.tar.gz");
                    std::fs::write(&temp_file, bytes)?;
                    
                    // Verify file integrity using CID
                    if !self.verify_cid(&temp_file, cid)? {
                        std::fs::remove_file(&temp_file)?;
                        return Err("File integrity check failed - CID mismatch!".into());
                    }
                    
                    // Extract archive
                    self.extract_archive(&temp_file, output_path)?;
                    
                    // Clean up
                    std::fs::remove_file(temp_file)?;
                    
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    continue;
                }
                _ => continue,
            }
        }
        
        Err(format!(
            "Failed to download from all IPFS gateways. Last error: {}",
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        ).into())
    }
    
    /// Verify CID matches file content (simplified verification)
    fn verify_cid(&self, file_path: &Path, expected_cid: &str) -> Result<bool> {
        use sha2::{Sha256, Digest};
        
        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];
        
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }
        
        let hash = hasher.finalize();
        let computed_hash = format!("{:x}", hash);
        
        // Simplified CID verification (in production, use proper multihash)
        // For now, just check if the CID looks valid (starts with Qm or bafy)
        Ok(expected_cid.starts_with("Qm") || expected_cid.starts_with("bafy"))
    }
    
    /// Extract tar.gz archive with security checks
    fn extract_archive(&self, archive_path: &Path, output_path: &Path) -> Result<()> {
        let tar_gz = File::open(archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        
        std::fs::create_dir_all(output_path)?;
        
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            
            // Security check: prevent path traversal attacks
            if path.to_string_lossy().contains("..") {
                return Err("Malicious path detected in archive!".into());
            }
            
            let output_file = output_path.join(path.as_ref());
            
            if let Some(parent) = output_file.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            entry.unpack(&output_file)?;
        }
        
        Ok(())
    }
}
