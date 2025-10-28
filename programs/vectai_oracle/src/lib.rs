// Simplified oracle for Raydium swap testing
// This removes Pyth dependencies to avoid SDK version conflicts
// For production, integrate properly with Pyth after resolving dependencies

use anchor_lang::prelude::*;

declare_id!("8FWpTEk2NPut6MrKXiCGVzz9ZY247fcYGdL9TEoXFqzw");

#[program]
pub mod vectai_oracle {
    use super::*;

    /// Mock price fetch - returns a fixed price for testing
    /// In production, this would fetch from Pyth price feeds
    pub fn get_price(_ctx: Context<GetPrice>) -> Result<PriceData> {
        msg!("⚠️  Using mock price data for testing");
        
        // Mock BTC price: $45,000
        let price_data = PriceData {
            price: 45000,
            conf: 100,
            expo: 0,
            publish_time: Clock::get()?.unix_timestamp,
        };
        
        msg!("📊 Mock price: ${}", price_data.price);
        Ok(price_data)
    }
}

#[derive(Accounts)]
pub struct GetPrice<'info> {
    /// CHECK: Price feed account (unused in mock)
    pub price_feed: UncheckedAccount<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceData {
    pub price: i64,
    pub conf: u64,
    pub expo: i32,
    pub publish_time: i64,
}
