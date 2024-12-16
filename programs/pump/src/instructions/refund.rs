use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::{TokenDetails, UserContribution};
use crate::errors::CustomError; 

pub fn refund(ctx: Context<Refund>) -> Result<()> {
    let token = &ctx.accounts.token;
    let user = &ctx.accounts.user;

    let current_time = Clock::get()?.unix_timestamp;
    if current_time > token.creation_time + 7 * 24 * 60 * 60 && token.total_contributed < token.target {
        let user_contribution = &mut ctx.accounts.user_contribution; 
        let refund_amount = user_contribution.amount;

        require!(refund_amount > 0, CustomError::NoContributionToRefund);

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.program_account.to_account_info(),
                    to: user.to_account_info(),
                },
            ),
            refund_amount,
        )?;

        user_contribution.amount = 0;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub token: Box<Account<'info, TokenDetails>>,
    #[account(mut)]
    pub user_contribution: Box<Account<'info, UserContribution>>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub program_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}