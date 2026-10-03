import * as anchor from "@anchor-lang/core";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import fs from "fs";
import os from "os";

export const DAY = 86_400;
export const USDC = 1_000_000; // 6 знаков, как у настоящего USDC

export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

// Публичный devnet иногда отвечает 429: тогда ждём и пробуем снова
export async function retry<T>(fn: () => Promise<T>): Promise<T> {
  for (let i = 0; ; i++) {
    try {
      return await fn();
    } catch (e: any) {
      const msg = String(e?.message ?? e);
      if (i < 6 && (msg.includes("429") || msg.includes("Too Many Requests"))) {
        await sleep(1500 * (i + 1));
        continue;
      }
      throw e;
    }
  }
}

export function setup() {
  const idl = JSON.parse(
    fs.readFileSync("../target/idl/kase_corporate_actions.json", "utf8")
  );
  const raw = JSON.parse(
    fs.readFileSync(os.homedir() + "/.config/solana/id.json", "utf8")
  );
  const issuer = Keypair.fromSecretKey(Uint8Array.from(raw));
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(issuer), {
    commitment: "confirmed",
  });
  const program = new anchor.Program(idl, provider);
  return { program, provider, connection, issuer };
}

const u64le = (n: number) => new anchor.BN(n).toArrayLike(Buffer, "le", 8);
const u32le = (n: number) => new anchor.BN(n).toArrayLike(Buffer, "le", 4);

export function pdas(programId: PublicKey, authority: PublicKey, seriesId: number) {
  const bond = PublicKey.findProgramAddressSync(
    [Buffer.from("bond"), authority.toBuffer(), u64le(seriesId)],
    programId
  )[0];
  const mint = PublicKey.findProgramAddressSync(
    [Buffer.from("bond_mint"), bond.toBuffer()],
    programId
  )[0];
  const vault = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), bond.toBuffer()],
    programId
  )[0];
  return { bond, mint, vault };
}

export function roundPda(programId: PublicKey, bond: PublicKey, idx: number) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("round"), bond.toBuffer(), u32le(idx)],
    programId
  )[0];
}

// Квитанции: "receipt" и "partial" (период + счёт облигаций), "redeem" (выпуск + холдер)
export function receiptPda(programId: PublicKey, seed: string, a: PublicKey, b: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(seed), a.toBuffer(), b.toBuffer()],
    programId
  )[0];
}

export type LogEntry = {
  n: number;
  title: string;
  detail: string;
  signature: string;
  url: string;
};
export const log: LogEntry[] = [];

export async function step(title: string, detail: string, fn: () => Promise<string>) {
  const signature = await retry(fn);
  const url = `https://explorer.solana.com/tx/${signature}?cluster=devnet`;
  const entry = { n: log.length + 1, title, detail, signature, url };
  log.push(entry);
  console.log(`${String(entry.n).padStart(2)}. ${title}: ${detail}\n    ${url}`);
  await sleep(500);
  return signature;
}
