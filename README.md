# Corporate Actions on Solana

Lifecycle automation for tokenized bonds: **identify eligible holders → calculate entitlements → execute settlement → record the outcome on-chain.**

Built for the Superteam Kazakhstan × KASE side track (Corporate Actions on Blockchain). All core logic is an Anchor program on Solana.

## What it does

A test bond (face value $1,000, 10% annual coupon, semi-annual, 2-year maturity) goes through its full life:

1. **Issue** tokenized bonds (1 SPL token = 1 bond) and distribute them to holders.
2. **Record date:** holder balances are frozen on-chain and the program checks that every issued bond is accounted for.
3. **Coupon payment:** the program computes each holder's entitlement and pays it from a program-owned vault.
4. **Partial redemption** (our additional corporate action): pro-rata repayment of part of the principal, which lowers the face value used by all later coupons.
5. **Final redemption** at maturity: principal is paid and the bond tokens are burned.

Example from the task: 10 bonds × $1,000 × 10% ÷ 2 = **$500** coupon, and $10,000 principal at redemption. This is covered by tests.

## Live demo and on-chain proof

- Dashboard: https://kase-side-track-corporate-actions-o.vercel.app/
- Program ID (devnet): `CNbAp4fpVhCP6TQ1uPkJv5VZHVZbffM2gvyVXFNJXYbX`
- Full transaction log of a complete lifecycle run (39 transactions with explorer links): [docs/onchain-log.md](docs/onchain-log.md)
- Second run with English labels (the series shown on the dashboard): [docs/live-onchain-log.md](docs/live-onchain-log.md)
- Demo video: _coming soon_

The dashboard reads everything directly from devnet: instrument state, holder registry per record date, corporate action rounds, payout receipts and redemptions.

## Architecture
**Accounts (all PDAs, owned by the program):**

| Account | Seeds | Purpose |
|---|---|---|
| `BondSeries` | `"bond", issuer, series_id` | Instrument terms, status, demo clock offset. Also the authority of the bond mint and the vault |
| Bond mint | `"bond_mint", bond` | SPL token, 0 decimals. Mint and freeze authority is the `BondSeries` PDA |
| Vault | `"vault", bond` | Token account holding the payment token (test USDC) for payouts |
| `CouponRound` | `"round", bond, index` | One record date: coupon per bond, snapshot supply, amounts due and paid, partial redemption state |
| `PayoutReceipt` (coupon) | `"receipt", round, holder_token_account` | Proof that a holder was paid for a round. Cannot be created twice |
| `PayoutReceipt` (partial) | `"partial", round, holder_token_account` | Same, for partial redemption |
| `RedemptionReceipt` | `"redeem", bond, holder` | Bonds burned and principal paid to a holder |

## Instructions

| Instruction | Signer | What it does |
|---|---|---|
| `initialize_bond` | issuer | Validates terms, creates the series, bond mint and vault |
| `issue_bonds` | issuer | Mints bonds to a holder (creates their token account if needed) |
| `open_coupon_round` | issuer | Opens the next round with a record date and computes the coupon per bond |
| `freeze_holder` | issuer | After the record date, freezes a holder account and adds its balance to the snapshot |
| `finalize_record_date` | issuer | Fixes the snapshot only if frozen supply equals total supply |
| `fund_vault` | issuer | Moves payment tokens into the vault |
| `pay_coupon` | issuer | Pays one frozen holder `bonds × coupon_per_bond`, writes a receipt |
| `schedule_partial_redemption` | issuer | Sets the share of face value to repay, in bps |
| `pay_partial_redemption` | issuer | Pays one frozen holder, writes a receipt |
| `apply_partial_redemption` | issuer | Lowers the face value, only after every holder is paid |
| `thaw_holder` | issuer | Unfreezes a holder, only after a receipt exists |
| `redeem` | **holder** | After maturity: burns the holder's bonds and pays principal from the vault |
| `advance_time` | issuer | Demo only: moves the simulated clock forward |

## Record-date logic

Solana has no historical balances, so the record date is enforced by **freezing** instead of reconstructing a snapshot afterwards (the same idea as an ex-date lock on a real market).

1. `open_coupon_round` stores the record date.
2. Once the record date has passed, `freeze_holder` freezes each holder's token account using the mint's freeze authority (the program PDA). A frozen balance can no longer move, so it equals the balance on the record date. Freezing the same account twice fails, so a balance cannot be counted twice.
3. `finalize_record_date` succeeds only if **frozen supply == total supply of the mint**. If the issuer skips even one holder, the totals differ and the snapshot is rejected. The snapshot supply and the total amount due are then fixed in the round.
4. `pay_coupon` only accepts frozen accounts of a finalized round, so a payment always uses the record-date balance.
5. `thaw_holder` releases an account, and only if its payout receipt exists. Holders cannot trade between the freeze and the payout.

## Entitlement calculations

All amounts are integers in token base units (6 decimals, like USDC). Rates are in basis points. There is no floating point anywhere.
Example: 1,000 × 1,000 ÷ 10,000 ÷ 2 = $50 per bond, so 10 bonds receive $500. Intermediate products use `u128` with checked arithmetic. Division truncates (rounds down), so rounding dust never overpays. Periods are equal and the issuer chooses record dates; accrued-interest day-count conventions are not modelled.

After a partial redemption is applied, `face_value` drops (for example $1,000 → $750). Every later coupon and the final principal use the new face value. In the demo the second coupon is $37.50 per bond (750 × 10% ÷ 2) and the final redemption pays $750 per bond.

## Settlement flow

1. The issuer funds the vault with the payment token (`fund_vault`).
2. Payments move **from the vault, signed by the program PDA**. No private key can move vault funds.
3. Each payment checks that the destination token account belongs to the same owner as the frozen bond account, and that cumulative payments do not exceed the round's total due.
4. A receipt PDA is created in the same transaction. It exists only once, which makes a double payment impossible.
5. Redemption is a single atomic transaction signed by the holder: check maturity → burn bonds → pay principal → write receipt. If the vault is short, everything rolls back and the bonds are not burned. When total supply reaches zero, the series is marked `Redeemed`.

## Implemented vs simulated

| Implemented on Solana | Simulated |
|---|---|
| SPL bond token, 1 token = 1 bond | Fiat settlement: payouts use a test USDC token |
| Holder registry via freeze at record date, with completeness check | KASE / central depository integration |
| Entitlement math computed by the program | Real KYC and holder whitelisting |
| Coupon payments with per-holder receipts | Date oracle: the issuer advances a demo clock (`advance_time`) so two years fit into a short demo |
| Partial redemption that changes face value | |
| Final redemption with token burn and status change | |
| Event-style on-chain records (receipts and program logs) | |

## Known limitations

- The issuer operates the lifecycle and is trusted to run each step. The program enforces the correctness of amounts and the completeness of the snapshot, but it does not force the issuer to act.
- Redemption is not blocked if the final coupon round has not been paid yet. A production version should link them.
- One transaction per holder. A production version would batch holders and use a crank.
- Freezing accounts stops trading between the record date and the payout. This is intentional, but it needs the right schedule on a real market.
- The `advance_time` instruction exists for demo purposes only and must be removed in production.

## Run it yourself

```bash
# program tests (LiteSVM, no network needed)
anchor build && cargo test -p kase-corporate-actions

# full lifecycle on devnet (needs ~0.4 SOL and the deployed program)
cd scripts && npm install && npx tsx demo.ts

# dashboard
cd app && npm install && npm run dev
```
