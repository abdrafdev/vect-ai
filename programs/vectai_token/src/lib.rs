use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("DfpsT9PAeWbwwfE8EqTDqVUiCrsoHF1fogmPw42eqLPH");

#[program]
pub mod vectai_token {
    use super::*;

    /// Initialize token with supply cap and authority
    pub fn initialize_token(
        ctx: Context<InitializeToken>,
        max_supply: u64,
        decimals: u8,
    ) -> Result<()> {
        let token_info = &mut ctx.accounts.token_info;
        
        // Initialize token metadata
        token_info.mint_authority = ctx.accounts.mint_authority.key();
        token_info.mint = ctx.accounts.mint.key();
        token_info.max_supply = max_supply;
        token_info.minted = 0;
        token_info.decimals = decimals;
        token_info.is_paused = false;
        
        msg!("VECTAI token initialized: {} max supply, {} decimals", max_supply, decimals);
        Ok(())
    }

    /// Secure mint tokens with authorization and supply checks
    pub fn mint_to(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        // ✅ CHECKS: Validate inputs and authorization
        require!(amount > 0, TokenError::InvalidAmount);
        require!(!ctx.accounts.token_info.is_paused, TokenError::TokenPaused);
        require!(
            ctx.accounts.mint_authority.key() == ctx.accounts.token_info.mint_authority,
            TokenError::UnauthorizedMintAuthority
        );
        
        // Check supply cap
        let new_total = ctx.accounts.token_info
            .minted
            .checked_add(amount)
            .ok_or(TokenError::MathOverflow)?;
        require!(new_total <= ctx.accounts.token_info.max_supply, TokenError::ExceedsMaxSupply);
        
        // ✅ EFFECTS: Update state before external call (CEI pattern)
        ctx.accounts.token_info.minted = new_total;
        
        // ✅ INTERACTIONS: Execute CPI after state update
        let cpi_accounts = anchor_spl::token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::mint_to(cpi_ctx, amount)?;
        
        msg!("✅ Minted {} VECTAI tokens (Total minted: {})", amount, new_total);
        Ok(())
    }

    /// Secure transfer tokens with ownership validation
    pub fn transfer(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        // ✅ CHECKS: Validate inputs and ownership
        require!(amount > 0, TokenError::InvalidAmount);
        require!(
            ctx.accounts.from.owner == ctx.accounts.authority.key(),
            TokenError::InvalidTokenAccount
        );
        require!(
            ctx.accounts.from.amount >= amount,
            TokenError::InsufficientBalance
        );
        
        // ✅ INTERACTIONS: Execute transfer (no state changes needed)
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(), 
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::transfer(cpi_ctx, amount)?;
        
        msg!("✅ Transferred {} VECTAI tokens from {} to {}", 
             amount, ctx.accounts.from.key(), ctx.accounts.to.key());
        Ok(())
    }

    /// Emergency pause function (admin only)
    pub fn pause_token(ctx: Context<PauseToken>) -> Result<()> {
        require!(
            ctx.accounts.admin.key() == ADMIN_AUTHORITY,
            TokenError::UnauthorizedAdmin
        );
        
        ctx.accounts.token_info.is_paused = true;
        msg!("🚨 VECTAI token paused by admin");
        Ok(())
    }

    /// Unpause token (admin only)
    pub fn unpause_token(ctx: Context<PauseToken>) -> Result<()> {
        require!(
            ctx.accounts.admin.key() == ADMIN_AUTHORITY,
            TokenError::UnauthorizedAdmin
        );
        
        ctx.accounts.token_info.is_paused = false;
        msg!("✅ VECTAI token unpaused by admin");
        Ok(())
    }
}

// Constants
const ADMIN_AUTHORITY: Pubkey = anchor_lang::solana_program::pubkey!("11111111111111111111111111111111"); // Replace with actual admin

#[derive(Accounts)]
#[instruction(max_supply: u64, decimals: u8)]
pub struct InitializeToken<'info> {
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
    
    pub mint_authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
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

#[derive(Accounts)]
pub struct PauseToken<'info> {
    #[account(
        mut,
        seeds = [b"token-info", token_info.mint.as_ref()],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    pub admin: Signer<'info>,
}

#[account]
pub struct TokenInfo {
    pub mint_authority: Pubkey,
    pub mint: Pubkey,
    pub max_supply: u64,
    pub minted: u64,
    pub decimals: u8,
    pub is_paused: bool,
}

impl TokenInfo {
    pub const LEN: usize = 8 + // discriminator
        32 + // mint_authority
        32 + // mint
        8 +  // max_supply
        8 +  // minted
        1 +  // decimals
        1;   // is_paused
}

#[error_code]
pub enum TokenError {
    #[msg("Invalid amount - must be greater than 0")]
    InvalidAmount,
    #[msg("Unauthorized mint authority")]
    UnauthorizedMintAuthority,
    #[msg("Exceeds maximum supply")]
    ExceedsMaxSupply,
    #[msg("Math overflow in calculation")]
    MathOverflow,
    #[msg("Token is paused")]
    TokenPaused,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
}
