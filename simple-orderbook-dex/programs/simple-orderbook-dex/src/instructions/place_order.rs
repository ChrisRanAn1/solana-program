use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::error::OrderbookError;
use crate::state::{Market, Order, Side};

pub fn handler(
    ctx: Context<PlaceOrder>,
    side: Side,
    price: u64,
    amount: u64,
) -> Result<()> {
    require!(price > 0, OrderbookError::InvalidPrice);
    require!(amount > 0, OrderbookError::InvalidAmount);

    let market = &mut ctx.accounts.market;
    let order_id = market.next_order_id;

    let order = &mut ctx.accounts.order;
    order.market = market.key();
    order.owner = ctx.accounts.owner.key();
    order.side = side;
    order.price = price;
    order.amount = amount;
    order.order_id = order_id;
    order.bump = ctx.bumps.order;

    match side {
        Side::Bid => {
            let locked = price.checked_mul(amount).ok_or(OrderbookError::Overflow)?;
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.owner_quote_account.to_account_info(),
                        to: ctx.accounts.quote_vault.to_account_info(),
                        authority: ctx.accounts.owner.to_account_info(),
                    },
                ),
                locked,
            )?;
        }
        Side::Ask => {
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.owner_base_account.to_account_info(),
                        to: ctx.accounts.base_vault.to_account_info(),
                        authority: ctx.accounts.owner.to_account_info(),
                    },
                ),
                amount,
            )?;
        }
    }

    market.next_order_id = market
        .next_order_id
        .checked_add(1)
        .ok_or(OrderbookError::Overflow)?;
    Ok(())
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"market", market.base_mint.as_ref(), market.quote_mint.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,
    #[account(
        init,
        payer = owner,
        space = 8 + Order::SIZE,
        seeds = [b"order", market.key().as_ref(), market.next_order_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub order: Account<'info, Order>,
    #[account(mut, address = market.base_vault)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut, address = market.quote_vault)]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(mut, token::mint = market.base_mint, token::authority = owner)]
    pub owner_base_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = market.quote_mint, token::authority = owner)]
    pub owner_quote_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
