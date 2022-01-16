use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

declare_id!("4SgBV6KvC6TvRMPQqwcuNzfNDYcXKCo5TR5T3PFxBau5");

#[program]
pub mod anchor_token_staking {
    use super::*;

    pub fn initialize_vault(_ctx: Context<InitializeVault>, _bump: u8) -> ProgramResult {
        Ok(())
    }

    pub fn initialize_stake_account(ctx: Context<InitializeStakeAccount>, _bump: u8) -> ProgramResult {
        // Verify the stake_account address is the correct PDA
        let (pda, _) = Pubkey::find_program_address(&[b"stake-account", ctx.accounts.mint.key().as_ref(), ctx.accounts.stake_authority.key().as_ref()], &id());
        if pda != ctx.accounts.stake_account.key() {
            return Err(ErrorCode::InvalidStakeAccountPda.into())
        }

        ctx.accounts.stake_account.authority = ctx.accounts.stake_authority.key();
        ctx.accounts.stake_account.amount = 0;

        Ok(())
    }

    pub fn stake_tokens(_ctx: Context<StakeTokens>) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeVault<'info> {
    #[account(init_if_needed,
        payer = payer,
        seeds = [b"vault", mint.key().as_ref()],
        bump = bump,
        token::mint = mint,
        token::authority = vault_account)]
    pub vault_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeStakeAccount<'info> {
    #[account(init_if_needed, 
        payer = stake_authority, 
        seeds = [b"stake-account", mint.key().as_ref(), stake_authority.key().as_ref()],
        bump = bump,
        space = 8 + 32 + 8)]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    pub stake_authority: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub vault_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = stake_account.authority == staker.key())]
    pub stake_account: Account<'info, StakeAccount>,
    pub staker: Signer<'info>,
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct StakeAccount {
    authority: Pubkey,  // 32
    amount: u64,        // 8
}

#[error]
pub enum ErrorCode {
    #[msg("Error: Invalid StakeAccount PDA")]
    InvalidStakeAccountPda,
}
