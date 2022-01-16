use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};

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

    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> ProgramResult {
        // verify the vault is our vault PDA of the tokens mint
        let mint = ctx.accounts.staker_token_account.mint;
        let (pda, _) = Pubkey::find_program_address(&[b"vault", mint.as_ref()], &id());

        if pda != ctx.accounts.vault_account.key() {
            return Err(ErrorCode::InvalidVaultPda.into())
        }

        // Verify the vault access address is the correct PDA
        let (pda, _) = Pubkey::find_program_address(&[b"stake-account", mint.as_ref(), ctx.accounts.staker.key().as_ref()], &id());

        if pda != ctx.accounts.stake_account.key() {
            return Err(ErrorCode::InvalidStakeAccountPda.into())
        }

        token::transfer((&*ctx.accounts).into(), amount)?;

        ctx.accounts.stake_account.amount += amount;

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

impl<'info> From<&StakeTokens<'info>> for CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    fn from(accounts: &StakeTokens<'info>) -> Self {
        let cpi_program = accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: accounts.staker_token_account.to_account_info(),
            to: accounts.vault_account.to_account_info(),
            authority: accounts.staker.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[account]
pub struct StakeAccount {
    authority: Pubkey,  // 32
    amount: u64,        // 8
}

#[error]
pub enum ErrorCode {
    #[msg("Error: Invalid vault PDA")]
    InvalidVaultPda,
    #[msg("Error: Invalid StakeAccount PDA")]
    InvalidStakeAccountPda,
}
