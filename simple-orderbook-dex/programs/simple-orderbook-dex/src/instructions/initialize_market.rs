use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::Market;

pub fn handler(ctx: Context<InitializeMarket>) -> Result<()> {
    let m = &mut ctx.accounts.market;
    m.authority = ctx.accounts.authority.key();
    m.base_mint = ctx.accounts.base_mint.key();
    m.quote_mint = ctx.accounts.quote_mint.key();
    m.base_vault = ctx.accounts.base_vault.key();
    m.quote_vault = ctx.accounts.quote_vault.key();
    m.next_order_id = 0;
    m.bump = ctx.bumps.market;
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub base_mint: Account<'info, Mint>,
    pub quote_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        space = 8 + Market::SIZE,
        seeds = [b"market", base_mint.key().as_ref(), quote_mint.key().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(
        init,
        payer = authority,
        token::mint = base_mint,
        token::authority = market,
        seeds = [b"base_vault", market.key().as_ref()],
        bump
    )]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        token::mint = quote_mint,
        token::authority = market,
        seeds = [b"quote_vault", market.key().as_ref()],
        bump
    )]
    pub quote_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
