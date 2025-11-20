use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;
use state::*;

declare_id!("A9igkBugcujD9Nw9d97FFN4aY3qHXnJxEqCChJt8C42S");

#[program]
pub mod antsol_registry {
    use super::*;

    /// Publish a new package to the registry
    pub fn publish_package(
        ctx: Context<PublishPackage>,
        name: String,
        version: String,
        ipfs_cid: String,
        description: String,
        dependencies: Vec<PackageDependency>,
    ) -> Result<()> {
        instructions::publish_package::handler(
            ctx,
            name,
            version,
            ipfs_cid,
            description,
            dependencies,
        )
    }

    /// Update an existing package with a new version
    pub fn update_package(
        ctx: Context<UpdatePackage>,
        name: String,
        new_version: String,
        ipfs_cid: String,
        description: String,
        dependencies: Vec<PackageDependency>,
    ) -> Result<()> {
        instructions::update_package::handler(
            ctx,
            name,
            new_version,
            ipfs_cid,
            description,
            dependencies,
        )
    }

    /// Transfer package ownership to a new authority
    pub fn transfer_authority(
        ctx: Context<TransferAuthority>,
        _name: String,
        _version: String,
    ) -> Result<()> {
        instructions::transfer_authority::handler(ctx)
    }
}
