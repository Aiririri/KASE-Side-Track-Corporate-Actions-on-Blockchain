use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::ErrorCode,
    state::{BondSeries, BondStatus, CouponRound, RoundStatus},
};

#[derive(Accounts)]
pub struct SchedulePartialRedemption<'info> {
    pub authority: Signer<'info>,

    #[account(
        has_one = authority,
        constraint = bond_series.status == BondStatus::Active @ ErrorCode::BondNotActive
    )]
    pub bond_series: Account<'info, BondSeries>,

    /// Реестр периода уже зафиксирован (замороженные холдеры)
    #[account(
        mut,
        has_one = bond_series,
        constraint = coupon_round.status == RoundStatus::Snapshotted @ ErrorCode::RoundNotSnapshotted
    )]
    pub coupon_round: Account<'info, CouponRound>,
}

pub fn handle_schedule_partial_redemption(
    ctx: Context<SchedulePartialRedemption>,
    bps: u16,
) -> Result<()> {
    require!(
        bps > 0 && (bps as u64) < BPS_DENOMINATOR,
        ErrorCode::InvalidPartialRate
    );
    require!(
        ctx.accounts.coupon_round.partial_bps == 0,
        ErrorCode::PartialAlreadyScheduled
    );

    // Сумма на одну облигацию: только целые числа
    let per_bond = (ctx.accounts.bond_series.face_value as u128)
        .checked_mul(bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        / (BPS_DENOMINATOR as u128);
    let per_bond = u64::try_from(per_bond).map_err(|_| ErrorCode::MathOverflow)?;
    require!(per_bond > 0, ErrorCode::PartialPerBondZero);

    let round = &mut ctx.accounts.coupon_round;
    round.partial_bps = bps;
    round.partial_per_bond = per_bond;
    round.partial_total_due = per_bond
        .checked_mul(round.snapshot_supply)
        .ok_or(ErrorCode::MathOverflow)?;

    msg!(
        "Partial redemption scheduled: round={}, bps={}, per_bond={}, total_due={}",
        round.round_index,
        bps,
        per_bond,
        round.partial_total_due
    );
    Ok(())
}
