use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum BondStatus {
    Active,
    Redeemed,
}

#[account]
#[derive(InitSpace)]
pub struct BondSeries {
    pub authority: Pubkey,
    pub series_id: u64,
    pub bond_mint: Pubkey,
    pub payment_mint: Pubkey,
    pub vault: Pubkey,
    pub face_value: u64,
    pub coupon_rate_bps: u16,
    pub coupons_per_year: u8,
    pub maturity_ts: i64,
    pub status: BondStatus,
    pub bump: u8,
}
