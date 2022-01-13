use anchor_lang::prelude::*;

declare_id!("4SgBV6KvC6TvRMPQqwcuNzfNDYcXKCo5TR5T3PFxBau5");

#[program]
pub mod anchor_token_staking {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
