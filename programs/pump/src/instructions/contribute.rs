use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::errors::CustomError; 
use crate::state::{TokenDetails, UserContribution};

pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    let token = &mut ctx.accounts.token;
    let user = &ctx.accounts.user;

    require!(
        user_contribution.amount + amount <= 10 * 1_000_000_000,
        CustomError::TargetExceeded
    );
    require!(token.total_contributed + amount <= token.target, CustomError::TargetExceeded);

    token.total_contributed = token.total_contributed.checked_add(amount)
        .ok_or(CustomError::OverflowOrUnderflowOccurred)?;
    
    user_contribution.amount = user_contribution.amount.checked_add(amount)
        .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

    token.contribution_count += 1;

    let user_contribution = &mut ctx.accounts.user_contribution;
    user_contribution.total_tokens = calculate_tokens(user_contribution.amount, token.total_supply, token.target);
    user_contribution.contribution_number = token.contribution_count - 1;
    user_contribution.timestamp = Clock::get()?.unix_timestamp;

    // Transfer contribution
    let transfer_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: user.to_account_info(),
            to: ctx.accounts.program_account.to_account_info(),
        },
    );

    system_program::transfer(transfer_ctx, amount)?;

    Ok(())
}

fn calculate_tokens(contribution: u64, total_supply: u64, target: u64) -> u64 {
    ((contribution as u128) * (total_supply as u128) / (target as u128)) as u64
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub token: Box<Account<'info, TokenDetails>>,

    #[account(
        init,
        payer = user,
        space = UserContribution::ACCOUNT_SIZE,
        seeds = [
            b"user-contribution",
            user.key().as_ref(),
            token.key().as_ref(),
            &token.contribution_count.to_le_bytes()
        ],
        bump
    )]
    pub user_contribution: Account<'info, UserContribution>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub program_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}