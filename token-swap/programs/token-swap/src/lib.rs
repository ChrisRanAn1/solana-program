use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount, TransferChecked, transfer_checked, CloseAccount, close_account};

declare_id!("6VwZo3JmKxYUwfTeHi8REU2MuuGMGCd5UH6qMWtuZ9oA");

#[program]
pub mod token_swap {
    

use super::*;

    pub fn init_swap(ctx: Context<InitSwap>,seed:u8,amount_provide:u64,amount_recieve:u64) -> Result<()> {
        let swapvault=&mut ctx.accounts.swap_vault;
        swapvault.maker=ctx.accounts.maker.key();
        swapvault.amount_provide=amount_provide;
        swapvault.amount_recieve=amount_recieve;
        swapvault.maker_mint=ctx.accounts.maker_mint.key();
        swapvault.taker_mint=ctx.accounts.taker_mint.key();
        swapvault.seed=seed;
        swapvault.bump=ctx.bumps.swap_vault;

        let cpi_program=ctx.accounts.token_program.to_account_info();
        let cpi_accounts=TransferChecked{
            from:ctx.accounts.maker_ata.to_account_info(),
            mint:ctx.accounts.maker_mint.to_account_info(),
            to:ctx.accounts.swap_vault_maker_mint_ata.to_account_info(),
            authority:ctx.accounts.maker.to_account_info(),
        };
        let cpi_ctx=CpiContext::new(cpi_program,cpi_accounts);
        let decimals=ctx.accounts.maker_mint.decimals;
        transfer_checked(cpi_ctx,amount_provide,decimals)?;
        
        Ok(())
    }

    


    pub fn swap(ctx:Context<Swap>)->Result<()>{
        require!(ctx.accounts.swap_vault_maker_mint_ata.amount>0,SwapError::EmptyVault);
        

        let cpi_program=ctx.accounts.token_program.to_account_info();
        let cpi_account=TransferChecked{
            from:ctx.accounts.taker_ata_pay.to_account_info(),
            mint:ctx.accounts.taker_mint.to_account_info(),
            to:ctx.accounts.maker_ata_recieve.to_account_info(),
            authority:ctx.accounts.taker.to_account_info(),
        };
        let cpi_ctx=CpiContext::new(cpi_program,cpi_account);
        let decimals=ctx.accounts.taker_mint.decimals;
        let amount=ctx.accounts.swap_vault.amount_recieve;
        transfer_checked(cpi_ctx,amount,decimals)?;

        let maker_key=ctx.accounts.swap_vault.maker;
        let seed=[ctx.accounts.swap_vault.seed];
        let bump=[ctx.accounts.swap_vault.bump];
        let signer_seeds:&[&[&[u8]]]=&[&[b"swapvault",maker_key.as_ref(),&seed,&bump]];
        let cpi_program=ctx.accounts.token_program.to_account_info();
        let cpi_accounts=TransferChecked{
            from:ctx.accounts.swap_vault_maker_mint_ata.to_account_info(),
            mint:ctx.accounts.maker_mint.to_account_info(),
            to:ctx.accounts.taker_ata_recieve.to_account_info(),
            authority:ctx.accounts.swap_vault.to_account_info(),
        };

        let cpi_ctx=CpiContext::new_with_signer(cpi_program,cpi_accounts,signer_seeds);
        let decimal=ctx.accounts.maker_mint.decimals;
        let amount=ctx.accounts.swap_vault_maker_mint_ata.amount;
        transfer_checked(cpi_ctx,amount,decimal)?;

        
        let cpi_accounts=CloseAccount{
            account:ctx.accounts.swap_vault_maker_mint_ata.to_account_info(),
            destination:ctx.accounts.maker.to_account_info(),
            authority:ctx.accounts.swap_vault.to_account_info(),
        };
        let cpi_ctx=CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(),cpi_accounts,signer_seeds);
        close_account(cpi_ctx)?;





