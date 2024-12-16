use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Token, TokenAccount};

#[account]
pub struct TokenDetails {
    pub name: String,
    pub ticker: String,
    pub total_contributed: u64,
    pub target: u64,
    pub total_supply: u64,
    pub creation_time: i64,
    pub is_virtual: bool,
    pub decimals: u8,
    pub contribution_count: u32,
    pub bump: u8,
}

impl TokenDetails {
    pub const MAX_NAME_LENGTH: usize = 32;
    pub const MAX_TICKER_LENGTH: usize = 10;
    
    pub const ACCOUNT_SIZE: usize = 8 +  
        4 + Self::MAX_NAME_LENGTH +      
        4 + Self::MAX_TICKER_LENGTH +    
        8 +                              
        8 +                              
        8 +                              
        8 +                              
        1 +                              
        1 +                              
        4 +                              
        1;                               
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ContributionRecord {
    pub user: Pubkey,
    pub token: Pubkey,
    pub amount: u64,
    pub total_tokens: u64,
    pub contribution_number: u32,
    pub timestamp: i64,
    pub bump: u8,
}

#[account]
#[derive(Default)]
pub struct UserContribution {
    pub user: Pubkey,
    pub token: Pubkey,
    pub amount: u64,
    pub total_tokens: u64,
    pub contribution_number: u32,
    pub timestamp: i64,
    pub bump: u8,
}

impl UserContribution {
    pub const ACCOUNT_SIZE: usize = 8 +  
        32 +                             
        32 +                             
        8 +                              
        8 +                              
        4 +                              
        8 +                              
        1;                               
}

#[account]
pub struct UserContributions {
    pub contributions: Vec<ContributionRecord>, 
}

impl UserContributions {
    pub const ACCOUNT_SIZE: usize = 8 + 4 + (10 * UserContribution::ACCOUNT_SIZE);
}

#[account]
pub struct LiquidityProvider {
    pub shares: u64,
}

impl LiquidityProvider {
    pub const SEED_PREFIX: &'static str = "LiquidityProvider";
    pub const ACCOUNT_SIZE: usize = 8 + 8;
}

#[account]
pub struct LiquidityPool {
    pub token_one: Pubkey, 
    pub token_two: Pubkey, 
    pub total_supply: u64, 
    pub reserve_one: u64,  
    pub reserve_two: u64,  
    pub bump: u8,          
}

impl LiquidityPool {
    pub const POOL_SEED_PREFIX: &'static str = "liquidity_pool";
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 1;

    pub fn new(token_one: Pubkey, bump: u8) -> Self {
        Self {
            token_one,
            token_two: token_one,
            total_supply: 0_u64,
            reserve_one: 0_u64,
            reserve_two: 0_u64,
            bump,
        }
    }
}

pub trait LiquidityPoolAccount<'info> {
    fn transfer_token_from_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
        authority: &AccountInfo<'info>,
        bump: u8
    ) -> Result<()>;

    fn transfer_token_to_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_sol_to_pool(
        &self,
        from: &Signer<'info>,
        to: &AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn transfer_sol_from_pool(
        &self,
        from: &AccountInfo<'info>,
        to: &AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
        bump: u8
    ) -> Result<()>;

    fn transfer_token_to_account(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
        authority: &Signer<'info>,
    ) -> Result<()>;
}

impl<'info> LiquidityPoolAccount<'info> for Account<'info, LiquidityPool> {
    fn transfer_token_from_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
        authority: &AccountInfo<'info>,
        bump: u8
    ) -> Result<()> {
        token::transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: authority.to_account_info(),
                },
                &[&[
                    "global".as_bytes(),
                    &[bump],
                ]],
            ),
            amount,
        )?;
        Ok(())
    }

    fn transfer_token_to_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: authority.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    fn transfer_sol_from_pool(
        &self,
        from: &AccountInfo<'info>,
        to: &AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
        bump: u8
    ) -> Result<()> {
        system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.to_account_info().clone(),
                    to: to.clone(),
                },
                &[&[
                    "global".as_bytes(),
                    &[bump],
                ]],
            ),
            amount,
        )?;
        Ok(())
    }

    fn transfer_sol_to_pool(
        &self,
        from: &Signer<'info>,
        to: &AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }
    
    fn transfer_token_to_account(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
        authority: &Signer<'info>,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: authority.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }
}