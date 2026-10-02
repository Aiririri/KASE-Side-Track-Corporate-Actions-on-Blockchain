use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum BondStatus {
    Active,
    Redeemed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum RoundStatus {
    /// Период открыт, идёт заморозка холдеров
    Open,
    /// Реестр зафиксирован, можно платить
    Snapshotted,
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
    pub next_round: u32,
}

#[account]
#[derive(InitSpace)]
pub struct CouponRound {
    pub bond_series: Pubkey,
    pub round_index: u32,
    pub record_ts: i64,
    pub coupon_per_bond: u64,
    pub frozen_supply: u64,
    pub snapshot_supply: u64,
    pub total_due: u64,
    pub status: RoundStatus,
    pub bump: u8,
    pub paid_total: u64,
    pub holders_paid: u32,
}

/// Квитанция: доказательство, что этому счёту за этот период уже заплачено
#[account]
#[derive(InitSpace)]
pub struct PayoutReceipt {
    pub coupon_round: Pubkey,
    pub holder: Pubkey,
    pub holder_token_account: Pubkey,
    pub bonds: u64,
    pub amount: u64,
    pub paid_ts: i64,
    pub bump: u8,
}
