use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::{
    constants::*,
    error::ErrorCode,
    state::{BondSeries, CouponRound, PayoutReceipt, RoundStatus},
};

#[derive(Accounts)]
pub struct PayCoupon<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(has_one = authority, has_one = bond_mint)]
    pub bond_series: Box<Account<'info, BondSeries>>,

    pub bond_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        has_one = bond_series,
        constraint = coupon_round.status == RoundStatus::Snapshotted @ ErrorCode::RoundNotSnapshotted
    )]
    pub coupon_round: Box<Account<'info, CouponRound>>,

    /// Замороженный счёт холдера: его баланс равен балансу на record date
    #[account(
        constraint = holder_token_account.mint == bond_mint.key() @ ErrorCode::InvalidPaymentAccount,
        constraint = holder_token_account.is_frozen() @ ErrorCode::HolderNotFrozen
    )]
    pub holder_token_account: Box<Account<'info, TokenAccount>>,

    /// Куда платим: платёжный счёт именно этого холдера
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
        seeds = [RECEIPT_SEED, coupon_round.key().as_ref(), holder_token_account.key().as_ref()],
        bump
    )]
    pub receipt: Box<Account<'info, PayoutReceipt>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_pay_coupon(ctx: Context<PayCoupon>) -> Result<()> {
    let bonds = ctx.accounts.holder_token_account.amount;
    require!(bonds > 0, ErrorCode::NothingToPay);

    // Положенная сумма = облигации × купон на облигацию (целые числа)
    let entitlement = bonds
        .checked_mul(ctx.accounts.coupon_round.coupon_per_bond)
        .ok_or(ErrorCode::MathOverflow)?;
    let new_paid_total = ctx
        .accounts
        .coupon_round
        .paid_total
        .checked_add(entitlement)
        .ok_or(ErrorCode::MathOverflow)?;
    require!(
        new_paid_total <= ctx.accounts.coupon_round.total_due,
        ErrorCode::PayoutExceedsDue
    );

    // Платим из vault: от его имени подписывает PDA выпуска
    let bond = &ctx.accounts.bond_series;
    let series_id_bytes = bond.series_id.to_le_bytes();
    let authority_key = bond.authority;
    let bump = [bond.bump];
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

    let round = &mut ctx.accounts.coupon_round;
    round.paid_total = new_paid_total;
    round.holders_paid = round
        .holders_paid
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;

    msg!(
        "Coupon paid: round={}, holder={}, bonds={}, amount={}",
        round.round_index,
        holder,
        bonds,
        entitlement
    );
    Ok(())
}
