#![allow(deprecated, dead_code)] // solana_sdk::system_instruction / system_program

use anchor_lang::InstructionData;
use anchor_lang::solana_program::program_pack::Pack;
use litesvm::LiteSVM;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};

pub const SEED: u64 = 74;
pub const DECIMALS: u8 = 6;
pub const INITIAL_TOKENS: u64 = 2_000_000;

pub const LP_AMOUNT: u64 = 5_000;
pub const DEPOSIT_X: u64 = 100;
pub const DEPOSIT_Y: u64 = 50;

pub const SWAP_AMOUNT: u64 = 25;
pub const SWAP_MIN_OUT: u64 = 10;
pub const VAULT_X_AFTER_SWAP: u64 = 125;
pub const VAULT_Y_AFTER_SWAP: u64 = 40;

pub fn pid() -> Pubkey { anchor_amm_q2_2026::ID }

pub fn send_tx(svm: &mut LiteSVM, signers: &[&Keypair], instructions: &[Instruction]) {
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&signers[0].pubkey()),
        signers,
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();
}

pub fn make_mint(svm: &mut LiteSVM, payer: &Keypair, authority: &Pubkey) -> Keypair {
    let mint = Keypair::new();
    let rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN);
    send_tx(svm, &[payer, &mint], &[
        system_instruction::create_account(
            &payer.pubkey(), &mint.pubkey(), rent,
            spl_token::state::Mint::LEN as u64, &spl_token::id(),
        ),
        spl_token::instruction::initialize_mint(
            &spl_token::id(), &mint.pubkey(), authority, None, DECIMALS,
        ).unwrap(),
    ]);
    mint
}

pub fn make_ata(svm: &mut LiteSVM, payer: &Keypair, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let ata = get_associated_token_address(owner, mint);
    send_tx(svm, &[payer], &[
        create_associated_token_account(&payer.pubkey(), owner, mint, &spl_token::id()),
    ]);
    ata
}

pub fn fund_ata(svm: &mut LiteSVM, payer: &Keypair, mint: &Pubkey, dest: &Pubkey, authority: &Keypair, amount: u64) {
    send_tx(svm, &[payer, authority], &[
        spl_token::instruction::mint_to(
            &spl_token::id(), mint, dest, &authority.pubkey(), &[], amount,
        ).unwrap(),
    ]);
}

pub fn token_balance(svm: &LiteSVM, account: &Pubkey) -> u64 {
    spl_token::state::Account::unpack(&svm.get_account(account).unwrap().data)
        .unwrap()
        .amount
}

pub struct AmmTest {
    pub svm: LiteSVM,
    pub user: Keypair,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub user_ata_x: Pubkey,
    pub user_ata_y: Pubkey,
    pub user_ata_lp: Pubkey,
    pub vault_x: Pubkey,
    pub vault_y: Pubkey,
    pub config: Pubkey,
    pub mint_lp: Pubkey,
}

