use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use vectai_oracle::{
    program::VectaiOracle,
    GetPrice, TradingCondition
};

declare_id!("FEmf6TbtffcKVptbshZvCcg3CjQqsWodNwQhpXJff4NP");

// Jupiter Program ID (mainnet-beta)
pub const JUPITER_PROGRAM_ID: Pubkey = solana_program::pubkey!("JUP4LHuHsMkzdFXx4J9qEqN9B7S8ESM5a4L5cdGrUdN");

#[program]
pub mod vectai_trader {
    use super::*;

    /// Initialize trader with secure validation
    pub fn initialize_trader(
        ctx: Context<InitializeTrader>,
        price_threshold: i64,
        swap_amount: u64,
    ) -> Result<()> {
        // Validate inputs
        require!(price_threshold > 0, TraderError::InvalidInput);
        require!(price_threshold < 10_000_000_000, TraderError::InvalidInput); // Max $10B
        require!(swap_amount > 0, TraderError::InvalidInput);
        require!(swap_amount <= 1_000_000_000_000, TraderError::InvalidInput); // Max 1T tokens
        
        let trader_config = &mut ctx.accounts.trader_config;
        trader_config.authority = ctx.accounts.authority.key();
        trader_config.price_threshold = price_threshold;
        trader_config.swap_amount = swap_amount;
        trader_config.total_swaps = 0;
        trader_config.last_swap_time = 0;

        msg!("Secure trader initialized: {} threshold, {} amount", price_threshold, swap_amount);
        Ok(())
    }

    /// Secure: Fetch price → Check condition → Execute swap
    pub fn execute_conditional_swap(ctx: Context<ExecuteSwap>) -> Result<()> {
        let trader_config = &ctx.accounts.trader_config;
        
        // Check: Validate authority
        require!(
            ctx.accounts.authority.key() == trader_config.authority,
            TraderError::Unauthorized
        );
        
        // Check: Rate limiting (max 1 swap per minute)
        let clock = Clock::get()?;
        let time_since_last = clock.unix_timestamp - trader_config.last_swap_time;
        require!(time_since_last >= 60, TraderError::RateLimited);
        
        // Check: Validate token account ownership
        require!(
            ctx.accounts.input_token_account.owner == ctx.accounts.authority.key() ||
            ctx.accounts.input_token_account.delegate.contains(&ctx.accounts.authority.key()),
            TraderError::InvalidTokenAccount
        );
        
        // Check: Sufficient balance
        require!(
            ctx.accounts.input_token_account.amount >= trader_config.swap_amount,
            TraderError::InsufficientBalance
        );
        
        // 1. Fetch price from oracle
        let oracle_cpi_accounts = GetPrice {
            price_update: ctx.accounts.price_update.to_account_info(),
        };
        let oracle_cpi_program = ctx.accounts.oracle_program.to_account_info();
        let oracle_cpi_ctx = CpiContext::new(oracle_cpi_program, oracle_cpi_accounts);
        
        let price_result = vectai_oracle::cpi::check_price_threshold(
            oracle_cpi_ctx,
            trader_config.price_threshold,
        )?;

        // 2. Check condition
        if !price_result.get() {
            msg!("Price threshold not met, no swap");
            return Ok(());
        }

        // Effects: Update state before external call (CEI pattern)
        let trader_config = &mut ctx.accounts.trader_config;
        trader_config.total_swaps = trader_config
            .total_swaps
            .checked_add(1)
            .ok_or(TraderError::MathOverflow)?;
        trader_config.last_swap_time = clock.unix_timestamp;

        // 3. Interactions: Execute swap via Jupiter (placeholder)
        Self::execute_jupiter_swap(&ctx, trader_config.swap_amount)?;

        msg!("Secure swap executed: {} tokens", trader_config.swap_amount);
        Ok(())
    }

