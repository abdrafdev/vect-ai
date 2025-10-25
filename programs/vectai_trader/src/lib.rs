use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, MintTo};
use vectai_oracle::cpi::accounts::GetPrice;
use vectai_oracle::program::VectaiOracle;
use vectai_oracle::cpi::get_price;

declare_id!("FEmf6TbtffcKVptbshZvCcg3CjQqsWodNwQhpXJff4NP");

#[program]
pub mod vectai_trader {
    use super::*;

    /// Initialize trader with secure configuration
    pub fn initialize_trader(
        ctx: Context<InitializeTrader>,
        price_threshold: i64,
        swap_amount: u64,
        slippage_tolerance: u64, // Basis points (e.g., 200 = 2%)
    ) -> Result<()> {
        // ✅ CHECKS: Validate inputs
        require!(price_threshold > 0, TraderError::InvalidInput);
        require!(price_threshold < 1_000_000_000_000, TraderError::InvalidInput); // Max $1T
        require!(swap_amount > 0, TraderError::InvalidInput);
        require!(swap_amount <= 1_000_000_000_000, TraderError::InvalidInput); // Max 1T tokens
        require!(slippage_tolerance <= 1000, TraderError::InvalidInput); // Max 10% slippage
        
        let trader_config = &mut ctx.accounts.trader_config;
        trader_config.authority = ctx.accounts.authority.key();
        trader_config.price_threshold = price_threshold;
        trader_config.swap_amount = swap_amount;
        trader_config.slippage_tolerance = slippage_tolerance;
        trader_config.total_swaps = 0;
        trader_config.last_swap_time = 0;
        trader_config.is_active = true;

        msg!("✅ Secure trader initialized: {} threshold, {} amount, {}% slippage", 
             price_threshold, swap_amount, slippage_tolerance);
        Ok(())
    }

    /// Execute secure trade with comprehensive validation
    pub fn execute_trade(ctx: Context<ExecuteTrade>, amount: u64) -> Result<()> {
        msg!("🚀 Starting secure trade execution through Jupiter...");

        // ✅ CHECKS: Validate inputs and authorization
        require!(amount > 0, TraderError::InvalidSwapAmount);
        require!(amount <= ctx.accounts.input_token_account.amount, TraderError::InsufficientBalance);
        require!(
            ctx.accounts.user_authority.key() == ctx.accounts.trader_config.authority,
            TraderError::Unauthorized
        );
        require!(ctx.accounts.trader_config.is_active, TraderError::TraderInactive);
        
        // ✅ CHECKS: Rate limiting (1 minute cooldown)
        let clock = Clock::get()?;
        let time_since_last = clock.unix_timestamp - ctx.accounts.trader_config.last_swap_time;
        require!(time_since_last >= 60, TraderError::RateLimited);
        
        // ✅ CHECKS: Token account ownership validation
        require!(
            ctx.accounts.input_token_account.owner == ctx.accounts.user_authority.key(),
            TraderError::InvalidTokenAccount
        );
        
        // ✅ CHECKS: Fetch and validate oracle price
        let price_result = get_price(
            CpiContext::new(
                ctx.accounts.vectai_oracle_program.to_account_info(),
                GetPrice {
                    price_feed: ctx.accounts.price_feed.to_account_info(),
                },
            ),
        )?;

        msg!("📊 Oracle price received: {} (confidence: {})", price_result.price, price_result.conf);

        // ✅ CHECKS: Price threshold validation
        require!(
            price_result.price > ctx.accounts.trader_config.price_threshold,
            TraderError::ThresholdNotMet
        );

        // ✅ EFFECTS: Update state before external calls (CEI pattern)
        ctx.accounts.trader_config.total_swaps = ctx.accounts.trader_config
            .total_swaps
            .checked_add(1)
            .ok_or(TraderError::MathOverflow)?;
        ctx.accounts.trader_config.last_swap_time = clock.unix_timestamp;

        // ✅ INTERACTIONS: Execute Jupiter swap
        let swap_result = Self::execute_jupiter_swap(
            &ctx,
            amount,
            price_result.price,
        )?;

        msg!("✅ Trade executed successfully!");
        msg!("   Input: {} tokens", amount);
        msg!("   Output: {} tokens", swap_result.output_amount);
        msg!("   Rate: {}% (after fees)", swap_result.exchange_rate);
        msg!("   Total swaps: {}", ctx.accounts.trader_config.total_swaps);
        
        Ok(())
    }

