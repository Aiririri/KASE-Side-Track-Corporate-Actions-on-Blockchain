use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::{
    constants::*,
    error::ErrorCode,
    state::{BondSeries, BondStatus, RedemptionReceipt},
};

#[derive(Accounts)]
pub struct Redeem<'info> {
    /// Холдер сам подписывает: сжигать токены может только владелец
    #[account(mut)]
    pub holder: Signer<'info>,

    #[account(mut, has_one = bond_mint)]
    pub bond_series: Box<Account<'info, BondSeries>>,

    #[account(mut)]
    pub bond_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = bond_mint,
        token::authority = holder,
        constraint = !holder_token_account.is_frozen() @ ErrorCode::HolderAccountFrozen
    )]
    pub holder_token_account: Box<Account<'info, TokenAccount>>,

    /// Куда платим номинал: платёжный счёт этого же холдера
    #[account(
        mut,
        constraint = holder_payment_account.mint == bond_series.payment_mint @ ErrorCode::InvalidPaymentAccount,
        constraint = holder_payment_account.owner == holder.key() @ ErrorCode::InvalidPaymentAccount
    )]
    pub holder_payment_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = bond_series.vault)]
    pub vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = holder,
        space = 8 + RedemptionReceipt::INIT_SPACE,
        seeds = [REDEEM_SEED, bond_series.key().as_ref(), holder.key().as_ref()],
        bump
    )]
    pub receipt: Box<Account<'info, RedemptionReceipt>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_redeem(ctx: Context<Redeem>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= ctx.accounts.bond_series.maturity_ts,
        ErrorCode::NotMatured
    );

    let bonds = ctx.accounts.holder_token_account.amount;
    require!(bonds > 0, ErrorCode::NothingToPay);

    // Номинал к выплате = облигации × номинал (целые числа)
    let principal = bonds
        .checked_mul(ctx.accounts.bond_series.face_value)
        .ok_or(ErrorCode::MathOverflow)?;
    let supply_after = ctx
        .accounts
        .bond_mint
        .supply
        .checked_sub(bonds)
        .ok_or(ErrorCode::MathOverflow)?;

    // 1. Сжигаем облигации (подписывает холдер)
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.key(),
        Burn {
            mint: ctx.accounts.bond_mint.to_account_info(),
            from: ctx.accounts.holder_token_account.to_account_info(),
            authority: ctx.accounts.holder.to_account_info(),
        },
    );
    token::burn(burn_ctx, bonds)?;

    // 2. Платим номинал из vault (подписывает PDA выпуска)
    let series_id_bytes = ctx.accounts.bond_series.series_id.to_le_bytes();
    let authority_key = ctx.accounts.bond_series.authority;
    let bump = [ctx.accounts.bond_series.bump];
    let seeds: &[&[u8]] = &[BOND_SEED, authority_key.as_ref(), &series_id_bytes, &bump];
    let signer_seeds = &[seeds];

    let pay_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.holder_payment_account.to_account_info(),
            authority: ctx.accounts.bond_series.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(pay_ctx, principal)?;

    // 3. Квитанция
    let bond_key = ctx.accounts.bond_series.key();
    let holder_key = ctx.accounts.holder.key();
    let receipt = &mut ctx.accounts.receipt;
    receipt.bond_series = bond_key;
    receipt.holder = holder_key;
    receipt.bonds_redeemed = receipt
        .bonds_redeemed
        .checked_add(bonds)
        .ok_or(ErrorCode::MathOverflow)?;
    receipt.amount_paid = receipt
        .amount_paid
        .checked_add(principal)
        .ok_or(ErrorCode::MathOverflow)?;
    receipt.last_ts = now;
    receipt.bump = ctx.bumps.receipt;

    // 4. Все облигации погашены: закрываем выпуск
    if supply_after == 0 {
        ctx.accounts.bond_series.status = BondStatus::Redeemed;
    }

    msg!(
        "Redeemed: holder={}, bonds={}, principal={}, supply_left={}",
        holder_key,
        bonds,
        principal,
        supply_after
    );
    Ok(())
}
