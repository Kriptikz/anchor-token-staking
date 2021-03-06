use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};

declare_id!("4SgBV6KvC6TvRMPQqwcuNzfNDYcXKCo5TR5T3PFxBau5");

const REWARDS_RATE_PER_SECOND_PER_STAKE: u64 = 1;
const STAKE_LOCK_TIME: i64 = 5;

#[program]
pub mod anchor_token_staking {
    use super::*;

    pub fn initialize_stake_vault(_ctx: Context<InitializeStakeVault>, _bump: u8) -> ProgramResult {
        Ok(())
    }

    pub fn initialize_reward_vault(_ctx: Context<InitializeRewardVault>, _bump: u8) -> ProgramResult {
        Ok(())
    }

    pub fn initialize_stake_account(ctx: Context<InitializeStakeAccount>) -> ProgramResult {
        // Set our defaults for the StakeAccount
        ctx.accounts.stake_account.authority = ctx.accounts.stake_authority.key();
        ctx.accounts.stake_account.staked_amount = 0;
        ctx.accounts.stake_account.stake_start_time = 0;
        ctx.accounts.stake_account.unclaimed_amount = 0;


        Ok(())
    }

    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> ProgramResult {
        // Transfer Tokens into the Stake Vault
        token::transfer((&*ctx.accounts).into(), amount)?;

        let clock = anchor_lang::solana_program::clock::Clock::get()?;
        let time = clock.unix_timestamp;

        // Update our unclaimed_amount -- needed here if the user is staking more tokens on top of their stake 
        let pending_rewards = ctx.accounts.stake_account.calculate_pending_rewards(time);
        ctx.accounts.stake_account.unclaimed_amount += pending_rewards;

        // After updating our unclaimed_amount with possible pending rewards, we set our time to the current time
        ctx.accounts.stake_account.stake_start_time = time;

        // After the stake_account rewards have been calculated, we can then update the amount staked.
        // If we update the amount before updating our unclaimed_amount it will mess up the rewards calculation.
        ctx.accounts.stake_account.staked_amount += amount;

        Ok(())
    }

    pub fn unstake_tokens(ctx: Context<UnstakeTokens>, bump: u8, amount: u64) -> ProgramResult {
        let clock = anchor_lang::solana_program::clock::Clock::get()?;
        let time = clock.unix_timestamp;

        if time < ctx.accounts.stake_account.stake_start_time + STAKE_LOCK_TIME {
            return Err(ErrorCode::StakeIsLocked.into())
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
                &[&[b"stake-vault", ctx.accounts.to.mint.key().as_ref(), &[bump]]]), 
            amount)?;

        // Update our unclaimed_amount -- needed here because we will be changing the staked_amount and stake_start_time so we need to calculate rewards first.
        let pending_rewards = ctx.accounts.stake_account.calculate_pending_rewards(time);
        ctx.accounts.stake_account.unclaimed_amount += pending_rewards;

        // After updating the unclaimed_amount rewards we can then change our staked_amount.
        ctx.accounts.stake_account.staked_amount -= amount;

        // If the staked_amount is > 0, we still have some stake left, so we can updated the time to now.
        // If the staked_amount is 0, we removed all stake, so we can set the time to 0. calculate_pending_rewards returns 0 if the stake_start_time is 0. 
        if ctx.accounts.stake_account.staked_amount > 0 {
            ctx.accounts.stake_account.stake_start_time = time;
        } else {
            ctx.accounts.stake_account.stake_start_time = 0;
        }
        

        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>, bump: u8) -> ProgramResult {
        let clock = anchor_lang::solana_program::clock::Clock::get()?;
        let time = clock.unix_timestamp;

        // Update our unclaimed_amount -- Needed here because we only claim from the unclaimed_amount and need to update it before we can claim anything.
        let pending_rewards = ctx.accounts.stake_account.calculate_pending_rewards(time);
        ctx.accounts.stake_account.unclaimed_amount += pending_rewards;


        // Transfer the amount that was unclaimed to our users token account.
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.reward_vault.to_account_info(),
        };

        token::transfer(
            CpiContext::new_with_signer(
                cpi_program, 
                cpi_accounts,
                &[&[b"reward-vault", ctx.accounts.to.mint.as_ref(), &[bump]]]), 
            ctx.accounts.stake_account.unclaimed_amount)?;

        // Set our unclaimed_amount to 0, since it has now been claimed.
        ctx.accounts.stake_account.unclaimed_amount = 0;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeStakeVault<'info> {
    #[account(init,
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
    #[account(init,
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
pub struct InitializeStakeAccount<'info> {
    #[account(init, 
        payer = stake_authority, 
        seeds = [b"stake-account", mint.key().as_ref(), stake_authority.key().as_ref()],
        bump,
        space = 8 + 32 + 8 + 8 + 8)]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    pub stake_authority: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut,
        seeds = [b"stake-vault", staker_token_account.mint.key().as_ref()],
        bump,)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(mut, 
        seeds = [b"stake-account", staker_token_account.mint.key().as_ref(), staker.key().as_ref()],
        bump,
        constraint = stake_account.authority == staker.key())]
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
#[instruction(bump: u8)]
pub struct UnstakeTokens<'info> {
    #[account(mut,
        seeds = [b"stake-vault", to.mint.key().as_ref()],
        bump = bump,)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(mut,
        seeds = [b"stake-account", to.mint.key().as_ref(), authority.key().as_ref()],
        bump,
        constraint = stake_account.authority == authority.key())]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(mut, constraint = to.owner == authority.key())]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ClaimRewards<'info> {
    #[account(mut,
        seeds = [b"reward-vault", to.mint.key().as_ref()],
        bump = bump,)]
    pub reward_vault: Account<'info, TokenAccount>,
    #[account(mut, 
        seeds = [b"stake-account", to.mint.key().as_ref(), authority.key().as_ref()],
        bump,
        constraint = stake_account.authority == authority.key())]
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
    fn calculate_pending_rewards(&mut self, current_time: i64) -> u64 {
        // If the stake_start_time is zero. Then we have no staked amount so we need to set pending rewards to 0
        // OR we end up with pending rewards going all the way back to Jan 1, 1970 - The start of the unix timestamp.
        if self.stake_start_time == 0 {
            return 0
        }

        // Calculate pending reward amount
        let pending_rewards = ((current_time - self.stake_start_time) as u64) * (REWARDS_RATE_PER_SECOND_PER_STAKE * self.staked_amount);

        pending_rewards
    }
}

#[error]
pub enum ErrorCode {
    #[msg("Error: Insufficient funds staked")]
    InsufficientFundsStaked,
    #[msg("Error: Stake is still locked")]
    StakeIsLocked,
}
