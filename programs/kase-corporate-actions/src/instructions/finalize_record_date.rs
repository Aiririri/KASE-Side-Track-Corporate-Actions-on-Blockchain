use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{
    error::ErrorCode,
    state::{BondSeries, CouponRound, RoundStatus},
};

#[derive(Accounts)]
pub struct FinalizeRecordDate<'info> {
    pub authority: Signer<'info>,

    #[account(has_one = authority, has_one = bond_mint)]
    pub bond_series: Account<'info, BondSeries>,

    pub bond_mint: Account<'info, Mint>,

    #[account(
        mut,
        has_one = bond_series,
        constraint = coupon_round.status == RoundStatus::Open @ ErrorCode::RoundNotOpen
    )]
    pub coupon_round: Account<'info, CouponRound>,
}

pub fn handle_finalize_record_date(ctx: Context<FinalizeRecordDate>) -> Result<()> {
    let now = ctx.accounts.bond_series.now()?;
    let round = &mut ctx.accounts.coupon_round;
    require!(now >= round.record_ts, ErrorCode::RecordDateNotReached);

    // Реестр полный, только если заморожены ВСЕ выпущенные облигации
    require!(
        round.frozen_supply == ctx.accounts.bond_mint.supply,
        ErrorCode::SnapshotIncomplete
    );

    round.snapshot_supply = round.frozen_supply;
    round.total_due = round
        .coupon_per_bond
        .checked_mul(round.snapshot_supply)
        .ok_or(ErrorCode::MathOverflow)?;
    round.status = RoundStatus::Snapshotted;

    msg!(
        "Round {} snapshot: supply={}, total_due={}",
        round.round_index,
        round.snapshot_supply,
        round.total_due
    );
    Ok(())
}
