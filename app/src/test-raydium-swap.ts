import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";

/**
 * Test script for Raydium swap integration
 * 
 * This script demonstrates how to:
 * 1. Initialize a trader configuration
 * 2. Execute a swap through Raydium
 * 3. Verify balance changes
 */

// ===== CONSTANTS =====

// Token mints (Devnet)
const USDT_MINT = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"); // Devnet USDC (proxy)
const WSOL_MINT = new PublicKey("So11111111111111111111111111111111111111112");

// Programs
const RAYDIUM_AMM_PROGRAM = new PublicKey("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
const VECTAI_TRADER_PROGRAM = new PublicKey("FEmf6TbtffcKVptbshZvCcg3CjQqsWodNwQhpXJff4NP");
const VECTAI_ORACLE_PROGRAM = new PublicKey("8FWpTEk2NPut6MrKXiCGVzz9ZY247fcYGdL9TEoXFqzw");

// Pyth price feed (Devnet BTC/USD)
const PYTH_BTC_USD_FEED = new PublicKey("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J");

// ===== RAYDIUM POOL ACCOUNTS =====
// TODO: Replace these with actual pool accounts from Raydium API
// You can fetch these using Raydium SDK or from their API
const POOL_ACCOUNTS = {
  // These are placeholders - you must fetch real accounts for your pool
  amm: new PublicKey("11111111111111111111111111111111"), // Replace
  ammAuthority: new PublicKey("11111111111111111111111111111111"), // Replace
  ammOpenOrders: new PublicKey("11111111111111111111111111111111"), // Replace
  ammTargetOrders: new PublicKey("11111111111111111111111111111111"), // Replace
  poolCoinTokenAccount: new PublicKey("11111111111111111111111111111111"), // Replace
  poolPcTokenAccount: new PublicKey("11111111111111111111111111111111"), // Replace
  serumProgram: new PublicKey("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin"), // Serum DEX v3
  serumMarket: new PublicKey("11111111111111111111111111111111"), // Replace
  serumBids: new PublicKey("11111111111111111111111111111111"), // Replace
  serumAsks: new PublicKey("11111111111111111111111111111111"), // Replace
  serumEventQueue: new PublicKey("11111111111111111111111111111111"), // Replace
  serumCoinVaultAccount: new PublicKey("11111111111111111111111111111111"), // Replace
  serumPcVaultAccount: new PublicKey("11111111111111111111111111111111"), // Replace
  serumVaultSigner: new PublicKey("11111111111111111111111111111111"), // Replace
};

async function main() {
  console.log("🚀 VECT.AI Raydium Swap Test\n");

  // ===== SETUP =====
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Load the program
  const program = anchor.workspace.VectaiTrader as Program;
  console.log("📦 Program ID:", program.programId.toBase58());
  console.log("👤 Wallet:", provider.wallet.publicKey.toBase58());
  console.log();

  // ===== STEP 1: Derive trader config PDA =====
  const [traderConfigPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("trader"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );
  console.log("📝 Trader Config PDA:", traderConfigPDA.toBase58());

  // ===== STEP 2: Get token accounts =====
  const usdtTokenAccount = await getAssociatedTokenAddress(
    USDT_MINT,
    provider.wallet.publicKey
  );
  const wsolTokenAccount = await getAssociatedTokenAddress(
    WSOL_MINT,
    provider.wallet.publicKey
  );

  console.log("💰 USDT Token Account:", usdtTokenAccount.toBase58());
  console.log("💰 wSOL Token Account:", wsolTokenAccount.toBase58());
  console.log();

  // ===== STEP 3: Initialize trader (if not already initialized) =====
  try {
    console.log("🔧 Initializing trader configuration...");
    
    const initTx = await program.methods
      .initializeTrader(
        new anchor.BN(40000), // Price threshold: $40,000
        new anchor.BN(1000000), // Swap amount: 1 USDT (6 decimals)
        200 // Slippage tolerance: 2% (200 basis points)
      )
      .accounts({
        traderConfig: traderConfigPDA,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("✅ Trader initialized!");
    console.log("   TX:", initTx);
    console.log();
  } catch (err) {
    console.log("⚠️  Trader already initialized or error:", err.message);
    console.log();
  }

  // ===== STEP 4: Check balances before swap =====
  console.log("📊 Checking balances before swap...");
  
  const connection = provider.connection;
  const usdtBalanceBefore = await connection.getTokenAccountBalance(usdtTokenAccount);
  const wsolBalanceBefore = await connection.getTokenAccountBalance(wsolTokenAccount);
  
  console.log("   USDT Balance:", usdtBalanceBefore.value.uiAmount);
  console.log("   wSOL Balance:", wsolBalanceBefore.value.uiAmount);
  console.log();

  // ===== STEP 5: Execute swap =====
  console.log("🔄 Executing swap through Raydium...");
  console.log("⚠️  NOTE: Make sure POOL_ACCOUNTS are configured with real addresses!");
  console.log();

  try {
    const swapTx = await program.methods
      .executeTrade(new anchor.BN(1000000)) // 1 USDT
      .accounts({
        userAuthority: provider.wallet.publicKey,
        traderConfig: traderConfigPDA,
        userSourceTokenAccount: usdtTokenAccount,
        userDestinationTokenAccount: wsolTokenAccount,
        
        // Raydium AMM accounts
        raydiumAmmProgram: RAYDIUM_AMM_PROGRAM,
        amm: POOL_ACCOUNTS.amm,
        ammAuthority: POOL_ACCOUNTS.ammAuthority,
        ammOpenOrders: POOL_ACCOUNTS.ammOpenOrders,
        ammTargetOrders: POOL_ACCOUNTS.ammTargetOrders,
        poolCoinTokenAccount: POOL_ACCOUNTS.poolCoinTokenAccount,
        poolPcTokenAccount: POOL_ACCOUNTS.poolPcTokenAccount,
        
        // Serum market accounts
        serumProgram: POOL_ACCOUNTS.serumProgram,
        serumMarket: POOL_ACCOUNTS.serumMarket,
        serumBids: POOL_ACCOUNTS.serumBids,
        serumAsks: POOL_ACCOUNTS.serumAsks,
        serumEventQueue: POOL_ACCOUNTS.serumEventQueue,
        serumCoinVaultAccount: POOL_ACCOUNTS.serumCoinVaultAccount,
        serumPcVaultAccount: POOL_ACCOUNTS.serumPcVaultAccount,
        serumVaultSigner: POOL_ACCOUNTS.serumVaultSigner,
        
        // Oracle accounts
        vectaiOracleProgram: VECTAI_ORACLE_PROGRAM,
        priceFeed: PYTH_BTC_USD_FEED,
        
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("✅ Swap executed successfully!");
    console.log("   TX:", swapTx);
    console.log();

    // ===== STEP 6: Check balances after swap =====
    console.log("📊 Checking balances after swap...");
    
    // Wait a bit for transaction to settle
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    const usdtBalanceAfter = await connection.getTokenAccountBalance(usdtTokenAccount);
    const wsolBalanceAfter = await connection.getTokenAccountBalance(wsolTokenAccount);
    
    console.log("   USDT Balance:", usdtBalanceAfter.value.uiAmount);
    console.log("   wSOL Balance:", wsolBalanceAfter.value.uiAmount);
    console.log();

    // Calculate differences
    const usdtDiff = usdtBalanceAfter.value.uiAmount! - usdtBalanceBefore.value.uiAmount!;
    const wsolDiff = wsolBalanceAfter.value.uiAmount! - wsolBalanceBefore.value.uiAmount!;
    
    console.log("💱 Swap results:");
    console.log("   USDT change:", usdtDiff, "USDT");
    console.log("   wSOL change:", wsolDiff, "SOL");
    console.log();

  } catch (err) {
    console.error("❌ Swap failed:", err);
    
    if (err.logs) {
      console.log("\n📜 Transaction logs:");
      err.logs.forEach((log: string) => console.log("  ", log));
    }
  }

  console.log("🏁 Test completed!");
}

// ===== HELPER FUNCTION: Fetch Raydium pool accounts =====
async function fetchRaydiumPoolAccounts(
  poolId: PublicKey
): Promise<typeof POOL_ACCOUNTS> {
  console.log("🔍 Fetching Raydium pool accounts for:", poolId.toBase58());
  
  // TODO: Implement this using Raydium SDK or API
  // For now, return placeholder accounts
  
  console.log("⚠️  Using placeholder accounts - replace with real ones!");
  
  return POOL_ACCOUNTS;
}

// Run the test
main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