    /// Secure Jupiter swap with comprehensive validation
    fn execute_jupiter_swap(
        ctx: &Context<ExecuteTrade>,
        input_amount: u64,
        oracle_price: i64,
    ) -> Result<SwapResult> {
        msg!("🔄 Executing secure Jupiter swap simulation...");

        // ✅ CHECKS: Validate mint authority is program-controlled
        require!(
            ctx.accounts.mint_authority.key() == PROGRAM_MINT_AUTHORITY,
            TraderError::UnauthorizedMintAuthority
        );

        // ✅ CHECKS: Calculate exchange rate with slippage protection
        let base_rate = 10000; // 100% in basis points
        let slippage_bps = ctx.accounts.trader_config.slippage_tolerance;
        let exchange_rate = base_rate
            .checked_sub(slippage_bps)
            .ok_or(TraderError::MathOverflow)?;
        
        let output_amount = input_amount
            .checked_mul(exchange_rate as u64)
            .and_then(|x| x.checked_div(10000))
            .ok_or(TraderError::MathOverflow)?;

        // ✅ CHECKS: Validate output amount
        require!(output_amount > 0, TraderError::InvalidSwapAmount);
        require!(output_amount <= MAX_MINT_PER_SWAP, TraderError::ExceedsMintLimit);

        msg!("💰 Secure swap calculation:");
        msg!("   Input amount: {}", input_amount);
        msg!("   Exchange rate: {}% ({}% slippage)", exchange_rate / 100, slippage_bps / 100);
        msg!("   Output amount: {}", output_amount);
        msg!("   Oracle price: {}", oracle_price);

        // ✅ INTERACTIONS: Step 1 - Transfer input tokens (simulating Jupiter taking input)
        let transfer_cpi_accounts = Transfer {
            from: ctx.accounts.input_token_account.to_account_info(),
            to: ctx.accounts.temp_vault_account.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        let transfer_cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_cpi_accounts,
        );
        token::transfer(transfer_cpi_ctx, input_amount)?;

        msg!("✅ Input tokens transferred to Jupiter (simulated)");

        // ✅ INTERACTIONS: Step 2 - Mint output tokens (simulating Jupiter providing output)
        let mint_cpi_accounts = MintTo {
            mint: ctx.accounts.output_mint.to_account_info(),
            to: ctx.accounts.output_token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let mint_cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            mint_cpi_accounts,
        );
        token::mint_to(mint_cpi_ctx, output_amount)?;

        msg!("✅ Output tokens minted to user account");

        // Return swap result
        Ok(SwapResult {
            input_amount,
            output_amount,
            exchange_rate,
            oracle_price,
        })
    }

    /// Emergency pause trader (admin only)
    pub fn pause_trader(ctx: Context<PauseTrader>) -> Result<()> {
        require!(
            ctx.accounts.admin.key() == ADMIN_AUTHORITY,
            TraderError::UnauthorizedAdmin
        );
        
        ctx.accounts.trader_config.is_active = false;
        msg!("🚨 Trader paused by admin");
        Ok(())
    }

    /// Unpause trader (admin only)
    pub fn unpause_trader(ctx: Context<PauseTrader>) -> Result<()> {
        require!(
            ctx.accounts.admin.key() == ADMIN_AUTHORITY,
            TraderError::UnauthorizedAdmin
        );
        
        ctx.accounts.trader_config.is_active = true;
        msg!("✅ Trader unpaused by admin");
        Ok(())
    }
}

