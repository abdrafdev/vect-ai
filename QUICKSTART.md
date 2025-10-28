# Quick Start Guide - VECT AI Solana

## ✅ Everything is Connected!

All 4 programs are properly integrated in the vectai-solana workspace:
- ✅ vectai_token
- ✅ vectai_oracle  
- ✅ vectai_trader
- ✅ raydium_swapper (NEW!)

## 🚀 Build in 30 Seconds

### Step 1: Run the Build Script
```powershell
cd "C:\Users\Aman Qureshi\Desktop\VECT AI\vectai-solana"
.\build.ps1
```

That's it! The script will:
- ✅ Set Rust 1.81.0 (correct version)
- ✅ Verify Solana CLI
- ✅ Verify Anchor CLI
- ✅ Build all 4 programs
- ✅ Show program IDs

### Step 2: Update Program IDs (First Build Only)

After the first build, update the raydium_swapper program ID:

1. Get the new program ID from build output
2. Update in `Anchor.toml`:
   ```toml
   raydium_swapper = "<NEW_PROGRAM_ID>"
   ```
3. Update in `programs/raydium_swapper/src/lib.rs`:
   ```rust
   declare_id!("<NEW_PROGRAM_ID>");
   ```
4. Rebuild: `.\build.ps1`

### Step 3: Deploy to Devnet
```powershell
anchor deploy --provider.cluster devnet
```

## 🎯 What Each Program Does

### 1. vectai_token
Token management and minting

### 2. vectai_oracle
Price feed integration (Pyth)

### 3. vectai_trader
Trading logic and strategy execution

### 4. raydium_swapper (NEW!)
**On-chain token swaps via Raydium AMM**
- Swaps USDC ↔ wSOL on Solana devnet
- Uses hardcoded Raydium pool addresses
- Enforces slippage protection
- Validates token ownership and balances

## 📝 Key Files

```
vectai-solana/
├── build.ps1              ← Run this to build everything
├── INTEGRATION_STATUS.md  ← Full integration details
├── QUICKSTART.md         ← This file
├── Cargo.toml            ← Workspace config (4 programs)
├── Anchor.toml           ← Anchor config (program IDs)
└── programs/
    ├── vectai_token/
    ├── vectai_oracle/
    ├── vectai_trader/
    └── raydium_swapper/  ← NEW! Fully integrated
        ├── Cargo.toml
        └── src/lib.rs
```

## ⚠️ Common Issues

### "Cannot find HOME directory"
```powershell
$env:HOME = $HOME
.\build.ps1
```

### "edition2024 required"
```powershell
rustup default 1.81.0
Remove-Item Cargo.lock -Force
.\build.ps1
```

### Build hangs
```powershell
taskkill /F /IM cargo.exe
Remove-Item -Recurse -Force target
.\build.ps1
```

## 📚 More Info

- **Full Integration Details**: See `INTEGRATION_STATUS.md`
- **Test Swaps**: You'll need devnet USDC and wSOL tokens
- **Pool Addresses**: Hardcoded for USDC/SOL devnet pool in `lib.rs`

## ✅ Verification Checklist

Before deploying:
- [ ] Build succeeds: `.\build.ps1`
- [ ] All 4 `.so` files in `target/deploy/`
- [ ] Program IDs updated in code and config
- [ ] Rebuilt after updating IDs

## 🎉 Success!

If `.\build.ps1` completes without errors:
→ **Everything is working perfectly!** 🚀

You can now deploy to devnet and test the Raydium swapper.
