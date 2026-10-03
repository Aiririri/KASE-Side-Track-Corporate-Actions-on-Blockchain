import * as anchor from "@anchor-lang/core";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  transfer,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import fs from "fs";
import { setup, retry, step, log, pdas, roundPda, receiptPda, DAY, USDC } from "./lib";

// Investor wallet (Phantom, devnet). It receives bonds and redeems them from the website.
const YOU = new PublicKey("5KQSHy8F2tfS7P5KpW226P8oWPdiyWm1dtQaqjXQXgny");

async function main() {
  const { program, provider, connection, issuer } = setup();
  const pid = program.programId;
  const BN = anchor.BN;
  const me = issuer.publicKey;
  const seriesId = Math.floor(Date.now() / 1000);
  const { bond, mint, vault } = pdas(pid, me, seriesId);

  const chainNow = async () => {
    const slot = await retry(() => connection.getSlot("confirmed"));
    const t = await retry(() => connection.getBlockTime(slot));
    return t ?? Math.floor(Date.now() / 1000);
  };
  // Simulated time = network clock + the offset the issuer advances
  const simNow = async () => {
    const b: any = await retry(() => program.account.bondSeries.fetch(bond));
    return (await chainNow()) + b.timeOffset.toNumber();
  };

  console.log("Issuer:", me.toBase58());
  console.log("Series id:", seriesId);

  // ---------- Setup: test USDC and participants ----------
  const usdc = await retry(() => createMint(connection, issuer, me, null, 6));
  console.log("Test USDC mint:", usdc.toBase58());
  const issuerUsdc = (
    await retry(() => getOrCreateAssociatedTokenAccount(connection, issuer, usdc, me))
  ).address;
  await retry(() =>
    mintTo(connection, issuer, usdc, issuerUsdc, issuer, BigInt(100_000) * BigInt(USDC))
  );

  const names = ["Alice", "Bob", "You (Phantom)"];
  const alice = Keypair.generate();
  const bob = Keypair.generate();
  const owners: PublicKey[] = [alice.publicKey, bob.publicKey, YOU];
  const signers: (Keypair | null)[] = [alice, bob, null];
  const usdcAta: PublicKey[] = [];
  for (const o of owners) {
    const acc = await retry(() =>
      getOrCreateAssociatedTokenAccount(connection, issuer, usdc, o)
    );
    usdcAta.push(acc.address);
  }
  const sol = [0.02, 0.02, 0.05];
  const fund = new Transaction();
  owners.forEach((o, i) =>
    fund.add(
      SystemProgram.transfer({
        fromPubkey: me,
        toPubkey: o,
        lamports: Math.round(sol[i] * LAMPORTS_PER_SOL),
      })
    )
  );
  await retry(() => provider.sendAndConfirm(fund, []));
  console.log("Holders:", owners.map((o, i) => `${names[i]}=${o.toBase58()}`).join(" "));
  console.log("---------- Scenario ----------");
  // ==== END PART 1 ====

  // ---------- 1. Issue ----------
  const t0 = await chainNow();
  const maturity = t0 + 2 * 365 * DAY;
  const FACE = 1000 * USDC;
  await step("Issue bond", "face value $1000, coupon 10%, semi-annual, matures in 2 years", () =>
    program.methods
      .initializeBond(new BN(seriesId), new BN(FACE), 1000, 2, new BN(maturity))
      .accountsPartial({
        authority: me,
        bondSeries: bond,
        bondMint: mint,
        paymentMint: usdc,
        vault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
  );

  // ---------- 2. Distribute ----------
  const bondAta = owners.map((o) => getAssociatedTokenAddressSync(mint, o));
  const issued = [10, 5, 4];
  for (let i = 0; i < owners.length; i++) {
    await step(`Issue bonds: ${names[i]}`, `${issued[i]} bonds`, () =>
      program.methods
        .issueBonds(new BN(issued[i]))
        .accountsPartial({
          authority: me,
          bondSeries: bond,
          bondMint: mint,
          holder: owners[i],
          holderTokenAccount: bondAta[i],
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc()
    );
  }

  // ---------- 3. Transfer before the record date ----------
  await step("Transfer between holders", "Alice sends 2 bonds to You (balances become 8 / 5 / 6)", () =>
    retry(() => transfer(connection, issuer, bondAta[0], bondAta[2], alice, 2))
  );
  const balances = [8, 5, 6];
  const totalBonds = 19;

  const freezeAll = async (round: PublicKey) => {
    for (let i = 0; i < owners.length; i++) {
      await step(`Freeze holder: ${names[i]}`, `${balances[i]} bonds at record date`, () =>
        program.methods
          .freezeHolder()
          .accountsPartial({
            authority: me,
            bondSeries: bond,
            bondMint: mint,
            couponRound: round,
            holderTokenAccount: bondAta[i],
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc()
      );
    }
    await step("Finalize record date", "frozen supply equals total supply; total due is fixed", () =>
      program.methods
        .finalizeRecordDate()
        .accountsPartial({
          authority: me,
          bondSeries: bond,
          bondMint: mint,
          couponRound: round,
        })
        .rpc()
    );
  };

  const fundVault = (amount: number, why: string) =>
    step("Fund vault", `$${amount / USDC}: ${why}`, () =>
      program.methods
        .fundVault(new BN(amount))
        .accountsPartial({
          authority: me,
          bondSeries: bond,
          source: issuerUsdc,
          vault,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc()
    );

  const payCoupons = async (round: PublicKey, label: string, perBond: number) => {
    for (let i = 0; i < owners.length; i++) {
      const sum = (balances[i] * perBond) / USDC;
      await step(`Coupon ${label}: ${names[i]}`, `${balances[i]} × $${perBond / USDC} = $${sum}`, () =>
        program.methods
          .payCoupon()
          .accountsPartial({
            authority: me,
            bondSeries: bond,
            bondMint: mint,
            couponRound: round,
            holderTokenAccount: bondAta[i],
            holderPaymentAccount: usdcAta[i],
            vault,
            receipt: receiptPda(pid, "receipt", round, bondAta[i]),
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .rpc()
      );
    }
  };

  const thawAll = async (round: PublicKey) => {
    for (let i = 0; i < owners.length; i++) {
      await step(`Thaw holder: ${names[i]}`, "payout received, account released", () =>
        program.methods
          .thawHolder()
          .accountsPartial({
            authority: me,
            bondSeries: bond,
            bondMint: mint,
            couponRound: round,
            holderTokenAccount: bondAta[i],
            receipt: receiptPda(pid, "receipt", round, bondAta[i]),
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc()
      );
    }
  };
  // ==== END PART 2 ====

  // ---------- 4. Round #1: coupon + partial redemption ----------
  const round0 = roundPda(pid, bond, 0);
  await step("Open coupon round #1", "record date in 182 days", () =>
    program.methods
      .openCouponRound(new BN(t0 + 182 * DAY))
      .accountsPartial({
        authority: me,
        bondSeries: bond,
        couponRound: round0,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
  );
  await step("Advance demo clock", "+182 days", () =>
    program.methods
      .advanceTime(new BN(182 * DAY + 60))
      .accountsPartial({ authority: me, bondSeries: bond })
      .rpc()
  );
  await freezeAll(round0);

  const coupon0 = 50 * USDC; // 1000 × 10% ÷ 2
  const partial0 = 250 * USDC; // 25% of $1000
  await fundVault((coupon0 + partial0) * totalBonds, "coupon + partial redemption");
  await payCoupons(round0, "#1", coupon0);

  await step("Schedule partial redemption", "25% of face value, $250 per bond", () =>
    program.methods
      .schedulePartialRedemption(2500)
      .accountsPartial({ authority: me, bondSeries: bond, couponRound: round0 })
      .rpc()
  );
  for (let i = 0; i < owners.length; i++) {
    await step(`Partial redemption: ${names[i]}`, `${balances[i]} × $250 = $${balances[i] * 250}`, () =>
      program.methods
        .payPartialRedemption()
        .accountsPartial({
          authority: me,
          bondSeries: bond,
          bondMint: mint,
          couponRound: round0,
          holderTokenAccount: bondAta[i],
          holderPaymentAccount: usdcAta[i],
          vault,
          receipt: receiptPda(pid, "partial", round0, bondAta[i]),
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc()
    );
  }
  await step("Apply partial redemption", "face value $1000 → $750", () =>
    program.methods
      .applyPartialRedemption()
      .accountsPartial({ authority: me, bondSeries: bond, couponRound: round0 })
      .rpc()
  );
  await thawAll(round0);

  // ---------- 5. Round #2: final coupon at maturity ----------
  const round1 = roundPda(pid, bond, 1);
  await step("Open coupon round #2 (final)", "record date = maturity", () =>
    program.methods
      .openCouponRound(new BN(maturity))
      .accountsPartial({
        authority: me,
        bondSeries: bond,
        couponRound: round1,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
  );
  const secs = Math.max(maturity - (await simNow()) + 60, 1);
  await step("Advance demo clock", `+${Math.round(secs / DAY)} days to maturity`, () =>
    program.methods
      .advanceTime(new BN(secs))
      .accountsPartial({ authority: me, bondSeries: bond })
      .rpc()
  );
  await freezeAll(round1);

  const coupon1 = 37.5 * USDC; // new face value $750 × 10% ÷ 2
  const face1 = 750 * USDC;
  await fundVault((coupon1 + face1) * totalBonds, "final coupon + principal for all holders");
  await payCoupons(round1, "#2", coupon1);
  await thawAll(round1);

  // ---------- 6. Redeem Alice and Bob; YOUR redemption is left for the website ----------
  for (const i of [0, 1]) {
    await step(`Redeem: ${names[i]}`, `${balances[i]} × $750 = $${balances[i] * 750}, bonds burned`, () =>
      program.methods
        .redeem()
        .accountsPartial({
          holder: owners[i],
          bondSeries: bond,
          bondMint: mint,
          holderTokenAccount: bondAta[i],
          holderPaymentAccount: usdcAta[i],
          vault,
          receipt: receiptPda(pid, "redeem", bond, owners[i]),
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([signers[i]!])
        .rpc()
    );
  }
  // ==== END PART 3 ====

  // ---------- Result ----------
  const b: any = await retry(() => program.account.bondSeries.fetch(bond));
  const vaultBal = await retry(() => connection.getTokenAccountBalance(vault));
  console.log("---------- Result ----------");
  console.log("Series status:", Object.keys(b.status)[0], "(stays active until you redeem)");
  console.log("Face value now: $" + b.faceValue.toNumber() / USDC);
  console.log("Vault holds: $" + vaultBal.value.uiAmountString, "(your 6 bonds × $750 = $4500)");
  console.log("Your wallet:", YOU.toBase58());

  const meta = {
    cluster: "devnet",
    programId: pid.toBase58(),
    issuer: me.toBase58(),
    seriesId,
    bondSeries: bond.toBase58(),
    bondMint: mint.toBase58(),
    vault: vault.toBase58(),
    testUsdcMint: usdc.toBase58(),
    maturity,
    youIndex: 2,
    holders: names.map((n, i) => ({
      name: n,
      wallet: owners[i].toBase58(),
      bondAccount: bondAta[i].toBase58(),
      usdcAccount: usdcAta[i].toBase58(),
    })),
    steps: log,
  };
  fs.writeFileSync("live-log.json", JSON.stringify(meta, null, 2));
  fs.mkdirSync("../docs", { recursive: true });
  const rows = log
    .map((e) => `| ${e.n} | ${e.title} | ${e.detail} | [${e.signature.slice(0, 8)}…](${e.url}) |`)
    .join("\n");
  fs.writeFileSync(
    "../docs/live-onchain-log.md",
    `# On-chain log: live series (devnet)\n\nProgram: \`${pid.toBase58()}\`\n\nBond series: \`${bond.toBase58()}\`\n\n| # | Action | Details | Transaction |\n|---|---|---|---|\n${rows}\n`
  );
  console.log("Done: live-log.json and docs/live-onchain-log.md saved.");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
