use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::{
    constants::*,
    error::ErrorCode,
    state::{BondSeries, CouponRound, PayoutReceipt, RoundStatus},
};

#[derive(Accounts)]
pub struct PayPartialRedemption<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(has_one = authority, has_one = bond_mint)]
    pub bond_series: Box<Account<'info, BondSeries>>,

    pub bond_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        has_one = bond_series,
        constraint = coupon_round.status == RoundStatus::Snapshotted @ ErrorCode::RoundNotSnapshotted,
        constraint = coupon_round.partial_bps > 0 @ ErrorCode::PartialNotScheduled,
        constraint = !coupon_round.partial_applied @ ErrorCode::PartialAlreadyApplied
    )]
    pub coupon_round: Box<Account<'info, CouponRound>>,

    /// Замороженный счёт холдера: баланс равен балансу на record date
    #[account(
        constraint = holder_token_account.mint == bond_mint.key() @ ErrorCode::InvalidPaymentAccount,
        constraint = holder_token_account.is_frozen() @ ErrorCode::HolderNotFrozen
    )]
    pub holder_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = holder_payment_account.mint == bond_series.payment_mint @ ErrorCode::InvalidPaymentAccount,
        constraint = holder_payment_account.owner == holder_token_account.owner @ ErrorCode::InvalidPaymentAccount
    )]
    pub holder_payment_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = bond_series.vault)]
    pub vault: Box<Account<'info, TokenAccount>>,

    /// Квитанция: повторное создание невозможно, значит, и двойная выплата тоже
    #[account(
        init,
        payer = authority,
        space = 8 + PayoutReceipt::INIT_SPACE,
        seeds = [PARTIAL_SEED, coupon_round.key().as_ref(), holder_token_account.key().as_ref()],
        bump
    )]
    pub receipt: Box<Account<'info, PayoutReceipt>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_pay_partial_redemption(ctx: Context<PayPartialRedemption>) -> Result<()> {
    let bonds = ctx.accounts.holder_token_account.amount;
    require!(bonds > 0, ErrorCode::NothingToPay);

    let entitlement = bonds
        .checked_mul(ctx.accounts.coupon_round.partial_per_bond)
        .ok_or(ErrorCode::MathOverflow)?;
    let new_paid_total = ctx
        .accounts
        .coupon_round
        .partial_paid_total
        .checked_add(entitlement)
        .ok_or(ErrorCode::MathOverflow)?;
    require!(
        new_paid_total <= ctx.accounts.coupon_round.partial_total_due,
        ErrorCode::PayoutExceedsDue
    );

    let series_id_bytes = ctx.accounts.bond_series.series_id.to_le_bytes();
    let authority_key = ctx.accounts.bond_series.authority;
    let bump = [ctx.accounts.bond_series.bump];
    let seeds: &[&[u8]] = &[BOND_SEED, authority_key.as_ref(), &series_id_bytes, &bump];
    let signer_seeds = &[seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.holder_payment_account.to_account_info(),
            authority: ctx.accounts.bond_series.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(cpi_ctx, entitlement)?;

    let now = Clock::get()?.unix_timestamp;
    let holder = ctx.accounts.holder_token_account.owner;
    let holder_token_account = ctx.accounts.holder_token_account.key();
    let round_key = ctx.accounts.coupon_round.key();

    let receipt = &mut ctx.accounts.receipt;
    receipt.coupon_round = round_key;
    receipt.holder = holder;
    receipt.holder_token_account = holder_token_account;
    receipt.bonds = bonds;
    receipt.amount = entitlement;
    receipt.paid_ts = now;
    receipt.bump = ctx.bumps.receipt;

    ctx.accounts.coupon_round.partial_paid_total = new_paid_total;

    msg!(
        "Partial redemption paid: holder={}, bonds={}, amount={}",
        holder,
        bonds,
        entitlement
    );
    Ok(())
}
