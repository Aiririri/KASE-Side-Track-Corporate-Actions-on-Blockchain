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
  // Симулированное время = часы сети + смещение, которое двигает эмитент
  const simNow = async () => {
    const b: any = await retry(() => program.account.bondSeries.fetch(bond));
    return (await chainNow()) + b.timeOffset.toNumber();
  };

  console.log("Issuer:", me.toBase58());
  console.log("Series id:", seriesId);

  // ---------- Подготовка: тестовый USDC, инвесторы ----------
  const usdc = await retry(() => createMint(connection, issuer, me, null, 6));
  console.log("Test USDC mint:", usdc.toBase58());
  const issuerUsdc = (
    await retry(() => getOrCreateAssociatedTokenAccount(connection, issuer, usdc, me))
  ).address;
  await retry(() =>
    mintTo(connection, issuer, usdc, issuerUsdc, issuer, BigInt(100_000) * BigInt(USDC))
  );

  const names = ["Alice", "Bob", "Carol"];
  const holders = names.map(() => Keypair.generate());
  const usdcAta: PublicKey[] = [];
  for (const h of holders) {
    const acc = await retry(() =>
      getOrCreateAssociatedTokenAccount(connection, issuer, usdc, h.publicKey)
    );
    usdcAta.push(acc.address);
  }
  const fund = new Transaction();
  for (const h of holders) {
    fund.add(
      SystemProgram.transfer({
        fromPubkey: me,
        toPubkey: h.publicKey,
        lamports: Math.round(0.02 * LAMPORTS_PER_SOL),
      })
    );
  }
  await retry(() => provider.sendAndConfirm(fund, []));
  console.log("Инвесторы:", holders.map((h, i) => `${names[i]}=${h.publicKey.toBase58()}`).join(" "));
  console.log("---------- Сценарий ----------");
  // ==== END PART 1 ====

  // ---------- 1. Выпуск ----------
  const t0 = await chainNow();
  const maturity = t0 + 2 * 365 * DAY;
  const FACE = 1000 * USDC;
  await step("Выпуск облигации", "номинал $1000, купон 10%, 2 раза в год, погашение через 2 года", () =>
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

  // ---------- 2. Раздача облигаций ----------
  const bondAta = holders.map((h) => getAssociatedTokenAddressSync(mint, h.publicKey));
  const issued = [10, 5, 3];
  for (let i = 0; i < holders.length; i++) {
    await step(`Выдача облигаций: ${names[i]}`, `${issued[i]} шт.`, () =>
      program.methods
        .issueBonds(new BN(issued[i]))
        .accountsPartial({
          authority: me,
          bondSeries: bond,
          bondMint: mint,
          holder: holders[i].publicKey,
          holderTokenAccount: bondAta[i],
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc()
    );
  }

  // ---------- 3. Перевод до record date ----------
  await step("Перевод между холдерами", "Alice передаёт Carol 2 облигации (баланс будет 8 / 5 / 5)", () =>
    retry(() => transfer(connection, issuer, bondAta[0], bondAta[2], holders[0], 2))
  );
  const balances = [8, 5, 5];
  const totalBonds = 18;

  const freezeAll = async (round: PublicKey) => {
    for (let i = 0; i < holders.length; i++) {
      await step(`Заморозка реестра: ${names[i]}`, `${balances[i]} шт. на record date`, () =>
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
    await step("Фиксация реестра", "замороженная сумма равна выпуску, считаем total_due", () =>
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
    step("Пополнение vault", `$${amount / USDC}: ${why}`, () =>
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
    for (let i = 0; i < holders.length; i++) {
      const sum = (balances[i] * perBond) / USDC;
      await step(`Купон ${label}: ${names[i]}`, `${balances[i]} × $${perBond / USDC} = $${sum}`, () =>
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
    for (let i = 0; i < holders.length; i++) {
      await step(`Разморозка: ${names[i]}`, "выплата получена, счёт снова свободен", () =>
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

  // ---------- 4. Период №1: купон + частичное погашение ----------
  const round0 = roundPda(pid, bond, 0);
  await step("Открытие купонного периода №1", "record date через 182 дня", () =>
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
  await step("Перемотка времени (демо)", "+182 дня", () =>
    program.methods
      .advanceTime(new BN(182 * DAY + 60))
      .accountsPartial({ authority: me, bondSeries: bond })
      .rpc()
  );
  await freezeAll(round0);

  const coupon0 = 50 * USDC; // 1000 × 10% ÷ 2
  const partial0 = 250 * USDC; // 25% от $1000
  await fundVault((coupon0 + partial0) * totalBonds, "купон + частичное погашение");
  await payCoupons(round0, "№1", coupon0);

  await step("Объявление частичного погашения", "25% номинала, $250 на облигацию", () =>
    program.methods
      .schedulePartialRedemption(2500)
      .accountsPartial({ authority: me, bondSeries: bond, couponRound: round0 })
      .rpc()
  );
  for (let i = 0; i < holders.length; i++) {
    await step(`Частичное погашение: ${names[i]}`, `${balances[i]} × $250 = $${balances[i] * 250}`, () =>
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
  await step("Применение частичного погашения", "номинал $1000 → $750", () =>
    program.methods
      .applyPartialRedemption()
      .accountsPartial({ authority: me, bondSeries: bond, couponRound: round0 })
      .rpc()
  );
  await thawAll(round0);

  // ---------- 5. Период №2: финальный купон на дату погашения ----------
  const round1 = roundPda(pid, bond, 1);
  await step("Открытие купонного периода №2 (финальный)", "record date = дата погашения", () =>
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
  await step("Перемотка времени (демо)", `+${Math.round(secs / DAY)} дней до даты погашения`, () =>
    program.methods
      .advanceTime(new BN(secs))
      .accountsPartial({ authority: me, bondSeries: bond })
      .rpc()
  );
  await freezeAll(round1);

  const coupon1 = 37.5 * USDC; // новый номинал $750 × 10% ÷ 2
  const face1 = 750 * USDC;
  await fundVault((coupon1 + face1) * totalBonds, "финальный купон + номинал к погашению");
  await payCoupons(round1, "№2", coupon1);
  await thawAll(round1);

  // ---------- 6. Погашение ----------
  for (let i = 0; i < holders.length; i++) {
    await step(`Погашение: ${names[i]}`, `${balances[i]} × $750 = $${balances[i] * 750}, облигации сожжены`, () =>
      program.methods
        .redeem()
        .accountsPartial({
          holder: holders[i].publicKey,
          bondSeries: bond,
          bondMint: mint,
          holderTokenAccount: bondAta[i],
          holderPaymentAccount: usdcAta[i],
          vault,
          receipt: receiptPda(pid, "redeem", bond, holders[i].publicKey),
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([holders[i]])
        .rpc()
    );
  }
  // ==== END PART 3 ====

  // ---------- Итог ----------
  const b: any = await retry(() => program.account.bondSeries.fetch(bond));
  console.log("---------- Итог ----------");
  console.log("Статус выпуска:", Object.keys(b.status)[0]);
  console.log("Номинал сейчас: $" + b.faceValue.toNumber() / USDC);
  for (let i = 0; i < holders.length; i++) {
    const bal = await retry(() => connection.getTokenAccountBalance(usdcAta[i]));
    console.log(`${names[i]} получил всего: $${bal.value.uiAmountString}`);
  }

  const meta = {
    cluster: "devnet",
    programId: pid.toBase58(),
    issuer: me.toBase58(),
    seriesId,
    bondSeries: bond.toBase58(),
    bondMint: mint.toBase58(),
    vault: vault.toBase58(),
    testUsdcMint: usdc.toBase58(),
    holders: names.map((n, i) => ({
      name: n,
      wallet: holders[i].publicKey.toBase58(),
      bondAccount: bondAta[i].toBase58(),
      usdcAccount: usdcAta[i].toBase58(),
    })),
    steps: log,
  };
  fs.writeFileSync("onchain-log.json", JSON.stringify(meta, null, 2));
  fs.writeFileSync(
    "demo-wallets.json",
    JSON.stringify(names.map((n, i) => ({ name: n, secretKey: Array.from(holders[i].secretKey) })))
  );
  fs.mkdirSync("../docs", { recursive: true });
  const rows = log
    .map((e) => `| ${e.n} | ${e.title} | ${e.detail} | [${e.signature.slice(0, 8)}…](${e.url}) |`)
    .join("\n");
  fs.writeFileSync(
    "../docs/onchain-log.md",
    `# On-chain журнал демо-сценария (devnet)\n\nProgram: \`${pid.toBase58()}\`\n\nBond series: \`${bond.toBase58()}\`\n\n| # | Действие | Детали | Транзакция |\n|---|---|---|---|\n${rows}\n`
  );
  console.log("Готово: onchain-log.json и docs/onchain-log.md сохранены.");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
