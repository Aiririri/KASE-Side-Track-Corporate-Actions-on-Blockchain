use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Face value must be greater than zero")]
    InvalidFaceValue,
    #[msg("Coupon rate must be between 0 and 10000 bps")]
    InvalidCouponRate,
    #[msg("Coupons per year must be 1, 2, 4 or 12")]
    InvalidCouponFrequency,
    #[msg("Maturity date must be in the future")]
    MaturityInPast,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Bond series is not active")]
    BondNotActive,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Record date must not be after maturity")]
    RecordDateAfterMaturity,
    #[msg("Record date has not been reached yet")]
    RecordDateNotReached,
    #[msg("Coupon round is not open")]
    RoundNotOpen,
    #[msg("Not all holders are frozen: snapshot is incomplete")]
    SnapshotIncomplete,
}
