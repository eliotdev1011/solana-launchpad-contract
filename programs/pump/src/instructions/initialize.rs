use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    token::{self, Token, Mint, MintTo, TokenAccount},
    associated_token::AssociatedToken,
};
use crate::state::{TokenDetails, UserContribution};
use crate::{errors::CustomError};

pub fn initialize(mut ctx: Context<CreateToken>, name: String, ticker: String, total_supply: u64, initial_target: u64, decimals: u8) -> Result<()> {
    require!(initial_target > 0 && initial_target <= 10 * 1_000_000_000, CustomError::InvalidInputValue);

    process_transfers(&ctx, initial_target)?;
    initialize_token(&mut ctx, name, ticker, total_supply, initial_target, decimals)?;
    initialize_user_contribution(&mut ctx, initial_target, total_supply)?;

    Ok(())
}

fn process_transfers(ctx: &Context<CreateToken>, initial_target: u64) -> Result<()> {
    let fee = 100_000_000;
    let user = &ctx.accounts.user;
    
    // Transfer fee
    let transfer_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: user.to_account_info(),
            to: ctx.accounts.program_account.to_account_info(),
        },
    );
    system_program::transfer(transfer_ctx, fee)?;

    // Transfer initial target
    let initial_target_transfer_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: user.to_account_info(),
            to: ctx.accounts.program_account.to_account_info(),
        },
    );
    system_program::transfer(initial_target_transfer_ctx, initial_target)?;

    Ok(())
}

fn initialize_token(
    ctx: &mut Context<CreateToken>,
    name: String,
    ticker: String,
    total_supply: u64,
    initial_target: u64,
    decimals: u8,
) -> Result<()> {

    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.mint_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::mint_to(cpi_ctx, total_supply)?;

    let token = &mut ctx.accounts.token;
    token.name = name;
    token.ticker = ticker;
    token.total_contributed = initial_target;
    token.target = 200 * 1_000_000_000;
    token.creation_time = Clock::get()?.unix_timestamp;
    token.total_supply = total_supply;
    token.is_virtual = true;
    token.decimals = decimals;
    token.contribution_count = 1;
    token.bump = ctx.bumps.token;

    Ok(())
}

fn initialize_user_contribution(
    ctx: &mut Context<CreateToken>,
    initial_target: u64,
    total_supply: u64,
) -> Result<()> {
    let user_contribution = &mut ctx.accounts.user_contribution;
    let user = &ctx.accounts.user;
    let token = &ctx.accounts.token;

    user_contribution.user = user.key();
    user_contribution.token = token.key();
    user_contribution.amount = initial_target;
    user_contribution.total_tokens = calculate_tokens(initial_target, total_supply, token.target);
    user_contribution.contribution_number = 0;
    user_contribution.timestamp = Clock::get()?.unix_timestamp;
    user_contribution.bump = ctx.bumps.user_contribution;

    Ok(())
}

fn calculate_tokens(contribution: u64, total_supply: u64, target: u64) -> u64 {
    ((contribution as u128) * (total_supply as u128) / (target as u128)) as u64
}

#[derive(Accounts)]
#[instruction(name: String, ticker: String, total_supply: u64, initial_target: u64, decimals: u8)]
pub struct CreateToken<'info> {
    #[account(
        init,
        payer = user,
        space = TokenDetails::ACCOUNT_SIZE,
        seeds = [b"token", user.key().as_ref(), ticker.as_bytes()],
        bump
    )]
    pub token: Box<Account<'info, TokenDetails>>,
    
    #[account(
        init,
        payer = user,
        mint::decimals = decimals,
        mint::authority = mint_authority.key(),
        seeds = [b"mint", user.key().as_ref(), ticker.as_bytes()],
        bump
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA used as mint authority
    #[account(
        seeds = [b"mint-authority", mint.key().as_ref()],
        bump
    )]
    pub mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub program_account: AccountInfo<'info>,

    #[account(
        init,
        payer = user,
        space = UserContribution::ACCOUNT_SIZE,
        seeds = [
            b"user-contribution",
            user.key().as_ref(),
            ticker.as_bytes(),
            &[0] 
        ],
        bump
    )]
    pub user_contribution: Box<Account<'info, UserContribution>>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}