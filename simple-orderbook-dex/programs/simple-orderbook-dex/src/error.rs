use anchor_lang::prelude::*;

#[error_code]
pub enum OrderbookError {
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Overflow")]
    Overflow,
}
