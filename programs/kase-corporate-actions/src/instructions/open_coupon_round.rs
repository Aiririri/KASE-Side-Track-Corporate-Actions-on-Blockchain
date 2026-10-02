use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::ErrorCode,
    state::{BondSeries, BondStatus, CouponRound, RoundStatus},
};

#[derive(Accounts)]
pub struct OpenCouponRound<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority,
        constraint = bond_series.status == BondStatus::Active @ ErrorCode::BondNotActive
    )]
    pub bond_series: Account<'info, BondSeries>,

    /// Адрес периода зависит от его номера, поэтому периоды идут строго по порядку
    #[account(
        init,
        payer = authority,
        space = 8 + CouponRound::INIT_SPACE,
        seeds = [ROUND_SEED, bond_series.key().as_ref(), &bond_series.next_round.to_le_bytes()],
        bump
    )]
    pub coupon_round: Account<'info, CouponRound>,

    pub system_program: Program<'info, System>,
}

pub fn handle_open_coupon_round(ctx: Context<OpenCouponRound>, record_ts: i64) -> Result<()> {
    let bond = &mut ctx.accounts.bond_series;
    require!(record_ts <= bond.maturity_ts, ErrorCode::RecordDateAfterMaturity);

    // Купон на одну облигацию: только целые числа, никаких дробей
    let coupon_per_bond = (bond.face_value as u128)
        .checked_mul(bond.coupon_rate_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        / (BPS_DENOMINATOR as u128)
        / (bond.coupons_per_year as u128);
    let coupon_per_bond = u64::try_from(coupon_per_bond).map_err(|_| ErrorCode::MathOverflow)?;

    let round = &mut ctx.accounts.coupon_round;
    round.bond_series = bond.key();
    round.round_index = bond.next_round;
    round.record_ts = record_ts;
    round.coupon_per_bond = coupon_per_bond;
    round.frozen_supply = 0;
    round.snapshot_supply = 0;
    round.total_due = 0;
    round.paid_total = 0;
    round.holders_paid = 0;
    round.partial_bps = 0;
    round.partial_per_bond = 0;
    round.partial_total_due = 0;
    round.partial_paid_total = 0;
    round.partial_applied = false;
    round.status = RoundStatus::Open;
    round.bump = ctx.bumps.coupon_round;

    bond.next_round = bond
        .next_round
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;

    msg!(
        "Coupon round {} opened, record_ts={}, coupon_per_bond={}",
        round.round_index,
        record_ts,
        coupon_per_bond
    );
    Ok(())
}
