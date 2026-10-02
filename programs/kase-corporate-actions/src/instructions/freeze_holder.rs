use anchor_lang::prelude::*;
use anchor_spl::token::{self, FreezeAccount, Mint, Token, TokenAccount};

use crate::{
    constants::*,
    error::ErrorCode,
    state::{BondSeries, CouponRound, RoundStatus},
};

#[derive(Accounts)]
pub struct FreezeHolder<'info> {
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

    #[account(mut, token::mint = bond_mint)]
    pub holder_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_freeze_holder(ctx: Context<FreezeHolder>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= ctx.accounts.coupon_round.record_ts,
        ErrorCode::RecordDateNotReached
    );

    let amount = ctx.accounts.holder_token_account.amount;

    let bond = &ctx.accounts.bond_series;
    let series_id_bytes = bond.series_id.to_le_bytes();
    let authority_key = bond.authority;
    let bump = [bond.bump];
    let seeds: &[&[u8]] = &[BOND_SEED, authority_key.as_ref(), &series_id_bytes, &bump];
    let signer_seeds = &[seeds];

    // Повторная заморозка того же счёта вернёт ошибку от токен-программы,
    // поэтому один баланс нельзя посчитать дважды
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        FreezeAccount {
            account: ctx.accounts.holder_token_account.to_account_info(),
            mint: ctx.accounts.bond_mint.to_account_info(),
            authority: ctx.accounts.bond_series.to_account_info(),
        },
        signer_seeds,
    );
    token::freeze_account(cpi_ctx)?;

    let round = &mut ctx.accounts.coupon_round;
    round.frozen_supply = round
        .frozen_supply
        .checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;

    msg!("Frozen {} bonds of holder {}", amount, ctx.accounts.holder_token_account.owner);
    Ok(())
}
