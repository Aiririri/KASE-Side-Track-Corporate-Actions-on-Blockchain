use anchor_lang::prelude::*;

use crate::{
    error::ErrorCode,
    state::{BondSeries, CouponRound},
};

#[derive(Accounts)]
pub struct ApplyPartialRedemption<'info> {
    pub authority: Signer<'info>,

    #[account(mut, has_one = authority)]
    pub bond_series: Account<'info, BondSeries>,

    #[account(mut, has_one = bond_series)]
    pub coupon_round: Account<'info, CouponRound>,
}

pub fn handle_apply_partial_redemption(ctx: Context<ApplyPartialRedemption>) -> Result<()> {
    let round = &mut ctx.accounts.coupon_round;
    require!(round.partial_bps > 0, ErrorCode::PartialNotScheduled);
    require!(!round.partial_applied, ErrorCode::PartialAlreadyApplied);
    // Номинал меняем только когда выплачено ВСЕМ холдерам
    require!(
        round.partial_paid_total == round.partial_total_due,
        ErrorCode::PartialIncomplete
    );

    let bond = &mut ctx.accounts.bond_series;
    let old_face = bond.face_value;
    bond.face_value = old_face
        .checked_sub(round.partial_per_bond)
        .ok_or(ErrorCode::MathOverflow)?;
    round.partial_applied = true;

    msg!(
        "Partial redemption applied: face value {} -> {}",
        old_face,
        bond.face_value
    );
    Ok(())
}
