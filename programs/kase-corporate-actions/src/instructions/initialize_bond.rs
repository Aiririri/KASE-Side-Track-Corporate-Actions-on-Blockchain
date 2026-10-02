use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    constants::*,
    error::ErrorCode,
    state::{BondSeries, BondStatus},
};

#[derive(Accounts)]
#[instruction(series_id: u64)]
pub struct InitializeBond<'info> {
    /// Эмитент: платит за создание аккаунтов и становится владельцем выпуска
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Паспорт выпуска (PDA)
    #[account(
        init,
        payer = authority,
        space = 8 + BondSeries::INIT_SPACE,
        seeds = [BOND_SEED, authority.key().as_ref(), &series_id.to_le_bytes()],
        bump
    )]
    pub bond_series: Account<'info, BondSeries>,

    /// Токен облигации: 0 знаков, управляется программой (PDA)
    #[account(
        init,
        payer = authority,
        seeds = [MINT_SEED, bond_series.key().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = bond_series,
        mint::freeze_authority = bond_series
    )]
    pub bond_mint: Account<'info, Mint>,

    /// Токен, которым платим купоны (тестовый USDC)
    pub payment_mint: Account<'info, Mint>,

    /// Хранилище денег на выплаты (PDA)
    #[account(
        init,
        payer = authority,
        seeds = [VAULT_SEED, bond_series.key().as_ref()],
        bump,
        token::mint = payment_mint,
        token::authority = bond_series
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_bond(
    ctx: Context<InitializeBond>,
    series_id: u64,
    face_value: u64,
    coupon_rate_bps: u16,
    coupons_per_year: u8,
    maturity_ts: i64,
) -> Result<()> {
    require!(face_value > 0, ErrorCode::InvalidFaceValue);
    require!(
        (coupon_rate_bps as u64) <= BPS_DENOMINATOR,
        ErrorCode::InvalidCouponRate
    );
    require!(
        matches!(coupons_per_year, 1 | 2 | 4 | 12),
        ErrorCode::InvalidCouponFrequency
    );
    let now = Clock::get()?.unix_timestamp;
    require!(maturity_ts > now, ErrorCode::MaturityInPast);

    let authority = ctx.accounts.authority.key();
    let bond_mint = ctx.accounts.bond_mint.key();
    let payment_mint = ctx.accounts.payment_mint.key();
    let vault = ctx.accounts.vault.key();
    let bump = ctx.bumps.bond_series;

    let bond = &mut ctx.accounts.bond_series;
    bond.authority = authority;
    bond.series_id = series_id;
    bond.bond_mint = bond_mint;
    bond.payment_mint = payment_mint;
    bond.vault = vault;
    bond.face_value = face_value;
    bond.coupon_rate_bps = coupon_rate_bps;
    bond.coupons_per_year = coupons_per_year;
    bond.maturity_ts = maturity_ts;
    bond.status = BondStatus::Active;
    bond.bump = bump;
    bond.next_round = 0;
    bond.time_offset = 0;

    msg!("Bond series {} initialized", series_id);
    Ok(())
}
