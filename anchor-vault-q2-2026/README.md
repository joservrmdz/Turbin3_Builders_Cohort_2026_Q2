# Anchor Vault (Q2 2026)
---

## Program Info

- **Program ID**: `6EpMWtU37d5LHj6YaA1EoyqkXWo1GKQP9W9b6T2k5YTN`
---

## Architecture & Account Layout

The program uses Program Derived Addresses (PDAs) to secure the vault funds and keep track of state bump seeds.

### 1. Vault State Account (`VaultState`)
Stores the bump seeds for validation of PDA derivation.

### 2. Vault PDA (`vault`)
A System Account owned by the system program but controlled via the program's PDA signer seeds. This account actually holds the deposited SOL.

---

## Instruction Set

### 1. `initialize`
Initializes the `VaultState` PDA and seeds the `vault` PDA with the minimum balance required for a system account.

### 2. `deposit(amount: u64)`
Transfers the specified `amount` of native SOL from the `user` to the `vault` PDA.

### 3. `withdraw(amount: u64)`
Transfers a specified `amount` of native SOL from the `vault` PDA back to the `user`. Uses CPI with signer seeds of the vault state.

### 4. `close`
Transfers all remaining SOL from the `vault` PDA back to the `user` and closes the `VaultState` PDA, reclaiming its rent-exemption lamports.

---

## Testing

Comprehensive integration tests are located in [tests/anchor-vault-q2-2026.ts](./tests/anchor-vault-q2-2026.ts).

### Run Tests:
Ensure you are in the project folder and run:
```bash
anchor build
yarn install
anchor test
```