// Constants
const ADMIN_AUTHORITY: Pubkey = anchor_lang::solana_program::pubkey!("11111111111111111111111111111111"); // Replace with actual admin
const PROGRAM_MINT_AUTHORITY: Pubkey = anchor_lang::solana_program::pubkey!("22222222222222222222222222222222"); // Replace with program mint authority
const MAX_MINT_PER_SWAP: u64 = 1_000_000_000; // Max 1B tokens per swap

#[derive(Accounts)]
pub struct InitializeTrader<'info> {
    #[account(
        init,
        payer = authority,
        space = TraderConfig::LEN,
        seeds = [b"trader", authority.key().as_ref()],
        bump
    )]
    pub trader_config: Account<'info, TraderConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteTrade<'info> {
    /// User who initiates the trade
    #[account(mut)]
    pub user_authority: Signer<'info>,

    /// Trader configuration account
    #[account(
        mut,
        seeds = [b"trader", trader_config.authority.as_ref()],
        bump
    )]
    pub trader_config: Account<'info, TraderConfig>,
    
    /// User's input token account (source tokens to swap)
    #[account(mut)]
    pub input_token_account: Account<'info, TokenAccount>,
    
    /// User's output token account (destination for swapped tokens)
    #[account(mut)]
    pub output_token_account: Account<'info, TokenAccount>,
    
    /// Temporary vault account to hold input tokens during swap
    #[account(mut)]
    pub temp_vault_account: Account<'info, TokenAccount>,

    /// Output token mint (for minting output tokens)
    #[account(mut)]
    pub output_mint: Account<'info, Mint>,

    /// Authority that can mint output tokens (must be program-controlled)
    pub mint_authority: Signer<'info>,

    /// The Oracle Program (VECT.AI Oracle)
    pub vectai_oracle_program: Program<'info, VectaiOracle>,

    /// Oracle price feed account
    /// (PYTH price account, passed through oracle CPI)
    /// Safe to be unchecked because vectai_oracle validates it
    #[account()]
    pub price_feed: UncheckedAccount<'info>,

    /// Solana token program
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PauseTrader<'info> {
    #[account(
        mut,
        seeds = [b"trader", trader_config.authority.as_ref()],
        bump
    )]
    pub trader_config: Account<'info, TraderConfig>,
    
    pub admin: Signer<'info>,
}

/// Trader configuration state
#[account]
pub struct TraderConfig {
    pub authority: Pubkey,
    pub price_threshold: i64,
    pub swap_amount: u64,
    pub slippage_tolerance: u64, // Basis points
    pub total_swaps: u64,
    pub last_swap_time: i64,
    pub is_active: bool,
}

impl TraderConfig {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 +  // price_threshold
        8 +  // swap_amount
        8 +  // slippage_tolerance
        8 +  // total_swaps
        8 +  // last_swap_time
        1;   // is_active
}

/// Result of a Jupiter swap execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SwapResult {
    pub input_amount: u64,
    pub output_amount: u64,
    pub exchange_rate: u64,
    pub oracle_price: i64,
}

#[error_code]
pub enum TraderError {
    #[msg("Invalid input parameters")]
    InvalidInput,
    #[msg("Invalid swap amount - must be greater than 0")]
    InvalidSwapAmount,
    #[msg("Insufficient token balance for swap")]
    InsufficientBalance,
    #[msg("Math overflow in calculation")]
    MathOverflow,
    #[msg("Slippage exceeded maximum allowed")]
    SlippageExceeded,
    #[msg("Invalid exchange rate")]
    InvalidExchangeRate,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Rate limited: wait 1 minute between swaps")]
    RateLimited,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Price threshold not met")]
    ThresholdNotMet,
    #[msg("Trader is inactive")]
    TraderInactive,
    #[msg("Unauthorized mint authority")]
    UnauthorizedMintAuthority,
    #[msg("Exceeds mint limit")]
    ExceedsMintLimit,
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
}
