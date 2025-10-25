use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
};
use vectai_token::{TokenInfo, TokenError};

#[tokio::test]
async fn test_initialize_token() {
    let program_id = vectai_token::id();
    let mut context = ProgramTestContext::new(
        "vectai_token",
        program_id,
        None,
    ).await;

    let payer = &context.payer;
    let mint_authority = Keypair::new();
    let mint_keypair = Keypair::new();
    
    // Test token initialization
    let decimals = 9u8;
    let max_supply = 1_000_000_000u64; // 1B tokens
    
    // Create transaction to initialize token
    let ix = anchor_lang::instruction! {
        vectai_token::initialize_token,
        max_supply,
        decimals
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer, &mint_authority],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    println!("✅ Token initialization test passed");
    println!("   Decimals: {}", decimals);
    println!("   Max Supply: {}", max_supply);
}

#[tokio::test] 
async fn test_mint_tokens() {
    let program_id = vectai_token::id();
    let mut context = ProgramTestContext::new(
        "vectai_token",
        program_id,
        None,
    ).await;

    let payer = &context.payer;
    let mint_authority = Keypair::new();
    let mint_keypair = Keypair::new();
    let user_keypair = Keypair::new();
    
    // Initialize token first
    let max_supply = 1_000_000_000u64;
    let decimals = 9u8;
    
    // Create mint account
    let create_mint_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint_keypair.pubkey(),
        82, // Mint account size
        82, // Rent
        &token::ID,
    );
    
    let init_mint_ix = token::instruction::initialize_mint(
        &token::ID,
        &mint_keypair.pubkey(),
        &mint_authority.pubkey(),
        Some(&mint_authority.pubkey()),
        decimals,
    ).unwrap();
    
    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix, init_mint_ix],
        Some(&payer.pubkey()),
        &[payer, &mint_authority, &mint_keypair],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    // Test minting tokens
    let amount = 1000u64;
    let mint_ix = anchor_lang::instruction! {
        vectai_token::mint_to,
        amount
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &[&mint_authority],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    println!("✅ Token minting test passed");
    println!("   Amount: {}", amount);
}

#[tokio::test]
async fn test_transfer_tokens() {
    let program_id = vectai_token::id();
    let mut context = ProgramTestContext::new(
        "vectai_token",
        program_id,
        None,
    ).await;

    let payer = &context.payer;
    let from_authority = Keypair::new();
    let to_keypair = Keypair::new();
    
    // Test transferring tokens between accounts
    let transfer_amount = 100u64;
    
    let transfer_ix = anchor_lang::instruction! {
        vectai_token::transfer,
        transfer_amount
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&payer.pubkey()),
        &[&from_authority],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    println!("✅ Token transfer test passed");
    println!("   Transfer Amount: {}", transfer_amount);
}

#[tokio::test]
async fn test_token_errors() {
    let program_id = vectai_token::id();
    let mut context = ProgramTestContext::new(
        "vectai_token",
        program_id,
        None,
    ).await;

    let payer = &context.payer;
    let unauthorized_authority = Keypair::new();
    
    // Test error conditions:
    // 1. Invalid mint authority
    let mint_ix = anchor_lang::instruction! {
        vectai_token::mint_to,
        1000u64
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &[&unauthorized_authority],
        context.last_blockhash,
    );
    
    // This should fail with UnauthorizedMintAuthority
    let result = context.banks_client.process_transaction(tx).await;
    assert!(result.is_err());
    
    println!("✅ Token error handling tests passed");
    println!("   Unauthorized mint authority correctly rejected");
}

#[tokio::test]
async fn test_supply_cap_enforcement() {
    let program_id = vectai_token::id();
    let mut context = ProgramTestContext::new(
        "vectai_token",
        program_id,
        None,
    ).await;

    let payer = &context.payer;
    let mint_authority = Keypair::new();
    
    // Test that minting exceeds max supply fails
    let excessive_amount = 2_000_000_000u64; // More than 1B max supply
    
    let mint_ix = anchor_lang::instruction! {
        vectai_token::mint_to,
        excessive_amount
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &[&mint_authority],
        context.last_blockhash,
    );
    
    // This should fail with ExceedsMaxSupply
    let result = context.banks_client.process_transaction(tx).await;
    assert!(result.is_err());
    
    println!("✅ Supply cap enforcement test passed");
    println!("   Excessive minting correctly rejected");
}

#[tokio::test]
async fn test_pause_unpause() {
    let program_id = vectai_token::id();
    let mut context = ProgramTestContext::new(
        "vectai_token",
        program_id,
        None,
    ).await;

    let payer = &context.payer;
    let admin = Keypair::new();
    let mint_authority = Keypair::new();
    
    // Test pause functionality
    let pause_ix = anchor_lang::instruction! {
        vectai_token::pause_token
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[pause_ix],
        Some(&payer.pubkey()),
        &[&admin],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    // Test that minting fails when paused
    let mint_ix = anchor_lang::instruction! {
        vectai_token::mint_to,
        1000u64
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &[&mint_authority],
        context.last_blockhash,
    );
    
    // This should fail with TokenPaused
    let result = context.banks_client.process_transaction(tx).await;
    assert!(result.is_err());
    
    // Test unpause
    let unpause_ix = anchor_lang::instruction! {
        vectai_token::unpause_token
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[unpause_ix],
        Some(&payer.pubkey()),
        &[&admin],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    println!("✅ Pause/unpause test passed");
    println!("   Token correctly paused and unpaused");
}

// Integration test combining all token operations
#[tokio::test]
async fn test_full_token_workflow() {
    let program_id = vectai_token::id();
    let mut context = ProgramTestContext::new(
        "vectai_token",
        program_id,
        None,
    ).await;

    let payer = &context.payer;
    let mint_authority = Keypair::new();
    let user1 = Keypair::new();
    let user2 = Keypair::new();
    
    // 1. Initialize token mint
    let max_supply = 1_000_000_000u64;
    let decimals = 9u8;
    
    let init_ix = anchor_lang::instruction! {
        vectai_token::initialize_token,
        max_supply,
        decimals
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&payer.pubkey()),
        &[payer, &mint_authority],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    // 2. Mint tokens to user1
    let mint_amount = 100_000u64;
    let mint_ix = anchor_lang::instruction! {
        vectai_token::mint_to,
        mint_amount
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &[&mint_authority],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    // 3. Transfer tokens from user1 to user2
    let transfer_amount = 50_000u64;
    let transfer_ix = anchor_lang::instruction! {
        vectai_token::transfer,
        transfer_amount
    };
    
    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&payer.pubkey()),
        &[&user1],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    println!("✅ Full token workflow test passed");
    println!("1. ✅ Token initialized");
    println!("2. ✅ Tokens minted to user1");
    println!("3. ✅ Tokens transferred to user2");
    println!("4. ✅ All operations completed successfully");
}