        Ok(())

    }

    


    pub fn refund(ctx:Context<Refund>)->Result<()>{
        let cpi_program=ctx.accounts.token_program.to_account_info();
        let seed=[ctx.accounts.swap_vault.seed];
        let bump=[ctx.accounts.swap_vault.bump];
        let maker_key=ctx.accounts.maker.key();
        let signer_seed:&[&[&[u8]]]=&[&[b"swapvault",maker_key.as_ref(),&seed,&bump]];
        let cpi_accounts=TransferChecked{
            from:ctx.accounts.swap_vault_maker_mint_ata.to_account_info(),
            mint:ctx.accounts.maker_mint.to_account_info(),
            to:ctx.accounts.maker_ata.to_account_info(),
            authority:ctx.accounts.swap_vault.to_account_info(),
        };

        let cpi_ctx=CpiContext::new_with_signer(cpi_program,cpi_accounts,signer_seed);
        let decimal=ctx.accounts.maker_mint.decimals;
        let amount=ctx.accounts.swap_vault_maker_mint_ata.amount;
        transfer_checked(cpi_ctx,amount,decimal)?;

        
        let cpi_accounts=CloseAccount{
            account:ctx.accounts.swap_vault_maker_mint_ata.to_account_info(),
            destination:ctx.accounts.maker.to_account_info(),
            authority:ctx.accounts.swap_vault.to_account_info(),
        };
        let cpi_ctx=CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(),cpi_accounts,signer_seed);
        close_account(cpi_ctx)?;

        Ok(())

    }

   


    
}

    #[derive(Accounts)]
    #[instruction(seed:u8)]
    pub struct InitSwap<'info>{
        #[account(
            init,
            payer=maker,
            space=8+SwapVault::INIT_SPACE,
            seeds=[b"swapvault",maker.key().as_ref(),&[seed]],
            bump,
        )]
        pub swap_vault:Account<'info, SwapVault>,

        #[account(mut)]
        pub maker:Signer<'info>,

        
        pub maker_mint:Account<'info,Mint>,

        
        pub taker_mint:Account<'info,Mint>,

        #[account(
            mut,
            associated_token::mint=maker_mint,
            associated_token::authority=maker,
        )]
        pub maker_ata:Account<'info, TokenAccount>,
        #[account(
            init,
            payer=maker,
            associated_token::mint=maker_mint,
            associated_token::authority=swap_vault,
        )]
        pub swap_vault_maker_mint_ata:Account<'info,TokenAccount>,

        pub system_program:Program<'info,System>,
        pub token_program:Program<'info,Token>,
        pub associated_token_program:Program<'info,AssociatedToken>,

    }

    #[derive(Accounts)]
    pub struct Swap<'info>{
        #[account(
            mut,
            seeds=[b"swapvault",maker.key().as_ref(),&[swap_vault.seed]],
            bump=swap_vault.bump,
            has_one=maker,
            has_one=maker_mint,
            has_one=taker_mint,
            close=maker,
        )]
        pub swap_vault:Account<'info,SwapVault>,
        ///CHECK:maker
        #[account(mut)]
        pub maker:AccountInfo<'info>,


        pub maker_mint:Account<'info,Mint>,


        pub taker_mint:Account<'info, Mint>,

        #[account(
            mut,
            associated_token::mint=maker_mint,
            associated_token::authority=swap_vault,
        )]
        pub swap_vault_maker_mint_ata:Account<'info, TokenAccount>,

        #[account(
            init_if_needed,
            payer=taker,
            associated_token::mint=taker_mint,
            associated_token::authority=maker,
        )]
        pub maker_ata_recieve:Account<'info,TokenAccount>,

        #[account(
            mut,
            associated_token::mint=taker_mint,
            associated_token::authority=taker,
        )]
        pub taker_ata_pay:Account<'info, TokenAccount>,

        #[account(
            init_if_needed,
            payer=taker,
            associated_token::mint=maker_mint,
            associated_token::authority=taker,
        )]
        pub taker_ata_recieve:Account<'info,TokenAccount>,

        #[account(mut)]
        pub taker:Signer<'info>,

        pub system_program:Program<'info,System>,
        pub token_program:Program<'info,Token>,
        pub associated_token_program:Program<'info,AssociatedToken>,
    }

    #[derive(Accounts)]
    pub struct Refund<'info>{
        #[account(

            mut,
            close=maker,
            seeds=[b"swapvault",maker.key().as_ref(), &[swap_vault.seed]],
            bump=swap_vault.bump,
            has_one=maker,
            has_one=maker_mint,
        )]
        pub swap_vault:Account<'info,SwapVault>,

       
        pub maker_mint:Account<'info,Mint>,

        #[account(
            mut,
            associated_token::mint=maker_mint,
            associated_token::authority=swap_vault,
        )]
        pub swap_vault_maker_mint_ata:Account<'info,TokenAccount>,

        #[account(
            mut,
            associated_token::mint=maker_mint,
            associated_token::authority=maker,
        )]
        pub maker_ata:Account<'info,TokenAccount>,

        #[account(mut)]
        pub maker:Signer<'info>,

        pub system_program:Program<'info,System>,
        pub token_program:Program<'info,Token>,
        pub associated_token_program:Program<'info,AssociatedToken>,
    }




#[account]
#[derive(InitSpace)]
pub struct SwapVault{
    pub maker:Pubkey,

    pub amount_provide:u64,

    pub amount_recieve:u64,

    pub maker_mint:Pubkey,

    pub taker_mint:Pubkey,

    pub seed:u8,

    pub bump:u8,

    
}

#[error_code]
pub enum SwapError{
    #[msg("Vault is empty")]
    EmptyVault,

}

