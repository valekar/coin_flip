use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    clock,
    program::{invoke, invoke_signed},
    system_instruction,
};
use std::mem::size_of;

declare_id!("J4hq2CKn2rasEXda9JBFgzZcWxw6rAjWPTiYc8nW5SFC");

#[program]
pub mod coin_flip {

    use super::*;

    pub fn initialize_coin_flip(
        ctx: Context<InitializeCoinFlip>,
        args: CoinFlipArgs,
    ) -> Result<()> {
        {
            let coin_flip = &mut ctx.accounts.coin_flip;
            coin_flip.authority = ctx.accounts.authority.key();
            coin_flip.bump = *ctx.bumps.get("coin_flip").unwrap();
            coin_flip.minimum_tokens = args.minimum_tokens;
        }

        {
            // transfer sols from user account to wallet of router
            invoke(
                &system_instruction::transfer(
                    &ctx.accounts.authority.key,
                    &ctx.accounts.coin_flip.key(),
                    args.amount,
                ),
                &[
                    ctx.accounts.authority.to_account_info().clone(),
                    ctx.accounts.coin_flip.to_account_info().clone(),
                    ctx.accounts.system_program.to_account_info().clone(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn bet(ctx: Context<Bet>, args: BetArgs) -> Result<()> {
        if args.amount <= 0 {
            return Err(ErrorCode::AmountMustBeGreaterThanZero.into());
        }

        // transfer sols from user account to wallet of vault
        {
            invoke(
                &system_instruction::transfer(
                    &ctx.accounts.payer.key,
                    &ctx.accounts.coin_flip.key(),
                    args.amount,
                ),
                &[
                    ctx.accounts.payer.to_account_info().clone(),
                    ctx.accounts.coin_flip.to_account_info().clone(),
                    ctx.accounts.system_program.to_account_info().clone(),
                ],
            )?;
        }

        // // now play
        let clock = clock::Clock::get().unwrap();

        if clock.unix_timestamp % 2 == 0 && args.bet_type == BetType::Head {
            ctx.accounts.distribute_money(args.amount)?;

        } else if clock.unix_timestamp % 2 != 0 && args.bet_type == BetType::Tail {
            ctx.accounts.distribute_money(args.amount)?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeCoinFlip<'info> {
    #[account(
        init,
        payer=authority,
        seeds=[ b"coin-flip".as_ref() ],
        bump,
        space=8 + size_of::<CoinFlip>(),
    )]
    pub coin_flip: Account<'info, CoinFlip>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Bet<'info> {
    #[account(
        mut,
        seeds=[ b"coin-flip".as_ref() ],
        bump = coin_flip.bump,
    )]
    pub coin_flip: Account<'info, CoinFlip>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct CoinFlipArgs {
    amount: u64,
    minimum_tokens: u64,
}

#[account]
pub struct BetArgs {
    amount: u64,
    bet_type: BetType,
}

#[account]
#[derive(Default)]
pub struct CoinFlip {
    authority: Pubkey,
    bump: u8,
    minimum_tokens: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Amount must be greater than zero.")]
    AmountMustBeGreaterThanZero,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, PartialEq)]
pub enum BetType {
    Head,
    Tail,
}

impl<'info> Bet<'info> {
    pub fn transfer_minimum_money_to_winners(&mut self) -> Result<()> {
        let winner = self.payer.to_account_info();
        **winner.try_borrow_mut_lamports()? +=
            self.coin_flip.to_account_info().lamports() - self.coin_flip.minimum_tokens;

        **self.coin_flip.to_account_info().try_borrow_mut_lamports()? =
            self.coin_flip.minimum_tokens;
        Ok(())
    }

    pub fn transfer_money_to_winners(&mut self, amount: u64) -> Result<()> {
        let winner = self.payer.to_account_info();
        **winner.try_borrow_mut_lamports()? += amount;
        **self.coin_flip.to_account_info().try_borrow_mut_lamports()? =
            self.coin_flip.to_account_info().lamports() - amount;

        Ok(())
    }

    pub fn distribute_money(&mut self, amount: u64) -> Result<()> {
        if self.coin_flip.to_account_info().lamports() <= (amount + self.coin_flip.minimum_tokens) {
            msg!("Congratulations, You won! Sry, we didn't have enough reward to gib you. So, we'll gib you all the remaining least reward in the vault");

            self.transfer_minimum_money_to_winners()?;
        } else {
            self.transfer_money_to_winners(amount)?;

            msg!("Congratulations, You won!");
        }

        Ok(())
    }
}
