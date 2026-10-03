import { useEffect, useState } from "react";
import * as anchor from "@anchor-lang/core";
import { Connection, PublicKey } from "@solana/web3.js";
import { Buffer } from "buffer";
import idl from "./idl.json";
import log from "./onchain-log.json";
import Investor from "./Investor";

const USDC = 1_000_000;
const money = (n: any) =>
  "$" + (Number(n.toString()) / USDC).toLocaleString("en-US", { maximumFractionDigits: 2 });
const day = (ts: any) => new Date(Number(ts.toString()) * 1000).toISOString().slice(0, 10);
const short = (s: string) => s.slice(0, 4) + "…" + s.slice(-4);
const addrUrl = (s: string) => `https://explorer.solana.com/address/${s}?cluster=devnet`;
const u32le = (n: number) => {
  const b = Buffer.alloc(4);
  b.writeUInt32LE(n);
  return b;
};

type Data = { bond: any; rounds: any[]; redeemed: any[]; bal: any[] };

async function load(): Promise<Data> {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  // Кошелёк-пустышка: страница только читает данные и ничего не подписывает
  const wallet: any = {
    publicKey: PublicKey.default,
    signTransaction: async (t: any) => t,
    signAllTransactions: async (t: any) => t,
  };
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  const program: any = new anchor.Program(idl as any, provider);
  const programId = new PublicKey(log.programId);
  const pda = (seeds: Buffer[]) => PublicKey.findProgramAddressSync(seeds, programId)[0];
  const bondKey = new PublicKey(log.bondSeries);

  const bond = await program.account.bondSeries.fetch(bondKey);
  const rounds: any[] = [];
  for (let i = 0; i < bond.nextRound; i++) {
    const rk = pda([Buffer.from("round"), bondKey.toBuffer(), u32le(i)]);
    const round = await program.account.couponRound.fetch(rk);
    const keys = (seed: string) =>
      log.holders.map((h) =>
        pda([Buffer.from(seed), rk.toBuffer(), new PublicKey(h.bondAccount).toBuffer()])
      );
    const coupons = await program.account.payoutReceipt.fetchMultiple(keys("receipt"));
    const partials = await program.account.payoutReceipt.fetchMultiple(keys("partial"));
    rounds.push({ i, round, coupons, partials });
  }
  const redeemed = await program.account.redemptionReceipt.fetchMultiple(
    log.holders.map((h) =>
      pda([Buffer.from("redeem"), bondKey.toBuffer(), new PublicKey(h.wallet).toBuffer()])
    )
  );
  const bal: any[] = [];
  for (const h of log.holders) {
    const b = await connection.getTokenAccountBalance(new PublicKey(h.bondAccount));
    const u = await connection.getTokenAccountBalance(new PublicKey(h.usdcAccount));
    bal.push({ bonds: b.value.uiAmountString, usdc: u.value.uiAmountString });
  }
  return { bond, rounds, redeemed, bal };
}
// ==== END PART 1 ====

const css = `
body{margin:0;background:#f5f6f8;color:#14161a;font-family:system-ui,-apple-system,Segoe UI,Roboto,sans-serif}
.wrap{max-width:1100px;margin:0 auto;padding:24px 16px 64px}
h1{margin:0 0 4px;font-size:26px} h2{margin:32px 0 10px;font-size:18px}
.sub{color:#5b6270;font-size:14px;margin-bottom:8px}
.card{background:#fff;border:1px solid #e3e6ec;border-radius:10px;padding:16px;margin-bottom:12px;overflow-x:auto}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:12px}
.k{font-size:12px;color:#5b6270;text-transform:uppercase;letter-spacing:.04em}
.v{font-size:20px;font-weight:600;margin-top:2px}
table{border-collapse:collapse;width:100%;font-size:14px}
th,td{text-align:left;padding:7px 10px;border-bottom:1px solid #eceef2;white-space:nowrap}
th{font-size:12px;color:#5b6270;text-transform:uppercase;letter-spacing:.04em}
a{color:#2457d6;text-decoration:none} a:hover{text-decoration:underline}
.badge{display:inline-block;padding:2px 10px;border-radius:99px;font-size:13px;font-weight:600;background:#e6f4ea;color:#17693a}
.tl{list-style:none;padding:0;margin:0}
.tl li{display:flex;gap:10px;padding:6px 0;border-bottom:1px solid #eceef2;font-size:14px}
.tl .n{width:28px;color:#8a91a0}
.two{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:12px}
ul.plain{margin:6px 0 0;padding-left:18px;font-size:14px;line-height:1.6}
`;

function Table({ head, rows }: { head: string[]; rows: any[][] }) {
  return (
    <table>
      <thead>
        <tr>{head.map((h, i) => <th key={i}>{h}</th>)}</tr>
      </thead>
      <tbody>
        {rows.map((r, i) => (
          <tr key={i}>{r.map((c, j) => <td key={j}>{c}</td>)}</tr>
        ))}
      </tbody>
    </table>
  );
}

const Stat = ({ k, v }: { k: string; v: any }) => (
  <div>
    <div className="k">{k}</div>
    <div className="v">{v}</div>
  </div>
);

