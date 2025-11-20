use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;
use crate::types::{Dependency, ExternalDependency, PackageAccount, Result};
use crate::config::Config;

pub struct AntSolClient {
    rpc_client: RpcClient,
    program_id: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct PackageDep {
    pub name: String,
    pub version: String,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct ExternalDep {
    pub name: String,
    pub version: String,
    pub dep_type: String,
    pub registry: Option<String>,
}

impl AntSolClient {
    pub fn new(config: &Config) -> Result<Self> {
        let rpc_client = RpcClient::new(config.rpc_url.clone());
        let program_id = Pubkey::from_str(&config.program_id)?;
        
        Ok(Self {
            rpc_client,
            program_id,
        })
    }
    
    pub fn derive_package_pda(&self, name: &str, version: &str) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"package",
                name.as_bytes(),
                version.as_bytes(),
            ],
            &self.program_id,
        )
    }
    
    pub async fn publish_package(
        &self,
        payer: &Keypair,
        name: String,
        version: String,
        ipfs_cid: String,
        description: String,
        dependencies: Vec<Dependency>,
        external_dependencies: Vec<ExternalDependency>,
    ) -> Result<String> {
        let (package_pda, _bump) = self.derive_package_pda(&name, &version);
        
        let deps: Vec<PackageDep> = dependencies
            .into_iter()
            .map(|d| PackageDep {
                name: d.name,
                version: d.version,
            })
            .collect();
        
        let ext_deps: Vec<ExternalDep> = external_dependencies
            .into_iter()
            .map(|d| ExternalDep {
                name: d.name,
                version: d.version,
                dep_type: d.dep_type,
                registry: d.registry,
            })
            .collect();
        
        let discriminator: [u8; 8] = [244, 240, 208, 233, 198, 38, 46, 197];
        let args_data = (name, version, ipfs_cid, description, deps, ext_deps).try_to_vec()?;
        
        let mut data = discriminator.to_vec();
        data.extend_from_slice(&args_data);
        
        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(package_pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };
        
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[payer],
            recent_blockhash,
        );

        match self.rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(sig) => Ok(sig.to_string()),
            Err(send_err) => {
                match self.rpc_client.simulate_transaction(&transaction) {
                    Ok(sim_result) => {
                        if let Some(logs) = sim_result.value.logs {
                            let joined = logs.join("\n");
                            return Err(format!("RPC send error: {}\nSimulation logs:\n{}", send_err, joined).into());
                        }
                    }
                    Err(_) => {}
                }
                Err(send_err.into())
            }
        }
    }
    
    pub async fn update_package(
        &self,
        payer: &Keypair,
        name: String,
        old_version: String,
        new_version: String,
        ipfs_cid: String,
        description: String,
        dependencies: Vec<Dependency>,
        external_dependencies: Vec<ExternalDependency>,
    ) -> Result<String> {
        let (existing_pda, _) = self.derive_package_pda(&name, &old_version);
        let (new_pda, _) = self.derive_package_pda(&name, &new_version);
        
        let deps: Vec<PackageDep> = dependencies
            .into_iter()
            .map(|d| PackageDep {
                name: d.name,
                version: d.version,
            })
            .collect();
        
        let ext_deps: Vec<ExternalDep> = external_dependencies
            .into_iter()
            .map(|d| ExternalDep {
                name: d.name,
                version: d.version,
                dep_type: d.dep_type,
                registry: d.registry,
            })
            .collect();
        
        let discriminator: [u8; 8] = [167, 29, 15, 20, 179, 137, 50, 145];
        let args_data = (name, new_version, ipfs_cid, description, deps, ext_deps).try_to_vec()?;
        
        let mut data = discriminator.to_vec();
        data.extend_from_slice(&args_data);
        
        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(existing_pda, false),
                AccountMeta::new(new_pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };
        
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[payer],
            recent_blockhash,
        );

        match self.rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(sig) => Ok(sig.to_string()),
            Err(send_err) => {
                match self.rpc_client.simulate_transaction(&transaction) {
                    Ok(sim_result) => {
                        if let Some(logs) = sim_result.value.logs {
                            let joined = logs.join("\n");
                            return Err(format!("RPC send error: {}\nSimulation logs:\n{}", send_err, joined).into());
                        }
                    }
                    Err(_) => {}
                }
                Err(send_err.into())
            }
        }
    }
    
    pub fn get_package(&self, name: &str, version: &str) -> Result<Option<PackageAccount>> {
        let (pda, _) = self.derive_package_pda(name, version);
        
        match self.rpc_client.get_account(&pda) {
            Ok(account) => {
                if account.owner != self.program_id {
                    return Ok(None);
                }
                
                if account.data.len() < 8 {
                    return Ok(None);
                }
                
                eprintln!("DEBUG: Account data length: {} bytes", account.data.len());
                eprintln!("DEBUG: First 16 bytes (discriminator + start): {:?}", &account.data[..16.min(account.data.len())]);
                
                let data = &account.data[8..];
                eprintln!("DEBUG: Deserializing {} bytes after discriminator", data.len());
                
                Ok(Some(self.deserialize_package_account(data)?))
            }
            Err(_) => Ok(None),
        }
    }
    
    fn deserialize_package_account(&self, data: &[u8]) -> Result<PackageAccount> {
        use borsh::BorshDeserialize;
        
        // Try new format first (with external_dependencies)
        #[derive(BorshDeserialize, Debug)]
        struct AnchorPackageNew {
            name: String,
            version: String,
            authority: Pubkey,
            ipfs_cid: String,
            published_at: i64,
            description: String,
            dependencies: Vec<PackageDep>,
            external_dependencies: Vec<ExternalDep>,
            _bump: u8,
        }
        
        // Fallback to old format (without external_dependencies)
        #[derive(BorshDeserialize, Debug)]
        struct AnchorPackageOld {
            name: String,
            version: String,
            authority: Pubkey,
            ipfs_cid: String,
            published_at: i64,
            description: String,
            dependencies: Vec<PackageDep>,
            _bump: u8,
        }
        
        // Try new format first
        let mut data_slice = data;
        let result = AnchorPackageNew::deserialize(&mut data_slice);
        
        match result {
            Ok(anchor_pkg) => {
                eprintln!("DEBUG: Successfully deserialized NEW format! Remaining bytes: {}", data_slice.len());
                
                let dependencies = anchor_pkg.dependencies
                    .into_iter()
                    .map(|d| Dependency {
                        name: d.name,
                        version: d.version,
                    })
                    .collect();
                
                let external_dependencies = anchor_pkg.external_dependencies
                    .into_iter()
                    .map(|d| ExternalDependency {
                        name: d.name,
                        version: d.version,
                        dep_type: d.dep_type,
                        registry: d.registry,
                    })
                    .collect();
                
                Ok(PackageAccount {
                    name: anchor_pkg.name,
                    version: anchor_pkg.version,
                    authority: anchor_pkg.authority,
                    ipfs_cid: anchor_pkg.ipfs_cid,
                    published_at: anchor_pkg.published_at,
                    description: anchor_pkg.description,
                    dependencies,
                    external_dependencies,
                })
            }
            Err(_) => {
                // Try old format
                eprintln!("DEBUG: New format failed, trying OLD format...");
                let mut data_slice = data;
                let anchor_pkg: AnchorPackageOld = BorshDeserialize::deserialize(&mut data_slice)
                    .map_err(|e| format!("Borsh deserialization error (old format): {}. Remaining bytes: {}", e, data_slice.len()))?;
                
                eprintln!("DEBUG: Successfully deserialized OLD format! Remaining bytes: {}", data_slice.len());
                
                let dependencies = anchor_pkg.dependencies
                    .into_iter()
                    .map(|d| Dependency {
                        name: d.name,
                        version: d.version,
                    })
                    .collect();
                
                Ok(PackageAccount {
                    name: anchor_pkg.name,
                    version: anchor_pkg.version,
                    authority: anchor_pkg.authority,
                    ipfs_cid: anchor_pkg.ipfs_cid,
                    published_at: anchor_pkg.published_at,
                    description: anchor_pkg.description,
                    dependencies,
                    external_dependencies: vec![], // Old format has no external deps
                })
            }
        }
    }
}
