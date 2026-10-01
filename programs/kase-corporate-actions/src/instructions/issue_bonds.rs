use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};

use crate::{
    constants::*,
    error::ErrorCode,
    state::{BondSeries, BondStatus},
};

#[derive(Accounts)]
pub struct IssueBonds<'info> {
    /// Только эмитент может выпускать облигации
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        has_one = authority,
        has_one = bond_mint,
        constraint = bond_series.status == BondStatus::Active @ ErrorCode::BondNotActive
    )]
    pub bond_series: Account<'info, BondSeries>,

    #[account(mut)]
    pub bond_mint: Account<'info, Mint>,

    /// CHECK: кошелёк инвестора, любой адрес
    pub holder: UncheckedAccount<'info>,

    /// Токен-аккаунт инвестора под облигации (создаётся при первой выдаче)
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = bond_mint,
        associated_token::authority = holder
    )]
    pub holder_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_issue_bonds(ctx: Context<IssueBonds>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    let bond = &ctx.accounts.bond_series;
    let series_id_bytes = bond.series_id.to_le_bytes();
    let authority_key = bond.authority;
    let bump = [bond.bump];
    let seeds: &[&[u8]] = &[BOND_SEED, authority_key.as_ref(), &series_id_bytes, &bump];
    let signer_seeds = &[seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        MintTo {
            mint: ctx.accounts.bond_mint.to_account_info(),
            to: ctx.accounts.holder_token_account.to_account_info(),
            authority: ctx.accounts.bond_series.to_account_info(),
        },
        signer_seeds,
    );
    token::mint_to(cpi_ctx, amount)?;

    msg!(
        "Issued {} bonds to {}",
        amount,
        ctx.accounts.holder.key()
    );
    Ok(())
}
