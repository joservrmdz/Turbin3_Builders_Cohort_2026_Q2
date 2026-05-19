# Anchor Vault (Q2 2026)

A secure Solana native SOL vault program built using the **Anchor Framework**. This contract allows users to initialize a vault, deposit SOL, withdraw SOL, and close the vault to claim back rent-exempt lamports.

---

## Program Info

- **Program ID**: `6EpMWtU37d5LHj6YaA1EoyqkXWo1GKQP9W9b6T2k5YTN`
- **Cluster**: `localnet` (configured in `Anchor.toml`)
- **Development Language**: Rust (Anchor Framework)
- **Client Testing**: TypeScript (Mocha, Chai)

---

## Architecture & Account Layout

The program uses Program Derived Addresses (PDAs) to secure the vault funds and keep track of state bump seeds.

### 1. Vault State Account (`VaultState`)
Stores the bump seeds for validation of PDA derivation.
```rust
#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
```

*   **PDA Seeds**: `[b"state", user_pubkey]`
*   **Space**: `8` (Anchor Discriminator) + `1` (vault_bump) + `1` (state_bump) = `10 bytes`

### 2. Vault PDA (`vault`)
A System Account owned by the system program but controlled via the program's PDA signer seeds. This account actually holds the deposited SOL.

*   **PDA Seeds**: `[b"vault", vault_state_pubkey]`

---

## Instruction Set

### 1. `initialize`
Initializes the `VaultState` PDA and seeds the `vault` PDA with the minimum rent-exempt balance required for a system account.
*   **Accounts Required**:
    *   `[signer, mut]` `user`: The authority/depositor initializing the vault.
    *   `[mut]` `vault_state`: The derived `VaultState` PDA.
    *   `[mut]` `vault`: The derived `vault` PDA.
    *   `[]` `system_program`: Solana System Program.

### 2. `deposit(amount: u64)`
Transfers the specified `amount` of native SOL from the `user` to the `vault` PDA.
*   **Accounts Required**:
    *   `[signer, mut]` `user`: The authority depositing SOL.
    *   `[mut]` `vault`: The derived `vault` PDA.
    *   `[]` `vault_state`: The derived `VaultState` PDA.
    *   `[]` `system_program`: Solana System Program.

### 3. `withdraw(amount: u64)`
Transfers the specified `amount` of native SOL from the `vault` PDA back to the `user`. Uses CPI with signer seeds of the vault state.
*   **Accounts Required**:
    *   `[signer]` `user`: The authority withdrawing SOL.
    *   `[mut]` `vault`: The derived `vault` PDA.
    *   `[]` `vault_state`: The derived `VaultState` PDA.
    *   `[]` `system_program`: Solana System Program.

### 4. `close`
Transfers all remaining SOL from the `vault` PDA back to the `user` and closes the `VaultState` PDA, reclaiming its rent-exemption lamports.
*   **Accounts Required**:
    *   `[signer, mut]` `user`: The authority closing the vault.
    *   `[mut]` `vault`: The derived `vault` PDA.
    *   `[mut]` `vault_state`: The derived `VaultState` PDA.
    *   `[]` `system_program`: Solana System Program.

---

## Testing

Comprehensive integration tests are located in [tests/anchor-vault-q2-2026.ts](./tests/anchor-vault-q2-2026.ts).

### Test Suite Flow:
1.  **Initialize the vault**: Asserts that `vault_state` is created correctly and the vault PDA is funded with rent-exempt lamports.
2.  **Deposit SOL into the vault**: Deposits `1 SOL` and checks that the vault balance increases and user balance decreases accordingly.
3.  **Withdraw SOL from the vault**: Withdraws `0.5 SOL` and checks that the balances update correctly.
4.  **Close the vault**: Closes the vault, checks that the vault balance is zero, and confirms `vault_state` is closed (null).

### Run Tests:
Ensure you are in the project folder and run:
```bash
yarn install
anchor test
```
