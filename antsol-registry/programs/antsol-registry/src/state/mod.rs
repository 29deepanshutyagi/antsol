use anchor_lang::prelude::*;

/// Maximum length for package name
pub const MAX_NAME_LENGTH: usize = 64;
/// Maximum length for version string (semver)
pub const MAX_VERSION_LENGTH: usize = 16;
/// Maximum length for IPFS CID
pub const MAX_CID_LENGTH: usize = 64;
/// Maximum length for description
pub const MAX_DESCRIPTION_LENGTH: usize = 256;
/// Maximum number of dependencies
pub const MAX_DEPENDENCIES: usize = 10;

/// Package account structure stored on-chain
/// Each version of a package gets its own account
#[account]
pub struct Package {
	/// Package name (e.g., "spl-token-utils")
	pub name: String,
	/// Semantic version (e.g., "1.0.0")
	pub version: String,
	/// Authority who can publish updates
	pub authority: Pubkey,
	/// IPFS Content Identifier for package files
	pub ipfs_cid: String,
	/// Unix timestamp when published
	pub published_at: i64,
	/// Short description of package
	pub description: String,
	/// List of required dependencies
	pub dependencies: Vec<PackageDependency>,
	/// PDA bump seed
	pub bump: u8,
}

impl Package {
	/// Calculate space needed for account
	pub const fn space(
		name_len: usize,
		version_len: usize,
		description_len: usize,
		deps_count: usize,
	) -> usize {
		8 + // discriminator
		4 + name_len + // String prefix + data
		4 + version_len +
		32 + // Pubkey
		4 + MAX_CID_LENGTH +
		8 + // i64
		4 + description_len +
		4 + (deps_count * PackageDependency::LEN) + // Vec prefix + data
		1 // bump
	}

	/// Maximum possible space for a package account
	pub const MAX_SPACE: usize = Self::space(
		MAX_NAME_LENGTH,
		MAX_VERSION_LENGTH,
		MAX_DESCRIPTION_LENGTH,
		MAX_DEPENDENCIES,
	);
}

/// Dependency structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct PackageDependency {
	/// Dependency package name
	pub name: String,
	/// Required version (exact match for POC)
	pub version: String,
}

impl PackageDependency {
	/// Fixed length for dependency (max sizes)
	pub const LEN: usize = 4 + MAX_NAME_LENGTH + 4 + MAX_VERSION_LENGTH;
}
