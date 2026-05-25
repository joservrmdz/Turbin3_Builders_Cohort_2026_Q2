use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{
        Mint, TokenAccount, TokenInterface, TransferChecked, Burn, transfer_checked, burn
    }
};
use constant_product_curve::ConstantProduct;

use crate::{state::Config, error::AmmError};


#[derive(Accounts)]
pub struct Withdraw<'info> {
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
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,
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
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,
    
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(
        &mut self,
        amount: u64,
        min_x: u64,
        min_y: u64
    ) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount != 0 && self.user_lp.amount >= amount, AmmError::InvalidAmount);

        let (x, y) = {
            let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
                self.vault_x.amount, 
                self.vault_y.amount, 
                self.mint_lp.supply, 
                amount, 
                10u32.pow(6)    // Precision is 6, same as decimals
            ).map_err(AmmError::from)?;
            (amounts.x, amounts.y)
        };

        require!(min_x <= x && min_y <= y, AmmError::SlippageExceeded);

        self.withdraw_tokens(true, x)?;
        self.withdraw_tokens(false, y)?;
        self.burn_lp_tokens(amount)
    }

    pub fn withdraw_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (self.vault_x.to_account_info(), self.user_x.to_account_info()),
            false => (self.vault_y.to_account_info(), self.user_y.to_account_info())
        };

        let program = self.token_program.to_account_info();

        let accounts = TransferChecked {
            from,
            to,
            authority: self.config.to_account_info(),
            mint: if is_x {self.mint_x.to_account_info()} else {self.mint_y.to_account_info()}
        };

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &self.config.seed.to_le_bytes(), &[self.config.config_bump]]];

        let ctx = CpiContext::new_with_signer(program, accounts, signer_seeds);
        let decimals = if is_x {self.mint_x.decimals} else {self.mint_y.decimals};
        transfer_checked(ctx, amount, decimals)
    }

    pub fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let program = self.token_program.to_account_info();

        let accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.user.to_account_info()
        };

        // let signer_seeds: &[&[&[u8]]] = &[&[b"config", &self.config.seed.to_le_bytes(), &[self.config.config_bump]]];

        let ctx = CpiContext::new(program, accounts);
        burn(ctx, amount)
    }
}

// lt 20_000_000
// nlt = 0.01 * 1000000 = 10000
// (20_000_000 + 10_000) * 1_000_000 / 20_000_000
// (20_000_000 + 10_000) * 6 / 20_000_000