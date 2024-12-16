use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::{solana_program::program::invoke_signed};

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    state::{UserContributions, TokenDetails, LiquidityPool, LiquidityPoolAccount},
};

use raydium_contract_instructions::amm_instruction;

pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
    let token = &mut ctx.accounts.token.clone();
    let program_account = &ctx.accounts.program_account;
    let kom_wallet = &ctx.accounts.kom_wallet;

    if token.total_contributed >= token.target {
        let fee = token.total_contributed / 20; // 5%
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: program_account.to_account_info(),
                    to: kom_wallet.to_account_info(),
                },
            ),
            fee,
        )?;

        let remaining_sol = token.total_contributed - fee;

        let tokens_for_liquidity = token.total_supply / 2;
        let tokens_for_users = token.total_supply - tokens_for_liquidity;

        provide_liquidity_on_raydium(&ctx, remaining_sol, tokens_for_liquidity)?;
        distribute_tokens_to_users(&ctx, tokens_for_users)?;

        token.is_virtual = false;
    } else {
        msg!("Total contributions have not reached 200 SOL. Users receive virtual tokens.");
    }

    Ok(())
}

fn provide_liquidity_on_raydium(ctx: &Context<Finalize>, sol_amount: u64, token_amount: u64) -> Result<()> {
    let seeds = &[
        "global".as_bytes(),
        &[ctx.bumps.global_account]
    ];
    let signed_seeds = &[&seeds[..]];

    msg!("Providing liquidity on Raydium");
    let initialize_ix = amm_instruction::initialize2(
        ctx.accounts.amm_program.key,
        ctx.accounts.amm.key,
        ctx.accounts.amm_authority.key,
        ctx.accounts.amm_open_orders.key,
        ctx.accounts.lp_mint.key,
        &ctx.accounts.coin_mint.key(),
        &ctx.accounts.pc_mint.key(),
        ctx.accounts.coin_vault.key,
        ctx.accounts.pc_vault.key,
        ctx.accounts.target_orders.key,
        ctx.accounts.amm_config.key,
        ctx.accounts.fee_destination.key,
        ctx.accounts.market_program.key,
        ctx.accounts.market.key,
        ctx.accounts.global_account.key,
        ctx.accounts.user_token_coin.key,
        ctx.accounts.user_token_pc.key,
        &ctx.accounts.user_token_lp.key(),
        ctx.bumps.global_account,
        Clock::get()?.unix_timestamp as u64,
        sol_amount,
        token_amount,
    )?;
    let account_infos = [
        ctx.accounts.amm_program.clone(),
        ctx.accounts.amm.clone(),
        ctx.accounts.amm_authority.clone(),
        ctx.accounts.amm_open_orders.clone(),
        ctx.accounts.lp_mint.clone(),
        ctx.accounts.coin_mint.to_account_info().clone(),
        ctx.accounts.pc_mint.to_account_info().clone(),
        ctx.accounts.coin_vault.clone(),
        ctx.accounts.pc_vault.clone(),
        ctx.accounts.target_orders.clone(),
        ctx.accounts.amm_config.clone(),
        ctx.accounts.fee_destination.clone(),
        ctx.accounts.market_program.clone(),
        ctx.accounts.market.clone(),
        ctx.accounts.global_account.clone(),
        ctx.accounts.user_token_coin.clone(),
        ctx.accounts.user_token_pc.clone(),
        ctx.accounts.user_token_lp.clone(),
        ctx.accounts.token_program.to_account_info().clone(),
        ctx.accounts.system_program.to_account_info().clone(),
        ctx.accounts.associated_token_program.to_account_info().clone(),
        ctx.accounts.sysvar_rent.to_account_info().clone(),
    ];
    invoke_signed(&initialize_ix, &account_infos, signed_seeds)?;

    Ok(())
}

fn distribute_tokens_to_users(ctx: &Context<Finalize>, tokens_for_users: u64) -> Result<()> {
    let total_tokens_to_distribute = tokens_for_users;
    let user_contributions = &ctx.accounts.user_contributions;

    for user_contribution in &user_contributions.contributions {
        let contribution = user_contribution.amount;

        let user_share = (contribution as f64 / ctx.accounts.token.total_contributed as f64) * total_tokens_to_distribute as f64;
        let user_share_u64 = user_share as u64;

        // Use the transfer_token_from_pool function from the LiquidityPoolAccount trait
        ctx.accounts.liquidity_pool.transfer_token_from_pool(
            &ctx.accounts.pool_token_account,
            &ctx.accounts.user_token_account,
            user_share_u64,
            &ctx.accounts.token_program,
            &ctx.accounts.global_account,
            ctx.bumps.global_account,
        )?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(mut)]
    pub token: Box<Account<'info, TokenDetails>>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub program_account: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub kom_wallet: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub pool_token_account: Account<'info, TokenAccount>, // Change to Account<'info, TokenAccount>
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>, // Change to Account<'info, TokenAccount>
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub amm_program: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub amm: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub amm_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub amm_open_orders: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub lp_mint: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub coin_mint: Box<Account<'info, Mint>>,
    pub pc_mint: Box<Account<'info, Mint>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub coin_vault: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub pc_vault: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub target_orders: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub amm_config: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub fee_destination: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub market_program: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub market: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"global"],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub global_account: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user_token_coin: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user_token_pc: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user_token_lp: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub sysvar_rent: Sysvar<'info, Rent>,
    pub user_contributions: Box<Account<'info, UserContributions>>,
    #[account(mut)]
    pub liquidity_pool: Account<'info, LiquidityPool>, // Access the liquidity pool
}