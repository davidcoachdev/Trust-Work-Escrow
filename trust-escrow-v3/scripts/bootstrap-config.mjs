import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import idl from "../target/idl/escrow.json" with { type: "json" };

const endpoint = process.env.ANCHOR_PROVIDER_URL || "http://127.0.0.1:8899";
const parsedEndpoint = new URL(endpoint);
if (parsedEndpoint.protocol !== "http:" || parsedEndpoint.hostname !== "127.0.0.1") {
  throw new Error(`Refusing non-localnet endpoint: ${endpoint}`);
}

const required = ["TRUST_ESCROW_V3_ADVISOR_PUBKEY", "TRUST_ESCROW_V3_TREASURY_PUBKEY", "TRUST_ESCROW_V3_ARBITRATION_TREASURY_PUBKEY"];
for (const name of required) if (!process.env[name]) throw new Error(`Missing ${name}; provide public identity only`);
const advisor = new PublicKey(process.env.TRUST_ESCROW_V3_ADVISOR_PUBKEY);
const treasury = new PublicKey(process.env.TRUST_ESCROW_V3_TREASURY_PUBKEY);
const arbitrationTreasury = new PublicKey(process.env.TRUST_ESCROW_V3_ARBITRATION_TREASURY_PUBKEY);
const feeBps = Number(process.env.TRUST_ESCROW_V3_FEE_BPS || "250");
if (!Number.isInteger(feeBps) || feeBps < 0 || feeBps > 10_000) throw new Error("Invalid fee bps");
if (treasury.equals(arbitrationTreasury)) throw new Error("Treasuries must be distinct");

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = new anchor.Program(idl, provider);
const [config] = PublicKey.findProgramAddressSync([Buffer.from("config")], program.programId);
const current = await program.account.config.fetchNullable(config);
if (current) {
  const matches = current.authority.equals(provider.wallet.publicKey)
    && current.advisor.equals(advisor)
    && current.treasury.equals(treasury)
    && current.arbitrationTreasury.equals(arbitrationTreasury)
    && current.feeBps === feeBps;
  if (!matches) throw new Error("Existing Config does not match approved manifest");
  console.log(JSON.stringify({ action: "verified", config: config.toBase58(), authority: current.authority.toBase58() }));
} else {
  await program.methods.initializeConfig(advisor, treasury, arbitrationTreasury, feeBps)
    .accounts({ authority: provider.wallet.publicKey, treasury, arbitrationTreasury, config, systemProgram: anchor.web3.SystemProgram.programId })
    .rpc();
  console.log(JSON.stringify({ action: "initialized", config: config.toBase58(), authority: provider.wallet.publicKey.toBase58() }));
}
