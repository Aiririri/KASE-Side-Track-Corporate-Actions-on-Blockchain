pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("CNbAp4fpVhCP6TQ1uPkJv5VZHVZbffM2gvyVXFNJXYbX");

#[program]
pub mod kase_corporate_actions {
    use super::*;

    pub fn initialize_bond(
        ctx: Context<InitializeBond>,
        series_id: u64,
        face_value: u64,
        coupon_rate_bps: u16,
        coupons_per_year: u8,
        maturity_ts: i64,
    ) -> Result<()> {
        crate::instructions::initialize_bond::handle_initialize_bond(
            ctx,
            series_id,
            face_value,
            coupon_rate_bps,
            coupons_per_year,
            maturity_ts,
        )
    }
}
