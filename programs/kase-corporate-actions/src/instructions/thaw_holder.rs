use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, ThawAccount, Token, TokenAccount};

use crate::{
    constants::*,
    state::{BondSeries, CouponRound, PayoutReceipt},
};

#[derive(Accounts)]
pub struct ThawHolder<'info> {
    pub authority: Signer<'info>,

    #[account(has_one = authority, has_one = bond_mint)]
    pub bond_series: Account<'info, BondSeries>,

    pub bond_mint: Account<'info, Mint>,

    #[account(has_one = bond_series)]
    pub coupon_round: Account<'info, CouponRound>,

    #[account(mut, token::mint = bond_mint)]
    pub holder_token_account: Account<'info, TokenAccount>,

    /// Разморозить можно только после выплаты: квитанция должна существовать
    #[account(
        seeds = [RECEIPT_SEED, coupon_round.key().as_ref(), holder_token_account.key().as_ref()],
        bump = receipt.bump
    )]
    pub receipt: Account<'info, PayoutReceipt>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_thaw_holder(ctx: Context<ThawHolder>) -> Result<()> {
    let bond = &ctx.accounts.bond_series;
    let series_id_bytes = bond.series_id.to_le_bytes();
    let authority_key = bond.authority;
    let bump = [bond.bump];
    let seeds: &[&[u8]] = &[BOND_SEED, authority_key.as_ref(), &series_id_bytes, &bump];
    let signer_seeds = &[seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        ThawAccount {
            account: ctx.accounts.holder_token_account.to_account_info(),
            mint: ctx.accounts.bond_mint.to_account_info(),
            authority: ctx.accounts.bond_series.to_account_info(),
        },
        signer_seeds,
    );
    token::thaw_account(cpi_ctx)?;

    msg!("Thawed holder {}", ctx.accounts.holder_token_account.owner);
    Ok(())
}
