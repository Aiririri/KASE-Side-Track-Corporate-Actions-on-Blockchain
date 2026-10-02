use anchor_lang::prelude::*;

use crate::{error::ErrorCode, state::BondSeries};

#[derive(Accounts)]
pub struct AdvanceTime<'info> {
    pub authority: Signer<'info>,

    #[account(mut, has_one = authority)]
    pub bond_series: Account<'info, BondSeries>,
}

/// ДЕМО: двигает симулированные часы выпуска вперёд (назад нельзя)
pub fn handle_advance_time(ctx: Context<AdvanceTime>, seconds: i64) -> Result<()> {
    require!(seconds > 0, ErrorCode::InvalidAmount);

    let bond = &mut ctx.accounts.bond_series;
    bond.time_offset = bond
        .time_offset
        .checked_add(seconds)
        .ok_or(ErrorCode::MathOverflow)?;

    msg!("Demo clock advanced by {}s, offset={}", seconds, bond.time_offset);
    Ok(())
}
