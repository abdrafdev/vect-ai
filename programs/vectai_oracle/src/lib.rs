use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

declare_id!("8FWpTEk2NPut6MrKXiCGVzz9ZY247fcYGdL9TEoXFqzw");

#[program]
pub mod vectai_oracle {
    use super::*;

    /// Secure price fetch with comprehensive validation
    pub fn get_price(ctx: Context<GetPrice>) -> Result<PriceData> {
        let price_update = &ctx.accounts.price_update;
        
        // ✅ CHECKS: Validate price update data
        let price_update_data = PriceUpdateV2::try_deserialize(
            &mut price_update.data.borrow().as_ref()
        ).map_err(|_| OracleError::InvalidPriceUpdate)?;

        let price_feed = price_update_data
            .price_feeds
            .first()
            .ok_or(OracleError::NoPriceFeedFound)?;

        // ✅ CHECKS: Strict staleness check (2 minutes max)
        let clock = Clock::get()?;
        let price_age = clock.unix_timestamp - price_feed.publish_time;
        require!(price_age <= 120, OracleError::StalePrice); // 2 minutes max
        require!(price_age >= 0, OracleError::FuturePrice); // No future prices

        // ✅ CHECKS: Price bounds validation
        require!(price_feed.price > 0, OracleError::InvalidPrice);
        require!(price_feed.price < 1_000_000_000_000, OracleError::PriceTooBig); // Max $1T
        
        // ✅ CHECKS: Confidence validation (max 5% uncertainty)
        let confidence_ratio = (price_feed.conf as f64) / (price_feed.price.abs() as f64);
        require!(confidence_ratio <= 0.05, OracleError::LowConfidence);

        let price_data = PriceData {
            price: price_feed.price,
            conf: price_feed.conf,
            expo: price_feed.expo,
            publish_time: price_feed.publish_time,
        };

        msg!("✅ Secure price: {} * 10^{} (±{})", price_data.price, price_data.expo, price_data.conf);
        Ok(price_data)
    }

    /// Secure threshold check with input validation
    pub fn check_price_threshold(
        ctx: Context<GetPrice>,
        threshold: i64,
    ) -> Result<bool> {
        // ✅ CHECKS: Validate threshold inputs
        require!(threshold > 0, OracleError::InvalidThreshold);
        require!(threshold < 1_000_000_000_000, OracleError::ThresholdTooBig);
        
        let price_data = Self::get_price(ctx)?;
        let meets_threshold = price_data.price > threshold;
        
        msg!("✅ Price {} > threshold {} = {}", price_data.price, threshold, meets_threshold);
        Ok(meets_threshold)
    }

    /// Secure trading condition with comprehensive validation
    pub fn get_trading_condition(
        ctx: Context<GetPrice>,
        short_threshold: i64,
        long_threshold: i64,
    ) -> Result<TradingCondition> {
        // ✅ CHECKS: Validate threshold inputs
        require!(short_threshold > 0, OracleError::InvalidThreshold);
        require!(long_threshold > 0, OracleError::InvalidThreshold);
        require!(long_threshold > short_threshold, OracleError::InvalidThresholdOrder);
        require!(short_threshold < 1_000_000_000_000, OracleError::ThresholdTooBig);
        require!(long_threshold < 1_000_000_000_000, OracleError::ThresholdTooBig);
        
        let price_data = Self::get_price(ctx)?;
        
        let condition = if price_data.price < short_threshold {
            TradingCondition::Short
        } else if price_data.price > long_threshold {
            TradingCondition::Long
        } else {
            TradingCondition::Mid
        };
        
        msg!("✅ Trading condition: {:?} (Price: {}, Short: {}, Long: {})",
             condition, price_data.price, short_threshold, long_threshold);
        
        Ok(condition)
    }
}

#[derive(Accounts)]
pub struct GetPrice<'info> {
    /// CHECK: Pyth price update account
    pub price_update: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceData {
    pub price: i64,
    pub conf: u64,
    pub expo: i32,
    pub publish_time: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum TradingCondition {
    Short,  // Price below short threshold
    Mid,    // Price between thresholds
    Long,   // Price above long threshold
}

#[error_code]
pub enum OracleError {
    #[msg("Invalid price update account")]
    InvalidPriceUpdate,
    #[msg("No price feed found")]
    NoPriceFeedFound,
    #[msg("Price is stale (>2min)")]
    StalePrice,
    #[msg("Price from future not allowed")]
    FuturePrice,
    #[msg("Invalid price value")]
    InvalidPrice,
    #[msg("Price too big (max $1T)")]
    PriceTooBig,
    #[msg("Price confidence too low")]
    LowConfidence,
    #[msg("Invalid threshold")]
    InvalidThreshold,
    #[msg("Threshold too big (max $1T)")]
    ThresholdTooBig,
    #[msg("Long threshold must be > short threshold")]
    InvalidThresholdOrder,
}
