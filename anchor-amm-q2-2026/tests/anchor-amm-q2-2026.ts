import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorAmmQ22026 } from "../target/types/anchor_amm_q2_2026";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createAssociatedTokenAccount, createMint, getAssociatedTokenAddressSync, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { assert } from "chai";

describe("anchor-amm-q2-2026", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.anchorAmmQ22026 as Program<AnchorAmmQ22026>;

  const user = provider.wallet.payer;
  const mints_authority = anchor.web3.Keypair.generate();

  const SEED = new anchor.BN(74);
  const DECIMALS = 6;

  // Mints
  let mint_x: anchor.web3.PublicKey;
  let mint_y: anchor.web3.PublicKey;
  
  // ATAs
  let vault_x: anchor.web3.PublicKey;
  let vault_y: anchor.web3.PublicKey;
  let user_ata_x: anchor.web3.PublicKey;
  let user_ata_y: anchor.web3.PublicKey;
  let user_ata_lp: anchor.web3.PublicKey;
  
  const [config] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("config"), SEED.toBuffer("le", 8)], program.programId);
  const [mint_lp] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("lp"), config.toBuffer()], program.programId);

  before(async () => {
    await provider.connection.requestAirdrop(mints_authority.publicKey, 10_000_000_000);

    mint_x = await createMint(provider.connection, user, mints_authority.publicKey, mints_authority.publicKey, DECIMALS);
    mint_y = await createMint(provider.connection, user, mints_authority.publicKey, mints_authority.publicKey, DECIMALS);

    user_ata_lp = getAssociatedTokenAddressSync(mint_lp, user.publicKey, true);
    user_ata_x = await createAssociatedTokenAccount(provider.connection, user, mint_x, user.publicKey, {commitment: "confirmed"});
    user_ata_y = await createAssociatedTokenAccount(provider.connection, user, mint_y, user.publicKey, {commitment: "confirmed"});

    vault_x = getAssociatedTokenAddressSync(mint_x, config, true);
    vault_y = getAssociatedTokenAddressSync(mint_y, config, true);

    await mintTo(provider.connection, mints_authority, mint_x, user_ata_x, mints_authority.publicKey, 2_000_000);
    await mintTo(provider.connection, mints_authority, mint_y, user_ata_y, mints_authority.publicKey, 2_000_000);
  })

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize(SEED, 0, null)
      .accountsPartial({
        user: user.publicKey,
        config,
        mintX: mint_x,
        mintY: mint_y,
        mintLp: mint_lp,
        vaultX: vault_x,
        vaultY: vault_y,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID
      })
      .rpc();
    let configStruct = await program.account.config.fetch(config);
    assert(configStruct.mintX.toString() == mint_x.toString(), `Incorrect mintX. Expected ${mint_x} Found ${configStruct.mintX}`);
    assert(configStruct.mintY.toString() == mint_y.toString(), `Incorrect mintY. Expected ${mint_y} Found ${configStruct.mintY}`);
    assert(configStruct.authority == null, `Incorrect authority. Expected null, Found ${configStruct.authority}`);
    assert(configStruct.seed.toString() == SEED.toString(), `Incorrect seed. Expected ${SEED}, Found ${configStruct.seed}`);
    console.log("Your transaction signature", tx);
  });

  it("Deposited 100X and 50Y successfully", async () => {
    const tx = await program.methods
      .deposit(new anchor.BN(5000), new anchor.BN(100), new anchor.BN(50))
      .accountsPartial({
        user: user.publicKey,
        config,
        mintX: mint_x,
        mintY: mint_y,
        mintLp: mint_lp,
        vaultX: vault_x,
        vaultY: vault_y,
        userX: user_ata_x,
        userY: user_ata_y,
        userLp: user_ata_lp,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID
      })
      .rpc();
      
      let vaultXBal = (await provider.connection.getTokenAccountBalance(vault_x)).value;
      let vaultYBal = (await provider.connection.getTokenAccountBalance(vault_y)).value;
      let userLpBalance = (await provider.connection.getTokenAccountBalance(user_ata_lp)).value;

      assert(vaultXBal.amount == `100`, `Invalid vault_x balance. Expected 100 found ${vaultXBal}`)
      assert(vaultYBal.amount == `50`, `Invalid vault_x balance. Expected 100 found ${vaultYBal}`)
      assert(userLpBalance.amount == `5000`, `Invalid vault_x balance. Expected 100 found ${userLpBalance}`);

      console.log("VaultX balance:", vaultXBal);
      console.log("VaultY balance:", vaultYBal);
      console.log("UserLp balance:", userLpBalance);
      console.log("Your transaction signature", tx);
  });

  it("Swapped 25units X for 10units of Y", async () => {
    const tx = await program.methods
      .swap(true, new anchor.BN(25), new anchor.BN(10))
      .accountsPartial({
        user: user.publicKey,
        config,
        mintX: mint_x,
        mintY: mint_y,
        vaultX: vault_x,
        vaultY: vault_y,
        userX: user_ata_x,
        userY: user_ata_y,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID
      })
      .rpc();
    
    let vaultXBal = (await provider.connection.getTokenAccountBalance(vault_x)).value;
    let vaultYBal = (await provider.connection.getTokenAccountBalance(vault_y)).value;

    assert(vaultXBal.amount == `125`, `Invalid vault_x balance. Expected 125 found ${vaultXBal}`)
    assert(vaultYBal.amount == `40`, `Invalid vault_x balance. Expected 40 found ${vaultYBal}`)

    console.log("VaultX balance:", vaultXBal);
    console.log("VaultY balance:", vaultYBal);
    console.log("Your transaction signature", tx);
  });

  it("Withdraw from pool using the 5000LP tokens", async () => {
    const tx = await program.methods
      .withdraw(new anchor.BN(5000), new anchor.BN(125), new anchor.BN(40))
      .accountsPartial({
        user: user.publicKey,
        config,
        mintX: mint_x,
        mintY: mint_y,
        mintLp: mint_lp,
        vaultX: vault_x,
        vaultY: vault_y,
        userX: user_ata_x,
        userY: user_ata_y,
        userLp: user_ata_lp,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID
      })
      .rpc()

    let vaultXBal = (await provider.connection.getTokenAccountBalance(vault_x)).value;
    let vaultYBal = (await provider.connection.getTokenAccountBalance(vault_y)).value;
    let userLpBalance = (await provider.connection.getTokenAccountBalance(user_ata_lp)).value;

    assert(vaultXBal.amount == `0`, `Invalid vault_x balance. Expected 0 found ${vaultXBal}`);
    assert(vaultYBal.amount == `0`, `Invalid vault_x balance. Expected 0 found ${vaultYBal}`);
    assert(userLpBalance.amount == `0`, `Invalid vault_x balance. Expected 0 found ${userLpBalance}`);

    console.log("VaultX balance:", vaultXBal);
    console.log("VaultY balance:", vaultYBal);
    console.log("UserLp balance:", userLpBalance);
    console.log("Your transaction signature", tx);
  })

  // TODO: Complete the rest of the tests
});
