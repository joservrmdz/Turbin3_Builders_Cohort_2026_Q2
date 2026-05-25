use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{
        Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked
    }
};
use constant_product_curve::{ConstantProduct};

use crate::{error::AmmError, state::Config};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_x: InterfaceAccount<'info, Mint>,
    #[account(
        mint::token_program = token_program
    )]
    pub mint_y: InterfaceAccount<'info, Mint>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
    )]
    pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> Swap<'info> {
    /// If is_x then we're swapping token_x for token_y
    /// else we're swapping token_y for token_x
    pub fn swap(&mut self, is_x: bool, amount: u64, min: u64) -> Result<()> {

        // TODO: use fee basis points to actually charge fees

        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount != 0, AmmError::InvalidAmount);
        require!(self.vault_x.amount > 0 && self.vault_y.amount > 0, AmmError::NoLiquidityInPool);

        let amount_out = match is_x {
            true => ConstantProduct::delta_y_from_x_swap_amount(self.vault_x.amount, self.vault_y.amount, amount).map_err(AmmError::from)?,
            false => ConstantProduct::delta_x_from_y_swap_amount(self.vault_x.amount, self.vault_y.amount, amount).map_err(AmmError::from)?
        };

        // Amount should at least be min
        require!(amount_out >= min, AmmError::SlippageExceeded);

        self.deposit_tokens(is_x, amount)?;
        self.withdraw_tokens(is_x, amount_out)
    }
    
    /// If is_x then we deposit token_x, else we_deposit token_y of size amount to the
    /// respective vault
    pub fn deposit_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (self.user_x.to_account_info(), self.vault_x.to_account_info()),
            false => (self.user_y.to_account_info(), self.vault_y.to_account_info())
        };

        let program = self.token_program.to_account_info();

        let accounts = TransferChecked {
            from,
            to,
            authority: self.user.to_account_info(),
            mint: if is_x {self.mint_x.to_account_info()} else {self.mint_y.to_account_info()}
        };

        let ctx = CpiContext::new(program, accounts);
        let decimals = if is_x {self.mint_x.decimals} else {self.mint_y.decimals};
        transfer_checked(ctx, amount, decimals)
    }

    /// If is_x then we have deposited token_x of size amount and are withdrawing token_y,
    /// else we have deposited token y of size amount and are withdrawing token_x
    pub fn withdraw_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (self.vault_y.to_account_info(), self.user_y.to_account_info()),
            false => (self.vault_x.to_account_info(), self.user_x.to_account_info())
        };

        let program = self.token_program.to_account_info();

        let accounts = TransferChecked {
            from,
            to,
            authority: self.config.to_account_info(),
            mint: if is_x {self.mint_y.to_account_info()} else {self.mint_x.to_account_info()}
        };

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &self.config.seed.to_le_bytes(), &[self.config.config_bump]]];

        let ctx = CpiContext::new_with_signer(program, accounts, signer_seeds);
        let decimals = if is_x {self.mint_y.decimals} else {self.mint_x.decimals};
        transfer_checked(ctx, amount, decimals)
    }
}