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

    pub fn issue_bonds(ctx: Context<IssueBonds>, amount: u64) -> Result<()> {
        crate::instructions::issue_bonds::handle_issue_bonds(ctx, amount)
    }

    pub fn open_coupon_round(ctx: Context<OpenCouponRound>, record_ts: i64) -> Result<()> {
        crate::instructions::open_coupon_round::handle_open_coupon_round(ctx, record_ts)
    }

    pub fn freeze_holder(ctx: Context<FreezeHolder>) -> Result<()> {
        crate::instructions::freeze_holder::handle_freeze_holder(ctx)
    }

    pub fn finalize_record_date(ctx: Context<FinalizeRecordDate>) -> Result<()> {
        crate::instructions::finalize_record_date::handle_finalize_record_date(ctx)
    }

    pub fn fund_vault(ctx: Context<FundVault>, amount: u64) -> Result<()> {
        crate::instructions::fund_vault::handle_fund_vault(ctx, amount)
    }

    pub fn pay_coupon(ctx: Context<PayCoupon>) -> Result<()> {
        crate::instructions::pay_coupon::handle_pay_coupon(ctx)
    }

    pub fn thaw_holder(ctx: Context<ThawHolder>) -> Result<()> {
        crate::instructions::thaw_holder::handle_thaw_holder(ctx)
    }

    pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
        crate::instructions::redeem::handle_redeem(ctx)
    }

    pub fn schedule_partial_redemption(
        ctx: Context<SchedulePartialRedemption>,
        bps: u16,
    ) -> Result<()> {
        crate::instructions::schedule_partial_redemption::handle_schedule_partial_redemption(
            ctx, bps,
        )
    }

    pub fn pay_partial_redemption(ctx: Context<PayPartialRedemption>) -> Result<()> {
        crate::instructions::pay_partial_redemption::handle_pay_partial_redemption(ctx)
    }

    pub fn apply_partial_redemption(ctx: Context<ApplyPartialRedemption>) -> Result<()> {
        crate::instructions::apply_partial_redemption::handle_apply_partial_redemption(ctx)
    }

    pub fn advance_time(ctx: Context<AdvanceTime>, seconds: i64) -> Result<()> {
        crate::instructions::advance_time::handle_advance_time(ctx, seconds)
    }
}
