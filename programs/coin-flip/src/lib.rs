use anchor_lang::prelude::*;
use std::mem::size_of;

declare_id!("J4hq2CKn2rasEXda9JBFgzZcWxw6rAjWPTiYc8nW5SFC");

#[program]
pub mod coin_flip {
    use super::*;

    pub fn initialize_coin_flip(ctx: Context<InitializeCoinFlip>, args : CoinFlipArgs) -> Result<()> {

        {
            let coin_flip = &mut ctx.accounts.coin_flip;
            coin_flip.authority = ctx.accounts.authority.key();

            let pool = &mut ctx.accounts.pool;
            pool.authority = ctx.accounts.authority.key();

        
        }

        //use invoke signed
        {
            // let pool = ctx.accounts.pool.to_account_info();
            // //let remaining_amount = ctx.accounts.coin_flip.to_account_info().lamports() -  args.amount;
            // **pool.try_borrow_mut_lamports()? += ctx.accounts.authority.to_account_info().lamports();
            // **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? = 0;
        }
       

       

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeCoinFlip<'info> {

    #[account(
        init,
        payer=authority,
        space= 8 + size_of::<Pool>(),
        seeds=[b"pool".as_ref(), coin_flip.key().as_ref()],
        bump
    )]
    pub pool : Account<'info, Pool>,

    #[account(
        init,
        payer=authority,
        seeds=[ b"coin-flip".as_ref() ],
        bump,
        space=8 + size_of::<CoinFlip>(),
    )]
    pub coin_flip : Account<'info, CoinFlip>,

    #[account(mut)]
    pub authority : Signer<'info>,

    pub system_program : Program<'info, System>,
    
}

#[account]
#[derive(Default)]
pub struct CoinFlipArgs {
 amount : u64
}


#[account]
#[derive(Default)]
pub struct CoinFlip {
    authority : Pubkey,
}

#[account]
#[derive(Default)]
pub struct Pool {
  authority : Pubkey  
}

