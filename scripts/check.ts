import * as anchor from "@anchor-lang/core";
import { Connection, Keypair } from "@solana/web3.js";
import fs from "fs";
import os from "os";

async function main() {
  const idl = JSON.parse(
    fs.readFileSync("../target/idl/kase_corporate_actions.json", "utf8")
  );
  const secret = JSON.parse(
    fs.readFileSync(os.homedir() + "/.config/solana/id.json", "utf8")
  );
  const wallet = new anchor.Wallet(Keypair.fromSecretKey(Uint8Array.from(secret)));
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  const program = new anchor.Program(idl, provider);

  console.log("Program:", program.programId.toBase58());
  console.log(
    "Instructions:",
    program.idl.instructions.map((i: any) => i.name).join(", ")
  );
  const lamports = await connection.getBalance(wallet.publicKey);
  console.log("Wallet:", wallet.publicKey.toBase58(), lamports / 1e9, "SOL");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
