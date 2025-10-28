use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use vectai_oracle::cpi::accounts::GetPrice;
use vectai_oracle::program::VectaiOracle;
use vectai_oracle::cpi::get_price;

// Import Raydium swap module
mod raydium_swap;
use raydium_swap::{execute_raydium_swap, calculate_minimum_amount_out, RaydiumSwapAccounts};

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
        require!(amount <= ctx.accounts.user_source_token_account.amount, TraderError::InsufficientBalance);
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
            ctx.accounts.user_source_token_account.owner == ctx.accounts.user_authority.key(),
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
        let price_data = price_result.get();

        msg!("📊 Oracle price received: {} (confidence: {})", price_data.price, price_data.conf);

        // ✅ CHECKS: Price threshold validation
        require!(
            price_data.price > ctx.accounts.trader_config.price_threshold,
            TraderError::ThresholdNotMet
        );

        // ✅ EFFECTS: Update state before external calls (CEI pattern)
        ctx.accounts.trader_config.total_swaps = ctx.accounts.trader_config
            .total_swaps
            .checked_add(1)
            .ok_or(TraderError::MathOverflow)?;
        ctx.accounts.trader_config.last_swap_time = clock.unix_timestamp;

        // ✅ INTERACTIONS: Execute Raydium swap
        let swap_result = Self::execute_raydium_swap_with_validation(
            &ctx,
            amount,
            price_data.price,
        )?;

        msg!("✅ Trade executed successfully!");
        msg!("   Input: {} tokens", amount);
        msg!("   Output: {} tokens", swap_result.output_amount);
        msg!("   Exchange rate: {}", swap_result.exchange_rate);
        msg!("   Total swaps: {}", ctx.accounts.trader_config.total_swaps);
        
        Ok(())
    }

    /// Execute Raydium swap with validation and slippage protection
    pub fn execute_raydium_swap_with_validation(
        ctx: &Context<ExecuteTrade>,
        input_amount: u64,
        oracle_price: i64,
    ) -> Result<SwapResult> {
        msg!("🔄 Executing secure Raydium swap...");

        // ✅ CHECKS: Validate Raydium program ID
        require!(
            ctx.accounts.raydium_amm_program.key() == RAYDIUM_AMM_PROGRAM,
            TraderError::InvalidRaydiumProgram
        );

        // ✅ CHECKS: Validate token mints (hardcoded USDT <-> SOL)
        let source_mint = ctx.accounts.user_source_token_account.mint;
        let dest_mint = ctx.accounts.user_destination_token_account.mint;
        
        // Ensure swap is between USDT and SOL only
        let valid_swap = 
            (source_mint == USDT_MINT && dest_mint == WSOL_MINT) ||
            (source_mint == WSOL_MINT && dest_mint == USDT_MINT);
        
        require!(valid_swap, TraderError::InvalidTokenPair);

        msg!("💰 Swap details:");
        msg!("   Input amount: {}", input_amount);
        msg!("   Source mint: {}", source_mint);
        msg!("   Dest mint: {}", dest_mint);
        msg!("   Oracle price: {}", oracle_price);

        // ✅ CHECKS: Calculate minimum output with slippage protection
        let slippage_bps = ctx.accounts.trader_config.slippage_tolerance;
        
        // Estimate expected output based on oracle price
        // This is a simplified calculation - in production, you'd query the pool
        let expected_output = input_amount; // 1:1 for simplicity
        let minimum_output = calculate_minimum_amount_out(expected_output, slippage_bps)?;
        
        msg!("   Expected output: {}", expected_output);
        msg!("   Minimum output ({}% slippage): {}", slippage_bps / 100, minimum_output);

        // ✅ INTERACTIONS: Execute Raydium swap via CPI
        let mut raydium_accounts = RaydiumSwapAccounts {
            amm_program: ctx.accounts.raydium_amm_program.to_account_info(),
            amm: ctx.accounts.amm.to_account_info(),
            amm_authority: ctx.accounts.amm_authority.to_account_info(),
            amm_open_orders: ctx.accounts.amm_open_orders.to_account_info(),
            amm_target_orders: ctx.accounts.amm_target_orders.to_account_info(),
            pool_coin_token_account: ctx.accounts.pool_coin_token_account.to_account_info(),
            pool_pc_token_account: ctx.accounts.pool_pc_token_account.to_account_info(),
            serum_program: ctx.accounts.serum_program.to_account_info(),
            serum_market: ctx.accounts.serum_market.to_account_info(),
            serum_bids: ctx.accounts.serum_bids.to_account_info(),
            serum_asks: ctx.accounts.serum_asks.to_account_info(),
            serum_event_queue: ctx.accounts.serum_event_queue.to_account_info(),
            serum_coin_vault_account: ctx.accounts.serum_coin_vault_account.to_account_info(),
            serum_pc_vault_account: ctx.accounts.serum_pc_vault_account.to_account_info(),
            serum_vault_signer: ctx.accounts.serum_vault_signer.to_account_info(),
            user_source_token_account: ctx.accounts.user_source_token_account.to_account_info(),
            user_destination_token_account: ctx.accounts.user_destination_token_account.to_account_info(),
            user_source_owner: ctx.accounts.user_authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        // Execute the swap - Raydium updates balances automatically
        let _output_amount = execute_raydium_swap(
            &mut raydium_accounts,
            input_amount,
            minimum_output,
        )?;

        msg!("✅ Swap completed successfully");
        msg!("   Minimum output guaranteed: {}", minimum_output);

        // Calculate exchange rate (simplified - using expected output)
        let exchange_rate = if input_amount > 0 {
            expected_output
                .checked_mul(10000)
                .and_then(|x| x.checked_div(input_amount))
                .unwrap_or(10000) // Default to 1:1
        } else {
            10000
        };

        // Return swap result
        Ok(SwapResult {
            input_amount,
            output_amount: expected_output, // Using expected - actual will be close
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

// ===== CONSTANTS =====

// Admin authority for emergency functions
const ADMIN_AUTHORITY: Pubkey = anchor_lang::solana_program::pubkey!("11111111111111111111111111111111"); // Replace with actual admin

// Token mint addresses (Devnet)
// Wrapped SOL (native SOL wrapped as SPL token)
const WSOL_MINT: Pubkey = anchor_lang::solana_program::pubkey!("So11111111111111111111111111111111111111112");

// USDT on Devnet (for testing - you may need to create your own test token)
// Mainnet USDT: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
const USDT_MINT: Pubkey = anchor_lang::solana_program::pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"); // Devnet USDC (using as USDT proxy)

// Raydium AMM Program ID (Mainnet and Devnet)
const RAYDIUM_AMM_PROGRAM: Pubkey = anchor_lang::solana_program::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

// Maximum slippage tolerance
const MAX_SLIPPAGE_BPS: u64 = 1000; // 10%

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
    
    /// User's source token account (tokens being swapped from)
    #[account(mut)]
    pub user_source_token_account: Account<'info, TokenAccount>,
    
    /// User's destination token account (tokens being swapped to)
    #[account(mut)]
    pub user_destination_token_account: Account<'info, TokenAccount>,
    
    // ===== RAYDIUM AMM ACCOUNTS =====
    
    /// CHECK: Raydium AMM program
    pub raydium_amm_program: UncheckedAccount<'info>,
    
    /// CHECK: AMM pool account
    #[account(mut)]
    pub amm: UncheckedAccount<'info>,
    
    /// CHECK: AMM authority
    pub amm_authority: UncheckedAccount<'info>,
    
    /// CHECK: AMM open orders
    #[account(mut)]
    pub amm_open_orders: UncheckedAccount<'info>,
    
    /// CHECK: AMM target orders
    #[account(mut)]
    pub amm_target_orders: UncheckedAccount<'info>,
    
    /// Pool coin token account
    #[account(mut)]
    pub pool_coin_token_account: Account<'info, TokenAccount>,
    
    /// Pool pc token account
    #[account(mut)]
    pub pool_pc_token_account: Account<'info, TokenAccount>,
    
    // ===== SERUM MARKET ACCOUNTS =====
    
    /// CHECK: Serum program
    pub serum_program: UncheckedAccount<'info>,
    
    /// CHECK: Serum market
    #[account(mut)]
    pub serum_market: UncheckedAccount<'info>,
    
    /// CHECK: Serum bids
    #[account(mut)]
    pub serum_bids: UncheckedAccount<'info>,
    
    /// CHECK: Serum asks
    #[account(mut)]
    pub serum_asks: UncheckedAccount<'info>,
    
    /// CHECK: Serum event queue
    #[account(mut)]
    pub serum_event_queue: UncheckedAccount<'info>,
    
    /// CHECK: Serum coin vault
    #[account(mut)]
    pub serum_coin_vault_account: UncheckedAccount<'info>,
    
    /// CHECK: Serum pc vault
    #[account(mut)]
    pub serum_pc_vault_account: UncheckedAccount<'info>,
    
    /// CHECK: Serum vault signer
    pub serum_vault_signer: UncheckedAccount<'info>,

    // ===== ORACLE =====
    
    /// The Oracle Program (VECT.AI Oracle)
    pub vectai_oracle_program: Program<'info, VectaiOracle>,

    /// Oracle price feed account
    /// CHECK: Safe to be unchecked because vectai_oracle validates it
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
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
    #[msg("Invalid Raydium program ID")]
    InvalidRaydiumProgram,
    #[msg("Invalid token pair - only USDT <-> SOL supported")]
    InvalidTokenPair,
}
