use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("Etv32C8ZjmHycJEpY6jUUWzMH1bbm7ztC3xUYsKi1366");

#[program]
pub mod pump {
    use super::*;

    pub fn initialize(
        ctx: Context<CreateToken>, 
        name: String, 
        ticker: String, 
        total_supply: u64, 
        initial_target: u64,
        decimals: u8
    ) -> Result<()> {
        instructions::initialize(ctx, name, ticker, total_supply, initial_target, decimals)
    }

    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        instructions::contribute(ctx, amount)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund(ctx)
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        instructions::finalize(ctx)
    }
}