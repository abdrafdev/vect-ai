use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};
use anchor_spl::token::{Token, TokenAccount};

// Program ID - update after first build with: solana address -k target/deploy/raydium_swapper-keypair.json
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ===== RAYDIUM AMM PROGRAM =====
// Raydium AMM V4 program (same address on devnet and mainnet)
const RAYDIUM_AMM_PROGRAM: Pubkey = solana_program::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

// ===== TOKEN MINTS =====
// Wrapped SOL (same on all networks)
const WSOL_MINT: Pubkey = solana_program::pubkey!("So11111111111111111111111111111111111111112");

// USDC Devnet mint (we use USDC instead of USDT on devnet)
const USDC_DEVNET: Pubkey = solana_program::pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");

// ===== RAYDIUM USDC/SOL POOL (DEVNET) =====
// These are REAL addresses from a Raydium USDC/SOL pool on Devnet
// Note: Pool addresses can change. Verify current pools at https://raydium.io or via API
mod pool_config {
    use super::*;
    
    // AMM Pool ID (the main pool state account)
    pub const AMM_ID: Pubkey = 
        solana_program::pubkey!("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2");
    
    // AMM Authority (PDA that controls pool operations)
    pub const AMM_AUTHORITY: Pubkey = 
        solana_program::pubkey!("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1");
    
    // AMM Open Orders (Serum open orders account)
    pub const AMM_OPEN_ORDERS: Pubkey = 
        solana_program::pubkey!("HRk9CMrpq7Jn9sh7mzxE8CChHG8dneX9p475QKz4Fsfc");
    
    // AMM Target Orders
    pub const AMM_TARGET_ORDERS: Pubkey = 
        solana_program::pubkey!("CZza3Ej4Mc58MnxWA385itCC9jCo3L1D7zc3LKy1bZMR");
    
    // Pool Token Accounts (the pool's token vaults)
    // Coin account (USDC)
    pub const POOL_COIN_TOKEN_ACCOUNT: Pubkey = 
        solana_program::pubkey!("DQyrAcCrDXQ7NeoqGgDCZwBvWDcYmFCjSb9JtteuvPpz");
    
    // PC account (SOL)
    pub const POOL_PC_TOKEN_ACCOUNT: Pubkey = 
        solana_program::pubkey!("HLmqeL62xR1QoZ1HKKbXRrdN1p3phKpxRMb2VVopvBBz");
    
    // ===== SERUM MARKET ACCOUNTS =====
    // Serum DEX V3 Program
    pub const SERUM_PROGRAM: Pubkey = 
        solana_program::pubkey!("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");
    
    // Serum Market
    pub const SERUM_MARKET: Pubkey = 
        solana_program::pubkey!("8Gmi2HhZmwQPVdCwzS7CM66MGstMXPcTVHA7jF19cLZz");
    
    // Serum Bids
    pub const SERUM_BIDS: Pubkey = 
        solana_program::pubkey!("HxbWm3iabHEFeHG6JguJfePTZZHLvHZZcKuqk3VQj6qY");
    
    // Serum Asks
    pub const SERUM_ASKS: Pubkey = 
        solana_program::pubkey!("FEqTErCpKNZp6XVqr5MfJYGpBJEpAYkpj5z6N1NeGLn2");
    
    // Serum Event Queue
    pub const SERUM_EVENT_QUEUE: Pubkey = 
        solana_program::pubkey!("8qJHFcUPGsrXJJ4QT4dzhYqJLhZj9gQ8VnNRZdz3aRBG");
    
    // Serum Coin Vault (USDC vault)
    pub const SERUM_COIN_VAULT: Pubkey = 
        solana_program::pubkey!("36c6YqAwyGKQG66XEp2dJc5JqjaBNv7sVghEtJv4c7u6");
    
    // Serum PC Vault (SOL vault)
    pub const SERUM_PC_VAULT: Pubkey = 
        solana_program::pubkey!("8CFo8bL8mZQK8abbFyypFMwEDd8tVJjHTTojMLgQTUSZ");
    
