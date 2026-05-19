# Anchor Escrow (Q2 2026)

---

## Program Info

- **Program ID**: `3SrnpRidwwR3hkzVrf5x9DkwDH3d9CAXHi2ZDbEarWBc`

---

## Swap Flow Overview

1.  **Make**: The `maker` initializes the escrow contract, defining the `seed`, the token they want to lock up (`Mint A`), the token they want to receive (`Mint B`), and the target `receive` amount. They transfer the `deposit` amount of `Mint A` into a vault owned by the escrow program.
2.  **Take**: A `taker` accepts the escrow terms. The program checks that they have provided the correct amount of `Mint B` tokens, transfers them to the `maker`, transfers the locked `Mint A` tokens from the vault to the `taker`, and closes the vault and state accounts, returning rent lamports to the `maker`.
3.  **Refund**: If no taker has fulfilled the escrow, the `maker` can cancel it. The program transfers the locked `Mint A` tokens from the vault back to the `maker` and closes the vault and state accounts.

---

## Instruction Set

### 1. `make(seed: u64, deposit: u64, receive: u64)`
Initializes the `Escrow` state and transfers `deposit` tokens from the maker's ATA to the `vault` ATA.

### 2. `take`
Taker deposits the requested `Mint B` tokens to the `maker_ata_b`, receives the locked `Mint A` tokens from the `vault`, and closes the vault and escrow state accounts.
### 3. `refund`
Returns the locked `Mint A` tokens in the `vault` to the `maker_ata_a` and closes both the vault and escrow state accounts.
---

## Testing

Integration tests are implemented in [tests/anchor-escrow-q2-2026.ts](./tests/anchor-escrow-q2-2026.ts).

### Test Cases Covered:
1.  **Makes and refunds the escrow**: Checks that the maker can initialize an escrow and successfully refund/cancel it, verifying all accounts close and tokens are returned.
2.  **Makes and takes the escrow**: Checks the successful path where the taker fulfills the escrow, verifying that the taker receives Mint A, the maker receives Mint B, and the program closes all escrow-related accounts.

### Run Tests:
Ensure you are in the project folder and run:
```bash
anchor build
yarn install
anchor test
```
