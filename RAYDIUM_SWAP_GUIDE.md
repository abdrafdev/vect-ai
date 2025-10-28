# 🔄 Raydium Swap Integration Guide

## Overview

This guide explains how to deploy and test the VECT.AI trader program with real Raydium swap functionality on Solana Devnet.

**What's Been Implemented:**
- ✅ Real Raydium AMM CPI calls (no simulation)
- ✅ Hardcoded USDT ↔ SOL pool support
- ✅ On-chain wallet balance updates
- ✅ Slippage protection
- ✅ Comprehensive validation

---

## 📋 Prerequisites

### 1. Install Required Tools

```bash
# Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Solana CLI v1.18+
sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"

# Anchor Framework v0.29+
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked

# Node.js v18+ (for testing scripts)
# Download from: https://nodejs.org/
```

### 2. Configure Solana CLI

```bash
# Set to Devnet
solana config set --url https://api.devnet.solana.com

# Create a new keypair (or use existing)
solana-keygen new --outfile ~/.config/solana/id.json

# Request airdrop for testing (2 SOL)
solana airdrop 2

# Check balance
solana balance
```

---

## 🏗️ Build & Deploy

### 1. Build the Program

```bash
cd "C:\Users\Aman Qureshi\Desktop\VECT AI\vectai-solana"

# Clean previous builds
anchor clean

# Build all programs
anchor build
```

**Expected Output:**
```
Building programs...
✔ vectai_token
✔ vectai_oracle
✔ vectai_trader
```

### 2. Deploy to Devnet

```bash
# Deploy all programs
anchor deploy --provider.cluster devnet
```

**Expected Output:**
```
Deploying cluster: devnet
Program Id: FEmf6TbtffcKVptbshZvCcg3CjQqsWodNwQhpXJff4NP (vectai_trader)
Program Id: 8FWpTEk2NPut6MrKXiCGVzz9ZY247fcYGdL9TEoXFqzw (vectai_oracle)
Program Id: DfpsT9PAeWbwwfE8EqTDqVUiCrsoHF1fogmPw42eqLPH (vectai_token)
```

### 3. Verify Deployment

```bash
# Check program exists on-chain
solana program show FEmf6TbtffcKVptbshZvCcg3CjQqsWodNwQhpXJff4NP
```

---

## 🔑 Token Mint Addresses

The program uses these hardcoded mint addresses:

### Wrapped SOL (wSOL)
```
So11111111111111111111111111111111111111112
```

### USDT (Devnet Proxy - using USDC)
```
4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU
```

> **Note:** Devnet doesn't have real USDT. We use Devnet USDC as a proxy for testing. On mainnet, use:
> `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`

### Raydium AMM Program
```
675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8
```

---

## 🧪 Testing the Swap

### Step 1: Find a USDT/SOL Pool on Raydium

