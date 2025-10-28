# ========================================
# VECT AI Solana Build Script
# ========================================
# This script ensures the correct Rust toolchain is used for building

Write-Host "🚀 VECT AI Solana Build Script" -ForegroundColor Cyan
Write-Host "=" * 50 -ForegroundColor Cyan

# Set HOME environment variable
$env:HOME = $HOME
Write-Host "✓ HOME set to: $env:HOME" -ForegroundColor Green

# Check Rust version
Write-Host "`n📦 Checking Rust version..." -ForegroundColor Yellow
$rustVersion = rustc --version
Write-Host "Current: $rustVersion" -ForegroundColor White

# Ensure we're using Rust 1.81
Write-Host "`n🔄 Setting Rust 1.81 (required for Solana + modern dependencies)..." -ForegroundColor Yellow
rustup default 1.81.0
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Rust 1.81 not found, installing..." -ForegroundColor Yellow
    rustup toolchain install 1.81.0
    rustup default 1.81.0
}
Write-Host "✓ Rust 1.81 active" -ForegroundColor Green

# Check Solana version
Write-Host "`n📦 Checking Solana version..." -ForegroundColor Yellow
$solanaVersion = solana --version 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Solana CLI not found. Installing 1.18.26..." -ForegroundColor Yellow
    solana-install init 1.18.26
} else {
    Write-Host "Current: $solanaVersion" -ForegroundColor White
}

# Check Anchor version
Write-Host "`n⚓ Checking Anchor version..." -ForegroundColor Yellow
$anchorVersion = anchor --version 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Anchor not found! Please install: cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked --force" -ForegroundColor Red
    exit 1
} else {
    Write-Host "Current: $anchorVersion" -ForegroundColor White
}

# Clean old build artifacts
Write-Host "`n🧹 Cleaning old build artifacts..." -ForegroundColor Yellow
if (Test-Path "target") {
    Remove-Item -Path "target\deploy\*.so" -Force -ErrorAction SilentlyContinue
    Write-Host "✓ Cleaned deploy folder" -ForegroundColor Green
}

# Remove problematic Cargo.lock if it exists
if (Test-Path "Cargo.lock") {
    $lockVersion = (Get-Content "Cargo.lock" | Select-String "^version = " | Select-Object -First 1).ToString()
    if ($lockVersion -match "version = 4") {
        Write-Host "⚠️  Removing incompatible Cargo.lock (version 4)..." -ForegroundColor Yellow
        Remove-Item -Path "Cargo.lock" -Force
        Write-Host "✓ Removed" -ForegroundColor Green
    }
}

# Build all programs
Write-Host "`n🔨 Building all programs..." -ForegroundColor Yellow
Write-Host "=" * 50 -ForegroundColor Cyan

anchor build

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ Build successful!" -ForegroundColor Green
    Write-Host "=" * 50 -ForegroundColor Cyan
    
    # List built programs
    Write-Host "`n📦 Built programs:" -ForegroundColor Cyan
    if (Test-Path "target\deploy") {
        Get-ChildItem -Path "target\deploy" -Filter "*.so" | ForEach-Object {
            Write-Host "  • $($_.Name)" -ForegroundColor White
        }
    }
    
    Write-Host "`n🎯 Program IDs:" -ForegroundColor Cyan
    Get-ChildItem -Path "target\deploy" -Filter "*-keypair.json" | ForEach-Object {
        $programName = $_.BaseName -replace "-keypair", ""
        $programId = solana address -k $_.FullName 2>$null
        if ($programId) {
            Write-Host "  • $programName`: $programId" -ForegroundColor White
        }
    }
    
    Write-Host "`n💡 Next steps:" -ForegroundColor Yellow
    Write-Host "  1. Update program IDs in Anchor.toml and declare_id!() in each lib.rs" -ForegroundColor White
    Write-Host "  2. Deploy: anchor deploy --provider.cluster devnet" -ForegroundColor White
    Write-Host "  3. Test: anchor test" -ForegroundColor White
} else {
    Write-Host "`n❌ Build failed!" -ForegroundColor Red
    Write-Host "=" * 50 -ForegroundColor Red
    Write-Host "`n🔍 Troubleshooting tips:" -ForegroundColor Yellow
    Write-Host "  1. Ensure Rust 1.81 is active: rustup default 1.81.0" -ForegroundColor White
    Write-Host "  2. Clean and rebuild: rm -Recurse -Force target, Cargo.lock" -ForegroundColor White
    Write-Host "  3. Update dependencies: cargo update" -ForegroundColor White
    exit 1
}
