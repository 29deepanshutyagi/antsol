use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::RegistryError;

/// Update an existing package with a new version
#[derive(Accounts)]
#[instruction(name: String, new_version: String)]
pub struct UpdatePackage<'info> {
	#[account(mut)]
	pub authority: Signer<'info>,

	#[account(
		seeds = [b"package", existing_package.name.as_bytes(), existing_package.version.as_bytes()],
		bump = existing_package.bump,
		constraint = existing_package.authority == authority.key() @ RegistryError::UnauthorizedAuthority
	)]
	pub existing_package: Account<'info, Package>,

	#[account(
		init,
		payer = authority,
		space = Package::MAX_SPACE,
		seeds = [b"package", name.as_bytes(), new_version.as_bytes()],
		bump
	)]
	pub new_package: Account<'info, Package>,

	pub system_program: Program<'info, System>,
}

pub fn handler(
	ctx: Context<UpdatePackage>,
	name: String,
	new_version: String,
	ipfs_cid: String,
	description: String,
	dependencies: Vec<PackageDependency>,
) -> Result<()> {
	let existing = &ctx.accounts.existing_package;

	require!(name == existing.name, RegistryError::UnauthorizedPackageName);

	require!(!new_version.is_empty(), RegistryError::VersionEmpty);
	require!(new_version.len() <= MAX_VERSION_LENGTH, RegistryError::VersionTooLong);
	require!(is_valid_semver(&new_version), RegistryError::InvalidVersionFormat);
	require!(is_version_greater(&new_version, &existing.version), RegistryError::VersionNotGreater);

	require!(!ipfs_cid.is_empty(), RegistryError::CidEmpty);
	require!(ipfs_cid.len() <= MAX_CID_LENGTH, RegistryError::CidTooLong);
	require!(is_valid_ipfs_cid(&ipfs_cid), RegistryError::InvalidCidFormat);
	require!(ipfs_cid != existing.ipfs_cid, RegistryError::SameCidAsExisting);

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

	let new_package = &mut ctx.accounts.new_package;
	new_package.name = name.clone();
	new_package.version = new_version.clone();
	new_package.authority = existing.authority;
	new_package.ipfs_cid = ipfs_cid.clone();
	new_package.published_at = current_timestamp;
	new_package.description = description;
	new_package.dependencies = dependencies;
	new_package.bump = ctx.bumps.new_package;

	emit!(PackageUpdated {
		name,
		old_version: existing.version.clone(),
		new_version,
		authority: existing.authority,
		timestamp: current_timestamp,
	});

	msg!("🔄 Package updated: {}@{}", new_package.name, new_package.version);
	Ok(())
}

fn is_version_greater(version1: &str, version2: &str) -> bool {
	let v1_parts: Vec<u32> = version1.split('.').filter_map(|s| s.parse().ok()).collect();
	let v2_parts: Vec<u32> = version2.split('.').filter_map(|s| s.parse().ok()).collect();
	if v1_parts.len() != 3 || v2_parts.len() != 3 { return false; }
	for i in 0..3 {
		if v1_parts[i] > v2_parts[i] { return true; }
		else if v1_parts[i] < v2_parts[i] { return false; }
	}
	false
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
pub struct PackageUpdated {
	pub name: String,
	pub old_version: String,
	pub new_version: String,
	pub authority: Pubkey,
	pub timestamp: i64,
}