export default function App() {
  const [d, setD] = useState<Data | null>(null);
  const [err, setErr] = useState("");
  useEffect(() => {
    load().then(setD).catch((e) => setErr(String(e?.message ?? e)));
  }, []);

  const holders = log.holders;
  const b = d?.bond;

  return (
    <div className="wrap">
      <style>{css}</style>
      <h1>Corporate Actions on Solana</h1>
      <div className="sub">
        Tokenized bond lifecycle · devnet · program{" "}
        <a href={addrUrl(log.programId)} target="_blank">{short(log.programId)}</a> · bond series{" "}
        <a href={addrUrl(log.bondSeries)} target="_blank">{short(log.bondSeries)}</a>
      </div>
      <Investor />
      {err && <div className="card" style={{ color: "crimson" }}>{err}</div>}
      {!d && !err && <div className="card">Loading from devnet…</div>}

      {d && (
        <>
          <h2>Instrument</h2>
          <div className="card grid">
            <Stat k="Status" v={<span className="badge">{Object.keys(b.status)[0]}</span>} />
            <Stat k="Face value now" v={money(b.faceValue)} />
            <Stat k="Coupon" v={`${b.couponRateBps / 100}% × ${b.couponsPerYear}/yr`} />
            <Stat k="Maturity" v={day(b.maturityTs)} />
            <Stat k="Demo clock offset" v={`+${Math.round(Number(b.timeOffset.toString()) / 86400)} days`} />
            <Stat k="Coupon rounds" v={b.nextRound} />
          </div>

          <h2>Holder registry (balance on each record date)</h2>
          <div className="card">
            <Table
              head={[
                "Holder",
                "Wallet",
                ...d.rounds.map((r) => `Record date #${r.i + 1} (${day(r.round.recordTs)})`),
                "Bonds now",
                "USDC received",
              ]}
              rows={holders.map((h, k) => [
                h.name,
                <a href={addrUrl(h.wallet)} target="_blank">{short(h.wallet)}</a>,
                ...d.rounds.map((r) => (r.coupons[k] ? r.coupons[k].bonds.toString() : "—")),
                d.bal[k].bonds,
                "$" + Number(d.bal[k].usdc).toLocaleString("en-US"),
              ])}
            />
          </div>

          <h2>Corporate actions</h2>
          {d.rounds.map((r) => (
            <div className="card grid" key={r.i}>
              <Stat k={`Round #${r.i + 1} record date`} v={day(r.round.recordTs)} />
              <Stat k="Coupon / bond" v={money(r.round.couponPerBond)} />
              <Stat k="Snapshot supply" v={r.round.snapshotSupply.toString()} />
              <Stat k="Total due" v={money(r.round.totalDue)} />
              <Stat k="Paid" v={`${money(r.round.paidTotal)} (${r.round.holdersPaid} holders)`} />
              <Stat
                k="Partial redemption"
                v={
                  r.round.partialBps > 0
                    ? `${r.round.partialBps / 100}% · ${money(r.round.partialPerBond)}/bond${r.round.partialApplied ? " · applied" : ""}`
                    : "—"
                }
              />
            </div>
          ))}

          <h2>Payout receipts (on-chain)</h2>
          <div className="card">
            <Table
              head={["Round", "Type", "Holder", "Bonds", "Amount", "Date"]}
              rows={d.rounds.flatMap((r) =>
                holders.flatMap((h, k) =>
                  [
                    { t: "Coupon", x: r.coupons[k] },
                    { t: "Partial redemption", x: r.partials[k] },
                  ]
                    .filter((e) => e.x)
                    .map((e) => [
                      `#${r.i + 1}`,
                      e.t,
                      h.name,
                      e.x.bonds.toString(),
                      money(e.x.amount),
                      day(e.x.paidTs),
                    ])
                )
              )}
            />
          </div>

          <h2>Redemptions (bonds burned)</h2>
          <div className="card">
            <Table
              head={["Holder", "Bonds burned", "Principal paid", "Date"]}
              rows={holders.map((h, k) => {
                const x = d.redeemed[k];
                return x
                  ? [h.name, x.bondsRedeemed.toString(), money(x.amountPaid), day(x.lastTs)]
                  : [h.name, "—", "—", "—"];
              })}
            />
          </div>
        </>
      )}

      <h2>Transaction timeline</h2>
      <div className="card">
        <ul className="tl">
          {log.steps.map((s: any) => (
            <li key={s.n}>
              <span className="n">{s.n}</span>
              <span>
                <b>{s.title}</b> · {s.detail}{" "}
                <a href={s.url} target="_blank">tx ↗</a>
              </span>
            </li>
          ))}
        </ul>
      </div>

      <h2>Implemented vs simulated</h2>
      <div className="two">
        <div className="card">
          <b>Implemented (on Solana)</b>
          <ul className="plain">
            <li>Tokenized bond (SPL token, 1 token = 1 bond)</li>
            <li>Holder registry via freeze at record date, completeness enforced on-chain</li>
            <li>Entitlement math in integers (bps), computed by the program</li>
            <li>Coupon payments from a program-owned vault, one receipt per holder per period</li>
            <li>Partial redemption (pro-rata) that lowers face value</li>
            <li>Final redemption with token burn and status change</li>
          </ul>
        </div>
        <div className="card">
          <b>Simulated</b>
          <ul className="plain">
            <li>Fiat settlement: payments use a test USDC token</li>
            <li>KASE / depository integration and real KYC</li>
            <li>Date oracle: the issuer advances a demo clock</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
// ==== END PART 2 ====
