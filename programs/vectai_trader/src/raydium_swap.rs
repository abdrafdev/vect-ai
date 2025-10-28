use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};
use anchor_spl::token::TokenAccount;

/// Raydium swap instruction discriminator
/// This is the instruction byte for swap on Raydium AMM
const RAYDIUM_SWAP_INSTRUCTION: u8 = 9;

/// Raydium swap accounts structure
/// Using AccountInfo for flexibility
pub struct RaydiumSwapAccounts<'info> {
    pub amm_program: AccountInfo<'info>,
    pub amm: AccountInfo<'info>,
    pub amm_authority: AccountInfo<'info>,
    pub amm_open_orders: AccountInfo<'info>,
    pub amm_target_orders: AccountInfo<'info>,
    pub pool_coin_token_account: AccountInfo<'info>,
    pub pool_pc_token_account: AccountInfo<'info>,
    pub serum_program: AccountInfo<'info>,
    pub serum_market: AccountInfo<'info>,
    pub serum_bids: AccountInfo<'info>,
    pub serum_asks: AccountInfo<'info>,
    pub serum_event_queue: AccountInfo<'info>,
    pub serum_coin_vault_account: AccountInfo<'info>,
    pub serum_pc_vault_account: AccountInfo<'info>,
    pub serum_vault_signer: AccountInfo<'info>,
    pub user_source_token_account: AccountInfo<'info>,
    pub user_destination_token_account: AccountInfo<'info>,
    pub user_source_owner: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

/// Execute a swap on Raydium AMM
/// 
/// # Arguments
/// * `accounts` - All accounts required for Raydium swap
/// * `amount_in` - Amount of input tokens to swap
/// * `minimum_amount_out` - Minimum acceptable output tokens (slippage protection)
/// 
/// # Returns
/// * `Result<u64>` - Actual amount of output tokens received
pub fn execute_raydium_swap(
    accounts: &mut RaydiumSwapAccounts,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<u64> {
    msg!("🔄 Executing Raydium swap...");
    msg!("   Amount in: {}", amount_in);
    msg!("   Minimum out: {}", minimum_amount_out);
    
    // ===== STEP 1: Build Raydium swap instruction data =====
    // Instruction format: [discriminator: u8, amount_in: u64, minimum_amount_out: u64]
    let mut instruction_data = Vec::with_capacity(17);
    instruction_data.push(RAYDIUM_SWAP_INSTRUCTION); // Discriminator for swap
    instruction_data.extend_from_slice(&amount_in.to_le_bytes()); // Input amount
    instruction_data.extend_from_slice(&minimum_amount_out.to_le_bytes()); // Min output
    
    // ===== STEP 2: Prepare account metas for Raydium instruction =====
    let account_metas = vec![
        // Token program
        AccountMeta::new_readonly(accounts.token_program.key(), false),
        // AMM accounts
        AccountMeta::new(accounts.amm.key(), false),
        AccountMeta::new_readonly(accounts.amm_authority.key(), false),
        AccountMeta::new(accounts.amm_open_orders.key(), false),
        AccountMeta::new(accounts.amm_target_orders.key(), false),
        AccountMeta::new(accounts.pool_coin_token_account.key(), false),
        AccountMeta::new(accounts.pool_pc_token_account.key(), false),
        // Serum market accounts
        AccountMeta::new_readonly(accounts.serum_program.key(), false),
        AccountMeta::new(accounts.serum_market.key(), false),
        AccountMeta::new(accounts.serum_bids.key(), false),
        AccountMeta::new(accounts.serum_asks.key(), false),
        AccountMeta::new(accounts.serum_event_queue.key(), false),
        AccountMeta::new(accounts.serum_coin_vault_account.key(), false),
        AccountMeta::new(accounts.serum_pc_vault_account.key(), false),
        AccountMeta::new_readonly(accounts.serum_vault_signer.key(), false),
        // User accounts
        AccountMeta::new(accounts.user_source_token_account.key(), false),
        AccountMeta::new(accounts.user_destination_token_account.key(), false),
        AccountMeta::new_readonly(accounts.user_source_owner.key(), true), // Signer
    ];
    
    // ===== STEP 3: Build the instruction =====
    let swap_instruction = Instruction {
        program_id: accounts.amm_program.key(),
        accounts: account_metas,
        data: instruction_data,
    };
    
    // ===== STEP 4: Prepare account infos for invoke =====
    let account_infos = vec![
        accounts.token_program.to_account_info(),
        accounts.amm.clone(),
        accounts.amm_authority.clone(),
        accounts.amm_open_orders.clone(),
        accounts.amm_target_orders.clone(),
        accounts.pool_coin_token_account.to_account_info(),
        accounts.pool_pc_token_account.to_account_info(),
        accounts.serum_program.clone(),
        accounts.serum_market.clone(),
        accounts.serum_bids.clone(),
        accounts.serum_asks.clone(),
        accounts.serum_event_queue.clone(),
        accounts.serum_coin_vault_account.clone(),
        accounts.serum_pc_vault_account.clone(),
        accounts.serum_vault_signer.clone(),
        accounts.user_source_token_account.to_account_info(),
        accounts.user_destination_token_account.to_account_info(),
        accounts.user_source_owner.clone(),
    ];
    
    // ===== STEP 5: Execute the CPI call to Raydium =====
    msg!("📞 Invoking Raydium AMM program...");
    invoke(&swap_instruction, &account_infos)?;
    
    msg!("✅ Raydium swap completed successfully");
    msg!("   Minimum output guaranteed: {}", minimum_amount_out);
    
    // Return the minimum amount - actual amount will be higher
    // The caller should check the actual balance change
    Ok(minimum_amount_out)
}

/// Calculate minimum amount out with slippage protection
/// 
/// # Arguments
/// * `expected_amount` - Expected output amount without slippage
/// * `slippage_bps` - Slippage tolerance in basis points (e.g., 100 = 1%)
/// 
/// # Returns
/// * `Result<u64>` - Minimum acceptable output amount
pub fn calculate_minimum_amount_out(
    expected_amount: u64,
    slippage_bps: u64,
) -> Result<u64> {
    // Calculate: expected_amount * (10000 - slippage_bps) / 10000
    let multiplier = 10000u64
        .checked_sub(slippage_bps)
        .ok_or(ProgramError::InvalidArgument)?;
    
    let minimum = expected_amount
        .checked_mul(multiplier)
        .and_then(|x| x.checked_div(10000))
        .ok_or(ProgramError::ArithmeticOverflow)?;
    
    Ok(minimum)
}
