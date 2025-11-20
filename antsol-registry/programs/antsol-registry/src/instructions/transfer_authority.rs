use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::RegistryError;

/// Transfer package ownership to a new authority
#[derive(Accounts)]
#[instruction(name: String, version: String)]
pub struct TransferAuthority<'info> {
	#[account(mut)]
	pub current_authority: Signer<'info>,
	#[account(
		mut,
		seeds = [b"package", name.as_bytes(), version.as_bytes()],
		bump = package.bump,
		constraint = package.authority == current_authority.key() @ RegistryError::UnauthorizedAuthority
	)]
	pub package: Account<'info, Package>,
	/// CHECK: New authority (doesn't need to sign)
	pub new_authority: AccountInfo<'info>,
}

pub fn handler(ctx: Context<TransferAuthority>) -> Result<()> {
	let package = &mut ctx.accounts.package;
	let old_authority = package.authority;
	let new_authority = ctx.accounts.new_authority.key();
	package.authority = new_authority;
	emit!(AuthorityTransferred {
		name: package.name.clone(),
		version: package.version.clone(),
		old_authority,
		new_authority,
		timestamp: Clock::get()?.unix_timestamp,
	});
	msg!("🔑 Authority transferred: {} -> {}", old_authority, new_authority);
	Ok(())
}

#[event]
pub struct AuthorityTransferred {
	pub name: String,
	pub version: String,
	pub old_authority: Pubkey,
	pub new_authority: Pubkey,
	pub timestamp: i64,
}