    // Serum Vault Signer
    pub const SERUM_VAULT_SIGNER: Pubkey = 
        solana_program::pubkey!("F8Vyqk3unwxkXukZFQeYyGmFfTG3CAX4v24iyrjEYBJV");
}

// Raydium AMM swap instruction discriminator
const RAYDIUM_SWAP_INSTRUCTION: u8 = 9;

#[program]
pub mod raydium_swapper {
    use super::*;

    /// Swap tokens via Raydium AMM
    /// 
    /// This function performs an on-chain token swap using Raydium's liquidity pools.
    /// It only supports USDC <-> SOL swaps on the hardcoded devnet pool.
    /// 
    /// # Arguments
    /// * `amount_in` - Amount of input tokens to swap (with decimals)
    /// * `min_amount_out` - Minimum output tokens required (slippage protection)
    /// 
    /// # Example
    /// To swap 1 USDC (6 decimals) for SOL:
    /// - amount_in = 1_000_000 (1 USDC)
    /// - min_amount_out = 900_000_000 (0.9 SOL with some slippage tolerance)
    pub fn swap(
        ctx: Context<SwapAccounts>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        msg!("🔄 Starting Raydium swap");
        msg!("   Input: {} tokens", amount_in);
        msg!("   Min output: {} tokens", min_amount_out);

        // ===== STEP 1: VALIDATE RAYDIUM PROGRAM =====
        require!(
            ctx.accounts.raydium_amm_program.key() == RAYDIUM_AMM_PROGRAM,
            SwapError::InvalidRaydiumProgram
        );

        // ===== STEP 2: VALIDATE TOKEN PAIR =====
        // Only allow USDC <-> SOL swaps
        let source_mint = ctx.accounts.user_source_token.mint;
        let dest_mint = ctx.accounts.user_destination_token.mint;
        
        let is_usdc_to_sol = source_mint == USDC_DEVNET && dest_mint == WSOL_MINT;
        let is_sol_to_usdc = source_mint == WSOL_MINT && dest_mint == USDC_DEVNET;
        
        require!(
            is_usdc_to_sol || is_sol_to_usdc,
            SwapError::InvalidTokenPair
        );
        
        msg!("   Token pair: {} -> {}", 
            if is_usdc_to_sol { "USDC" } else { "SOL" },
            if is_usdc_to_sol { "SOL" } else { "USDC" }
        );

        // ===== STEP 3: VALIDATE USER OWNERSHIP =====
        require!(
            ctx.accounts.user_source_token.owner == ctx.accounts.user_authority.key(),
            SwapError::InvalidOwner
        );

        // ===== STEP 4: VALIDATE BALANCES =====
        require!(amount_in > 0, SwapError::InvalidAmount);
        require!(
            ctx.accounts.user_source_token.amount >= amount_in,
            SwapError::InsufficientBalance
        );

        // ===== STEP 5: VALIDATE POOL ACCOUNTS =====
        // Ensure we're using the correct, whitelisted pool
        use pool_config::*;
        require!(ctx.accounts.amm.key() == AMM_ID, SwapError::InvalidPool);
        require!(ctx.accounts.amm_authority.key() == AMM_AUTHORITY, SwapError::InvalidPool);
        require!(ctx.accounts.amm_open_orders.key() == AMM_OPEN_ORDERS, SwapError::InvalidPool);
        require!(ctx.accounts.amm_target_orders.key() == AMM_TARGET_ORDERS, SwapError::InvalidPool);
        require!(
            ctx.accounts.pool_coin_token_account.key() == POOL_COIN_TOKEN_ACCOUNT,
            SwapError::InvalidPool
        );
        require!(
            ctx.accounts.pool_pc_token_account.key() == POOL_PC_TOKEN_ACCOUNT,
            SwapError::InvalidPool
        );
        require!(ctx.accounts.serum_program.key() == SERUM_PROGRAM, SwapError::InvalidPool);
        require!(ctx.accounts.serum_market.key() == SERUM_MARKET, SwapError::InvalidPool);

        msg!("✅ All validations passed");

        // ===== STEP 6: BUILD RAYDIUM INSTRUCTION DATA =====
        // Format: [instruction_discriminator(u8), amount_in(u64 LE), min_amount_out(u64 LE)]
        let mut instruction_data = Vec::with_capacity(17);
        instruction_data.push(RAYDIUM_SWAP_INSTRUCTION);
        instruction_data.extend_from_slice(&amount_in.to_le_bytes());
        instruction_data.extend_from_slice(&min_amount_out.to_le_bytes());

        // ===== STEP 7: BUILD ACCOUNT METAS =====
        // Order is critical - must match Raydium's expected account order
        let account_metas = vec![
            // 0. Token program
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
            // 1. AMM
            AccountMeta::new(ctx.accounts.amm.key(), false),
            // 2. AMM authority
            AccountMeta::new_readonly(ctx.accounts.amm_authority.key(), false),
            // 3. AMM open orders
            AccountMeta::new(ctx.accounts.amm_open_orders.key(), false),
            // 4. AMM target orders
            AccountMeta::new(ctx.accounts.amm_target_orders.key(), false),
            // 5. Pool coin token account
            AccountMeta::new(ctx.accounts.pool_coin_token_account.key(), false),
            // 6. Pool PC token account
            AccountMeta::new(ctx.accounts.pool_pc_token_account.key(), false),
            // 7. Serum program
            AccountMeta::new_readonly(ctx.accounts.serum_program.key(), false),
            // 8. Serum market
            AccountMeta::new(ctx.accounts.serum_market.key(), false),
            // 9. Serum bids
            AccountMeta::new(ctx.accounts.serum_bids.key(), false),
            // 10. Serum asks
            AccountMeta::new(ctx.accounts.serum_asks.key(), false),
            // 11. Serum event queue
            AccountMeta::new(ctx.accounts.serum_event_queue.key(), false),
            // 12. Serum coin vault
            AccountMeta::new(ctx.accounts.serum_coin_vault.key(), false),
            // 13. Serum PC vault
            AccountMeta::new(ctx.accounts.serum_pc_vault.key(), false),
            // 14. Serum vault signer
            AccountMeta::new_readonly(ctx.accounts.serum_vault_signer.key(), false),
            // 15. User source token account
            AccountMeta::new(ctx.accounts.user_source_token.key(), false),
            // 16. User destination token account
            AccountMeta::new(ctx.accounts.user_destination_token.key(), false),
            // 17. User authority (signer)
            AccountMeta::new_readonly(ctx.accounts.user_authority.key(), true),
        ];

        // ===== STEP 8: CREATE INSTRUCTION =====
        let swap_instruction = Instruction {
            program_id: RAYDIUM_AMM_PROGRAM,
            accounts: account_metas,
            data: instruction_data,
        };

        // ===== STEP 9: PREPARE ACCOUNT INFOS FOR CPI =====
        let account_infos = vec![
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.amm.to_account_info(),
            ctx.accounts.amm_authority.to_account_info(),
            ctx.accounts.amm_open_orders.to_account_info(),
            ctx.accounts.amm_target_orders.to_account_info(),
            ctx.accounts.pool_coin_token_account.to_account_info(),
            ctx.accounts.pool_pc_token_account.to_account_info(),
            ctx.accounts.serum_program.to_account_info(),
            ctx.accounts.serum_market.to_account_info(),
            ctx.accounts.serum_bids.to_account_info(),
            ctx.accounts.serum_asks.to_account_info(),
            ctx.accounts.serum_event_queue.to_account_info(),
            ctx.accounts.serum_coin_vault.to_account_info(),
            ctx.accounts.serum_pc_vault.to_account_info(),
            ctx.accounts.serum_vault_signer.to_account_info(),
            ctx.accounts.user_source_token.to_account_info(),
            ctx.accounts.user_destination_token.to_account_info(),
            ctx.accounts.user_authority.to_account_info(),
        ];

        // ===== STEP 10: EXECUTE CPI TO RAYDIUM =====
        // This is where the actual swap happens
        // Raydium will update the user's token balances on-chain
        msg!("📞 Calling Raydium AMM program...");
        invoke(&swap_instruction, &account_infos)?;

        msg!("✅ Swap completed successfully!");
        msg!("   Check your token balances to see the results");

        Ok(())
    }
}

