use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Transfer};

declare_id!("DfpsT9PAeWbwwfE8EqTDqVUiCrsoHF1fogmPw42eqLPH");

#[program]
pub mod vectai_token {
    use super::*;

    /// Initialize the VECTAI token mint with authority and supply
    pub fn initialize(
        ctx: Context<Initialize>, 
        decimals: u8,
        total_supply: u64
    ) -> Result<()> {
        let token_info = &mut ctx.accounts.token_info;
        
        // Store token metadata
        token_info.authority = ctx.accounts.mint_authority.key();
        token_info.mint = ctx.accounts.mint.key();
        token_info.total_supply = total_supply;
        token_info.minted = 0;
        token_info.decimals = decimals;
        
        msg!("VECTAI token initialized: {} decimals, {} total supply", 
             decimals, total_supply);
        Ok(())
    }

    /// Mint tokens to a specified account (only mint authority)
    pub fn mint_to(ctx: Context<MintTo>, amount: u64) -> Result<()> {
        let token_info = &mut ctx.accounts.token_info;
        
        // Check: Validate inputs and mint authority
        require!(amount > 0, TokenError::InvalidAmount);
        require!(
            ctx.accounts.mint_authority.key() == token_info.authority,
            TokenError::InvalidMintAuthority
        );
        
        // Check: Do not exceed total supply (track minted)
        let new_total = token_info
            .minted
            .checked_add(amount)
            .ok_or(TokenError::InsufficientSupply)?;
        require!(new_total <= token_info.total_supply, TokenError::InsufficientSupply);
        
        // Effects: Update state before external call (CEI; tx reverts on CPI failure)
        token_info.minted = new_total;
        
        // Interactions: Call SPL token program
        let cpi_accounts = anchor_spl::token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::mint_to(cpi_ctx, amount)?;
        
        msg!("Minted {} VECTAI tokens", amount);
        Ok(())
    }

    /// Transfer tokens between accounts
    pub fn transfer(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        // Check: Basic validation (SPL token program handles most checks)
        require!(amount > 0, TokenError::InvalidAmount);
        
        // Interactions: Call SPL token program (follows CEI pattern)
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(), 
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::transfer(cpi_ctx, amount)?;
        
        msg!("Transferred {} VECTAI tokens", amount);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        mint::decimals = decimals,
        mint::authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = payer,
        space = TokenInfo::LEN,
        seeds = [b"token-info", mint.key().as_ref()],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    /// CHECK: This is the mint authority and will be validated
    pub mint_authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintTo<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"token-info", mint.key().as_ref()],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    pub mint_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct TokenInfo {
    pub authority: Pubkey,   // Mint authority
    pub mint: Pubkey,        // Mint account
    pub total_supply: u64,   // Maximum supply cap
    pub minted: u64,         // Amount minted so far
    pub decimals: u8,        // Token decimals
}

impl TokenInfo {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // mint
        8 +  // total_supply
        8 +  // minted
        1;   // decimals
}

#[error_code]
pub enum TokenError {
    #[msg("Invalid mint authority")]
    InvalidMintAuthority,
    #[msg("Insufficient supply for minting")]
    InsufficientSupply,
    #[msg("Invalid amount")]
    InvalidAmount,
}