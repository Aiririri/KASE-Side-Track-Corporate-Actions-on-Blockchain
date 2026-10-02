use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{error::ErrorCode, state::BondSeries};

#[derive(Accounts)]
pub struct FundVault<'info> {
    pub authority: Signer<'info>,

    #[account(has_one = authority)]
    pub bond_series: Account<'info, BondSeries>,

    /// Платёжный счёт эмитента, откуда берём деньги
    #[account(
        mut,
        constraint = source.mint == bond_series.payment_mint @ ErrorCode::InvalidPaymentAccount,
        constraint = source.owner == authority.key() @ ErrorCode::InvalidPaymentAccount
    )]
    pub source: Account<'info, TokenAccount>,

    #[account(mut, address = bond_series.vault)]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_fund_vault(ctx: Context<FundVault>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.key(),
        Transfer {
            from: ctx.accounts.source.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, amount)?;

    msg!("Vault funded with {}", amount);
    Ok(())
}