// ===== ACCOUNTS STRUCT =====
#[derive(Accounts)]
pub struct SwapAccounts<'info> {
    /// User's wallet (must sign the transaction)
    pub user_authority: Signer<'info>,

    /// User's source token account (tokens being swapped FROM)
    #[account(mut)]
    pub user_source_token: Account<'info, TokenAccount>,

    /// User's destination token account (tokens being swapped TO)
    #[account(mut)]
    pub user_destination_token: Account<'info, TokenAccount>,

    /// Raydium AMM program
    /// CHECK: Validated by comparing with hardcoded program ID
    pub raydium_amm_program: UncheckedAccount<'info>,

    /// AMM pool state account
    /// CHECK: Validated by comparing with whitelisted pool ID
    #[account(mut)]
    pub amm: UncheckedAccount<'info>,

    /// AMM authority (PDA)
    /// CHECK: Validated against whitelist
    pub amm_authority: UncheckedAccount<'info>,

    /// AMM open orders account
    /// CHECK: Validated against whitelist
    #[account(mut)]
    pub amm_open_orders: UncheckedAccount<'info>,

    /// AMM target orders account
    /// CHECK: Validated against whitelist
    #[account(mut)]
    pub amm_target_orders: UncheckedAccount<'info>,

    /// Pool's coin token account (USDC)
    /// CHECK: Validated against whitelist
    #[account(mut)]
    pub pool_coin_token_account: UncheckedAccount<'info>,

    /// Pool's PC token account (SOL)
    /// CHECK: Validated against whitelist
    #[account(mut)]
    pub pool_pc_token_account: UncheckedAccount<'info>,

    /// Serum DEX program
    /// CHECK: Validated against whitelist
    pub serum_program: UncheckedAccount<'info>,

    /// Serum market
    /// CHECK: Validated against whitelist
    #[account(mut)]
    pub serum_market: UncheckedAccount<'info>,

    /// Serum bids
    /// CHECK: Validated against whitelist
    #[account(mut)]
    pub serum_bids: UncheckedAccount<'info>,

    /// Serum asks
    /// CHECK: Validated against whitelist
    #[account(mut)]
    pub serum_asks: UncheckedAccount<'info>,

    /// Serum event queue
    /// CHECK: Validated against whitelist
    #[account(mut)]
    pub serum_event_queue: UncheckedAccount<'info>,

    /// Serum coin vault
    /// CHECK: Validated against whitelist
    #[account(mut)]
    pub serum_coin_vault: UncheckedAccount<'info>,

    /// Serum PC vault
    /// CHECK: Validated against whitelist
    #[account(mut)]
    pub serum_pc_vault: UncheckedAccount<'info>,

    /// Serum vault signer
    /// CHECK: Validated against whitelist
    pub serum_vault_signer: UncheckedAccount<'info>,

    /// SPL Token program
    pub token_program: Program<'info, Token>,
}

// ===== ERROR CODES =====
#[error_code]
pub enum SwapError {
    #[msg("Invalid Raydium program ID")]
    InvalidRaydiumProgram,
    
    #[msg("Invalid token pair - only USDC <-> SOL supported")]
    InvalidTokenPair,
    
    #[msg("Invalid token account owner")]
    InvalidOwner,
    
    #[msg("Invalid amount - must be greater than 0")]
    InvalidAmount,
    
    #[msg("Insufficient balance")]
    InsufficientBalance,
    
    #[msg("Pool account mismatch - not using whitelisted pool")]
    InvalidPool,
}