Visit [Raydium Devnet](https://raydium.io/swap/) or use the Raydium SDK to find pool accounts:

```typescript
// Example pool accounts (you'll need to fetch these for your specific pool)
const USDT_SOL_POOL = {
  amm: "...",                    // AMM pool state account
  ammAuthority: "...",           // AMM authority PDA
  ammOpenOrders: "...",          // AMM open orders
  ammTargetOrders: "...",        // AMM target orders
  poolCoinTokenAccount: "...",   // Pool's USDT account
  poolPcTokenAccount: "...",     // Pool's SOL account
  serumProgram: "...",           // Serum DEX program
  serumMarket: "...",            // Serum market
  serumBids: "...",              // Serum bids
  serumAsks: "...",              // Serum asks
  serumEventQueue: "...",        // Serum event queue
  serumCoinVaultAccount: "...",  // Serum coin vault
  serumPcVaultAccount: "...",    // Serum pc vault
  serumVaultSigner: "..."        // Serum vault signer
};
```

> **Important:** These accounts are specific to each pool. You must fetch them from Raydium's API or on-chain data.

### Step 2: Create Token Accounts

```bash
# Create USDT token account for your wallet
spl-token create-account 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU

# Create wrapped SOL account (if not exists)
spl-token create-account So11111111111111111111111111111111111111112

# Get some test USDT from devnet faucet
# (You'll need to find a devnet USDC faucet)
```

### Step 3: Initialize Trader

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";

const program = anchor.workspace.VectaiTrader as Program<VectaiTrader>;
const provider = anchor.AnchorProvider.env();

// Initialize trader with conditions
await program.methods
  .initializeTrader(
    new anchor.BN(40000),  // Price threshold: $40,000
    new anchor.BN(1000000), // Swap amount: 1 USDT (6 decimals)
    200                     // Slippage: 2% (200 bps)
  )
  .accounts({
    traderConfig: traderConfigPDA,
    authority: provider.wallet.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();

console.log("✅ Trader initialized");
```

### Step 4: Execute Swap

```typescript
// Execute trade through Raydium
await program.methods
  .executeTrade(new anchor.BN(1000000)) // 1 USDT
  .accounts({
    userAuthority: provider.wallet.publicKey,
    traderConfig: traderConfigPDA,
    userSourceTokenAccount: usdtTokenAccount,
    userDestinationTokenAccount: wsolTokenAccount,
    
    // Raydium AMM accounts
    raydiumAmmProgram: RAYDIUM_AMM_PROGRAM,
    amm: poolAccounts.amm,
    ammAuthority: poolAccounts.ammAuthority,
    ammOpenOrders: poolAccounts.ammOpenOrders,
    ammTargetOrders: poolAccounts.ammTargetOrders,
    poolCoinTokenAccount: poolAccounts.poolCoinTokenAccount,
    poolPcTokenAccount: poolAccounts.poolPcTokenAccount,
    
    // Serum market accounts
    serumProgram: poolAccounts.serumProgram,
    serumMarket: poolAccounts.serumMarket,
    serumBids: poolAccounts.serumBids,
    serumAsks: poolAccounts.serumAsks,
    serumEventQueue: poolAccounts.serumEventQueue,
    serumCoinVaultAccount: poolAccounts.serumCoinVaultAccount,
    serumPcVaultAccount: poolAccounts.serumPcVaultAccount,
    serumVaultSigner: poolAccounts.serumVaultSigner,
    
    // Oracle accounts
    vectaiOracleProgram: VECTAI_ORACLE_PROGRAM,
    priceFeed: pythBtcUsdPriceFeed,
    
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();

console.log("✅ Swap executed!");
```

### Step 5: Verify Balance Changes

```bash
# Check USDT balance (should decrease)
spl-token balance 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU

# Check SOL balance (should increase)
spl-token balance So11111111111111111111111111111111111111112
```

---

## 📊 How It Works

### 1. **Validation Phase (CHECKS)**
```rust
// Validate Raydium program ID
require!(raydium_program == RAYDIUM_AMM_PROGRAM);

// Validate token pair (only USDT <-> SOL)
require!(
    (source == USDT && dest == SOL) ||
    (source == SOL && dest == USDT)
);

// Check price threshold via oracle
let price = vectai_oracle.get_price()?;
require!(price > threshold);
```

### 2. **State Update Phase (EFFECTS)**
```rust
// Update swap counter and timestamp
trader_config.total_swaps += 1;
trader_config.last_swap_time = clock.unix_timestamp;
```

### 3. **Interaction Phase (INTERACTIONS)**
```rust
// Execute Raydium CPI call
invoke(
    &raydium_swap_instruction,
    &accounts,
)?;

// User's wallet balances are updated automatically by Raydium
```

### Key Features:
- ✅ **Real swaps**: Uses actual Raydium liquidity pools
- ✅ **On-chain updates**: Wallet balances updated directly on-chain
- ✅ **No minting**: Exchanges existing tokens from pools
- ✅ **Slippage protection**: Minimum output amount enforced
- ✅ **CEI pattern**: Checks-Effects-Interactions for security

---

## 🔒 Security Features

1. **Hardcoded Token Validation**
   - Only USDT ↔ SOL swaps allowed
   - Prevents unauthorized token pairs

2. **Program ID Validation**
   - Verifies Raydium program ID
   - Prevents malicious program calls

3. **Slippage Protection**
   - Maximum 10% slippage allowed
   - User sets tolerance per trade

4. **Rate Limiting**
   - 1 minute cooldown between swaps
   - Prevents spam/exploitation

5. **Authorization Checks**
   - Only trader authority can execute
   - Oracle price must meet threshold

---

## 🐛 Troubleshooting

### Error: "Invalid Raydium program ID"
**Solution:** Ensure you're passing the correct Raydium AMM program:
```
675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8
```

### Error: "Invalid token pair"
**Solution:** Verify you're swapping between USDT and SOL only. Check token account mints.

### Error: "Price threshold not met"
**Solution:** Oracle price must exceed configured threshold. Lower threshold or wait for price movement.

### Error: "Rate limited"
**Solution:** Wait 60 seconds between swaps.

### Error: "Insufficient balance"
**Solution:** Ensure source token account has enough tokens for swap amount.

---

## 📚 Additional Resources

- [Raydium Documentation](https://docs.raydium.io/)
- [Anchor Framework](https://www.anchor-lang.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [SPL Token Guide](https://spl.solana.com/token)

---

## 🎯 Next Steps

1. **Production Deployment:**
   - Replace devnet USDC with mainnet USDT
   - Use mainnet Raydium pools
   - Implement proper admin key management

2. **Enhanced Features:**
   - Add more token pairs
   - Implement swap history storage
   - Add detailed event logs
   - Create frontend interface

3. **Optimization:**
   - Batch multiple swaps
   - Dynamic slippage calculation
   - Pool liquidity checks before swap

---

## ⚠️ Important Notes

1. **Devnet Testing Only**: This configuration uses devnet tokens. Don't use on mainnet without proper testing.

2. **Pool Accounts**: You must fetch real Raydium pool accounts for the USDT/SOL pair. The accounts vary by pool.

3. **Gas Fees**: Ensure your wallet has SOL for transaction fees (~0.01 SOL per transaction).

4. **No Simulation**: This implementation uses **real Raydium swaps**, not simulated minting.

5. **Wallet Updates**: User balances are updated **directly on-chain** by the Raydium program through CPI.

---

## 📝 Summary

You now have a **fully functional on-chain swap** implementation that:
- ✅ Uses real Raydium AMM pools
- ✅ Exchanges actual tokens (no minting)
- ✅ Updates wallet balances on-chain
- ✅ Includes slippage protection
- ✅ Validates USDT ↔ SOL pairs only
- ✅ Follows security best practices

**Ready to deploy to devnet and start testing!** 🚀
