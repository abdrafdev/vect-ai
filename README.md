# 🪙 VECT.AI — Solana MVP (Phase 0)

Minimal, secure Solana smart contracts for automated trading based on price conditions.

## 📋 Overview

**Architecture**: `Fetch Price → Check Condition → Execute Swap`

Three core programs:
- **vectai_token**: Standard SPL token (mint, transfer, balance)
- **vectai_oracle**: Lightweight Pyth price feed reader  
- **vectai_trader**: Conditional swap executor via Jupiter

## 🏗️ Project Structure

```
vectai-solana/
├── Cargo.toml
├── Anchor.toml
├── README.md
├── programs/
│   ├── vectai_token/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── vectai_oracle/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── vectai_trader/
│       ├── Cargo.toml
│       └── src/lib.rs
├── app/
│   ├── package.json
│   └── src/index.ts
└── tests/
    ├── token_tests.rs
    ├── oracle_tests.rs
    └── trader_tests.rs
```

## 🚀 Quick Start

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
```

### Build & Deploy
```bash
# Build all programs
anchor build

# Deploy to local validator
solana-test-validator --reset &
anchor deploy

# Deploy to devnet  
anchor deploy --provider.cluster devnet
```

### Run Tests
```bash
# Run all tests
anchor test

# Run specific program tests
cargo test --package vectai_token
cargo test --package vectai_oracle  
cargo test --package vectai_trader
```

### Run Frontend
```bash
cd app
npm install
npm start
```

## 🔧 Programs

### vectai_token
Standard SPL token implementation:
- `initialize()` - Set mint authority and total supply
- `transfer()` - Transfer tokens between accounts
- Uses standard SPL token logic, no governance/vesting

### vectai_oracle  
Lightweight Pyth price reader:
- `get_price()` - Fetch current asset price
- Read-only operations, no state mutation
- Integrated with Pyth network for reliable data

### vectai_trader
Simple conditional swap executor:
- `initialize_trader()` - Set price threshold and swap parameters
- `execute_conditional_swap()` - Check price and execute Jupiter swap
- Follows Checks-Effects-Interactions pattern
- Whitelisted external calls only

## 🔒 Security Features

✅ **Implemented:**
- Checks-Effects-Interactions pattern
- Strict account validation
- Whitelisted external program calls
- No arbitrary function execution
- SafeMath operations where needed

❌ **Avoided:**  
- Governance mechanisms
- Complex strategy logic
- Arbitrary external calls
- Flash loan vulnerabilities

## 📊 Example Usage

```typescript
// Connect to devnet
const connection = new Connection(clusterApiUrl('devnet'), 'confirmed');

// Get price from oracle
const price = await program.methods
  .getPrice()
  .accounts({ priceFeed: PYTH_BTC_FEED })
  .rpc();

// Execute conditional swap
await program.methods
  .executeConditionalSwap()
  .accounts({
    trader: traderAccount,
    oracle: oracleAccount,
    // ... other accounts
  })
  .rpc();
```

## 🌐 Networks

- **Localnet**: Development and testing
- **Devnet**: Staging with Pyth testnet feeds
- **Mainnet**: Production with live Pyth feeds

## 📈 Pyth Price Feeds

Common feeds for testing:
```
BTC/USD: E62dniuSvzKH9QqfR4jCNZrsCwSKzD8cuQPvD2CWcUmC
ETH/USD: JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB
SOL/USD: H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG
```

## 🔧 Development

```bash
# Format code
cargo fmt

# Run linter  
cargo clippy -- -D warnings

# Clean build
anchor clean && anchor build

# Generate TypeScript client
anchor build && anchor ts
```