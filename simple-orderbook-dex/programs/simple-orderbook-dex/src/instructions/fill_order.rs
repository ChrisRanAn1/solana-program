use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::error::OrderbookError;
use crate::state::{Market, Order, Side};

pub fn handler(ctx: Context<FillOrder>, fill_amount: u64) -> Result<()> {
    let order = &mut ctx.accounts.order;
    require!(fill_amount > 0, OrderbookError::InvalidAmount);
    require!(fill_amount <= order.amount, OrderbookError::InvalidAmount);

    let quote_amount = order
        .price
        .checked_mul(fill_amount)
        .ok_or(OrderbookError::Overflow)?;

    let base_mint = ctx.accounts.market.base_mint;
    let quote_mint = ctx.accounts.market.quote_mint;
    let market_bump = ctx.accounts.market.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"market",
        base_mint.as_ref(),
        quote_mint.as_ref(),
        &[market_bump],
    ]];

    match order.side {
        Side::Bid => {
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.taker_base_account.to_account_info(),
                        to: ctx.accounts.maker_base_account.to_account_info(),
                        authority: ctx.accounts.taker.to_account_info(),
                    },
                ),
                fill_amount,
            )?;
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.quote_vault.to_account_info(),
                        to: ctx.accounts.taker_quote_account.to_account_info(),
                        authority: ctx.accounts.market.to_account_info(),
                    },
                    signer_seeds,
                ),
                quote_amount,
            )?;
        }
        Side::Ask => {
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.taker_quote_account.to_account_info(),
                        to: ctx.accounts.maker_quote_account.to_account_info(),
                        authority: ctx.accounts.taker.to_account_info(),
                    },
                ),
                quote_amount,
            )?;
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.base_vault.to_account_info(),
                        to: ctx.accounts.taker_base_account.to_account_info(),
                        authority: ctx.accounts.market.to_account_info(),
                    },
                    signer_seeds,
                ),
                fill_amount,
            )?;
        }
    }

    order.amount = order
        .amount
        .checked_sub(fill_amount)
        .ok_or(OrderbookError::Overflow)?;
    Ok(())
}

#[derive(Accounts)]
pub struct FillOrder<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(
        seeds = [b"market", market.base_mint.as_ref(), market.quote_mint.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,
    #[account(mut, has_one = market)]
    pub order: Account<'info, Order>,
    #[account(mut, address = market.base_vault)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut, address = market.quote_vault)]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(mut, token::mint = market.base_mint, token::authority = taker)]
    pub taker_base_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = market.quote_mint, token::authority = taker)]
    pub taker_quote_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = market.base_mint, token::authority = order.owner)]
    pub maker_base_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = market.quote_mint, token::authority = order.owner)]
    pub maker_quote_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
