use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::error::OrderbookError;
use crate::state::{Market, Order, Side};

pub fn handler(ctx: Context<CancelOrder>) -> Result<()> {
    let order = &ctx.accounts.order;
    let base_mint = ctx.accounts.market.base_mint;
    let quote_mint = ctx.accounts.market.quote_mint;
    let market_bump = ctx.accounts.market.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"market",
        base_mint.as_ref(),
        quote_mint.as_ref(),
        &[market_bump],
    ]];

    if order.amount > 0 {
        match order.side {
            Side::Bid => {
                let refund = order
                    .price
                    .checked_mul(order.amount)
                    .ok_or(OrderbookError::Overflow)?;
                token::transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.quote_vault.to_account_info(),
                            to: ctx.accounts.owner_quote_account.to_account_info(),
                            authority: ctx.accounts.market.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    refund,
                )?;
            }
            Side::Ask => {
                token::transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.base_vault.to_account_info(),
                            to: ctx.accounts.owner_base_account.to_account_info(),
                            authority: ctx.accounts.market.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    order.amount,
                )?;
            }
        }
    }
    Ok(())
}

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [b"market", market.base_mint.as_ref(), market.quote_mint.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        close = owner,
        has_one = owner,
        has_one = market,
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
}