    /// Secure condition swap: Fetch Price Data → Condition Check → Swap
    pub fn execute_condition_swap(
        ctx: Context<ExecuteSwap>,
        short_threshold: i64,
        long_threshold: i64,
        target_condition: TradingCondition,
    ) -> Result<()> {
        let trader_config = &ctx.accounts.trader_config;
        
        // Check: Validate authority
        require!(
            ctx.accounts.authority.key() == trader_config.authority,
            TraderError::Unauthorized
        );
        
        // Check: Rate limiting
        let clock = Clock::get()?;
        let time_since_last = clock.unix_timestamp - trader_config.last_swap_time;
        require!(time_since_last >= 60, TraderError::RateLimited);
        
        // Check: Validate inputs 
        require!(short_threshold > 0, TraderError::InvalidInput);
        require!(long_threshold > short_threshold, TraderError::InvalidInput);
        require!(short_threshold < 10_000_000_000, TraderError::InvalidInput);
        require!(long_threshold < 10_000_000_000, TraderError::InvalidInput);
        
        // Check: Token account validation
        require!(
            ctx.accounts.input_token_account.owner == ctx.accounts.authority.key() ||
            ctx.accounts.input_token_account.delegate.contains(&ctx.accounts.authority.key()),
            TraderError::InvalidTokenAccount
        );
        require!(
            ctx.accounts.input_token_account.amount >= trader_config.swap_amount,
            TraderError::InsufficientBalance
        );
        
        // 1. Fetch price and get trading condition
        let oracle_cpi_accounts = GetPrice {
            price_update: ctx.accounts.price_update.to_account_info(),
        };
        let oracle_cpi_program = ctx.accounts.oracle_program.to_account_info();
        let oracle_cpi_ctx = CpiContext::new(oracle_cpi_program, oracle_cpi_accounts);
        
        let current_condition = vectai_oracle::cpi::get_trading_condition(
            oracle_cpi_ctx,
            short_threshold,
            long_threshold,
        )?;

        // 2. Check if condition matches target
        let should_execute = match (current_condition.get(), &target_condition) {
            (TradingCondition::Short, TradingCondition::Short) => true,
            (TradingCondition::Mid, TradingCondition::Mid) => true,
            (TradingCondition::Long, TradingCondition::Long) => true,
            _ => false,
        };
        
        if !should_execute {
            msg!(
                "Condition {:?} doesn't match target {:?}, no swap",
                current_condition.get(),
                target_condition
            );
            return Ok(());
        }

        // Effects: Update state before external call (CEI pattern)
        let trader_config = &mut ctx.accounts.trader_config;
        trader_config.total_swaps = trader_config
            .total_swaps
            .checked_add(1)
            .ok_or(TraderError::MathOverflow)?;
        trader_config.last_swap_time = clock.unix_timestamp;

        // 3. Interactions: Execute swap via Jupiter
        Self::execute_jupiter_swap(&ctx, trader_config.swap_amount)?;

        msg!(
            "Secure condition swap: {:?} → {} tokens",
            current_condition.get(),
            trader_config.swap_amount
        );
        Ok(())
    }

    /// Simple Jupiter swap placeholder
    fn execute_jupiter_swap(
        ctx: &Context<ExecuteSwap>,
        amount: u64,
    ) -> Result<()> {
        // Placeholder: Simple token transfer
        // TODO: Replace with actual Jupiter swap
        let transfer_accounts = Transfer {
            from: ctx.accounts.input_token_account.to_account_info(),
            to: ctx.accounts.output_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, transfer_accounts);
        
        token::transfer(cpi_ctx, amount)?;
        msg!("Jupiter swap: {} tokens", amount);
        Ok(())
    }
}

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
pub struct ExecuteSwap<'info> {
    #[account(
        mut,
        seeds = [b"trader", trader_config.authority.as_ref()],
        bump
    )]
    pub trader_config: Account<'info, TraderConfig>,
    
    /// CHECK: Pyth price update account
    pub price_update: AccountInfo<'info>,
    
    #[account(mut)]
    pub input_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub output_token_account: Account<'info, TokenAccount>,
    
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub oracle_program: Program<'info, VectaiOracle>,
    
    /// CHECK: Jupiter program
    #[account(address = JUPITER_PROGRAM_ID)]
    pub jupiter_program: AccountInfo<'info>,
}

#[account]
pub struct TraderConfig {
    pub authority: Pubkey,
    pub price_threshold: i64,
    pub swap_amount: u64,
    pub total_swaps: u64,
    pub last_swap_time: i64,
}

impl TraderConfig {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 +  // price_threshold
        8 +  // swap_amount
        8 +  // total_swaps
        8;   // last_swap_time
}

#[error_code]
pub enum TraderError {
    #[msg("Invalid input parameters")]
    InvalidInput,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Rate limited: wait 1 minute between swaps")]
    RateLimited,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Insufficient token balance")]
    InsufficientBalance,
    #[msg("Math overflow")]
    MathOverflow,
}
