use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

use instructions::*;
use state::Side;

declare_id!("C96G3mRtG4X9RcaY9peUufVcmD5FgiMVbFpLJeHghiR5");


#[program]
pub mod simple_orderbook_dex {
    use super::*;

    pub fn initialize_market(ctx: Context<InitializeMarket>) -> Result<()> {
        instructions::initialize_market::handler(ctx)
    }

    pub fn place_order(
        ctx: Context<PlaceOrder>,
        side: Side,
        price: u64,
        amount: u64,
    ) -> Result<()> {
        instructions::place_order::handler(ctx, side, price, amount)
    }

    pub fn fill_order(ctx: Context<FillOrder>, fill_amount: u64) -> Result<()> {
        instructions::fill_order::handler(ctx, fill_amount)
    }

    pub fn cancel_order(ctx: Context<CancelOrder>) -> Result<()> {
        instructions::cancel_order::handler(ctx)
    }
}
