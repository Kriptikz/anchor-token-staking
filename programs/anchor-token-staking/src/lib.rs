use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};

declare_id!("4SgBV6KvC6TvRMPQqwcuNzfNDYcXKCo5TR5T3PFxBau5");

const REWARDS_RATE_PER_SECOND_PER_STAKE: u64 = 1;

#[program]
pub mod anchor_token_staking {
    use super::*;

    pub fn initialize_stake_vault(_ctx: Context<InitializeStakeVault>, _bump: u8) -> ProgramResult {
        Ok(())
    }

    pub fn initialize_reward_vault(_ctx: Context<InitializeRewardVault>, _bump: u8) -> ProgramResult {
        Ok(())
    }

    pub fn initialize_stake_account(ctx: Context<InitializeStakeAccount>, _bump: u8) -> ProgramResult {
        // Verify the stake_account address is the correct PDA
        let (pda, _) = Pubkey::find_program_address(&[b"stake-account", ctx.accounts.mint.key().as_ref(), ctx.accounts.stake_authority.key().as_ref()], &id());
        if pda != ctx.accounts.stake_account.key() {
            return Err(ErrorCode::InvalidStakeAccountPda.into())
        }

        ctx.accounts.stake_account.authority = ctx.accounts.stake_authority.key();
        ctx.accounts.stake_account.staked_amount = 0;
        ctx.accounts.stake_account.stake_start_time = 0;
        ctx.accounts.stake_account.unclaimed_amount = 0;


        Ok(())
    }

    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> ProgramResult {
        // verify the vault is our vault PDA of the tokens mint
        let mint = ctx.accounts.staker_token_account.mint;
        let (pda, _) = Pubkey::find_program_address(&[b"stake-vault", mint.as_ref()], &id());

        if pda != ctx.accounts.stake_vault.key() {
            return Err(ErrorCode::InvalidVaultPda.into())
        }

        // Verify the vault access address is the correct PDA
        let (pda, _) = Pubkey::find_program_address(&[b"stake-account", mint.as_ref(), ctx.accounts.staker.key().as_ref()], &id());

        if pda != ctx.accounts.stake_account.key() {
            return Err(ErrorCode::InvalidStakeAccountPda.into())
        }

        token::transfer((&*ctx.accounts).into(), amount)?;

        let clock = anchor_lang::solana_program::clock::Clock::get()?;
        let time = clock.unix_timestamp;

        ctx.accounts.stake_account.update_unclaimed_amount(time);

        ctx.accounts.stake_account.stake_start_time = time;

        ctx.accounts.stake_account.staked_amount += amount;

        Ok(())
    }

    pub fn unstake_tokens(ctx: Context<UnstakeTokens>, amount: u64) -> ProgramResult {
        // verify the vault is our vault PDA of the tokens mint
        let mint = ctx.accounts.to.mint;
        let (pda, bump) = Pubkey::find_program_address(&[b"stake-vault", mint.as_ref()], &id());

        if pda != ctx.accounts.stake_vault.key() {
            return Err(ErrorCode::InvalidVaultPda.into())
        }

        // Verify the vault access address is the correct PDA
        let (pda, _) = Pubkey::find_program_address(&[b"stake-account", mint.as_ref(), ctx.accounts.authority.key().as_ref()], &id());

        if pda != ctx.accounts.stake_account.key() {
            return Err(ErrorCode::InvalidStakeAccountPda.into())
        }


        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.stake_vault.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.stake_vault.to_account_info(),
        };

        if amount > ctx.accounts.stake_account.staked_amount {
            return Err(ErrorCode::InsufficientFundsStaked.into())
        }

        token::transfer(
            CpiContext::new_with_signer(
                cpi_program, 
                cpi_accounts,
                &[&[b"stake-vault", mint.as_ref(), &[bump]]]), 
            amount)?;
        
        let clock = anchor_lang::solana_program::clock::Clock::get()?;
        let time = clock.unix_timestamp;

        ctx.accounts.stake_account.update_unclaimed_amount(time);

        if ctx.accounts.stake_account.staked_amount > 0 {
            ctx.accounts.stake_account.stake_start_time = time;
        } else {
            ctx.accounts.stake_account.stake_start_time = 0;
        }

        ctx.accounts.stake_account.staked_amount -= amount;
        

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeStakeVault<'info> {
    #[account(init_if_needed,
        payer = payer,
        seeds = [b"stake-vault", mint.key().as_ref()],
        bump = bump,
        token::mint = mint,
        token::authority = stake_vault)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeRewardVault<'info> {
    #[account(init_if_needed,
        payer = payer,
        seeds = [b"reward-vault", mint.key().as_ref()],
        bump = bump,
        token::mint = mint,
        token::authority = reward_vault)]
    pub reward_vault: Account<'info, TokenAccount>,
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
        space = 8 + 32 + 8 + 8 + 8)]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    pub stake_authority: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,
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
            to: accounts.stake_vault.to_account_info(),
            authority: accounts.staker.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = stake_account.authority == authority.key())]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(mut, constraint = to.owner == authority.key())]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct StakeAccount {
    authority: Pubkey,      // 32
    staked_amount: u64,     // 8
    stake_start_time: i64,  // 8
    unclaimed_amount: u64,  // 8
}

impl StakeAccount {
    fn update_unclaimed_amount(&mut self, current_time: i64) {
        // Calculate pending reward amount
        let mut pending_rewards = ((current_time - self.stake_start_time) as u64) * (REWARDS_RATE_PER_SECOND_PER_STAKE * self.staked_amount);
        if self.stake_start_time == 0 {
            pending_rewards = 0;
        }

        // Increase unclaimed_amount by pending reward amount
        self.unclaimed_amount = pending_rewards;
    }
}

#[error]
pub enum ErrorCode {
    #[msg("Error: Invalid vault PDA")]
    InvalidVaultPda,
    #[msg("Error: Invalid StakeAccount PDA")]
    InvalidStakeAccountPda,
    #[msg("Error: Insufficient funds staked")]
    InsufficientFundsStaked,
}
