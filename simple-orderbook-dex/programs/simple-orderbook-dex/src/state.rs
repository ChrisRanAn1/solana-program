use anchor_lang::prelude::*;

#[account]
pub struct Market {
    pub authority: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub next_order_id: u64,
    pub bump: u8,
}
impl Market {
    pub const SIZE: usize = 32 * 5 + 8 + 1;
}

#[account]
pub struct Order {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub side: Side,
    pub price: u64,
    pub amount: u64,
    pub order_id: u64,
    pub bump: u8,
}
impl Order {
    pub const SIZE: usize = 32 + 32 + 1 + 8 + 8 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Bid,
    Ask,
}
