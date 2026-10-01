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
}
