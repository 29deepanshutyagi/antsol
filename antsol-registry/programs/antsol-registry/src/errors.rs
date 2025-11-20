use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("Package name is too long (max 64 characters)")] 
    NameTooLong,
    #[msg("Package name is empty")] 
    NameEmpty,
    #[msg("Package name contains invalid characters (use lowercase alphanumeric and hyphens only)")] 
    InvalidNameFormat,
    #[msg("Version is too long (max 16 characters)")] 
    VersionTooLong,
    #[msg("Version is empty")] 
    VersionEmpty,
    #[msg("Version format is invalid (use semantic versioning: X.Y.Z)")] 
    InvalidVersionFormat,
    #[msg("IPFS CID is too long (max 64 characters)")] 
    CidTooLong,
    #[msg("IPFS CID is empty")] 
    CidEmpty,
    #[msg("IPFS CID format is invalid (must start with 'Qm' or 'bafy')")] 
    InvalidCidFormat,
    #[msg("Description is too long (max 256 characters)")] 
    DescriptionTooLong,
    #[msg("Too many dependencies (max 10)")] 
    TooManyDependencies,
    #[msg("Dependency name is invalid")] 
    InvalidDependencyName,
    #[msg("Dependency version is invalid")] 
    InvalidDependencyVersion,
    #[msg("New version must be greater than existing versions")] 
    VersionNotGreater,
    #[msg("IPFS CID must be different from previous versions")] 
    SameCidAsExisting,
    #[msg("Package name already exists with different authority")] 
    UnauthorizedPackageName,
    #[msg("Only the package authority can perform this action")] 
    UnauthorizedAuthority,
    #[msg("Arithmetic overflow occurred")] 
    ArithmeticOverflow,
}
