use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::RegistryError;

/// Publish a new package to the registry
#[derive(Accounts)]
#[instruction(name: String, version: String)]
pub struct PublishPackage<'info> {
	#[account(mut)]
	pub authority: Signer<'info>,
	#[account(
		init,
		payer = authority,
		space = Package::MAX_SPACE,
		seeds = [b"package", name.as_bytes(), version.as_bytes()],
		bump
	)]
	pub package: Account<'info, Package>,
	pub system_program: Program<'info, System>,
}

pub fn handler(
	ctx: Context<PublishPackage>,
	name: String,
	version: String,
	ipfs_cid: String,
	description: String,
	dependencies: Vec<PackageDependency>,
) -> Result<()> {
	require!(!name.is_empty(), RegistryError::NameEmpty);
	require!(name.len() <= MAX_NAME_LENGTH, RegistryError::NameTooLong);
	require!(is_valid_package_name(&name), RegistryError::InvalidNameFormat);

	require!(!version.is_empty(), RegistryError::VersionEmpty);
	require!(version.len() <= MAX_VERSION_LENGTH, RegistryError::VersionTooLong);
	require!(is_valid_semver(&version), RegistryError::InvalidVersionFormat);

	require!(!ipfs_cid.is_empty(), RegistryError::CidEmpty);
	require!(ipfs_cid.len() <= MAX_CID_LENGTH, RegistryError::CidTooLong);
	require!(is_valid_ipfs_cid(&ipfs_cid), RegistryError::InvalidCidFormat);

	require!(description.len() <= MAX_DESCRIPTION_LENGTH, RegistryError::DescriptionTooLong);

	require!(dependencies.len() <= MAX_DEPENDENCIES, RegistryError::TooManyDependencies);
	for dep in &dependencies {
		require!(!dep.name.is_empty(), RegistryError::InvalidDependencyName);
		require!(dep.name.len() <= MAX_NAME_LENGTH, RegistryError::InvalidDependencyName);
		require!(is_valid_package_name(&dep.name), RegistryError::InvalidDependencyName);
		require!(!dep.version.is_empty(), RegistryError::InvalidDependencyVersion);
		require!(dep.version.len() <= MAX_VERSION_LENGTH, RegistryError::InvalidDependencyVersion);
		require!(is_valid_semver(&dep.version), RegistryError::InvalidDependencyVersion);
	}

	let clock = Clock::get()?;
	let current_timestamp = clock.unix_timestamp;

	let package = &mut ctx.accounts.package;
	package.name = name;
	package.version = version;
	package.authority = ctx.accounts.authority.key();
	package.ipfs_cid = ipfs_cid.clone();
	package.published_at = current_timestamp;
	package.description = description;
	package.dependencies = dependencies;
	package.bump = ctx.bumps.package;

	emit!(PackagePublished {
		name: package.name.clone(),
		version: package.version.clone(),
		authority: package.authority,
		ipfs_cid: ipfs_cid,
		timestamp: current_timestamp,
	});

	msg!("📦 Package published: {}@{}", package.name, package.version);
	Ok(())
}

fn is_valid_package_name(name: &str) -> bool {
	if name.is_empty() || name.len() > MAX_NAME_LENGTH { return false; }
	if name.starts_with('-') || name.ends_with('-') { return false; }
	name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
fn is_valid_semver(version: &str) -> bool {
	let parts: Vec<&str> = version.split('.').collect();
	if parts.len() != 3 { return false; }
	parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}
fn is_valid_ipfs_cid(cid: &str) -> bool {
	if cid.is_empty() || cid.len() > MAX_CID_LENGTH { return false; }
	cid.starts_with("Qm") || cid.starts_with("bafy")
}

#[event]
pub struct PackagePublished {
	pub name: String,
	pub version: String,
	pub authority: Pubkey,
	pub ipfs_cid: String,
	pub timestamp: i64,
}
