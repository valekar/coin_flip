use anchor_lang::prelude::*;
use anchor_lang::solana_program::{clock, program::invoke, system_instruction};
use std::mem::size_of;
declare_id!("4DGK6QrK9gQHMxrWbfAffGsuVztMZ2KqMGw66UaWRuGZ");

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
            return Err(CoinFlipErrorCode::AmountMustBeGreaterThanZero.into());
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

        let claimant_bump = *ctx.bumps.get("claimant").unwrap();
        let claimant = &mut ctx.accounts.claimant;
        claimant.amount = args.amount;
        claimant.claimant_bump = claimant_bump;
        claimant.claimant = claimant.key();

        // heads win case
        if clock.unix_timestamp % 2 == 0 && args.bet_type == BetType::Head {
            claimant.success = true;
            ctx.accounts.distribute_money(args.amount)?;

            emit!(CoinFlipEvent {
                message: String::from("Congratulations you've won!"),
                status: String::from("OK")
            })
        } else if clock.unix_timestamp % 2 == 0 && args.bet_type == BetType::Tail {
            claimant.success = false;
            msg!("Sorry, you have lost");
            emit!(CoinFlipEvent {
                message: String::from("Sorry you've lost"),
                status: String::from("OK")
            })

            // tails win case
        } else if clock.unix_timestamp % 2 != 0 && args.bet_type == BetType::Tail {
            claimant.success = true;
            ctx.accounts.distribute_money(args.amount)?;

            emit!(CoinFlipEvent {
                message: String::from("Congratulations you've won!"),
                status: String::from("OK")
            })
        } else if clock.unix_timestamp % 2 != 0 && args.bet_type == BetType::Head {
            claimant.success = false;

            msg!("Sorry, you have lost");
            emit!(CoinFlipEvent {
                message: String::from("Sorry you've lost"),
                status: String::from("OK")
            })
        }

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, _args: ClaimArgs) -> Result<()> {
        let (claimant, _) = Pubkey::find_program_address(
            &[
                b"claimant".as_ref(),
                ctx.accounts.payer.to_account_info().key().as_ref(),
            ],
            &ID,
        );

        require!(
            ctx.accounts.claimant.key() == claimant,
            CoinFlipErrorCode::OwnerMismatch
        );
        Ok(())
    }
}

// ACCOUNTS
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
#[instruction(args : BetArgs)]
pub struct Bet<'info> {
    #[account(
        mut,
        seeds=[ b"coin-flip".as_ref()],
        bump = coin_flip.bump,
    )]
    pub coin_flip: Account<'info, CoinFlip>,

    #[account(
        init_if_needed,
        seeds = [
            b"claimant".as_ref(), 
            payer.key().as_ref()
        ],
        bump,
        space = 8 + size_of::<Claimant>(),
        payer = payer,
        constraint = payer.is_signer
    )]
    pub claimant: Account<'info, Claimant>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(
        mut,
        seeds = [
            b"claimant".as_ref(), 
            payer.key().as_ref()
        ],
        bump = claimant.claimant_bump,
        close = payer
    )]
    pub claimant: Account<'info, Claimant>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// IMPLEMENTATIONS
impl<'info> Bet<'info> {
    pub fn transfer_minimum_money_to_winners(&mut self) -> Result<()> {
        let winner = self.claimant.to_account_info();
        **winner.try_borrow_mut_lamports()? +=
            self.coin_flip.to_account_info().lamports() - self.coin_flip.minimum_tokens;

        **self.coin_flip.to_account_info().try_borrow_mut_lamports()? =
            self.coin_flip.minimum_tokens;
        Ok(())
    }

    pub fn transfer_money_to_winners(&mut self, amount: u64) -> Result<()> {
        let winner = self.claimant.to_account_info();
        **winner.try_borrow_mut_lamports()? += 2 * amount;
        **self.coin_flip.to_account_info().try_borrow_mut_lamports()? =
            self.coin_flip.to_account_info().lamports() - 2 * amount;
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

// STATE
#[account]
#[derive(Default)]
pub struct CoinFlip {
    authority: Pubkey,
    bump: u8,
    minimum_tokens: u64,
}

#[derive(Default)]
#[account]
pub struct Claimant {
    success: bool,
    amount: u64,
    claimant_bump: u8,
    claimant: Pubkey,
}

// ERROR
#[error_code]
pub enum CoinFlipErrorCode {
    #[msg("Amount must be greater than zero.")]
    AmountMustBeGreaterThanZero,
    #[msg("Owner mismatch")]
    OwnerMismatch,
}

// EVENTS
#[event]
pub struct CoinFlipEvent {
    pub status: String,
    pub message: String,
}

// ARGS
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
pub struct ClaimArgs {}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, PartialEq)]
pub enum BetType {
    Head,
    Tail,
}
