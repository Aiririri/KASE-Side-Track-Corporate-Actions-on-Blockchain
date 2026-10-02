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
    #[msg("Coupon round snapshot is not finalized")]
    RoundNotSnapshotted,
    #[msg("Invalid payment token account")]
    InvalidPaymentAccount,
    #[msg("Holder account must be frozen at record date")]
    HolderNotFrozen,
    #[msg("Holder has no bonds to pay")]
    NothingToPay,
    #[msg("Total payouts would exceed the amount due")]
    PayoutExceedsDue,
    #[msg("Bond has not reached maturity yet")]
    NotMatured,
    #[msg("Holder token account is frozen: thaw it before redeeming")]
    HolderAccountFrozen,
    #[msg("Partial redemption rate must be between 1 and 9999 bps")]
    InvalidPartialRate,
    #[msg("Partial redemption is already scheduled for this round")]
    PartialAlreadyScheduled,
    #[msg("Partial redemption is not scheduled for this round")]
    PartialNotScheduled,
    #[msg("Partial redemption was already applied")]
    PartialAlreadyApplied,
    #[msg("Not all holders have received the partial redemption")]
    PartialIncomplete,
    #[msg("Partial redemption amount per bond rounds to zero")]
    PartialPerBondZero,
}
