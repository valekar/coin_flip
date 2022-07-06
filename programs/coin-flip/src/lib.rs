use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    clock,
    program::{invoke, invoke_signed},
    system_instruction,
};
use std::io::Write;
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

        // heads win case
        if clock.unix_timestamp % 2 == 0 && args.bet_type == BetType::Head {
            //ctx.accounts.distribute_money(args.amount)?;

            ctx.accounts
                .create_claimant(args.amount, args.claimant_bump)?;

            {
                ctx.accounts.distribute_money(args.amount)?;
            }
            emit!(CoinFlipEvent {
                message: String::from("Congratulations you've won!"),
                status: String::from("OK")
            })
        } else if clock.unix_timestamp % 2 == 0 && args.bet_type == BetType::Tail {
            msg!("Sorry, you have lost");
            emit!(CoinFlipEvent {
                message: String::from("Sorry you've lost"),
                status: String::from("OK")
            })
        // tails win case
        } else if clock.unix_timestamp % 2 != 0 && args.bet_type == BetType::Tail {
            ctx.accounts
                .create_claimant(args.amount, args.claimant_bump)?;
            {
                ctx.accounts.distribute_money(args.amount)?;
            }
            emit!(CoinFlipEvent {
                message: String::from("Congratulations you've won!"),
                status: String::from("OK")
            })
        } else if clock.unix_timestamp % 2 != 0 && args.bet_type == BetType::Head {
            msg!("Sorry, you have lost");
            emit!(CoinFlipEvent {
                message: String::from("Sorry you've lost"),
                status: String::from("OK")
            })
        }

        Ok(())
    }

    pub fn claim_prize(ctx: Context<ClaimPrize>, args: ClaimPrizeArgs) -> Result<()> {
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

        **ctx.accounts.payer.try_borrow_mut_lamports()? +=
            ctx.accounts.claimant.to_account_info().lamports();
        **ctx.accounts.claimant.try_borrow_mut_lamports()? = 0;

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
#[instruction(args : BetArgs)]
pub struct Bet<'info> {
    #[account(
        mut,
        seeds=[ b"coin-flip".as_ref()],
        bump = coin_flip.bump,
    )]
    pub coin_flip: Account<'info, CoinFlip>,

    /// CHECK:
    #[account(
        mut,
        seeds = [
            b"claimant".as_ref(), 
            payer.key().as_ref()
        ],
        bump = args.claimant_bump)]
    pub claimant: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args : ClaimPrizeArgs)]
pub struct ClaimPrize<'info> {
    /// CHECK:
    #[account(
        mut,
        seeds = [
            b"claimant".as_ref(), 
            payer.key().as_ref()
        ],
        bump = args.claimant_bump)]
    pub claimant: AccountInfo<'info>,

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
    claimant_bump: u8,
}

#[account]
pub struct ClaimPrizeArgs {
    claimant_bump: u8,
}

#[account]
#[derive(Default)]
pub struct CoinFlip {
    authority: Pubkey,
    bump: u8,
    minimum_tokens: u64,
}

#[account]
#[derive(Default)]
pub struct Claimant {
    success: bool,
    amount: u64,
    bump: u8,
    claimant: Pubkey,
}

#[error_code]
pub enum CoinFlipErrorCode {
    #[msg("Amount must be greater than zero.")]
    AmountMustBeGreaterThanZero,
    #[msg("Owner mismatch")]
    OwnerMismatch,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, PartialEq)]
pub enum BetType {
    Head,
    Tail,
}

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
        **winner.try_borrow_mut_lamports()? += amount;
        **self.coin_flip.to_account_info().try_borrow_mut_lamports()? =
            self.coin_flip.to_account_info().lamports() - amount;
        Ok(())
    }

    pub fn create_claimant(&mut self, amount: u64, claimant_bump: u8) -> Result<()> {
        let claimant_state = self.claimant.lamports() == 0;

        let rent = &Rent::get()?;
        let space = 8 + Claimant::default().try_to_vec().unwrap().len();
        if claimant_state {
            let payer_key = self.payer.key();
            let seeds = &[
                b"claimant" as &[u8],
                &payer_key.to_bytes(),
                &[claimant_bump],
            ];
            let claimer_signer_seeds = &[&seeds[..]];
            let lamports = rent.minimum_balance(space);

            invoke_signed(
                &system_instruction::create_account(
                    &payer_key,
                    &self.claimant.key(),
                    lamports,
                    space as u64,
                    &ID,
                ),
                &[
                    self.payer.to_account_info().clone(),
                    self.claimant.to_account_info().clone(),
                    self.system_program.to_account_info().clone(),
                ],
                claimer_signer_seeds,
            )?;

            let mut data = self.claimant.try_borrow_mut_data()?;
            let dst: &mut [u8] = &mut data;
            let mut cursor = std::io::Cursor::new(dst);
            let buffer = &<Claimant as anchor_lang::Discriminator>::discriminator();
            cursor.write_all(buffer).unwrap();

            msg!("New claimant account created at {}", self.claimant.key());
        }

        require!(
            *self.claimant.to_account_info().owner == ID,
            CoinFlipErrorCode::OwnerMismatch
        );

        let mut claimant_account_state: Account<Claimant> = Account::try_from(&self.claimant)?;

        {
            // first time initialization

            claimant_account_state.bump = claimant_bump;
            claimant_account_state.claimant = self.payer.key();
            claimant_account_state.success = true;
            claimant_account_state.amount = amount;
            let mut claimant_data: &mut [u8] = &mut self.claimant.try_borrow_mut_data()?;
            claimant_account_state.try_serialize(&mut claimant_data)?;
        }

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

#[event]
pub struct CoinFlipEvent {
    pub status: String,
    pub message: String,
}