impl AmmTest {
    pub fn new() -> Self {
        let so_path = format!(
            "{}/../../target/deploy/anchor_amm_q2_2026.so",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut svm = LiteSVM::new();
        svm.add_program_from_file(pid(), so_path).unwrap();

        let user = Keypair::new();
        let mints_authority = Keypair::new();
        svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();
        svm.airdrop(&mints_authority.pubkey(), 10_000_000_000).unwrap();

        let mint_x_kp = make_mint(&mut svm, &user, &mints_authority.pubkey());
        let mint_y_kp = make_mint(&mut svm, &user, &mints_authority.pubkey());
        let mint_x = mint_x_kp.pubkey();
        let mint_y = mint_y_kp.pubkey();

        let (config, _) = Pubkey::find_program_address(&[b"config", &SEED.to_le_bytes()], &pid());
        let (mint_lp, _) = Pubkey::find_program_address(&[b"lp", config.as_ref()], &pid());

        let user_ata_x = make_ata(&mut svm, &user, &user.pubkey(), &mint_x);
        let user_ata_y = make_ata(&mut svm, &user, &user.pubkey(), &mint_y);
        let user_ata_lp = get_associated_token_address(&user.pubkey(), &mint_lp);
        let vault_x = get_associated_token_address(&config, &mint_x);
        let vault_y = get_associated_token_address(&config, &mint_y);

        fund_ata(&mut svm, &user, &mint_x, &user_ata_x, &mints_authority, INITIAL_TOKENS);
        fund_ata(&mut svm, &user, &mint_y, &user_ata_y, &mints_authority, INITIAL_TOKENS);

        AmmTest { svm, user, mint_x, mint_y, user_ata_x, user_ata_y, user_ata_lp, vault_x, vault_y, config, mint_lp }
    }

    pub fn send(&mut self, ix: Instruction) {
        let user = self.user.insecure_clone();
        send_tx(&mut self.svm, &[&user], &[ix]);
    }

    pub fn initialized() -> Self { let mut t = Self::new(); t.initialize(); t }
    pub fn with_liquidity() -> Self { let mut t = Self::initialized(); t.deposit(); t }
    pub fn after_swap() -> Self { let mut t = Self::with_liquidity(); t.swap(); t }

    pub fn initialize(&mut self) {
        self.send(Instruction {
            program_id: pid(),
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new_readonly(self.mint_x, false),
                AccountMeta::new_readonly(self.mint_y, false),
                AccountMeta::new(self.config, false),
                AccountMeta::new(self.mint_lp, false),
                AccountMeta::new(self.vault_x, false),
                AccountMeta::new(self.vault_y, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: anchor_amm_q2_2026::instruction::Initialize { seed: SEED, fee: 0, authority: None }.data(),
        });
    }

    pub fn deposit(&mut self) {
        self.send(Instruction {
            program_id: pid(),
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new_readonly(self.mint_x, false),
                AccountMeta::new_readonly(self.mint_y, false),
                AccountMeta::new_readonly(self.config, false),
                AccountMeta::new(self.mint_lp, false),
                AccountMeta::new(self.vault_x, false),
                AccountMeta::new(self.vault_y, false),
                AccountMeta::new(self.user_ata_x, false),
                AccountMeta::new(self.user_ata_y, false),
                AccountMeta::new(self.user_ata_lp, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: anchor_amm_q2_2026::instruction::Deposit { amount: LP_AMOUNT, max_x: DEPOSIT_X, max_y: DEPOSIT_Y }.data(),
        });
    }

    pub fn swap(&mut self) {
        self.send(Instruction {
            program_id: pid(),
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new_readonly(self.mint_x, false),
                AccountMeta::new_readonly(self.mint_y, false),
                AccountMeta::new_readonly(self.config, false),
                AccountMeta::new(self.vault_x, false),
                AccountMeta::new(self.vault_y, false),
                AccountMeta::new(self.user_ata_x, false),
                AccountMeta::new(self.user_ata_y, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: anchor_amm_q2_2026::instruction::Swap { is_x: true, amount: SWAP_AMOUNT, min: SWAP_MIN_OUT }.data(),
        });
    }

    pub fn withdraw(&mut self) {
        self.send(Instruction {
            program_id: pid(),
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new_readonly(self.mint_x, false),
                AccountMeta::new_readonly(self.mint_y, false),
                AccountMeta::new_readonly(self.config, false),
                AccountMeta::new(self.mint_lp, false),
                AccountMeta::new(self.vault_x, false),
                AccountMeta::new(self.vault_y, false),
                AccountMeta::new(self.user_ata_x, false),
                AccountMeta::new(self.user_ata_y, false),
                AccountMeta::new(self.user_ata_lp, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: anchor_amm_q2_2026::instruction::Withdraw { amount: LP_AMOUNT, min_x: VAULT_X_AFTER_SWAP, min_y: VAULT_Y_AFTER_SWAP }.data(),
        });
    }
}
