import { useEffect, useState } from "react";
import * as anchor from "@anchor-lang/core";
import { Connection, PublicKey, SystemProgram } from "@solana/web3.js";
import { Buffer } from "buffer";
import idl from "./idl.json";
import live from "./live-log.json";

const TOKEN = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const ATA_PROGRAM = new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const USDC = 1_000_000;

const ata = (owner: PublicKey, mint: PublicKey) =>
  PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN.toBuffer(), mint.toBuffer()],
    ATA_PROGRAM
  )[0];
const usd = (n: number) =>
  "$" + (n / USDC).toLocaleString("en-US", { maximumFractionDigits: 2 });

const connection = new Connection("https://api.devnet.solana.com", "confirmed");
// Read-only provider: the real signing is done by Phantom
const dummy: any = {
  publicKey: PublicKey.default,
  signTransaction: async (t: any) => t,
  signAllTransactions: async (t: any) => t,
};
const provider = new anchor.AnchorProvider(connection, dummy, { commitment: "confirmed" });
const program: any = new anchor.Program(idl as any, provider);

const phantom = () => (window as any).phantom?.solana ?? (window as any).solana;

async function balanceOf(addr: PublicKey): Promise<number> {
  try {
    const r = await connection.getTokenAccountBalance(addr);
    return Number(r.value.amount);
  } catch {
    return 0;
  }
}

const btn: any = {
  background: "#2457d6",
  color: "#fff",
  border: 0,
  borderRadius: 8,
  padding: "10px 18px",
  fontSize: 15,
  fontWeight: 600,
  cursor: "pointer",
};

export default function Investor() {
  const [addr, setAddr] = useState<string | null>(null);
  const [st, setSt] = useState<any>(null);
  const [msg, setMsg] = useState("");
  const [busy, setBusy] = useState(false);
  const [tx, setTx] = useState("");

  const bondKey = new PublicKey(live.bondSeries);
  const mint = new PublicKey(live.bondMint);
  const payMint = new PublicKey(live.testUsdcMint);

  async function refresh(a: string) {
    const user = new PublicKey(a);
    const b = await program.account.bondSeries.fetch(bondKey);
    const bonds = await balanceOf(ata(user, mint));
    const usdc = await balanceOf(ata(user, payMint));
    const now = Math.floor(Date.now() / 1000) + Number(b.timeOffset.toString());
    setSt({
      bonds,
      usdc,
      face: Number(b.faceValue.toString()),
      matured: now >= Number(b.maturityTs.toString()),
      status: Object.keys(b.status)[0],
    });
  }

  async function connect() {
    setMsg("");
    const p = phantom();
    if (!p) {
      setMsg("Phantom wallet not found. Install it from phantom.com and reload this page.");
      return;
    }
    try {
      const r = await p.connect();
      const a = r.publicKey.toString();
      setAddr(a);
      await refresh(a);
    } catch (e: any) {
      setMsg(String(e?.message ?? e));
    }
  }

  useEffect(() => {
    const p = phantom();
    p?.connect?.({ onlyIfTrusted: true })
      .then((r: any) => {
        const a = r.publicKey.toString();
        setAddr(a);
        return refresh(a);
      })
      .catch(() => {});
  }, []);

  async function redeem() {
    if (!addr) return;
    setBusy(true);
    setMsg("");
    setTx("");
    try {
      const p = phantom();
      const user = new PublicKey(addr);
      const t = await program.methods
        .redeem()
        .accountsPartial({
          holder: user,
          bondSeries: bondKey,
          bondMint: mint,
          holderTokenAccount: ata(user, mint),
          holderPaymentAccount: ata(user, payMint),
          vault: new PublicKey(live.vault),
          receipt: PublicKey.findProgramAddressSync(
            [Buffer.from("redeem"), bondKey.toBuffer(), user.toBuffer()],
            new PublicKey(live.programId)
          )[0],
          tokenProgram: TOKEN,
          systemProgram: SystemProgram.programId,
        })
        .transaction();
      const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
      t.feePayer = user;
      t.recentBlockhash = blockhash;
      const signed = await p.signTransaction(t);
      const sig = await connection.sendRawTransaction(signed.serialize());
      await connection.confirmTransaction(
        { signature: sig, blockhash, lastValidBlockHeight },
        "confirmed"
      );
      setTx(sig);
      await refresh(addr);
    } catch (e: any) {
      setMsg(String(e?.message ?? e));
    }
    setBusy(false);
  }

  const Stat = ({ k, v }: { k: string; v: any }) => (
    <div>
      <div className="k">{k}</div>
      <div className="v">{v}</div>
    </div>
  );

  return (
    <div className="card" style={{ borderColor: "#2457d6", borderWidth: 2 }}>
      <b>Investor view · live series</b>
      <div className="sub">
        Connect Phantom (devnet) to see your bonds and redeem them. The redemption is signed by you,
        the holder.
      </div>

      {!addr && (
        <button style={btn} onClick={connect}>
          Connect Phantom
        </button>
      )}

      {addr && st && (
        <>
          <div className="sub">
            Wallet:{" "}
            <a href={`https://explorer.solana.com/address/${addr}?cluster=devnet`} target="_blank">
              {addr.slice(0, 4)}…{addr.slice(-4)}
            </a>
          </div>
          <div className="grid" style={{ margin: "10px 0" }}>
            <Stat k="Bonds held" v={st.bonds} />
            <Stat k="Face value now" v={usd(st.face)} />
            <Stat k="Redemption value" v={usd(st.bonds * st.face)} />
            <Stat k="Test USDC balance" v={usd(st.usdc)} />
            <Stat k="Series status" v={st.status} />
          </div>

          {st.bonds > 0 && st.matured && (
            <button style={{ ...btn, opacity: busy ? 0.6 : 1 }} disabled={busy} onClick={redeem}>
              {busy ? "Waiting for wallet…" : `Redeem ${st.bonds} bonds for ${usd(st.bonds * st.face)}`}
            </button>
          )}
          {st.bonds > 0 && !st.matured && <div>Bonds have not reached maturity yet.</div>}
          {st.bonds === 0 && (
            <div>This wallet holds no bonds of the live series (not a holder, or already redeemed).</div>
          )}
        </>
      )}

      {tx && (
        <div style={{ marginTop: 10 }}>
          Redeemed.{" "}
          <a href={`https://explorer.solana.com/tx/${tx}?cluster=devnet`} target="_blank">
            View transaction ↗
          </a>
        </div>
      )}
      {msg && <pre style={{ color: "crimson", whiteSpace: "pre-wrap" }}>{msg}</pre>}
    </div>
  );
}
